//! `oxframe check`: cargo check, meta validation, version agreement, visual lint.

use std::path::Path;
use std::process::Command;

use oxframe_protocol::{Meta, PROTOCOL_VERSION};

use crate::{cargo, config, exit, render, ui};

pub fn run(frames: bool) -> u8 {
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
    let mut code = exit::OK;

    crate::note("cargo check");
    match Command::new(cargo::cargo())
        .current_dir(&root)
        .arg("check")
        .status()
    {
        Ok(s) if s.success() => {}
        Ok(_) => return exit::BUILD,
        Err(e) => {
            crate::error(format!("cannot run cargo: {e}"));
            return exit::ENV;
        }
    }

    let mut metas = Vec::new();
    collect_meta(&root.join("src"), &mut metas);
    for path in &metas {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        match Meta::parse(&text) {
            Ok(_) => anstream::eprintln!(
                "  ok  {}",
                path.strip_prefix(&root).unwrap_or(path).display()
            ),
            Err(e) => {
                crate::error(format!("{}: {e}", path.display()));
                code = exit::BUILD;
            }
        }
    }

    // Versions: CLI, the SDK the project links (asked from its binary) and
    // the embedded editor must speak the same protocol.
    crate::note("checking protocol versions");
    let build = match cargo::build(&root, false, cfg.bin.as_deref()) {
        Ok(b) if b.success => b,
        _ => return exit::BUILD,
    };
    let exe = build.executable.clone();
    let sdk = exe.as_ref().and_then(|exe| {
        let out = Command::new(exe)
            .current_dir(&root)
            .arg("info")
            .output()
            .ok()?;
        let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
        Some((
            v["protocol_version"].as_u64()? as u32,
            v["engine_version"].as_str()?.to_owned(),
        ))
    });
    let ui_v = ui::embedded_protocol_version();
    anstream::eprintln!(
        "  cli        protocol v{PROTOCOL_VERSION} (oxframe-cli {})",
        env!("CARGO_PKG_VERSION")
    );
    match &sdk {
        Some((p, v)) => anstream::eprintln!("  sdk        protocol v{p} (oxframe-sdk {v})"),
        None => anstream::eprintln!("  sdk        unknown (project binary has no `info`)"),
    }
    match ui_v {
        Some(p) => anstream::eprintln!(
            "  editor-ui  protocol v{p} (editor-ui {})",
            ui::embedded_version().unwrap_or_default()
        ),
        None => anstream::eprintln!("  editor-ui  debug client (no manifest)"),
    }
    let mismatch = sdk.as_ref().is_some_and(|(p, _)| *p != PROTOCOL_VERSION)
        || ui_v.is_some_and(|p| p != PROTOCOL_VERSION);
    if mismatch {
        crate::error(
            "protocol versions differ: update oxframe-cli and the oxframe SDK to matching releases",
        );
        return exit::VERSION;
    }

    if render::find_ffmpeg(cfg.render.ffmpeg.as_deref()).is_none() {
        anstream::eprintln!(
            "\nwarning: ffmpeg not found — `oxframe render` will only produce PNG sequences.\n{}",
            render::FFMPEG_HELP
        );
    }
    if code != exit::OK {
        return code;
    }
    let Some(exe) = exe else {
        crate::error("build produced no binary to visually check");
        return exit::BUILD;
    };
    crate::note("visual check");
    let mut cmd = Command::new(exe);
    cmd.current_dir(&root).arg("check");
    let c = &cfg.check;
    let interval = c.interval.to_string();
    let inset = c.safe_inset.to_string();
    let contrast = c.min_contrast.to_string();
    let motion = c.min_motion.to_string();
    let still = c.max_still.to_string();
    let cps = c.chars_per_second.to_string();
    let chars = c.min_chars.to_string();
    let cell = c.cell_width.to_string();
    cmd.args([
        "--interval",
        &interval,
        "--safe-inset",
        &inset,
        "--min-contrast",
        &contrast,
        "--min-motion",
        &motion,
        "--max-still",
        &still,
        "--chars-per-second",
        &cps,
        "--min-chars",
        &chars,
        "--cell-width",
        &cell,
    ]);
    if frames {
        cmd.arg("--frames");
    }
    match cmd.status() {
        Ok(s) if s.success() => exit::OK,
        Ok(s) => s.code().map_or(exit::BUILD, |c| c.clamp(0, 255) as u8),
        Err(e) => {
            crate::error(format!("cannot run the project binary: {e}"));
            exit::ENV
        }
    }
}

/// `oxframe inspect --at <seconds>`.
pub fn inspect(at: f64, scene: Option<String>) -> u8 {
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
    let build = match cargo::build(&root, false, cfg.bin.as_deref()) {
        Ok(b) if b.success => b,
        Ok(_) => return exit::BUILD,
        Err(e) => {
            crate::error(format!("cannot run cargo: {e}"));
            return exit::ENV;
        }
    };
    let Some(exe) = build.executable else {
        crate::error("build produced no binary");
        return exit::BUILD;
    };
    let mut cmd = Command::new(exe);
    cmd.current_dir(&root)
        .args(["inspect", "--at", &at.to_string()]);
    if let Some(scene) = &scene {
        cmd.args(["--scene", scene]);
    }
    match cmd.status() {
        Ok(s) => s.code().map_or(exit::BUILD, |c| c.clamp(0, 255) as u8),
        Err(e) => {
            crate::error(format!("cannot run the project binary: {e}"));
            exit::ENV
        }
    }
}

fn collect_meta(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            collect_meta(&p, out);
        } else if p.to_string_lossy().ends_with(".meta.json") {
            out.push(p);
        }
    }
}
