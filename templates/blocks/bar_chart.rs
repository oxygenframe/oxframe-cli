//! A titled bar chart whose bars grow to their values.
//!
//! `blocks::bar_chart(view, "Build time, s", &[("rustc", 3.2), ("gcc", 7.5)]).await`

use oxframe::prelude::*;

/// Draws the axes and the title, then grows each bar in turn. Returns the
/// block's group.
pub async fn bar_chart(view: View, title: &str, bars: &[(&str, f64)]) -> Group {
    let group = view.add(Group::new().name("bar_chart"));
    let heading = group.add(
        Txt::new()
            .text(title)
            .h1()
            .y(-380.0)
            .opacity(0.0)
            .name("bar_chart_title"),
    );
    let top = bars.iter().map(|b| b.1).fold(0.0, f64::max).max(1e-9) * 1.15;
    let n = bars.len().max(1) as f64;
    let axes = group.add(
        Axes::new()
            .x_range(-0.5..n - 0.5)
            .y_range(0.0..top)
            .width(1400.0)
            .height(600.0)
            .y(60.0),
    );
    let chart = axes
        .bars(bars.iter().map(|_| 0.0))
        .labels(bars.iter().map(|b| b.0));
    heading.fade_in().await;
    stagger(0.15, bars.iter().enumerate(), |(i, b)| {
        chart.value(i).to(b.1, 1.0)
    })
    .await;
    group
}
