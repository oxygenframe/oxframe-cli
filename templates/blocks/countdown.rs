//! A countdown: each number fills a ring, then the last line pops in.
//!
//! `blocks::countdown(view, 3, "Go!").await`

use oxframe::prelude::*;

/// Counts down from `from` (one second per number) and shows `done`.
/// Returns the block's group.
pub async fn countdown(view: View, from: u32, done: &str) -> Group {
    let group = view.add(Group::new().name("countdown"));
    let track = group.add(
        Circle::new()
            .size(420.0)
            .fill(Background)
            .stroke(Surface)
            .line_width(16.0),
    );
    let ring = group.add(
        Circle::new()
            .size(420.0)
            .fill(Background)
            .stroke(Accent)
            .line_width(16.0)
            .end(0.0)
            .name("countdown_ring"),
    );
    let number = group.add(
        Txt::new()
            .text(from.to_string())
            .font_size(220.0)
            .weight(700)
            .name("countdown_number"),
    );
    for n in (1..=from).rev() {
        number.text().set(n.to_string());
        ring.end().set(0.0);
        all((number.pop_in(), ring.end().to(1.0, 1.0))).await;
    }
    all((track.fade_out(), ring.fade_out())).await;
    number.text().set(done.to_string());
    number.fill().set(Accent);
    number.pop_in().await;
    group
}
