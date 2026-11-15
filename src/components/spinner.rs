//! shadcn `spinner.tsx` — an animated loading indicator.
//!
//! The original (`apps/v4/registry/new-york-v4/ui/spinner.tsx`, 15 lines)
//! renders lucide's `Loader2Icon` with `role="status"`,
//! `aria-label="Loading"` and `className="size-4 animate-spin"` — one
//! 16px icon rotating once per second. There are no variants, sizes,
//! states, or keyboard behaviour, and no `.cn-spinner*` rule exists in
//! any style sheet, so the base recipe is the effective recipe under
//! every style (frozen S0 spec; LAYER ruling).
//!
//! Icon identity: the source imports `Loader2Icon`. Whether the
//! original's pinned `lucide-react` aliases that name to the
//! `loader-circle` geometry is NOT established by the snapshot (spec
//! OPEN-2), so this port paints the Phase 1 embedded Lucide dataset's
//! [`Icon::LoaderCircle`] geometry (`Icon::elements_static`) and the
//! rendered glyph is judged visually at S5 (spec OPEN-1). The `role` /
//! `aria-label` attributes have no egui expression (spec deviation D1).
//!
//! Colour is `currentColor`: the stroke paints in the surrounding text
//! colour ([`egui::Visuals::text_color`]), so a spinner in a button
//! matches the button's text and one in muted prose matches that. The
//! spinner references no colour token of its own. `.color()` is the
//! Rust equivalent of a caller `text-*` class.
//!
//! Animation is the Tailwind v4 `animate-spin` default: 1 second per
//! revolution, linear, infinite, clockwise, rotating about the centre
//! of the icon box. Rotation is driven by egui's input time (the same
//! clock the Phase 1 motion system builds on); the motion system has no
//! continuous-rotation primitive, so there is no transition API to reuse
//! here and no timer is hand-rolled. Repaints are requested only while
//! the spinner is visible, and painting allocates nothing per frame:
//! the embedded polyline is drawn as individual line segments plus a
//! filled disc at each end (egui's open-path stroking has flat caps;
//! the discs supply the round caps Lucide specifies).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Spinner::new().show(ui);
//! Spinner::new().size(24.0).show(ui);
//! ```

use crate::icons::{El, Icon};
use egui::{Color32, Pos2, Response, Sense, Stroke, Ui, Vec2, Widget};

/// The historical sampled-arc geometry: center, radius, and sweep on the
/// 24x24 icon grid (`M21 12a9 9 0 1 1-9-9`). These helpers are shared
/// with `attachment` and `sonner`, which still paint their own loader
/// arcs from them; they are preserved unchanged for those callers.
/// `Spinner` itself no longer paints from them — it paints the embedded
/// Lucide dataset (see [`Icon::LoaderCircle`]), whose arc ends at grid
/// point (14.781, 3.44), not at the 270° point (12, 3) below. The 270°
/// sweep here is the port's historical approximation, not a spec fact.
pub(crate) const ARC_CENTER: [f32; 2] = [12.0, 12.0];
pub(crate) const ARC_RADIUS: f32 = 9.0;
/// Sweep in degrees (270° of the 360° circle, matching the historical
/// approximation of the lucide path).
pub(crate) const ARC_SWEEP: f32 = 270.0;

/// Rotation angle at `elapsed_secs`: 360° per second (`animate-spin`).
pub(crate) fn angle_at(elapsed_secs: f64) -> f32 {
    (elapsed_secs * 360.0) as f32 % 360.0
}

/// Points along the spinner arc, rotated by `angle_deg` (pure, testable).
pub(crate) fn arc_points(angle_deg: f32, samples: usize) -> Vec<[f32; 2]> {
    (0..=samples)
        .map(|i| {
            let t = ARC_SWEEP * (i as f32 / samples as f32);
            let a = (angle_deg + t).to_radians();
            let (sin, cos) = a.sin_cos();
            [
                ARC_CENTER[0] + ARC_RADIUS * cos,
                ARC_CENTER[1] + ARC_RADIUS * sin,
            ]
        })
        .collect()
}

/// Stroke width in points for a spinner rendered at `size` points.
///
/// Lucide strokes are 2 grid units on the 24x24 grid and scale with the
/// rendered size (no `vector-effect` in the source): 2s/24, so 1.333px
/// at the 16px default (frozen spec §3.2).
fn stroke_width(size: f32) -> f32 {
    2.0 * size / 24.0
}

