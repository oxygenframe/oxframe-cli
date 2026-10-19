//! `oxframe dev`: editor server + supervisor of the project process.
//!
//! ```text
//! browser ──WS──▶ oxframe dev (127.0.0.1:3000) ──WS──▶ project binary `preview --port 0`
//!                  │ serves editor-ui                   (restarted on every change)
//!                  └ watches src/, assets/, Cargo.toml → cargo build
//! ```
//!
//! The editor stays connected to the CLI while the project binary is
//! rebuilt or crashes; the CLI reports `Building` / `Error` itself and sends
//! `Reloaded` once a new binary answers `Ready`.

mod hub;
mod server;
mod supervisor;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::ui::UiSource;
use crate::{config, exit};

pub struct DevArgs {
    pub port: Option<u16>,
    pub open: bool,
    pub no_open: bool,
    pub release: bool,
    pub ui_dir: Option<PathBuf>,
    pub cpu: bool,
}

pub fn run(args: DevArgs) -> u8 {
    let root = match config::project_root() {
        Ok(r) => r,
        Err(e) => {
            crate::error(e);
            return exit::ENV;
        }
    };
    let cfg = match config::Config::load(&root) {
        Ok(c) => c,
        Err(e) => {
            crate::error(e);
            return exit::ENV;
        }
    };
    let port = args.port.unwrap_or(cfg.dev.port);
    let release = args.release || cfg.dev.profile == "release";
    let ui = match args.ui_dir {
        Some(d) => UiSource::Dir(d),
        None => UiSource::Embedded,
    };
    let token = editor_token(&root);

    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            crate::error(e);
            return exit::ENV;
        }
    };
    rt.block_on(async move {
        let listener = match tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            Ok(l) => l,
            Err(e) => {
                crate::error(format!(
                    "cannot listen on 127.0.0.1:{port}: {e} (another `oxframe dev`? use --port)"
                ));
                return exit::ENV;
            }
        };
        let port = listener.local_addr().map(|a| a.port()).unwrap_or(port);
        let hub = Arc::new(hub::Hub::default());
        let url = format!("http://127.0.0.1:{port}/?token={token}");
        crate::note(format!("editor at {url}"));
        if !args.no_open && (args.open || cfg.dev.open) {
            let _ = open::that_detached(&url);
        }
        let state = server::State {
            hub: hub.clone(),
            ui,
            token: token.clone(),
            port,
        };
        let sup = supervisor::Supervisor {
            root,
            release,
            bin: cfg.bin,
            cpu: args.cpu,
            hub,
        };
        tokio::select! {
            r = server::serve(listener, state) => {
                if let Err(e) = r {
                    crate::error(e);
                    return exit::ENV;
                }
                exit::OK
            }
            code = sup.run() => code,
            _ = tokio::signal::ctrl_c() => {
                crate::note("bye");
                exit::OK
            }
        }
    })
}

/// 128 random bits as hex.
pub(crate) fn random_token() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("OS random source");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The editor URL's token, kept per project in `target/oxframe/dev-token`
/// so an open tab reconnects after `oxframe dev` restarts. `cargo clean`
/// rotates it. Falls back to a one-off token if the file cannot be written.
fn editor_token(root: &Path) -> String {
    let target = std::env::var_os("CARGO_TARGET_DIR")
        .map(|d| root.join(d))
        .unwrap_or_else(|| root.join("target"));
    let path = target.join("oxframe").join("dev-token");
    if let Ok(t) = std::fs::read_to_string(&path) {
        let t = t.trim();
        if t.len() == 32 && t.bytes().all(|b| b.is_ascii_hexdigit()) {
            return t.to_owned();
        }
    }
    let token = random_token();
    if let Err(e) = write_private(&path, &token) {
        tracing::debug!("cannot save {}: {e}", path.display());
    }
    token
}

/// Writes a file only the current user can read.
fn write_private(path: &Path, contents: &str) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut opts, 0o600);
    opts.open(path)?.write_all(contents.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_token_is_stable_per_project() {
        let dir = tempfile::tempdir().unwrap();
        let a = editor_token(dir.path());
        assert_eq!(a.len(), 32);
        assert_eq!(editor_token(dir.path()), a, "same token after a restart");
        let file = dir.path().join("target/oxframe/dev-token");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&file).unwrap().permissions().mode();
            assert_eq!(mode & 0o077, 0, "token file is private");
        }
        std::fs::write(&file, "garbage").unwrap();
        assert_ne!(editor_token(dir.path()), a, "a corrupt file is replaced");
    }
}
