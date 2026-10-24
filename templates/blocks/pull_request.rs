//! A pull request card: number, title, lines changed, then "Merged".
//!
//! `blocks::pull_request(view, 482, "Render on the GPU by default", 120, 38).await`

use oxframe::prelude::*;

const W: f64 = 1200.0;
const H: f64 = 360.0;

/// Shows the card as open, counts the added and removed lines up, then
/// turns the status to merged. Returns the block's group.
pub async fn pull_request(
    view: View,
    number: u32,
    title: &str,
    added: u32,
    removed: u32,
) -> Group {
    let group = view.add(Group::new().opacity(0.0).name("pull_request"));
    let _ = group.add(Rect::new().width(W).height(H).radius(28.0).fill(Surface));
    let left = -W / 2.0 + 60.0;
    let _ = group.add(
        Txt::new()
            .text(format!("#{number}"))
            .h2()
            .fill(Muted)
            .anchor((-1.0, 0.0))
            .x(left)
            .y(-110.0),
    );
    let _ = group.add(
        Txt::new()
            .text(title)
            .h1()
            .max_width(W - 120.0)
            .anchor((-1.0, 0.0))
            .x(left)
            .y(-10.0)
            .name("pr_title"),
    );
    let plus = Signal::new(0.0f64);
    let minus = Signal::new(0.0f64);
    let adds = group.add(
        Txt::new()
            .h2()
            .fill(Success)
            .anchor((-1.0, 0.0))
            .x(left)
            .y(110.0),
    );
    let p = plus.clone();
    adds.text().bind(move || format!("+{:.0}", p.get()));
    let dels = group.add(Txt::new().h2().fill(Danger).y(110.0));
    let m = minus.clone();
    dels.text().bind(move || format!("−{:.0}", m.get()));
    dels.next_to(adds, Side::Right);
    let pill = group.add(Group::new().x(W / 2.0 - 160.0).y(-110.0).name("pr_status"));
    let back = pill.add(Rect::new().width(200.0).height(64.0).radius(32.0).fill(Success));
    let status = pill.add(Txt::new().text("Open").body().weight(700).fill(OnAccent));
    group.fade_in().await;
    all((
        plus.to(added as f64, 1.0),
        minus.to(removed as f64, 1.0),
    ))
    .await;
    wait_for(0.4).await;
    status.text().set("Merged");
    all((back.fill().to(Accent, 0.3), pill.pop_in())).await;
    group
}
