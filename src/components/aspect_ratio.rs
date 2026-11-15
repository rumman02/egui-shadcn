//! shadcn `aspect-ratio.tsx` — a container that constrains its child to a
//! fixed width/height ratio and paints nothing itself.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/aspect-ratio.tsx`, an 11-line
//! pass-through wrapper around the Radix `AspectRatio` root with zero class
//! strings: no variants, no sizes, no states, no colours, no animation, and
//! no keyboard behaviour. The only behaviour is the geometric constraint
//! `height = width / ratio`, driven by the width the parent layout supplies.
//!
//! The original's `ratio` prop defaults to `1` (square) via the Radix
//! primitive default; [`AspectRatio::default`] matches that. The shadcn docs
//! mark `ratio` required as API guidance, but omitting it renders a square
//! box at runtime.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! AspectRatio::new(16.0 / 9.0).show(ui, |ui| {
//!     // draw an image or anything else; the ui fills the ratio box exactly
//! });
//! ```
//!
//! Rust-side notes (not original behaviour): [`AspectRatio::max_width`] is a
//! Rust-only convenience with no counterpart in the original; degenerate
//! ratios are clamped as a safety decision (see `height_for_width`); child
//! overflow handling is an implementation choice pending spec OPEN item O2
//! (see [`AspectRatio::show`]).

use egui::{InnerResponse, Sense, Ui, UiBuilder, Vec2};

/// Height for `width` at the given ratio (pure, testable).
///
/// Rust-side safety decision (frozen S0 spec §9 D3; OPEN item O4): the
/// original's behaviour for degenerate ratios is unestablished, so any
/// ratio that is non-finite, zero, negative, or below `0.01` is treated
/// as `0.01`, and any non-finite or negative width is treated as `0`.
/// The resulting height is therefore always finite and non-negative, and
/// the allocated rect is never `NaN`, negative, or infinite. This
/// clamping is NOT spec compliance — no clamping rule exists in the
/// original — and ratios below `0.01` (taller than 100:1) render at the
/// `0.01` floor.
pub(crate) fn height_for_width(width: f32, ratio: f32) -> f32 {
    let width = if width.is_finite() {
        width.max(0.0)
    } else {
        0.0
    };
    let ratio = if ratio.is_finite() && ratio > 0.01 {
        ratio
    } else {
        0.01
    };
    width / ratio
}

/// A shadcn aspect-ratio container: a pure layout box that constrains its
/// child to `height = width / ratio` and paints nothing itself.
///
/// No variants, sizes, states, colours, tokens, animation, focus, or
/// keyboard behaviour exist for this component (frozen S0 spec §5): the
/// wrapper declares no classes and references no tokens, so both recipes
/// (base and nova) and both themes render identically.
///
/// The child is supplied as a closure receiving a `Ui` filling exactly the
/// ratio box — the Rust analogue of the original's arbitrary JSX children.
/// There is no `asChild`/Slot analogue: egui has no element-substitution
/// mechanism (spec §9 D2). The original's `data-slot="aspect-ratio"`
/// attribute is a styling hook with no snapshot consumer and no Rust
/// equivalent.
#[derive(Clone, Copy, Debug)]
pub struct AspectRatio {
    ratio: f32,
    max_width: Option<f32>,
}

impl Default for AspectRatio {
    // Square (ratio `1`), matching the original's effective default via
    // the Radix primitive default (spec §3.2; coordinator decision D1-a).
    fn default() -> Self {
        Self::new(1.0)
    }
}

impl AspectRatio {
    /// Creates a container with the given width/height ratio.
    ///
    /// Any positive finite number is a valid ratio (e.g. `16.0 / 9.0`,
    /// `1.0`, `9.0 / 16.0`, `4.0 / 3.0`). Degenerate ratios (zero,
    /// negative, `NaN`, infinite) are clamped by a Rust-side safety
    /// decision, not spec compliance — see `height_for_width`.
    pub fn new(ratio: f32) -> Self {
        Self {
            ratio,
            max_width: None,
        }
    }

