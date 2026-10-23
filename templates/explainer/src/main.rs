use oxframe::prelude::*;

/// A short explainer: a title, three points that reveal one at a time, and an
/// end card. Built from oxframe's components, so the layout, theme and timing
/// are handled for you — edit the words and run `oxframe check` to see the
/// storyboard, then `oxframe render` for the video.
#[oxframe::scene]
async fn explainer(view: View) {
    let title = view.add(TitleCard::new("{{name}}").subtitle("An explainer in three points"));
    title.enter().await;
    title.next().await; // the subtitle
    wait_for(1.0).await;
    title.exit().await;

    let points = view.add(BulletList::new(["What it is", "Why it matters", "How to use it"]));
    points.enter().await;
    for _ in 0..3 {
        points.next().await; // reveal the next point
        wait_for(1.2).await;
    }
    points.exit().await;

    let end = view.add(EndCard::new("Thanks for watching"));
    end.enter().await;
    wait_for(1.5).await;
}

fn main() {
    Project::new("{{name}}").scene(explainer()).main();
}
