use oxframe::prelude::*;

/// A square slides from left to right and becomes a circle.
#[oxframe::scene]
async fn intro(view: View) {
    let square = view.add(
        Rect::new()
            .size(200.0)
            .x(-300.0)
            .fill(color!("#e13238")),
    );
    all((
        square.x().to(300.0, 1.0),
        square.radius().to(100.0, 1.0),
        square.fill().to(color!("#68abdf"), 1.0),
    ))
    .await;

    // Drag this marker in the editor timeline to retime what follows.
    wait_until("hello").await;
    let title = view.add(Txt::new().text("").font_size(96.0).y(220.0));
    title.text().to("Hello, {{name}}", 1.0).ease(presets::LINEAR).await;
    wait_for(1.0).await;
}

fn main() {
    Project::new("{{name}}").scene(intro()).main();
}
