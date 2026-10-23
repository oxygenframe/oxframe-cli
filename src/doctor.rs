//! `oxframe doctor`: checks what a first build and render need, and says how
//! to fix what is missing. Exit 2 when something required is missing;
//! warnings (no ffmpeg, a project on another SDK) keep exit 0.

use std::path::Path;
use std::process::Command;

use serde_json::{Value, json};

use crate::init::SDK_VERSION;
use crate::update_check::parse_version;
use crate::upgrade::{Plan, plan};

/// The oldest Rust the SDK builds with (its `rust-version`).
const MSRV: &str = env!("CARGO_PKG_RUST_VERSION");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Ok,
    Warn,
    Fail,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Warn => "warn",
            Status::Fail => "fail",
        }
    }
}

struct Check {
    name: &'static str,
    status: Status,
    detail: String,
    fix: Option<String>,
}

pub fn run(json_mode: bool) -> crate::reply::Done {
    let mut checks = vec![rust(), linker(), ffmpeg()];
    if let Some(project) = project() {
        checks.push(project);
    }
    let code = if checks.iter().any(|c| c.status == Status::Fail) {
        crate::exit::ENV
    } else {
        crate::exit::OK
    };
    if json_mode {
        let list: Vec<Value> = checks
            .iter()
            .map(|c| {
                json!({ "name": c.name, "status": c.status.label(), "detail": c.detail, "fix": c.fix })
            })
            .collect();
        return crate::reply::Done::with(code, json!({ "checks": list }));
    }
    println!("oxframe doctor (oxframe-cli {})", env!("CARGO_PKG_VERSION"));
    for c in &checks {
        println!("  {:<5} {:<8} {}", c.status.label(), c.name, c.detail);
        if let Some(fix) = &c.fix {
            for line in fix.lines() {
                println!("                 {line}");
            }
        }
    }
    let (fails, warns) = (
        checks.iter().filter(|c| c.status == Status::Fail).count(),
        checks.iter().filter(|c| c.status == Status::Warn).count(),
    );
    println!(
        "{}",
        match (fails, warns) {
            (0, 0) => "Everything oxframe needs is here.".to_owned(),
            (0, w) => format!("Ready to build; {w} warning(s) above."),
            (f, _) => format!(
                "{f} problem(s) above stop a build. Fix them and run `oxframe doctor` again."
            ),
        }
    );
    crate::reply::Done::plain(code)
}

fn output(cmd: &str, args: &[&str]) -> Option<String> {
    let out = Command::new(cmd).args(args).output().ok()?;
    out.status.success().then(|| {
        let mut s = String::from_utf8_lossy(&out.stdout).into_owned();
        s.push_str(&String::from_utf8_lossy(&out.stderr));
        s
    })
}

/// `rustc 1.97.1 (…)` → `(1, 97, 1)`.
fn rustc_version(text: &str) -> Option<(u64, u64, u64)> {
    let v = text.split_whitespace().nth(1)?;
    parse_version(v.split('-').next()?)
}

/// `ffmpeg version 7.1.1-1ubuntu1 Copyright …` → `7.1.1-1ubuntu1`.
fn ffmpeg_version(text: &str) -> Option<&str> {
    let line = text.lines().next()?;
    line.strip_prefix("ffmpeg version ")?
        .split_whitespace()
        .next()
}

fn msrv() -> (u64, u64, u64) {
    parse_version(&format!("{MSRV}.0"))
        .or_else(|| parse_version(MSRV))
        .unwrap_or((1, 0, 0))
}

fn rust() -> Check {
    let install = "Install Rust with rustup: https://rustup.rs".to_owned();
    let Some(text) = output("rustc", &["--version"]) else {
        return Check {
            name: "rust",
            status: Status::Fail,
            detail: "rustc not found".into(),
            fix: Some(install),
        };
    };
    if output("cargo", &["--version"]).is_none() {
        return Check {
            name: "rust",
            status: Status::Fail,
            detail: "rustc found, cargo not found".into(),
            fix: Some(install),
        };
    }
    match rustc_version(&text) {
        Some(v) if v >= msrv() => Check {
            name: "rust",
            status: Status::Ok,
            detail: format!("rustc {}.{}.{} (needs {MSRV} or newer)", v.0, v.1, v.2),
            fix: None,
        },
        Some(v) => Check {
            name: "rust",
            status: Status::Fail,
            detail: format!("rustc {}.{}.{} is older than {MSRV}", v.0, v.1, v.2),
            fix: Some("rustup update stable".into()),
        },
        None => Check {
            name: "rust",
            status: Status::Warn,
            detail: format!("cannot read the version from `{}`", text.trim()),
            fix: None,
        },
    }
}

