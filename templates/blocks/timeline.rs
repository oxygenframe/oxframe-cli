//! Dates on a line, revealed left to right.
//!
//! `blocks::timeline(view, &[("2019", "First commit"), ("2022", "1.0"), ("2026", "1M users")]).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1400.0;

/// Draws the line, then each date and its text one after another. Returns
/// the block's group.
pub async fn timeline(view: View, events: &[(&str, &str)]) -> Group {
    let group = view.add(Group::new().name("timeline"));
    let line = group.add(
        Line::new()
            .points([(-WIDTH / 2.0, 0.0), (WIDTH / 2.0, 0.0)])
            .stroke(Muted)
            .line_width(4.0)
            .name("timeline_line"),
    );
    line.draw_on().await;
    let step = if events.len() > 1 {
        WIDTH / (events.len() - 1) as f64
    } else {
        0.0
    };
    for (i, (date, text)) in events.iter().enumerate() {
        let x = -WIDTH / 2.0 + step * i as f64;
        let x = if events.len() == 1 { 0.0 } else { x };
        let dot = group.add(Circle::new().size(28.0).fill(Accent).x(x));
        let year = group.add(
            Txt::new()
                .text(*date)
                .h2()
                .fill(Accent)
                .x(x)
                .y(-70.0)
                .opacity(0.0),
        );
        let label = group.add(
            Txt::new()
                .text(*text)
                .body()
                .max_width(300.0)
                .x(x)
                .y(80.0)
                .opacity(0.0),
        );
        all((dot.pop_in(), year.fade_in(), label.fade_in())).await;
        wait_for(0.2).await;
    }
    group
}
