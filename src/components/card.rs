//! shadcn `card.tsx` — a bordered content container with slot children.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/card.tsx`. The base recipe is
//! `rounded-xl border bg-card py-6 gap-6 text-card-foreground shadow-sm`
//! with `px-6` slots; the nova style overrides the border to a
//! `ring-foreground/10` hairline, tightens the spacing to `--card-spacing`
//! (16px), sets `text-sm`, drops the shadow, and gives the footer a
//! `bg-muted/50 border-t` full-bleed band.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Card::new().show(ui, |ui| {
//!     CardHeader::new().show(ui, |ui| {
//!         CardTitle::new("Account").show(ui);
//!         CardDescription::new("Manage your billing details.").show(ui);
//!     });
//!     CardContent::new().show(ui, |ui| {
//!         ui.label("Card content");
//!     });
//!     CardFooter::new().show(ui, |ui| {
//!         Button::new("Save").show(ui);
//!     });
//! });
//! ```

use crate::components::avatar::{cover_uv, resolve_texture};
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{Color32, CornerRadius, FontId, ImageSource, InnerResponse, Margin, Pos2, Rect, Ui};

/// Card density presets, mirroring the source's nova `data-[size=sm]`
/// variant (`--card-spacing: 12px` + `text-sm` title). The base recipe has
/// no size variants, so [`CardSize::Sm`] only changes nova cards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardSize {
    /// The default card (`--card-spacing: 16px` in nova).
    #[default]
    Default,
    /// The compact card (`--card-spacing: 12px` in nova).
    Sm,
}

/// The nova `--card-spacing` token: `24px` in the base recipe, `16px` in
/// nova and `12px` for nova's `data-[size=sm]` card.
pub(crate) fn card_spacing(style: Style, size: CardSize) -> f32 {
    match (style, size) {
        (Style::Nova, CardSize::Default) => 16.0,
        (Style::Nova, CardSize::Sm) => 12.0,
        _ => 24.0,
    }
}

/// Resolved card surface decoration (pure, unit-testable).
///
/// Nova replaces the `border` with a translucent foreground ring, drops the
/// shadow and gives the title `text-base` (`text-sm` on `size=sm`); other
/// styles use `border` + `shadow-sm` and inherit the 16px body size.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CardColors {
    pub fill: Color32,
    pub stroke: egui::Stroke,
    pub title_size: f32,
    /// Title line height multiplier: `leading-none` (1.0) in the base
    /// recipe, `leading-snug` (1.375) in nova.
    pub title_line_height: f32,
}

pub(crate) fn resolve_card_colors(p: &Palette, style: Style, size: CardSize) -> CardColors {
    match style {
        Style::Nova => CardColors {
            fill: p.card,
            stroke: egui::Stroke::new(1.0, color_alpha(p.foreground, 0.1)),
            title_size: match size {
                CardSize::Sm => crate::theme::tokens::type_scale::SM,
                CardSize::Default => crate::theme::tokens::type_scale::BASE,
            },
            title_line_height: 1.375, // leading-snug
        },
        _ => CardColors {
            fill: p.card,
            stroke: egui::Stroke::new(1.0, p.border),
            title_size: crate::theme::tokens::type_scale::BASE,
            title_line_height: 1.0, // leading-none
        },
    }
}

/// Enclosing-card context shared with [`CardTitle`] through the ui temp
/// stack (the nova `group-data-[size=sm]/card` selectors).
#[derive(Clone, Copy, Debug, Default)]
struct CardCtx {
    size: CardSize,
}

fn card_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_card_ctx_stack")
}

fn push_card(ui: &Ui, ctx: CardCtx) {
    let mut stack: Vec<CardCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(card_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(card_stack_id(), stack));
}

fn pop_card(ui: &Ui) {
    let mut stack: Vec<CardCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(card_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(card_stack_id(), stack));
}

fn current_card(ui: &Ui) -> Option<CardCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<CardCtx>>(card_stack_id()))
        .and_then(|s| s.last().copied())
}

/// Header geometry shared with [`CardAction`] through the ui temp stack.
#[derive(Clone, Copy, Debug, Default)]
struct HeaderCtx {
    left: f32,
    top: f32,
    right: f32,
}

fn header_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_card_header_stack")
}

fn push_header(ui: &Ui, ctx: HeaderCtx) {
    let mut stack: Vec<HeaderCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(header_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx()
        .data_mut(|d| d.insert_temp(header_stack_id(), stack));
}

fn pop_header(ui: &Ui) {
    let mut stack: Vec<HeaderCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(header_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(header_stack_id(), stack));
}

fn current_header(ui: &Ui) -> Option<HeaderCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<HeaderCtx>>(header_stack_id()))
        .and_then(|s| s.last().copied())
}

