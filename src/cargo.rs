//! Running cargo and reading its JSON messages.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use oxframe_protocol::Span;
use serde::Deserialize;

pub struct BuildOutput {
    pub success: bool,
    pub executable: Option<PathBuf>,
    /// Plain-text compiler output (errors and warnings), for the editor.
    pub messages: String,
    pub spans: Vec<Span>,
}

#[derive(Deserialize)]
struct Msg {
    reason: String,
    #[serde(default)]
    message: Option<Diagnostic>,
    #[serde(default)]
    executable: Option<PathBuf>,
    #[serde(default)]
    manifest_path: Option<PathBuf>,
    #[serde(default)]
    target: Option<Target>,
    #[serde(default)]
    success: Option<bool>,
}

#[derive(Deserialize)]
struct Target {
    name: String,
    kind: Vec<String>,
}

#[derive(Deserialize)]
struct Diagnostic {
    level: String,
    #[serde(default)]
    rendered: Option<String>,
    #[serde(default)]
    spans: Vec<DiagSpan>,
    message: String,
}

#[derive(Deserialize)]
struct DiagSpan {
    file_name: String,
    line_start: u32,
    column_start: u32,
    is_primary: bool,
}

/// `cargo build` in `root`, streaming colored diagnostics to the terminal.
pub fn build(root: &Path, release: bool, bin: Option<&str>) -> std::io::Result<BuildOutput> {
    let mut cmd = Command::new(cargo());
    cmd.current_dir(root)
        .args(["build", "--message-format=json-diagnostic-rendered-ansi"])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    if release {
        cmd.arg("--release");
    }
    if let Some(b) = bin {
        cmd.args(["--bin", b]);
    }
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("piped");
    let manifest = root
        .join("Cargo.toml")
        .canonicalize()
        .unwrap_or_else(|_| root.join("Cargo.toml"));
    let mut out = BuildOutput {
        success: false,
        executable: None,
        messages: String::new(),
        spans: Vec::new(),
    };
    for line in BufReader::new(stdout).lines() {
        let line = line?;
        let Ok(msg) = serde_json::from_str::<Msg>(&line) else {
            continue;
        };
        match msg.reason.as_str() {
            "compiler-message" => {
                let Some(d) = msg.message else { continue };
                if let Some(r) = &d.rendered {
                    anstream::eprint!("{r}");
                    if d.level == "error" || d.level == "warning" {
                        out.messages.push_str(&strip_ansi(r));
                    }
                }
                if d.level == "error" {
                    for s in d.spans.iter().filter(|s| s.is_primary) {
                        out.spans.push(Span {
                            file: s.file_name.clone(),
                            line: s.line_start,
                            column: s.column_start,
                            message: d.message.clone(),
                        });
                    }
                }
            }
            "compiler-artifact" => {
                let is_root = msg
                    .manifest_path
                    .as_ref()
                    .and_then(|p| p.canonicalize().ok())
                    .is_some_and(|p| p == manifest);
                let is_bin = msg.target.as_ref().is_some_and(|t| {
                    t.kind.iter().any(|k| k == "bin") && bin.is_none_or(|b| b == t.name)
                });
                if is_root && is_bin && msg.executable.is_some() {
                    out.executable = msg.executable;
                }
            }
            "build-finished" => out.success = msg.success.unwrap_or(false),
            _ => {}
        }
    }
    let status = child.wait()?;
    out.success &= status.success();
    Ok(out)
}

pub fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".into())
}

/// Removes ANSI escape sequences.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_ansi() {
        assert_eq!(
            strip_ansi("\u{1b}[1m\u{1b}[38;5;9merror\u{1b}[0m: x"),
            "error: x"
        );
    }
}
