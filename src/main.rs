//! `oxframe` — the command line for oxframe projects.
//!
//! The CLI holds no animation logic: it builds the project with cargo, runs
//! its binary (`preview`/`render`/`info` commands provided by the SDK),
//! watches files, restarts the process and serves the editor UI.

mod agents;
mod cargo;
mod check;
mod config;
mod dev;
mod init;
mod render;
mod snapshot;
mod ui;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

/// Exit codes (spec §5): 0 ok, 1 build error, 2 environment (no ffmpeg, port
/// in use), 3 version mismatch.
pub mod exit {
    pub const OK: u8 = 0;
    pub const BUILD: u8 = 1;
    pub const ENV: u8 = 2;
    pub const VERSION: u8 = 3;
}

#[derive(Parser)]
#[command(name = "oxframe", version, about = "Procedural animation in Rust")]
struct Cli {
    /// More logging (same as RUST_LOG=debug).
    #[arg(long, short, global = true)]
    verbose: bool,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a new project.
    Init {
        name: String,
        /// Use a local oxframe-sdk checkout instead of crates.io.
        #[arg(long, env = "OXFRAME_SDK_PATH")]
        sdk_path: Option<PathBuf>,
    },
    /// Watch, rebuild and preview in the browser editor.
    Dev {
        #[arg(long)]
        port: Option<u16>,
        /// Open the browser.
        #[arg(long)]
        open: bool,
        /// Do not open the browser even if oxframe.toml says so.
        #[arg(long, conflicts_with = "open")]
        no_open: bool,
        /// Build with --release.
        #[arg(long)]
        release: bool,
        /// Serve the editor from a directory instead of the embedded one.
        #[arg(long)]
        ui_dir: Option<PathBuf>,
        /// Render previews on the CPU.
        #[arg(long)]
        cpu: bool,
    },
    /// Build in release mode and render a video.
    Render {
        #[arg(long)]
        out: Option<PathBuf>,
        #[arg(long)]
        scene: Option<String>,
        /// Seconds, e.g. 1.5..4
        #[arg(long)]
        range: Option<String>,
        /// mp4 | webm | prores | png
        #[arg(long)]
        format: Option<String>,
        #[arg(long)]
        size: Option<String>,
        /// Render on the CPU (tiny-skia), e.g. in CI without a GPU.
        #[arg(long)]
        cpu: bool,
        #[arg(long)]
        no_audio: bool,
    },
    /// Type-check, validate meta files and compare protocol versions.
    Check,
    /// Render single frames as PNG (default: out/snapshots/), e.g. `--at 1.5`.
    Snapshot {
        /// Time in seconds (repeatable).
        #[arg(long, required = true)]
        at: Vec<f64>,
        /// Output file (one --at, ending in .png) or directory.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Time is measured from the start of this scene.
        #[arg(long)]
        scene: Option<String>,
        /// Output size, e.g. 1920x1080 (default 960x540).
        #[arg(long)]
        size: Option<String>,
        #[arg(long)]
        release: bool,
    },
    /// Write or refresh instructions for AI coding agents (AGENTS.md, .claude/skills/oxframe).
    Agents {
        /// Replace files you edited yourself.
        #[arg(long)]
        force: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let filter = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(filter)),
        )
        .with_target(false)
        .without_time()
        .with_writer(std::io::stderr)
        .init();

    let code = match cli.command {
        Cmd::Init { name, sdk_path } => init::run(&name, sdk_path.as_deref()),
        Cmd::Dev {
            port,
            open,
            no_open,
            release,
            ui_dir,
            cpu,
        } => dev::run(dev::DevArgs {
            port,
            open,
            no_open,
            release,
            ui_dir,
            cpu,
        }),
        Cmd::Render {
            out,
            scene,
            range,
            format,
            size,
            cpu,
            no_audio,
        } => render::run(render::RenderArgs {
            out,
            scene,
            range,
            format,
            size,
            cpu,
            no_audio,
        }),
        Cmd::Check => check::run(),
        Cmd::Snapshot {
            at,
            out,
            scene,
            size,
            release,
        } => snapshot::run(snapshot::SnapshotArgs {
            at,
            out,
            scene,
            size,
            release,
        }),
        Cmd::Agents { force } => agents::run(force),
    };
    ExitCode::from(code)
}

/// Prints an error in red.
pub fn error(msg: impl std::fmt::Display) {
    let style = anstyle::Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::Red.into()));
    anstream::eprintln!("{style}error{style:#}: {msg}");
}

pub fn note(msg: impl std::fmt::Display) {
    let style = anstyle::Style::new()
        .bold()
        .fg_color(Some(anstyle::AnsiColor::Cyan.into()));
    anstream::eprintln!("{style}oxframe{style:#}: {msg}");
}