/// A shadcn card container. See the [module docs](self) for an example.
#[derive(Clone, Debug, Default)]
pub struct Card<'a> {
    id_salt: Option<String>,
    size: CardSize,
    image_top: Option<ImageSource<'a>>,
}

impl<'a> Card<'a> {
    /// Creates a card.
    pub fn new() -> Self {
        Self {
            id_salt: None,
            size: CardSize::default(),
            image_top: None,
        }
    }

    /// Sets a stable id salt (Card keeps no state, but nested containers
    /// derive ids from the scope).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Sets the card size preset. Only the nova style implements
    /// `data-[size=sm]` (`--card-spacing: 12px`, `text-sm` title); other
    /// styles are unaffected.
    pub fn size(mut self, size: CardSize) -> Self {
        self.size = size;
        self
    }

    /// Paints a full-bleed image above the card content, with rounded top
    /// corners and no top padding (the nova `has-[>img:first-child]:pt-0` /
    /// `rounded-t-xl` media slot). Texture sources paint immediately;
    /// URI/bytes sources reserve no space until they resolve.
    pub fn image_top(mut self, image: impl Into<ImageSource<'a>>) -> Self {
        self.image_top = Some(image.into());
        self
    }

    /// Renders the card around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_card_colors(&p, theme.style, self.size);
        let nova = theme.style == Style::Nova;
        let spacing = card_spacing(theme.style, self.size);
        let radius = theme.radius.xl_u8();
        let dark = theme.mode.is_dark();
        let shadow = if nova {
            egui::epaint::Shadow::NONE
        } else {
            crate::utils::shadow::shadow_sm(dark)
        };
        // Nova `has-[>img:first-child]:pt-0` — a top image bleeds to the
        // card edge (base cards use the same full-bleed media extension).
        let pad_v = if self.image_top.is_some() {
            0
        } else {
            spacing as i8
        };

        let frame = egui::Frame::new()
            .fill(c.fill)
            .stroke(c.stroke)
            .corner_radius(CornerRadius::same(radius))
            .shadow(shadow)
            .inner_margin(Margin::symmetric(0, pad_v));

        push_card(ui, CardCtx { size: self.size });
        let image_top = self.image_top.clone();
        let inner = frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width()); // `w-full`
                                                    // Card text defaults to `text-card-foreground`; widgets that
                                                    // paint their own colors (buttons etc.) are unaffected.
            ui.style_mut().visuals.override_text_color = Some(p.card_foreground);
            ui.spacing_mut().item_spacing.y = spacing; // gap-6 / nova gap-4
            if let Some(src) = image_top {
                paint_image_top(ui, &src, radius);
            }
            content(ui)
        });
        pop_card(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// Paints the [`Card::image_top`] media slot: a full-width image with
/// rounded top corners, height derived from the texture's aspect ratio.
fn paint_image_top(ui: &mut Ui, source: &ImageSource<'_>, radius: u8) {
    let width = ui.available_width().max(1.0);
    let Some(tex) = resolve_texture(ui, source, width) else {
        return;
    };
    let height = if tex.size.x > 0.0 {
        width * tex.size.y / tex.size.x
    } else {
        0.0
    };
    let (rect, _) = ui.allocate_exact_size(
        egui::Vec2::new(width, height.max(1.0)),
        egui::Sense::hover(),
    );
    if !ui.is_rect_visible(rect) {
        return;
    }
    let uv = cover_uv(tex.size, rect.size());
    let corners = CornerRadius {
        nw: radius,
        ne: radius,
        sw: 0,
        se: 0,
    };
    ui.painter().add(
        egui::epaint::RectShape::filled(rect, corners, Color32::WHITE).with_texture(tex.id, uv),
    );
}

/// `CardHeader` — the `gap-2 px-6` (nova `gap-1 px-4`) title block.
///
/// Children flow vertically; a [`CardAction`] shown inside is lifted to the
/// header's top-right corner, like the source's two-column grid.
#[derive(Clone, Debug, Default)]
pub struct CardHeader {
    id_salt: Option<String>,
}

impl CardHeader {
    /// Creates a card header.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let nova = theme.style == Style::Nova;
        // `gap-2` base / `gap-1` nova.
        let gap = if nova { 4.0 } else { 8.0 };
        let pad = card_spacing(
            theme.style,
            current_card(ui).map(|c| c.size).unwrap_or_default(),
        );

        let inner = egui::Frame::new()
            .inner_margin(Margin::symmetric(pad as i8, 0))
            .show(ui, |ui| {
                // Record the header content area for CardAction anchoring,
                // inset by the slot's `px-6`/`px-4` padding (recorded inside
                // the padded frame so `right` excludes the padding).
                let cursor = ui.cursor();
                push_header(
                    ui,
                    HeaderCtx {
                        left: cursor.left(),
                        top: cursor.top(),
                        right: cursor.left() + ui.available_width(),
                    },
                );
                let ret = ui
                    .vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = gap;
                        content(ui)
                    })
                    .inner;
                pop_header(ui);
                ret
            });

        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `CardTitle` — `leading-none font-semibold` (nova `text-base
/// leading-snug font-medium`); `text-sm` on nova's `size=sm` card.
#[derive(Clone, Debug)]
pub struct CardTitle {
    text: String,
}

