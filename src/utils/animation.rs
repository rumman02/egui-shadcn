//! Animation helpers on top of egui's time-based animation system.

use egui::{Context, Id};

/// How fast shadcn animations run (CSS `transition`-like durations).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Duration {
    /// 75ms — `transition-colors` micro-transitions.
    Fast,
    /// 150ms — the shadcn default.
    Default,
    /// 200ms — accordion content expand/collapse (`duration-200`).
    Md,
    /// 300ms — dialogs, sheets.
    Slow,
}

impl Duration {
    /// Duration in seconds.
    pub fn secs(self) -> f32 {
        match self {
            Duration::Fast => 0.075,
            Duration::Default => 0.15,
            Duration::Md => 0.2,
            Duration::Slow => 0.3,
        }
    }
}

/// Animates `target` with an explicit duration, storing the animated value
/// under `id` in the context.
pub fn animate_value(ctx: &Context, id: Id, target: f32, duration: Duration) -> f32 {
    ctx.animate_value_with_time(id, target, duration.secs())
}

/// Eases an open/closed progress 0.0..1.0 (dialogs, accordions, sheets).
///
/// `open` is the target; the returned value follows it smoothly. Closing is
/// slightly faster, like the source's `data-closed` animations.
pub fn open_progress(ctx: &Context, id: Id, open: bool) -> f32 {
    let target = if open { 1.0 } else { 0.0 };
    let duration = if open {
        Duration::Default
    } else {
        Duration::Fast
    };
    ctx.animate_value_with_time(id, target, duration.secs())
}

/// The classic shadcn fade+slide+scale "in" animation factors for overlays:
/// callers apply `alpha`, `scale` and `offset_y`.
pub fn overlay_progress(ctx: &Context, id: Id, open: bool) -> OverlayProgress {
    overlay_factors(open_progress(ctx, id, open))
}

/// Nova's overlay timing: the style runs its popup/backdrop transitions at
/// `duration-100` (100ms) in both directions (`style-nova.css`), so the
/// fade+slide+scale factors follow `open` faster than the base recipe.
pub fn nova_overlay_progress(ctx: &Context, id: Id, open: bool) -> OverlayProgress {
    /// Nova `duration-100` in seconds.
    const NOVA_OVERLAY_SECS: f32 = 0.1;
    let target = if open { 1.0 } else { 0.0 };
    let t = ctx.animate_value_with_time(id, target, NOVA_OVERLAY_SECS);
    overlay_factors(t)
}

/// Fade+slide+scale factors for a 0..1 progress value.
fn overlay_factors(t: f32) -> OverlayProgress {
    OverlayProgress {
        alpha: t,
        scale: 0.95 + 0.05 * t,
        offset_y: (1.0 - t) * 4.0,
    }
}

/// Resolved overlay animation factors.
#[derive(Clone, Copy, Debug)]
pub struct OverlayProgress {
    /// 0..1, multiply alpha by this.
    pub alpha: f32,
    /// 0.95..1.0, multiply sizes by this.
    pub scale: f32,
    /// Slide-down offset in points when appearing.
    pub offset_y: f32,
}

// ---------------------------------------------------------------------------
// Color / opacity transitions
//
// The one transition system for hover and state color changes. The original
// animates these with CSS `transition-colors` / `transition-opacity` (see the
// grounding notes on [`Easing`] and [`TransitionSpec`]); the Rust side drives
// an eased 0..1 progress with egui's own value animator and mixes colors in
// OKLab, so mid-fade colors never dip through muddy sRGB midpoints.
// ---------------------------------------------------------------------------

