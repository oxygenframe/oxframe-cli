# oxframe-cli

> **Status:** oxframe is under very active development. The API is not stable yet and may change from version to version. Pin the exact version in your Cargo.toml and check the [changelog](https://github.com/oxygenframe/oxframe-sdk/blob/main/CHANGELOG.md) before upgrading.

The `oxframe` command for [oxframe](https://github.com/oxygenframe/oxframe-sdk)
projects: create, preview with hot reload, render, check.

```sh
cargo install oxframe-cli          # or: curl -fsSL …/install.sh | sh
oxframe init hello && cd hello
oxframe dev --open                 # editor on http://127.0.0.1:3000/?token=…
oxframe render                     # out/hello.mp4
oxframe check                      # compile, then storyboard + report.json
oxframe inspect --at 2.5           # scene tree at that time, as JSON (--graph adds passes)
oxframe snapshot --at 1.5          # one frame as PNG in out/snapshots/
oxframe agents                     # refresh AGENTS.md and the Claude Code skill
oxframe upgrade                    # move the project to this CLI's SDK version
```

## Updating

```sh
cargo install oxframe-cli --force  # the CLI
oxframe upgrade                    # then, in each project: SDK pin, Cargo.lock, agent files
```

Projects pin the SDK (`oxframe = "=0.10.0"`), so a new release never changes
an existing project by itself. Once a day the CLI checks crates.io in the
background (`cargo search`) and mentions a newer version; set
`OXFRAME_NO_UPDATE_CHECK=1` to turn that off (it is off on CI).

Full reference: <https://oxframe.dev/docs/cli>.

## AI coding agents

`oxframe init` writes `AGENTS.md`, `CLAUDE.md` (`@AGENTS.md`) and a skill in
`.claude/skills/oxframe/` (workflow + API cheat sheet). They tell agents to
verify visual changes with `oxframe check` (storyboard and `report.json`) and never to run `oxframe dev`.
`oxframe agents` refreshes them after an upgrade; files without the
`oxframe-agents` marker (edited by you) are kept unless `--force`.

## How `oxframe dev` works

```
browser ──WS──▶ oxframe dev (127.0.0.1:3000) ──WS──▶ <project> preview --port 0
                 serves the embedded editor           restarted on every change
                 watches src/, assets/, Cargo.toml → cargo build
```

- Changes (debounced 200 ms) trigger `cargo build`; the editor shows
  *Building…*, then either *Reloaded* (it re-seeks to the same frame) or the
  compiler errors with file and line — while the previous build keeps running.
- The old process gets `Shutdown`, 2 s grace, then is killed (process tree on
  Windows). Crashes restart the project up to three times.
- Security: loopback only, `Origin` check, one-time token in the URL; the CLI
  runs only cargo and your project and makes no network requests.

Exit codes: 0 ok, 1 build error, 2 environment (ffmpeg, port), 3 version mismatch.

## The embedded editor

`ui/dist` is the `editor-ui` release pinned in `ui.lock`, packed into the
binary with `rust-embed` (no network access in `build.rs` or `cargo install`).

```sh
./scripts/fetch-ui.sh             # download the pinned release
just build-ui ../oxframe-web       # or build it from a sibling checkout
oxframe dev --ui-dir ../oxframe-web/packages/editor-ui/dist   # develop the editor
```

`ui/debug-client/` is a minimal single-file protocol client kept for debugging.

## Development

```sh
cargo test                                          # unit tests, ui.lock check
cargo test --test e2e -- --ignored --nocapture      # init → dev → reload → render (needs ../oxframe-sdk)
```

Licensed under MIT or Apache-2.0.
