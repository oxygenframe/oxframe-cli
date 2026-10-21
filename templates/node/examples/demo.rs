//! `cargo run --example demo -- render` writes out/demo.mp4;
//! `-- preview` opens it in the editor.

use oxframe::prelude::*;
use {{crate}}::Gauge;

#[oxframe::scene]
async fn demo(view: View) {
    let gauge = view.add(Gauge::new().size(320.0));
    gauge.value().to(0.8, 1.2).await;
    wait_for(0.5).await;
}

fn main() {
    Project::new("demo").scene(demo()).main();
}
