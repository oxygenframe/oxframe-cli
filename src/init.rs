//! `oxframe init <name>`.

use std::path::Path;

use crate::exit;

const CARGO_TOML: &str = include_str!("../templates/default/Cargo.toml.tmpl");
const MAIN_RS: &str = include_str!("../templates/default/src/main.rs");
const META: &str = include_str!("../templates/default/src/intro.meta.json");
const OXFRAME_TOML: &str = include_str!("../templates/default/oxframe.toml");
const GITIGNORE: &str = include_str!("../templates/default/gitignore");
const EXPLAINER_MAIN_RS: &str = include_str!("../templates/explainer/src/main.rs");
const SHORT_MAIN_RS: &str = include_str!("../templates/short/src/main.rs");
const CHART_MAIN_RS: &str = include_str!("../templates/chart/src/main.rs");

const NODE_CARGO_TOML: &str = include_str!("../templates/node/Cargo.toml.tmpl");
const NODE_LIB_RS: &str = include_str!("../templates/node/src/lib.rs");
const NODE_DEMO_RS: &str = include_str!("../templates/node/examples/demo.rs");
const NODE_TEST_RS: &str = include_str!("../templates/node/tests/reference.rs");
const NODE_README: &str = include_str!("../templates/node/README.md");
const NODE_GITIGNORE: &str = include_str!("../templates/node/gitignore");

/// What `oxframe init` creates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Template {
    /// A video project with one scene.
    Default,
    /// A library crate with a node of your own, a demo scene and a test.
    Node,
    /// A video project built from components: a title, a bullet list and an end card.
    Explainer,
    /// A vertical 9:16 short for social, built from components.
    Short,
    /// A chart explainer: a function on axes with a dot that rides the curve.
    Chart,
}

/// SDK version this CLI is made for (pinned in generated projects).
pub const SDK_VERSION: &str = "0.23.3";

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

pub fn oxframe_dependency(sdk_path: Option<&Path>) -> String {
    dependency(sdk_path, "")
}

/// `oxframe = { … }` with `extra` (such as `, features = [..]`) appended.
fn dependency(sdk_path: Option<&Path>, extra: &str) -> String {
    match sdk_path {
        Some(p) => {
            let crate_dir = p.join("crates/oxframe");
            let dir = crate_dir.canonicalize().unwrap_or(crate_dir);
            format!("oxframe = {{ path = {:?}{extra} }}", dir.to_string_lossy())
        }
        None => format!("oxframe = {{ version = \"={SDK_VERSION}\"{extra} }}"),
    }
}

/// A node crate: `src/lib.rs`, `examples/demo.rs`, `tests/reference.rs`.
pub fn generate_node(dir: &Path, name: &str, sdk_path: Option<&Path>) -> std::io::Result<()> {
    let krate = name.replace('-', "_");
    let dep = dependency(sdk_path, ", features = [\"unstable-node-api\"]");
    let fill = |t: &str| {
        t.replace("{{name}}", name)
            .replace("{{crate}}", &krate)
            .replace("{{oxframe_dep}}", &dep)
    };
    for sub in ["src", "examples", "tests"] {
        std::fs::create_dir_all(dir.join(sub))?;
    }
    std::fs::write(dir.join("Cargo.toml"), fill(NODE_CARGO_TOML))?;
    std::fs::write(dir.join("src/lib.rs"), fill(NODE_LIB_RS))?;
    std::fs::write(dir.join("examples/demo.rs"), fill(NODE_DEMO_RS))?;
    std::fs::write(dir.join("tests/reference.rs"), fill(NODE_TEST_RS))?;
    std::fs::write(dir.join("README.md"), fill(NODE_README))?;
    std::fs::write(dir.join(".gitignore"), NODE_GITIGNORE)?;
    crate::agents::write(dir, false)?;
    Ok(())
}

pub fn generate(dir: &Path, name: &str, sdk_path: Option<&Path>) -> std::io::Result<()> {
    generate_scene(dir, name, sdk_path, MAIN_RS, "intro")
}

