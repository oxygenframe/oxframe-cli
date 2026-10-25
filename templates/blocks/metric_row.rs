//! Up to four numbers side by side, each counting up over its label.
//!
//! `blocks::metric_row(view, &[(98.0, "% uptime"), (1.2, "s to render"), (47.0, "prompts")]).await`

use oxframe::prelude::*;

const CELL: f64 = 420.0;

/// Counts every number up at once, the labels fading in under them. A value
/// with a fraction keeps one decimal. Returns the block's group.
pub async fn metric_row(view: View, metrics: &[(f64, &str)]) -> Group {
    let group = view.add(Group::new().name("metric_row"));
    let n = metrics.len().min(4);
    let left = -(n as f64 - 1.0) * CELL / 2.0;
    let mut counts = Vec::new();
    let mut labels = Vec::new();
    for (i, &(value, label)) in metrics.iter().take(n).enumerate() {
        let x = left + i as f64 * CELL;
        let count = Signal::new(0.0f64);
        let decimals = if value.fract() == 0.0 { 0 } else { 1 };
        let number = group.add(
            Txt::new()
                .text("0")
                .display()
                .fill(Accent)
                .x(x)
                .y(-40.0)
                .name(format!("metric_{i}")),
        );
        let shown = count.clone();
        number
            .text()
            .bind(move || format!("{:.*}", decimals, shown.get()));
        labels.push(group.add(
            Txt::new()
                .text(label)
                .body()
                .fill(Muted)
                .max_width(CELL - 40.0)
                .x(x)
                .y(70.0)
                .opacity(0.0),
        ));
        counts.push((count, value));
    }
    let rises: Vec<_> = counts
        .iter()
        .map(|(c, v)| c.to(*v, 1.5).ease(presets::OUT_CUBIC))
        .collect();
    all((all(rises), stagger(0.15, &labels, |l| l.fade_in()))).await;
    group
}
