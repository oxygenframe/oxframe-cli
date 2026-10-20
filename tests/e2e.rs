//! End-to-end: `oxframe init` → `oxframe dev` → editor protocol over WS →
//! hot reload → build error overlay → `oxframe render --cpu`.
//!
//! Slow (compiles a project), so ignored by default:
//! `cargo test --test e2e -- --ignored --nocapture`.
//! Needs the oxframe-sdk checkout next to this repo (or OXFRAME_SDK_PATH).

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use oxframe_protocol::{Command as Cmd, ErrorKind, Event, FrameHeader, PROTOCOL_VERSION};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

fn sdk_path() -> PathBuf {
    std::env::var_os("OXFRAME_SDK_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../oxframe-sdk"))
}

fn oxframe() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_oxframe"));
    // Share compiled dependencies between runs.
    c.env(
        "CARGO_TARGET_DIR",
        Path::new(env!("CARGO_MANIFEST_DIR")).join("target/e2e-projects"),
    );
    c
}

struct Dev(Child);
impl Drop for Dev {
    fn drop(&mut self) {
        let _ = self.0.kill();
    }
}

type Ws =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

async fn send(ws: &mut Ws, cmd: &Cmd) {
    ws.send(Message::Text(serde_json::to_string(cmd).unwrap().into()))
        .await
        .unwrap();
}

/// Next event matching `pred`, skipping others (frames are returned as
/// `None` events when `want_frame`).
async fn wait_for(ws: &mut Ws, timeout: Duration, mut pred: impl FnMut(&Event) -> bool) -> Event {
    let deadline = Instant::now() + timeout;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let msg = tokio::time::timeout(left, ws.next())
            .await
            .expect("timed out waiting for event");
        if let Some(Ok(Message::Text(t))) = msg {
            let e: Event = serde_json::from_str(&t).unwrap();
            // A build error will not turn into the awaited event: fail now
            // with the compiler's message instead of at the timeout.
            if let Event::Error {
                kind: ErrorKind::Build,
                message,
                ..
            } = &e
                && !pred(&e)
            {
                panic!("the project failed to build:\n{message}");
            }
            if pred(&e) {
                return e;
            }
        }
    }
}

