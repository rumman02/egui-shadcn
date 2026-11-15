//! shadcn `separator.tsx` — a 1px visual divider.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/separator.tsx`, following the
//! frozen Batch A S0 spec (`~/workspace/batches-a-h/specs/separator-spec.md`).
//! One constant class string, no variants, no sizes (spec §2.1, quoted
//! verbatim):
//!
//! ```text
//! shrink-0 bg-border data-[orientation=horizontal]:h-px
//! data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full
//! data-[orientation=vertical]:w-px
//! ```
//!
//! The nova `.cn-separator*` rules (`style-nova.css:1068-1078`) restate the
//! base values exactly, so under the settled LAYER ruling both recipes render
//! identically (spec §2.3).
//!
//! The separator has no states, no animation, and no keyboard behaviour: it
//! is a non-focusable 1px filled rect whose long axis fills its container.
//! `decorative` (default `true`) is semantics-only — it has no visual effect
//! (spec §3, deviation D1); egui has no ARIA tree in which to express it.
//!
//! Deviations (spec §10, approval pending with the Expert Reviewer): D1, the
//! semantic separator role is not expressed (see `Separator::decorative`);
//! D2, the `data-orientation` / `data-slot` attributes have no egui
//! equivalent — the orientation value drives geometry directly and
//! `data-slot` is dropped, with no rendered difference.
//!
//! OPEN items (spec §11): how `data-orientation` is emitted (OPEN-1), what
//! `decorative` changes semantically (OPEN-2), whether a non-decorative
//! separator is Tab-reachable (OPEN-3), the nova `.cn-*` attach mechanism
//! (OPEN-4), `cn()` merge semantics (OPEN-5), and the full Radix `Root` prop
//! surface (OPEN-6). All six stay open: no Radix or `cn()` mechanism is
//! asserted as snapshot truth anywhere in this module. Substitute evidence
//! from current public Radix / `cn` sources, retrieved during S6 with
//! explicit version caveats, is recorded in
//! `~/workspace/batches-a-h/reports/separator-s6-report.md` §4 (OPEN-4 was
//! out of scope for that retrieval and is untouched).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Separator::new().show(ui);                 // full-width horizontal line
//! Separator::new().vertical(true).length(24.0).show(ui); // 24pt tall vertical line
//! ```

use crate::theme::ShadcnTheme;
use egui::{Color32, Response, Sense, Ui, Vec2, Widget};

/// The divider direction, mirroring the source `orientation` prop
/// (default `"horizontal"`, `separator.tsx:9`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    /// `h-px w-full`: 1px tall, full container width.
    #[default]
    Horizontal,
    /// `h-full w-px`: full container height, 1px wide.
    Vertical,
}

/// Resolves the separator fill (`bg-border`, spec §5): exactly one token,
/// `--border` (`--color-border` alias, `globals.css:73`; light
/// `oklch(0.922 0 0)`, `globals.css:117`; dark `oklch(1 0 0 / 10%)`,
/// `globals.css:160`). The value is used as resolved by the token layer,
/// alpha included — the dark value is white at 10% alpha and must not be
/// pre-composited against an assumed background.
pub(crate) fn fill_color(palette: &crate::theme::Palette) -> Color32 {
    palette.border
}