/// Rotates a 24x24-grid point around the icon center (12, 12) by
/// `angle_deg`, clockwise on screen (positive angles rotate clockwise
/// because egui's y axis points down, matching CSS `rotate()`).
fn rotate_grid_point(p: [f32; 2], angle_deg: f32) -> [f32; 2] {
    let (sin, cos) = angle_deg.to_radians().sin_cos();
    let (x, y) = (p[0] - 12.0, p[1] - 12.0);
    [12.0 + x * cos - y * sin, 12.0 + x * sin + y * cos]
}

/// The embedded `LoaderCircle` polyline in 24x24 grid coordinates: the
/// single `El::Poly` from the Phase 1 Lucide dataset, starting at grid
/// point (21, 12) and proceeding clockwise. The dataset shape is pinned
/// by `loader_geometry_is_a_single_polyline`; anything else would be a
/// dataset regression and paints nothing.
fn loader_polyline() -> &'static [[f32; 2]] {
    match Icon::LoaderCircle.elements_static() {
        [El::Poly(pts)] => pts,
        _ => {
            debug_assert!(
                false,
                "LoaderCircle dataset changed shape; spinner paints nothing"
            );
            &[]
        }
    }
}

/// A shadcn spinner. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Spinner {
    size: f32,
    color: Option<Color32>,
    id_salt: Option<String>,
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl Spinner {
    /// Creates a 16px spinner (`size-4` in the source).
    pub fn new() -> Self {
        Self {
            size: crate::theme::tokens::spacing::LG,
            color: None,
            id_salt: None,
        }
    }

    /// Sets the square size in points (the Rust equivalent of a caller
    /// `className="size-*"`; the override replaces the 16px default, per
    /// the frozen spec's interim position on `cn()` merge semantics).
    /// The glyph and stroke scale proportionally; the rotation period
    /// stays 1 second.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Overrides the stroke color; defaults to the surrounding text
    /// color (lucide's `currentColor`).
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets a stable id salt (Spinner keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the spinner.
    pub fn show(self, ui: &mut Ui) -> Response {
        // `currentColor`: the inherited text colour of this context. The
        // theme sets `override_text_color` to the palette foreground, so
        // in themed contexts this equals the foreground — and it stays
        // equal to surrounding text under caller style overrides too.
        let color = self.color.unwrap_or_else(|| ui.visuals().text_color());
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());

        if ui.is_rect_visible(rect) {
            // Keep the animation running while on screen — and only
            // while on screen (spec deviation D3).
            ui.ctx().request_repaint();
            let angle = angle_at(ui.input(|i| i.time));
            let scale = self.size / 24.0;
            let stroke = Stroke::new(stroke_width(self.size), color);
            let painter = ui.painter();
            let to_screen = |p: [f32; 2]| {
                let r = rotate_grid_point(p, angle);
                Pos2::new(rect.min.x + r[0] * scale, rect.min.y + r[1] * scale)
            };
            // Drawn segment-by-segment from the static polyline so
            // painting allocates nothing per frame (spec D3).
            let pts = loader_polyline();
            for w in pts.windows(2) {
                painter.line_segment([to_screen(w[0]), to_screen(w[1])], stroke);
            }
            // Round caps: egui's open-path stroking ends flat, so a disc
            // of half the stroke width at each arc end supplies the
            // round caps Lucide specifies.
            if let (Some(first), Some(last)) = (pts.first(), pts.last()) {
                let cap = stroke.width / 2.0;
                painter.circle_filled(to_screen(*first), cap, color);
                painter.circle_filled(to_screen(*last), cap, color);
            }
        }

        response
    }
}

impl Widget for Spinner {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotates_once_per_second() {
        assert!((angle_at(0.0)).abs() < 1e-4);
        assert!((angle_at(0.25) - 90.0).abs() < 1e-3);
        assert!((angle_at(1.0)).abs() < 1e-3); // wraps to 0
        assert!((angle_at(1.5) - 180.0).abs() < 1e-3);
    }

