//! `oxframe render`: release build, then the project binary's `render`.

use std::process::Command;

use serde_json::json;

use crate::reply::Done;
use crate::{cargo, config, exit};

pub struct RenderArgs {
    pub out: Option<std::path::PathBuf>,
    pub scene: Option<String>,
    pub range: Option<String>,
    pub format: Option<String>,
    pub size: Option<String>,
    /// `16:9`, `9:16`, `1:1`, or `all`.
    pub aspect: Option<String>,
    pub cpu: bool,
    pub no_audio: bool,
    /// Spoken lines as .srt or .vtt.
    pub subtitles: Option<std::path::PathBuf>,
    /// A PDF of the presentation stops instead of a video.
    pub slides: Option<std::path::PathBuf>,
    pub json: bool,
}

pub fn run(a: RenderArgs) -> Done {
    let root = match config::project_root() {
        Ok(r) => r,
        Err(e) => return crate::reply::fail(a.json, exit::ENV, e),
    };
    let cfg = match config::Config::load(&root) {
        Ok(c) => c,
        Err(e) => return crate::reply::fail(a.json, exit::ENV, e),
    };
    // Check ffmpeg before a potentially long build.
    let png = a.slides.is_some()
        || a.format.as_deref() == Some("png")
        || a.out
            .as_ref()
            .is_some_and(|o| o.extension().is_none() && a.format.is_none());
    if !png && find_ffmpeg(cfg.render.ffmpeg.as_deref()).is_none() {
        let hints = crate::hints::from_text(FFMPEG_HELP);
        if !a.json {
            crate::error(FFMPEG_HELP);
            crate::hints::show(&hints);
        }
        return crate::reply::done(
            a.json,
            exit::ENV,
            json!({ "error": FFMPEG_HELP, "hints": hints }),
        );
    }

    if !a.json {
        crate::note("building (release)…");
    }
    let build = match if a.json {
        cargo::build_quiet(&root, true, cfg.bin.as_deref())
    } else {
        cargo::build(&root, true, cfg.bin.as_deref())
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
    cmd.current_dir(&root).arg("render");
    if let Some(o) = &a.out {
        cmd.arg("--out").arg(o);
    }
    for (flag, v) in [
        ("--scene", &a.scene),
        ("--range", &a.range),
        ("--format", &a.format),
        ("--size", &a.size),
        ("--aspect", &a.aspect),
    ] {
        if let Some(v) = v {
            cmd.args([flag, v]);
        }
    }
    if let Some(f) = &cfg.render.ffmpeg {
        cmd.arg("--ffmpeg").arg(f);
    }
    if a.cpu {
        cmd.arg("--cpu");
    }
    if a.no_audio {
        cmd.arg("--no-audio");
    }
    if let Some(p) = &a.subtitles {
        cmd.arg("--subtitles").arg(p);
    }
    if let Some(p) = &a.slides {
        cmd.arg("--slides").arg(p);
    }
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

pub const FFMPEG_HELP: &str = "\
ffmpeg was not found in PATH. Install it and try again:
  Linux:   sudo apt install ffmpeg      (or dnf/pacman install ffmpeg)
  macOS:   brew install ffmpeg
  Windows: winget install Gyan.FFmpeg   (then reopen the terminal)
or set `ffmpeg = \"/path/to/ffmpeg\"` under [render] in oxframe.toml.
PNG sequences (--format png) do not need ffmpeg.";

/// `ffmpeg` from config or PATH (`where`-style lookup on Windows).
pub fn find_ffmpeg(explicit: Option<&std::path::Path>) -> Option<std::path::PathBuf> {
    if let Some(p) = explicit {
        return p.is_file().then(|| p.to_path_buf());
    }
    let name = if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join(name))
            .find(|p| p.is_file())
    })
}
