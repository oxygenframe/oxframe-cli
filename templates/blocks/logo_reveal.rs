//! A mark, the name written out next to it, and a tagline.
//!
//! `blocks::logo_reveal(view, "oxframe", "Motion graphics as Rust code").await`

use oxframe::prelude::*;

/// Pops the mark in, writes the name, fades the tagline in. The mark and the
/// name are centered as a pair whatever the name's length. Returns the
/// block's group.
pub async fn logo_reveal(view: View, name: &str, tagline: &str) -> Group {
    let group = view.add(Group::new().name("logo_reveal"));
    let row = group.add(Layout::new().row().gap(36.0).y(-40.0));
    let mark = row.add(Circle::new().size(140.0).fill(Accent).name("logo_mark"));
    // The full name is laid out from the start and revealed left to right,
    // so the pair stays centered while it writes.
    let word = row.add(Txt::new().text(name).display().end(0.0).name("logo_name"));
    let tag = group.add(
        Txt::new()
            .text(tagline)
            .h2()
            .fill(Muted)
            .y(110.0)
            .opacity(0.0)
            .name("logo_tagline"),
    );
    mark.pop_in().await;
    word.end()
        .to(1.0, 0.06 * name.chars().count() as f64)
        .ease(presets::LINEAR)
        .await;
    tag.fade_in().await;
    group
}
