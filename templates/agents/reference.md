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

## Sound

```rust
.audio(Audio::track("assets/music.mp3").duck_under_voice().volume(0.8).master_fade(0.2, 0.4))
// a path still works: .audio("assets/music.mp3")

sound("assets/whoosh.wav").volume(0.6).fade_in(0.05).play();       // starts now, does not wait
sound("assets/click.wav").trim(0.0, 0.2).priority().await;         // waits out the slice
```

`.play()` records the clip at the current frame and continues. Awaiting waits until the clip ends. `.trim(start, end)` is the length of that wait and does not need to open the file. `.priority()` ducks a soundtrack built with `.duck_under_voice()`. Effects are clips on the timeline and play during playback and seeking. `oxframe render` mixes them with the soundtrack before ffmpeg encodes; `--no-audio` drops both, and `--range` keeps only the part inside the range. Dragging the waveform changes the soundtrack offset in `meta.json`.

## Voice

```rust
let line = voice("First draw a square, then rotate it").start();
line.at_word("square").await;          // waits until that word
square.draw_on().await;
line.await;                             // waits until the line ends
```

`voice("one two").start()` speaks now and returns a handle. `.at_word()` matches the word with surrounding punctuation stripped and panics if it is missing, listing the words. Awaiting the line waits until the end. A line is `.priority()`, so it ducks a soundtrack built with `.duck_under_voice()`. Await the line: one that runs past the scene is an error from `oxframe check`, and two overlapping lines are a warning.

oxframe does not call a speech service. Implement `TtsProvider` and pass it to `Project::tts`. `synthesize` gets `TtsRequest` (`text`, `voice`) and returns `Synthesis`: `rate`, mono `i16` `samples`, and `words` (`TimedWord`: `text`, `start`, `end` in seconds from the start of the samples). `.at_word()` waits on those times. Read the API key from the environment inside the provider.

```rust
use oxframe::{Synthesis, TtsProvider, TtsRequest};

struct ElevenLabs;

impl TtsProvider for ElevenLabs {
    fn id(&self) -> &str { "elevenlabs" }   // part of the cache key

    fn synthesize(&self, req: &TtsRequest) -> Result<Synthesis, String> {
        let key = std::env::var("ELEVENLABS_API_KEY")
            .map_err(|_| "set ELEVENLABS_API_KEY".to_string())?;
        let (rate, samples, words) = speak(&key, req.voice, req.text)?;
        Ok(Synthesis { rate, samples, words })
    }
}

Project::new("explainer").tts(ElevenLabs).speaker("Rachel")
    .audio(Audio::track("assets/music.mp3").duck_under_voice())
    .scene(intro())
```

The first render writes `tts/{hash}.wav` and `tts/{hash}.json`. The next render plays the file. Commit that folder so CI has the audio without a key. A missing key or a failed call falls back to a speech-rate estimate (2.5 words per second, at least 0.3s) and the scene still runs, with no speech audio. An estimate is not cached. With no provider, a committed cache is still found by the line text. The playground always estimates.

`oxframe render --subtitles out.srt` writes the lines. A path ending in `.vtt` writes WebVTT. `--no-audio` still writes them, and `--range` shifts the cues.

## Transitions

At the start of any scene except the first:

```rust
transition(Transition::slide(Dir::Left)).await;          // theme duration and enter easing
transition(Transition::fade().duration(0.6).ease(presets::LINEAR)).await;
transition(Transition::wipe(Dir::Right).overlap()).await; // previous scene keeps playing
transition(Transition::zoom_in()).await;
transition(Transition::zoom_out()).await;
transition(Transition::custom(|t| Blend {                 // t is eased progress, 0..=1
    from: LayerPose::new().opacity(1.0 - t),
    to: LayerPose::new().opacity(t).offset(1.0 - t, 0.0),
    incoming_on_top: true,
    mix: false,
})).await;
```

`Dir` is `Left`, `Right`, `Up`, `Down`. The call waits, so nodes added before it show during the blend and nodes added after show when it ends. The first scene ignores it. Length is stored in `meta.json` under `transition.seconds` and the timeline can drag it. `oxframe render --scene name` draws that scene without the blend into it. `oxframe check` puts a blend frame on the storyboard. `oxframe inspect` adds `"transition": { "kind", "dir", "progress", "overlap" }` while one is running.

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
| `Code` | `.code(src) .language("rust"/"js"/"python"/"json"/"toml"/"shell") .theme(CodeTheme::dark()) .focus(first, last) .dim(0.3)` | `font_size focus dim`; `.code().to(next, secs)` diffs tokens (not a typewriter); `.insert(line, col, text, secs)`, `.remove(((l, c), (l, c)), secs)`, `.replace(range, text, secs)`, `.flash()` on that tween; `.focus_lines(a, b, secs)` — lines are 1-based, columns 0-based |
| `Img` | `.src(include_bytes!("../assets/a.png")) .file("assets/a.png") .width .height` (PNG/JPEG/SVG) | `width height` |
| `Svg` | `.src(include_bytes!("../assets/logo.svg")) .file("assets/logo.svg") .width .height` (aspect kept if only one is set) | `width height`; `.find("id")` the element with that id; `.paths()` every contour in paint order. Filters, masks and clip paths are skipped with a warning |
| `Layout` | `.row() .column() .wrap(bool) .grid(cols) .gap .padding .align(LayoutAlign::..) .justify(Justify::..) .width .height .radius` | `gap padding width height radius` |
| `Grid` | `.width .height .spacing` | `width height spacing` |
| `Group` | — | — |

## Theme

