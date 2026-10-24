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
    let b = blocks::release_notes(
        view,
        "v0.23.0",
        &["Custom shaders run", "Effects inside groups", "Six new blocks"],
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::terminal(
        view,
        "cargo install oxframe-cli",
        &["Compiling oxframe v0.23.0", "Installed oxframe"],
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::code_change(
        view,
        "rust",
        "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}",
        "fn add(a: i64, b: i64) -> i64 {\n    a.saturating_add(b)\n}",
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::bar_chart(
        view,
        "Build time, s",
        &[("rustc", 3.2), ("gcc", 7.5), ("clang", 5.1)],
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::kinetic_title(view, "Motion graphics your agent checks").await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::feature_grid(
        view,
        &[
            ("Fast", "Renders natively, no browser"),
            ("Checked", "Agents read the report"),
            ("Open", "MIT or Apache-2.0"),
        ],
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::checklist(
        view,
        "Before you ship",
        &["Tests pass", "Docs updated", "Video rendered"],
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::comparison(
        view,
        "oxframe",
        "Others",
        &[
            ("Renders without a browser", true, false),
            ("Scenes in Rust", true, false),
            ("Live preview", true, true),
        ],
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::chat_messages(
        view,
        &[
            (false, "Can it render on CI?"),
            (true, "Yes, on the CPU, without a GPU"),
            (false, "Nice, adding it today"),
        ],
    )
    .await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::pull_request(view, 482, "Render on the GPU by default", 120, 38).await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::countdown(view, 3, "Go!").await;
    wait_for(1.5).await;
    b.fade_out().await;
    let b = blocks::call_to_action(view, "Try it today", "oxframe.dev").await;
    wait_for(1.5).await;
    b.fade_out().await;
}

fn main() {
    Project::new("blocks").scene(tour()).main();
}
