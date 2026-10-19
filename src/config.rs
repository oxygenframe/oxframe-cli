//! `oxframe.toml` in the project root.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    pub dev: Dev,
    pub render: Render,
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
