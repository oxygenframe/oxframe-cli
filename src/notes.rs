//! Review notes: what a person points at in the `oxframe dev` editor and
//! writes about, for the agent that changes the scene.
//!
//! They live in `out/notes/notes.json` (crops next to it as `<id>.png`).
//! `oxframe check` reports every open note as a `review_note` issue, first
//! in the list; `oxframe notes done <id>` closes one.

use std::path::{Path, PathBuf};

use base64::Engine as _;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::reply::Done;
use crate::{config, exit};

/// The issue rule a note is reported under in `check`.
pub const RULE: &str = "review_note";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    #[default]
    Open,
    Done,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Note {
    pub id: u32,
    /// What the person wrote.
    pub text: String,
    /// Seconds from the start of the project, `[from, to]`.
    pub time: [f64; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<String>,
    /// The node pointed at: its name, or its kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    /// `file:line` of that node's `add`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// The marked area `[x0, y0, x1, y1]` in scene units (origin at the
    /// frame center, y down), as `bounds` of a `check` issue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<[f64; 4]>,
    /// The frame around the area, `<id>.png` in the notes directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<String>,
    #[serde(default)]
    pub status: Status,
    /// What the agent says it changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply: Option<String>,
    /// Unix seconds.
    #[serde(default)]
    pub created: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Notes {
    pub notes: Vec<Note>,
}

pub fn dir(root: &Path) -> PathBuf {
    root.join("out/notes")
}

pub fn load(root: &Path) -> Notes {
    std::fs::read_to_string(dir(root).join("notes.json"))
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

/// Written whole (temp file and rename): the editor and `oxframe notes`
/// may both write, and the editor reloads the file when it changes.
pub fn save(root: &Path, notes: &Notes) -> std::io::Result<()> {
    let dir = dir(root);
    std::fs::create_dir_all(&dir)?;
    let tmp = dir.join(format!("notes.json.{}.tmp", std::process::id()));
    std::fs::write(&tmp, serde_json::to_string_pretty(notes)?)?;
    std::fs::rename(&tmp, dir.join("notes.json"))
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// A message from the editor about notes; `None` for any other message.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum Edit {
    #[serde(rename = "NoteAdd")]
    Add {
        text: String,
        time: [f64; 2],
        #[serde(default)]
        scene: Option<String>,
        #[serde(default)]
        node: Option<String>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        bounds: Option<[f64; 4]>,
        /// PNG of the area, base64.
        #[serde(default)]
        crop_png: Option<String>,
    },
    #[serde(rename = "NoteSet")]
    Set {
        id: u32,
        #[serde(default)]
        status: Option<Status>,
        #[serde(default)]
        text: Option<String>,
    },
    #[serde(rename = "NoteDelete")]
    Delete { id: u32 },
}

impl Edit {
    pub fn parse(text: &str) -> Option<Self> {
        if !text.contains("\"Note") {
            return None;
        }
        serde_json::from_str(text).ok()
    }
}

/// Applies an editor's edit to the notes on disk.
pub fn apply(root: &Path, edit: Edit) -> std::io::Result<()> {
    let mut notes = load(root);
    match edit {
        Edit::Add {
            text,
            time,
            scene,
            node,
            source,
            bounds,
            crop_png,
        } => {
            let id = notes.notes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
            let crop = crop_png.and_then(|b64| {
                let png = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
                // Only a PNG, and not a huge one.
                if !png.starts_with(b"\x89PNG") || png.len() > 8 << 20 {
                    return None;
                }
                let name = format!("{id}.png");
                std::fs::create_dir_all(dir(root)).ok()?;
                std::fs::write(dir(root).join(&name), png).ok()?;
                Some(name)
            });
            let (a, b) = (time[0].max(0.0), time[1].max(0.0));
            notes.notes.push(Note {
                id,
                text: text.trim().to_owned(),
                time: [a.min(b), a.max(b)],
                scene,
                node,
                source,
                bounds,
                crop,
                status: Status::Open,
                reply: None,
                created: now(),
            });
        }
        Edit::Set { id, status, text } => {
            if let Some(n) = notes.notes.iter_mut().find(|n| n.id == id) {
                if let Some(s) = status {
                    n.status = s;
                }
                if let Some(t) = text {
                    n.text = t.trim().to_owned();
                }
            }
        }
        Edit::Delete { id } => {
            if let Some(crop) = notes
                .notes
                .iter()
                .find(|n| n.id == id)
                .and_then(|n| n.crop.clone())
            {
                let _ = std::fs::remove_file(dir(root).join(crop));
            }
            notes.notes.retain(|n| n.id != id);
        }
    }
    save(root, &notes)
}

/// The `Notes` message for the editor: every note, with crops as URLs the
/// dev server answers.
pub fn message(root: &Path) -> String {
    json!({ "type": "Notes", "notes": load(root).notes }).to_string()
}

/// Open notes as `check` issues, with crops copied into the check output.
pub fn issues(root: &Path, check_out: &Path) -> Vec<Value> {
    let mut out = Vec::new();
    for n in load(root).notes {
        if n.status != Status::Open {
            continue;
        }
        let crop = n.crop.as_ref().and_then(|c| {
            let rel = format!("notes/{c}");
            std::fs::create_dir_all(check_out.join("notes")).ok()?;
            std::fs::copy(dir(root).join(c), check_out.join(&rel)).ok()?;
            Some(rel)
        });
        let mut issue = json!({
            "rule": RULE,
            "severity": "error",
            "time": n.time,
            "node": n.node.clone().unwrap_or_else(|| "the marked area".into()),
            "hint": format!(
                "The reviewer wrote: \"{}\". Change the scene so this is no longer true, \
                 then `oxframe notes done {} --reply \"what you changed\"`.",
                n.text, n.id
            ),
            "note": n.id,
            "note_text": n.text,
            "docs": "https://oxframe.dev/docs/editor#review-notes",
        });
        if let Some(s) = &n.source {
            issue["source"] = json!(s);
        }
        if let Some(b) = n.bounds {
            issue["bounds"] = json!(b);
        }
        if let Some(c) = crop {
            issue["crop"] = json!(c);
        }
        out.push(issue);
    }
    out
}

/// `oxframe notes [list|done|reopen|delete]`.
pub fn run(action: Action, json: bool) -> Done {
    let root = match config::project_root() {
        Ok(r) => r,
        Err(e) => return crate::reply::fail(json, exit::ENV, e),
    };
    let mut notes = load(&root);
    let find = |notes: &mut Notes, id: u32| notes.notes.iter().position(|n| n.id == id);
    let closing = matches!(action, Action::Done { .. });
    match action {
        Action::List { all } => {
            let shown: Vec<&Note> = notes
                .notes
                .iter()
                .filter(|n| all || n.status == Status::Open)
                .collect();
            if json {
                return crate::reply::done(json, exit::OK, json!({ "notes": shown }));
            }
            if shown.is_empty() {
                crate::note("no open review notes");
            }
            for n in shown {
                let at = if (n.time[1] - n.time[0]).abs() < 1e-6 {
                    format!("{:.2}s", n.time[0])
                } else {
                    format!("{:.2}–{:.2}s", n.time[0], n.time[1])
                };
                let what = n.node.as_deref().unwrap_or("area");
                let src = n
                    .source
                    .as_deref()
                    .map_or(String::new(), |s| format!(" ({s})"));
                let done = if n.status == Status::Done {
                    " [done]"
                } else {
                    ""
                };
                println!("#{} at {at}, {what}{src}{done}: {}", n.id, n.text);
                if let Some(r) = &n.reply {
                    println!("    reply: {r}");
                }
            }
            Done::plain(exit::OK)
        }
        Action::Done { id, reply } | Action::Reopen { id, reply } => {
            let (status, verb) = if closing {
                (Status::Done, "closed")
            } else {
                (Status::Open, "reopened")
            };
            let Some(i) = find(&mut notes, id) else {
                return crate::reply::fail(json, exit::ENV, format!("no note #{id}"));
            };
            notes.notes[i].status = status;
            if reply.is_some() {
                notes.notes[i].reply = reply;
            }
            finish(&root, &notes, json, format!("{verb} note #{id}"))
        }
        Action::Delete { id } => {
            let Some(i) = find(&mut notes, id) else {
                return crate::reply::fail(json, exit::ENV, format!("no note #{id}"));
            };
            let n = notes.notes.remove(i);
            if let Some(c) = n.crop {
                let _ = std::fs::remove_file(dir(&root).join(c));
            }
            finish(&root, &notes, json, format!("deleted note #{id}"))
        }
    }
}

fn finish(root: &Path, notes: &Notes, json: bool, msg: String) -> Done {
    if let Err(e) = save(root, notes) {
        return crate::reply::fail(json, exit::ENV, format!("cannot write the notes: {e}"));
    }
    if json {
        crate::reply::done(json, exit::OK, json!({ "message": msg }))
    } else {
        crate::note(msg);
        Done::plain(exit::OK)
    }
}

#[derive(clap::Subcommand, Debug)]
pub enum Action {
    /// List the open notes (all with --all).
    List {
        #[arg(long)]
        all: bool,
    },
    /// Close a note once the scene is changed.
    Done {
        id: u32,
        /// What you changed; the editor shows it under the note.
        #[arg(long)]
        reply: Option<String>,
    },
    /// Open a closed note again.
    Reopen {
        id: u32,
        #[arg(long)]
        reply: Option<String>,
    },
    /// Remove a note.
    Delete { id: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\nfake";

    fn add(root: &Path, text: &str, crop: Option<&[u8]>) {
        let msg = json!({
            "type": "NoteAdd",
            "text": text,
            "time": [3.0, 1.5],
            "node": "title",
            "source": "src/main.rs:12",
            "bounds": [-100.0, -50.0, 100.0, 50.0],
            "crop_png": crop.map(|c| base64::engine::general_purpose::STANDARD.encode(c)),
        })
        .to_string();
        apply(root, Edit::parse(&msg).expect("a note edit")).unwrap();
    }

    #[test]
    fn engine_messages_are_not_note_edits() {
        assert!(Edit::parse(r#"{"type":"Seek","frame":3}"#).is_none());
        assert!(
            Edit::parse(r#"{"type":"Hello","protocol_version":1,"ui_version":"Note"}"#).is_none()
        );
    }

    #[test]
    fn notes_round_trip_and_open_ones_become_issues() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        add(root, " the title is too fast ", Some(PNG));
        add(root, "not a png crop", Some(b"GIF89a"));
        let notes = load(root).notes;
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].text, "the title is too fast");
        assert_eq!(notes[0].time, [1.5, 3.0], "time is ordered");
        assert_eq!(notes[0].crop.as_deref(), Some("1.png"));
        assert_eq!(notes[1].crop, None, "only PNGs are kept");

        let out = root.join("out/check");
        let issues = issues(root, &out);
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0]["rule"], RULE);
        assert_eq!(issues[0]["severity"], "error");
        assert_eq!(issues[0]["source"], "src/main.rs:12");
        assert_eq!(issues[0]["crop"], "notes/1.png");
        assert!(out.join("notes/1.png").is_file());
        assert!(
            issues[0]["hint"]
                .as_str()
                .unwrap()
                .contains("oxframe notes done 1")
        );

        let set = r#"{"type":"NoteSet","id":1,"status":"done"}"#;
        apply(root, Edit::parse(set).unwrap()).unwrap();
        assert_eq!(
            super::issues(root, &out).len(),
            1,
            "a closed note is no issue"
        );

        apply(
            root,
            Edit::parse(r#"{"type":"NoteDelete","id":1}"#).unwrap(),
        )
        .unwrap();
        assert_eq!(load(root).notes.len(), 1);
        assert!(!dir_crop(root, "1.png").exists(), "its crop goes too");
        add(root, "third", None);
        assert_eq!(load(root).notes.last().unwrap().id, 3, "ids are not reused");
    }

    fn dir_crop(root: &Path, name: &str) -> PathBuf {
        super::dir(root).join(name)
    }
}
