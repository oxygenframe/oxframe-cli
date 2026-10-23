//! `oxframe add`: copies a ready-made, checked block into the project. A
//! block is an `async fn` in `src/blocks/<name>.rs` that builds its nodes in
//! a group, plays them in and returns the group.

use std::path::Path;

use serde_json::{Value, json};

use crate::exit;

/// One block: its name (also the file and function name), the line that
/// calls it, and its source.
pub struct Block {
    pub name: &'static str,
    pub usage: &'static str,
    pub source: &'static str,
}

impl Block {
    /// The first line of the file's doc comment.
    pub fn summary(&self) -> &str {
        self.source
            .lines()
            .next()
            .and_then(|l| l.strip_prefix("//! "))
            .unwrap_or("")
    }
}

pub const BLOCKS: &[Block] = &[
    Block {
        name: "logo_reveal",
        usage: r#"blocks::logo_reveal(view, "oxframe", "Motion graphics as Rust code").await"#,
        source: include_str!("../templates/blocks/logo_reveal.rs"),
    },
    Block {
        name: "stat_counter",
        usage: r#"blocks::stat_counter(view, 1280.0, "stars on GitHub").await"#,
        source: include_str!("../templates/blocks/stat_counter.rs"),
    },
    Block {
        name: "progress_bar",
        usage: r#"blocks::progress_bar(view, "Tests passing", 0.92).await"#,
        source: include_str!("../templates/blocks/progress_bar.rs"),
    },
    Block {
        name: "quote_card",
        usage: r#"blocks::quote_card(view, "Make it work, then make it fast.", "Kent Beck").await"#,
        source: include_str!("../templates/blocks/quote_card.rs"),
    },
    Block {
        name: "before_after",
        usage: r#"blocks::before_after(view, "12 s to render", "1.2 s to render").await"#,
        source: include_str!("../templates/blocks/before_after.rs"),
    },
    Block {
        name: "timeline",
        usage: r#"blocks::timeline(view, &[("2019", "First commit"), ("2026", "1.0")]).await"#,
        source: include_str!("../templates/blocks/timeline.rs"),
    },
];

/// `quote-card` and `quote_card` both name `quote_card`.
pub fn find(name: &str) -> Option<&'static Block> {
    let name = name.replace('-', "_");
    BLOCKS.iter().find(|b| b.name == name)
}

pub fn run(block: Option<String>, force: bool, json_mode: bool) -> crate::reply::Done {
    let Some(name) = block else {
        return catalog(json_mode);
    };
    let Some(block) = find(&name) else {
        let names: Vec<_> = BLOCKS.iter().map(|b| b.name.replace('_', "-")).collect();
        return crate::reply::fail(
            json_mode,
            exit::BUILD,
            format!("no block `{name}`; the blocks are: {}", names.join(", ")),
        );
    };
    let root = match crate::config::project_root() {
        Ok(r) => r,
        Err(e) => return crate::reply::fail(json_mode, exit::ENV, e),
    };
    match add(&root, block, force) {
        Ok(wrote) => {
            if !json_mode {
                crate::note(format!("added src/blocks/{}.rs", block.name));
                println!("Call it in a scene:\n  {}", block.usage);
                if !wrote.main {
                    println!(
                        "Add `mod blocks;` next to the other `mod` lines of your crate root to use it."
                    );
                }
            }
            crate::reply::done(
                json_mode,
                exit::OK,
                json!({ "block": block.name, "file": format!("src/blocks/{}.rs", block.name), "usage": block.usage }),
            )
        }
        Err(e) => crate::reply::fail(json_mode, exit::ENV, e),
    }
}

fn catalog(json_mode: bool) -> crate::reply::Done {
    if json_mode {
        let list: Vec<Value> = BLOCKS
            .iter()
            .map(|b| json!({ "name": b.name, "summary": b.summary(), "usage": b.usage }))
            .collect();
        return crate::reply::Done::with(exit::OK, json!({ "blocks": list }));
    }
    println!("Blocks for `oxframe add <name>`:\n");
    for b in BLOCKS {
        println!("  {:<14} {}", b.name.replace('_', "-"), b.summary());
        println!("  {:<14} {}\n", "", b.usage);
    }
    crate::reply::Done::plain(exit::OK)
}

struct Wrote {
    /// `mod blocks;` is (now) in src/main.rs.
    main: bool,
}

