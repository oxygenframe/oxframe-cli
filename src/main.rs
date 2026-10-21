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
mod hints;
mod init;
mod mcp;
mod render;
mod reply;
mod snapshot;
mod ui;
mod update_check;
mod upgrade;

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
    /// Print one JSON object on stdout instead of human text.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a new project.
    Init {
        name: String,
        /// `node` makes a library crate with a node of your own.
        #[arg(long, value_enum, default_value = "default")]
        template: init::Template,
        /// Use a local oxframe-sdk checkout instead of crates.io.
        #[arg(long, env = "OXFRAME_SDK_PATH")]
        sdk_path: Option<PathBuf>,
    },
    /// Show the project as a presentation: full screen, stopping at every
    /// `slide_break` and scene end. Arrows, space, page keys and slide
    /// clickers move between stops; N opens the speaker notes.
    Present {
        #[arg(long)]
        port: Option<u16>,
        /// Do not open the browser.
        #[arg(long)]
        no_open: bool,
        /// Build with --release.
        #[arg(long)]
        release: bool,
        /// Serve the editor from a directory instead of the embedded one.
        #[arg(long)]
        ui_dir: Option<PathBuf>,
        /// Render on the CPU.
        #[arg(long)]
        cpu: bool,
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
        /// 16:9, 9:16, 1:1, or all. `all` renders every format the project declares.
        #[arg(long)]
        aspect: Option<String>,
        /// Render on the CPU (tiny-skia), e.g. in CI without a GPU.
        #[arg(long)]
        cpu: bool,
        #[arg(long)]
        no_audio: bool,
        /// Also write the spoken lines as .srt or .vtt (by extension).
        #[arg(long)]
        subtitles: Option<PathBuf>,
        /// Instead of a video, a PDF with the frame of every presentation
        /// stop (`slide_break` and scene ends). No ffmpeg needed.
        #[arg(long)]
        slides: Option<PathBuf>,
        /// Render 2-second segments in this many processes at once (default:
        /// cores, at most 8). The file is the same for any number.
        #[arg(long)]
        jobs: Option<u32>,
    },
    /// Type-check, then write a storyboard and a visual-lint report to out/check/.
    Check {
        /// Also write each storyboard frame at full size under out/check/frames/.
        #[arg(long)]
        frames: bool,
    },
    /// Print the scene tree at a time as JSON (kind, name, bounds, text).
    Inspect {
        /// Time in seconds.
        #[arg(long)]
        at: f64,
        /// Time is measured from the start of this scene.
        #[arg(long)]
        scene: Option<String>,
        /// Also list render passes.
        #[arg(long)]
        graph: bool,
    },
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
    /// Move the project to the SDK version this CLI is made for, then refresh
    /// Cargo.lock and the AI agent files.
    Upgrade,
    /// Write or refresh instructions for AI coding agents (AGENTS.md, .claude/skills/oxframe).
    Agents {
        /// Replace files you edited yourself.
        #[arg(long)]
        force: bool,
    },
    /// MCP server on stdin/stdout (tools: check, storyboard, render_frame, inspect, list_components, docs_search, render).
    Mcp,
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

    if !matches!(cli.command, Cmd::Mcp) {
        update_check::run();
    }
    let done = match cli.command {
        Cmd::Init {
            name,
            template,
            sdk_path,
        } => {
            let code = init::run(&name, template, sdk_path.as_deref());
            reply::done(cli.json, code, serde_json::json!({"command": "init"}))
        }
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
            present: false,
            json: cli.json,
        }),
        Cmd::Present {
            port,
            no_open,
            release,
            ui_dir,
            cpu,
        } => dev::run(dev::DevArgs {
            port,
            open: !no_open,
            no_open,
            release,
            ui_dir,
            cpu,
            present: true,
            json: cli.json,
        }),
        Cmd::Render {
            out,
            scene,
            range,
            format,
            size,
            aspect,
            cpu,
            no_audio,
            subtitles,
            slides,
            jobs,
        } => render::run(render::RenderArgs {
            out,
            scene,
            range,
            format,
            size,
            aspect,
            cpu,
            no_audio,
            subtitles,
            slides,
            jobs,
            json: cli.json,
        }),
        Cmd::Check { frames } => check::run(frames, cli.json),
        Cmd::Inspect { at, scene, graph } => check::inspect(at, scene, graph, cli.json),
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
            json: cli.json,
        }),
        Cmd::Agents { force } => {
            let code = agents::run(force);
            reply::done(cli.json, code, serde_json::json!({"command": "agents"}))
        }
        Cmd::Upgrade => {
            let code = upgrade::run();
            reply::done(cli.json, code, serde_json::json!({"command": "upgrade"}))
        }
        Cmd::Mcp => mcp::serve(),
    };
    if cli.json && !done.printed {
        reply::emit(&done);
    }
    ExitCode::from(done.code)
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
