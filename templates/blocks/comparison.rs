//! Two products compared feature by feature, a row at a time.
//!
//! `blocks::comparison(view, "oxframe", "Others", &[("Renders without a browser", true, false)]).await`

use oxframe::prelude::*;

/// Shows the header, then adds each feature with "Yes" or "No" for both
/// sides. Returns the block's group.
pub async fn comparison(
    view: View,
    ours: &str,
    theirs: &str,
    features: &[(&str, bool, bool)],
) -> Group {
    let group = view.add(Group::new().name("comparison"));
    let table = group.add(
        Table::new(["", ours, theirs])
            .widths([760.0, 280.0, 280.0])
            .row_height(88.0)
            .font_size(44.0)
            .opacity(0.0)
            .name("comparison_table"),
    );
    table.header()[1].fill().set(Accent);
    table.fade_in().await;
    let answer = |yes: bool| if yes { "Yes" } else { "No" };
    for (i, &(feature, a, b)) in features.iter().enumerate() {
        let added = table.insert_row(i, [feature, answer(a), answer(b)]);
        table.cell(i, 1).fill().set(if a { Success } else { Danger });
        table.cell(i, 2).fill().set(if b { Success } else { Danger });
        added.await;
        wait_for(0.4).await;
    }
    group
}
