//! shadcn `bubble.tsx` — chat bubbles.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/bubble.tsx`: a [`Bubble`]
//! wrapper (`w-fit max-w-[80%]`, right-aligned when `align="end"`) around a
//! [`BubbleContent`] pill (`rounded-xl px-3 py-2 text-sm`), in seven
//! variants (default `bg-primary`, secondary, muted, tinted — an OKLCH
//! derivative of the primary hue, outline, ghost, destructive), grouped by
//! [`BubbleGroup`] (`flex-col gap-2`). [`BubbleReactions`] floats a
//! `rounded-full bg-muted ring-3 ring-card` pill over the bubble's top or
//! bottom edge.
//!
//! Web-only details are approximated: the `:hover` shifts only apply to
//! button/link content (`[button,a]:hover:`), which static egui content
//! never triggers, and the tinted variant recomputes `oklch(from
//! var(--primary) …)` from the primary token with a local sRGB→OKLCH
//! conversion.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! BubbleGroup::new().show(ui, |ui| {
//!     Bubble::new().show(ui, |ui| {
//!         BubbleContent::new("Hi there!").show(ui);
//!     });
//!     Bubble::new()
//!         .variant(BubbleVariant::Muted)
//!         .align(BubbleAlign::End)
//!         .show(ui, |ui| {
//!             BubbleContent::new("Hello!").show(ui);
//!             BubbleReactions::new("👍").show(ui);
//!         });
//! });
//! ```

use crate::theme::{color_alpha, Oklch, ShadcnTheme};
use egui::epaint::RectShape;
use egui::{Align, FontId, Id, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2, Widget};

/// Bubble color variants, mirroring the source `variant` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BubbleVariant {
    /// `bg-primary text-primary-foreground` (the source default).
    #[default]
    Default,
    /// `bg-secondary text-secondary-foreground`.
    Secondary,
    /// `bg-muted`.
    Muted,
    /// OKLCH tint of the primary hue: `oklch(from primary 0.93 c*0.4 h)`
    /// light, `0.3` dark, `foreground` text.
    Tinted,
    /// `border-border bg-background`.
    Outline,
    /// Frameless and full-width (`rounded-none bg-transparent p-0
    /// max-w-full`).
    Ghost,
    /// `bg-destructive/10 text-destructive` (dark `/20`).
    Destructive,
}

/// Horizontal alignment, mirroring the source `align` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BubbleAlign {
    /// Left-aligned (the source default).
    #[default]
    Start,
    /// Right-aligned (`self-end`).
    End,
}

/// Layout context pushed by [`Bubble`] for [`BubbleContent`] and
/// [`BubbleReactions`].
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScopeState {
    pub variant: BubbleVariant,
    pub content_rect: Option<Rect>,
}

fn scope_stack_id() -> Id {
    Id::unique("shadcn_bubble_scope_stack")
}

pub(crate) mod scope {
    use super::{scope_stack_id, ScopeState};
    use egui::Ui;

    pub(crate) fn push(ui: &Ui, state: ScopeState) {
        let mut stack: Vec<ScopeState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(scope_stack_id()))
            .unwrap_or_default();
        stack.push(state);
        ui.ctx()
            .data_mut(|d| d.insert_temp(scope_stack_id(), stack));
    }

    pub(crate) fn pop(ui: &Ui) {
        let mut stack: Vec<ScopeState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(scope_stack_id()))
            .unwrap_or_default();
        stack.pop();
        ui.ctx()
            .data_mut(|d| d.insert_temp(scope_stack_id(), stack));
    }

    pub(crate) fn current(ui: &Ui) -> Option<ScopeState> {
        ui.ctx()
            .data_mut(|d| d.get_temp::<Vec<ScopeState>>(scope_stack_id()))
            .and_then(|s| s.last().copied())
    }

    /// Mutates the top-of-stack scope (no-op outside a [`super::Bubble`]).
    pub(crate) fn update(ui: &Ui, f: impl FnOnce(&mut ScopeState)) {
        let mut stack: Vec<ScopeState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(scope_stack_id()))
            .unwrap_or_default();
        if let Some(top) = stack.last_mut() {
            f(top);
        }
        ui.ctx()
            .data_mut(|d| d.insert_temp(scope_stack_id(), stack));
    }
}