/// An explainer video project: the same layout as the default, with a scene
/// built from the components (TitleCard, BulletList, EndCard).
pub fn generate_explainer(dir: &Path, name: &str, sdk_path: Option<&Path>) -> std::io::Result<()> {
    generate_scene(dir, name, sdk_path, EXPLAINER_MAIN_RS, "explainer")
}

/// A vertical 9:16 short, built from the components.
pub fn generate_short(dir: &Path, name: &str, sdk_path: Option<&Path>) -> std::io::Result<()> {
    generate_scene(dir, name, sdk_path, SHORT_MAIN_RS, "short")
}

/// A chart explainer: a function on axes.
pub fn generate_chart(dir: &Path, name: &str, sdk_path: Option<&Path>) -> std::io::Result<()> {
    generate_scene(dir, name, sdk_path, CHART_MAIN_RS, "chart")
}

/// Writes a one-scene video project; `main_rs` is the scene template and
/// `scene` names its meta file (`src/<scene>.meta.json`).
fn generate_scene(
    dir: &Path,
    name: &str,
    sdk_path: Option<&Path>,
    main_rs: &str,
    scene: &str,
) -> std::io::Result<()> {
    let fill = |t: &str| {
        t.replace("{{name}}", name)
            .replace("{{oxframe_dep}}", &oxframe_dependency(sdk_path))
    };
    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::create_dir_all(dir.join("assets"))?;
    std::fs::write(dir.join("Cargo.toml"), fill(CARGO_TOML))?;
    std::fs::write(dir.join("src/main.rs"), fill(main_rs))?;
    std::fs::write(dir.join(format!("src/{scene}.meta.json")), META)?;
    std::fs::write(dir.join("oxframe.toml"), OXFRAME_TOML)?;
    std::fs::write(dir.join(".gitignore"), GITIGNORE)?;
    std::fs::write(dir.join("assets/.gitkeep"), "")?;
    crate::agents::write(dir, false)?;
    Ok(())
}

pub fn run(name: &str, template: Template, sdk_path: Option<&Path>) -> u8 {
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
    let made = match template {
        Template::Default => generate(dir, name, sdk_path),
        Template::Node => generate_node(dir, name, sdk_path),
        Template::Explainer => generate_explainer(dir, name, sdk_path),
        Template::Short => generate_short(dir, name, sdk_path),
        Template::Chart => generate_chart(dir, name, sdk_path),
    };
    if let Err(e) = made {
        crate::error(format!("cannot create project: {e}"));
        return exit::ENV;
    }
    crate::note(format!("created `{name}`"));
    if template == Template::Node {
        anstream::eprintln!(
            "\n  cd {name}\n  cargo test                           # reference tests\n  cargo run --example demo -- render   # out/demo.mp4\n\nThe node API is behind the `unstable-node-api` feature and may change.\n"
        );
        return exit::OK;
    }
    anstream::eprintln!(
        "\n  cd {name}\n  oxframe dev                 # editor with hot reload\n  oxframe check               # storyboard + report.json\n  oxframe snapshot --at 1     # one frame as PNG\n  oxframe render              # out/{name}.mp4\n\nAI agents: AGENTS.md and .claude/skills/oxframe explain the project to them.\n"
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
    fn generates_a_node_crate() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("my-gauge");
        generate_node(&dir, "my-gauge", None).unwrap();
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"my-gauge\""));
        assert!(
            cargo.contains("features = [\"unstable-node-api\"]"),
            "{cargo}"
        );
        let demo = std::fs::read_to_string(dir.join("examples/demo.rs")).unwrap();
        assert!(demo.contains("use my_gauge::Gauge;"), "{demo}");
        for f in [
            "src/lib.rs",
            "tests/reference.rs",
            "README.md",
            ".gitignore",
            "AGENTS.md",
        ] {
            assert!(dir.join(f).exists(), "{f}");
        }
        assert!(
            !std::fs::read_to_string(dir.join("tests/reference.rs"))
                .unwrap()
                .contains("{{")
        );
    }

    #[test]
    fn generates_files() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("demo");
        generate(&dir, "demo", None).unwrap();
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"demo\""));
        assert!(cargo.contains(&format!("version = \"={SDK_VERSION}\"")));
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
