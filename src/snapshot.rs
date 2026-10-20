//! `oxframe snapshot`: build (dev profile) and write single frames as PNG —
//! the quickest way for people and agents to check a change visually.

use std::path::PathBuf;
use std::process::Command;

use serde_json::json;

use crate::reply::Done;
use crate::{cargo, config, exit};

pub struct SnapshotArgs {
    pub at: Vec<f64>,
    pub out: Option<PathBuf>,
    pub scene: Option<String>,
    pub size: Option<String>,
    pub release: bool,
    pub json: bool,
}

pub fn run(a: SnapshotArgs) -> Done {
    let root = match config::project_root() {
        Ok(r) => r,
        Err(e) => return crate::reply::fail(a.json, exit::ENV, e),
    };
    let cfg = match config::Config::load(&root) {
        Ok(c) => c,
        Err(e) => return crate::reply::fail(a.json, exit::ENV, e),
    };
    let build = match if a.json {
        cargo::build_quiet(&root, a.release, cfg.bin.as_deref())
    } else {
        cargo::build(&root, a.release, cfg.bin.as_deref())
    } {
        Ok(b) => b,
        Err(e) => return crate::reply::fail(a.json, exit::ENV, format!("cannot run cargo: {e}")),
    };
    let (true, Some(exe)) = (build.success, build.executable) else {
        if !a.json {
            crate::hints::show(&build.hints);
            crate::error("build failed");
        }
        return crate::reply::done(
            a.json,
            exit::BUILD,
            json!({
                "error": "build failed",
                "diagnostics": build.messages,
                "hints": build.hints,
            }),
        );
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
    cmd.args(cfg.theme.args());
    let (status, stdout, stderr) = match crate::reply::capture(&mut cmd, a.json) {
        Ok(o) => o,
        Err(e) => {
            return crate::reply::fail(
                a.json,
                exit::ENV,
                format!("cannot run the project binary: {e}"),
            );
        }
    };
    let hints = crate::hints::from_text(&stderr);
    if !a.json {
        crate::hints::show(&hints);
    }
    crate::reply::done(
        a.json,
        status,
        json!({ "stdout": stdout, "stderr": stderr, "hints": hints }),
    )
}