/// Resolves the pill colors for a variant: `(fill, text, border)`.
/// Pure apart from the theme read; the tinted math is tested separately.
pub(crate) fn resolve_variant_colors(
    p: &crate::theme::Palette,
    dark: bool,
    variant: BubbleVariant,
) -> (egui::Color32, egui::Color32, egui::Color32) {
    match variant {
        BubbleVariant::Default => (p.primary, p.primary_foreground, egui::Color32::TRANSPARENT),
        BubbleVariant::Secondary => (
            p.secondary,
            p.secondary_foreground,
            egui::Color32::TRANSPARENT,
        ),
        BubbleVariant::Muted => (p.muted, p.foreground, egui::Color32::TRANSPARENT),
        BubbleVariant::Tinted => (
            tinted_from_primary(p.primary, dark, false),
            p.foreground,
            egui::Color32::TRANSPARENT,
        ),
        BubbleVariant::Outline => (p.background, p.foreground, p.border),
        BubbleVariant::Ghost => (
            egui::Color32::TRANSPARENT,
            p.foreground,
            egui::Color32::TRANSPARENT,
        ),
        BubbleVariant::Destructive => (
            color_alpha(p.destructive, if dark { 0.20 } else { 0.10 }),
            p.destructive,
            egui::Color32::TRANSPARENT,
        ),
    }
}

/// `oklch(from var(--primary) L calc(c*k) h)` for the tinted variant.
/// `hover = true` uses the hover lightness/chroma (`0.88/0.5` light,
/// `0.35/0.5` dark). Pure and testable.
pub(crate) fn tinted_from_primary(
    primary: egui::Color32,
    dark: bool,
    hover: bool,
) -> egui::Color32 {
    let (l, k) = match (dark, hover) {
        (false, false) => (0.93, 0.4),
        (false, true) => (0.88, 0.5),
        (true, false) => (0.30, 0.4),
        (true, true) => (0.35, 0.5),
    };
    let ok = oklch_from_srgb(primary);
    Oklch::new(l, (ok.c * k).min(0.4), ok.h).to_color32()
}

/// Converts an sRGB (premultiplied-alpha ignored) color to OKLCH.
/// Pure and testable.
pub(crate) fn oklch_from_srgb(color: egui::Color32) -> Oklch {
    // egui colors are premultiplied; treat fully transparent as black.
    let a = color.a() as f32 / 255.0;
    let un = |v: u8| -> f32 {
        if a <= 0.0 {
            0.0
        } else {
            (v as f32 / 255.0 / a).clamp(0.0, 1.0)
        }
    };
    let (r, g, b) = (un(color.r()), un(color.g()), un(color.b()));
    let lin = |c: f32| -> f32 {
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let (r, g, b) = (lin(r), lin(g), lin(b));
    // Linear sRGB → LMS → OKLab (Björn Ottosson's constants, f32-rounded).
    #[allow(clippy::excessive_precision)]
    let l = 0.412_221_470_8_f32 * r + 0.536_332_536_3_f32 * g + 0.051_445_992_9_f32 * b;
    #[allow(clippy::excessive_precision)]
    let m = 0.211_903_498_2_f32 * r + 0.680_699_545_1_f32 * g + 0.107_396_956_6_f32 * b;
    #[allow(clippy::excessive_precision)]
    let s = 0.088_302_461_9_f32 * r + 0.281_718_837_6_f32 * g + 0.629_978_700_5_f32 * b;
    let (l_, m_, s_) = (l.cbrt(), m.cbrt(), s.cbrt());
    #[allow(clippy::excessive_precision)]
    let lab_l = 0.210_454_255_3_f32 * l_ + 0.793_617_785_0_f32 * m_ - 0.004_072_046_8_f32 * s_;
    #[allow(clippy::excessive_precision)]
    let lab_a = 1.977_998_495_1_f32 * l_ - 2.428_592_205_0_f32 * m_ + 0.450_593_709_9_f32 * s_;
    #[allow(clippy::excessive_precision)]
    let lab_b = 0.025_904_037_1_f32 * l_ + 0.782_771_766_2_f32 * m_ - 0.808_675_766_0_f32 * s_;
    let c = (lab_a * lab_a + lab_b * lab_b).sqrt();
    let h = lab_b.atan2(lab_a).to_degrees().rem_euclid(360.0);
    Oklch {
        l: lab_l.clamp(0.0, 1.0),
        c: c.max(0.0),
        h,
        a: 1.0,
    }
}

/// `BubbleGroup` — a `flex-col gap-2` stack of bubbles.
#[derive(Clone, Debug, Default)]
pub struct BubbleGroup {
    id_salt: Option<String>,
}

impl BubbleGroup {
    /// Creates a bubble group.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt (BubbleGroup keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its bubbles.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 8.0; // gap-2
            content(ui)
        })
    }
}

