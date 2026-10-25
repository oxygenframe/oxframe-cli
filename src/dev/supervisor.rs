//! Builds the project, runs `<bin> preview --port 0`, connects to it and
//! restarts it on file changes or crashes.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use notify::{EventKind, RecursiveMode, Watcher};
use oxframe_protocol::{Command, ErrorKind, Event, PROTOCOL_VERSION};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command as Proc};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use tokio_tungstenite::tungstenite::Message as WsMessage;

use super::hub::{Hub, Msg};
use crate::{cargo, exit};

const MAX_RESTARTS: u32 = 3;

pub struct Supervisor {
    pub root: PathBuf,
    pub release: bool,
    pub bin: Option<String>,
    pub cpu: bool,
    /// Keep the project binary's stdout off the terminal, so `--json` stays one object.
    pub json: bool,
    pub hub: Arc<Hub>,
}

struct Running {
    child: Child,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
    engine_task: tokio::task::JoinHandle<()>,
}

impl Supervisor {
    pub async fn run(self) -> u8 {
        let mut changes = match watch(&self.root) {
            Ok(rx) => rx,
            Err(e) => {
                crate::error(format!("cannot watch files: {e}"));
                return exit::ENV;
            }
        };
        let mut exe: Option<PathBuf> = None;
        let mut running: Option<Running> = None;
        let mut restarts = 0;
        let mut first = true;
        let mut rebuild = true;

        loop {
            if rebuild {
                rebuild = false;
                if let Some(new_exe) = self.build().await {
                    if let Some(old) = running.take() {
                        self.stop(old).await;
                    }
                    exe = Some(new_exe);
                    restarts = 0;
                    running = self.start(exe.as_deref().expect("set"), !first).await;
                    first = false;
                }
            }
            // Wait for a change or for the process to exit.
            let exited = async {
                match running.as_mut() {
                    Some(r) => r.child.wait().await.ok(),
                    None => std::future::pending().await,
                }
            };
            tokio::select! {
                change = changes.recv() => {
                    if change.is_none() {
                        return exit::ENV;
                    }
                    // Collapse bursts of changes into one rebuild.
                    while changes.try_recv().is_ok() {}
                    rebuild = true;
                }
                status = exited => {
                    let r = running.take().expect("was running");
                    r.engine_task.abort();
                    self.hub.set_engine(None);
                    let tail: Vec<String> = r.stderr_tail.lock().expect("lock").iter().cloned().collect();
                    let status = status.map_or("unknown".into(), |s| s.to_string());
                    if restarts < MAX_RESTARTS {
                        restarts += 1;
                        crate::note(format!("project exited ({status}); restarting ({restarts}/{MAX_RESTARTS})"));
                        if let Some(exe) = exe.as_deref() {
                            running = self.start(exe, true).await;
                        }
                    } else {
                        crate::error("project keeps crashing; fix the code and save to retry");
                        self.hub.status(Some(&Event::Error {
                            kind: ErrorKind::Runtime,
                            message: format!("the project process exited ({status}):\n\n{}", tail.join("\n")),
                            spans: Vec::new(),
                        }));
                    }
                }
            }
        }
    }

    async fn build(&self) -> Option<PathBuf> {
        self.hub.status(Some(&Event::Building));
        crate::note("building…");
        let (root, release, bin) = (self.root.clone(), self.release, self.bin.clone());
        let started = std::time::Instant::now();
        let out = tokio::task::spawn_blocking(move || cargo::build(&root, release, bin.as_deref()))
            .await
            .expect("build task");
        match out {
            Ok(b) if b.success && b.executable.is_some() => {
                crate::note(format!("built in {:.1}s", started.elapsed().as_secs_f64()));
                self.hub.status(None);
                b.executable
            }
            Ok(b) => {
                let msg = if b.success {
                    "the package has no binary target to run".to_owned()
                } else {
                    b.messages
                };
                crate::error("build failed; the previous version keeps running");
                self.hub.build_error(msg, b.spans);
                None
            }
            Err(e) => {
                crate::error(format!("cannot run cargo: {e}"));
                self.hub
                    .build_error(format!("cannot run cargo: {e}"), Vec::new());
                None
            }
        }
    }

