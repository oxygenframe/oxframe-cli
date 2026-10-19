//! `oxframe upgrade`: moves the project to the SDK version this CLI is made
//! for, then refreshes Cargo.lock and the AI agent files.

use std::path::Path;
use std::process::Command;

use toml_edit::{DocumentMut, Item, Value};

use crate::init::SDK_VERSION;
use crate::update_check::parse_version;
use crate::{cargo, config, exit};

/// What the project's `oxframe` dependency looks like.
#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    /// Already pinned to `SDK_VERSION`.
    Current,
    /// From this version to `SDK_VERSION`.
    Upgrade(String),
    /// The project needs a newer CLI.
    Newer(String),
    /// A path or git dependency: nothing to pin.
    Local,
}

/// Rewrites `[dependencies] oxframe` in `cargo_toml` to `=SDK_VERSION`,
/// keeping formatting and comments. Returns the new text (unchanged unless
/// the plan is `Upgrade`).
pub fn plan(cargo_toml: &str) -> anyhow::Result<(Plan, String)> {
    let mut doc: DocumentMut = cargo_toml.parse()?;
    let dep = doc
        .get_mut("dependencies")
        .and_then(|d| d.get_mut("oxframe"))
        .filter(|d| !d.is_none())
        .ok_or_else(|| anyhow::anyhow!("Cargo.toml has no `oxframe` dependency"))?;
    let version = match dep {
        Item::Value(Value::String(s)) => Some(s.value().clone()),
        Item::Value(Value::InlineTable(t)) => {
            if t.contains_key("path") || t.contains_key("git") {
                None
            } else {
                t.get("version").and_then(|v| v.as_str()).map(str::to_owned)
            }
        }
        Item::Table(t) => {
            if t.contains_key("path") || t.contains_key("git") {
                None
            } else {
                t.get("version").and_then(|v| v.as_str()).map(str::to_owned)
            }
        }
        _ => None,
    };
    let Some(req) = version else {
        return Ok((Plan::Local, cargo_toml.to_owned()));
    };
    let current = req.trim_start_matches(['=', '^', '~', ' ']).to_owned();
    let (Some(have), Some(want)) = (parse_version(&current), parse_version(SDK_VERSION)) else {
        anyhow::bail!("cannot read the oxframe version requirement `{req}`");
    };
    if have == want {
        return Ok((Plan::Current, cargo_toml.to_owned()));
    }
    if have > want {
        return Ok((Plan::Newer(current), cargo_toml.to_owned()));
    }
    let pinned = format!("={SDK_VERSION}");
    match dep {
        Item::Value(Value::String(s)) => {
            let decor = s.decor().clone();
            let mut v = toml_edit::Formatted::new(pinned);
            *v.decor_mut() = decor;
            *s = v;
        }
        Item::Value(Value::InlineTable(t)) => {
            t.insert("version", pinned.into());
        }
        Item::Table(t) => {
            t.insert("version", toml_edit::value(pinned));
        }
        _ => unreachable!("checked above"),
    }
    Ok((Plan::Upgrade(current), doc.to_string()))
}

pub fn run() -> u8 {
    let root = match config::project_root() {
        Ok(r) => r,
        Err(e) => {
            crate::error(e);
            return exit::ENV;
        }
    };
    let path = root.join("Cargo.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(e) => {
            crate::error(format!("{}: {e}", path.display()));
            return exit::ENV;
        }
    };
    let (plan, new_text) = match plan(&text) {
        Ok(p) => p,
        Err(e) => {
            crate::error(format!("{}: {e}", path.display()));
            return exit::ENV;
        }
    };
    match &plan {
        Plan::Current => crate::note(format!("already on oxframe {SDK_VERSION}")),
        Plan::Local => crate::note("oxframe is a path/git dependency; leaving its version alone"),
        Plan::Newer(v) => {
            crate::error(format!(
                "this project uses oxframe {v}, newer than this CLI supports ({SDK_VERSION}); \
                 update the CLI: cargo install oxframe-cli --force"
            ));
            return exit::VERSION;
        }
        Plan::Upgrade(from) => {
            if let Err(e) = std::fs::write(&path, new_text) {
                crate::error(format!("{}: {e}", path.display()));
                return exit::ENV;
            }
            crate::note(format!("oxframe {from} → {SDK_VERSION} in Cargo.toml"));
            if !cargo_update(&root) {
                return exit::BUILD;
            }
        }
    }
    match crate::agents::write(&root, false) {
        Ok(r) => {
            for f in &r.skipped {
                crate::note(format!(
                    "kept {f} (edited by you; `oxframe agents --force` replaces it)"
                ));
            }
        }
        Err(e) => crate::error(format!("cannot update agent files: {e}")),
    }
    if matches!(plan, Plan::Upgrade(_)) {
        crate::note("done; run `oxframe check` to build with the new SDK");
    }
    exit::OK
}

fn cargo_update(root: &Path) -> bool {
    match Command::new(cargo::cargo())
        .current_dir(root)
        .args(["update", "-p", "oxframe"])
        .status()
    {
        Ok(s) => s.success(),
        Err(e) => {
            crate::error(format!("cannot run cargo: {e}"));
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_dep(dep: &str) -> String {
        format!("[package]\nname = \"demo\"\n\n[dependencies]\n# the SDK\n{dep}\nserde = \"1\"\n")
    }

    #[test]
    fn upgrades_inline_table_and_keeps_the_rest() {
        let (p, out) = plan(&with_dep(
            "oxframe = { version = \"=0.0.9\", default-features = false }",
        ))
        .unwrap();
        assert_eq!(p, Plan::Upgrade("0.0.9".into()));
        assert!(
            out.contains(&format!("version = \"={SDK_VERSION}\"")),
            "{out}"
        );
        assert!(out.contains("default-features = false"));
        assert!(out.contains("# the SDK\n"));
        assert!(out.contains("serde = \"1\""));
    }

    #[test]
    fn upgrades_plain_string() {
        let (p, out) = plan(&with_dep("oxframe = \"0.0.1\"")).unwrap();
        assert_eq!(p, Plan::Upgrade("0.0.1".into()));
        assert!(
            out.contains(&format!("oxframe = \"={SDK_VERSION}\"")),
            "{out}"
        );
    }

    #[test]
    fn upgrades_dotted_table() {
        let text = "[dependencies.oxframe]\nversion = \"0.0.1\"\nfeatures = [\"wasm\"]\n";
        let (p, out) = plan(text).unwrap();
        assert_eq!(p, Plan::Upgrade("0.0.1".into()));
        assert!(
            out.contains(&format!("version = \"={SDK_VERSION}\"")),
            "{out}"
        );
        assert!(out.contains("features = [\"wasm\"]"));
    }

    #[test]
    fn leaves_current_newer_and_local_alone() {
        let current = with_dep(&format!("oxframe = \"={SDK_VERSION}\""));
        assert_eq!(plan(&current).unwrap().0, Plan::Current);
        assert_eq!(
            plan(&with_dep("oxframe = \"99.0.0\"")).unwrap().0,
            Plan::Newer("99.0.0".into())
        );
        let local = with_dep("oxframe = { path = \"../oxframe-sdk/crates/oxframe\" }");
        assert_eq!(plan(&local).unwrap(), (Plan::Local, local.clone()));
    }

    #[test]
    fn needs_an_oxframe_dependency() {
        assert!(plan("[dependencies]\nserde = \"1\"\n").is_err());
    }
}