fn add(root: &Path, block: &Block, force: bool) -> anyhow::Result<Wrote> {
    let dir = root.join("src/blocks");
    std::fs::create_dir_all(&dir)?;
    let file = dir.join(format!("{}.rs", block.name));
    if file.exists() && !force {
        anyhow::bail!(
            "src/blocks/{}.rs exists; pass --force to replace it",
            block.name
        );
    }
    std::fs::write(&file, block.source)?;
    let module = dir.join("mod.rs");
    let current = std::fs::read_to_string(&module).unwrap_or_default();
    std::fs::write(&module, register(&current, block.name))?;
    let main = root.join("src/main.rs");
    let wrote_main = match std::fs::read_to_string(&main) {
        Ok(text) => {
            std::fs::write(&main, with_mod_blocks(&text))?;
            true
        }
        Err(_) => false,
    };
    Ok(Wrote { main: wrote_main })
}

/// A block you have added but not called yet is not a warning.
const ALLOW: &str = "#![allow(dead_code)]";
const MOD_HEADER: &str = "//! Blocks added with `oxframe add`. Each is an `async fn` that builds its\n//! nodes in a group, plays them in and returns the group.\n";

/// `blocks/mod.rs` with `pub mod name;` and `pub use name::name;`, each
/// list kept sorted and without duplicates.
pub fn register(current: &str, name: &str) -> String {
    let mut mods: Vec<String> = Vec::new();
    let mut uses: Vec<String> = Vec::new();
    let mut other: Vec<String> = Vec::new();
    for line in current.lines() {
        let t = line.trim();
        if t.starts_with("pub mod ") {
            mods.push(t.to_owned());
        } else if t.starts_with("pub use ") {
            uses.push(t.to_owned());
        } else if !t.is_empty() && !t.starts_with("//!") && t != ALLOW {
            other.push(line.to_owned());
        }
    }
    mods.push(format!("pub mod {name};"));
    uses.push(format!("pub use {name}::{name};"));
    for list in [&mut mods, &mut uses] {
        list.sort();
        list.dedup();
    }
    let mut out = String::from(MOD_HEADER);
    out.push_str(ALLOW);
    out.push_str("\n\n");
    for m in &mods {
        out.push_str(m);
        out.push('\n');
    }
    out.push('\n');
    for u in &uses {
        out.push_str(u);
        out.push('\n');
    }
    if !other.is_empty() {
        out.push('\n');
        for o in &other {
            out.push_str(o);
            out.push('\n');
        }
    }
    out
}

/// `main.rs` with `mod blocks;` after its leading `use` lines, unless it
/// already declares it.
pub fn with_mod_blocks(main: &str) -> String {
    if main
        .lines()
        .any(|l| matches!(l.trim(), "mod blocks;" | "pub mod blocks;"))
    {
        return main.to_owned();
    }
    let lines: Vec<&str> = main.lines().collect();
    let mut at = 0;
    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        if t.starts_with("use ") || t.starts_with("//!") || t.starts_with("#![") {
            at = i + 1;
        } else if !t.is_empty() {
            break;
        }
    }
    let mut out: Vec<String> = lines.iter().map(|l| (*l).to_owned()).collect();
    out.insert(at, String::new());
    out.insert(at + 1, "mod blocks;".to_owned());
    let mut text = out.join("\n");
    if main.ends_with('\n') {
        text.push('\n');
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_block_has_a_summary_and_matching_function() {
        for b in BLOCKS {
            assert!(!b.summary().is_empty(), "{}", b.name);
            assert!(
                b.source.contains(&format!("pub async fn {}(", b.name)),
                "{}",
                b.name
            );
            assert!(b.usage.starts_with(&format!("blocks::{}(", b.name)));
        }
        assert_eq!(find("quote-card").map(|b| b.name), Some("quote_card"));
        assert!(find("nope").is_none());
    }

    #[test]
    fn registering_keeps_mod_rs_sorted_and_unique() {
        let one = register("", "timeline");
        let two = register(&one, "quote_card");
        let again = register(&two, "timeline");
        assert_eq!(two, again);
        assert_eq!(two.matches(ALLOW).count(), 1);
        assert!(two.contains("pub mod quote_card;\npub mod timeline;\n"));
        assert!(two.contains("pub use quote_card::quote_card;\npub use timeline::timeline;\n"));
    }

    #[test]
    fn mod_blocks_goes_after_the_use_lines_once() {
        let main = "use oxframe::prelude::*;\n\n#[oxframe::scene]\nasync fn intro(view: View) {}\n";
        let once = with_mod_blocks(main);
        assert_eq!(
            once,
            "use oxframe::prelude::*;\n\nmod blocks;\n\n#[oxframe::scene]\nasync fn intro(view: View) {}\n"
        );
        assert_eq!(with_mod_blocks(&once), once);
    }
}
