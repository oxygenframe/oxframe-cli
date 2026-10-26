//! `oxframe mcp`: an MCP server on stdin/stdout.
//!
//! Speaks newline-delimited JSON-RPC, which Claude Code and the MCP SDK use,
//! and also the `Content-Length` framing of older clients. Logs stay on stderr
//! so the protocol owns stdout.

use std::io::{BufRead, Write};

use serde_json::{Value, json};

use crate::reply::Done;
use crate::{agents, check, exit, render, snapshot};

const PROTOCOL: &str = "2024-11-05";

/// Serves until stdin closes.
pub fn serve() -> Done {
    let stdin = std::io::stdin();
    let mut reader = std::io::BufReader::new(stdin.lock());
    while let Some(msg) = read_message(&mut reader) {
        if let Some(resp) = handle(&msg) {
            write_message(&resp);
        }
    }
    Done::printed(exit::OK)
}

/// One request in, a response out. Notifications return `None`.
pub fn handle(msg: &Value) -> Option<Value> {
    let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = msg.get("id").cloned()?;
    let params = msg.get("params").cloned().unwrap_or(json!({}));
    Some(match method {
        "initialize" => result(
            id,
            json!({
                "protocolVersion": PROTOCOL,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "oxframe", "version": env!("CARGO_PKG_VERSION") },
            }),
        ),
        "ping" => result(id, json!({})),
        "tools/list" => result(id, json!({ "tools": tools() })),
        "tools/call" => {
            let name = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let (text, is_error) = call_tool(name, &args);
            result(
                id,
                json!({
                    "content": [{ "type": "text", "text": text }],
                    "isError": is_error,
                }),
            )
        }
        _ => error(id, -32601, format!("method not found: {method}")),
    })
}

fn tools() -> Vec<Value> {
    vec![
        tool(
            "check",
            "Compile the project and run the visual check. Returns report.json, the storyboard path and hints for common mistakes.",
            json!({
                "type": "object",
                "properties": {
                    "frames": { "type": "boolean", "description": "Also write every sampled frame as its own PNG." },
                    "against": { "type": "string", "description": "An earlier check output made with frames (e.g. out/before): list the frames that changed and write diff images." },
                    "out": { "type": "string", "description": "Write the storyboard and report here instead of out/check (e.g. out/before, with frames, to compare against later)." }
                }
            }),
        ),
        tool(
            "storyboard",
            "Run the visual check and return the storyboard path plus a short summary of report.json.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "render_frame",
            "Render one frame as a PNG and return the path the project prints.",
            json!({
                "type": "object",
                "properties": {
                    "at": { "type": "number", "description": "Time in seconds." },
                    "scene": { "type": "string" },
                    "size": { "type": "string", "description": "WxH, default 960x540." }
                },
                "required": ["at"]
            }),
        ),
        tool(
            "inspect",
            "Scene tree at one time: kind, name, bounds, text and file:line. While a transition is playing, also kind, direction and progress.",
            json!({
                "type": "object",
                "properties": {
                    "at": { "type": "number" },
                    "scene": { "type": "string" }
                },
                "required": ["at"]
            }),
        ),
        tool(
            "render_graph",
            "Render passes at one time, in the order they run: vector passes, layers composited over the frame (opacity, blend, blur, clip), and effect passes with their shader (lens_drops reads the frame, ripple changes one node). Use it to see why a shader is skipped or a layer is costly.",
            json!({
                "type": "object",
                "properties": {
                    "at": { "type": "number", "description": "Time in seconds." },
                    "scene": { "type": "string" }
                },
                "required": ["at"]
            }),
        ),
        tool(
            "list_components",
            "The explainer components (TitleCard, BulletList, CodeExplainer, FlowDiagram, Callout, LowerThird, EndCard, ChapterMarker) and a one-line example of each.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "docs_search",
            "Search this version of the oxframe docs bundled with the CLI (AGENTS.md, the skill and the API cheat sheet).",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        ),
        tool(
            "render",
            "Release-build the project and render the video (or a PNG sequence, or a GIF for a README or a pull request).",
            json!({
                "type": "object",
                "properties": {
                    "scene": { "type": "string" },
                    "aspect": { "type": "string", "description": "16:9, 9:16, 1:1 or all." },
                    "format": { "type": "string", "description": "mp4 (default), webm, prores, png or gif." },
                    "fps": { "type": "integer", "description": "GIF frame rate (default 15)." },
                    "size": { "type": "string" },
                    "cpu": { "type": "boolean" },
                    "gpu": { "type": "boolean" }
                }
            }),
        ),
    ]
}

fn tool(name: &str, description: &str, schema: Value) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": schema,
    })
}

