//! shadcn `card.tsx` — a bordered content container with slot children.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/card.tsx`. The base recipe is
//! `rounded-xl border bg-card py-6 gap-6 text-card-foreground shadow-sm`
//! with `px-6` slots; the nova style keeps the `border` and `shadow-sm`
//! (LAYER: the `.cn-card` rule declares neither, so both survive) and adds a
//! `ring-1 ring-foreground/10` outside the border, tightens the spacing to
//! `--card-spacing` (16px), sets `text-sm`, clips overflow, and gives the
//! footer a `bg-muted/50 border-t` band that bleeds to the card's inner
//! bottom edge.
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
/// Both styles paint the 1px `border` (`--border` token). Nova ADDS a 1px
/// `ring-foreground/10` outside it (§0 A2 — the `.cn-card` rule declares no
/// border change, so base `border` survives under LAYER); other styles have
/// no ring. The shadow is `shadow-sm` in both styles (§0 A1) — see
/// [`card_shadow`]. The title is `text-base` in nova (`text-sm` on
/// `size=sm`) and inherits the 16px body size otherwise.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CardColors {
    pub fill: Color32,
    /// The 1px card border (`--border`), both styles.
    pub stroke: egui::Stroke,
    /// The 1px ring outside the border (`foreground` at 10%): `Some` under
    /// nova, `None` under other styles.
    pub ring: Option<egui::Stroke>,
    pub title_size: f32,
    /// Title line height multiplier: `leading-none` (1.0) in the base
    /// recipe, `leading-snug` (1.375) in nova.
    pub title_line_height: f32,
}

/// The card title's font: `font-semibold` (600) in the base recipe,
/// `font-medium` (500) in nova — `text-base` (16px), `text-sm` (14px) on
/// nova's `size=sm` card. Pure, unit-testable (§2.3, §5.2).
pub(crate) fn card_title_font(style: Style, size: CardSize) -> FontId {
    let px = match (style, size) {
        (Style::Nova, CardSize::Sm) => crate::theme::tokens::type_scale::SM,
        _ => crate::theme::tokens::type_scale::BASE,
    };
    match style {
        Style::Nova => crate::theme::fonts::medium(px),
        _ => crate::theme::fonts::semibold(px),
    }
}

/// The card shadow: `shadow-sm` in BOTH styles (§0 A1 — the nova `.cn-card`
/// rule declares no shadow utility, so base `shadow-sm` survives under
/// LAYER). Pure, unit-testable.
pub(crate) fn card_shadow(dark: bool) -> egui::epaint::Shadow {
    crate::utils::shadow::shadow_sm(dark)
}

/// The card's vertical padding `(top, bottom)` per the frozen §3.2 table
/// (N1): `pt-0` zeroes the TOP padding only, and only under nova; the bottom
/// padding is 0 under nova when a footer is present (the band bleeds to the
/// card's inner bottom edge). Pure, unit-testable.
pub(crate) fn card_vertical_padding(
    style: Style,
    size: CardSize,
    image_top: bool,
    has_footer: bool,
) -> (f32, f32) {
    let spacing = card_spacing(style, size);
    match style {
        Style::Nova => (
            if image_top { 0.0 } else { spacing },
            if has_footer { 0.0 } else { spacing },
        ),
        _ => (spacing, spacing),
    }
}

pub(crate) fn resolve_card_colors(p: &Palette, style: Style, size: CardSize) -> CardColors {
    let font = card_title_font(style, size);
    match style {
        Style::Nova => CardColors {
            fill: p.card,
            stroke: egui::Stroke::new(1.0, p.border),
            ring: Some(ring_stroke(p)),
            title_size: font.size,
            title_line_height: 1.375, // leading-snug
        },
        _ => CardColors {
            fill: p.card,
            stroke: egui::Stroke::new(1.0, p.border),
            ring: None,
            title_size: font.size,
            title_line_height: 1.0, // leading-none
        },
    }
}

