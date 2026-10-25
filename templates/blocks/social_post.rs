//! A social media post: author, text, then the likes counting up.
//!
//! `blocks::social_post(view, "Ada", "@ada", "Rendered my first video in Rust today.", 1284.0).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1100.0;
const PAD: f64 = 56.0;

/// Slides the card in, then counts the likes from 0 to `likes`. Returns the
/// block's group.
pub async fn social_post(view: View, name: &str, handle: &str, text: &str, likes: f64) -> Group {
    let group = view.add(Group::new().name("social_post"));
    let card = group.add(Group::new().opacity(0.0));
    let left = -WIDTH / 2.0 + PAD;
    let body = card.add(
        Txt::new()
            .text(text)
            .h2()
            .max_width(WIDTH - 2.0 * PAD)
            .anchor((-1.0, -1.0))
            .x(left)
            .name("post_text"),
    );
    let (_, text_h) = body.size();
    let height = 120.0 + 40.0 + text_h + 40.0 + 60.0 + 2.0 * PAD;
    let top = -height / 2.0;
    body.y().set(top + PAD + 120.0 + 40.0);
    let back = card.add(
        Rect::new()
            .width(WIDTH)
            .height(height)
            .radius(32.0)
            .fill(Surface)
            .name("post_card"),
    );
    back.z_index().set(-1);
    let initial: String = name.chars().take(1).collect();
    let _ = card.add(Circle::new().size(96.0).fill(Accent).x(left + 48.0).y(top + PAD + 48.0));
    let _ = card.add(
        Txt::new()
            .text(initial)
            .h2()
            .weight(700)
            .fill(OnAccent)
            .x(left + 48.0)
            .y(top + PAD + 48.0),
    );
    let _ = card.add(
        Txt::new()
            .text(name)
            .h2()
            .weight(700)
            .anchor((-1.0, 0.0))
            .x(left + 130.0)
            .y(top + PAD + 26.0),
    );
    let _ = card.add(
        Txt::new()
            .text(handle)
            .body()
            .fill(Muted)
            .anchor((-1.0, 0.0))
            .x(left + 130.0)
            .y(top + PAD + 74.0),
    );
    let count = Signal::new(0.0f64);
    let shown = count.clone();
    let likes_text = card.add(
        Txt::new()
            .text("♥ 0")
            .h2()
            .fill(Danger)
            .anchor((-1.0, 0.0))
            .x(left)
            .y(-top - PAD - 30.0)
            .name("post_likes"),
    );
    likes_text
        .text()
        .bind(move || format!("♥ {:.0}", shown.get()));
    card.slide_in(Side::Bottom).await;
    count.to(likes, 1.5).ease(presets::OUT_CUBIC).await;
    group
}