/// Easing curves for transitions, matching the CSS `ease-*` utilities and the
/// custom curves used in the original's style sheets.
///
/// Every variant is a cubic-bezier timing function (CSS `cubic-bezier(x1, y1,
/// x2, y2)`), evaluated by solving for the parameter whose x-coordinate equals
/// the input progress — the same definition browsers use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    /// `linear` — constant speed. Used by the original for spinners and
    /// progress indicators (`ease-linear` in `spinner`/`progress` recipes).
    Linear,
    /// `ease-in`, `cubic-bezier(0.4, 0, 1, 1)` — slow start, fast end.
    EaseIn,
    /// `ease-out`, `cubic-bezier(0, 0, 0.2, 1)` — fast start, slow end.
    EaseOut,
    /// `ease-in-out`, `cubic-bezier(0.4, 0, 0.2, 1)`. This is also Tailwind
    /// v4's `--default-transition-timing-function`, so it is the curve the
    /// original's bare `transition-colors` rules (no explicit `ease-*`
    /// class) actually run with — e.g. `style-nova.css` badge (`:133`),
    /// checkbox (`:284`), combobox (`:345`), input (`:678`).
    EaseInOut,
    /// `ease-[cubic-bezier(0.22, 1, 0.36, 1)]` — the pronounced ease-out used
    /// by the original's navigation-menu motion rules in `style-luma.css`
    /// (`:1011`, `:1027`, `:1031`), `style-lyra.css` and `style-maia.css`.
    EaseOutExpo,
    /// An arbitrary CSS cubic-bezier curve: `CubicBezier(x1, y1, x2, y2)`.
    CubicBezier(f32, f32, f32, f32),
}

impl Easing {
    /// The default curve: [`Easing::EaseInOut`], Tailwind v4's default
    /// transition timing function.
    pub const DEFAULT: Easing = Easing::EaseInOut;

    /// Control points `(x1, y1, x2, y2)` of the cubic-bezier curve.
    pub fn control_points(self) -> (f32, f32, f32, f32) {
        match self {
            Easing::Linear => (0.0, 0.0, 1.0, 1.0),
            Easing::EaseIn => (0.4, 0.0, 1.0, 1.0),
            Easing::EaseOut => (0.0, 0.0, 0.2, 1.0),
            Easing::EaseInOut => (0.4, 0.0, 0.2, 1.0),
            Easing::EaseOutExpo => (0.22, 1.0, 0.36, 1.0),
            Easing::CubicBezier(x1, y1, x2, y2) => (x1, y1, x2, y2),
        }
    }

    /// Maps linear progress `t` (0..1) through the easing curve. Inputs are
    /// clamped; `apply(0.0) == 0.0` and `apply(1.0) == 1.0` for every curve.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        if t == 0.0 || t == 1.0 || self == Easing::Linear {
            return t;
        }
        let (x1, y1, x2, y2) = self.control_points();
        cubic_bezier_ease(x1, y1, x2, y2, t)
    }
}

impl Default for Easing {
    fn default() -> Self {
        Easing::DEFAULT
    }
}

/// Evaluates a CSS cubic-bezier timing function at input progress `x`:
/// solves `x(t) = x` for `t` (Newton-Raphson with a bisection fallback),
/// then returns `y(t)`.
fn cubic_bezier_ease(x1: f32, y1: f32, x2: f32, y2: f32, x: f32) -> f32 {
    // Cubic bezier basis with P0 = (0, 0), P3 = (1, 1).
    let sample_x = |t: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t
    };
    let sample_y = |t: f32| {
        let u = 1.0 - t;
        3.0 * u * u * t * y1 + 3.0 * u * t * t * y2 + t * t * t
    };
    let sample_dx = |t: f32| {
        let u = 1.0 - t;
        3.0 * u * u * x1 + 6.0 * u * t * (x2 - x1) + 3.0 * t * t * (1.0 - x2)
    };

    // Newton-Raphson from t = x; x(t) is monotonic for timing functions
    // (x1, x2 in [0, 1]), so this converges in a few iterations.
    let mut t = x;
    for _ in 0..8 {
        let err = sample_x(t) - x;
        if err.abs() < 1e-6 {
            return sample_y(t).clamp(0.0, 1.0);
        }
        let d = sample_dx(t);
        if d.abs() < 1e-6 {
            break;
        }
        t = (t - err / d).clamp(0.0, 1.0);
    }
    // Fallback: bisection, guaranteed to converge for monotonic x(t).
    let (mut lo, mut hi) = (0.0f32, 1.0f32);
    t = x;
    for _ in 0..24 {
        let v = sample_x(t);
        if (v - x).abs() < 1e-6 {
            break;
        }
        if v < x {
            lo = t;
        } else {
            hi = t;
        }
        t = (lo + hi) * 0.5;
    }
    sample_y(t).clamp(0.0, 1.0)
}

