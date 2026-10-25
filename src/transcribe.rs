//! `oxframe transcribe`: word timings for a recorded voice-over, so
//! `voice("…").recording("vo/intro.wav")` can wait for words and caption
//! them. Runs whisper.cpp (`whisper-cli`) on the machine: nothing is sent
//! anywhere. The model is downloaded once into the oxframe cache, pinned by
//! SHA-256.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use sha2::{Digest, Sha256};

/// Models from huggingface.co/ggerganov/whisper.cpp: name, size, SHA-256.
const MODELS: &[(&str, u64, &str)] = &[
    (
        "tiny",
        77_691_713,
        "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
    ),
    (
        "base",
        147_951_465,
        "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
    ),
    (
        "small",
        487_601_967,
        "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
    ),
];

pub const WHISPER_HELP: &str = "whisper.cpp is not installed. \
Install it (macOS and Linux with Homebrew: `brew install whisper-cpp`; \
or build it from https://github.com/ggml-org/whisper.cpp), \
or pass its whisper-cli with --whisper <path>.";

/// One timed word, as the SDK reads it.
#[derive(Debug, Serialize, PartialEq)]
pub struct Word {
    pub text: String,
    pub start: f64,
    pub end: f64,
}

/// `<stem>.words.json`.
#[derive(Debug, Serialize)]
pub struct WordsFile {
    pub text: String,
    pub seconds: f64,
    pub words: Vec<Word>,
}

pub struct Options {
    pub files: Vec<PathBuf>,
    pub model: String,
    pub language: String,
    pub whisper: Option<PathBuf>,
}

/// Where the words of `audio` go: `vo/intro.wav` → `vo/intro.words.json`.
pub fn words_path(audio: &Path) -> PathBuf {
    audio.with_extension("words.json")
}

pub fn run(opts: &Options, json: bool) -> crate::reply::Done {
    match transcribe_all(opts, json) {
        Ok(done) => crate::reply::done(json, crate::exit::OK, serde_json::json!({ "files": done })),
        Err(e) => crate::reply::fail(json, crate::exit::ENV, e),
    }
}

fn transcribe_all(opts: &Options, quiet: bool) -> Result<Vec<serde_json::Value>, String> {
    if opts.files.is_empty() {
        return Err("name at least one audio file: oxframe transcribe vo/intro.wav".into());
    }
    let whisper = find_whisper(opts.whisper.as_deref())?;
    let ffmpeg = crate::ffmpeg::ensure(None, quiet)?;
    let model = model_path(&opts.model, quiet)?;
    let mut done = Vec::new();
    for file in &opts.files {
        if !file.is_file() {
            return Err(format!("{}: no such file", file.display()));
        }
        let words = transcribe(file, &whisper, &ffmpeg, &model, &opts.language)?;
        let out = words_path(file);
        let body = serde_json::to_string_pretty(&words).map_err(|e| e.to_string())?;
        std::fs::write(&out, body + "\n")
            .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
        if !quiet {
            println!(
                "{} → {} ({} words, {:.1} s)",
                file.display(),
                out.display(),
                words.words.len(),
                words.seconds
            );
        }
        done.push(serde_json::json!({
            "audio": file,
            "words_file": out,
            "words": words.words.len(),
            "seconds": words.seconds,
            "text": words.text,
        }));
    }
    Ok(done)
}

fn transcribe(
    file: &Path,
    whisper: &Path,
    ffmpeg: &Path,
    model: &Path,
    language: &str,
) -> Result<WordsFile, String> {
    let tmp = std::env::temp_dir().join(format!("oxframe-transcribe-{}", std::process::id()));
    std::fs::create_dir_all(&tmp).map_err(|e| format!("cannot make a temp folder: {e}"))?;
    let result = transcribe_in(&tmp, file, whisper, ffmpeg, model, language);
    let _ = std::fs::remove_dir_all(&tmp);
    result
}

