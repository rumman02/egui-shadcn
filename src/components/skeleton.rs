//! shadcn `skeleton.tsx` — a pulsing placeholder for loading content.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/skeleton.tsx` (frozen spec
//! `~/workspace/batches-a-h/specs/skeleton-spec.md`). The entire base recipe
//! is one class string (`skeleton.tsx:7`):
//!
//! ```text
//! "animate-pulse rounded-md bg-accent"
//! ```
//!
//! **Fill (spec §2).** The base recipe fill is `bg-accent`, NOT `bg-muted`.
//! Every one of the 8 style files overrides `.cn-skeleton` to `bg-muted`
//! (nova `style-nova.css:1223-1225`, luma `:1263-1265`, lyra `:1198-1200`,
//! maia `:1224-1226`, mira `:1225-1227`, rhea `:1263-1265`, sera
//! `:1214-1216`, vega `:1223-1225`), so the base `bg-accent` is the
//! unstyled fallback only and never ships under any active style: the
//! rendered fill is the `muted` token in every style. The distinction
//! matters in dark mode only (light `accent` and `muted` share
//! `oklch(0.97 0 0)`; dark `accent` is `oklch(0.371 0 0)` per
//! `globals.css:156`, dark `muted` is `oklch(0.269 0 0)` per
//! `globals.css:154`).
//!
//! **Radius (spec §3.2).** Per-style, from the `.cn-skeleton` rules
//! (px at the snapshot's `--radius: 0.625rem`):
//!
//! | Style | Radius class | Radius |
//! |---|---|---|
//! | Base (no style) | `rounded-md` | 8px |
//! | Nova, mira, vega | `rounded-md` | 8px |
//! | Maia | `rounded-xl` | 14px |
//! | Luma, rhea | `rounded-2xl` | 18px |
//! | Lyra | `rounded-none` | 0px |
//! | Sera | *(silent — base survives)* | 8px |
//!
//! Sera is silent on radius, so its 8px comes from the surviving base
//! `rounded-md` under LAYER (silence keeps the base value) — implemented
//! as the base default, not as a sera-specific value.
//!
//! **Pulse (spec §6, OPEN-1 interim).** `animate-pulse` is the Tailwind v4
//! framework default — `pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite`,
//! opacity 1 → 0.5 → 1 — because the snapshot contains no animation
//! definition. The values are implemented as stated and flagged OPEN-1
//! (a project-level `--animate-pulse` override cannot be ruled out from
//! snapshot evidence). The cubic-bezier curve is reproduced exactly via
//! the Phase 1 motion system's [`Easing::CubicBezier`], so deviation D2
//! (easing approximation) is void. Phase derives from egui's absolute
//! input clock, so simultaneously visible skeletons pulse in phase; the
//! spec states no synchronisation requirement (§9).
//!
//! **Repaints (audit N9).** A repaint is requested only while the
//! skeleton is visible and animating, so offscreen or static skeletons
//! cost no frames (standing performance rule, plan §9).
//!
//! **Deviations (spec §10).** D1: no ARIA deviation is needed — the
//! original defines no ARIA behaviour for skeleton (§7), so egui's lack
//! of an ARIA tree removes nothing. D2 is VOID: the exact cubic-bezier
//! pulse easing is reproduced (see **Pulse** above), so no easing
//! substitution exists. D3: [`Skeleton::animate`] is a
//! Rust-only extension (the original always pulses); it defaults to
//! pulsing and is labelled as an extension where defined.
//!
//! **OPEN items (spec §11).** OPEN-1 (pulse defaults, interim position
//! implemented), OPEN-2 (`cn()` merge semantics — the caller-radius-wins
//! outcome is implemented, the mechanism is unjudged), OPEN-3 (rendered
//! confirmation), OPEN-4 (docs usages). None is closed by this module.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Skeleton::new().size(120.0, 16.0).show(ui);
//! Skeleton::new().size(250.0, 125.0).show(ui);
//! ```

use crate::theme::{Palette, ShadcnTheme, Style};
use crate::utils::animation::Easing;
use egui::{Color32, Response, Sense, Ui, Vec2, Widget};

