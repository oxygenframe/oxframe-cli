---
name: oxframe
description: Write and edit oxframe animation scenes (Rust, `#[oxframe::scene]`), verify them with `oxframe check` (storyboard.png and report.json), and render videos. Use for any change to scenes, timing, shapes, text, layout or effects in this project.
---
<!-- oxframe-agents {{VERSION}} -->
# oxframe scenes

The project is on oxframe {{SDK_VERSION}}: docs https://oxframe.dev/docs/{{DOCS_SERIES}},
API https://docs.rs/oxframe/{{SDK_VERSION}}/oxframe/.

## Workflow

1. Read `src/main.rs` and the scene's `src/<scene>.meta.json` (markers/tunables already tuned by a human — keep them).
2. Make the change. Prefer small, readable steps: one `.await` per beat of the story, `all((..))` for things that happen together.
3. `oxframe check`. Fix compile errors first (see AGENTS.md, "Common compile errors"). Then open `out/check/storyboard.png` and `out/check/report.json`. Each issue names a rule, a time range, a node and `file:line`.
4. Fix and run `oxframe check` again until it exits 0. Warnings do not fail the command; errors do.
5. `oxframe inspect --at <t>` when a frame looks wrong and you need the node tree (kind, name, bounds, text). `oxframe snapshot --at <t>` for one full-size still.
6. Only then report. Mention the storyboard and which issues you fixed.

Never run `oxframe dev` (interactive server). Never hand-edit `meta.json` unless asked.

## API

The full cheat sheet is in [reference.md](reference.md): nodes and their
properties, flow functions, easing, signals, text, layout, effects, meta helpers.
