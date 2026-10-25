//! A static ffmpeg for machines that have none: `render` and `publish`
//! download it once into the oxframe cache, and the CLI puts that folder on
//! `PATH` for the project it runs (render, preview, video clips).
//!
//! The builds are the ones the `ffmpeg-static` npm package ships, from its
//! GitHub releases, pinned by tag and checked by SHA-256.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use sha2::{Digest, Sha256};

/// Release of github.com/eugeneware/ffmpeg-static the builds come from.
const TAG: &str = "b6.1.1";

/// `(os, arch)` → (asset, SHA-256 of the asset).
const BUILDS: &[(&str, &str, &str, &str)] = &[
    (
        "linux",
        "x86_64",
        "ffmpeg-linux-x64.gz",
        "bfe8a8fc511530457b528c48d77b5737527b504a3797a9bc4866aeca69c2dffa",
    ),
    (
        "linux",
        "aarch64",
        "ffmpeg-linux-arm64.gz",
        "754a678672298bc68156adff58aa7385a592c2b30b1d0ae8750c45c915c4bac0",
    ),
    (
        "macos",
        "x86_64",
        "ffmpeg-darwin-x64.gz",
        "929b375c1182d956c51f7ac25e0b2b0411fb01f6f407aa15c9758efeb4242106",
    ),
    (
        "macos",
        "aarch64",
        "ffmpeg-darwin-arm64.gz",
        "8923876afa8db5585022d7860ec7e589af192f441c56793971276d450ed3bbfa",
    ),
    (
        "windows",
        "x86_64",
        "ffmpeg-win32-x64.gz",
        "8883a3dffbd0a16cf4ef95206ea05283f78908dbfb118f73c83f4951dcc06d77",
    ),
];

/// Set to `1` to never download (the CLI then asks you to install ffmpeg).
pub const NO_DOWNLOAD: &str = "OXFRAME_NO_FFMPEG_DOWNLOAD";

fn exe_name() -> &'static str {
    if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    }
}

/// Folder the downloaded ffmpeg lives in.
pub fn dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("LOCALAPPDATA").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("oxframe").join("ffmpeg").join(TAG))
}

/// The downloaded ffmpeg, if there is one.
pub fn cached() -> Option<PathBuf> {
    dir().map(|d| d.join(exe_name())).filter(|p| p.is_file())
}

/// `PATH` for a process this CLI starts (the project: render, preview,
/// video clips): with the downloaded ffmpeg's folder first when no other
/// ffmpeg is on `PATH`. `None` leaves `PATH` as it is.
pub fn child_path() -> Option<std::ffi::OsString> {
    let bin = cached()?;
    if on_path().is_some() {
        return None;
    }
    let mut paths: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).collect())
        .unwrap_or_default();
    paths.insert(0, bin.parent()?.to_path_buf());
    std::env::join_paths(paths).ok()
}

/// `ffmpeg` on `PATH`.
pub fn on_path() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths)
            .map(|d| d.join(exe_name()))
            .find(|p| p.is_file())
    })
}

/// ffmpeg for a command that needs it: the configured one, one on `PATH`,
/// the downloaded one, or a fresh download.
pub fn ensure(explicit: Option<&Path>, quiet: bool) -> Result<PathBuf, String> {
    if let Some(found) = crate::render::find_ffmpeg(explicit) {
        return Ok(found);
    }
    if explicit.is_some() {
        return Err(format!(
            "ffmpeg not found at {}",
            explicit.unwrap_or(Path::new("")).display()
        ));
    }
    if std::env::var_os(NO_DOWNLOAD).is_some_and(|v| v != "0") {
        return Err(crate::render::FFMPEG_HELP.into());
    }
    download(quiet)
}

/// Downloads the build for this machine unless it is already there.
pub fn download(quiet: bool) -> Result<PathBuf, String> {
    if let Some(bin) = cached() {
        return Ok(bin);
    }
    let dir = dir().ok_or("no cache folder (HOME is not set)")?;
    download_into(&dir, quiet)
}

