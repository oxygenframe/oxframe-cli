//! A sentence whose words pop in one after another.
//!
//! `blocks::kinetic_title(view, "Motion graphics your agent checks").await`

use oxframe::prelude::*;

/// Lays the words out in centered rows and pops them in, 0.12 s apart.
/// Returns the block's group.
pub async fn kinetic_title(view: View, text: &str) -> Group {
    let group = view.add(Group::new().name("kinetic_title"));
    let row = group.add(
        Layout::new()
            .row()
            .wrap(true)
            .gap(28.0)
            .justify(Justify::Center)
            .align(LayoutAlign::Center)
            .width(1500.0),
    );
    let words: Vec<_> = text
        .split_whitespace()
        .map(|w| row.add(Txt::new().text(w).display().opacity(0.0)))
        .collect();
    stagger(0.12, &words, |w| all((w.fade_in(), w.pop_in()))).await;
    group
}