/// Enclosing-card context shared with [`CardTitle`] through the ui temp
/// stack (the nova `group-data-[size=sm]/card` selectors). `has_footer` is
/// set by [`CardFooter::show`] so [`Card::show`] can apply the §3.2 bottom
/// padding (0 under nova with a footer) after the content closure runs.
#[derive(Clone, Copy, Debug, Default)]
struct CardCtx {
    size: CardSize,
    has_footer: bool,
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

/// Records that a [`CardFooter`] was shown inside the enclosing card (used
/// for the §3.2 bottom padding; no-op outside a card).
fn mark_footer(ui: &Ui) {
    let mut stack: Vec<CardCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(card_stack_id()))
        .unwrap_or_default();
    if let Some(top) = stack.last_mut() {
        top.has_footer = true;
    }
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

    /// Paints a top image above the card content (the §2.8 docs-markup
    /// pattern — not a `.tsx` slot). Under nova it is full-bleed: no top
    /// padding (`has-[>img:first-child]:pt-0`) and 14px top corners. Under
    /// other styles (coordinator decision D2(a), source-faithful) it sits
    /// below the 24px top padding with square corners. Texture sources paint
    /// immediately; URI/bytes sources reserve no space until they resolve.
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
        // A1: `shadow-sm` in BOTH styles — the nova `.cn-card` rule declares
        // no shadow utility, so the base shadow survives under LAYER.
        let shadow = card_shadow(dark);
        // N1 (§3.2): `pt-0` zeroes the TOP padding only, and only under
        // nova. The bottom padding depends on whether a footer is present,
        // which is only known after the content closure runs — the frame's
        // bottom margin is 0 and the padding is added after content below.
        let has_image_top = self.image_top.is_some();
        let (pad_top, _) = card_vertical_padding(theme.style, self.size, has_image_top, false);

        let frame = egui::Frame::new()
            .fill(c.fill)
            .stroke(c.stroke)
            .corner_radius(CornerRadius::same(radius))
            .shadow(shadow)
            .inner_margin(Margin {
                left: 0,
                right: 0,
                top: pad_top as i8,
                bottom: 0,
            });

