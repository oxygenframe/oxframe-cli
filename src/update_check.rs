//! "A newer oxframe-cli is available" hint, at most one network check a day.
//!
//! The check runs `cargo search` as a detached child that writes its output
//! into a cache file, so it never slows a command down and respects the
//! user's registry and proxy settings. The next run reads the cache.
//! Disabled with `OXFRAME_NO_UPDATE_CHECK=1` and on CI.

use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

const CRATE: &str = "oxframe-cli";
const INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Prints the hint if the cache knows a newer version, and refreshes the
/// cache in the background when it is older than a day.
pub fn run() {
    if std::env::var_os("OXFRAME_NO_UPDATE_CHECK").is_some() || std::env::var_os("CI").is_some() {
        return;
    }
    let Some(file) = cache_file() else { return };
    let text = std::fs::read_to_string(&file).unwrap_or_default();
    if let Some(latest) = parse_search(&text)
        && is_newer(&latest, env!("CARGO_PKG_VERSION"))
    {
        crate::note(format!(
            "oxframe-cli {latest} is available (you have {}): cargo install {CRATE} --force",
            env!("CARGO_PKG_VERSION")
        ));
    }
    let stale = std::fs::metadata(&file)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_none_or(|age| age > INTERVAL);
    if stale {
        refresh(&file);
    }
}

/// Starts `cargo search` writing into `file`; the child outlives us.
fn refresh(file: &PathBuf) {
    if let Some(dir) = file.parent()
        && std::fs::create_dir_all(dir).is_err()
    {
        return;
    }
    // Truncating also marks the cache fresh, so a failing search (offline)
    // is retried tomorrow, not on every command.
    let Ok(out) = File::create(file) else { return };
    let _ = Command::new(crate::cargo::cargo())
        .args(["search", CRATE, "--limit", "1", "--color", "never"])
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(Stdio::null())
        .spawn();
}

fn cache_file() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("oxframe").join("latest-cli.txt"))
}

/// `oxframe-cli = "0.2.0"    # description` → `0.2.0`.
fn parse_search(text: &str) -> Option<String> {
    let line = text
        .lines()
        .find(|l| l.starts_with(&format!("{CRATE} = ")))?;
    let v = line.split('"').nth(1)?;
    Some(v.to_owned())
}

/// Release versions only: `a.b.c` compared numerically; anything else is
/// never "newer".
pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

pub fn parse_version(v: &str) -> Option<(u64, u64, u64)> {
    let mut it = v.split('.').map(|p| p.parse::<u64>().ok());
    let r = (it.next()??, it.next()??, it.next()??);
    it.next().is_none().then_some(r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cargo_search_output() {
        let out = "oxframe-cli = \"0.3.1\"    # The `oxframe` command\n... and 2 crates more\n";
        assert_eq!(parse_search(out).as_deref(), Some("0.3.1"));
        assert_eq!(parse_search(""), None);
        assert_eq!(parse_search("oxframe = \"9.0.0\"  # other crate"), None);
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer("0.2.0", "0.1.9"));
        assert!(is_newer("0.10.0", "0.9.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.2.0-beta.1", "0.1.0"));
        assert!(!is_newer("garbage", "0.1.0"));
    }
}
