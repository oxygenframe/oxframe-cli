use oxframe::prelude::*;

/// A vertical 9:16 short for social: a title, three quick facts that pop in one
/// at a time, and a sign-off. Big, readable text, timed for a phone. Edit the
/// words, run `oxframe check` for the storyboard, then `oxframe render`.
#[oxframe::scene]
async fn short(view: View) {
    let title = view.add(TitleCard::new("{{name}}").subtitle("Three quick facts"));
    title.enter().await;
    wait_for(1.0).await;
    title.exit().await;

    let facts = view.add(BulletList::new(["First fact", "Second fact", "Third fact"]));
    facts.enter().await;
    for _ in 0..3 {
        facts.next().await;
        wait_for(1.0).await;
    }
    facts.exit().await;

    let end = view.add(EndCard::new("Follow for more"));
    end.enter().await;
    wait_for(1.2).await;
}

fn main() {
    // 1080 x 1920 is a 9:16 vertical short; swap to (1080, 1080) for a square.
    Project::new("{{name}}").size(1080, 1920).scene(short()).main();
}