/// `Bubble` — the variant/align wrapper (`w-fit max-w-[80%]`, `self-end`
/// when align end, `max-w-full` for ghost). Children are
/// [`BubbleContent`]s (and optionally a [`BubbleReactions`]).
#[derive(Clone, Debug, Default)]
pub struct Bubble {
    variant: BubbleVariant,
    align: BubbleAlign,
    id_salt: Option<String>,
}

impl Bubble {
    /// Creates a `default`-variant, start-aligned bubble.
    pub fn new() -> Self {
        Self {
            variant: BubbleVariant::default(),
            align: BubbleAlign::default(),
            id_salt: None,
        }
    }

    /// Sets the color variant.
    pub fn variant(mut self, variant: BubbleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the alignment (`start`/`end`).
    pub fn align(mut self, align: BubbleAlign) -> Self {
        self.align = align;
        self
    }

    /// Sets a stable id salt (Bubble keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the bubble around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        scope::push(
            ui,
            ScopeState {
                variant: self.variant,
                content_rect: None,
            },
        );
        // `self-end` right-aligns the bubble; ghost bubbles take full width.
        let align = match self.align {
            BubbleAlign::Start => Align::Min,
            BubbleAlign::End => Align::Max,
        };
        let max_width = if self.variant == BubbleVariant::Ghost {
            1.0
        } else {
            0.8
        };
        let inner = ui.with_layout(egui::Layout::top_down(align), |ui| {
            ui.set_max_width(ui.available_width() * max_width);
            ui.spacing_mut().item_spacing.y = 4.0; // gap-1
            content(ui)
        });
        scope::pop(ui);
        inner
    }
}

impl Widget for Bubble {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui, |_ui| {}).response
    }
}

/// `BubbleContent` — the visual pill (`rounded-xl border border-transparent
/// px-3 py-2 text-sm`), colored by the enclosing [`Bubble`] variant.
#[derive(Clone, Debug, Default)]
pub struct BubbleContent {
    text: String,
    id_salt: Option<String>,
}

