//! A version number and its changes, one line at a time.
//!
//! `blocks::release_notes(view, "v0.23.0", &["Custom shaders run", "Effects inside groups"]).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1200.0;

/// Shows the version, then each change with a mark, one after another.
/// Returns the block's group.
pub async fn release_notes(view: View, version: &str, changes: &[&str]) -> Group {
    let group = view.add(Group::new().name("release_notes"));
    let title = group.add(
        Txt::new()
            .text(version)
            .display()
            .fill(Accent)
            .y(-300.0)
            .opacity(0.0)
            .name("release_version"),
    );
    title.fade_in().await;
    let list = group.add(
        Layout::new()
            .column()
            .gap(36.0)
            .align(LayoutAlign::Start)
            .width(WIDTH)
            .y(60.0),
    );
    let mut rows = Vec::new();
    for &text in changes {
        let row = list.add(
            Layout::new()
                .row()
                .gap(28.0)
                .align(LayoutAlign::Center)
                .justify(Justify::Start)
                .width(WIDTH)
                .opacity(0.0),
        );
        let _ = row.add(Rect::new().size(18.0).radius(5.0).fill(Accent));
        let _ = row.add(Txt::new().text(text).h2().max_width(WIDTH - 60.0));
        rows.push(row);
    }
    for row in rows {
        row.fade_in().await;
        wait_for(0.25).await;
    }
    group
}