/// A shadcn separator. See the [module docs](self) for an example.
///
/// Stateless: no controlled/uncontrolled API, no compound parts, no events.
#[derive(Clone, Debug)]
pub struct Separator {
    orientation: SeparatorOrientation,
    length: Option<f32>,
    color: Option<Color32>,
    decorative: bool,
    id_salt: Option<String>,
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Separator {
    /// Creates a horizontal, decorative separator (the source defaults:
    /// `orientation="horizontal"`, `decorative=true`).
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            length: None,
            color: None,
            decorative: true,
            id_salt: None,
        }
    }

    /// Sets the orientation (`"horizontal"` | `"vertical"`), replacing the
    /// CSS `data-[orientation]` mechanism: the geometry of spec §2.3 is
    /// selected directly from this value. The `data-orientation` attribute
    /// itself has no egui equivalent (deviation D2, spec §10); how the
    /// original emits it is Radix-internal and OPEN (spec OPEN-1).
    pub fn orientation(mut self, orientation: SeparatorOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Makes the separator vertical (`orientation="vertical"`).
    pub fn vertical(mut self, vertical: bool) -> Self {
        if vertical {
            self.orientation = SeparatorOrientation::Vertical;
        } else {
            self.orientation = SeparatorOrientation::Horizontal;
        }
        self
    }

    /// Overrides the length (the long axis). By default a horizontal
    /// separator spans the available width and a vertical one the available
    /// height, like `w-full` / `h-full`.
    ///
    /// Rust-side extension: the original has no length prop. The default
    /// (no override) is the spec behaviour (full extent). Negative or NaN
    /// lengths are unspecified by the spec and are passed through
    /// unhandled; only finite, non-negative lengths are supported.
    pub fn length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }

    /// Overrides the fill colour. By default the fill is the `border` token
    /// (`bg-border`, spec §5).
    ///
    /// Rust-side extension: the original's override path is a caller
    /// `className` merged after the base string. When set, the override wins
    /// over the spec default (spec §9.4); the exact CSS conflict-resolution
    /// order is OPEN-5 and is not asserted here.
    pub fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    /// Marks the separator as decorative (source default `true`).
    ///
    /// Semantics-only: the class string does not depend on `decorative`, so a
    /// decorative and a non-decorative separator render identically in every
    /// state, orientation, style, and theme (spec §3). What `decorative`
    /// changes in the rendered HTML output (ARIA `role`, `aria-orientation`,
    /// `aria-hidden`, or other attributes) is Radix-internal and OPEN (spec
    /// OPEN-2); egui has no ARIA tree, so the value is stored and exposed
    /// via [`Self::is_decorative`] with no visual effect (deviation D1).
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    /// Sets a stable id salt (Separator keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The fill this separator will paint: the [`Self::color`] override when
    /// set, otherwise the `border` token.
    fn fill(&self, palette: &crate::theme::Palette) -> Color32 {
        self.color.unwrap_or_else(|| fill_color(palette))
    }

    /// The size this separator will occupy given the current ui (spec §2.3):
    /// 1px on the cross axis, full available extent on the long axis, unless
    /// a [`Self::length`] override is set.
    ///
    /// With a zero-size container the long axis is zero and the 1px
    /// cross-axis extent is still allocated; there is no minimum size, no
    /// panic, and no fallback size (spec §9.3). A vertical separator claims
    /// the full available height of the parent `Ui`, matching the common
    /// definite-height case; against a parent of indefinite height the
    /// result follows egui layout (spec §9.2, implementation note).
    pub fn size_in(&self, ui: &Ui) -> Vec2 {
        match self.orientation {
            SeparatorOrientation::Horizontal => {
                Vec2::new(self.length.unwrap_or(ui.available_width()), 1.0)
            }
            SeparatorOrientation::Vertical => {
                Vec2::new(1.0, self.length.unwrap_or(ui.available_height()))
            }
        }
    }

    /// Renders the separator: a 1px filled rect, no border, no radius, no
    /// shadow, no padding, no margin (spec §2.3). The 1px thickness is one
    /// logical pixel on egui's pixel grid (spec §9.8, implementation note).
    /// The rect is allocated with `allocate_exact_size`, so it is never
    /// compressed by siblings — the `shrink-0` equivalent (spec §9.1).
    ///
    /// Non-interactive: the widget takes no focus sense and paints no focus
    /// ring (spec §7). Whether a non-decorative separator is Tab-reachable
    /// in the original is Radix behaviour and OPEN (spec OPEN-3); this port
    /// renders it non-focusable and records the item as unresolved rather
    /// than assuming tabbability. Nothing animates (spec §6).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let color = self.fill(&theme.palette);
        let desired = self.size_in(ui);

        let (rect, response) = match self.id_salt.as_deref() {
            Some(salt) => {
                let id = ui.scope_id().with(salt);
                let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
                (rect, ui.interact(rect, id, Sense::hover()))
            }
            None => ui.allocate_exact_size(desired, Sense::hover()),
        };

        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 0, color);
        }

        response
    }

    /// True when [`Self::decorative`] was left at the source default.
    /// Semantics-only; see [`Self::decorative`].
    pub fn is_decorative(&self) -> bool {
        self.decorative
    }
}

