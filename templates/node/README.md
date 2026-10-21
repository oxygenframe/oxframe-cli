# {{name}}

A node for [oxframe](https://oxframe.dev): a gauge that fills from 0 to 1.

```rust
use {{crate}}::Gauge;

let gauge = view.add(Gauge::new().size(240.0));
gauge.value().to(0.8, 1.0).await;
```

- `src/lib.rs` — the node: `#[derive(Node)]` plus `Draw`.
- `examples/demo.rs` — a scene: `cargo run --example demo -- render`.
- `tests/reference.rs` — pixel tests: `cargo test`.

The node API is behind oxframe's `unstable-node-api` feature and may change
between versions. Guide: https://oxframe.dev/docs/custom-nodes
