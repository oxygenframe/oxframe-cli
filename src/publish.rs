//! `oxframe publish`: render (or take an MP4), upload it to oxframe.dev and
//! print a link with a player that unfolds as a video in chats.
//!
//! The server takes MP4s of at most 16 MB and 30 s with a JPEG poster and
//! keeps them for 30 days (`/api/share`, the same endpoint as the
//! playground's "Share video"). A render that is too large is re-encoded at
//! 720p once. The upload goes through `curl`, which ships with Linux, macOS
//! and Windows 10+, so the CLI needs no HTTP client of its own.
//! `OXFRAME_PUBLISH_URL` points it at another server (tests, previews).

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;

use crate::reply::Done;
use crate::{config, exit};

const ENDPOINT: &str = "https://oxframe.dev/api/share";
const MAX_BYTES: u64 = 16 * 1024 * 1024;
const MAX_POSTER: u64 = 1024 * 1024;
const MAX_SECONDS: f64 = 30.0;
const MAX_TITLE: usize = 100;

pub struct PublishArgs {
    /// An MP4 to publish instead of rendering the project.
    pub file: Option<PathBuf>,
    pub scene: Option<String>,
    pub range: Option<String>,
    pub title: Option<String>,
    pub cpu: bool,
    pub gpu: bool,
    pub json: bool,
}

pub fn run(a: PublishArgs) -> Done {
    let title = a.title.as_deref().map(str::trim).unwrap_or_default();
    if title.chars().count() > MAX_TITLE {
        return crate::reply::fail(
            a.json,
            exit::ENV,
            format!("the title is longer than {MAX_TITLE} characters"),
        );
    }
    let root = config::project_root().ok();
    let cfg = root
        .as_deref()
        .and_then(|r| config::Config::load(r).ok())
        .unwrap_or_default();
    let Some(ffmpeg) = crate::render::find_ffmpeg(cfg.render.ffmpeg.as_deref()) else {
        return crate::reply::fail(a.json, exit::ENV, crate::render::FFMPEG_HELP);
    };

    let video = match &a.file {
        Some(f) => f.clone(),
        None => {
            let Some(root) = &root else {
                return crate::reply::fail(
                    a.json,
                    exit::ENV,
                    "no Cargo.toml here or above: run this in a project, or pass an MP4",
                );
            };
            let out = root.join("out/publish.mp4");
            let done = crate::render::run(crate::render::RenderArgs {
                out: Some(out.clone()),
                scene: a.scene.clone(),
                range: a.range.clone(),
                format: Some("mp4".into()),
                size: None,
                aspect: None,
                cpu: a.cpu,
                gpu: a.gpu,
                no_audio: false,
                subtitles: None,
                slides: None,
                jobs: None,
                json: a.json,
            });
            if done.code != exit::OK {
                return done;
            }
            out
        }
    };

    let bytes = match std::fs::read(&video) {
        Ok(b) => b,
        Err(e) => {
            return crate::reply::fail(a.json, exit::ENV, format!("{}: {e}", video.display()));
        }
    };
    let Some(seconds) = mp4_seconds(&bytes) else {
        return crate::reply::fail(
            a.json,
            exit::ENV,
            format!("{} is not an MP4", video.display()),
        );
    };
    if seconds > MAX_SECONDS + 0.5 {
        return crate::reply::fail(
            a.json,
            exit::ENV,
            format!(
                "the video is {seconds:.1} s long; oxframe.dev keeps videos up to {MAX_SECONDS} s. \
                 Publish part of it with --range, e.g. --range 0..30"
            ),
        );
    }

    // The poster and a re-encoded copy live in a scratch folder, removed on return.
    let scratch =
        Scratch(std::env::temp_dir().join(format!("oxframe-publish-{}", std::process::id())));
    if let Err(e) = std::fs::create_dir_all(&scratch.0) {
        return crate::reply::fail(a.json, exit::ENV, format!("{}: {e}", scratch.0.display()));
    }
    let dir = scratch.0.as_path();
    let mut upload = video.clone();
    if bytes.len() as u64 > MAX_BYTES {
        if !a.json {
            crate::note(format!(
                "{:.1} MB is over the 16 MB limit; re-encoding at 720p…",
                bytes.len() as f64 / 1e6
            ));
        }
        let small = dir.join("publish-720p.mp4");
        if let Err(e) = shrink(&ffmpeg, &video, &small) {
            return crate::reply::fail(a.json, exit::ENV, e);
        }
        let size = std::fs::metadata(&small).map_or(u64::MAX, |m| m.len());
        if size > MAX_BYTES {
            return crate::reply::fail(
                a.json,
                exit::ENV,
                format!(
                    "the video is {:.1} MB even at 720p; oxframe.dev keeps videos up to 16 MB. \
                     Publish a shorter part with --range",
                    size as f64 / 1e6
                ),
            );
        }
        upload = small;
    }

    let poster = dir.join("publish-poster.jpg");
    if let Err(e) = grab_poster(&ffmpeg, &upload, seconds / 2.0, &poster) {
        return crate::reply::fail(a.json, exit::ENV, e);
    }

    if !a.json {
        crate::note("uploading; anyone with the link can watch the video for 30 days…");
    }
    let endpoint = std::env::var("OXFRAME_PUBLISH_URL").unwrap_or_else(|_| ENDPOINT.into());
    let reply = match send(&endpoint, &upload, &poster, title) {
        Ok(r) => r,
        Err(e) => return crate::reply::fail(a.json, exit::ENV, e),
    };
    if a.json {
        Done::with(
            exit::OK,
            json!({ "url": reply.url, "id": reply.id, "seconds": seconds }),
        )
    } else {
        println!("{}", reply.url);
        Done::plain(exit::OK)
    }
}

struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Re-encodes to 720p H.264 at a quality that keeps 30 s well under 16 MB.
fn shrink(ffmpeg: &Path, from: &Path, to: &Path) -> Result<(), String> {
    let out = Command::new(ffmpeg)
        .args(["-y", "-v", "error", "-i"])
        .arg(from)
        .args([
            "-vf",
            "scale=-2:'min(720,ih)'",
            "-c:v",
            "libx264",
            "-preset",
            "slow",
            "-crf",
            "26",
            "-maxrate",
            "3M",
            "-bufsize",
            "6M",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-movflags",
            "+faststart",
        ])
        .arg(to)
        .output()
        .map_err(|e| format!("cannot run ffmpeg: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "ffmpeg could not re-encode the video:\n{}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// One frame at `at` seconds as a JPEG under the server's 1 MB limit.
fn grab_poster(ffmpeg: &Path, video: &Path, at: f64, to: &Path) -> Result<(), String> {
    for quality in ["3", "8"] {
        let out = Command::new(ffmpeg)
            .args(["-y", "-v", "error", "-ss", &format!("{at:.3}"), "-i"])
            .arg(video)
            .args([
                "-frames:v",
                "1",
                "-vf",
                "scale='min(1280,iw)':-2",
                "-q:v",
                quality,
            ])
            .arg(to)
            .output()
            .map_err(|e| format!("cannot run ffmpeg: {e}"))?;
        if !out.status.success() {
            return Err(format!(
                "ffmpeg could not take a poster frame:\n{}",
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        if std::fs::metadata(to).is_ok_and(|m| m.len() <= MAX_POSTER) {
            return Ok(());
        }
    }
    Err("the poster frame is larger than 1 MB".into())
}

struct Reply {
    id: String,
    url: String,
}

/// Posts the form with curl and reads the server's JSON answer.
fn send(endpoint: &str, video: &Path, poster: &Path, title: &str) -> Result<Reply, String> {
    let mut cmd = Command::new("curl");
    cmd.args(["-sS", "--max-time", "300", "-w", "\n%{http_code}"])
        .arg("-F")
        .arg(format!("video=@{};type=video/mp4", quoted(video)))
        .arg("-F")
        .arg(format!("poster=@{};type=image/jpeg", quoted(poster)));
    if !title.is_empty() {
        // --form-string: a title starting with @ or < is text, not a file.
        cmd.arg("--form-string").arg(format!("title={title}"));
    }
    let out = cmd.arg(endpoint).output().map_err(|e| {
        format!("cannot run curl ({e}); install curl, or upload the MP4 from the playground")
    })?;
    let text = String::from_utf8_lossy(&out.stdout);
    let (body, status) = text.rsplit_once('\n').unwrap_or(("", text.as_ref()));
    if !out.status.success() {
        return Err(format!(
            "upload failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let value: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    match (status, value["id"].as_str(), value["url"].as_str()) {
        ("201", Some(id), Some(url)) => Ok(Reply {
            id: id.to_owned(),
            url: url.to_owned(),
        }),
        _ => Err(match value["error"].as_str() {
            Some(e) => format!("{endpoint} refused the video: {e}"),
            None => format!("{endpoint} answered {status}: {}", body.trim()),
        }),
    }
}

/// A path for curl's `-F name=@"…"`, which would otherwise split at `;` and `,`.
fn quoted(path: &Path) -> String {
    let p = path.to_string_lossy();
    format!("\"{}\"", p.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The duration of an MP4 from its `mvhd` box; None if it is not an MP4.
pub fn mp4_seconds(buf: &[u8]) -> Option<f64> {
    let header = |at: usize| -> Option<(usize, [u8; 4], usize)> {
        let word = buf.get(at..at + 8)?;
        let mut size = u32::from_be_bytes(word[..4].try_into().ok()?) as usize;
        let kind: [u8; 4] = word[4..].try_into().ok()?;
        let mut head = 8;
        if size == 1 {
            size = u64::from_be_bytes(buf.get(at + 8..at + 16)?.try_into().ok()?) as usize;
            head = 16;
        } else if size == 0 {
            size = buf.len() - at;
        }
        (size >= head && at.checked_add(size)? <= buf.len()).then_some((size, kind, head))
    };
    let (_, first, _) = header(0)?;
    if &first != b"ftyp" {
        return None;
    }
    let mut at = 0;
    while let Some((size, kind, head)) = header(at) {
        if &kind == b"moov" {
            let mut inner = at + head;
            while let Some((isize, ikind, ihead)) = header(inner).filter(|_| inner < at + size) {
                if &ikind == b"mvhd" {
                    let b = buf.get(inner + ihead..inner + isize)?;
                    let read32 =
                        |o: usize| Some(u32::from_be_bytes(b.get(o..o + 4)?.try_into().ok()?));
                    let (scale, duration) = if b.first() == Some(&1) {
                        let d = u64::from_be_bytes(b.get(24..32)?.try_into().ok()?);
                        (read32(20)?, d as f64)
                    } else {
                        (read32(12)?, read32(16)? as f64)
                    };
                    return (scale > 0).then(|| duration / scale as f64);
                }
                inner += isize;
            }
            return None;
        }
        at += size;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bx(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut v = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        v.extend_from_slice(kind);
        v.extend_from_slice(body);
        v
    }

    #[test]
    fn reads_the_duration_of_an_mp4() {
        // mvhd v0: version+flags, created, modified, timescale 1000, duration 2500.
        let mut mvhd = vec![0u8; 12];
        mvhd.extend_from_slice(&1000u32.to_be_bytes());
        mvhd.extend_from_slice(&2500u32.to_be_bytes());
        mvhd.extend_from_slice(&[0; 80]);
        let mut file = bx(b"ftyp", b"isom\0\0\x02\0");
        file.extend(bx(b"free", &[0; 4]));
        file.extend(bx(b"moov", &bx(b"mvhd", &mvhd)));
        assert_eq!(mp4_seconds(&file), Some(2.5));
        assert_eq!(mp4_seconds(b"<html>not a video</html>"), None);
        assert_eq!(mp4_seconds(&file[..20]), None);
    }

    #[test]
    fn quotes_paths_for_curl() {
        assert_eq!(quoted(Path::new("a;b,\"c\".mp4")), "\"a;b,\\\"c\\\".mp4\"");
    }
}
