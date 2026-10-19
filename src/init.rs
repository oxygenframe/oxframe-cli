//! `oxframe init <name>`.

use std::path::Path;

use crate::exit;

const CARGO_TOML: &str = include_str!("../templates/default/Cargo.toml.tmpl");
const MAIN_RS: &str = include_str!("../templates/default/src/main.rs");
const META: &str = include_str!("../templates/default/src/intro.meta.json");
const OXFRAME_TOML: &str = include_str!("../templates/default/oxframe.toml");
const GITIGNORE: &str = include_str!("../templates/default/gitignore");

/// SDK version this CLI is made for (pinned in generated projects).
pub const SDK_VERSION: &str = "0.1.0";

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

pub fn oxframe_dependency(sdk_path: Option<&Path>) -> String {
    match sdk_path {
        Some(p) => {
            let crate_dir = p.join("crates/oxframe");
            let dir = crate_dir.canonicalize().unwrap_or(crate_dir);
            format!("oxframe = {{ path = {:?} }}", dir.to_string_lossy())
        }
        None => format!("oxframe = {{ version = \"={SDK_VERSION}\" }}"),
    }
}

pub fn generate(dir: &Path, name: &str, sdk_path: Option<&Path>) -> std::io::Result<()> {
    let fill = |t: &str| {
        t.replace("{{name}}", name)
            .replace("{{oxframe_dep}}", &oxframe_dependency(sdk_path))
    };
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::create_dir_all(dir.join("assets"))?;
    std::fs::write(dir.join("Cargo.toml"), fill(CARGO_TOML))?;
    std::fs::write(dir.join("src/main.rs"), fill(MAIN_RS))?;
    std::fs::write(dir.join("src/intro.meta.json"), META)?;
    std::fs::write(dir.join("oxframe.toml"), OXFRAME_TOML)?;
    std::fs::write(dir.join(".gitignore"), GITIGNORE)?;
    std::fs::write(dir.join("assets/.gitkeep"), "")?;
    crate::agents::write(dir, false)?;
    Ok(())
}

pub fn run(name: &str, sdk_path: Option<&Path>) -> u8 {
    if !valid_name(name) {
        crate::error(format!(
            "`{name}` is not a valid project name: use lowercase letters, digits, `-` and `_`, starting with a letter"
        ));
        return exit::ENV;
    }
    let dir = Path::new(name);
    if dir.exists() {
        crate::error(format!("`{name}` already exists"));
        return exit::ENV;
    }
    if let Err(e) = generate(dir, name, sdk_path) {
        crate::error(format!("cannot create project: {e}"));
        return exit::ENV;
    }
    crate::note(format!("created `{name}`"));
    anstream::eprintln!(
        "\n  cd {name}\n  oxframe dev                 # editor with hot reload\n  oxframe snapshot --at 1     # one frame as PNG\n  oxframe render              # out/{name}.mp4\n\nAI agents: AGENTS.md and .claude/skills/oxframe explain the project to them.\n"
    );
    exit::OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names() {
        assert!(valid_name("my-video_2"));
        assert!(!valid_name("2fast"));
        assert!(!valid_name("Hello"));
        assert!(!valid_name("a b"));
    }

    #[test]
    fn generates_files() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        generate(&dir, "demo", None).unwrap();
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"demo\""));
        assert!(cargo.contains("version = \"=0.1.0\""));
        assert!(cargo.contains("opt-level = 2"));
        assert!(
            std::fs::read_to_string(dir.join("src/main.rs"))
                .unwrap()
                .contains("Hello, demo")
        );
        for f in [
            "src/intro.meta.json",
            "oxframe.toml",
            ".gitignore",
            "assets/.gitkeep",
            "AGENTS.md",
            "CLAUDE.md",
            ".claude/skills/oxframe/SKILL.md",
            ".claude/skills/oxframe/reference.md",
        ] {
            assert!(dir.join(f).exists(), "{f}");
        }
        oxframe_protocol::Meta::parse(
            &std::fs::read_to_string(dir.join("src/intro.meta.json")).unwrap(),
        )
        .unwrap();
    }
}
