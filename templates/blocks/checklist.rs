//! A title and a list whose items are ticked off one after another.
//!
//! `blocks::checklist(view, "Before you ship", &["Tests pass", "Docs updated"]).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1100.0;

/// Shows the title and the open items, then ticks each one. Returns the
/// block's group.
pub async fn checklist(view: View, title: &str, items: &[&str]) -> Group {
    let group = view.add(Group::new().name("checklist"));
    let heading = group.add(
        Txt::new()
            .text(title)
            .h1()
            .y(-300.0)
            .opacity(0.0)
            .name("checklist_title"),
    );
    let list = group.add(
        Layout::new()
            .column()
            .gap(36.0)
            .align(LayoutAlign::Start)
            .width(WIDTH)
            .y(60.0)
            .opacity(0.0),
    );
    let mut ticks = Vec::new();
    for &text in items {
        let row = list.add(
            Layout::new()
                .row()
                .gap(32.0)
                .align(LayoutAlign::Center)
                .justify(Justify::Start)
                .width(WIDTH),
        );
        let boxed = row.add(
            Rect::new()
                .size(56.0)
                .radius(12.0)
                .fill(Surface)
                .stroke(Muted)
                .line_width(3.0),
        );
        let tick = boxed.add(
            Path::new()
                .svg("M-14 0 L-4 11 L16 -12")
                .stroke(Success)
                .line_width(7.0)
                .end(0.0),
        );
        let label = row.add(Txt::new().text(text).h2().max_width(WIDTH - 100.0));
        ticks.push((boxed, tick, label));
    }
    all((heading.fade_in(), list.fade_in())).await;
    for (boxed, tick, label) in ticks {
        all((
            tick.end().to(1.0, 0.35),
            boxed.stroke().to(Success, 0.35),
            label.fill().to(Muted, 0.35),
        ))
        .await;
        wait_for(0.35).await;
    }
    group
}
