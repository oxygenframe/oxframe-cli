//! Numbered steps on a line; each one fills in as the process reaches it.
//!
//! `blocks::steps(view, &["Write the scene", "Check it", "Render"]).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1300.0;
const DOT: f64 = 96.0;

/// Shows every step, then marks them done one after another, the line
/// between them drawing on. Returns the block's group.
pub async fn steps(view: View, labels: &[&str]) -> Group {
    let group = view.add(Group::new().name("steps").opacity(0.0));
    let n = labels.len().max(1);
    let step = if n > 1 { WIDTH / (n - 1) as f64 } else { 0.0 };
    let xs: Vec<f64> = (0..n)
        .map(|i| if n > 1 { -WIDTH / 2.0 + step * i as f64 } else { 0.0 })
        .collect();
    let mut links = Vec::new();
    for pair in xs.windows(2) {
        let _ = group.add(
            Line::new()
                .points([(pair[0] + DOT / 2.0, -40.0), (pair[1] - DOT / 2.0, -40.0)])
                .stroke(Surface)
                .line_width(8.0),
        );
        links.push(group.add(
            Line::new()
                .points([(pair[0] + DOT / 2.0, -40.0), (pair[1] - DOT / 2.0, -40.0)])
                .stroke(Accent)
                .line_width(8.0)
                .end(0.0),
        ));
    }
    let mut dots = Vec::new();
    for (i, (label, x)) in labels.iter().zip(&xs).enumerate() {
        let dot = group.add(Circle::new().size(DOT).fill(Surface).x(*x).y(-40.0));
        let number = group.add(
            Txt::new()
                .text((i + 1).to_string())
                .h2()
                .weight(700)
                .x(*x)
                .y(-40.0),
        );
        let _ = group.add(
            Txt::new()
                .text(*label)
                .body()
                .max_width((step - 40.0).clamp(240.0, 420.0))
                .x(*x)
                .y(70.0)
                .name(format!("step_{i}")),
        );
        dots.push((dot, number));
    }
    group.fade_in().await;
    for (i, (dot, number)) in dots.iter().enumerate() {
        if i > 0 {
            links[i - 1].end().to(1.0, 0.4).await;
        }
        all((dot.fill().to(Accent, 0.2), number.fill().to(OnAccent, 0.2))).await;
        wait_for(0.4).await;
    }
    group
}
