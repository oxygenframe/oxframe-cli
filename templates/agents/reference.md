<!-- oxframe-agents {{VERSION}} -->
# oxframe API cheat sheet

`use oxframe::prelude::*;` brings everything below into scope.

## Scenes and project

```rust
#[oxframe::scene]
async fn intro(view: View) { /* … */ }          // view: the root; view.size() -> (w, h)

fn main() {
    Project::new("name").size(1920, 1080).fps(60).seed(7)
        .background(color!("#141414")).audio("assets/voice.mp3")
        .scene(intro()).scene(outro())             // scenes play in order
        .main();
}
```

## Nodes (builder → `view.add(..)` / `parent.add(..)` → Copy handle)

| Node | Builder | Handle signals |
|---|---|---|
| `Rect` | `.width .height .size .radius` | `width height radius` |
| `Circle` | `.size` (diameter) `.start_angle .end_angle` (deg, from 12 o'clock) `.pie(bool)` | `size start_angle end_angle` |
| `Polygon` | `.sides .size` | `sides size` |
| `Star` | `.points .size .inner` (0..1) | `points size inner` |
| `Path` | `.data(BezPath) .svg("M0 0 L10 0 Z")` | `data` (morphs between any paths) |
| `Line` | `.points([(x, y), ..]) .smooth .closed .start_arrow .end_arrow .arrow_size` | `points arrow_size` |
| `Txt` | `.text .rich([span("a").bold().italic().color(c)]) .font_size .weight .family(SANS/MONO) .align .max_width` | `text spans font_size weight`; `.morph_to("new", secs)` |
| `Code` | `.code(src) .language("rust"/"js"/"python"/"json"/"toml"/"shell") .theme(CodeTheme::dark()) .focus(first, last) .dim(0.3)` | `code font_size focus dim`; `.focus_lines(a, b, secs)` |
| `Img` | `.src(include_bytes!("../assets/a.png")) .file("assets/a.png") .width .height` (PNG/JPEG/SVG) | `width height` |
| `Layout` | `.row() .column() .wrap(bool) .grid(cols) .gap .padding .align(LayoutAlign::..) .justify(Justify::..) .width .height .radius` | `gap padding width height radius` |
| `Grid` | `.width .height .spacing` | `width height spacing` |
| `Group` | — | — |

## Theme

`Project::new("name").theme(Theme::dark().accent(color!("#68abdf")))` — also `Theme::light()`, `midnight`, `paper`, `vivid`. Colors: `Background Surface Text Muted Accent OnAccent Success Warning Danger` (`.fill(Accent)`, `.stroke(Muted)`). Type: `.display() .h1() .h2() .body() .caption() .code()`. Space and radius: `Xs Sm Md Lg Xl` (`.padding(Md)`, `.gap(Sm)`, `.margin(Lg)`, `.radius(Lg)`). Shadow: `.elevation(Md)`. Motion: `.to(v, Normal)` (`Fast`/`Slow` too); a color tween takes the token converted, `.fill().to(Accent.into(), Normal)`. `node.fade_in()` / `fade_out()` use the theme's normal duration. `[theme]` in `oxframe.toml` (`name`, `accent`) overrides the compiled theme. Text with no fill uses `Text`.

Every node: `.name("title")` (readable in check/inspect; otherwise the `add` call site), `x y rotation scale(Vec2) opacity fill stroke line_width start end`
(`start/end` trim outlines 0..1), `fill_gradient(Gradient) stroke_gradient dash(vec) dash_offset
line_cap line_join blur shadow(color, blur, (dx, dy)) glow(color, blur) clip(bool) blend(BlendMode)
z_index anchor((ax, ay))`, and inside a `Layout`: `grow shrink basis margin align_self`.
Builders take values; handles return `Signal`s of the same names. Children: `.child(builder)` on a
builder, or `handle.add(builder)` for a handle. `node.remove()`, `node.reparent(other)`.

## Position

`node.next_to(other, Side::Right)` (also `Left`, `Top`, `Bottom`) sits outside the neighbor with the theme's `Md` gap, and follows it every frame. `align_to(other, Edge::Top)`, `to_edge(Edge::Bottom)` (pins to the safe area, not the frame edge), `center_in(other)`. The same methods exist on builders: `.h1().to_edge(Edge::Top)`. `.width(pct(80))` / `.height(pct(50))` is a fraction of the frame, or of the parent when the parent has a fixed size. `view.safe_area()` is the scene-space rectangle; `view.is_portrait()` is true when the frame is taller than wide. `Project::aspects([Aspect::Landscape, Aspect::Portrait, Aspect::Square])` — `oxframe check` lints each one, `oxframe render --aspect 9:16` or `--aspect all` exports them.

## Animation

```rust
s.x().to(300.0, 1.0).await;                                  // seconds
s.x().to(300.0, 1.0).ease(presets::OUT_BACK).await;           // presets::*, oxframe::bounce_out, oxframe::elastic_out, |t| t * t
s.fill().to(color!("#68abdf"), 0.5).await;                   // colors blend in OKLab
s.scale().to(Vec2::new(2.0, 2.0), 0.5).await;
all((a.x().to(1.0, 1.0), b.y().to(2.0, 0.5))).await;          // together (≤ 12)
all_vec(vec_of_boxed).await;                                  // together, dynamic count
sequence(0.1, vec_of_boxed).await;                            // staggered starts
wait_for(0.5).await;  next_frame().await;
let h = spawn(async move { /* … */ }); h.await;              // background task
```
`vec_of_boxed: Vec<oxframe::BoxFuture>` — `Box::pin(anim)` each element.

## Signals

```rust
let x = s.x();          // Signal<f64>, cheap to clone (shared)
x.get(); x.set(10.0);
t.text().bind(move || format!("{:.0}", x.get()));   // derived, recomputed lazily
let free = Signal::new(0.0f64);                     // your own signal, animatable with .to()
```

## Editor-tunable values (stored in `<scene>.meta.json`)

```rust
wait_until("beat").await;                // timeline marker (first run records the current time)
let n = tune!("count", 7.0);             // slider, f64
x.to(1.0, 1.0).ease(ease!("slide"));     // curve editor; ease!("slide", presets::OUT_CUBIC) for a default
```

## Randomness

`random()` in `[0, 1)`, `random_range(a, b)`; seeded by `Project::seed`. Same output every run.

## Rendering

`oxframe check [--frames]` — storyboard and `report.json` in `out/check/`. `--frames` also writes full-size PNGs. Thresholds live in `oxframe.toml` `[check]`. Errors exit 1.
`oxframe inspect --at <seconds> [--scene s]` — JSON scene tree at that time.
`oxframe render [--out out.mp4|.webm|.mov|dir] [--format mp4|webm|prores|png] [--scene s] [--range 1..3] [--size 1280x720] [--cpu]`
`oxframe snapshot --at <seconds> [--at …] [--scene s] [--size WxH] [--out file.png]`