/// Timing for one transition: how long it runs and which curve it follows.
///
/// The default is the original's bare-`transition-colors` behavior:
/// 150 ms ([`Duration::Default`], Tailwind v4's
/// `--default-transition-duration`) with [`Easing::EaseInOut`] (Tailwind v4's
/// `--default-transition-timing-function`). Explicit `duration-*` classes in
/// the original map onto [`Duration`] (`duration-200` is the most common
/// explicit value, e.g. accordion content) or an arbitrary second count via
/// [`TransitionSpec::secs`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransitionSpec {
    /// Transition duration in seconds. `<= 0.0` snaps instantly.
    pub duration_secs: f32,
    /// Easing curve applied to the linear progress.
    pub easing: Easing,
}

impl TransitionSpec {
    /// A spec from a [`Duration`] preset and an [`Easing`] curve.
    pub fn new(duration: Duration, easing: Easing) -> Self {
        Self {
            duration_secs: duration.secs(),
            easing,
        }
    }

    /// A spec from an arbitrary duration in seconds and an [`Easing`] curve.
    /// Use this for durations with no [`Duration`] preset (e.g. nova's
    /// `duration-100` overlays, or slow demo timings).
    pub fn secs(duration_secs: f32, easing: Easing) -> Self {
        Self {
            duration_secs,
            easing,
        }
    }

    /// This spec with a different [`Duration`] preset.
    pub fn with_duration(self, duration: Duration) -> Self {
        Self {
            duration_secs: duration.secs(),
            ..self
        }
    }

    /// This spec with a different [`Easing`] curve.
    pub fn with_easing(self, easing: Easing) -> Self {
        Self { easing, ..self }
    }
}

impl Default for TransitionSpec {
    fn default() -> Self {
        Self {
            duration_secs: Duration::Default.secs(),
            easing: Easing::DEFAULT,
        }
    }
}

/// The eased progress of a transition: `t` runs 0.0 (rest state) to 1.0
/// (active state) through the spec's easing curve.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    /// Eased progress, 0.0..1.0. Exactly 0.0 or 1.0 once settled.
    pub t: f32,
    /// Whether the transition is still running. When `false`, no repaint
    /// was requested for this transition and `t` will not change until the
    /// target flips.
    pub animating: bool,
}

/// A transitioned color plus its running state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTransition {
    /// The current color: [`mix_oklab`] of the endpoints at the eased
    /// progress. Exactly the target endpoint once settled.
    pub color: egui::Color32,
    /// Whether the transition is still running (see [`Transition`]).
    pub animating: bool,
}

/// A transitioned opacity plus its running state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlphaTransition {
    /// The current opacity, 0.0..1.0. Exactly the target endpoint once
    /// settled.
    pub alpha: f32,
    /// Whether the transition is still running (see [`Transition`]).
    pub animating: bool,
}

/// Eased 0..1 progress toward `active`, stored under `id` in the context.
///
/// This is the primitive the other transition helpers build on: call it once
/// per widget per frame with a stable `id`, then mix as many properties as
/// needed from the one progress value (fill, border, text color, ...).
///
/// Progress is driven by egui's own value animator
/// (`Context::animate_value_with_time`, linear in time) with the spec's
/// easing applied on top, so re-targeting mid-flight continues smoothly
/// from the current value — never a snap — and completion lands exactly on
/// the target. Repaints are requested only while the transition runs
/// (egui requests one exactly when the animated value differs from the
/// target). The first call for an `id` returns the target state immediately
/// (no fade-in from the wrong state on mount).
///
/// The `spec` should be stable per `id` across frames; changing it
/// mid-flight re-times the running animation.
pub fn transition_progress(
    ctx: &Context,
    id: Id,
    active: bool,
    spec: TransitionSpec,
) -> Transition {
    let target = if active { 1.0 } else { 0.0 };
    let duration = spec.duration_secs.max(0.0);
    let raw = ctx.animate_value_with_time(id, target, duration);
    Transition {
        t: spec.easing.apply(raw),
        animating: raw != target,
    }
}