        push_card(
            ui,
            CardCtx {
                size: self.size,
                has_footer: false,
            },
        );
        let image_top = self.image_top.clone();
        let inner = frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width()); // `w-full`
                                                    // Card text defaults to `text-card-foreground`; widgets that
                                                    // paint their own colors (buttons etc.) are unaffected.
            ui.style_mut().visuals.override_text_color = Some(p.card_foreground);
            ui.spacing_mut().item_spacing.y = spacing; // gap-6 / nova gap-4
            if nova {
                // N3 (§3.6): `overflow-hidden` under nova only — clip
                // descendants to the card's padding box (the rounded rect's
                // bounding box; the per-part corner radii handle the
                // corners). The left/right/top edges are exact; the bottom
                // is the available bound — the card's height is
                // content-driven, so an exact bottom edge cannot be known
                // before layout. The concrete overflow source, the action
                // overlay, is confined to the header height instead.
                let max = ui.max_rect();
                ui.set_clip_rect(Rect::from_min_max(
                    Pos2::new(max.left(), max.top() - pad_top),
                    Pos2::new(max.right(), max.bottom()),
                ));
            }
            let content_top = ui.max_rect().top();
            if let Some(src) = image_top {
                paint_image_top(ui, &src, radius, nova);
            }
            let ret = content(ui);
            // Bottom padding (§3.2): 0 under nova when a footer is present
            // (the band bleeds to the card's inner bottom edge); otherwise
            // `spacing`. The layout cursor already sits one `item_spacing`
            // (= `spacing`) past the last slot, so expanding the frame to
            // the cursor supplies the full bottom padding; with a footer
            // nothing is added. An empty card has no pending gap, so the
            // padding is allocated explicitly.
            let has_footer = current_card(ui).map(|c| c.has_footer).unwrap_or(false);
            let (_, pad_bottom) =
                card_vertical_padding(theme.style, self.size, has_image_top, has_footer);
            if pad_bottom > 0.0 {
                if ui.cursor().top() > content_top {
                    ui.add_space(0.0);
                } else {
                    ui.add_space(pad_bottom);
                }
            }
            ret
        });
        // A2: the 1px ring paints OUTSIDE the 1px border (the frame's
        // stroke), concentric corners (`radius + 1`), under nova only.
        if let Some(ring) = c.ring {
            let rect = inner.response.rect;
            let corners = CornerRadius {
                nw: radius.saturating_add(1),
                ne: radius.saturating_add(1),
                sw: radius.saturating_add(1),
                se: radius.saturating_add(1),
            };
            ui.painter().rect_stroke(
                rect.expand(1.0),
                corners,
                ring,
                egui::epaint::StrokeKind::Inside,
            );
        }
        pop_card(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// Paints the [`Card::image_top`] media slot: a full-width image, height
/// derived from the texture's aspect ratio. Under nova the top corners are
/// rounded (`rounded-t-xl`); under other styles the corners are square
/// (D2(a) — no rounding rule in base).
fn paint_image_top(ui: &mut Ui, source: &ImageSource<'_>, radius: u8, rounded: bool) {
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
    let top = if rounded { radius } else { 0 };
    let corners = CornerRadius {
        nw: top,
        ne: top,
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
        // `font-semibold` (600) in the base recipe, `font-medium` (500) in
        // nova (§2.3) — the embedded Inter faces, wired here.
        let font = card_title_font(theme.style, size);
        // The line height is set explicitly (`leading-none` /
        // `leading-snug`); the title wraps at the header content width
        // (N2 — no `whitespace-nowrap` in either recipe). With an action
        // present the wrap width is the full header width, not the `1fr`
        // column — the D1 overlay limit, documented in the module docs.
        let format = egui::TextFormat {
            font_id: font,
            color: p.card_foreground,
            line_height: Some(c.title_size * c.title_line_height),
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = ui.available_width().max(1.0);
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
        // `text-sm` default line height is 20px in both styles (§2.4) — no
        // explicit `leading` class in either recipe, so the default is the
        // frozen value; pin it explicitly rather than relying on the font's
        // natural line height.
        let format = egui::TextFormat {
            font_id: font,
            color: p.muted_foreground,
            line_height: Some(20.0),
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = ui.available_width().max(1.0);
        let galley = ui.painter().layout_job(job);
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
                // action overlays the title rows like the source grid. The
                // region spans only the header content laid out so far (at
                // least one small action's height) — under nova the card
                // clips overflow (N3), so an unbounded region could paint
                // past the card. A taller-than-header action does not grow
                // the header (D1 limit).
                let height = (ui.cursor().top() - ctx.top).max(32.0);
                let region = Rect::from_min_max(
                    Pos2::new(ctx.left, ctx.top),
                    Pos2::new(ctx.right, ctx.top + height),
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

/// `CardFooter` — `flex items-center px-6` with 0 gap; in nova it is a
/// `bg-muted/50 border-t p-4 rounded-b-xl` band filling the card's inner width.
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
        // Tells the enclosing card a footer is present (§3.2 bottom padding).
        mark_footer(ui);
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
        // band that bleeds to the card's inner edges. The source is a flex
        // row, so children are laid out horizontally; the gap between
        // children is 0 (A3) — spacing is the caller's responsibility.
        let pad_v = if nova { spacing as i8 } else { 0 };
        let inner = egui::Frame::new()
            .inner_margin(Margin::symmetric(spacing as i8, pad_v))
            .show(ui, |ui| {
                // The footer is a full-width block row (`flex` in the
                // source), so the frame spans the card's inner width even
                // when its children are narrow — the nova band (§3.4) IS
                // this frame's rect, so without this it would be
                // content-width. Same `w-full` mechanism as `Card::show`.
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    content(ui)
                })
                .inner
            });

        if nova {
            // §3.4: the band IS the footer frame's rect — the card's content
            // area is already inset by the 1px border, so the frame spans
            // exactly the card's inner width and its bottom is the card's
            // inner bottom edge (the bottom padding is 0 with a footer,
            // §3.2). The card's 1px border + 1px ring paint outside the
            // band, so neither is covered or needs restoring. This assumes
            // the footer is the card's last slot (source composition order,
            // §8); a slot after the footer would sit below the band.
            let radius = theme.radius.xl_u8();
            let band = inner.response.rect;
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
        } else {
            painter.set(band_idx, egui::Shape::Noop);
        }

        inner
    }
}

/// The nova card ring (`ring-1 ring-foreground/10`), painted outside the
/// 1px border by [`Card::show`] (§0 A2).
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
    fn nova_keeps_border_and_adds_ring() {
        // §0 A1/A2: nova keeps the base 1px `border` and ADDS the
        // `ring-1 ring-foreground/10` outside it — it does NOT replace the
        // border with the ring (the old assertion that the nova stroke was
        // the ring pinned the audit's wrong "ring-only" row).
        let p = palette(Mode::Light);
        let nova = resolve_card_colors(&p, Style::Nova, CardSize::Default);
        assert_eq!(nova.stroke.width, 1.0);
        assert_eq!(nova.stroke.color, p.border);
        assert_eq!(
            nova.ring,
            Some(egui::Stroke::new(1.0, color_alpha(p.foreground, 0.1)))
        );
        assert_eq!(nova.fill, p.card);
        assert_eq!(nova.title_size, 16.0);
        assert_eq!(nova.title_line_height, 1.375); // leading-snug

        // Base CardTitle has no size class and inherits the 16px body size
        // with `leading-none`; no ring.
        let base = resolve_card_colors(&p, Style::Vega, CardSize::Default);
        assert_eq!(base.stroke.width, 1.0);
        assert_eq!(base.stroke.color, p.border);
        assert_eq!(base.ring, None);
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
    fn shadow_is_sm_in_both_styles() {
        // §0 A1: the nova `.cn-card` rule declares no shadow utility, so
        // base `shadow-sm` survives under nova — never `Shadow::NONE`.
        assert_ne!(card_shadow(false), egui::epaint::Shadow::NONE);
        assert_ne!(card_shadow(true), egui::epaint::Shadow::NONE);
        assert_eq!(card_shadow(false), crate::utils::shadow::shadow_sm(false));
        assert_eq!(card_shadow(true), crate::utils::shadow::shadow_sm(true));
    }

    #[test]
    fn vertical_padding_matches_n1_table() {
        // §3.2 — all ten table rows, plus the two nova-sm no-image rows.
        use Style::{Nova, Vega};
        // Base: 24/24 in every case (`pt-0` is nova-only).
        for (image, footer) in [(false, false), (false, true), (true, false), (true, true)] {
            assert_eq!(
                card_vertical_padding(Vega, CardSize::Default, image, footer),
                (24.0, 24.0),
                "base image={image} footer={footer}"
            );
        }
        // Nova: top zeroes with a top image only; bottom zeroes with a
        // footer only.
        assert_eq!(
            card_vertical_padding(Nova, CardSize::Default, false, false),
            (16.0, 16.0)
        );
        assert_eq!(
            card_vertical_padding(Nova, CardSize::Default, false, true),
            (16.0, 0.0)
        );
        assert_eq!(
            card_vertical_padding(Nova, CardSize::Default, true, false),
            (0.0, 16.0)
        );
        assert_eq!(
            card_vertical_padding(Nova, CardSize::Default, true, true),
            (0.0, 0.0)
        );
        // Nova-sm: same shape at 12px.
        assert_eq!(
            card_vertical_padding(Nova, CardSize::Sm, false, false),
            (12.0, 12.0)
        );
        assert_eq!(
            card_vertical_padding(Nova, CardSize::Sm, false, true),
            (12.0, 0.0)
        );
        assert_eq!(
            card_vertical_padding(Nova, CardSize::Sm, true, false),
            (0.0, 12.0)
        );
        assert_eq!(
            card_vertical_padding(Nova, CardSize::Sm, true, true),
            (0.0, 0.0)
        );
    }

    #[test]
    fn title_font_matches_recipe_weights() {
        // §2.3/§5.2: base `font-semibold` (600), nova `font-medium` (500);
        // nova-sm drops to 14px, base has no size variant.
        let fonts = crate::theme::fonts::semibold_family();
        assert_eq!(card_title_font(Style::Vega, CardSize::Default).size, 16.0);
        assert_eq!(
            card_title_font(Style::Vega, CardSize::Default).family,
            fonts
        );
        assert_eq!(card_title_font(Style::Nova, CardSize::Default).size, 16.0);
        assert_eq!(
            card_title_font(Style::Nova, CardSize::Default).family,
            crate::theme::fonts::medium_family()
        );
        assert_eq!(card_title_font(Style::Nova, CardSize::Sm).size, 14.0);
        assert_eq!(
            card_title_font(Style::Nova, CardSize::Sm).family,
            crate::theme::fonts::medium_family()
        );
        assert_eq!(card_title_font(Style::Vega, CardSize::Sm).size, 16.0);
    }

    #[test]
    fn light_foreground_follows_themes_ts() {
        // §5.4 / R1: light `foreground` / `card-foreground` is
        // `oklch(0.145 0 0)` (themes.ts), not `oklch(0% 0 0)` (globals.css).
        let expected = crate::theme::Oklch::parse("oklch(0.145 0 0)")
            .unwrap()
            .to_color32();
        let p = palette(Mode::Light);
        assert_eq!(p.foreground, expected);
        assert_eq!(p.card_foreground, expected);
    }

    #[test]
    fn title_wraps_at_header_width() {
        // §9 N2: the title wraps at the header content width (no `INFINITY`).
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx); // nova default
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("card_title_wrap_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 400.0))),
            );
            let resp =
                CardTitle::new("A very long card title that must wrap onto several lines").show(ui);
            assert!(
                resp.rect.width() <= 200.0 + 0.5,
                "title must not exceed the available width: {:?}",
                resp.rect
            );
            assert!(
                resp.rect.height() > 22.0 + 0.5,
                "title must wrap to more than one line: {:?}",
                resp.rect
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn description_line_height_is_20() {
        // §2.4: `text-sm` default line height is 20px in both styles.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("card_description_lh_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            let resp = CardDescription::new("One line of description").show(ui);
            assert!(
                (resp.rect.height() - 20.0).abs() < 0.5,
                "one-line description height: {:?}",
                resp.rect
            );
        });
        out.textures_delta.clear();
    }

    /// Renders a card in a fresh 400px-wide Ui and returns the card's outer
    /// height (the frame's response rect, including the 1px stroke top and
    /// bottom). Expected heights are computed from the frozen §3.1/§3.2
    /// values plus the 2px stroke: nova header 22+4+20 = 46, base header
    /// 16+8+20 = 44, gaps 16/24, content 30, nova footer 36+32 = 68. The
    /// footer row is 36 tall, not the 20 of the allocated child: the theme's
    /// `interact_size.y` floor is 36pt and `ui.horizontal` stretches the row
    /// to it (cf. `input_group.rs` "36pt theme floor").
    fn card_height(style: Style, build: impl FnOnce(&mut Ui)) -> f32 {
        let ctx = egui::Context::default();
        Preset::default()
            .with_style(style)
            .build(Mode::Light)
            .install(&ctx);
        let height: std::cell::Cell<f32> = std::cell::Cell::new(f32::NAN);
        // `run_ui` takes an `FnMut` closure, so the `FnOnce` build closure
        // rides in an `Option` and is taken exactly once inside.
        let mut build = Some(build);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("card_height_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            let resp = Card::new().show(ui, build.take().unwrap());
            height.set(resp.response.rect.height());
        });
        out.textures_delta.clear();
        height.get()
    }

    #[test]
    fn card_heights_match_padding_table() {
        // Nova, header + content (no footer): 16 + 46 + 16 + 30 + 16 + 2 = 126.
        let h = card_height(Style::Nova, |ui| {
            CardHeader::new().show(ui, |ui| {
                CardTitle::new("T").show(ui);
                CardDescription::new("D").show(ui);
            });
            CardContent::new().show(ui, |ui| {
                ui.allocate_exact_size(Vec2::new(60.0, 30.0), egui::Sense::hover());
            });
        });
        assert!((h - 126.0).abs() < 0.5, "nova no-footer card height: {h}");

        // Nova, header + content + footer: 16 + 46 + 16 + 30 + 16 + 68 + 0 + 2 = 194.
        // The footer child is 36 tall (a button-height child — the theme's
        // `interact_size.y` floor; a shorter child would still give a 36 row).
        let h = card_height(Style::Nova, |ui| {
            CardHeader::new().show(ui, |ui| {
                CardTitle::new("T").show(ui);
                CardDescription::new("D").show(ui);
            });
            CardContent::new().show(ui, |ui| {
                ui.allocate_exact_size(Vec2::new(60.0, 30.0), egui::Sense::hover());
            });
            CardFooter::new().show(ui, |ui| {
                ui.allocate_exact_size(Vec2::new(60.0, 36.0), egui::Sense::hover());
            });
        });
        assert!((h - 194.0).abs() < 0.5, "nova footer card height: {h}");

        // Base, header + content: 24 + 44 + 24 + 30 + 24 + 2 = 148.
        let h = card_height(Style::Vega, |ui| {
            CardHeader::new().show(ui, |ui| {
                CardTitle::new("T").show(ui);
                CardDescription::new("D").show(ui);
            });
            CardContent::new().show(ui, |ui| {
                ui.allocate_exact_size(Vec2::new(60.0, 30.0), egui::Sense::hover());
            });
        });
        assert!((h - 148.0).abs() < 0.5, "base no-footer card height: {h}");

        // Empty cards: padding top + bottom + 2px stroke, no slots.
        assert!(
            (card_height(Style::Nova, |_| {}) - 34.0).abs() < 0.5,
            "empty nova card height"
        );
        assert!(
            (card_height(Style::Vega, |_| {}) - 50.0).abs() < 0.5,
            "empty base card height"
        );
    }

    /// Flattens painted shapes into their `RectShape`s (recursing into
    /// `Shape::Vec`, e.g. the frame's shadow + fill pair).
    fn rect_shapes(shapes: &[egui::epaint::ClippedShape]) -> Vec<egui::epaint::RectShape> {
        fn push(shape: &egui::Shape, out: &mut Vec<egui::epaint::RectShape>) {
            match shape {
                egui::Shape::Rect(r) => out.push(r.clone()),
                egui::Shape::Vec(shapes) => {
                    for s in shapes {
                        push(s, out);
                    }
                }
                _ => {}
            }
        }
        let mut out = Vec::new();
        for c in shapes {
            push(&c.shape, &mut out);
        }
        out
    }

    #[test]
    fn painted_edge_and_band_match_spec() {
        // §0 A2 + §3.4, painted-shape level: the frame paints the 1px border
        // at the card rect, the ring paints 1px outside it, and the nova
        // footer band is exactly the footer frame's rect.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx); // nova default
        let p = palette(Mode::Light);
        let card_rect: std::cell::Cell<Rect> = std::cell::Cell::new(Rect::NOTHING);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("card_shapes_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            let resp = Card::new().show(ui, |ui| {
                CardHeader::new().show(ui, |ui| {
                    CardTitle::new("T").show(ui);
                    CardDescription::new("D").show(ui);
                });
                CardContent::new().show(ui, |ui| {
                    ui.allocate_exact_size(Vec2::new(60.0, 30.0), egui::Sense::hover());
                });
                CardFooter::new().show(ui, |ui| {
                    ui.allocate_exact_size(Vec2::new(60.0, 36.0), egui::Sense::hover());
                });
            });
            card_rect.set(resp.response.rect);
        });
        // Clear the texture deltas FIRST: if an assertion below panics,
        // dropping `out` with unapplied deltas panics again in its `Drop`
        // and aborts the whole test process.
        out.textures_delta.clear();
        let r = card_rect.get();
        // 16 + 46 + 16 + 30 + 16 + 68 + 0 + 2 = 194 (footer row 36: the
        // theme's `interact_size.y` floor — see `card_height`).
        assert!((r.height() - 194.0).abs() < 0.5, "card height: {r:?}");
        let rects = rect_shapes(&out.shapes);

        // Frame: fill `card`, 1px `border` stroke, at the card rect.
        let frame = rects
            .iter()
            .find(|s| s.fill == p.card)
            .expect("card frame shape");
        assert_eq!(frame.rect, r);
        assert_eq!(frame.stroke.width, 1.0);
        assert_eq!(frame.stroke.color, p.border);
        assert_eq!(frame.corner_radius.nw, 14);
        assert_eq!(frame.corner_radius.ne, 14);
        assert_eq!(frame.corner_radius.sw, 14);
        assert_eq!(frame.corner_radius.se, 14);

        // Ring: transparent fill, `foreground/10` stroke, 1px outside the card.
        let ring_color = color_alpha(p.foreground, 0.1);
        let ring = rects
            .iter()
            .find(|s| s.stroke.color == ring_color)
            .expect("nova ring shape");
        assert_eq!(ring.rect, r.expand(1.0));
        assert_eq!(ring.fill, Color32::TRANSPARENT);
        assert_eq!(ring.corner_radius.nw, 15);
        assert_eq!(ring.corner_radius.ne, 15);
        assert_eq!(ring.corner_radius.sw, 15);
        assert_eq!(ring.corner_radius.se, 15);

        // Band: `muted/50` fill, exactly the footer frame's rect —
        // [R.left+1, R.top+125, R.right-1, R.bottom-1], bottom corners 14px.
        // (Top: 16 pad + 1 stroke + 46 header + 16 gap + 30 content + 16 gap;
        // height 68 = 36 row + 32 padding, so its bottom IS the card's inner
        // bottom edge — the §3.2 zero bottom padding, rendered.)
        let band_fill = color_alpha(p.muted, 0.5);
        let band = rects
            .iter()
            .find(|s| s.fill == band_fill)
            .expect("footer band shape");
        let expected_band = Rect::from_min_max(
            Pos2::new(r.left() + 1.0, r.top() + 125.0),
            Pos2::new(r.right() - 1.0, r.bottom() - 1.0),
        );
        assert_eq!(band.rect, expected_band);
        assert_eq!(band.corner_radius.nw, 0);
        assert_eq!(band.corner_radius.ne, 0);
        assert_eq!(band.corner_radius.sw, 14);
        assert_eq!(band.corner_radius.se, 14);
        assert_eq!(band.stroke, egui::Stroke::NONE);
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
        // A3: the gap between footer children is 0 — they sit adjacent.
        assert!(
            (rects[1].left() - rects[0].right()).abs() < 0.5,
            "footer children must be adjacent (gap 0): {:?}",
            rects
        );
        assert!(rects[0].right() <= rects[1].left() + 0.5);
    }
}