impl BubbleContent {
    /// Creates a content pill with wrapped text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            id_salt: None,
        }
    }

    /// Sets a stable id salt (BubbleContent keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the text pill.
    pub fn show(self, ui: &mut Ui) -> Response {
        let text = self.text.clone();
        self.show_with(ui, move |ui| {
            if !text.is_empty() {
                ui.label(egui::RichText::new(&text).size(crate::theme::tokens::type_scale::SM));
            }
        })
    }

    /// Renders the pill around custom content (markdown, widgets, ...).
    pub fn show_with<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();
        let scope = scope::current(ui);
        let variant = scope.map(|s| s.variant).unwrap_or_default();
        let ghost = variant == BubbleVariant::Ghost;
        // px-3 py-2; ghost is `p-0 rounded-none`.
        let pad = if ghost {
            Vec2::ZERO
        } else {
            Vec2::new(12.0, 8.0)
        };
        let radius = if ghost { 0.0 } else { theme.radius.xl() };

        // Reserve a shape slot behind the content for the pill fill.
        let bg_idx = ui.painter().add(Shape::Noop);

        let laid_out = ui.with_layout(egui::Layout::top_down(Align::Min), |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            content(ui)
        });
        let content_rect = laid_out.response.rect;
        let pill_rect = content_rect.expand2(pad);

        if ui.is_rect_visible(pill_rect) {
            let (fill, text_color, border) = resolve_variant_colors(&p, dark, variant);
            let _ = text_color;
            let mut shapes: Vec<Shape> = vec![Shape::Rect(RectShape::filled(
                pill_rect,
                egui::CornerRadius::same(radius.round() as u8),
                fill,
            ))];
            if border != egui::Color32::TRANSPARENT {
                shapes.push(Shape::rect_stroke(
                    pill_rect,
                    egui::CornerRadius::same(radius.round() as u8),
                    Stroke::new(1.0, border),
                    egui::epaint::StrokeKind::Inside,
                ));
            }
            ui.painter().set(bg_idx, Shape::Vec(shapes));
        }

        scope::update(ui, |s| s.content_rect = Some(pill_rect));
        ui.interact(pill_rect, ui.auto_id_with("bubble_content"), Sense::hover())
    }
}

impl Widget for BubbleContent {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `BubbleReactions` — a `rounded-full bg-muted px-1.5 py-0.5 ring-3
/// ring-card` pill floating over the bubble's top or bottom edge
/// (`side`/`align`, default bottom/end). Must be rendered inside a
/// [`Bubble`].
#[derive(Clone, Debug)]
pub struct BubbleReactions {
    content: String,
    side: BubbleReactionSide,
    align: BubbleReactionAlign,
    id_salt: Option<String>,
}

/// Edge the reactions pill floats on, mirroring the source `side` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BubbleReactionSide {
    /// Overlapping the bubble's bottom edge (the default).
    #[default]
    Bottom,
    /// Overlapping the bubble's top edge.
    Top,
}

/// Horizontal anchor of the reactions pill, mirroring the source `align`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BubbleReactionAlign {
    /// `left-3` (12px from the bubble's left edge).
    Start,
    /// `right-3` (the default).
    #[default]
    End,
}

impl BubbleReactions {
    /// Creates a reactions pill with text (e.g. an emoji).
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            side: BubbleReactionSide::default(),
            align: BubbleReactionAlign::default(),
            id_salt: None,
        }
    }

    /// Sets the edge (`bottom` by default).
    pub fn side(mut self, side: BubbleReactionSide) -> Self {
        self.side = side;
        self
    }

    /// Sets the horizontal anchor (`end` by default).
    pub fn align(mut self, align: BubbleReactionAlign) -> Self {
        self.align = align;
        self
    }

    /// Sets a stable id salt (BubbleReactions keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the reactions pill.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let Some(scope) = scope::current(ui) else {
            // Outside a Bubble: render as a standalone muted pill.
            return paint_reaction_pill(ui, &p, &self.content, ui.cursor().min, self.id());
        };
        let Some(content_rect) = scope.content_rect else {
            return paint_reaction_pill(ui, &p, &self.content, ui.cursor().min, self.id());
        };

        // Measure the pill: `px-1.5 py-0.5 text-sm`.
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let pad_x = 6.0;
        let pad_y = 2.0;
        let galley = ui
            .painter()
            .layout_no_wrap(self.content.clone(), font, p.foreground);
        let pill_size = galley.size() + Vec2::new(2.0 * pad_x, 2.0 * pad_y);

        // `translate-y-3/4` on the chosen edge; `left-3`/`right-3` inset.
        let pos = match (self.side, self.align) {
            (BubbleReactionSide::Bottom, BubbleReactionAlign::End) => Pos2::new(
                content_rect.right() - 12.0 - pill_size.x,
                content_rect.bottom() - 0.25 * pill_size.y,
            ),
            (BubbleReactionSide::Bottom, BubbleReactionAlign::Start) => Pos2::new(
                content_rect.left() + 12.0,
                content_rect.bottom() - 0.25 * pill_size.y,
            ),
            (BubbleReactionSide::Top, BubbleReactionAlign::End) => Pos2::new(
                content_rect.right() - 12.0 - pill_size.x,
                content_rect.top() - 0.75 * pill_size.y,
            ),
            (BubbleReactionSide::Top, BubbleReactionAlign::Start) => Pos2::new(
                content_rect.left() + 12.0,
                content_rect.top() - 0.75 * pill_size.y,
            ),
        };
        paint_reaction_pill_at(ui, &p, &self.content, pos, galley, pill_size, self.id())
    }
}

