use oxframe::prelude::*;

mod blocks;

#[oxframe::scene]
async fn tour(view: View) {
    let b = blocks::logo_reveal(view, "oxframe", "Motion graphics as Rust code").await;
    wait_for(1.0).await;
    b.fade_out().await;
    let b = blocks::stat_counter(view, 1280.0, "stars on GitHub").await;
    wait_for(1.0).await;
    b.fade_out().await;
    let b = blocks::progress_bar(view, "Tests passing", 0.92).await;
    wait_for(1.0).await;
    b.fade_out().await;
    let b = blocks::quote_card(view, "Make it work, then make it fast.", "Kent Beck").await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::before_after(view, "12 s to render", "1.2 s to render").await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::timeline(
        view,
        &[
            ("2019", "First commit"),
            ("2022", "Version 1.0"),
            ("2026", "A million users"),
        ],
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
}

fn main() {
    Project::new("blocks").scene(tour()).main();
}
