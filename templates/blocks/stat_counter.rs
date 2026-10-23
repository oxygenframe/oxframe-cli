//! A number that counts up to its value, with a label under it.
//!
//! `blocks::stat_counter(view, 1280.0, "stars on GitHub").await`

use oxframe::prelude::*;

/// Counts from 0 to `value` in 1.5 s while the label fades in. Returns the
/// block's group, so you can move it or fade it out.
pub async fn stat_counter(view: View, value: f64, label: &str) -> Group {
    let group = view.add(Group::new().name("stat_counter"));
    let count = Signal::new(0.0f64);
    let number = group.add(
        Txt::new()
            .text("0")
            .display()
            .font_size(180.0)
            .fill(Accent)
            .y(-50.0)
            .name("stat_number"),
    );
    let shown = count.clone();
    number.text().bind(move || format!("{:.0}", shown.get()));
    let caption = group.add(
        Txt::new()
            .text(label)
            .h2()
            .fill(Muted)
            .y(90.0)
            .opacity(0.0)
            .name("stat_label"),
    );
    all((
        count.to(value, 1.5).ease(presets::OUT_CUBIC),
        caption.fade_in(),
    ))
    .await;
    group
}
