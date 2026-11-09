//! shadcn `spinner.tsx` — an animated loading indicator.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/spinner.tsx`, which renders
//! lucide's `LoaderCircle` (`size-4 animate-spin`). The arc is drawn with
//! the egui `Painter` and rotates once per second, like CSS `animate-spin`.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Spinner::new().show(ui);
//! Spinner::new().size(24.0).show(ui);
//! ```

use crate::theme::ShadcnTheme;
use egui::{Color32, Pos2, Response, Sense, Shape, Stroke, Ui, Vec2, Widget};

/// The lucide `loader-circle` geometry: center, radius, and sweep on the
/// 24x24 icon grid (`M21 12a9 9 0 1 1-9-9`). The gap in the circle (the
/// top-right quadrant) is what reads as "spinning".
pub(crate) const ARC_CENTER: [f32; 2] = [12.0, 12.0];
pub(crate) const ARC_RADIUS: f32 = 9.0;
/// Sweep in degrees (270° of the 360° circle, matching the lucide path).
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

    /// Sets the square size in points (`className="size-8"` etc.).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Overrides the stroke color; defaults to the theme foreground
    /// (lucide's `currentColor`).
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
        let theme = ShadcnTheme::of_ui(ui);
        let color = self.color.unwrap_or(theme.palette.foreground);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());

        if ui.is_rect_visible(rect) {
            // Keep the animation running while on screen.
            ui.ctx().request_repaint();
            let elapsed = ui.input(|i| i.time);
            let angle = angle_at(elapsed);
            let scale = self.size / 24.0;
            let stroke = Stroke::new(2.0 * scale, color);
            let points: Vec<Pos2> = arc_points(angle, 24)
                .iter()
                .map(|p| Pos2::new(rect.min.x + p[0] * scale, rect.min.y + p[1] * scale))
                .collect();
            ui.painter().add(Shape::line(points, stroke));
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
}
