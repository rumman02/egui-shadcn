//! shadcn `progress.tsx` — an animated value bar.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/progress.tsx`:
//! `relative h-2 w-full overflow-hidden rounded-full bg-primary/20` with a
//! `bg-primary` indicator that slides to `value%` (`transition-all`).
//! The nova style overrides the track to `bg-muted h-1`.
//!
//! Frozen spec (`~/workspace/batch-c/specs/progress-spec.md`, Batch C S0):
//! - The indicator carries NO rounding class: at partial fill its right end
//!   is a SQUARE cut; only at 100%, when the fill meets the track edge, does
//!   the track's `rounded-full` shape the pill end (N6, spec §3.3).
//! - `value || 0` coerces falsy values (0, NaN, null) to an empty bar; there
//!   is no indeterminate visual state (spec §4.2).
//! - Out-of-range values are clamped to 0..=100 by the port
//!   (`fill_fraction`); the literal `.tsx` translate math does NOT clamp, so
//!   the clamp is a port-side decision (deviation D3, contingent on OPEN-2 —
//!   whether Radix clamps upstream — reviewer decides).
//! - No label/value slots: the nova `.cn-progress-label` /
//!   `.cn-progress-value` classes have no `.tsx` consumer and are NOT built
//!   (spec §2.3, OPEN-3).
//! - Progress is non-interactive: no hover, focus, pressed, disabled, or
//!   invalid recipe, and it takes no Tab stop (spec §7).
//! - `transition-all` is 150ms with the `cubic-bezier(0.4, 0, 0.2, 1)` curve;
//!   the current animation drives the fill through egui's
//!   `animate_value_with_time`, which is LINEAR in time — the linear-vs-bezier
//!   difference is an S7 verification item, not spec (spec §6).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Progress::new(65.0).show(ui);
//! Progress::new(30.0).width(200.0).show(ui);
//! ```

use crate::theme::{color_alpha, ShadcnTheme, Style};
use egui::{Color32, CornerRadius, Response, Sense, Ui, Vec2, Widget};

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

/// Indicator fill: `bg-primary`, full opacity, in both styles.
pub(crate) fn indicator_color(p: &crate::theme::Palette) -> Color32 {
    p.primary
}

/// The indicator fill fraction for a raw value, clamped to 0.0..=1.0.
/// The source's `value || 0` coerces NaN to 0, so NaN maps to an empty
/// bar (f32::clamp would otherwise pass NaN through).
///
/// The clamp itself is a port-side decision (deviation D3, spec §4.4 / §10):
/// the literal `.tsx` translate math does NOT clamp (v=120 shifts the
/// indicator right by 20%; v=-10 hides it entirely). Whether Radix clamps
/// upstream is OPEN-2; the reviewer decides D3 once it is settled.
pub(crate) fn fill_fraction(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        (value / 100.0).clamp(0.0, 1.0)
    }
}

/// Corner radius for the indicator fill (N6, spec §3.3).
///
/// The indicator carries no rounding class of its own: at partial fill its
/// right end is a SQUARE cut (the track's `overflow-hidden` only clips at
/// the track bounds), while its left end follows the track's `rounded-full`
/// shape. Only when the fill meets the track edge (100%) do the right
/// corners take the track's rounded shape.
pub(crate) fn fill_corner_radius(
    full: CornerRadius,
    fill_width: f32,
    track_width: f32,
) -> CornerRadius {
    if fill_width >= track_width {
        full
    } else {
        CornerRadius {
            nw: full.nw,
            ne: 0,
            sw: full.sw,
            se: 0,
        }
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

        // `transition-all` on the indicator: animate the fill width over
        // 150ms (`Duration::Default`). The spec requires the
        // `cubic-bezier(0.4, 0, 0.2, 1)` curve, but `animate_value_with_time`
        // is LINEAR in time — the difference is an S7 verification item, not
        // a spec change (spec §6).
        let target = fill_fraction(self.value) * track.x;
        let animated = ui.ctx().animate_value_with_time(
            id,
            target,
            crate::utils::animation::Duration::Default.secs(),
        );
        if animated > 0.0 {
            let fill_w = animated.min(track.x);
            let fill_rect = egui::Rect::from_min_size(rect.min, Vec2::new(fill_w, track.y));
            // N6 (spec §3.3): square cut at partial fill; the track's
            // rounded end only when the fill meets the track edge (100%).
            let fill_radius = fill_corner_radius(CornerRadius::same(radius), fill_w, track.x);
            painter.rect_filled(fill_rect, fill_radius, indicator_color(&p));
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
    fn fill_corner_radius_square_cut_at_partial_fill() {
        let full = CornerRadius::same(255);
        // Partial fill: left corners rounded, right corners square (N6).
        let partial = fill_corner_radius(full, 50.0, 100.0);
        assert_eq!(partial.nw, 255);
        assert_eq!(partial.sw, 255);
        assert_eq!(partial.ne, 0);
        assert_eq!(partial.se, 0);
        // Just short of full width is still a square cut.
        let almost = fill_corner_radius(full, 99.9, 100.0);
        assert_eq!(almost.nw, 255);
        assert_eq!(almost.sw, 255);
        assert_eq!(almost.ne, 0);
        assert_eq!(almost.se, 0);
    }

    #[test]
    fn fill_corner_radius_full_at_100_percent() {
        let full = CornerRadius::same(255);
        // At 100% the fill meets the track edge: all four corners rounded.
        let at_full = fill_corner_radius(full, 100.0, 100.0);
        assert_eq!(at_full.nw, 255);
        assert_eq!(at_full.ne, 255);
        assert_eq!(at_full.sw, 255);
        assert_eq!(at_full.se, 255);
    }

    #[test]
    fn indicator_fill_is_primary() {
        for mode in [Mode::Light, Mode::Dark] {
            let p = palette(mode);
            assert_eq!(indicator_color(&p), p.primary);
        }
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
