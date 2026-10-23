//! Two cards side by side: how it was, then how it is.
//!
//! `blocks::before_after(view, "12 s to render", "1.2 s to render").await`

use oxframe::prelude::*;

/// Shows "Before" on the left, then slides "After" in on the right in the
/// accent color. Returns the block's group.
pub async fn before_after(view: View, before: &str, after: &str) -> Group {
    let group = view.add(Group::new().name("before_after"));
    let left = card(group, "Before", before, Muted, -330.0);
    let right = card(group, "After", after, Accent, 330.0);
    left.fade_in().await;
    wait_for(0.4).await;
    right.slide_in(Side::Right).await;
    group
}

fn card(group: Group, title: &str, body: &str, tone: oxframe::Ink, x: f64) -> Group {
    let side = group.add(Group::new().x(x).opacity(0.0).name(format!("{title}_card")));
    let _ = side.add(
        Rect::new()
            .width(560.0)
            .height(360.0)
            .radius(24.0)
            .fill(Surface),
    );
    let _ = side.add(Txt::new().text(title).body().fill(tone).y(-110.0));
    let _ = side.add(Txt::new().text(body).h2().max_width(480.0).y(20.0));
    side
}