impl CardTitle {
    /// Creates a card title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let size = current_card(ui).map(|c| c.size).unwrap_or_default();
        let c = resolve_card_colors(&p, theme.style, size);
        let font = FontId::proportional(c.title_size);
        // The line height is set explicitly (`leading-none` /
        // `leading-snug`); layout is left-to-right, no wrap.
        let format = egui::TextFormat {
            font_id: font,
            color: p.card_foreground,
            line_height: Some(c.title_size * c.title_line_height),
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = f32::INFINITY;
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.card_foreground);
        }
        response
    }
}

/// `CardDescription` — `text-sm text-muted-foreground`, wraps to the width.
#[derive(Clone, Debug)]
pub struct CardDescription {
    text: String,
}

impl CardDescription {
    /// Creates a card description.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout(self.text, font, p.muted_foreground, ui.available_width());
        let (rect, response) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.muted_foreground);
        }
        response
    }
}

/// `CardAction` — the `col-start-2 row-start-1 row-span-2 justify-self-end`
/// slot. Shown inside a [`CardHeader`], it paints at the header's top-right
/// corner without consuming flow height.
#[derive(Clone, Debug, Default)]
pub struct CardAction {
    id_salt: Option<String>,
}

impl CardAction {
    /// Creates a card action slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the action contents at the header's top-right.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let ctx = current_header(ui);
        match ctx {
            Some(ctx) if ctx.right > ctx.left => {
                // Lay out in a right-to-left child Ui pinned to the header's
                // top-right, so the content hugs the right edge regardless
                // of its natural width. The flow position is unchanged: the
                // action overlays the title rows like the source grid.
                let region = Rect::from_min_max(
                    Pos2::new(ctx.left, ctx.top),
                    Pos2::new(ctx.right, ctx.top + 400.0),
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(region)
                        .layout(egui::Layout::right_to_left(egui::Align::Min)),
                );
                let ret = content(&mut child);
                let response = child.response();
                InnerResponse::new(ret, response)
            }
            _ => {
                // Outside a header: lay out in the flow, right-aligned.
                let inner = ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), content);
                InnerResponse::new(inner.inner, inner.response)
            }
        }
    }
}

/// `CardContent` — the `px-6` / nova `px-4` body slot.
#[derive(Clone, Debug, Default)]
pub struct CardContent {
    id_salt: Option<String>,
}

impl CardContent {
    /// Creates a card content slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the content slot.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let pad = card_spacing(
            theme.style,
            current_card(ui).map(|c| c.size).unwrap_or_default(),
        ) as i8;
        egui::Frame::new()
            .inner_margin(Margin::symmetric(pad, 0))
            .show(ui, content)
    }
}

/// `CardFooter` — `flex items-center px-6`; in nova it is a full-bleed
/// `bg-muted/50 border-t p-4 rounded-b-xl` band.
#[derive(Clone, Debug, Default)]
pub struct CardFooter {
    id_salt: Option<String>,
}

impl CardFooter {
    /// Creates a card footer.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the footer around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let nova = theme.style == Style::Nova;
        let spacing = card_spacing(
            theme.style,
            current_card(ui).map(|c| c.size).unwrap_or_default(),
        );

        let painter = ui.painter().clone();
        let band_idx = painter.add(egui::Shape::Noop);

        // Base recipe: `flex items-center px-6` (no fill). Nova: a padded
        // band whose fill bleeds to the card edges. The source is a flex
        // row, so children are laid out horizontally.
        let pad_v = if nova { spacing as i8 } else { 0 };
        let inner = egui::Frame::new()
            .inner_margin(Margin::symmetric(spacing as i8, pad_v))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    content(ui)
                })
                .inner
            });

        if nova {
            // Extend the band over the card's bottom padding and the 1px
            // card stroke, so it bleeds to the card's bottom edge.
            let s = 1.0; // card stroke width (nova ring-foreground/10)
            let radius = theme.radius.xl_u8();
            let band = Rect::from_min_max(
                Pos2::new(inner.response.rect.left() - s, inner.response.rect.min.y),
                Pos2::new(
                    inner.response.rect.right() + s,
                    inner.response.rect.max.y + spacing + s,
                ),
            );
            let corner = CornerRadius {
                nw: 0,
                ne: 0,
                sw: radius,
                se: radius,
            };
            painter.set(
                band_idx,
                egui::Shape::Rect(egui::epaint::RectShape::filled(
                    band,
                    corner,
                    color_alpha(p.muted, 0.5),
                )),
            );
            // `border-t` on top of the band.
            painter.line_segment(
                [
                    Pos2::new(band.left(), band.top() + 0.5),
                    Pos2::new(band.right(), band.top() + 0.5),
                ],
                egui::Stroke::new(1.0, p.border),
            );
            // The band covered the card ring on its sides/bottom; restore it.
            painter.rect_stroke(
                band,
                corner,
                ring_stroke(&p),
                egui::epaint::StrokeKind::Inside,
            );
        } else {
            painter.set(band_idx, egui::Shape::Noop);
        }

        inner
    }
}

