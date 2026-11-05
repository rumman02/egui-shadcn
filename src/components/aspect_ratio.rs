//! shadcn `aspect-ratio.tsx` — a container that keeps a fixed width/height
//! ratio, wrapping its child (images, videos, maps).
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/aspect-ratio.tsx` (a thin
//! wrapper around the Radix `AspectRatio` root).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! AspectRatio::new(16.0 / 9.0).show(ui, |ui| {
//!     // draw an image or anything else; the ui is sized to the ratio
//! });
//! ```

use egui::{InnerResponse, Sense, Ui, UiBuilder, Vec2};

/// Height for `width` at the given ratio (pure, testable). The ratio is
/// clamped away from zero so no input can panic or produce NaN.
pub(crate) fn height_for_width(width: f32, ratio: f32) -> f32 {
    let ratio = if ratio.is_finite() && ratio > 0.01 {
        ratio
    } else {
        0.01
    };
    width / ratio
}

/// A shadcn aspect-ratio container. See the [module docs](self) for an example.
#[derive(Clone, Copy, Debug)]
pub struct AspectRatio {
    ratio: f32,
    max_width: Option<f32>,
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::new(16.0 / 9.0)
    }
}

impl AspectRatio {
    /// Creates a container with the given width/height ratio (16/9 in the
    /// shadcn docs demo).
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio,
            max_width: None,
        }
    }

    /// Caps the container width instead of spanning the full available width.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// The rect this container will occupy given the current ui.
    pub fn size_in(&self, ui: &Ui) -> Vec2 {
        let width = match self.max_width {
            Some(max) => ui.available_width().min(max),
            None => ui.available_width(),
        };
        Vec2::new(width, height_for_width(width, self.ratio))
    }

    /// Renders the container and lays out `content` inside it.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let (rect, response) = ui.allocate_exact_size(self.size_in(ui), Sense::hover());
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(*ui.layout()));
        let inner = content(&mut child);
        InnerResponse { inner, response }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ratio_math_matches_css() {
        // 16:9 at 640 → 360
        assert!((height_for_width(640.0, 16.0 / 9.0) - 360.0).abs() < 1e-3);
        // 1:1 square
        assert!((height_for_width(200.0, 1.0) - 200.0).abs() < 1e-6);
        // 4:3
        assert!((height_for_width(400.0, 4.0 / 3.0) - 300.0).abs() < 1e-3);
    }

    #[test]
    fn degenerate_ratios_are_clamped() {
        assert!((height_for_width(100.0, 0.0) - 100.0 / 0.01).abs() < 1e-3);
        assert!(height_for_width(100.0, -1.0).is_finite());
        assert!(height_for_width(100.0, f32::NAN).is_finite());
        assert!(height_for_width(100.0, f32::INFINITY).is_finite());
    }

    #[test]
    fn builder_fields() {
        let ar = AspectRatio::new(4.0 / 3.0).max_width(320.0);
        assert!((ar.ratio - 4.0 / 3.0).abs() < 1e-6);
        assert_eq!(ar.max_width, Some(320.0));
    }
}
