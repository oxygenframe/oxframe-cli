# Changelog

`oxframe-cli` is released together with the oxframe SDK crates under the
same version (see the SDK's
[CHANGELOG](https://github.com/oxygenframe/oxframe-sdk/blob/main/CHANGELOG.md)).

## Unreleased

- `oxframe transcribe vo/intro.wav` times the words of a recorded voice-over
  with whisper.cpp (`whisper-cli`, installed separately: `brew install
  whisper-cpp`) and writes `vo/intro.words.json` for
  `voice("…").recording("vo/intro.wav")`. Nothing leaves the machine. The
  model (`--model tiny|base|small`, base by default) is downloaded once into
  the oxframe cache and checked by SHA-256; `--language ru` skips detection.

## 0.23.6 — 2026-10-25

- Creates projects on oxframe 0.23.6. 0.23.5 broke projects on oxframe
  0.23.0–0.23.4 (and is yanked): `oxframe upgrade` moves a project on it to
  0.23.6.

## 0.23.5 — 2026-10-25

- Creates projects on oxframe 0.23.5.
- Six more blocks for `oxframe add` (24 in all): `metric-row` (numbers
  counting up side by side), `key-combo` (a keyboard shortcut pressed key by
  key), `steps` (numbered steps filling in along a line), `social-post`
  (author, text and likes counting up), `split-screen` (two columns with a
  divider) and `search-bar` (a query typed, results dropping in).
- The agent files mention `Project::loudness`.
- No ffmpeg installed? `oxframe render` and `oxframe publish` download a
  static build once (about 25 MB, checked by SHA-256) into the oxframe cache
  and use it for the project, the preview and video clips. `oxframe ffmpeg`
  downloads it ahead of time (CI, offline work); `OXFRAME_NO_FFMPEG_DOWNLOAD=1`
  turns it off. `oxframe doctor` shows which ffmpeg is used.
- The agent files point to `fix` in each `oxframe check` issue: the change
  with the numbers worked out.
- `oxframe render --format gif` (or `--out demo.gif`): an animated GIF for a
  README, a pull request or a chat, 720 wide and 15 fps unless `--size` and
  `--fps` say otherwise. The MCP `render` tool takes `format: "gif"` and `fps`.

## 0.23.4 — 2026-10-25

- Creates projects on oxframe 0.23.4.
- `oxframe mcp --dir <project>`: serve a project other than the current
  directory, for MCP clients that start servers elsewhere.
- The MCP Registry entry links to https://oxframe.dev/docs/mcp.

## 0.23.3 — 2026-10-24

- Creates projects on oxframe 0.23.3.
- `oxframe add`: six more blocks — `feature-grid` (feature cards in a row),
  `checklist` (items ticked off), `comparison` (a yes/no table against
  another product), `chat-messages` (a conversation in bubbles),
  `pull-request` (a PR card that gets merged) and `countdown` (numbers in a
  filling ring). All eighteen pass `oxframe check` in one tour (e2e test).
- The agent files no longer say a custom `shade` skips the playground.
- The agent files say that a request to explain or show something means a
  scene, and cover four compile errors common in the 0.23.2 agent eval.

## 0.23.2 — 2026-10-24

- Creates projects on oxframe 0.23.2.
- `oxframe publish`: renders the project (or takes an MP4), uploads it to
  oxframe.dev and prints a link with a player that unfolds as a video in
  Discord and Slack. Kept for 30 days, up to 30 s and 16 MB (a larger render
  is re-encoded at 720p). `--title` names the page; `--range` publishes part
  of a longer video. Needs ffmpeg and curl.
- The agent files know `oxframe publish` and to use it only when asked.
- Listed in the official MCP Registry as `io.github.oxygenframe/oxframe`;
  every release updates it.

## 0.23.1 — 2026-10-24

- Creates projects on oxframe 0.23.1: your own WGSL shader paints in the
  browser playground too.
- The agent files say so (about 0.3 s for a 1080p frame in the browser).

## 0.23.0 — 2026-10-24

- Creates projects on oxframe 0.23.0: your own WGSL shader is painted, and
  it can read an image, another node or the frame drawn before it.
- The agent files know 0.23: `Signal::set` takes plain values, a custom
  `shade` runs, `sample_<name>` reads an input, and `input_cycle` is gone.
- `oxframe add`: six more blocks — `release-notes` (a version and its
  changes), `terminal` (a typed command and its output), `code-change` (code
  that diffs into its new version), `bar-chart`, `kinetic-title` (words pop
  in) and `call-to-action` (a closing line and a link). All twelve pass
  `oxframe check` in one tour (e2e test).

## 0.22.0 — 2026-10-23

- Creates projects on oxframe 0.22.0: `oxframe check` compares with an
  earlier check, lists what moves and when, and shows both sides of hard cuts.
- `oxframe check --against <dir>` (and the `against` argument of the MCP
  `check` tool): compare with an earlier `oxframe check --frames --out <dir>`
  and list the frames that changed, with diff pictures.
- `oxframe add`: ready-made blocks copied into `src/blocks/` — `logo-reveal`,
  `stat-counter`, `progress-bar`, `quote-card`, `before-after`, `timeline`.
  Each is one `async fn` call that plays itself in, and each passes `oxframe
  check`. With no name it lists them; `--json` for agents. The agent files
  describe them.
- A container image, `ghcr.io/oxygenframe/oxframe`: the CLI, Rust and
  ffmpeg, with the SDK already compiled, so a project's first `oxframe
  check` or `oxframe render` in CI takes seconds, not minutes. Published
  with each release.
- The prebuilt Linux binaries (install.sh, cargo binstall) run on Debian 12,
  Ubuntu 22.04 and other systems with glibc 2.34 or newer. 0.21.0's were
  built on Ubuntu 24.04 and stopped with `GLIBC_2.39 not found` there.
- `cargo binstall oxframe-cli` downloads the prebuilt binary from the
  GitHub release instead of compiling (about 5 s). Linux, macOS (Intel and
  Apple Silicon) and Windows.
- Windows gets a prebuilt installer too: `install.ps1` from the release
  downloads the binary, checks its sha256 and adds its folder to the user
  PATH. CI runs it in Windows PowerShell 5.1 and pwsh 7.
- `oxframe doctor` checks what a first build and render need — Rust 1.95 or
  newer with cargo, a C linker, ffmpeg, and, inside a project, whether it is
  on the SDK this CLI makes — and prints how to fix each missing piece for
  the system it runs on. A missing requirement exits 2; no ffmpeg or a
  project on another SDK is a warning. `--json` gives the checks as a list.

## 0.21.0 — 2026-10-23

- Creates projects on oxframe 0.21.0: the CPU draws frames by default
  (`--gpu` opts into vello), `oxframe check` writes a close-up crop for each
  issue on a node, and `Math` formulas come with the `math` feature.
- `oxframe render`, `oxframe dev` and `oxframe present` take `--gpu`;
  `--cpu` still works and wins.
- `oxframe init --template explainer|short|chart`: a scene built from
  components, a vertical 9:16 short, and a function plotted on axes. Each
  compiles and passes `oxframe check` with no errors.
- The project skill and AGENTS.md: a Recipes section (code explanation goes
  to the `Code` node), the check crops, and the `Math` node.

## 0.20.5 — 2026-10-22

- Creates projects on oxframe 0.20.5 (a new example, no API changes).

## 0.20.4 — 2026-10-22

- `oxframe dev`: after a rebuild while the video plays, the picture and the
  sound go on together from the same frame (the editor started the sound
  from the beginning over a still picture). Editor 0.1.6.
- `install.sh` finds the latest release without the GitHub API, which CI
  runners hit the rate limit of.

## 0.20.3 — 2026-10-22

- Creates projects on oxframe 0.20.3: `oxframe check` no longer reports
  text in nested layouts, text written on word by word or a moving sea as
  mistakes.

## 0.20.2 — 2026-10-22

- Creates projects on oxframe 0.20.2: long text wraps inside title and end
  cards, and centered text stays inside its box.

## 0.20.1 — 2026-10-22

- `oxframe mcp` can be listed in the official MCP Registry: `server.json`
  and the `mcp-name:` line in this README.
- Creates projects on oxframe 0.20.1.

## 0.20.0 — 2026-10-21

- `oxframe render --jobs 4` passes the number of parallel segments to the
  project; by default the project uses one per core, at most eight.
- Creates projects on oxframe 0.20.0.

## 0.19.0 — 2026-10-21

- Creates projects on oxframe 0.19.0.
- The agent cheat sheet covers 3D: `Scene3d`, meshes and materials,
  `View3d`, the camera and lights, and 2D labels on 3D points; a reference
  prompt in `evals/` puts a model in a diagram.

## 0.18.0 — 2026-10-21

- `oxframe present` opens the project as a presentation: full screen,
  stopping at every `slide_break` and scene end; arrows, space, page keys
  and clickers move between stops, N opens the speaker notes with a timer.
- `oxframe render --slides talk.pdf` writes one PDF page per stop.
- `oxframe render --subtitles out.srt` is passed to the project; before,
  the CLI rejected the flag that the docs described.
- Creates projects on oxframe 0.18.0; the agent cheat sheet covers
  presentations, and a reference prompt in `evals/` makes a talk.

## 0.17.0 — 2026-10-21

- Creates projects on oxframe 0.17.0.
- The agent cheat sheet covers `Matrix`, vectors and `apply_matrix` on
  axes, and `Table` with animated rows.
- Reference prompts in `evals/`: matrix multiplication step by step and
  a table that sorts itself.

## 0.16.0 — 2026-10-21

- Creates projects on oxframe 0.16.0.
- The agent cheat sheet covers icons: `icon!("lucide:server")`, the three
  sets, `color()` and `stroke_width()`, and icons in `FlowDiagram`.
- Reference prompts for icons in `evals/`: an architecture diagram and a
  row of brand logos.

## 0.15.0 — 2026-10-21

- Creates projects on oxframe 0.15.0.
- The agent cheat sheet covers `Video`: playing, pausing, seeking and
  slowing down a clip, and the `video` feature it needs.

## 0.14.0 — 2026-10-21

- Creates projects on oxframe 0.14.0.
- `oxframe init --template node my-node` makes a library crate with a node
  of your own (`#[derive(Node)]` plus `Draw`), a demo scene in
  `examples/demo.rs` and a reference test.
- The agent cheat sheet covers nodes of your own.

## 0.13.0 — 2026-10-21

- Creates projects on oxframe 0.13.0.
- The agent cheat sheet covers charts: `Axes`, `axes.plot`, `point_on`,
  `to_fn`, `bars`, `line_series`, `scatter` and `NumberLine`.
- Reference prompts for charts in `evals/`: a function that morphs and a
  number line, and the benchmark and line prompts now expect the chart API.

## 0.12.2 — 2026-10-21

- Creates projects on oxframe 0.12.1.
- `list_components` in `oxframe mcp` showed calls that do not compile:
  `BulletList::new().item(..)`, `CodeExplainer::caption`,
  `FlowDiagram::node`, `Callout::new("text").point_at(..)` and arguments to
  `enter`. It now shows the real API, each line checked against the SDK.

## 0.12.1 — 2026-10-21

Still creates projects on oxframe 0.12.0.

- `oxframe mcp` has `render_graph`: the render passes at one time, as
  `oxframe inspect --graph` gives them but without the node tree. An agent
  sees that `lens_drops` or `ripple` runs, or why a shader is skipped.
- The agent cheat sheet has a preset catalog (waves, particles, lens drops,
  ripple, the water wipe, `Custom`) and says to make events signal changes,
  not formulas of `ox.time`.
- Five reference prompts for shaders in `evals/`, from a sea under a title
  to the whole ocean scene.

## 0.12.0 — 2026-10-21

- Creates projects on oxframe 0.12.0.
- The agent cheat sheet covers `shaders::lens_drops()`, `.wetness()`,
  `Node::effect`, `shaders::ripple()`, `Transition::shader` and
  `shaders::water_wipe()`, and when `check` reports `input_cycle`.

## 0.11.0 — 2026-10-21

- Creates projects on oxframe 0.11.0.
- The agent cheat sheet covers `shaders::waves()`, `.amp()`, `Shader::source`
  and `Shader::file`.

## 0.10.0 — 2026-10-21

- Creates projects on oxframe 0.10.0.
- The agent cheat sheet covers `voice("one two").start()`, `.at_word()`,
  `Project::tts`, `TtsProvider` and `oxframe render --subtitles out.srt`.

## 0.9.0 — 2026-10-21

- Creates projects on oxframe 0.9.0.
- The agent cheat sheet covers `sound`, `Audio::track`, `.play()`, `.trim()`,
  `.priority()` and `.duck_under_voice()`.

## 0.8.0 — 2026-10-21

- Creates projects on oxframe 0.8.0.
- `oxframe inspect --graph` adds render passes to the scene JSON: a vector
  pass, then a composite for each layer. Without the flag the JSON is unchanged.

## 0.6.0 — 2026-10-21

- Creates projects on oxframe 0.6.0.
- The agent cheat sheet and instructions describe scene transitions:
  `transition(Transition::slide(Dir::Left)).await`, the other built-ins,
  and a custom blend.

## 0.5.0 — 2026-10-20

- Creates projects on oxframe 0.5.0.
- The agent cheat sheet describes code diffs: `code().to(next, secs)` slides
  tokens, and `insert`, `remove`, `replace` and `.flash()` edit one spot.
- The cheat sheet describes `Svg`: a file becomes groups, paths and images,
  `find("id")` picks one element, `paths()` lists the contours.

## 0.4.0 — 2026-10-20

- Creates projects on oxframe 0.4.0.
- The agent files explain the `Camera` and that `.to(..)` takes theme tokens
  and `&str` directly (no `.into()`).
- The new-project template no longer writes `.to("…".into(), …)`. In an
  existing project, `oxframe upgrade` to 0.4.0 needs the same edit: drop the
  `.into()` inside `.to(..)`.

## 0.3.0 — 2026-10-20

- Creates projects on oxframe 0.3.0 (`oxframe upgrade` moves existing ones).
- The agent files teach the new style: `stagger`, `all` over collections,
  `any`, `repeat`, the node helpers (`fade_in`, `pop_in`, `slide_in`,
  `draw_on`) and the `;`-in-a-closure fix. No `Box::pin` or `BoxFuture`.

## 0.2.6 — 2026-10-20

The first published release after 0.2.1: it also contains everything listed
under 0.2.2–0.2.5 below, which were not published separately.

### Added

- `oxframe mcp`, an MCP server on stdin and stdout. Tools: `check`,
  `storyboard`, `render_frame`, `inspect`, `list_components`, `docs_search`
  and `render`. Newline-delimited JSON-RPC, and `Content-Length` frames.
- `--json` on every command. One object on stdout. A compile error carries
  the compiler text and a hint for the mistakes agents make most often: a
  forgotten `.await`, a type mismatch, a method on the wrong value, a missing
  type, a builder never passed to `add`, a name outside the prelude, the
  wrong number of arguments, a builder used after `add`, `all` without its
  tuple, and a text tween given an `&str`.
- Agent files link `llms.txt` and `llms-full.txt` for the project's docs series.
- Creates projects on oxframe 0.2.6 (`oxframe upgrade` moves existing ones).

## 0.2.5 — 2026-10-20

### Added

- Agent files list the explainer components (`TitleCard`, `BulletList`,
  `CodeExplainer`, `FlowDiagram`, `Callout`, `LowerThird`, `EndCard`,
  `ChapterMarker`) with a one-line example each.
- Creates projects on oxframe 0.2.5 (`oxframe upgrade` moves existing ones).

## 0.2.4 — 2026-10-20

### Added

- `oxframe render --aspect 16:9|9:16|1:1|all`. `all` writes one file per format
  the project declares with `Project::aspects`.
- `oxframe check` lints each of those formats. Extra storyboards are
  `storyboard-9x16.png` and `storyboard-1x1.png`.
- Creates projects on oxframe 0.2.4 (`oxframe upgrade` moves existing ones).
- Agent files describe `next_to`, `to_edge`, `pct` and aspects.

## 0.2.3 — 2026-10-20

### Added

- `[theme]` in `oxframe.toml` (`name`, `accent`) overrides the theme compiled
  into the project for `dev`, `render`, `check`, `inspect` and `snapshot`.
- Creates projects on oxframe 0.2.3 (`oxframe upgrade` moves existing ones).
- Agent files mention theme tokens.

## 0.2.2 — 2026-10-20

### Added

- `oxframe check` now also lints the picture: it writes `out/check/storyboard.png`
  and `report.json` (rule, severity, time, node, `file:line`, hint) and exits 1
  when any issue is an error. `--frames` saves the storyboard cells at full size.
  Thresholds are `[check]` in `oxframe.toml`.
- `oxframe inspect --at <seconds>` prints the scene tree at that time as JSON.
- Agent files (`AGENTS.md`, the Claude skill) tell the agent to run `oxframe check`
  and read the storyboard after every visual change, and to name nodes with `.name()`.
- Creates projects on oxframe 0.2.2 (`oxframe upgrade` moves existing ones).

## 0.2.1 — 2026-10-20

- Creates projects on oxframe 0.2.1 (`oxframe upgrade` moves existing ones).

- `AGENTS.md` and the Claude Code skill link the docs and the docs.rs API
  reference of the exact oxframe version the project is pinned to.
- README: development-status notice.

## 0.1.1 — 2026-10-20

- `oxframe dev` keeps the editor token per project
  (`target/oxframe/dev-token`), so an open tab reconnects after a restart.
- A stale or missing token closes the editor connection with code 4401; the
  editor says "Link expired" instead of reconnecting forever.

## 0.1.0 — 2026-10-19

First release: `init`, `dev` (editor with hot reload), `render`, `snapshot`,
`check`, `agents`, `upgrade`, and a daily update hint.
