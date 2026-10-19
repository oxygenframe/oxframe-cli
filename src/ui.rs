//! The embedded editor (`ui/dist`, see `ui.lock`), or a directory given with
//! `--ui-dir` when developing the editor itself.

use std::borrow::Cow;
use std::path::PathBuf;

use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "ui/dist/"]
struct Embedded;

#[derive(Clone)]
pub enum UiSource {
    Embedded,
    Dir(PathBuf),
}

impl UiSource {
    /// File contents and MIME type.
    pub fn get(&self, path: &str) -> Option<(Cow<'static, [u8]>, String)> {
        let path = path.trim_start_matches('/');
        let path = if path.is_empty() { "index.html" } else { path };
        let mime = mime_for(path);
        match self {
            Self::Embedded => Embedded::get(path).map(|f| (f.data, mime)),
            Self::Dir(dir) => {
                // No `..` escapes from the UI directory.
                if path.split('/').any(|c| c == "..") {
                    return None;
                }
                std::fs::read(dir.join(path))
                    .ok()
                    .map(|d| (Cow::Owned(d), mime))
            }
        }
    }
}

fn mime_for(path: &str) -> String {
    match path.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8".into(),
        Some("js" | "mjs") => "text/javascript; charset=utf-8".into(),
        Some("css") => "text/css; charset=utf-8".into(),
        Some("json") => "application/json".into(),
        Some("svg") => "image/svg+xml".into(),
        Some("png") => "image/png".into(),
        Some("woff2") => "font/woff2".into(),
        Some("wasm") => "application/wasm".into(),
        _ => "application/octet-stream".into(),
    }
}

/// `protocol_version` the embedded editor was built for (from its
/// `manifest.json`), if present.
pub fn embedded_protocol_version() -> Option<u32> {
    let f = Embedded::get("manifest.json")?;
    let v: serde_json::Value = serde_json::from_slice(&f.data).ok()?;
    v.get("protocol_version")?.as_u64().map(|v| v as u32)
}

pub fn embedded_version() -> Option<String> {
    let f = Embedded::get("manifest.json")?;
    let v: serde_json::Value = serde_json::from_slice(&f.data).ok()?;
    v.get("version")?.as_str().map(str::to_owned)
}

#[cfg(test)]
mod tests {
    /// The embedded bundle must be the one pinned in ui.lock.
    #[test]
    fn ui_lock_matches_embedded_bundle() {
        let lock = include_str!("../ui.lock");
        let pinned = lock
            .lines()
            .find_map(|l| l.strip_prefix("version = "))
            .map(|v| v.trim_matches('"'))
            .expect("ui.lock has a version");
        let embedded =
            super::embedded_version().expect("ui/dist has manifest.json (run scripts/fetch-ui.sh)");
        assert_eq!(embedded, pinned, "ui/dist does not match ui.lock");
        assert_eq!(
            super::embedded_protocol_version(),
            Some(oxframe_protocol::PROTOCOL_VERSION)
        );
    }
}