/// The `animate-pulse` timing function (Tailwind v4 default, spec §6):
/// `cubic-bezier(0.4, 0, 0.6, 1)`, evaluated exactly by the Phase 1 motion
/// system — the same solver definition browsers use.
const PULSE_EASING: Easing = Easing::CubicBezier(0.4, 0.0, 0.6, 1.0);

/// CSS `animate-pulse` opacity at `elapsed_secs`: 1 → 0.5 → 1 over a 2s
/// period, with the `cubic-bezier(0.4, 0, 0.6, 1)` timing function applied
/// to each keyframe interval (0%→50% and 50%→100%), as CSS applies the
/// animation timing function between each pair of keyframes.
///
/// OPEN-1 interim position: the 2s period, the curve, and the 0.5
/// midpoint are Tailwind v4 framework defaults, not snapshot-verified
/// values (see the module docs).
pub(crate) fn pulse_alpha(elapsed_secs: f64) -> f32 {
    let phase = elapsed_secs.rem_euclid(2.0) as f32;
    if phase < 1.0 {
        1.0 - 0.5 * PULSE_EASING.apply(phase)
    } else {
        0.5 + 0.5 * PULSE_EASING.apply(phase - 1.0)
    }
}

/// The skeleton fill: `bg-muted` in every registry style (spec §2.2).
/// The base recipe's `bg-accent` is the unstyled fallback only; all 8
/// style files override `.cn-skeleton` to `bg-muted`, and the theme
/// system always has an active style, so the `muted` slot is the
/// rendered fill in every configuration.
pub(crate) fn fill_color(p: &Palette) -> Color32 {
    p.muted
}

/// The per-style corner radius (spec §3.2). Styles that declare a radius
/// class in their `.cn-skeleton` rule take that value; the remaining arm
/// is the base `rounded-md` — nova, mira, and vega restate it, and sera
/// is silent, so the base value survives under LAYER.
pub(crate) fn style_radius(style: Style, radius: &crate::theme::RadiusTokens) -> u8 {
    match style {
        Style::Luma | Style::Rhea => radius.x2l_u8(), // rounded-2xl
        Style::Maia => radius.xl_u8(),                // rounded-xl
        Style::Lyra => 0,                             // rounded-none
        // rounded-md: restated by nova/mira/vega, base survives for sera.
        Style::Nova | Style::Mira | Style::Vega | Style::Sera => radius.md_u8(),
    }
}

/// A shadcn skeleton. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Skeleton {
    size: Vec2,
    corner_radius: Option<u8>,
    animate: bool,
    id_salt: Option<String>,
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl Skeleton {
    /// Creates a 120x20 placeholder (roughly a text line).
    ///
    /// The 120x20 size is a Rust-side convenience default, not original
    /// behaviour: the original declares no width or height (spec §3.1)
    /// and dimensions come entirely from the caller. Use [`Self::size`]
    /// for the caller-supplied dimensions.
    pub fn new() -> Self {
        Self {
            size: Vec2::new(120.0, 20.0),
            corner_radius: None,
            animate: true,
            id_salt: None,
        }
    }

    /// Sets the size in points (`className="h-4 w-[250px]"` etc.).
    ///
    /// This is the caller-sizing mechanism corresponding to the
    /// caller's `className` in the original (spec §8.2). No minimum, no
    /// maximum, no aspect rule (spec §9).
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Overrides the corner radius (the per-style recipe radius by default).
    ///
    /// This is the caller-radius mechanism corresponding to a caller
    /// `className` radius such as `rounded-full` (spec §3.3): a
    /// caller-supplied radius wins over the recipe radius. How `cn()`
    /// resolves the conflict in the original is OPEN-2; only the
    /// caller-wins outcome is specified, and only the outcome is
    /// implemented here.
    pub fn corner_radius(mut self, radius: u8) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Enables/disables the pulse (still painted, at full opacity when off).
    ///
    /// Rust extension (spec §10, D3): the original always pulses. The
    /// default is pulsing; disabling is a host convenience, not
    /// original behaviour.
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }

    /// Sets a stable id salt (Skeleton keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the skeleton.
    ///
    /// The widget takes no focus and handles no input (spec §7):
    /// `Sense::hover()` only. Repaints are requested only while the
    /// skeleton is visible and animating, so offscreen skeletons cost
    /// no frames (standing performance rule, plan §9; audit N9).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());

        if ui.is_rect_visible(rect) {
            if self.animate {
                ui.ctx().request_repaint();
            }
            let mut fill = fill_color(&theme.palette);
            if self.animate {
                let t = ui.input(|i| i.time);
                fill = crate::theme::color_alpha(fill, pulse_alpha(t));
            }
            let radius = self
                .corner_radius
                .unwrap_or_else(|| style_radius(theme.style, &theme.radius));
            ui.painter().rect_filled(rect, radius, fill);
        }

        response
    }
}