fn download_into(dir: &Path, quiet: bool) -> Result<PathBuf, String> {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let Some(&(_, _, asset, sha)) = BUILDS.iter().find(|b| b.0 == os && b.1 == arch) else {
        return Err(format!(
            "no ffmpeg build to download for {os} {arch}.\n{}",
            crate::render::FFMPEG_HELP
        ));
    };
    std::fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let url =
        format!("https://github.com/eugeneware/ffmpeg-static/releases/download/{TAG}/{asset}");
    if !quiet {
        crate::note(format!(
            "ffmpeg not found: downloading a static build (about 25 MB, once) into {}",
            dir.display()
        ));
    }
    let gz = dir.join(format!("{asset}.part"));
    let status = Command::new("curl")
        .args(["-fL", "--retry", "3", if quiet { "-s" } else { "-#" }, "-o"])
        .arg(&gz)
        .arg(&url)
        .status()
        .map_err(|e| format!("cannot run curl to download ffmpeg: {e}"))?;
    if !status.success() {
        let _ = std::fs::remove_file(&gz);
        return Err(format!(
            "downloading {url} failed ({status}).\n{}",
            crate::render::FFMPEG_HELP
        ));
    }
    let bytes = std::fs::read(&gz).map_err(|e| format!("cannot read {}: {e}", gz.display()))?;
    let _ = std::fs::remove_file(&gz);
    let got = hex(&Sha256::digest(&bytes));
    if got != sha {
        return Err(format!(
            "the downloaded ffmpeg does not match its checksum (got {got}); nothing was installed"
        ));
    }
    let mut binary = Vec::new();
    flate2::read::GzDecoder::new(&bytes[..])
        .read_to_end(&mut binary)
        .map_err(|e| format!("cannot unpack ffmpeg: {e}"))?;
    let bin = dir.join(exe_name());
    let tmp = dir.join(format!("{}.tmp", exe_name()));
    std::fs::write(&tmp, &binary).map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("cannot make {} executable: {e}", tmp.display()))?;
    }
    std::fs::rename(&tmp, &bin).map_err(|e| format!("cannot move ffmpeg into place: {e}"))?;
    if !quiet {
        crate::note(format!("ffmpeg ready: {}", bin.display()));
    }
    Ok(bin)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_build_has_a_full_checksum() {
        for (os, arch, asset, sha) in BUILDS {
            assert_eq!(sha.len(), 64, "{os} {arch}");
            assert!(sha.chars().all(|c| c.is_ascii_hexdigit()), "{os} {arch}");
            assert!(asset.ends_with(".gz"), "{asset}");
        }
    }

    /// Downloads the build for this machine and checks it has every encoder
    /// and filter oxframe uses. Run in CI on each OS:
    /// `cargo test -- --ignored downloaded_ffmpeg`.
    #[test]
    #[ignore = "downloads about 25 MB"]
    fn downloaded_ffmpeg_has_what_oxframe_needs() {
        let tmp = std::env::temp_dir().join(format!("oxframe-ffmpeg-{}", std::process::id()));
        let bin = download_into(&tmp, true).unwrap();
        let encoders = Command::new(&bin)
            .args(["-hide_banner", "-encoders"])
            .output()
            .unwrap();
        let encoders = String::from_utf8_lossy(&encoders.stdout);
        for name in [
            "libx264",
            "libvpx-vp9",
            "prores_ks",
            "aac",
            "libopus",
            "gif",
        ] {
            assert!(encoders.contains(&format!(" {name} ")), "no {name} encoder");
        }
        let filters = Command::new(&bin)
            .args(["-hide_banner", "-filters"])
            .output()
            .unwrap();
        let filters = String::from_utf8_lossy(&filters.stdout);
        for name in ["palettegen", "paletteuse", "apad", "scale"] {
            assert!(filters.contains(&format!(" {name} ")), "no {name} filter");
        }
        let _ = std::fs::remove_dir_all(tmp);
    }
}