/// Transitions between two colors as `active` flips, mixing in OKLab (see
/// [`mix_oklab`]).
///
/// ```ignore
/// let fill = transition_color(
///     ui.ctx(),
///     id.with("hover_fill"),
///     p.primary,
///     color_alpha(p.primary, 0.9),
///     response.hovered(),
///     TransitionSpec::default(),
/// );
/// painter.rect_filled(rect, radius, fill.color);
/// ```
pub fn transition_color(
    ctx: &Context,
    id: Id,
    from: egui::Color32,
    to: egui::Color32,
    active: bool,
    spec: TransitionSpec,
) -> ColorTransition {
    let progress = transition_progress(ctx, id, active, spec);
    ColorTransition {
        color: mix_oklab(from, to, progress.t),
        animating: progress.animating,
    }
}

/// Transitions opacity between `from` and `to` (both 0.0..1.0) as `active`
/// flips, for `transition-opacity` behavior.
///
/// Alpha is interpolated linearly in the 0..1 range (opacity is already a
/// perceptual quantity) under the spec's easing curve. Prefer this over
/// mixing a color toward [`egui::Color32::TRANSPARENT`]: a fully
/// transparent endpoint has no defined hue in the OKLab path, so
/// [`mix_oklab`] toward transparent fades through black.
pub fn transition_alpha(
    ctx: &Context,
    id: Id,
    from: f32,
    to: f32,
    active: bool,
    spec: TransitionSpec,
) -> AlphaTransition {
    let progress = transition_progress(ctx, id, active, spec);
    AlphaTransition {
        alpha: (from + (to - from) * progress.t).clamp(0.0, 1.0),
        animating: progress.animating,
    }
}

/// Mixes two [`egui::Color32`] values at `t` (0.0..1.0, clamped) in the
/// OKLab perceptual color space, with alpha interpolated linearly.
///
/// sRGB channel lerps darken visibly at mid-fade (black→white passes
/// through 50% gray, which reads too dark); OKLab keeps lightness
/// perceptually uniform (black→white passes through sRGB 99). The
/// conversion is implemented locally in this module: `src/theme/` owns the
/// OKLCH token type, but `src/utils/` must not depend on theme types for a
/// leaf helper, and OKLab (not hue-preserving OKLCH) is the established
/// choice for UI color mixing.
///
/// Endpoints are exact: `t <= 0.0` returns `a` and `t >= 1.0` returns `b`
/// unchanged, with no round-trip error. Colors are unmultiplied before
/// conversion (egui stores premultiplied sRGBA); a fully transparent
/// endpoint contributes black, so opacity fades should use
/// [`transition_alpha`] instead of mixing toward transparent.
pub fn mix_oklab(a: egui::Color32, b: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        return a;
    }
    if t == 1.0 {
        return b;
    }
    let [ar, ag, ab, aa] = a.to_srgba_unmultiplied();
    let [br, bg, bb, ba] = b.to_srgba_unmultiplied();
    let (al, aa_, ab_) = srgb_to_oklab(ar, ag, ab);
    let (bl, ba_, bb_) = srgb_to_oklab(br, bg, bb);
    let l = al + (bl - al) * t;
    let c1 = aa_ + (ba_ - aa_) * t;
    let c2 = ab_ + (bb_ - ab_) * t;
    let (r, g, bl_) = oklab_to_srgb(l, c1, c2);
    let alpha = (aa as f32 + (ba as f32 - aa as f32) * t).round() as u8;
    egui::Color32::from_rgba_unmultiplied(r, g, bl_, alpha)
}

