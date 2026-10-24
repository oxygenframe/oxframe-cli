//! Up to four feature cards in a row, each with a title and a line of text.
//!
//! `blocks::feature_grid(view, &[("Fast", "Renders natively"), ("Checked", "Agents read the report")]).await`

use oxframe::prelude::*;

const CARD_W: f64 = 400.0;
const CARD_H: f64 = 300.0;
const GAP: f64 = 40.0;

/// Slides the cards up one after another. Returns the block's group.
pub async fn feature_grid(view: View, features: &[(&str, &str)]) -> Group {
    let group = view.add(Group::new().name("feature_grid"));
    let n = features.len().min(4);
    let total = n as f64 * CARD_W + (n.saturating_sub(1)) as f64 * GAP;
    let mut cards = Vec::new();
    for (i, (title, text)) in features.iter().take(n).enumerate() {
        let x = -total / 2.0 + CARD_W / 2.0 + i as f64 * (CARD_W + GAP);
        let card = group.add(Group::new().x(x).opacity(0.0).name(format!("feature_{i}")));
        let _ = card.add(
            Rect::new()
                .width(CARD_W)
                .height(CARD_H)
                .radius(24.0)
                .fill(Surface),
        );
        let _ = card.add(Rect::new().width(64.0).height(8.0).radius(4.0).fill(Accent).y(-100.0));
        let _ = card.add(Txt::new().text(*title).h2().max_width(CARD_W - 60.0).y(-40.0));
        let _ = card.add(
            Txt::new()
                .text(*text)
                .body()
                .fill(Muted)
                .max_width(CARD_W - 60.0)
                .y(60.0),
        );
        cards.push(card);
    }
    stagger(0.2, &cards, |c| c.slide_in(Side::Bottom)).await;
    group
}
