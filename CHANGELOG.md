# Changelog

`oxframe-cli` is released together with the oxframe SDK crates under the
same version (see the SDK's
[CHANGELOG](https://github.com/oxygenframe/oxframe-sdk/blob/main/CHANGELOG.md)).

## 0.2.0 — 2026-10-20

- Creates projects on oxframe 0.2.0 (`oxframe upgrade` moves existing ones).

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
