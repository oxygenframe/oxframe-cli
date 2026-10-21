//! A node for oxframe: a gauge from 0 to 1.
//!
//! ```ignore
//! use oxframe::prelude::*;
//! use {{crate}}::Gauge;
//!
//! let gauge = view.add(Gauge::new().size(240.0));
//! gauge.value().to(0.8, 1.0).await;
//! ```

use oxframe::prelude::*;

/// A dial that fills from 0 to 1. `view.add(Gauge::new())` returns a
/// `GaugeHandle`; `.value()` and `.size()` are signals.
#[derive(Node)]
pub struct Gauge {
    /// 0 is empty, 1 is full.
    #[signal]
    pub value: f64,
    /// Diameter in pixels.
    #[signal]
    pub size: f64,
}

impl Draw for Gauge {
    // A function of the values and ctx.time() only: the frame is painted
    // again on seek, and `oxframe check` flags a paint that changes.
    fn draw(&self, ctx: &mut DrawCtx) {
        let r = (self.size / 2.0 - 12.0).max(0.0);
        ctx.arc((0.0, 0.0), r, 135.0, 270.0).stroke(Surface, 18.0);
        ctx.arc((0.0, 0.0), r, 135.0, 270.0 * self.value.clamp(0.0, 1.0))
            .stroke(Accent, 18.0);
    }

    // The box that layout, Camera::fit and check use.
    fn size(&self) -> (f64, f64) {
        (self.size, self.size)
    }
}