    async fn start(&self, exe: &Path, reload: bool) -> Option<Running> {
        let token = super::random_token();
        let mut cmd = Proc::new(exe);
        cmd.current_dir(&self.root)
            .args(["preview", "--port", "0"])
            .env("OXFRAME_PREVIEW_TOKEN", &token)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(path) = crate::ffmpeg::child_path() {
            cmd.env("PATH", path);
        }
        if self.cpu {
            cmd.arg("--cpu");
        }
        if let Ok(cfg) = crate::config::Config::load(&self.root) {
            cmd.args(cfg.theme.args());
        }
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.runtime_error(format!("cannot start {}: {e}", exe.display()));
                return None;
            }
        };

        // stderr → terminal, keeping a tail for crash reports.
        let tail = Arc::new(Mutex::new(VecDeque::new()));
        let stderr = child.stderr.take().expect("piped");
        let t = tail.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                anstream::eprintln!("{line}");
                let mut t = t.lock().expect("lock");
                if t.len() >= 40 {
                    t.pop_front();
                }
                t.push_back(line);
            }
        });

        // First stdout line is the port.
        let stdout = child.stdout.take().expect("piped");
        let mut lines = BufReader::new(stdout).lines();
        let port = match tokio::time::timeout(Duration::from_secs(60), lines.next_line()).await {
            Ok(Ok(Some(l))) => l.trim().parse::<u16>().ok(),
            _ => None,
        };
        let Some(port) = port else {
            let tail: Vec<String> = tail.lock().expect("lock").iter().cloned().collect();
            self.runtime_error(format!(
                "the project did not start its preview server. Does main() call `Project::main()`?\n\n{}",
                tail.join("\n")
            ));
            let _ = child.start_kill();
            return None;
        };
        let json = self.json;
        tokio::spawn(async move {
            while let Ok(Some(line)) = lines.next_line().await {
                if json {
                    anstream::eprintln!("{line}");
                } else {
                    anstream::println!("{line}");
                }
            }
        });

        let url = format!("ws://127.0.0.1:{port}/ws?token={token}");
        let mut conn = None;
        for _ in 0..50 {
            match tokio_tungstenite::connect_async(&url).await {
                Ok((ws, _)) => {
                    conn = Some(ws);
                    break;
                }
                Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
            }
        }
        let Some(ws) = conn else {
            self.runtime_error(format!("cannot connect to the project on port {port}"));
            let _ = child.start_kill();
            return None;
        };
        let (mut sink, mut stream) = ws.split();
        let (tx, mut rx) = unbounded_channel::<Msg>();
        self.hub.set_engine(Some(tx.clone()));
        if let Some(hello) = self.hub.hello() {
            let _ = tx.send(Msg::Text(hello));
        }
        let hub = self.hub.clone();
        let announce = Arc::new(AtomicBool::new(reload));
        let engine_task = tokio::spawn(async move {
            let writer = tokio::spawn(async move {
                while let Some(m) = rx.recv().await {
                    let msg = match m {
                        Msg::Text(t) => WsMessage::Text(t.into()),
                        Msg::Binary(b) => WsMessage::Binary(b.into()),
                    };
                    if sink.send(msg).await.is_err() {
                        break;
                    }
                }
            });
            while let Some(Ok(msg)) = stream.next().await {
                match msg {
                    WsMessage::Text(t) => {
                        let text = t.to_string();
                        if let Ok(Event::Ready {
                            protocol_version, ..
                        }) = serde_json::from_str::<Event>(&text)
                        {
                            if protocol_version != PROTOCOL_VERSION {
                                hub.status(Some(&Event::Error {
                                    kind: ErrorKind::Incompatible,
                                    message: format!(
                                        "the project's SDK speaks protocol v{protocol_version}, this oxframe-cli speaks v{PROTOCOL_VERSION}"
                                    ),
                                    spans: Vec::new(),
                                }));
                                continue;
                            }
                            hub.to_ui(Msg::Text(text));
                            if announce.swap(false, Ordering::SeqCst) {
                                hub.event(&Event::Reloaded);
                            }
                            continue;
                        }
                        hub.to_ui(Msg::Text(text));
                    }
                    WsMessage::Binary(b) => hub.to_ui(Msg::Binary(b.to_vec())),
                    WsMessage::Close(_) => break,
                    _ => {}
                }
            }
            writer.abort();
        });
        crate::note(format!("project running (preview port {port})"));
        Some(Running {
            child,
            stderr_tail: tail,
            engine_task,
        })
    }

    /// `Shutdown`, up to 2 s grace, then kill (the whole tree on Windows).
    async fn stop(&self, mut r: Running) {
        self.hub.forward_from_ui(Msg::Text(
            serde_json::to_string(&Command::Shutdown).expect("json"),
        ));
        self.hub.set_engine(None);
        if tokio::time::timeout(Duration::from_secs(2), r.child.wait())
            .await
            .is_err()
        {
            kill_tree(&mut r.child).await;
        }
        r.engine_task.abort();
    }

    fn runtime_error(&self, message: String) {
        crate::error(&message);
        self.hub.status(Some(&Event::Error {
            kind: ErrorKind::Runtime,
            message,
            spans: Vec::new(),
        }));
    }
}

