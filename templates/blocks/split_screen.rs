//! Two columns side by side, a title and lines in each, a divider between.
//!
//! `blocks::split_screen(view, ("Before", &["12 s renders", "No preview"]), ("After", &["1.2 s renders", "Live editor"])).await`

use oxframe::prelude::*;

const COLUMN: f64 = 760.0;
const LINE: f64 = 80.0;

/// Draws the divider, then fills the left column and the right one, line
/// by line. The right title is in the accent color. Returns the block's
/// group.
pub async fn split_screen(
    view: View,
    left: (&str, &[&str]),
    right: (&str, &[&str]),
) -> Group {
    let group = view.add(Group::new().name("split_screen"));
    let rows = left.1.len().max(right.1.len()) as f64;
    let height = 140.0 + rows * LINE;
    let top = -height / 2.0;
    let divider = group.add(
        Line::new()
            .points([(0.0, top - 40.0), (0.0, -top + 40.0)])
            .stroke(Surface)
            .line_width(6.0)
            .end(0.0),
    );
    divider.end().to(1.0, 0.5).await;
    for (side, (title, lines)) in [(-1.0, left), (1.0, right)] {
        let x = side * (COLUMN / 2.0 + 40.0);
        let heading = group.add(
            Txt::new()
                .text(title)
                .h1()
                .fill(if side > 0.0 { Accent } else { Text })
                .max_width(COLUMN)
                .x(x)
                .y(top + 40.0)
                .opacity(0.0)
                .name(if side > 0.0 { "split_right" } else { "split_left" }),
        );
        heading.fade_in().await;
        let items: Vec<_> = lines
            .iter()
            .enumerate()
            .map(|(i, line)| {
                group.add(
                    Txt::new()
                        .text(*line)
                        .h2()
                        .fill(if side > 0.0 { Text } else { Muted })
                        .max_width(COLUMN)
                        .x(x)
                        .y(top + 150.0 + i as f64 * LINE)
                        .opacity(0.0),
                )
            })
            .collect();
        stagger(0.15, &items, |t| t.fade_in()).await;
        wait_for(0.3).await;
    }
    group
}