    /// Caps the container width instead of spanning the full available width.
    ///
    /// Rust-only convenience (spec §9 D4): the original has no
    /// width-limiting prop — callers limit width via `className`/`style`
    /// on the component or its parent. A non-finite cap is ignored; a
    /// negative cap is treated as zero.
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = Some(max_width);
        self
    }

    /// The size this container will occupy given the current ui.
    ///
    /// The width is the available width, capped by `max_width` if set;
    /// the height is `width / ratio`. The size is always finite with
    /// non-negative dimensions (see `height_for_width`).
    pub fn size_in(&self, ui: &Ui) -> Vec2 {
        let available = ui.available_width();
        let available = if available.is_finite() {
            available.max(0.0)
        } else {
            0.0
        };
        let width = match self.max_width {
            Some(max) if max.is_finite() => available.min(max.max(0.0)),
            _ => available,
        };
        Vec2::new(width, height_for_width(width, self.ratio))
    }

    /// Renders the container and lays out `content` inside it.
    ///
    /// The child `Ui` fills exactly the ratio box (spec §4.1). Overflow
    /// and clipping behaviour is an implementation choice pending spec
    /// OPEN item O2: the child is layout-bounded to the box via
    /// `max_rect`, but its paint is not clipped to the box — egui child
    /// `Ui`s inherit the parent clip rect, so overflowing content may
    /// paint outside the box. This must be revisited if O2 closes with a
    /// contradicting Radix behaviour. The container itself takes no
    /// focus and introduces no tab stop.
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
        // 9:16 portrait at 360 → 640
        assert!((height_for_width(360.0, 9.0 / 16.0) - 640.0).abs() < 1e-3);
    }

    #[test]
    fn default_ratio_is_square() {
        assert_eq!(AspectRatio::default().ratio, 1.0);
        assert!((height_for_width(200.0, AspectRatio::default().ratio) - 200.0).abs() < 1e-6);
    }

    #[test]
    fn zero_width_is_zero_height() {
        assert_eq!(height_for_width(0.0, 16.0 / 9.0), 0.0);
        assert_eq!(height_for_width(0.0, 1.0), 0.0);
    }

    #[test]
    fn extreme_ratios_follow_the_formula() {
        // Very wide: 100:1 at 100 → 1
        assert!((height_for_width(100.0, 100.0) - 1.0).abs() < 1e-6);
        // Very tall: 1:100 at 100 → 10000
        assert!((height_for_width(100.0, 0.01) - 10_000.0).abs() < 1e-3);
    }

    #[test]
    fn degenerate_ratios_are_clamped() {
        // Each degenerate input renders at the 0.01 floor (D3), finite.
        for ratio in [0.0, -1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let h = height_for_width(100.0, ratio);
            assert!(h.is_finite(), "ratio {ratio}: height {h}");
            assert!((h - 100.0 / 0.01).abs() < 1e-3, "ratio {ratio}: height {h}");
        }
        // Ratios below the floor (taller than 100:1) render at the floor.
        assert!((height_for_width(100.0, 0.005) - 100.0 / 0.01).abs() < 1e-3);
    }

    #[test]
    fn degenerate_widths_are_sanitized() {
        assert_eq!(height_for_width(-50.0, 2.0), 0.0);
        assert_eq!(height_for_width(f32::NAN, 2.0), 0.0);
        assert_eq!(height_for_width(f32::INFINITY, 2.0), 0.0);
    }

    #[test]
    fn builder_fields() {
        let ar = AspectRatio::new(4.0 / 3.0).max_width(320.0);
        assert!((ar.ratio - 4.0 / 3.0).abs() < 1e-6);
        assert_eq!(ar.max_width, Some(320.0));
        assert_eq!(AspectRatio::new(1.0).max_width, None);
    }
}
