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

use std::path::PathBuf;
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
    let token = random_token();

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

fn random_token() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).expect("OS random source");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
