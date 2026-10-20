//! Fixes for the compile and setup errors agents hit most often.
//!
//! The ten covered mistakes, in order: a forgotten `.await`, a type mismatch,
//! a method that belongs to a builder or a handle, a missing type annotation,
//! a builder never passed to `add`, a name that is not in the prelude, the
//! wrong number of arguments, a builder used after `add` moved it, `all`
//! called without its tuple, and a text tween given an `&str`.

use serde::Serialize;

/// One suggested fix, attached to a compiler or runtime message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Hint {
    /// A rustc error code such as `E0308`, or empty for a runtime message.
    pub code: String,
    /// The compiler or runtime message this hint answers.
    pub message: String,
    /// What to change.
    pub hint: String,
}

const AWAIT: &str =
    "This animation is a future. Add `.await`, or pass it to `all`, `sequence` or `spawn`.";
const TYPES: &str = "A type does not match. Theme tokens are not numbers: `Accent.into()` where a `Color` is required, `pct(80)` for a percent, and `0.0f64` when a float is ambiguous.";
const METHOD: &str = "That method lives on a builder or a handle. `use oxframe::prelude::*`. `x`, `fill` and `next_to` exist on the handle that `view.add(...)` returns, and only while the scene is running.";
const ANNOTATION: &str = "Add a type the compiler can see: `Signal::new(0.0f64)`, or `\"Hello\".into()` when a text tween wants a `String`.";
const ADD: &str = "A builder does nothing until you pass it to `view.add(...)`. Keep the handle that `add` returns.";
const SCOPE: &str = "That name is not in scope. `use oxframe::prelude::*` brings in the nodes, `pct`, `Side`, `Edge` and the explainer components.";
const ARITY: &str = "The number of arguments does not match. `.to(target, seconds)` takes both, and `all((a, b))` needs the extra parentheses around the tuple.";
const MOVED: &str = "A builder is moved into `view.add`. Use the handle `add` returns; do not use the builder again.";
const ALL: &str = "`all` takes one tuple: `all((a, b)).await`. Two arguments without the inner parentheses do not compile.";
const TEXT: &str = "A text tween takes a `String`: `.text().to(\"new text\".into(), 1.0)`.";
const SCENE: &str = "Node handles only work inside a running scene. Build the node with `view.add(...)` in the scene function, then animate the handle it returns.";
const FFMPEG: &str = "Install ffmpeg, or render a PNG sequence with `--format png`, which does not need it. The path can also be set as `ffmpeg` under `[render]` in oxframe.toml.";
const PROTOCOL: &str = "The CLI, the SDK and the editor must be the same release. Run `oxframe upgrade`, then install a matching `oxframe-cli`.";

/// A hint for this compiler diagnostic, when it is one of the known mistakes.
pub fn attach(code: Option<&str>, message: &str) -> Option<Hint> {
    let hint = explain(code.unwrap_or(""), message)?;
    Some(Hint {
        code: code.unwrap_or("").to_owned(),
        message: message.to_owned(),
        hint: hint.to_owned(),
    })
}

/// Hints found in a blob of compiler or runtime output. Duplicates are dropped.
pub fn from_text(text: &str) -> Vec<Hint> {
    let mut out = Vec::new();
    for chunk in text.lines() {
        if let Some(hint) = attach(None, chunk) {
            push_unique(&mut out, hint);
        }
    }
    out
}

/// Prints each hint once, for a human-readable command.
pub fn show(hints: &[Hint]) {
    let mut seen = Vec::new();
    for hint in hints {
        if seen.contains(&hint.hint) {
            continue;
        }
        seen.push(hint.hint.clone());
        crate::note(format!("hint: {}", hint.hint));
    }
}

fn push_unique(out: &mut Vec<Hint>, hint: Hint) {
    if !out.iter().any(|e| e.hint == hint.hint) {
        out.push(hint);
    }
}

fn explain(code: &str, message: &str) -> Option<&'static str> {
    let m = message.to_ascii_lowercase();
    if m.contains("must be awaited")
        || (m.contains("unused") && m.contains("future"))
        || (code == "E0277" && (m.contains("future") || m.contains("await")))
    {
        return Some(AWAIT);
    }
    if m.contains("does nothing until") || (m.contains("unused") && m.contains("builder")) {
        return Some(ADD);
    }
    if m.contains("all") && (m.contains("tuple") || m.contains("1 argument but")) {
        return Some(ALL);
    }
    if (m.contains("expected struct `string`") || m.contains("expected string,"))
        && m.contains("str")
    {
        return Some(TEXT);
    }
    if code == "E0308" || m.contains("mismatched types") {
        return Some(TYPES);
    }
    if code == "E0599" || m.contains("no method named") {
        return Some(METHOD);
    }
    if code == "E0283" || m.contains("type annotations needed") {
        return Some(ANNOTATION);
    }
    if code == "E0425" || m.contains("cannot find") {
        return Some(SCOPE);
    }
    if code == "E0061" || m.contains("this function takes") || m.contains("this method takes") {
        return Some(ARITY);
    }
    if code == "E0382" || m.contains("use of moved value") {
        return Some(MOVED);
    }
    if m.contains("node handles can only be used while a scene is running") {
        return Some(SCENE);
    }
    if m.contains("ffmpeg") && m.contains("not found") {
        return Some(FFMPEG);
    }
    if m.contains("protocol versions differ") {
        return Some(PROTOCOL);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_the_ten_agent_mistakes() {
        let found = [
            attach(Some("E0277"), "the trait `Future` is not implemented"),
            attach(Some("E0308"), "mismatched types"),
            attach(Some("E0599"), "no method named `next_to` found"),
            attach(Some("E0283"), "type annotations needed"),
            attach(
                None,
                "unused builder that does nothing until it is passed to add",
            ),
            attach(Some("E0425"), "cannot find type `TitleCard` in this scope"),
            attach(
                Some("E0061"),
                "this function takes 2 arguments but 1 argument was supplied",
            ),
            attach(Some("E0382"), "use of moved value: `card`"),
            attach(
                None,
                "this function takes 1 argument but 2 arguments were supplied to `all`",
            ),
            attach(Some("E0308"), "expected struct `String`, found `&str`"),
        ];
        let needles = [
            "Add `.await`",
            "Accent.into()",
            "view.add",
            "0.0f64",
            "view.add",
            "prelude",
            "extra parentheses",
            "handle",
            "all((a, b))",
            "\".into()",
        ];
        for (hint, needle) in found.iter().zip(needles) {
            let hint = hint
                .as_ref()
                .unwrap_or_else(|| panic!("no hint for {needle}"));
            assert!(hint.hint.contains(needle), "{} missing {needle}", hint.hint);
        }
    }

    #[test]
    fn reads_a_runtime_message() {
        let hints = from_text("oxframe: node handles can only be used while a scene is running");
        assert_eq!(hints.len(), 1);
        assert!(hints[0].hint.contains("running scene"));
    }
}
