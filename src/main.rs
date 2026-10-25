//! `oxframe` — the command line for oxframe projects.
//!
//! The CLI holds no animation logic: it builds the project with cargo, runs
//! its binary (`preview`/`render`/`info` commands provided by the SDK),
//! watches files, restarts the process and serves the editor UI.

mod add;
mod agents;
mod cargo;
mod check;
mod config;
mod dev;
mod doctor;
mod ffmpeg;
mod hints;
mod init;
mod mcp;
mod publish;
mod render;
mod reply;
mod snapshot;
mod transcribe;
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
        /// mp4 | webm | prores | png | gif
        #[arg(long)]
        format: Option<String>,
        /// Output size, e.g. 1280x720. A GIF is 720 wide at most unless set.
        #[arg(long)]
        size: Option<String>,
        /// Frames per second of a GIF (default 15).
        #[arg(long)]
        fps: Option<u32>,
        /// 16:9, 9:16, 1:1, or all. `all` renders every format the project declares.
        #[arg(long)]
        aspect: Option<String>,
        /// Render on the CPU (tiny-skia), e.g. in CI without a GPU.
        #[arg(long)]
        cpu: bool,
        /// Render on the GPU (vello) instead of the default CPU renderer.
        #[arg(long)]
        gpu: bool,
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
    /// Render the project (or take an MP4) and upload it to oxframe.dev: prints
    /// a public link with a player, kept for 30 days. Up to 30 s and 16 MB.
    Publish {
        /// An MP4 to publish instead of rendering the project.
        file: Option<PathBuf>,
        #[arg(long, conflicts_with = "file")]
        scene: Option<String>,
        /// Seconds, e.g. 0..30
        #[arg(long, conflicts_with = "file")]
        range: Option<String>,
        /// Shown on the page and in link previews.
        #[arg(long)]
        title: Option<String>,
        /// Render on the CPU (tiny-skia).
        #[arg(long, conflicts_with = "file")]
        cpu: bool,
        /// Render on the GPU (vello).
        #[arg(long, conflicts_with = "file")]
        gpu: bool,
    },
    /// Type-check, then write a storyboard and a visual-lint report to out/check/.
    Check {
        /// Also write each storyboard frame at full size under out/check/frames/.
        #[arg(long)]
        frames: bool,
        /// Compare the frames with an earlier `oxframe check --frames --out <dir>`:
        /// lists the frames that changed and writes out/check/diff/.
        #[arg(long, value_name = "DIR")]
        against: Option<PathBuf>,
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
    /// Add a ready-made block to the project (`src/blocks/`); with no name,
    /// list them.
    Add {
        /// Block name, e.g. `stat-counter`.
        block: Option<String>,
        /// Replace the block's file if it exists.
        #[arg(long)]
        force: bool,
    },
    /// Check what building and rendering need (Rust, a C linker, ffmpeg, the
    /// project's SDK version) and say how to fix what is missing.
    Doctor,
    /// Write or refresh instructions for AI coding agents (AGENTS.md, .claude/skills/oxframe).
    Agents {
        /// Replace files you edited yourself.
        #[arg(long)]
        force: bool,
    },
    /// Download a static ffmpeg into the oxframe cache and print its path.
    /// `render` does this by itself when no ffmpeg is installed; set
    /// OXFRAME_NO_FFMPEG_DOWNLOAD=1 to turn that off.
    Ffmpeg,
    /// Time the words of recorded voice-overs with whisper.cpp, on this
    /// machine: writes `<name>.words.json` next to each file for
    /// `voice("…").recording("<file>")`.
    Transcribe {
        /// Audio files (any format ffmpeg reads).
        files: Vec<std::path::PathBuf>,
        /// tiny, base or small (downloaded once), or the path of a ggml model.
        #[arg(long, default_value = "base")]
        model: String,
        /// Spoken language, such as en or ru; auto detects it.
        #[arg(long, default_value = "auto")]
        language: String,
        /// whisper.cpp's whisper-cli, if it is not on PATH.
        #[arg(long)]
        whisper: Option<std::path::PathBuf>,
    },
    /// MCP server on stdin/stdout (tools: check, storyboard, render_frame, inspect, render_graph, list_components, docs_search, render).
    Mcp {
        /// Project directory; the current one by default. For clients that start servers elsewhere.
        #[arg(long)]
        dir: Option<std::path::PathBuf>,
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

    if !matches!(cli.command, Cmd::Mcp { .. }) {
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
            gpu,
            no_audio,
            subtitles,
            slides,
            jobs,
            fps,
        } => render::run(render::RenderArgs {
            out,
            scene,
            range,
            format,
            size,
            aspect,
            cpu,
            gpu,
            no_audio,
            subtitles,
            slides,
            jobs,
            fps,
            json: cli.json,
        }),
        Cmd::Publish {
            file,
            scene,
            range,
            title,
            cpu,
            gpu,
        } => publish::run(publish::PublishArgs {
            file,
            scene,
            range,
            title,
            cpu,
            gpu,
            json: cli.json,
        }),
        Cmd::Check { frames, against } => check::run(frames, against, cli.json),
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
        Cmd::Doctor => doctor::run(cli.json),
        Cmd::Ffmpeg => match ffmpeg::download(cli.json) {
            Ok(bin) => {
                if !cli.json {
                    println!("{}", bin.display());
                }
                reply::done(cli.json, exit::OK, serde_json::json!({ "ffmpeg": bin }))
            }
            Err(e) => reply::fail(cli.json, exit::ENV, e),
        },
        Cmd::Transcribe {
            files,
            model,
            language,
            whisper,
        } => transcribe::run(
            &transcribe::Options {
                files,
                model,
                language,
                whisper,
            },
            cli.json,
        ),
        Cmd::Add { block, force } => add::run(block, force, cli.json),
        Cmd::Upgrade => {
            let code = upgrade::run();
            reply::done(cli.json, code, serde_json::json!({"command": "upgrade"}))
        }
        Cmd::Mcp { dir } => {
            if let Some(dir) = dir
                && let Err(e) = std::env::set_current_dir(&dir)
            {
                error(format!("cannot open {}: {e}", dir.display()));
                return ExitCode::from(exit::ENV);
            }
            mcp::serve()
        }
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
