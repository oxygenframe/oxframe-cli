//! `oxframe render`: release build, then the project binary's `render`.

use std::process::Command;

use crate::{cargo, config, exit};

pub struct RenderArgs {
    pub out: Option<std::path::PathBuf>,
    pub scene: Option<String>,
    pub range: Option<String>,
    pub format: Option<String>,
    pub size: Option<String>,
    pub cpu: bool,
    pub no_audio: bool,
}

pub fn run(a: RenderArgs) -> u8 {
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
    // Check ffmpeg before a potentially long build.
    let png = a.format.as_deref() == Some("png")
        || a.out
            .as_ref()
            .is_some_and(|o| o.extension().is_none() && a.format.is_none());
    if !png && find_ffmpeg(cfg.render.ffmpeg.as_deref()).is_none() {
        crate::error(FFMPEG_HELP);
        return exit::ENV;
    }

    crate::note("building (release)…");
    let build = match cargo::build(&root, true, cfg.bin.as_deref()) {
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
    cmd.current_dir(&root).arg("render");
    if let Some(o) = &a.out {
        cmd.arg("--out").arg(o);
    }
    for (flag, v) in [
        ("--scene", &a.scene),
        ("--range", &a.range),
        ("--format", &a.format),
        ("--size", &a.size),
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
    cmd.args(cfg.theme.args());
    match cmd.status() {
        Ok(s) => s.code().map_or(exit::BUILD, |c| c.clamp(0, 255) as u8),
        Err(e) => {
            crate::error(format!("cannot run the project binary: {e}"));
            exit::ENV
        }
    }
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