fn transcribe_in(
    tmp: &Path,
    file: &Path,
    whisper: &Path,
    ffmpeg: &Path,
    model: &Path,
    language: &str,
) -> Result<WordsFile, String> {
    let wav = tmp.join("audio.wav");
    // whisper.cpp reads 16 kHz mono 16-bit WAV.
    let out = Command::new(ffmpeg)
        .args(["-v", "error", "-y", "-i"])
        .arg(file)
        .args([
            "-ar",
            "16000",
            "-ac",
            "1",
            "-c:a",
            "pcm_s16le",
            "-map_metadata",
            "-1",
            "-fflags",
            "+bitexact",
        ])
        .arg(&wav)
        .output()
        .map_err(|e| format!("cannot run ffmpeg: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "ffmpeg cannot read {}: {}",
            file.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let seconds = wav_seconds(&std::fs::read(&wav).map_err(|e| e.to_string())?)
        .ok_or_else(|| format!("ffmpeg wrote an unreadable WAV for {}", file.display()))?;
    let base = tmp.join("words");
    let out = Command::new(whisper)
        .arg("-m")
        .arg(model)
        .arg("-f")
        .arg(&wav)
        .args(["-ml", "1", "-sow", "-oj", "-np", "-l", language, "-of"])
        .arg(&base)
        .output()
        .map_err(|e| format!("cannot run {}: {e}", whisper.display()))?;
    if !out.status.success() {
        return Err(format!(
            "whisper-cli failed on {}: {}",
            file.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let raw = std::fs::read_to_string(base.with_extension("json"))
        .map_err(|e| format!("whisper-cli wrote no JSON: {e}"))?;
    parse(&raw, seconds)
}

/// whisper.cpp's JSON with one segment per word (`-ml 1 -sow`).
fn parse(raw: &str, seconds: f64) -> Result<WordsFile, String> {
    let v: serde_json::Value =
        serde_json::from_str(raw).map_err(|e| format!("whisper-cli JSON: {e}"))?;
    let segments = v["transcription"]
        .as_array()
        .ok_or("whisper-cli JSON has no transcription")?;
    let mut text = String::new();
    let mut words = Vec::new();
    for seg in segments {
        let said = seg["text"].as_str().unwrap_or("").trim();
        if said.is_empty() || said.starts_with('[') {
            continue;
        }
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(said);
        let from = seg["offsets"]["from"].as_f64().unwrap_or(0.0) / 1000.0;
        let to = seg["offsets"]["to"].as_f64().unwrap_or(0.0) / 1000.0;
        let bare = said.trim_matches(|c: char| !c.is_alphanumeric());
        if bare.is_empty() {
            continue;
        }
        words.push(Word {
            text: bare.to_owned(),
            start: from.min(seconds),
            end: to.min(seconds),
        });
    }
    Ok(WordsFile {
        text,
        seconds,
        words,
    })
}

/// Length of a PCM WAV from its `fmt ` and `data` chunks.
fn wav_seconds(bytes: &[u8]) -> Option<f64> {
    if bytes.get(0..4)? != b"RIFF" || bytes.get(8..12)? != b"WAVE" {
        return None;
    }
    let mut at = 12;
    let mut byte_rate = None;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().ok()?) as usize;
        let body = at + 8;
        if id == b"fmt " {
            byte_rate = Some(u32::from_le_bytes(
                bytes.get(body + 8..body + 12)?.try_into().ok()?,
            ));
        } else if id == b"data" {
            let len = len.min(bytes.len() - body);
            return Some(len as f64 / f64::from(byte_rate?.max(1)));
        }
        at = body + len + (len & 1);
    }
    None
}

fn find_whisper(explicit: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(p) = explicit {
        return if p.is_file() {
            Ok(p.to_path_buf())
        } else {
            Err(format!("whisper-cli not found at {}", p.display()))
        };
    }
    let names: &[&str] = if cfg!(windows) {
        &["whisper-cli.exe"]
    } else {
        &["whisper-cli"]
    };
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .flat_map(|d| names.iter().map(move |n| d.join(n)))
                .find(|p| p.is_file())
        })
        .ok_or_else(|| WHISPER_HELP.to_owned())
}

/// A model file: a path as given, or a name from [`MODELS`], downloaded once.
fn model_path(model: &str, quiet: bool) -> Result<PathBuf, String> {
    let as_path = Path::new(model);
    if as_path.is_file() {
        return Ok(as_path.to_path_buf());
    }
    let Some(&(name, size, sha)) = MODELS.iter().find(|m| m.0 == model) else {
        let names: Vec<&str> = MODELS.iter().map(|m| m.0).collect();
        return Err(format!(
            "unknown model {model:?}: use {} or the path of a ggml model file",
            names.join(", ")
        ));
    };
    let dir = crate::ffmpeg::cache_root()
        .ok_or("no cache folder (HOME is not set)")?
        .join("whisper");
    let file = dir.join(format!("ggml-{name}.bin"));
    if file.metadata().is_ok_and(|m| m.len() == size) {
        return Ok(file);
    }
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let url = format!("https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-{name}.bin");
    if !quiet {
        crate::note(format!(
            "downloading the whisper {name} model ({} MB, once) into {}",
            size / 1_000_000,
            dir.display()
        ));
    }
    let part = dir.join(format!("ggml-{name}.bin.part"));
    let status = Command::new("curl")
        .args(["-fL", "--retry", "3", if quiet { "-s" } else { "-#" }, "-o"])
        .arg(&part)
        .arg(&url)
        .status()
        .map_err(|e| format!("cannot run curl to download the model: {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_file(&part);
        return Err(format!("downloading {url} failed ({status})"));
    }
    let bytes = std::fs::read(&part).map_err(|e| format!("cannot read {}: {e}", part.display()))?;
    let got = crate::ffmpeg::hex(&Sha256::digest(&bytes));
    if got != sha {
        let _ = std::fs::remove_file(&part);
        return Err(format!(
            "the downloaded model does not match its checksum (got {got}); nothing was installed"
        ));
    }
    std::fs::rename(&part, &file).map_err(|e| format!("cannot move the model into place: {e}"))?;
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words_drop_punctuation_and_blank_segments() {
        let raw = r#"{"transcription":[
            {"offsets":{"from":0,"to":320},"text":""},
            {"offsets":{"from":320,"to":400},"text":" And"},
            {"offsets":{"from":400,"to":690},"text":" so,"},
            {"offsets":{"from":700,"to":900},"text":" [BLANK_AUDIO]"},
            {"offsets":{"from":9760,"to":10500},"text":" country."}
        ]}"#;
        let w = parse(raw, 10.2).unwrap();
        assert_eq!(w.text, "And so, country.");
        let texts: Vec<&str> = w.words.iter().map(|w| w.text.as_str()).collect();
        assert_eq!(texts, ["And", "so", "country"]);
        assert!((w.words[1].start - 0.4).abs() < 1e-9);
        assert!((w.words[2].end - 10.2).abs() < 1e-9);
    }

    #[test]
    fn wav_length_comes_from_the_data_chunk() {
        let mut wav = b"RIFF\0\0\0\0WAVE".to_vec();
        wav.extend(b"fmt ");
        wav.extend(16u32.to_le_bytes());
        wav.extend([1, 0, 1, 0]);
        wav.extend(16_000u32.to_le_bytes());
        wav.extend(32_000u32.to_le_bytes());
        wav.extend([2, 0, 16, 0]);
        wav.extend(b"LIST");
        wav.extend(3u32.to_le_bytes());
        wav.extend([0, 0, 0, 0]);
        wav.extend(b"data");
        wav.extend(48_000u32.to_le_bytes());
        wav.extend(vec![0; 48_000]);
        assert!((wav_seconds(&wav).unwrap() - 1.5).abs() < 1e-9);
    }

    #[test]
    fn the_words_file_sits_next_to_the_audio() {
        assert_eq!(
            words_path(Path::new("vo/intro.wav")),
            PathBuf::from("vo/intro.words.json")
        );
    }

    #[test]
    fn every_model_has_a_full_checksum() {
        for (name, _, sha) in MODELS {
            assert_eq!(sha.len(), 64, "{name}");
        }
    }
}
