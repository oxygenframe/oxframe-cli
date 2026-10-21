//! Reference test: the node paints the same pixels every time, and the
//! filled part grows with `value`.

use oxframe::nodes::{SceneGraph, View};
use oxframe::render::{CpuRenderer, RenderOptions, Renderer};
use oxframe::Color;
use {{crate}}::Gauge;

fn paint(value: f64) -> Vec<u8> {
    let graph = SceneGraph::new(200, 200);
    graph.enter(|| {
        View::current().add(Gauge::new().size(180.0).value(value));
    });
    CpuRenderer::new()
        .render(
            &graph,
            &RenderOptions {
                width: 200,
                height: 200,
                background: Color::BLACK,
            },
        )
        .data
}

/// Pixels that are not the black background.
fn painted(data: &[u8]) -> usize {
    data.chunks_exact(4).filter(|p| p[..3] != [0, 0, 0]).count()
}

#[test]
fn the_same_value_paints_the_same_pixels() {
    assert_eq!(paint(0.5), paint(0.5));
}

#[test]
fn a_fuller_gauge_paints_more_accent() {
    // The accent arc is brighter than the track; count pixels with blue.
    let accent = |d: &[u8]| d.chunks_exact(4).filter(|p| p[2] > 150).count();
    assert!(accent(&paint(0.9)) > accent(&paint(0.2)));
    assert!(painted(&paint(0.0)) > 0, "the track is drawn");
}