/// The nova card ring (`ring-foreground/10`).
fn ring_stroke(p: &Palette) -> egui::Stroke {
    egui::Stroke::new(1.0, color_alpha(p.foreground, 0.1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Vec2;

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn spacing_matches_style_recipes() {
        // nova `--card-spacing: --spacing(4)` (sm: `--spacing(3)`),
        // base `py-6 gap-6`.
        assert_eq!(card_spacing(Style::Nova, CardSize::Default), 16.0);
        assert_eq!(card_spacing(Style::Nova, CardSize::Sm), 12.0);
        assert_eq!(card_spacing(Style::Vega, CardSize::Default), 24.0);
        assert_eq!(card_spacing(Style::Vega, CardSize::Sm), 24.0);
    }

    #[test]
    fn nova_uses_ring_and_larger_title() {
        let p = palette(Mode::Light);
        let nova = resolve_card_colors(&p, Style::Nova, CardSize::Default);
        assert_eq!(nova.stroke.color, color_alpha(p.foreground, 0.1));
        assert_eq!(nova.fill, p.card);
        assert_eq!(nova.title_size, 16.0);
        assert_eq!(nova.title_line_height, 1.375); // leading-snug

        // Base CardTitle has no size class and inherits the 16px body size
        // with `leading-none`.
        let base = resolve_card_colors(&p, Style::Vega, CardSize::Default);
        assert_eq!(base.stroke.color, p.border);
        assert_eq!(base.title_size, 16.0);
        assert_eq!(base.title_line_height, 1.0);

        // nova `group-data-[size=sm]/card:text-sm`.
        let nova_sm = resolve_card_colors(&p, Style::Nova, CardSize::Sm);
        assert_eq!(nova_sm.title_size, 14.0);
        // The base recipe has no size variant.
        let base_sm = resolve_card_colors(&p, Style::Vega, CardSize::Sm);
        assert_eq!(base_sm.title_size, 16.0);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("card_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                Card::new().show(ui, |ui| {
                    CardHeader::new().show(ui, |ui| {
                        CardTitle::new("Title").show(ui);
                        CardDescription::new("Description").show(ui);
                    });
                    CardContent::new().show(ui, |ui| {
                        ui.label("Body");
                    });
                    CardFooter::new().show(ui, |ui| {
                        ui.label("Footer");
                    });
                });
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn header_ctx_stacks_and_pops() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("card_header_stack_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            assert!(current_header(ui).is_none());
            CardHeader::new().show(ui, |ui| {
                let h = current_header(ui).expect("inside header");
                assert_eq!(h.top, ui.cursor().top());
                // The action region is inset by the slot's `px-6`/`px-4`
                // padding, not flush with the card edge.
                assert!(h.right < 400.0 - 8.0, "right = {}", h.right);
                CardAction::new().show(ui, |ui| {
                    ui.label("action");
                });
            });
            assert!(current_header(ui).is_none());
        });
        out.textures_delta.clear();
    }

    #[test]
    fn footer_lays_out_children_horizontally() {
        // `flex items-center`: sibling footer children sit side by side.
        use std::cell::RefCell;
        let ctx = egui::Context::default();
        let rects: RefCell<Vec<egui::Rect>> = RefCell::new(Vec::new());
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("card_footer_layout_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            Card::new().show(ui, |ui| {
                CardFooter::new().show(ui, |ui| {
                    let (a, _) =
                        ui.allocate_exact_size(Vec2::new(60.0, 20.0), egui::Sense::hover());
                    let (b, _) =
                        ui.allocate_exact_size(Vec2::new(60.0, 20.0), egui::Sense::hover());
                    rects.borrow_mut().push(a);
                    rects.borrow_mut().push(b);
                });
            });
        });
        out.textures_delta.clear();
        let rects = rects.into_inner();
        assert_eq!(rects.len(), 2);
        assert!(
            (rects[0].top() - rects[1].top()).abs() < 2.0,
            "children must share a row: {:?}",
            rects
        );
        assert!(rects[0].right() <= rects[1].left() + 0.5);
    }
}