/// sRGB bytes → OKLab (Björn Ottosson's matrices, as used by CSS
/// `color-mix(in oklab, ...)` implementations).
fn srgb_to_oklab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let lin = |c: u8| {
        let c = c as f32 / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (lin(r), lin(g), lin(b));
    let l = 0.4122215 * r + 0.5363325 * g + 0.05144599 * b;
    let m = 0.2119035 * r + 0.6806995 * g + 0.107397 * b;
    let s = 0.08830246 * r + 0.2817189 * g + 0.6299787 * b;
    let (l, m, s) = (l.cbrt(), m.cbrt(), s.cbrt());
    (
        0.2104543 * l + 0.7936178 * m - 0.004072047 * s,
        1.977998 * l - 2.428592 * m + 0.4505937 * s,
        0.02590404 * l + 0.7827718 * m - 0.8086758 * s,
    )
}

/// OKLab → sRGB bytes (inverse of [`srgb_to_oklab`]).
fn oklab_to_srgb(l: f32, a: f32, b: f32) -> (u8, u8, u8) {
    let l_ = l + 0.3963378 * a + 0.2158038 * b;
    let m_ = l - 0.1055613 * a - 0.06385417 * b;
    let s_ = l - 0.08948418 * a - 1.291486 * b;
    let (l_, m_, s_) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    let r = 4.076742 * l_ - 3.307712 * m_ + 0.2309699 * s_;
    let g = -1.268438 * l_ + 2.609757 * m_ - 0.3413194 * s_;
    let b_ = -0.004196086 * l_ - 0.7034186 * m_ + 1.707615 * s_;
    let enc = |c: f32| {
        let c = c.clamp(0.0, 1.0);
        let v = if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        };
        (v * 255.0).round().clamp(0.0, 255.0) as u8
    };
    (enc(r), enc(g), enc(b_))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_match_source_transitions() {
        assert_eq!(Duration::Fast.secs(), 0.075);
        assert_eq!(Duration::Default.secs(), 0.15);
        assert_eq!(Duration::Md.secs(), 0.2);
        assert_eq!(Duration::Slow.secs(), 0.3);
    }

    #[test]
    fn overlay_progress_bounds() {
        let ctx = egui::Context::default();
        let closed = overlay_progress(&ctx, Id::unique("t1"), false);
        assert!((closed.alpha - 0.0).abs() < 1e-3);
        let open = overlay_progress(&ctx, Id::unique("t2"), true);
        assert!((open.alpha - 1.0).abs() < 1e-3);
    }

    #[test]
    fn nova_overlay_progress_bounds() {
        let ctx = egui::Context::default();
        let closed = nova_overlay_progress(&ctx, Id::unique("t3"), false);
        assert!((closed.alpha - 0.0).abs() < 1e-3);
        assert!((closed.scale - 0.95).abs() < 1e-3);
        let open = nova_overlay_progress(&ctx, Id::unique("t4"), true);
        assert!((open.alpha - 1.0).abs() < 1e-3);
        assert!((open.scale - 1.0).abs() < 1e-3);
        assert!((open.offset_y - 0.0).abs() < 1e-3);
    }

    const ALL_EASINGS: [Easing; 5] = [
        Easing::Linear,
        Easing::EaseIn,
        Easing::EaseOut,
        Easing::EaseInOut,
        Easing::EaseOutExpo,
    ];

    #[test]
    fn easing_endpoints_exact() {
        for easing in ALL_EASINGS {
            assert_eq!(easing.apply(0.0), 0.0, "{easing:?}");
            assert_eq!(easing.apply(1.0), 1.0, "{easing:?}");
        }
        assert_eq!(Easing::CubicBezier(0.3, 0.7, 0.6, 0.2).apply(0.0), 0.0);
        assert_eq!(Easing::CubicBezier(0.3, 0.7, 0.6, 0.2).apply(1.0), 1.0);
    }

    #[test]
    fn easing_linear_is_identity() {
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            assert_eq!(Easing::Linear.apply(t), t);
        }
    }

    #[test]
    fn easing_is_monotonic() {
        for easing in ALL_EASINGS {
            let mut prev = 0.0f32;
            for i in 0..=100 {
                let v = easing.apply(i as f32 / 100.0);
                assert!(
                    v >= prev - 1e-6,
                    "{easing:?} decreased at {i}: {prev} -> {v}"
                );
                assert!((0.0..=1.0).contains(&v), "{easing:?} out of range: {v}");
                prev = v;
            }
        }
    }

    #[test]
    fn easing_matches_reference_values() {
        // Reference values computed with an independent cubic-bezier solver
        // (Newton-Raphson, Python) from the CSS control points.
        let cases = [
            (Easing::EaseIn, 0.25, 0.0986),
            (Easing::EaseIn, 0.5, 0.3248),
            (Easing::EaseIn, 0.75, 0.6301),
            (Easing::EaseOut, 0.25, 0.5776),
            (Easing::EaseOut, 0.5, 0.8392),
            (Easing::EaseOut, 0.75, 0.9642),
            (Easing::EaseInOut, 0.25, 0.2366),
            (Easing::EaseInOut, 0.5, 0.7756),
            (Easing::EaseInOut, 0.75, 0.9594),
            (Easing::EaseOutExpo, 0.25, 0.7649),
            (Easing::EaseOutExpo, 0.5, 0.9614),
            (Easing::EaseOutExpo, 0.75, 0.9969),
        ];
        for (easing, input, expected) in cases {
            let got = easing.apply(input);
            assert!(
                (got - expected).abs() < 1e-3,
                "{easing:?}({input}) = {got}, expected {expected}"
            );
        }
    }

    #[test]
    fn easing_direction_relative_to_diagonal() {
        // Ease-out runs ahead of linear (fast start), ease-in behind it.
        assert!(Easing::EaseOut.apply(0.5) > 0.5);
        assert!(Easing::EaseIn.apply(0.5) < 0.5);
        assert!(Easing::EaseOutExpo.apply(0.25) > Easing::EaseOut.apply(0.25));
    }

    #[test]
    fn easing_custom_bezier_matches_named_curve() {
        let (x1, y1, x2, y2) = Easing::EaseOut.control_points();
        let custom = Easing::CubicBezier(x1, y1, x2, y2);
        for i in 0..=20 {
            let t = i as f32 / 20.0;
            assert!((custom.apply(t) - Easing::EaseOut.apply(t)).abs() < 1e-5);
        }
    }

    #[test]
    fn default_spec_is_tailwind_defaults() {
        let spec = TransitionSpec::default();
        assert_eq!(spec.duration_secs, 0.15);
        assert_eq!(spec.easing, Easing::EaseInOut);
        assert_eq!(
            TransitionSpec::new(Duration::Md, Easing::EaseOut),
            TransitionSpec::secs(0.2, Easing::EaseOut)
        );
    }

    #[test]
    fn mix_oklab_endpoints_exact() {
        let a = egui::Color32::from_rgb(12, 34, 56);
        let b = egui::Color32::from_rgba_unmultiplied(200, 100, 50, 128);
        assert_eq!(mix_oklab(a, b, 0.0), a);
        assert_eq!(mix_oklab(a, b, 1.0), b);
        // Out-of-range t clamps to the endpoints.
        assert_eq!(mix_oklab(a, b, -1.0), a);
        assert_eq!(mix_oklab(a, b, 2.0), b);
    }

    #[test]
    fn mix_oklab_midpoint_is_perceptual() {
        // Black -> white at t = 0.5 in OKLab is sRGB 99 (lightness 0.5),
        // not the sRGB-lerp 128 that reads too dark. Value computed with an
        // independent OKLab implementation (Python).
        let mid = mix_oklab(egui::Color32::BLACK, egui::Color32::WHITE, 0.5);
        let [r, g, b, a] = mid.to_srgba_unmultiplied();
        assert_eq!((r, g, b, a), (99, 99, 99, 255));
        // Red -> blue keeps chroma instead of dipping through dark purple:
        // OKLab (140, 83, 162) vs sRGB lerp (128, 0, 128).
        let mid = mix_oklab(egui::Color32::RED, egui::Color32::BLUE, 0.5);
        let [r, g, b, _] = mid.to_srgba_unmultiplied();
        assert_eq!((r, g, b), (140, 83, 162));
    }

    #[test]
    fn mix_oklab_alpha_lerps_linearly() {
        let a = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 0);
        let b = egui::Color32::from_rgba_unmultiplied(255, 255, 255, 255);
        assert_eq!(mix_oklab(a, b, 0.5).a(), 128);
        assert_eq!(mix_oklab(a, b, 0.25).a(), 64);
    }

    #[test]
    fn mix_oklab_same_color_is_identity() {
        let c = egui::Color32::from_rgb(90, 140, 200);
        assert_eq!(mix_oklab(c, c, 0.37), c);
    }

    /// Runs `f` inside one egui pass at the given input time.
    fn run_at<T>(ctx: &Context, time: f64, f: impl FnOnce() -> T) -> T {
        ctx.begin_pass(egui::RawInput {
            time: Some(time),
            ..Default::default()
        });
        let out = f();
        // Drain the pass output: dropping a `FullOutput` with unapplied
        // texture deltas panics.
        let mut full = ctx.end_pass();
        full.textures_delta.clear();
        out
    }

    #[test]
    fn transition_first_call_is_settled_at_target() {
        let ctx = egui::Context::default();
        let spec = TransitionSpec::default();
        let on = run_at(&ctx, 0.0, || {
            transition_progress(&ctx, Id::unique("first_on"), true, spec)
        });
        assert_eq!(
            on,
            Transition {
                t: 1.0,
                animating: false
            }
        );
        let off = run_at(&ctx, 0.0, || {
            transition_progress(&ctx, Id::unique("first_off"), false, spec)
        });
        assert_eq!(
            off,
            Transition {
                t: 0.0,
                animating: false
            }
        );
    }

    #[test]
    fn transition_retarget_continues_from_current_value() {
        let ctx = egui::Context::default();
        let spec = TransitionSpec::new(Duration::Default, Easing::Linear);
        let id = Id::unique("retarget");
        // Mount settled at 1.0.
        let settled = run_at(&ctx, 0.0, || transition_progress(&ctx, id, true, spec));
        assert_eq!(settled.t, 1.0);
        assert!(!settled.animating);
        // Flip to 0.0: the flip frame still reports the old value, animating.
        let flip = run_at(&ctx, 0.0, || transition_progress(&ctx, id, false, spec));
        assert_eq!(flip.t, 1.0);
        assert!(flip.animating);
        // Halfway through the 150ms fade toward 0.0.
        let mid = run_at(&ctx, 0.075, || transition_progress(&ctx, id, false, spec));
        assert!(mid.t > 0.3 && mid.t < 0.6, "mid t = {}", mid.t);
        assert!(mid.animating);
        // Re-target back to 1.0 mid-flight: continues from the CURRENT
        // value — no snap back to 0.0 or 1.0.
        let retarget = run_at(&ctx, 0.075, || transition_progress(&ctx, id, true, spec));
        assert_eq!(retarget.t, mid.t);
        assert!(retarget.animating);
        // Moving toward 1.0 again, strictly above the retarget point.
        let rising = run_at(&ctx, 0.15, || transition_progress(&ctx, id, true, spec));
        assert!(
            rising.t > mid.t && rising.t < 1.0,
            "rising t = {}",
            rising.t
        );
        assert!(rising.animating);
        // Past the duration: lands exactly on the target and settles.
        let done = run_at(&ctx, 0.30, || transition_progress(&ctx, id, true, spec));
        assert_eq!(
            done,
            Transition {
                t: 1.0,
                animating: false
            }
        );
        // Settled state is stable: repeated calls do not move or animate.
        let again = run_at(&ctx, 0.60, || transition_progress(&ctx, id, true, spec));
        assert_eq!(again, done);
    }

    #[test]
    fn transition_easing_shapes_progress() {
        let ctx = egui::Context::default();
        let linear = TransitionSpec::new(Duration::Default, Easing::Linear);
        let eased = TransitionSpec::new(Duration::Default, Easing::EaseOut);
        let lin_id = Id::unique("ease_lin");
        let out_id = Id::unique("ease_out");
        // Mount both settled at 0.0, then flip both to 1.0 at t = 0.0.
        run_at(&ctx, 0.0, || {
            transition_progress(&ctx, lin_id, false, linear)
        });
        run_at(&ctx, 0.0, || {
            transition_progress(&ctx, out_id, false, eased)
        });
        run_at(&ctx, 0.0, || {
            transition_progress(&ctx, lin_id, true, linear)
        });
        run_at(&ctx, 0.0, || transition_progress(&ctx, out_id, true, eased));
        // Mid-flight, ease-out must be ahead of linear on the same timeline.
        let lin = run_at(&ctx, 0.075, || {
            transition_progress(&ctx, lin_id, true, linear)
        });
        let out = run_at(&ctx, 0.075, || {
            transition_progress(&ctx, out_id, true, eased)
        });
        assert!(lin.t > 0.0 && lin.t < 1.0);
        assert!(
            out.t > lin.t,
            "ease-out {} should lead linear {}",
            out.t,
            lin.t
        );
        assert!(lin.animating && out.animating);
    }

    #[test]
    fn transition_color_lands_exactly_and_tracks_progress() {
        let ctx = egui::Context::default();
        let spec = TransitionSpec::new(Duration::Default, Easing::Linear);
        let id = Id::unique("color_land");
        let from = egui::Color32::BLACK;
        let to = egui::Color32::WHITE;
        let settled = run_at(&ctx, 0.0, || {
            transition_color(&ctx, id, from, to, false, spec)
        });
        assert_eq!(settled.color, from);
        assert!(!settled.animating);
        run_at(&ctx, 0.0, || {
            transition_color(&ctx, id, from, to, true, spec)
        });
        let mid = run_at(&ctx, 0.075, || {
            transition_color(&ctx, id, from, to, true, spec)
        });
        assert!(mid.animating);
        let [r, g, b, _] = mid.color.to_srgba_unmultiplied();
        assert_eq!((r, g), (r, b), "mid-fade gray must be neutral");
        assert!(r > 0 && r < 255, "mid-fade value = {r}");
        let done = run_at(&ctx, 0.30, || {
            transition_color(&ctx, id, from, to, true, spec)
        });
        assert_eq!(done.color, to);
        assert!(!done.animating);
    }

    #[test]
    fn transition_alpha_follows_progress() {
        let ctx = egui::Context::default();
        let spec = TransitionSpec::new(Duration::Default, Easing::Linear);
        let id = Id::unique("alpha_follow");
        let settled = run_at(&ctx, 0.0, || {
            transition_alpha(&ctx, id, 0.0, 1.0, false, spec)
        });
        assert_eq!(settled.alpha, 0.0);
        assert!(!settled.animating);
        run_at(&ctx, 0.0, || {
            transition_alpha(&ctx, id, 0.0, 1.0, true, spec)
        });
        let mid = run_at(&ctx, 0.075, || {
            transition_alpha(&ctx, id, 0.0, 1.0, true, spec)
        });
        assert!(mid.alpha > 0.0 && mid.alpha < 1.0);
        assert!(mid.animating);
        let done = run_at(&ctx, 0.30, || {
            transition_alpha(&ctx, id, 0.0, 1.0, true, spec)
        });
        assert_eq!(done.alpha, 1.0);
        assert!(!done.animating);
    }

    #[test]
    fn transition_zero_duration_snaps() {
        let ctx = egui::Context::default();
        let spec = TransitionSpec::secs(0.0, Easing::Linear);
        let id = Id::unique("zero_dur");
        run_at(&ctx, 0.0, || transition_progress(&ctx, id, false, spec));
        let snapped = run_at(&ctx, 0.0, || transition_progress(&ctx, id, true, spec));
        assert_eq!(
            snapped,
            Transition {
                t: 1.0,
                animating: false
            }
        );
    }
}