fn call_tool(name: &str, args: &Value) -> (String, bool) {
    match name {
        "check" => text_of(check::run(
            args["frames"].as_bool().unwrap_or(false),
            args["against"].as_str().map(std::path::PathBuf::from),
            args["out"].as_str().map(std::path::PathBuf::from),
            true,
        )),
        "storyboard" => {
            let done = check::run(false, None, None, true);
            let summary = storyboard_summary(&done.body);
            let mut body = done.body.clone();
            if let Some(obj) = body.as_object_mut() {
                obj.insert("summary".to_owned(), json!(summary));
            }
            (
                serde_json::to_string_pretty(&body).unwrap_or_default(),
                done.code != exit::OK,
            )
        }
        "render_frame" => {
            let at = args["at"].as_f64();
            if at.is_none() {
                return ("render_frame needs a number `at` (seconds).".into(), true);
            }
            text_of(snapshot::run(snapshot::SnapshotArgs {
                at: vec![at.unwrap()],
                out: None,
                scene: arg_string(args, "scene"),
                size: arg_string(args, "size"),
                release: false,
                json: true,
            }))
        }
        "inspect" => {
            let Some(at) = args["at"].as_f64() else {
                return ("inspect needs a number `at` (seconds).".into(), true);
            };
            text_of(check::inspect(at, arg_string(args, "scene"), false, true))
        }
        "render_graph" => {
            let Some(at) = args["at"].as_f64() else {
                return ("render_graph needs a number `at` (seconds).".into(), true);
            };
            let done = check::inspect(at, arg_string(args, "scene"), true, true);
            let failed = done.code != exit::OK;
            (
                serde_json::to_string_pretty(&passes_only(done.body)).unwrap_or_default(),
                failed,
            )
        }
        "list_components" => (COMPONENTS.into(), false),
        "docs_search" => {
            let query = args["query"].as_str().unwrap_or("").trim();
            if query.is_empty() {
                return ("docs_search needs a `query`.".into(), true);
            }
            (search_docs(query), false)
        }
        "render" => text_of(render::run(render::RenderArgs {
            out: None,
            scene: arg_string(args, "scene"),
            range: None,
            format: arg_string(args, "format"),
            size: arg_string(args, "size"),
            aspect: arg_string(args, "aspect"),
            cpu: args["cpu"].as_bool().unwrap_or(false),
            gpu: args["gpu"].as_bool().unwrap_or(false),
            no_audio: false,
            subtitles: None,
            slides: None,
            jobs: None,
            fps: args["fps"].as_u64().and_then(|n| u32::try_from(n).ok()),
            json: true,
        })),
        _ => (format!("unknown tool: {name}"), true),
    }
}

fn text_of(done: Done) -> (String, bool) {
    (
        serde_json::to_string_pretty(&done.body).unwrap_or_default(),
        done.code != exit::OK,
    )
}

