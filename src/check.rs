//! `oxframe check`: cargo check, meta validation, version agreement, visual lint.

use std::path::{Path, PathBuf};
use std::process::Command;

use oxframe_protocol::{Meta, PROTOCOL_VERSION};

use serde_json::{Value, json};

use crate::reply::Done;
use crate::{cargo, config, exit, render, ui};

pub fn run(frames: bool, against: Option<PathBuf>, out: Option<PathBuf>, json: bool) -> Done {
    let root = match config::project_root() {
        Ok(r) => r,
        Err(e) => return crate::reply::fail(json, exit::ENV, e),
    };
    let cfg = match config::Config::load(&root) {
        Ok(c) => c,
        Err(e) => return crate::reply::fail(json, exit::ENV, e),
    };
    let mut code = exit::OK;
    let mut notes = Vec::new();

    if !json {
        crate::note("cargo check");
    }
    let checked = match cargo::check(&root, cfg.bin.as_deref(), json) {
        Ok(b) => b,
        Err(e) => return crate::reply::fail(json, exit::ENV, format!("cannot run cargo: {e}")),
    };
    if !checked.success {
        if !json {
            crate::hints::show(&checked.hints);
        }
        return crate::reply::done(
            json,
            exit::BUILD,
            json!({
                "error": "cargo check failed",
                "diagnostics": checked.messages,
                "hints": checked.hints,
            }),
        );
    }

    let mut metas = Vec::new();
    collect_meta(&root.join("src"), &mut metas);
    for path in &metas {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        match Meta::parse(&text) {
            Ok(_) => {
                if !json {
                    anstream::eprintln!(
                        "  ok  {}",
                        path.strip_prefix(&root).unwrap_or(path).display()
                    );
                }
            }
            Err(e) => {
                let msg = format!("{}: {e}", path.display());
                if json {
                    notes.push(msg);
                } else {
                    crate::error(&msg);
                }
                code = exit::BUILD;
            }
        }
    }

    // Versions: CLI, the SDK the project links (asked from its binary) and
    // the embedded editor must speak the same protocol.
    if !json {
        crate::note("checking protocol versions");
    }
    let build = match if json {
        cargo::build_quiet(&root, false, cfg.bin.as_deref())
    } else {
        cargo::build(&root, false, cfg.bin.as_deref())
    } {
        Ok(b) if b.success => b,
        Ok(b) => {
            if !json {
                crate::hints::show(&b.hints);
            }
            return crate::reply::done(
                json,
                exit::BUILD,
                json!({
                    "error": "build failed",
                    "diagnostics": b.messages,
                    "hints": b.hints,
                }),
            );
        }
        Err(e) => return crate::reply::fail(json, exit::ENV, format!("cannot run cargo: {e}")),
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
    if !json {
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
    }
    let mismatch = sdk.as_ref().is_some_and(|(p, _)| *p != PROTOCOL_VERSION)
        || ui_v.is_some_and(|p| p != PROTOCOL_VERSION);
    if mismatch {
        let error =
            "protocol versions differ: update oxframe-cli and the oxframe SDK to matching releases";
        if !json {
            crate::error(error);
            crate::hints::show(&crate::hints::from_text(error));
        }
        return crate::reply::done(
            json,
            exit::VERSION,
            json!({
                "error": error,
                "hints": crate::hints::from_text(error),
            }),
        );
    }

    if render::find_ffmpeg(cfg.render.ffmpeg.as_deref()).is_none() && !json {
        anstream::eprintln!(
            "\nnote: ffmpeg not found — `oxframe render` will download a static build the first time.\n{}",
            render::FFMPEG_HELP
        );
    }
    if code != exit::OK {
        return crate::reply::done(json, code, json!({ "error": notes.join("\n") }));
    }
    let Some(exe) = exe else {
        return crate::reply::fail(
            json,
            exit::BUILD,
            "build produced no binary to visually check",
        );
    };
    if !json {
        crate::note("visual check");
    }
    let mut cmd = Command::new(exe);
    cmd.current_dir(&root).arg("check");
    if let Some(path) = crate::ffmpeg::child_path() {
        cmd.env("PATH", path);
    }
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
    // The project binary runs in the project root; the paths are the user's.
    let user_path =
        |dir: &PathBuf| std::env::current_dir().map_or(dir.clone(), |cwd| cwd.join(dir));
    if let Some(dir) = &against {
        cmd.arg("--against").arg(user_path(dir));
    }
    let out = out
        .as_ref()
        .map_or_else(|| root.join("out/check"), user_path);
    cmd.arg("--out").arg(&out);
    cmd.args(cfg.theme.args());
    let (status, stdout, stderr) = match crate::reply::capture(&mut cmd, json) {
        Ok(o) => o,
        Err(e) => {
            return crate::reply::fail(
                json,
                exit::ENV,
                format!("cannot run the project binary: {e}"),
            );
        }
    };
    let hints = crate::hints::from_text(&stderr);
    if !json {
        crate::hints::show(&hints);
    }
    let mut report = read_json(&out.join("report.json"));
    // A person's review notes come first: they say what is wrong in words.
    let review = crate::notes::issues(&root, &out);
    let mut status = status;
    if !review.is_empty() {
        if report.is_null() {
            report = json!({ "issues": [] });
        }
        if let Some(issues) = report.get_mut("issues").and_then(Value::as_array_mut) {
            issues.splice(0..0, review.iter().cloned());
            let _ = std::fs::write(
                out.join("report.json"),
                serde_json::to_string_pretty(&report).unwrap_or_default(),
            );
        }
        if !json {
            anstream::eprintln!(
                "
review notes from the editor ({}):",
                review.len()
            );
            for n in &review {
                let src = n["source"]
                    .as_str()
                    .map_or(String::new(), |s| format!(" {s}"));
                anstream::eprintln!(
                    "  error[{}] #{} at {:.2}s {}{src}: {}",
                    crate::notes::RULE,
                    n["note"],
                    n["time"][0].as_f64().unwrap_or(0.0),
                    n["node"].as_str().unwrap_or(""),
                    n["note_text"].as_str().unwrap_or(""),
                );
            }
            anstream::eprintln!(
                "  close each with `oxframe notes done <id> --reply \"what you changed\"`"
            );
        }
        if status == exit::OK {
            status = exit::BUILD;
        }
    }
    let storyboard = out.join("storyboard.png");
    crate::reply::done(
        json,
        status,
        json!({
            "storyboard": storyboard.display().to_string(),
            "report": report,
            "stdout": stdout,
            "stderr": stderr,
            "hints": hints,
        }),
    )
}

/// `oxframe inspect --at <seconds>`.
pub fn inspect(at: f64, scene: Option<String>, graph: bool, json: bool) -> Done {
    let root = match config::project_root() {
        Ok(r) => r,
        Err(e) => return crate::reply::fail(json, exit::ENV, e),
    };
    let cfg = match config::Config::load(&root) {
        Ok(c) => c,
        Err(e) => return crate::reply::fail(json, exit::ENV, e),
    };
    let build = match if json {
        cargo::build_quiet(&root, false, cfg.bin.as_deref())
    } else {
        cargo::build(&root, false, cfg.bin.as_deref())
    } {
        Ok(b) if b.success => b,
        Ok(b) => {
            if !json {
                crate::hints::show(&b.hints);
            }
            return crate::reply::done(
                json,
                exit::BUILD,
                json!({
                    "error": "build failed",
                    "diagnostics": b.messages,
                    "hints": b.hints,
                }),
            );
        }
        Err(e) => return crate::reply::fail(json, exit::ENV, format!("cannot run cargo: {e}")),
    };
    let Some(exe) = build.executable else {
        return crate::reply::fail(json, exit::BUILD, "build produced no binary");
    };
    let mut cmd = Command::new(exe);
    cmd.current_dir(&root)
        .args(["inspect", "--at", &at.to_string()]);
    if let Some(path) = crate::ffmpeg::child_path() {
        cmd.env("PATH", path);
    }
    if let Some(scene) = &scene {
        cmd.args(["--scene", scene]);
    }
    if graph {
        cmd.arg("--graph");
    }
    cmd.args(cfg.theme.args());
    let (status, stdout, stderr) = match crate::reply::capture(&mut cmd, json) {
        Ok(o) => o,
        Err(e) => {
            return crate::reply::fail(
                json,
                exit::ENV,
                format!("cannot run the project binary: {e}"),
            );
        }
    };
    let hints = crate::hints::from_text(&stderr);
    if !json {
        crate::hints::show(&hints);
    }
    let inspect = serde_json::from_str::<Value>(&stdout).unwrap_or(Value::String(stdout));
    crate::reply::done(
        json,
        status,
        json!({ "inspect": inspect, "stderr": stderr, "hints": hints }),
    )
}

fn read_json(path: &Path) -> Value {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null)
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