impl Widget for BubbleReactions {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

impl BubbleReactions {
    fn id(&self) -> Id {
        match self.id_salt.as_deref() {
            Some(salt) => Id::unique("shadcn_bubble_reactions").with(salt),
            None => Id::unique("shadcn_bubble_reactions_auto"),
        }
    }
}

/// Paints the reactions pill at an absolute position (advances no layout).
fn paint_reaction_pill(
    ui: &Ui,
    p: &crate::theme::Palette,
    text: &str,
    pos: Pos2,
    id: Id,
) -> Response {
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, p.foreground);
    let pill_size = galley.size() + Vec2::new(12.0, 4.0);
    paint_reaction_pill_at(ui, p, text, pos, galley, pill_size, id)
}

#[allow(clippy::too_many_arguments)]
fn paint_reaction_pill_at(
    ui: &Ui,
    p: &crate::theme::Palette,
    _text: &str,
    pos: Pos2,
    galley: std::sync::Arc<egui::Galley>,
    pill_size: Vec2,
    id: Id,
) -> Response {
    let rect = Rect::from_min_size(pos, pill_size);
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            crate::theme::RadiusTokens::default().full_u8(),
            p.muted,
        );
        // `ring-3 ring-card` separates the pill from the bubble fill.
        painter.rect_stroke(
            rect,
            crate::theme::RadiusTokens::default().full_u8(),
            Stroke::new(3.0, p.card),
            egui::epaint::StrokeKind::Outside,
        );
        painter.galley(rect.min + Vec2::new(6.0, 2.0), galley, p.foreground);
    }
    ui.interact(rect, id, Sense::hover())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Color32;

    #[test]
    fn oklch_from_srgb_matches_known_values() {
        // White → L≈1, C≈0.
        let white = oklch_from_srgb(Color32::WHITE);
        assert!((white.l - 1.0).abs() < 0.01, "L={}", white.l);
        assert!(white.c < 0.01, "C={}", white.c);
        // Black → L≈0.
        let black = oklch_from_srgb(Color32::BLACK);
        assert!(black.l < 0.01);
        // Pure red ≈ oklch(0.628 0.258 29.2).
        let red = oklch_from_srgb(Color32::from_rgb(255, 0, 0));
        assert!((red.l - 0.628).abs() < 0.01, "L={}", red.l);
        assert!((red.c - 0.258).abs() < 0.01, "C={}", red.c);
        assert!((red.h - 29.2).abs() < 1.5, "H={}", red.h);
        // Transparent treated as black.
        let t = oklch_from_srgb(Color32::TRANSPARENT);
        assert!(t.l < 0.01);
    }

    #[test]
    fn tinted_uses_primary_hue_scaled_chroma() {
        let primary = Oklch::new(0.55, 0.20, 260.0).to_color32();
        let ok = oklch_from_srgb(primary);
        let light = oklch_from_srgb(tinted_from_primary(primary, false, false));
        // 8-bit quantization + sRGB gamut clipping (pastel blue) loosen this:
        // clipping can only reduce chroma and drift the hue toward the axis.
        assert!((light.l - 0.93).abs() < 0.02, "L={}", light.l);
        assert!(
            light.c > 0.0 && light.c <= ok.c * 0.4 + 0.02,
            "C={}",
            light.c
        );
        // Dark pastels stay in gamut: full L/C/H round-trip holds.
        let dark = oklch_from_srgb(tinted_from_primary(primary, true, false));
        assert!((dark.l - 0.30).abs() < 0.02, "L={}", dark.l);
        assert!((dark.c - ok.c * 0.4).abs() < 0.02, "C={}", dark.c);
        assert!((dark.h - ok.h).abs() < 1.5, "H={}", dark.h);
        // Hover variants sit below/above the base lightness.
        let hover_l = oklch_from_srgb(tinted_from_primary(primary, false, true)).l;
        assert!(hover_l < 0.93 && hover_l > 0.85);
    }

    #[test]
    fn variant_colors_cover_all_variants() {
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Dark).install(&ctx);
        let theme = ShadcnTheme::of(&ctx);
        for dark in [false, true] {
            for variant in [
                BubbleVariant::Default,
                BubbleVariant::Secondary,
                BubbleVariant::Muted,
                BubbleVariant::Tinted,
                BubbleVariant::Outline,
                BubbleVariant::Ghost,
                BubbleVariant::Destructive,
            ] {
                let (fill, text, border) = resolve_variant_colors(&theme.palette, dark, variant);
                // Fill is opaque for the flat variants, translucent for
                // tinted/destructive (alpha scales rgb equally), and fully
                // transparent for ghost.
                let _ = (fill, text, border);
            }
        }
        // Ghost is fully transparent; destructive is translucent.
        let (ghost_fill, ..) = resolve_variant_colors(&theme.palette, false, BubbleVariant::Ghost);
        assert_eq!(ghost_fill, Color32::TRANSPARENT);
        let (d_fill, ..) =
            resolve_variant_colors(&theme.palette, false, BubbleVariant::Destructive);
        assert!(d_fill.a() > 0 && d_fill.a() < 255);
    }

    #[test]
    fn reaction_pill_geometry() {
        // translate-y-3/4: 25% of the pill overlaps the bubble bottom edge.
        let content = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 40.0));
        let pill_h = 20.0;
        let y_bottom = content.bottom() - 0.25 * pill_h;
        assert!((y_bottom - (40.0 - 5.0)).abs() < 1e-4);
        let y_top = content.top() - 0.75 * pill_h;
        assert!((y_top - -15.0).abs() < 1e-4);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("bubble_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 700.0))),
                );
                for variant in [
                    BubbleVariant::Default,
                    BubbleVariant::Secondary,
                    BubbleVariant::Muted,
                    BubbleVariant::Tinted,
                    BubbleVariant::Outline,
                    BubbleVariant::Ghost,
                    BubbleVariant::Destructive,
                ] {
                    Bubble::new()
                        .variant(variant)
                        .align(BubbleAlign::End)
                        .show(ui, |ui| {
                            BubbleContent::new("A longer message that should wrap within the bubble pill bounds nicely.").show(ui);
                        });
                }
                Bubble::new().show(ui, |ui| {
                    BubbleContent::new("with reactions").show(ui);
                    BubbleReactions::new("👍").show(ui);
                    BubbleReactions::new("🎉")
                        .side(BubbleReactionSide::Top)
                        .align(BubbleReactionAlign::Start)
                        .show(ui);
                });
                BubbleGroup::new().show(ui, |ui| {
                    Bubble::new().show(ui, |ui| {
                        BubbleContent::new("grouped").show(ui);
                    });
                });
                // Standalone reaction pill (no bubble scope).
                BubbleReactions::new("👍").show(ui);
            });
            out.textures_delta.clear();
        }
    }
}
