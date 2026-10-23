---
name: oxframe
description: Write and edit oxframe animation scenes (Rust, `#[oxframe::scene]`), verify them with `oxframe check` (storyboard.png and report.json), and render videos. Use for any change to scenes, timing, shapes, text, layout or effects in this project.
---
<!-- oxframe-agents {{VERSION}} -->
# oxframe scenes

The project is on oxframe {{SDK_VERSION}}: docs https://oxframe.dev/docs/{{DOCS_SERIES}},
model text https://oxframe.dev/docs/{{DOCS_SERIES}}/llms.txt
(and https://oxframe.dev/docs/{{DOCS_SERIES}}/llms-full.txt),
API https://docs.rs/oxframe/{{SDK_VERSION}}/oxframe/.
`oxframe mcp` exposes check, inspect, render_graph, render_frame and docs_search. `oxframe --json check` prints one JSON object.

## Workflow

1. Read `src/main.rs` and the scene's `src/<scene>.meta.json` (markers/tunables already tuned by a human — keep them).
2. Make the change. Prefer small, readable steps: one `.await` per beat of the story, `all((..))` for things that happen together.
3. `oxframe check`. Fix compile errors first (see AGENTS.md, "Common compile errors"). Then open `out/check/storyboard.png` and `out/check/report.json`. Each issue names a rule, a time range, a node and `file:line`. An issue on a node also has `crop`, a close-up PNG under `out/check/issues/` with the node outlined: open it.
4. Fix and run `oxframe check` again until it exits 0. Warnings do not fail the command; errors do.
5. `oxframe inspect --at <t>` when a frame looks wrong and you need the node tree (kind, name, bounds, text). Add `--graph` to see render passes. `oxframe snapshot --at <t>` for one full-size still.
6. Only then report. Mention the storyboard and which issues you fixed.

Never run `oxframe dev` (interactive server). Never hand-edit `meta.json` unless asked.

## API

The full cheat sheet is in [reference.md](reference.md): nodes and their
properties, flow functions, easing, signals, text, layout, effects, meta helpers.

## Recipes

Reach for the right node before writing the scene:

- **Explaining code** (what an expression does, why a line matters, a bug fix):
  a `Code` node is the tool, not plain `Txt`. Show the source, then walk
  attention through it with `.focus(first, last)` or `.focus_lines(a, b, secs)`
  and `.dim(0.3)` on the rest; to explain parts of one expression, focus each
  column range in turn. Morph to the next version with `.code().to(next, secs)`,
  or use `.insert`/`.remove`/`.replace`. For a line-by-line walkthrough with a
  note per step, `CodeExplainer::new(src).step(line, line, "note")`.
- **Vertical (9:16) or square short:** `Project::new(..).size(1080, 1920)` for
  portrait, `(1080, 1080)` for square — or `Project::aspects([Aspect::Portrait])`
  and `oxframe render --aspect 9:16`. `view.is_portrait()` adapts the layout.
- **Charts and math:** `Axes`/`NumberLine` with `.plot(..)`, `bars`, `scatter`;
  `Matrix`, `axes.vector(..)`. A title, bullets or an end card: the components
  (`TitleCard`, `BulletList`, `EndCard`), not hand-placed `Txt`.
