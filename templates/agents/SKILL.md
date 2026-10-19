---
name: oxframe
description: Write and edit oxframe animation scenes (Rust, `#[oxframe::scene]`), check them visually with `oxframe snapshot`, and render videos. Use for any change to scenes, timing, shapes, text, layout or effects in this project.
---
<!-- oxframe-agents {{VERSION}} -->
# oxframe scenes

## Workflow

1. Read `src/main.rs` and the scene's `src/<scene>.meta.json` (markers/tunables already tuned by a human — keep them).
2. Make the change. Prefer small, readable steps: one `.await` per beat of the story, `all((..))` for things that happen together.
3. `oxframe check` — fix compile errors (see AGENTS.md, "Common compile errors").
4. `oxframe snapshot --at <t> [--at <t2> …]` at the moments you changed; open the PNGs and verify. Use `cargo run -q -- info` to get scene length and marker times.
5. Only then report. Mention which timestamps you checked.

Never run `oxframe dev` (interactive server). Never hand-edit `meta.json` unless asked.

## API

The full cheat sheet is in [reference.md](reference.md): nodes and their
properties, flow functions, easing, signals, text, layout, effects, meta helpers.
