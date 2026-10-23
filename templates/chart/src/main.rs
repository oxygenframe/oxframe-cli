use oxframe::prelude::*;

/// A chart explainer: a function drawn on axes, a dot riding along it, then the
/// curve morphs and the range widens. Edit the function and the ranges, run
/// `oxframe check` for the storyboard, then `oxframe render` for the video.
#[oxframe::scene]
async fn chart(view: View) {
    let axes = view.add(Axes::new().x_range(-5.0..5.0).y_range(-2.0..2.0));
    let wave = axes.plot(|x| x.sin()).stroke(Accent);
    wave.draw_on().await;

    let x = Signal::new(-5.0);
    axes.point_on(&wave, x.clone());
    x.to(5.0, 2.0).await;

    wave.to_fn(|x| (2.0 * x).sin(), 1.0).await;
    axes.x_range().to(-10.0..10.0, 1.0).await;
    wait_for(1.0).await;
}

fn main() {
    Project::new("{{name}}").scene(chart()).main();
}
