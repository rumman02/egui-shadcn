//! shadcn `progress.tsx` — an animated value bar.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/progress.tsx`:
//! `relative h-2 w-full overflow-hidden rounded-full bg-primary/20` with a
//! `bg-primary` indicator that slides to `value%` (`transition-all`).
//! The nova style overrides the track to `bg-muted h-1`.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Progress::new(65.0).show(ui);
//! Progress::new(30.0).width(200.0).show(ui);
//! ```

use crate::theme::{color_alpha, ShadcnTheme, Style};
use egui::{Color32, Response, Sense, Ui, Vec2, Widget};

/// Track height: base `h-2` (8px), nova `h-1` (4px).
pub(crate) fn track_height(style: Style) -> f32 {
    match style {
        Style::Nova => 4.0,
        _ => 8.0,
    }
}

/// Track fill: base `bg-primary/20`, nova `bg-muted`.
pub(crate) fn track_color(p: &crate::theme::Palette, style: Style) -> Color32 {
    match style {
        Style::Nova => p.muted,
        _ => color_alpha(p.primary, 0.2),
    }
}

/// The indicator fill fraction for a raw value, clamped to 0.0..=1.0.
/// The source's `value || 0` coerces NaN to 0, so NaN maps to an empty
/// bar (f32::clamp would otherwise pass NaN through).
pub(crate) fn fill_fraction(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        (value / 100.0).clamp(0.0, 1.0)
    }
}

/// A shadcn progress bar. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Progress {
    value: f32,
    width: Option<f32>,
    id_salt: Option<String>,
}

impl Progress {
    /// Creates a progress bar at the given value (0..=100).
    pub fn new(value: f32) -> Self {
        Self {
            value,
            width: None,
            id_salt: None,
        }
    }

    /// Overrides the full-track width (default: available width, `w-full`).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a stable id salt so the value animation tracks this instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the progress bar.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let height = track_height(theme.style);
        let width = self.width.unwrap_or_else(|| ui.available_width());
        let track = Vec2::new(width.max(0.0), height);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_progress"),
        };

        let (rect, response) = ui.allocate_exact_size(track, Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }

        let radius = theme.radius.full_u8();
        let painter = ui.painter();
        painter.rect_filled(rect, radius, track_color(&p, theme.style));

        // `transition-all` on the indicator: animate the fill width.
        let target = fill_fraction(self.value) * track.x;
        let animated = ui.ctx().animate_value_with_time(
            id,
            target,
            crate::utils::animation::Duration::Default.secs(),
        );
        if animated > 0.0 {
            let fill_rect =
                egui::Rect::from_min_size(rect.min, Vec2::new(animated.min(track.x), track.y));
            painter.rect_filled(fill_rect, radius, p.primary);
        }

        response
    }
}

impl Widget for Progress {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> crate::theme::Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn track_matches_style_recipes() {
        assert_eq!(track_height(Style::Nova), 4.0); // h-1
        assert_eq!(track_height(Style::Vega), 8.0); // h-2
        let p = palette(Mode::Light);
        assert_eq!(track_color(&p, Style::Nova), p.muted);
        assert_eq!(track_color(&p, Style::Vega), color_alpha(p.primary, 0.2));
    }

    #[test]
    fn fill_fraction_clamps() {
        assert_eq!(fill_fraction(0.0), 0.0);
        assert_eq!(fill_fraction(50.0), 0.5);
        assert_eq!(fill_fraction(100.0), 1.0);
        assert_eq!(fill_fraction(120.0), 1.0);
        assert_eq!(fill_fraction(-10.0), 0.0);
        // The source's `value || 0` coerces NaN to an empty bar; f32::clamp
        // would pass NaN through.
        assert_eq!(fill_fraction(f32::NAN), 0.0);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("progress_render_test"),
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(300.0, 200.0),
                    )),
                );
                Progress::new(65.0).show(ui);
                Progress::new(0.0).width(120.0).show(ui);
                Progress::new(100.0).width(120.0).show(ui);
                Progress::new(250.0).id_salt("over").show(ui);
            });
            out.textures_delta.clear();
        }
    }
}