impl Widget for Skeleton {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect};

    fn test_ui(width: f32, height: f32) -> egui::Ui {
        let ctx = egui::Context::default();
        egui::Ui::new(
            ctx,
            egui::Id::unique("skeleton_test"),
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, height))),
        )
    }

    #[test]
    fn pulse_matches_the_css_keyframes() {
        // 0% → 1.0, 50% of the cycle → 0.5, 100% → 1.0.
        assert!((pulse_alpha(0.0) - 1.0).abs() < 1e-4);
        assert!((pulse_alpha(1.0) - 0.5).abs() < 1e-4);
        assert!((pulse_alpha(2.0) - 1.0).abs() < 1e-4);
        // Midpoint of each half: the symmetric curve gives 0.75.
        assert!((pulse_alpha(0.5) - 0.75).abs() < 1e-4);
        assert!((pulse_alpha(1.5) - 0.75).abs() < 1e-4);
        // Bounded between the two keyframe opacities.
        for i in 0..=40 {
            let a = pulse_alpha(f64::from(i) * 0.05);
            assert!((0.5..=1.0).contains(&a), "pulse_alpha at {i}: {a}");
        }
    }

    #[test]
    fn pulse_uses_the_exact_cubic_bezier_curve() {
        // Reference values from an independent bisection solver for
        // cubic-bezier(0.4, 0, 0.6, 1) — NOT from the motion system, so
        // this test discriminates the exact curve from the old raised
        // cosine approximation (which gives 0.9268 at t = 0.25s and
        // 0.5732 at t = 0.75s).
        // First half, x = 0.25: y = 0.135307 → opacity 0.932346.
        assert!((pulse_alpha(0.25) - 0.932_346).abs() < 1e-4);
        // First half, x = 0.75: y = 0.864693 → opacity 0.567654.
        assert!((pulse_alpha(0.75) - 0.567_654).abs() < 1e-4);
        // Second half mirrors the first: x = 0.25 → 0.567654,
        // x = 0.75 → 0.932346.
        assert!((pulse_alpha(1.25) - 0.567_654).abs() < 1e-4);
        assert!((pulse_alpha(1.75) - 0.932_346).abs() < 1e-4);
        // The curve is symmetric about (0.5, 0.5), so the pulse is
        // symmetric about the 1s midpoint: alpha(1 - d) == alpha(1 + d).
        for i in 0..=10 {
            let d = f64::from(i) * 0.1;
            assert!(
                (pulse_alpha(1.0 - d) - pulse_alpha(1.0 + d)).abs() < 1e-4,
                "symmetry at d = {d}"
            );
        }
        // Monotonic decrease on the first half, increase on the second.
        let mut prev = pulse_alpha(0.0);
        for i in 1..=20 {
            let a = pulse_alpha(f64::from(i) * 0.05);
            assert!(a <= prev + 1e-6, "first half not decreasing at {i}");
            prev = a;
        }
        prev = pulse_alpha(1.0);
        for i in 1..=20 {
            let a = pulse_alpha(1.0 + f64::from(i) * 0.05);
            assert!(a >= prev - 1e-6, "second half not increasing at {i}");
            prev = a;
        }
    }

    #[test]
    fn pulse_period_is_two_seconds() {
        for i in 0..=20 {
            let t = f64::from(i) * 0.1;
            assert!(
                (pulse_alpha(t) - pulse_alpha(t + 2.0)).abs() < 1e-6,
                "period at t = {t}"
            );
        }
    }

    #[test]
    fn fill_is_muted_for_every_style_and_mode() {
        // All 8 style files override `.cn-skeleton` to `bg-muted`.
        let styles = [
            Style::Nova,
            Style::Vega,
            Style::Maia,
            Style::Lyra,
            Style::Mira,
            Style::Luma,
            Style::Sera,
            Style::Rhea,
        ];
        for mode in [Mode::Light, Mode::Dark] {
            let p = Preset::default().build(mode).palette;
            for style in styles {
                assert_eq!(fill_color(&p), p.muted, "style {style:?} mode {mode:?}");
            }
        }
        // The slot choice matters in dark mode: dark muted (0.269) and
        // dark accent (0.371, Position A) differ, so filling with the
        // wrong slot would be visible.
        let dark = Preset::default().build(Mode::Dark).palette;
        assert_ne!(dark.muted, dark.accent);
        assert_eq!(fill_color(&dark), dark.muted);
    }

    #[test]
    fn radius_follows_the_style_overrides() {
        let r = Preset::default().build(Mode::Light).radius;
        // nova/vega/mira/sera: rounded-md (sera via LAYER silence).
        assert_eq!(style_radius(Style::Nova, &r), r.md_u8());
        assert_eq!(style_radius(Style::Vega, &r), r.md_u8());
        assert_eq!(style_radius(Style::Mira, &r), r.md_u8());
        assert_eq!(style_radius(Style::Sera, &r), r.md_u8());
        // luma/rhea: rounded-2xl; maia: rounded-xl; lyra: rounded-none
        assert_eq!(style_radius(Style::Luma, &r), r.x2l_u8());
        assert_eq!(style_radius(Style::Rhea, &r), r.x2l_u8());
        assert_eq!(style_radius(Style::Maia, &r), r.xl_u8());
        assert_eq!(style_radius(Style::Lyra, &r), 0);
    }

    #[test]
    fn radius_absolute_values_match_the_spec() {
        // Spec §3.2 at `--radius: 0.625rem` (10px): md 8, xl 14, 2xl 18.
        let r = Preset::default().build(Mode::Light).radius;
        assert_eq!(r.base, 10.0);
        assert_eq!(style_radius(Style::Nova, &r), 8);
        assert_eq!(style_radius(Style::Sera, &r), 8);
        assert_eq!(style_radius(Style::Maia, &r), 14);
        assert_eq!(style_radius(Style::Luma, &r), 18);
        assert_eq!(style_radius(Style::Rhea, &r), 18);
        assert_eq!(style_radius(Style::Lyra, &r), 0);
    }

    #[test]
    fn show_allocates_the_requested_size() {
        let mut ui = test_ui(300.0, 200.0);
        let r = Skeleton::new().size(250.0, 125.0).show(&mut ui);
        assert_eq!(
            r.rect,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(250.0, 125.0))
        );
        // Zero size allocates a zero rect without panicking.
        let mut ui = test_ui(300.0, 200.0);
        let r = Skeleton::new().size(0.0, 0.0).show(&mut ui);
        assert_eq!(r.rect, Rect::from_min_size(Pos2::ZERO, Vec2::ZERO));
    }

    #[test]
    fn builder_fields() {
        let s = Skeleton::new().size(250.0, 125.0);
        assert_eq!(s.size, Vec2::new(250.0, 125.0));
        assert!(s.animate);
        assert_eq!(s.corner_radius, None);
        let s = Skeleton::new()
            .corner_radius(12)
            .animate(false)
            .id_salt("x");
        assert_eq!(s.corner_radius, Some(12));
        assert!(!s.animate);
        assert_eq!(s.id_salt.as_deref(), Some("x"));
    }
}