async fn kill_tree(child: &mut Child) {
    #[cfg(windows)]
    if let Some(pid) = child.id() {
        // Kills the process and its children (e.g. a spawned ffmpeg).
        let _ = Proc::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .status()
            .await;
    }
    let _ = child.kill().await;
}

/// File changes in `src/`, `assets/` and `Cargo.toml`. Only content
/// changes count (cargo *reads* these files during the build, and access
/// events must not trigger another build). `target/` and `*.meta.json`
/// (written by the engine) are ignored.
fn watch(root: &Path) -> notify::Result<UnboundedReceiver<()>> {
    let (raw_tx, mut raw_rx) = unbounded_channel::<()>();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(ev) = res
            && matches!(
                ev.kind,
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
            )
            && !matches!(
                ev.kind,
                EventKind::Modify(notify::event::ModifyKind::Metadata(_))
            )
            && ev.paths.iter().any(|p| relevant(p))
        {
            let _ = raw_tx.send(());
        }
    })?;
    for dir in ["src", "assets"] {
        let p = root.join(dir);
        if p.is_dir() {
            watcher.watch(&p, RecursiveMode::Recursive)?;
        }
    }
    watcher.watch(&root.join("Cargo.toml"), RecursiveMode::NonRecursive)?;

    // Debounce: one notification per burst, 200 ms after the last event.
    let (tx, rx) = unbounded_channel();
    tokio::spawn(async move {
        let _watcher = watcher;
        while raw_rx.recv().await.is_some() {
            loop {
                match tokio::time::timeout(Duration::from_millis(200), raw_rx.recv()).await {
                    Ok(Some(())) => continue,
                    Ok(None) => return,
                    Err(_) => break,
                }
            }
            if tx.send(()).is_err() {
                return;
            }
        }
    });
    Ok(rx)
}

fn relevant(path: &Path) -> bool {
    let s = path.to_string_lossy();
    !(s.contains("/target/")
        || s.contains("\\target\\")
        || s.ends_with(".meta.json")
        || s.ends_with(".json.tmp")
        || s.ends_with('~'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_engine_written_files() {
        assert!(relevant(Path::new("/p/src/main.rs")));
        assert!(!relevant(Path::new("/p/src/intro.meta.json")));
        assert!(!relevant(Path::new("/p/src/intro.meta.json.tmp")));
        assert!(!relevant(Path::new("/p/target/debug/x")));
    }
}