/// Rust links with the system C toolchain on Linux and macOS; a fresh
/// machine often lacks it and the first build fails with "linker `cc` not
/// found". On Windows rustup installs or asks for the MSVC tools itself.
fn linker() -> Check {
    if cfg!(windows) {
        return Check {
            name: "linker",
            status: Status::Ok,
            detail: "MSVC (rustup sets it up on Windows)".into(),
            fix: None,
        };
    }
    if output("cc", &["--version"]).is_some() {
        return Check {
            name: "linker",
            status: Status::Ok,
            detail: "cc".into(),
            fix: None,
        };
    }
    Check {
        name: "linker",
        status: Status::Fail,
        detail: "no C linker (`cc`): Rust cannot link a program".into(),
        fix: Some(if cfg!(target_os = "macos") {
            "xcode-select --install".into()
        } else {
            "Debian/Ubuntu: sudo apt install build-essential\nFedora: sudo dnf install gcc\nArch: sudo pacman -S base-devel".into()
        }),
    }
}

fn ffmpeg() -> Check {
    let explicit = crate::config::project_root()
        .ok()
        .and_then(|root| crate::config::Config::load(&root).ok())
        .and_then(|cfg| cfg.render.ffmpeg);
    let Some(path) = crate::render::find_ffmpeg(explicit.as_deref()) else {
        return Check {
            name: "ffmpeg",
            status: Status::Warn,
            detail: "not found: needed for MP4, WebM and ProRes; PNG frames and `oxframe dev` work without it".into(),
            fix: Some(
                crate::render::FFMPEG_HELP
                    .lines()
                    .skip(1)
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
        };
    };
    let version = output(&path.to_string_lossy(), &["-version"])
        .and_then(|t| ffmpeg_version(&t).map(str::to_owned))
        .unwrap_or_else(|| "version unknown".into());
    Check {
        name: "ffmpeg",
        status: Status::Ok,
        detail: format!("{version} ({})", path.display()),
        fix: None,
    }
}

/// The project in this directory, when there is one: is it on the SDK this
/// CLI makes projects for?
fn project() -> Option<Check> {
    let root = crate::config::project_root().ok()?;
    let toml = std::fs::read_to_string(root.join("Cargo.toml")).ok()?;
    let name = project_name(&root);
    let (status, detail, fix) = match plan(&toml) {
        Ok((Plan::Current, _)) => (Status::Ok, format!("{name}: oxframe {SDK_VERSION}"), None),
        Ok((Plan::Upgrade(have), _)) => (
            Status::Warn,
            format!("{name}: oxframe {have}, this CLI makes {SDK_VERSION} projects"),
            Some("oxframe upgrade   (moves the project to this CLI's SDK)".into()),
        ),
        Ok((Plan::Newer(have), _)) => (
            Status::Warn,
            format!("{name}: oxframe {have} is newer than this CLI ({SDK_VERSION})"),
            Some("cargo install oxframe-cli   (or the installer you used)".into()),
        ),
        Ok((Plan::Local, _)) => (
            Status::Ok,
            format!("{name}: a local oxframe checkout"),
            None,
        ),
        // Not an oxframe project (no `oxframe` dependency): nothing to say.
        Err(_) => return None,
    };
    Some(Check {
        name: "project",
        status,
        detail,
        fix,
    })
}

fn project_name(root: &Path) -> String {
    root.file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rustc_and_ffmpeg_versions() {
        assert_eq!(
            rustc_version("rustc 1.97.1 (a1b2c3d4e 2026-09-30)\n"),
            Some((1, 97, 1))
        );
        assert_eq!(
            rustc_version("rustc 1.98.0-nightly (abc 2026-10-01)"),
            Some((1, 98, 0))
        );
        assert_eq!(rustc_version("nonsense"), None);
        assert_eq!(
            ffmpeg_version("ffmpeg version 7.1.1-1ubuntu1 Copyright (c) 2000-2025\nbuilt with gcc"),
            Some("7.1.1-1ubuntu1")
        );
        assert_eq!(ffmpeg_version("something else"), None);
    }

    #[test]
    fn msrv_is_the_crate_rust_version() {
        let (major, minor, _) = msrv();
        assert_eq!(format!("{major}.{minor}"), MSRV);
    }
}