`Project::new("name").theme(Theme::dark().accent(color!("#68abdf")))` — also `Theme::light()`, `midnight`, `paper`, `vivid`. Colors: `Background Surface Text Muted Accent OnAccent Success Warning Danger` (`.fill(Accent)`, `.stroke(Muted)`). Type: `.display() .h1() .h2() .body() .caption() .code()`. Space and radius: `Xs Sm Md Lg Xl` (`.padding(Md)`, `.gap(Sm)`, `.margin(Lg)`, `.radius(Lg)`). Shadow: `.elevation(Md)`. Motion: `.to(v, Normal)` (`Fast`/`Slow` too); a tween takes tokens directly, `.fill().to(Accent, Normal)`. `node.fade_in()` / `fade_out()` use the theme's normal duration. `[theme]` in `oxframe.toml` (`name`, `accent`) overrides the compiled theme. Text with no fill uses `Text`.

Every node: `.name("title")` (readable in check/inspect; otherwise the `add` call site), `x y rotation scale(Vec2) opacity fill stroke line_width start end`
(`start/end` trim outlines 0..1), `fill_gradient(Gradient) stroke_gradient dash(vec) dash_offset
line_cap line_join blur shadow(color, blur, (dx, dy)) glow(color, blur) clip(bool) blend(BlendMode)
z_index anchor((ax, ay))`, and inside a `Layout`: `grow shrink basis margin align_self`.
Builders take values; handles return `Signal`s of the same names. Children: `.child(builder)` on a
builder, or `handle.add(builder)` for a handle. `node.remove()`, `node.reparent(other)`.

## Components

`view.add(TitleCard::new("Binary search").subtitle("O(log n)"))`, `ChapterMarker::new("1  Search")`, `BulletList::new(["Take the middle", "Compare"])`, `CodeExplainer::new(src).step(1, 1, "the middle")`, `FlowDiagram::new().edge("Client", "API").down()`, `Callout::new(node).text("here")`, `LowerThird::new("Ada").role("walks the half")`, `EndCard::new("Done").note("thanks")`. All of them: `enter().await`, `next().await`, `exit().await`. They read the theme and the safe area.

## Camera

Zoom, pan and turn the view of everything added to the camera; nodes added
to `view` stay on screen (titles, captions).

```rust
let cam = view.add(Camera::new());
let diagram = cam.add(Group::new());            // build the world inside it
cam.fit(diagram, 120.0, 0.0).await;             // show all of it (padding px, seconds)
cam.center_on(node, 1.0).await;                 // bring a node to the middle
cam.zoom().to(2.5, 1.0).await;                  // x(), y() (point looked at), zoom(), rotation()
cam.x().bind(move || ball.x().get());           // follow a moving node
cam.reset(1.2).await;                           // back to x = y = 0, zoom 1, rotation 0
```

`oxframe check` does not report `out_of_frame` for nodes inside a camera.

## Position

`node.next_to(other, Side::Right)` (also `Left`, `Top`, `Bottom`) sits outside the neighbor with the theme's `Md` gap, and follows it every frame. `align_to(other, Edge::Top)`, `to_edge(Edge::Bottom)` (pins to the safe area, not the frame edge), `center_in(other)`. The same methods exist on builders: `.h1().to_edge(Edge::Top)`. `.width(pct(80))` / `.height(pct(50))` is a fraction of the frame, or of the parent when the parent has a fixed size. `view.safe_area()` is the scene-space rectangle; `view.is_portrait()` is true when the frame is taller than wide. `Project::aspects([Aspect::Landscape, Aspect::Portrait, Aspect::Square])` — `oxframe check` lints each one, `oxframe render --aspect 9:16` or `--aspect all` exports them.

## Animation

```rust
s.x().to(300.0, 1.0).await;                                  // seconds
s.x().to(300.0, 1.0).ease(presets::OUT_BACK).await;           // presets::*, oxframe::bounce_out, oxframe::elastic_out, |t| t * t
s.fill().to(color!("#68abdf"), 0.5).await;                   // colors blend in OKLab
s.scale().to(Vec2::new(2.0, 2.0), 0.5).await;
all((a.x().to(1.0, 1.0), b.y().to(2.0, 0.5))).await;          // together (tuple ≤ 12)
all(cards.iter().map(|c| c.fade_in())).await;                 // together, any count
stagger(0.1, &cards, |c| c.pop_in()).await;                   // staggered starts
stagger(0.1, &cards, |c| async move { c.pop_in().await; c.fade_out().await; }).await;
let first = any((a.x().to(1.0, 2.0), wait_for(0.5))).await;   // first to finish (index)
repeat(3, || dot.scale().to(Vec2::new(1.2, 1.2), 0.3)).await;
spawn(loop_while(|| true, move || dot.rotation().to(dot.rotation().get() + 360.0, 2.0)));  // background loop
wait_for(0.5).await;  next_frame().await;
let h = spawn(async move { /* … */ }); h.await;              // background task
```

Helpers on every node, timed by the theme (`Fast`/`Normal`/`Slow`), with
`.duration(s)` to override:

```rust
title.fade_in().await;  title.fade_out().duration(0.3).await;
badge.pop_in().await;                     // scale from 0 with overshoot
card.slide_in(Side::Left).await;          // slides in from the left, fading in
path.draw_on().await;                     // outline drawn start → end
```

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
`oxframe inspect --at <seconds> [--scene s] [--graph]` — JSON scene tree at that time. `--graph` adds the render passes.
`oxframe render [--out out.mp4|.webm|.mov|dir] [--format mp4|webm|prores|png] [--scene s] [--range 1..3] [--size 1280x720] [--cpu]`
`oxframe snapshot --at <seconds> [--at …] [--scene s] [--size WxH] [--out file.png]`
