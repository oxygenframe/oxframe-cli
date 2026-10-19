//! `oxframe snapshot`: build (dev profile) and write single frames as PNG —
//! the quickest way for people and agents to check a change visually.

use std::path::PathBuf;
use std::process::Command;

use crate::{cargo, config, exit};

pub struct SnapshotArgs {
    pub at: Vec<f64>,
    pub out: Option<PathBuf>,
    pub scene: Option<String>,
    pub size: Option<String>,
    pub release: bool,
}

pub fn run(a: SnapshotArgs) -> u8 {
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
    let build = match cargo::build(&root, a.release, cfg.bin.as_deref()) {
        Ok(b) => b,
        Err(e) => {
            crate::error(format!("cannot run cargo: {e}"));
            return exit::ENV;
        }
    };
    let (true, Some(exe)) = (build.success, build.executable) else {
        crate::error("build failed");
        return exit::BUILD;
    };
    let mut cmd = Command::new(exe);
    // CPU rendering: identical output everywhere, no GPU needed.
    cmd.current_dir(&root).args(["render", "--cpu"]);
    for t in &a.at {
        cmd.args(["--at", &t.to_string()]);
    }
    if let Some(o) = &a.out {
        cmd.arg("--out").arg(o);
    }
    if let Some(s) = &a.scene {
        cmd.args(["--scene", s]);
    }
    cmd.args(["--size", a.size.as_deref().unwrap_or("960x540")]);
    match cmd.status() {
        Ok(s) => s.code().map_or(exit::BUILD, |c| c.clamp(0, 255) as u8),
        Err(e) => {
            crate::error(format!("cannot run the project binary: {e}"));
            exit::ENV
        }
    }
}
