//! `oxframe.toml` in the project root.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub dev: Dev,
    pub render: Render,
    pub check: Check,
    /// Binary target to run when the package has several.
    pub bin: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Dev {
    pub port: u16,
    pub open: bool,
    /// "dev" or "release".
    pub profile: String,
}

impl Default for Dev {
    fn default() -> Self {
        Self {
            port: 3000,
            open: false,
            profile: "dev".into(),
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Render {
    pub ffmpeg: Option<PathBuf>,
}

/// Thresholds for the visual half of `oxframe check`.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Check {
    /// Seconds between storyboard frames; every marker is included too.
    pub interval: f64,
    /// Safe area inset in pixels. 0 disables that warning.
    pub safe_inset: f64,
    /// Minimum text contrast (WCAG). Below this is an error.
    pub min_contrast: f64,
    /// A large move shorter than this (seconds) is a warning.
    pub min_motion: f64,
    /// A pause with nothing changing, when the project has no soundtrack.
    pub max_still: f64,
    /// Reading speed for the `too_fast` text rule.
    pub chars_per_second: f64,
    /// Shorter text is not checked for reading time.
    pub min_chars: u32,
    /// Storyboard cell width in pixels.
    pub cell_width: u32,
}

impl Default for Check {
    fn default() -> Self {
        Self {
            interval: 1.0,
            safe_inset: 48.0,
            min_contrast: 4.5,
            min_motion: 0.15,
            max_still: 3.0,
            chars_per_second: 20.0,
            min_chars: 24,
            cell_width: 320,
        }
    }
}

impl Config {
    pub fn load(root: &Path) -> anyhow::Result<Self> {
        let path = root.join("oxframe.toml");
        match std::fs::read_to_string(&path) {
            Ok(text) => {
                toml::from_str(&text).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(anyhow::anyhow!("{}: {e}", path.display())),
        }
    }
}

/// Directory containing the project's `Cargo.toml`, searching upwards.
pub fn project_root() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    cwd.ancestors()
        .find(|d| d.join("Cargo.toml").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| {
            anyhow::anyhow!("no Cargo.toml here or above; run `oxframe init <name>` first")
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_config() {
        let c: Config =
            toml::from_str("[dev]\nport = 4000\nopen = true\n[render]\nffmpeg = \"/x/ffmpeg\"")
                .unwrap();
        assert_eq!(c.dev.port, 4000);
        assert!(c.dev.open);
        assert_eq!(c.dev.profile, "dev");
        assert_eq!(c.render.ffmpeg.unwrap(), PathBuf::from("/x/ffmpeg"));
        assert!(toml::from_str::<Config>("[dev]\nprot = 1").is_err());
    }
}