    #[test]
    fn arc_covers_270_degrees() {
        let pts = arc_points(0.0, 24);
        assert_eq!(pts.len(), 25);
        // Starts at angle 0 (right side of the circle, lucide M21 12).
        assert!((pts[0][0] - 21.0).abs() < 1e-4);
        assert!((pts[0][1] - 12.0).abs() < 1e-4);
        // Ends 270° clockwise (y-down): at the top of the circle, leaving the
        // top-right quadrant as the gap (lucide `-9-9`).
        let last = pts[24];
        assert!((last[0] - 12.0).abs() < 1e-4);
        assert!((last[1] - 3.0).abs() < 1e-4);
        let a = 270.0_f32.to_radians();
        let (sin, cos) = a.sin_cos();
        assert!((last[0] - (12.0 + ARC_RADIUS * cos)).abs() < 1e-4);
        assert!((last[1] - (12.0 + ARC_RADIUS * sin)).abs() < 1e-4);
    }

    #[test]
    fn rotation_shifts_all_points() {
        let a = arc_points(90.0, 8);
        // At +90° the first point sits at the bottom of the circle (y-down).
        assert!((a[0][0] - 12.0).abs() < 1e-4);
        assert!((a[0][1] - 21.0).abs() < 1e-4);
    }

    #[test]
    fn builder_defaults_to_16px() {
        let s = Spinner::new();
        assert_eq!(s.size, 16.0);
        assert!(s.color.is_none());
        let s = Spinner::new().size(24.0);
        assert_eq!(s.size, 24.0);
    }

    #[test]
    fn loader_geometry_is_a_single_polyline() {
        // Pins the dataset shape `loader_polyline` relies on: one
        // polyline, starting at (21, 12) and proceeding clockwise, ending
        // at (14.781, 3.44) — the true Lucide `loader-circle` arc, not
        // the historical 270° approximation.
        let elements = Icon::LoaderCircle.elements_static();
        assert_eq!(elements.len(), 1, "LoaderCircle must be a single element");
        let El::Poly(pts) = elements[0] else {
            panic!("LoaderCircle must be a single polyline");
        };
        assert_eq!(pts.len(), 97);
        assert_eq!(pts[0], [21.0, 12.0]);
        // Second point moves down-left: the arc proceeds clockwise.
        assert!(pts[1][0] < 21.0 && pts[1][1] > 12.0);
        let last = pts[pts.len() - 1];
        assert!((last[0] - 14.781).abs() < 1e-3);
        assert!((last[1] - 3.44).abs() < 1e-3);
        // Every point stays on the 24x24 grid.
        for p in pts {
            assert!((0.0..=24.0).contains(&p[0]) && (0.0..=24.0).contains(&p[1]));
        }
    }

    #[test]
    fn grid_rotation_is_clockwise_about_the_center() {
        // 0° is the identity.
        assert_eq!(rotate_grid_point([21.0, 12.0], 0.0), [21.0, 12.0]);
        // +90° moves 3 o'clock to 6 o'clock (y-down = clockwise).
        let p = rotate_grid_point([21.0, 12.0], 90.0);
        assert!((p[0] - 12.0).abs() < 1e-4);
        assert!((p[1] - 21.0).abs() < 1e-4);
        // 360° is the identity again.
        let p = rotate_grid_point([21.0, 12.0], 360.0);
        assert!((p[0] - 21.0).abs() < 1e-4);
        assert!((p[1] - 12.0).abs() < 1e-4);
        // Rotation preserves the radius from the center.
        let p = rotate_grid_point([21.0, 12.0], 45.0);
        let dx = p[0] - 12.0;
        let dy = p[1] - 12.0;
        assert!((dx.hypot(dy) - 9.0).abs() < 1e-4);
        // The center itself never moves.
        assert_eq!(rotate_grid_point([12.0, 12.0], 137.0), [12.0, 12.0]);
    }

    #[test]
    fn stroke_width_scales_with_rendered_size() {
        // Spec §3.2: 2 grid units scaled by size/24.
        assert!((stroke_width(16.0) - 4.0 / 3.0).abs() < 1e-6);
        assert!((stroke_width(24.0) - 2.0).abs() < 1e-6);
        assert!((stroke_width(32.0) - 8.0 / 3.0).abs() < 1e-6);
        assert!((stroke_width(12.0) - 1.0).abs() < 1e-6);
    }
}