async fn wait_frame(ws: &mut Ws, timeout: Duration) -> FrameHeader {
    let deadline = Instant::now() + timeout;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if let Some(Ok(Message::Binary(b))) = tokio::time::timeout(left, ws.next())
            .await
            .expect("timed out waiting for frame")
        {
            return FrameHeader::decode(&b).unwrap();
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "slow: builds a project"]
async fn init_dev_reload_render() {
    let tmp = tempfile::tempdir().unwrap();
    let st = oxframe()
        .current_dir(tmp.path())
        .args(["init", "demo"])
        .arg("--sdk-path")
        .arg(sdk_path())
        .status()
        .unwrap();
    assert!(st.success());
    let dir = tmp.path().join("demo");

    // --- oxframe dev ---------------------------------------------------
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let mut child = oxframe()
        .current_dir(&dir)
        // Never open a browser from tests: an opened tab is a second editor.
        .args(["dev", "--cpu", "--no-open", "--port", &port.to_string()])
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stderr = child.stderr.take().unwrap();
    let _dev = Dev(child);
    let (url_tx, url_rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            eprintln!("[dev] {line}");
            if let Some(i) = line.find("http://127.0.0.1:") {
                let _ = url_tx.send(line[i..].trim().to_owned());
            }
        }
    });
    let url = url_rx
        .recv_timeout(Duration::from_secs(30))
        .expect("dev prints its URL");
    let token = url.split("token=").nth(1).unwrap().to_owned();

    // Wrong origin is rejected.
    let mut bad = format!("ws://127.0.0.1:{port}/ws?token={token}")
        .into_client_request()
        .unwrap();
    bad.headers_mut()
        .insert("origin", "http://evil.example".parse().unwrap());
    assert!(tokio_tungstenite::connect_async(bad).await.is_err());

    // A stale token gets close code 4401, which tells the editor to stop.
    let mut stale = format!("ws://127.0.0.1:{port}/ws?token=stale")
        .into_client_request()
        .unwrap();
    stale.headers_mut().insert(
        "origin",
        format!("http://127.0.0.1:{port}").parse().unwrap(),
    );
    let (mut s, _) = tokio_tungstenite::connect_async(stale).await.unwrap();
    match s.next().await {
        Some(Ok(Message::Close(Some(f)))) => assert_eq!(u16::from(f.code), 4401),
        other => panic!("expected close 4401, got {other:?}"),
    }
    assert_eq!(
        std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("target/e2e-projects/oxframe/dev-token")
        )
        .unwrap(),
        token,
        "the token is saved for the next run"
    );

    let mut req = format!("ws://127.0.0.1:{port}/ws?token={token}")
        .into_client_request()
        .unwrap();
    req.headers_mut().insert(
        "origin",
        format!("http://127.0.0.1:{port}").parse().unwrap(),
    );
    let (mut ws, _) = tokio_tungstenite::connect_async(req).await.unwrap();
    send(
        &mut ws,
        &Cmd::Hello {
            protocol_version: PROTOCOL_VERSION,
            ui_version: "e2e".into(),
        },
    )
    .await;
    let ready = wait_for(&mut ws, Duration::from_secs(900), |e| {
        matches!(e, Event::Ready { .. })
    })
    .await;
    let Event::Ready { fps, size, .. } = ready else {
        unreachable!()
    };
    assert_eq!((fps, size), (60, [1920, 1080]));

    // Scrub without a UI: seeks answer with frames.
    send(
        &mut ws,
        &Cmd::SetViewport {
            w: 480,
            h: 270,
            dpr: 1.0,
        },
    )
    .await;
    for f in [10, 30, 50] {
        send(&mut ws, &Cmd::Seek { frame: f }).await;
    }
    let t = Instant::now();
    send(&mut ws, &Cmd::Seek { frame: 42 }).await;
    loop {
        let h = wait_frame(&mut ws, Duration::from_secs(10)).await;
        if h.frame == 42 {
            assert_eq!((h.width, h.height), (480, 270));
            break;
        }
    }
    eprintln!("seek → frame: {:?}", t.elapsed());

    // A marker moved by command is saved to meta.json.
    send(
        &mut ws,
        &Cmd::SetMarker {
            name: "hello".into(),
            time: 2.5,
            scene: None,
        },
    )
    .await;
    wait_for(&mut ws, Duration::from_secs(30), |e| {
        matches!(e, Event::MetaChanged { .. })
    })
    .await;
    let meta = std::fs::read_to_string(dir.join("src/intro.meta.json")).unwrap();
    assert!(meta.contains("\"hello\": 2.5"), "{meta}");

    // --- hot reload ---------------------------------------------------
    let main = dir.join("src/main.rs");
    let src = std::fs::read_to_string(&main).unwrap();
    let t = Instant::now();
    std::fs::write(&main, src.replace("#68abdf", "#2ecc71")).unwrap();
    wait_for(&mut ws, Duration::from_secs(120), |e| {
        matches!(e, Event::Building)
    })
    .await;
    wait_for(&mut ws, Duration::from_secs(300), |e| {
        matches!(e, Event::Reloaded)
    })
    .await;
    let reload = t.elapsed();
    eprintln!("hot reload: {reload:?}");

    // --- build error: overlay, old process keeps serving ----------------
    std::fs::write(&main, src.replace("all((", "all((oops,")).unwrap();
    let err = wait_for(&mut ws, Duration::from_secs(300), |e| {
        matches!(e, Event::Error { .. })
    })
    .await;
    let Event::Error {
        kind,
        message,
        spans,
    } = err
    else {
        unreachable!()
    };
    assert_eq!(kind, ErrorKind::Build);
    assert!(message.contains("oops"), "{message}");
    assert!(
        spans
            .iter()
            .any(|s| s.file.ends_with("main.rs") && s.line > 0)
    );
    send(&mut ws, &Cmd::Seek { frame: 5 }).await;
    assert_eq!(wait_frame(&mut ws, Duration::from_secs(10)).await.frame, 5);
    std::fs::write(&main, &src).unwrap();
    wait_for(&mut ws, Duration::from_secs(300), |e| {
        matches!(e, Event::Reloaded)
    })
    .await;
    drop(ws);
    drop(_dev);

    // --- render ---------------------------------------------------------
    let out = dir.join("out/test.mp4");
    let st = oxframe()
        .current_dir(&dir)
        .args([
            "render", "--cpu", "--range", "0..1", "--size", "640x360", "--out",
        ])
        .arg(&out)
        .status()
        .unwrap();
    assert!(st.success());
    assert!(std::fs::metadata(&out).unwrap().len() > 1000);
    eprintln!("hot reload took {reload:?} (target ≤ 3 s)");
}
