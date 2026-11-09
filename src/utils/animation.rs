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
}