impl Widget for Separator {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset, Style};
    use egui::{Pos2, Rect};

    fn test_ui(width: f32, height: f32) -> egui::Ui {
        let ctx = egui::Context::default();
        egui::Ui::new(
            ctx,
            egui::Id::unique("sep_test"),
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, height))),
        )
    }

    #[test]
    fn sizes_match_source() {
        let ui = &mut test_ui(300.0, 200.0);
        let h = Separator::new();
        assert_eq!(h.size_in(ui), Vec2::new(300.0, 1.0)); // h-px w-full
        let v = Separator::new().vertical(true);
        assert_eq!(v.size_in(ui), Vec2::new(1.0, 200.0)); // w-px h-full
        let v_fixed = Separator::new().vertical(true).length(24.0);
        assert_eq!(v_fixed.size_in(ui), Vec2::new(1.0, 24.0));
        let h_fixed = Separator::new().length(120.0);
        assert_eq!(h_fixed.size_in(ui), Vec2::new(120.0, 1.0));
    }

    #[test]
    fn zero_size_container_allocates_cross_axis_only() {
        // Spec §9.3: zero long axis, 1px cross-axis extent, no panic.
        let ui = &mut test_ui(0.0, 0.0);
        assert_eq!(Separator::new().size_in(ui), Vec2::new(0.0, 1.0));
        assert_eq!(
            Separator::new().vertical(true).size_in(ui),
            Vec2::new(1.0, 0.0)
        );
        // Showing into a zero-size container must not panic either.
        let response = Separator::new().show(ui);
        assert_eq!(response.rect.size(), Vec2::new(0.0, 1.0));
    }

    #[test]
    fn fill_is_the_border_token() {
        let p = Preset::default().build(Mode::Light).palette;
        assert_eq!(fill_color(&p), p.border);
    }

    #[test]
    fn fill_is_the_border_token_for_every_style_and_mode() {
        // No per-style code paths: both recipes render identically (spec
        // §2.3), and the only slot is `border` (spec §5).
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
            for mode in [Mode::Light, Mode::Dark] {
                let preset = Preset {
                    style,
                    ..Preset::default()
                };
                let p = preset.build(mode).palette;
                assert_eq!(fill_color(&p), p.border, "style {style:?} mode {mode:?}");
            }
        }
    }

    #[test]
    fn dark_border_alpha_is_preserved() {
        // Spec §5: dark `--border` is white at 10% alpha; the component must
        // preserve the alpha, not pre-composite against a background.
        let p = Preset::default().build(Mode::Dark).palette;
        let border = fill_color(&p);
        assert_eq!(border, p.border);
        assert!(
            border.a() > 0 && border.a() < 255,
            "dark border must stay translucent, got alpha {}",
            border.a()
        );
    }

    #[test]
    fn orientation_builder() {
        let s = Separator::new().vertical(false);
        assert_eq!(s.orientation, SeparatorOrientation::Horizontal);
        let s = Separator::new().vertical(true);
        assert_eq!(s.orientation, SeparatorOrientation::Vertical);
        let s = Separator::new().orientation(SeparatorOrientation::Vertical);
        assert_eq!(s.orientation, SeparatorOrientation::Vertical);
        let s = s.orientation(SeparatorOrientation::Horizontal);
        assert_eq!(s.orientation, SeparatorOrientation::Horizontal);
        assert!(s.is_decorative());
        assert!(!Separator::new().decorative(false).is_decorative());
    }

    #[test]
    fn decorative_has_no_visual_effect() {
        // Spec §3: decorative and non-decorative render identically.
        let mut ui_a = test_ui(300.0, 200.0);
        let mut ui_b = test_ui(300.0, 200.0);
        let a = Separator::new().decorative(true).show(&mut ui_a);
        let b = Separator::new().decorative(false).show(&mut ui_b);
        assert_eq!(a.rect, b.rect);
        let mut ui_c = test_ui(300.0, 200.0);
        let mut ui_d = test_ui(300.0, 200.0);
        let c = Separator::new()
            .vertical(true)
            .decorative(true)
            .show(&mut ui_c);
        let d = Separator::new()
            .vertical(true)
            .decorative(false)
            .show(&mut ui_d);
        assert_eq!(c.rect, d.rect);
    }

    #[test]
    fn show_allocates_spec_geometry() {
        let mut ui = test_ui(300.0, 200.0);
        let h = Separator::new().show(&mut ui);
        assert_eq!(
            h.rect,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 1.0))
        );
        let mut ui = test_ui(300.0, 200.0);
        let v = Separator::new().vertical(true).show(&mut ui);
        assert_eq!(
            v.rect,
            Rect::from_min_size(Pos2::ZERO, Vec2::new(1.0, 200.0))
        );
    }

    #[test]
    fn color_override_wins_over_default() {
        // Spec §9.4: a caller-supplied override wins over the spec default.
        let p = Preset::default().build(Mode::Light).palette;
        let custom = Color32::from_rgb(255, 0, 0);
        assert_eq!(Separator::new().fill(&p), p.border);
        assert_eq!(Separator::new().color(custom).fill(&p), custom);
    }
}
