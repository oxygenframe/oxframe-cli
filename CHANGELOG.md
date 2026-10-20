# Changelog

`oxframe-cli` is released together with the oxframe SDK crates under the
same version (see the SDK's
[CHANGELOG](https://github.com/oxygenframe/oxframe-sdk/blob/main/CHANGELOG.md)).

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
