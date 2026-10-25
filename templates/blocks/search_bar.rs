//! A search box: the query is typed, then the results drop in under it.
//!
//! `blocks::search_bar(view, "rust motion graphics", &["oxframe.dev", "docs.rs/oxframe"]).await`

use oxframe::prelude::*;

const WIDTH: f64 = 1200.0;
const BAR: f64 = 110.0;
const ROW: f64 = 100.0;

/// Types `query` into the box, then shows each result as a row. Returns
/// the block's group.
pub async fn search_bar(view: View, query: &str, results: &[&str]) -> Group {
    let group = view.add(Group::new().name("search_bar"));
    let top = -(BAR + 30.0 + ROW * results.len() as f64) / 2.0;
    let bar = group.add(Group::new().y(top + BAR / 2.0).opacity(0.0));
    let _ = bar.add(
        Rect::new()
            .width(WIDTH)
            .height(BAR)
            .radius(BAR / 2.0)
            .fill(Surface)
            .name("search_box"),
    );
    let _ = bar.add(
        Circle::new()
            .size(36.0)
            .fill(Surface)
            .stroke(Muted)
            .line_width(6.0)
            .x(-WIDTH / 2.0 + 70.0),
    );
    let typed = bar.add(
        Txt::new()
            .text("")
            .h2()
            .anchor((-1.0, 0.0))
            .x(-WIDTH / 2.0 + 120.0)
            .name("search_query"),
    );
    bar.fade_in().await;
    typed
        .text()
        .to(query, 0.05 * query.chars().count() as f64)
        .await;
    wait_for(0.3).await;
    let rows: Vec<_> = results
        .iter()
        .enumerate()
        .map(|(i, r)| {
            group.add(
                Txt::new()
                    .text(*r)
                    .h2()
                    .fill(Accent)
                    .anchor((-1.0, 0.0))
                    .max_width(WIDTH - 120.0)
                    .x(-WIDTH / 2.0 + 60.0)
                    .y(top + BAR + 30.0 + ROW / 2.0 + i as f64 * ROW)
                    .opacity(0.0)
                    .name(format!("search_result_{i}")),
            )
        })
        .collect();
    stagger(0.15, &rows, |r| r.slide_in(Side::Top)).await;
    group
}
