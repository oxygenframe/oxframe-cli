//! A closing line and a link in a pill that pulses once.
//!
//! `blocks::call_to_action(view, "Try it today", "oxframe.dev").await`

use oxframe::prelude::*;

/// Shows the headline, pops the link in under it and pulses it. Returns
/// the block's group.
pub async fn call_to_action(view: View, headline: &str, link: &str) -> Group {
    let group = view.add(Group::new().name("call_to_action"));
    let title = group.add(
        Txt::new()
            .text(headline)
            .display()
            .max_width(1500.0)
            .y(-90.0)
            .opacity(0.0)
            .name("cta_headline"),
    );
    let pill = group.add(Group::new().y(120.0).name("cta_link"));
    let _ = pill.add(
        Rect::new()
            .width(80.0 + 26.0 * link.chars().count() as f64)
            .height(96.0)
            .radius(48.0)
            .fill(Accent),
    );
    let _ = pill.add(Txt::new().text(link).h2().fill(Background));
    title.fade_in().await;
    pill.pop_in().await;
    pill.scale().to(1.08, 0.25).await;
    pill.scale().to(1.0, 0.25).await;
    group
}
