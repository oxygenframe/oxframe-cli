# Changelog

`oxframe-cli` is released together with the oxframe SDK crates under the
same version (see the SDK's
[CHANGELOG](https://github.com/oxygenframe/oxframe-sdk/blob/main/CHANGELOG.md)).

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
