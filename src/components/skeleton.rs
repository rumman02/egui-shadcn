//! shadcn `skeleton.tsx` — a pulsing placeholder for loading content.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/skeleton.tsx`:
//! `animate-pulse rounded-md bg-accent`. Every registry style overrides
//! `.cn-skeleton` to `bg-muted` (nova, vega, maia, lyra, mira, luma, sera,
//! rhea all `@apply bg-muted`), so the fill is the muted token everywhere;
//! the corner radius follows the active style (`rounded-md` nova/vega/mira/
//! sera, `rounded-2xl` luma/rhea, `rounded-xl` maia, `rounded-none` lyra).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Skeleton::new().size(120.0, 16.0).show(ui);
//! Skeleton::new().size(250.0, 125.0).show(ui);
//! ```

use crate::theme::{Palette, ShadcnTheme, Style};
use egui::{Color32, Response, Sense, Ui, Vec2, Widget};

/// CSS `animate-pulse`: opacity oscillates 100% → 50% → 100% over 2s with
/// ease-in-out on both halves; approximated with a raised cosine.
pub(crate) fn pulse_alpha(elapsed_secs: f64) -> f32 {
    let phase = (elapsed_secs % 2.0) as f32;
    0.75 + 0.25 * (std::f32::consts::PI * phase).cos()
}

/// The skeleton fill: `bg-muted` in every registry style.
pub(crate) fn fill_color(p: &Palette) -> Color32 {
    p.muted
}

/// The per-style corner radius (`rounded-md` base; style overrides).
pub(crate) fn style_radius(style: Style, radius: &crate::theme::RadiusTokens) -> u8 {
    match style {
        Style::Luma | Style::Rhea => radius.x2l_u8(), // rounded-2xl
        Style::Maia => radius.xl_u8(),                // rounded-xl
        Style::Lyra => 0,                             // rounded-none
        _ => radius.md_u8(),                          // rounded-md (nova, vega, mira, sera)
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
    pub fn new() -> Self {
        Self {
            size: Vec2::new(120.0, 20.0),
            corner_radius: None,
            animate: true,
            id_salt: None,
        }
    }

    /// Sets the size in points (`className="h-4 w-[250px]"` etc.).
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = Vec2::new(width, height);
        self
    }

    /// Overrides the corner radius (`rounded-md` by default).
    pub fn corner_radius(mut self, radius: u8) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    /// Enables/disables the pulse (still painted, at full opacity when off).
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
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());

        if self.animate {
            ui.ctx().request_repaint();
        }

        if ui.is_rect_visible(rect) {
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

    #[test]
    fn pulse_matches_the_css_keyframes() {
        // 0% → 1.0, 50% of the cycle → 0.5, 100% → 1.0.
        assert!((pulse_alpha(0.0) - 1.0).abs() < 1e-4);
        assert!((pulse_alpha(1.0) - 0.5).abs() < 1e-4);
        assert!((pulse_alpha(2.0) - 1.0).abs() < 1e-4);
        // Bounded between the two keyframe opacities.
        let mid = pulse_alpha(0.5);
        assert!(mid > 0.5 && mid < 1.0);
    }

    #[test]
    fn fill_is_muted_for_every_style() {
        // All 8 style files override `.cn-skeleton` to `bg-muted`.
        let p = Preset::default().build(Mode::Light).palette;
        for style in [
            Style::Nova,
            Style::Vega,
            Style::Maia,
            Style::Lyra,
            Style::Mira,
            Style::Luma,
            Style::Sera,
            Style::Rhea,
        ] {
            assert_eq!(fill_color(&p), p.muted, "style {style:?} must use muted");
        }
    }

    #[test]
    fn radius_follows_the_style_overrides() {
        let r = Preset::default().build(Mode::Light).radius;
        // nova/vega/mira/sera: rounded-md
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