fn arg_string(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

/// `inspect --graph` without the node tree: the time, the frame and the passes.
fn passes_only(mut body: Value) -> Value {
    if let Some(inspect) = body.get_mut("inspect").and_then(Value::as_object_mut) {
        inspect.retain(|k, _| matches!(k.as_str(), "time" | "frame" | "passes" | "transition"));
    }
    body
}

fn storyboard_summary(body: &Value) -> String {
    let path = body["storyboard"]
        .as_str()
        .unwrap_or("out/check/storyboard.png");
    let errors = body["report"]["errors"].as_u64();
    let warnings = body["report"]["warnings"].as_u64();
    match (errors, warnings) {
        (Some(e), Some(w)) => format!("{path}: {e} errors, {w} warnings"),
        _ => format!("{path}: report.json has no error counts yet"),
    }
}

const COMPONENTS: &str = "\
Explainer components, all in the prelude. Add them like any node. They start invisible and take their colors from the theme.

- TitleCard::new(\"The half\").subtitle(\"A worked example\") — opening card. next() reveals the subtitle.
- ChapterMarker::new(\"2  The search\") — chapter tab on the top safe edge.
- BulletList::new([\"First\", \"Second\"]) — rows stay hidden; each next() reveals one.
- CodeExplainer::new(\"fn main() {}\").step(1, 1, \"the whole program\") — a code card. next() moves to the following step (lines first..=last and a caption).
- FlowDiagram::new().edge(\"Parse\", \"Run\").edge(\"Run\", \"Print\") — boxes in layers from the edges. .down() stacks the layers.
- Callout::new(node).text(\"Watch the gap\") — a label with a line to a node.
- LowerThird::new(\"Ada\").role(\"walks the half\") — name plate on the bottom safe edge.
- EndCard::new(\"That's the search\").note(\"Repeat from the top\") — closing card.

Each has enter(), next() and exit(), with no arguments: card.enter().await, then card.next().await per beat, then card.exit().await.
";

fn search_docs(query: &str) -> String {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|t| t.to_ascii_lowercase())
        .filter(|t| t.len() > 1)
        .collect();
    let corpus = [
        ("AGENTS.md", agents::AGENTS_MD),
        ("reference.md", agents::REFERENCE_MD),
        ("SKILL.md", agents::SKILL_MD),
    ];
    let mut hits: Vec<(usize, &str, &str)> = Vec::new();
    for (name, text) in corpus {
        for para in text.split("\n\n") {
            let lower = para.to_ascii_lowercase();
            let score = terms.iter().filter(|t| lower.contains(t.as_str())).count();
            if score > 0 {
                hits.push((score, name, para.trim()));
            }
        }
    }
    if hits.is_empty() {
        return format!(
            "No match for {query:?} in the bundled docs. The same text is on https://oxframe.dev/llms-full.txt."
        );
    }
    hits.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(b.1)));
    hits.truncate(6);
    let mut out = format!(
        "Matches in the oxframe {} docs bundled with this CLI:\n\n",
        env!("CARGO_PKG_VERSION")
    );
    for (_, name, para) in hits {
        out.push_str(&format!("## {name}\n{para}\n\n"));
    }
    out
}

fn result(id: Value, value: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": value})
}

fn error(id: Value, code: i32, message: String) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn write_message(value: &Value) {
    let line = serde_json::to_string(value).unwrap_or_else(|_| "{}".into());
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

/// A JSON-RPC message: one JSON line, or an LSP-style `Content-Length` frame.
fn read_message(reader: &mut impl BufRead) -> Option<Value> {
    loop {
        let mut line = String::new();
        let n = reader.read_line(&mut line).ok()?;
        if n == 0 {
            return None;
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.to_ascii_lowercase().starts_with("content-length:") {
            let len: usize = trimmed.split(':').nth(1)?.trim().parse().ok()?;
            loop {
                let mut header = String::new();
                if reader.read_line(&mut header).ok()? == 0 {
                    return None;
                }
                if header.trim().is_empty() {
                    break;
                }
            }
            let mut buf = vec![0u8; len];
            reader.read_exact(&mut buf).ok()?;
            return serde_json::from_slice(&buf).ok();
        }
        return serde_json::from_str(trimmed).ok();
    }
}

#[cfg(test)]
mod tests {
    /// The MCP Registry entry (server.json) names this crate and version,
    /// and the README carries the name the registry checks on crates.io.
    #[test]
    fn registry_entry_matches_the_crate() {
        let server: serde_json::Value =
            serde_json::from_str(include_str!("../server.json")).expect("server.json is JSON");
        let version = env!("CARGO_PKG_VERSION");
        assert_eq!(server["version"], version);
        assert_eq!(server["packages"][0]["identifier"], env!("CARGO_PKG_NAME"));
        assert_eq!(server["packages"][0]["version"], version);
        let name = server["name"].as_str().unwrap();
        assert!(include_str!("../README.md").contains(&format!("mcp-name: {name}")));
    }

    use super::*;

    #[test]
    fn initialize_lists_tools_and_answers_two_of_them() {
        let init = handle(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": { "protocolVersion": PROTOCOL, "capabilities": {} }
        }))
        .unwrap();
        assert_eq!(init["id"], 1);
        assert!(init["result"]["capabilities"]["tools"].is_object());
        assert!(
            handle(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none()
        );

        let list = handle(&json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"})).unwrap();
        let names: Vec<&str> = list["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        for name in [
            "check",
            "storyboard",
            "render_frame",
            "inspect",
            "render_graph",
            "list_components",
            "docs_search",
            "render",
        ] {
            assert!(names.contains(&name), "{name} missing from {names:?}");
        }

        let listed = handle(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "list_components", "arguments": {} }
        }))
        .unwrap();
        let text = listed["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("BulletList"));
        assert_eq!(listed["result"]["isError"], false);

        let found = handle(&json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": { "name": "docs_search", "arguments": { "query": "next_to" } }
        }))
        .unwrap();
        assert!(
            found["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("next_to")
        );
    }

    #[test]
    fn inspect_and_check_reject_a_bad_call() {
        let inspect = handle(&json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": { "name": "inspect", "arguments": {} }
        }))
        .unwrap();
        assert_eq!(inspect["result"]["isError"], true);
        assert!(
            inspect["result"]["content"][0]["text"]
                .as_str()
                .unwrap()
                .contains("`at`")
        );
        let frame = handle(&json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": { "name": "render_frame", "arguments": {} }
        }))
        .unwrap();
        assert_eq!(frame["result"]["isError"], true);
        let graph = handle(&json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": { "name": "render_graph", "arguments": {} }
        }))
        .unwrap();
        assert_eq!(graph["result"]["isError"], true);
    }

    #[test]
    fn render_graph_keeps_the_passes_and_drops_the_nodes() {
        let body = json!({
            "ok": true,
            "inspect": {
                "time": 2.5,
                "frame": 75,
                "nodes": [{ "id": 1 }],
                "passes": [{ "kind": "effect", "shader": "lens_drops" }],
                "size": [1920, 1080]
            }
        });
        let out = passes_only(body);
        assert_eq!(out["inspect"]["passes"][0]["shader"], "lens_drops");
        assert!(out["inspect"].get("nodes").is_none());
        assert_eq!(out["ok"], true);
    }

    #[test]
    fn reads_content_length_frames() {
        let body = r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#;
        let frame = format!("Content-Length: {}\r\n\r\n{body}", body.len());
        let mut reader = std::io::Cursor::new(frame.into_bytes());
        let msg = read_message(&mut reader).unwrap();
        let reply = handle(&msg).unwrap();
        assert_eq!(reply["id"], 7);
        assert!(reply["result"].is_object());
    }
}
