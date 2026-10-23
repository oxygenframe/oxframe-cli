//! A quote on a card, with who said it.
//!
//! `blocks::quote_card(view, "Make it work, then make it fast.", "Kent Beck").await`

use oxframe::prelude::*;

/// The card rises in, the quote fades in, then the author. Returns the
/// block's group.
pub async fn quote_card(view: View, quote: &str, author: &str) -> Group {
    let group = view.add(Group::new().name("quote_card"));
    let card = group.add(
        Rect::new()
            .width(1200.0)
            .height(420.0)
            .radius(28.0)
            .fill(Surface)
            .name("quote_card_bg"),
    );
    let text = group.add(
        Txt::new()
            .text(format!("“{quote}”"))
            .h1()
            .max_width(1040.0)
            .y(-40.0)
            .opacity(0.0)
            .name("quote_text"),
    );
    let by = group.add(
        Txt::new()
            .text(format!("— {author}"))
            .body()
            .fill(Muted)
            .y(130.0)
            .opacity(0.0)
            .name("quote_author"),
    );
    card.slide_in(Side::Bottom).await;
    text.fade_in().await;
    by.fade_in().await;
    group
}
