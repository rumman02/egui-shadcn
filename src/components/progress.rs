//! shadcn `progress.tsx` — an animated value bar (Radix `Progress`).
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/progress.tsx`:
//! `relative h-2 w-full overflow-hidden rounded-full bg-primary/20` with a
//! `bg-primary` indicator that slides to `value%` (`transition-all`).
//! The nova style overrides the track to `bg-muted h-1` (LAYER: silence
//! keeps the base value, so `overflow-hidden` / `relative` / `w-full` and
//! the indicator's `h-full w-full flex-1 transition-all` survive under
//! nova — frozen spec `~/workspace/batch-c/specs/progress-spec.md` §0/§2.2).
//!
//! Verified final state (all values S10-verified, S10 report
//! `~/workspace/batch-c/reports/progress-s10-report.md`; S10 supersedes
//! S5/S6 where they differ):
//!
//! - Geometry: track 8px base / 4px nova (`h-2` / `h-1`); `rounded-full`
//!   (4px / 2px effective radius); track fill primary @ 20% base, muted
//!   nova; indicator fill primary, full opacity, both styles. The
//!   indicator is always full track width, translated left by
//!   `-(100 - value)%`, so the visible fill is `value%` from the left
//!   edge. Settled endpoints on the demo track: fill30 = 305px,
//!   fill40 = 407px in every config (S10 §1).
//! - N6 (spec §3.3): the indicator carries NO rounding class — at partial
//!   fill its right end is a SQUARE cut; only at 100%, when the fill meets
//!   the track edge, does the track's `rounded-full` shape the pill end.
//! - `value || 0` coerces falsy values (0, NaN, null) to an empty bar;
//!   there is no indeterminate visual state (spec §4.2).
//! - Animation (PRG-01, fixed S9 `fe6c6db`): `transition-all` is 150ms
//!   cubic-bezier(0.4, 0, 0.2, 1), driven by the shared
//!   `animate_value_eased` helper (`Easing::EaseInOut`) on top of egui's
//!   linear `animate_value_with_time`. S10 distribution verdict (20
//!   transitions per config, 6330 frames): combined mid-band share 20.2%
//!   (43/213) vs the S7 linear baseline 51.7% (152/294) — end-dominant in
//!   every config; 0 overshoot in 793 distinct frames; endpoints exact.
//!   Mid-flight re-targets continue from the displayed value (no snap).
//!   The linear `animate_value` (accordion, collapsible) is untouched and
//!   `animate_value_eased` is called only by `progress.rs:161` (S10 §2).
//! - Value clamping (D3): the port clamps out-of-range values to 0..=100
//!   (`fill_fraction`); the literal `.tsx` translate math does NOT clamp
//!   (v=120 shifts the indicator right 20%; v=-10 hides it). **D3 is
//!   APPROVED as a deliberate defensive deviation (S8 §1)** — Radix does
//!   not clamp upstream (OPEN-2 settled at S6 from the Radix source), so
//!   the clamp stays; it is not a defect.
//! - Progress is non-interactive: no hover, focus, pressed, disabled, or
//!   invalid recipe, and it takes no Tab stop (spec §7). Activation
//!   counting is resolved via the §1 value readout (S10 §3, S6 legs
//!   re-run: R2 clicks exactly-once 4/4, R3 Space/Return, R8 boundaries;
//!   ring counts match S6 exactly in every config).
//! - Focus rings (S10 §5): settled focus+hover ring 72px nova / 88px
//!   vega; focus-only 468 nova / 466–488 vega. OBS-PRG-S7-01 (ring ~6.5×
//!   fainter under hover) is decided NO CHANGE (S8 §3.10): the ring IS
//!   present under hover, just fainter — a property of the button's hover
//!   fill interacting with the ring paint, and the buttons are
//!   button-line owned, not progress-owned.
//! - Budgets (S10 §7): idle CPU ≤0.05%, animating 72–132% (<150%), RSS
//!   growth 0.0%; determinism 0; round-trips byte-identical. Render floor
//!   NOT VERIFIED (2.2–2.4 fps Tab-driven, pipeline-capped — the standing
//!   S7 caveat, never a defect).
//! - No label/value slots: the nova `.cn-progress-label` /
//!   `.cn-progress-value` classes have no `.tsx` consumer and are NOT
//!   built (spec §2.3, OPEN-3). `max` has no visual effect (the transform
//!   hardcodes the 100 scale, spec §4.4).
//! - Token divergence (spec §5.4): light `--primary` is `oklch(0% 0 0)`
//!   in `globals.css` vs `oklch(0.205 0 0)` in `themes.ts` (dark agrees at
//!   0.922). The effective light primary consumed by the Rust `Palette`
//!   is the `themes.ts` value per the standing R1 reading — **CONTESTED**
//!   (card S11 evidence: the R1 phrase appears only as settlement labels
//!   in the parity matrix, while Batch F verified `globals.css` as the
//!   source); goes to S12 / Expert Reviewer, not an S11 edit.
//!
//! Deviations / OPEN inventory (frozen spec §10/§11, current disposition):
//! D1 (`data-slot` dropped) PROPOSED; D2 (no ARIA tree) PROPOSED; D3
//! (clamp) APPROVED defensive deviation (S8 §1); D4 (`className` has no
//! Rust equivalent) NOTE. OPEN-1 (`cn()` merge semantics) stays OPEN;
//! OPEN-2 (upstream clamping) settled — Radix does not clamp; OPEN-3
//! (label/value consumer) none found, slots not built; OPEN-4
//! (`data-state="indeterminate"`) no visual consequence; OPEN-5
//! (`max != 100` usage) none in the snapshot.
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
/// The clamp itself is a port-side decision (deviation D3, spec §4.4 / §10),
/// **APPROVED as a deliberate defensive deviation (S8 §1)**: the literal
/// `.tsx` translate math does NOT clamp (v=120 shifts the indicator right
/// by 20%; v=-10 hides it entirely), and Radix does not clamp upstream
/// (OPEN-2 settled at S6 from the Radix source).
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
        // 150ms (`Duration::Default`) with the spec's
        // `cubic-bezier(0.4, 0, 0.2, 1)` curve (`Easing::EaseInOut`),
        // applied by the shared `animate_value_eased` helper on top of
        // egui's linear `animate_value_with_time` (PRG-01, spec §6).
        let target = fill_fraction(self.value) * track.x;
        let spec = crate::utils::animation::TransitionSpec::new(
            crate::utils::animation::Duration::Default,
            crate::utils::animation::Easing::EaseInOut,
        );
        let animated = crate::utils::animation::animate_value_eased(ui.ctx(), id, target, spec);
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

    /// Width of the widest rect painted in `fill` in one pass, i.e. the
    /// progress indicator's fill width (the track paints a different
    /// colour: `bg-primary/20` or `bg-muted`).
    fn painted_fill_width(out: &egui::FullOutput, fill: egui::Color32) -> f32 {
        let mut best = 0.0f32;
        for clipped in &out.shapes {
            if let egui::Shape::Rect(rect) = &clipped.shape {
                if rect.fill == fill {
                    best = best.max(rect.rect.width());
                }
            }
        }
        best
    }

    #[test]
    fn fill_animates_with_bezier_curve() {
        // PRG-01: the fill must follow cubic-bezier(0.4, 0, 0.2, 1) over
        // 150ms. At the mid-flight sample (t = 0.075, raw animator
        // progress 0.5556) the eased fill is 76.74px vs 71.11px linear
        // on a 200px track, so the band (73.0, 80.0) discriminates the
        // curve from linear with margin on both sides; the settled
        // endpoints are exact. (Pre-fix this test fails: the linear fill
        // measures 71.11px at the mid-flight sample, below the band.)
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let primary = palette(Mode::Light).primary;
        // The root Ui's id is stable across passes, so the animation id
        // (derived from it) is stable too.
        let fill_at = |ctx: &egui::Context, time: f64, value: f32| -> f32 {
            let mut out = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ui| {
                    Progress::new(value).width(200.0).show(ui);
                },
            );
            let width = painted_fill_width(&out, primary);
            out.textures_delta.clear();
            width
        };
        // Mount settled at 30, flip to 40 at t = 0.0. The settled fills
        // are computed the same way the component does — f32(0.3) * 200
        // is 60.000004, not exactly 60.0, so compare with an epsilon.
        let fill30 = fill_fraction(30.0) * 200.0;
        let fill40 = fill_fraction(40.0) * 200.0;
        assert!((fill_at(&ctx, 0.0, 30.0) - fill30).abs() < 1e-4);
        assert!((fill_at(&ctx, 0.0, 40.0) - fill30).abs() < 1e-4);
        // Mid-flight: ahead of the linear midpoint (71.11px), below the
        // settled 80px.
        let mid = fill_at(&ctx, 0.075, 40.0);
        assert!(mid > 73.0 && mid < 80.0, "mid-flight fill width = {mid}");
        // Settled: lands exactly on the 40% fill.
        assert!((fill_at(&ctx, 0.30, 40.0) - fill40).abs() < 1e-4);
    }
}
