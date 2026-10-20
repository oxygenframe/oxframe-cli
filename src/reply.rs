//! The `--json` envelope: one object on stdout, nothing else.

use std::process::Command;

use serde_json::{Value, json};

/// What a command finished with. `body` is the `--json` object.
pub struct Done {
    pub code: u8,
    /// JSON object for `--json`. Null when the command talked to a human.
    pub body: Value,
    /// The command already wrote its JSON (the long-running dev server, MCP).
    pub printed: bool,
}

impl Done {
    pub fn plain(code: u8) -> Self {
        Self {
            code,
            body: Value::Null,
            printed: false,
        }
    }

    /// An envelope with `ok` and `exit` filled in.
    pub fn with(code: u8, mut body: Value) -> Self {
        if let Some(obj) = body.as_object_mut() {
            obj.entry("ok".to_owned())
                .or_insert(json!(code == crate::exit::OK));
            obj.entry("exit".to_owned()).or_insert(json!(code));
        }
        Self {
            code,
            body,
            printed: false,
        }
    }

    pub fn printed(code: u8) -> Self {
        Self {
            code,
            body: Value::Null,
            printed: true,
        }
    }
}

/// Writes the envelope. Used only when `--json` is set.
pub fn emit(done: &Done) {
    let body = if done.body.is_null() {
        json!({"ok": done.code == crate::exit::OK, "exit": done.code})
    } else {
        done.body.clone()
    };
    println!(
        "{}",
        serde_json::to_string(&body).unwrap_or_else(|_| "{}".into())
    );
}

/// Runs `cmd`. In JSON mode its output is captured; otherwise it inherits the terminal.
pub fn capture(cmd: &mut Command, json_mode: bool) -> std::io::Result<(u8, String, String)> {
    if json_mode {
        let out = cmd.output()?;
        let code = out
            .status
            .code()
            .map_or(crate::exit::BUILD, |c| c.clamp(0, 255) as u8);
        Ok((
            code,
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        ))
    } else {
        let status = cmd.status()?;
        let code = status
            .code()
            .map_or(crate::exit::BUILD, |c| c.clamp(0, 255) as u8);
        Ok((code, String::new(), String::new()))
    }
}

/// A JSON body when `json_mode`, otherwise a plain exit code.
pub fn done(json_mode: bool, code: u8, body: Value) -> Done {
    if json_mode {
        Done::with(code, body)
    } else {
        Done::plain(code)
    }
}

/// `Done` for `json` mode, or a plain code after printing `error` for a human.
pub fn fail(json_mode: bool, code: u8, error: impl std::fmt::Display) -> Done {
    let error = error.to_string();
    if json_mode {
        Done::with(code, json!({ "error": error }))
    } else {
        crate::error(error);
        Done::plain(code)
    }
}
