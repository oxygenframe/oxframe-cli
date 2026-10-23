//! A labelled bar that fills to a share, with the percentage counting up.
//!
//! `blocks::progress_bar(view, "Tests passing", 0.92).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1000.0;
const HEIGHT: f64 = 36.0;

/// Fills the bar to `fraction` (0 to 1) in 1.2 s. Returns the block's group.
pub async fn progress_bar(view: View, label: &str, fraction: f64) -> Group {
    let fraction = fraction.clamp(0.0, 1.0);
    let group = view.add(Group::new().name("progress_bar"));
    let _ = group.add(
        Txt::new()
            .text(label)
            .h2()
            .x(-WIDTH / 2.0)
            .y(-70.0)
            .anchor((-1.0, 0.0))
            .name("progress_label"),
    );
    let _ = group.add(
        Rect::new()
            .width(WIDTH)
            .height(HEIGHT)
            .radius(HEIGHT / 2.0)
            .fill(Surface)
            .name("progress_track"),
    );
    let fill = group.add(
        Rect::new()
            .width(0.0)
            .height(HEIGHT)
            .radius(HEIGHT / 2.0)
            .fill(Accent)
            .x(-WIDTH / 2.0)
            .anchor((-1.0, 0.0))
            .name("progress_fill"),
    );
    let percent = group.add(
        Txt::new()
            .text("0%")
            .h2()
            .fill(Accent)
            .x(WIDTH / 2.0)
            .y(-70.0)
            .anchor((1.0, 0.0))
            .name("progress_percent"),
    );
    let width = fill.width();
    percent
        .text()
        .bind(move || format!("{:.0}%", width.get() / WIDTH * 100.0));
    fill.width()
        .to(WIDTH * fraction, 1.2)
        .ease(presets::OUT_CUBIC)
        .await;
    group
}
