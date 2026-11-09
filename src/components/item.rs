//! shadcn `item.tsx` — flexible list item rows with media/content/actions.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/item.tsx` with the nova
//! recipe as the baseline (`rounded-lg border`, sizes `px-3 py-2.5 gap-2.5`
//! default/sm and `px-2.5 py-2 gap-2` for the nova-only `xs`). Variants:
//! `default` (transparent), `outline` (bordered), `muted` (`bg-muted/50`).
//!
//! Slots: [`ItemMedia`] (icon box or image), [`ItemContent`] (title +
//! description), [`ItemActions`] (right-aligned inside the item, like the
//! source's `justify-between` rows), plus [`ItemHeader`]/[`ItemFooter`]
//! full-width rows, [`ItemGroup`] (vertical list with a per-size gap) and
//! [`ItemSeparator`].
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ItemGroup::new().show(ui, |ui| {
//!     Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
//!         ItemMedia::icon(Icon::User).show(ui);
//!         ItemContent::new().show(ui, |ui| {
//!             ItemTitle::new("Ada Lovelace").show(ui);
//!             ItemDescription::new("Engineer").show(ui);
//!         });
//!         ItemActions::new().show(ui, |ui| {
//!             Button::new("Follow").size(ButtonSize::Sm).show(ui);
//!         });
//!     });
//! });
//! ```

use crate::components::avatar::{cover_uv, resolve_texture};
use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{
    Color32, FontId, ImageSource, InnerResponse, Margin, Pos2, Rect, Sense, Stroke, Ui, Vec2,
};

/// Title line-height multiplier (`leading-snug`).
const TITLE_LINE_HEIGHT: f32 = 1.375;
/// Description line-height multiplier (`leading-normal`).
const DESCRIPTION_LINE_HEIGHT: f32 = 1.5;

/// Item variants, mirroring the source `itemVariants.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemVariant {
    /// `bg-transparent border-transparent`.
    #[default]
    Default,
    /// `border-border`.
    Outline,
    /// `bg-muted/50 border-transparent`.
    Muted,
}

/// Item sizes, mirroring the source `itemVariants.size` (plus the nova
/// `xs`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ItemSize {
    /// `gap-4 p-4` (nova `gap-2.5 px-3 py-2.5`).
    #[default]
    Default,
    /// `gap-2.5 px-4 py-3` (nova same as default).
    Sm,
    /// Nova-only `gap-2 px-2.5 py-2`.
    Xs,
}

/// Geometry `(pad_x, pad_y, gap)` per style and size.
pub(crate) fn item_geometry(style: Style, size: ItemSize) -> (f32, f32, f32) {
    match style {
        Style::Nova => match size {
            ItemSize::Xs => (10.0, 8.0, 8.0),
            _ => (12.0, 10.0, 10.0),
        },
        _ => match size {
            ItemSize::Default => (16.0, 16.0, 16.0),
            ItemSize::Sm => (16.0, 12.0, 10.0),
            ItemSize::Xs => (12.0, 10.0, 8.0),
        },
    }
}

/// The gap an [`ItemGroup`] uses between items (nova `gap-4` / `gap-2.5` /
/// `gap-2` by size). The base recipe's group is a plain `flex flex-col`
/// with no gap, so non-nova styles return 0 (use [`ItemSeparator`] there).
pub(crate) fn group_gap(style: Style, size: ItemSize) -> f32 {
    match style {
        Style::Nova => match size {
            ItemSize::Xs => 8.0,
            ItemSize::Sm => 10.0,
            ItemSize::Default => 16.0,
        },
        _ => 0.0,
    }
}

/// Resolved item colors (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ItemColors {
    pub fill: Color32,
    pub stroke: Stroke,
}

pub(crate) fn resolve_item_colors(p: &Palette, variant: ItemVariant) -> ItemColors {
    match variant {
        ItemVariant::Default => ItemColors {
            fill: Color32::TRANSPARENT,
            stroke: Stroke::NONE,
        },
        ItemVariant::Outline => ItemColors {
            fill: Color32::TRANSPARENT,
            stroke: Stroke::new(1.0, p.border),
        },
        ItemVariant::Muted => ItemColors {
            fill: color_alpha(p.muted, 0.5),
            stroke: Stroke::NONE,
        },
    }
}

/// The icon media box side (`size-8`).
pub const ITEM_MEDIA_ICON_BOX: f32 = 32.0;
/// The image media side (`size-10`).
pub const ITEM_MEDIA_IMAGE_SIZE: f32 = 40.0;
/// The media icon glyph size (`size-4`).
pub const ITEM_MEDIA_ICON: f32 = 16.0;

/// Context for the item currently being laid out.
#[derive(Clone, Copy, Debug)]
struct ItemCtx {
    id: egui::Id,
    size: ItemSize,
}

fn item_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_item_stack")
}

fn push_item(ui: &Ui, ctx: ItemCtx) {
    let mut stack: Vec<ItemCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(item_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(item_stack_id(), stack));
}

fn pop_item(ui: &Ui) {
    let mut stack: Vec<ItemCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(item_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(item_stack_id(), stack));
}

fn current_item(ui: &Ui) -> Option<ItemCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<ItemCtx>>(item_stack_id()))
        .and_then(|s| s.last().copied())
}

/// Context for the header/footer row currently being laid out (so
/// [`ItemActions`] can right-align, like the source's `justify-between`).
#[derive(Clone, Copy, Debug)]
struct RowCtx {
    left: f32,
    top: f32,
    right: f32,
}

fn row_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_item_row_stack")
}

fn push_row(ui: &Ui, ctx: RowCtx) {
    let mut stack: Vec<RowCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(row_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(row_stack_id(), stack));
}

fn pop_row(ui: &Ui) {
    let mut stack: Vec<RowCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(row_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(row_stack_id(), stack));
}

fn current_row(ui: &Ui) -> Option<RowCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<RowCtx>>(row_stack_id()))
        .and_then(|s| s.last().copied())
}

/// Group membership for [`ItemGroup`] size inheritance.
#[derive(Clone, Copy, Debug)]
struct GroupCtx {
    size: ItemSize,
}

fn group_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_item_group_stack")
}

fn push_group(ui: &Ui, ctx: GroupCtx) {
    let mut stack: Vec<GroupCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(group_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx()
        .data_mut(|d| d.insert_temp(group_stack_id(), stack));
}

fn pop_group(ui: &Ui) {
    let mut stack: Vec<GroupCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(group_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(group_stack_id(), stack));
}

fn current_group(ui: &Ui) -> Option<GroupCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<GroupCtx>>(group_stack_id()))
        .and_then(|s| s.last().copied())
}

/// A shadcn item. See the [module docs](self) for an example.
#[derive(Clone, Debug, Default)]
pub struct Item {
    variant: ItemVariant,
    size: Option<ItemSize>,
    interactive: bool,
    id_salt: Option<String>,
}

impl Item {
    /// Creates a default item.
    pub fn new() -> Self {
        Self {
            variant: ItemVariant::default(),
            size: None,
            interactive: false,
            id_salt: None,
        }
    }

    /// Sets the visual variant (default: [`ItemVariant::Default`]).
    pub fn variant(mut self, variant: ItemVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the size (inherits the enclosing [`ItemGroup`] size when
    /// unset).
    pub fn size(mut self, size: ItemSize) -> Self {
        self.size = Some(size);
        self
    }

    /// Makes the item interactive (the source's `asChild` anchor rows):
    /// hovering fills it with `bg-accent/50` (nova `bg-muted`) and focusing
    /// paints the `focus-visible:ring-[3px] ring-ring/50 border-ring`
    /// ring. The returned response reports `clicked()`.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Sets a stable id salt (item children coordinate through it).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the item around its slots.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_item_colors(&p, self.variant);
        let size = self
            .size
            .or_else(|| current_group(ui).map(|g| g.size))
            .unwrap_or_default();
        let (pad_x, pad_y, gap) = item_geometry(theme.style, size);
        let radius = if theme.style == Style::Nova {
            theme.radius.lg_u8()
        } else {
            theme.radius.md_u8()
        };

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_item"),
        };

        push_item(ui, ItemCtx { id, size });

        // `[a]:hover:bg-accent/50` (nova `bg-muted`): the hover fill
        // *replaces* the base fill, read from the previous frame's response
        // (one-frame lag, standard egui hover styling).
        let hover_bg = if theme.style == Style::Nova {
            p.muted
        } else {
            color_alpha(p.accent, 0.5)
        };
        let hovered = self.interactive
            && ui
                .ctx()
                .read_response(id)
                .map(|r| r.hovered())
                .unwrap_or(false);
        let fill = if hovered { hover_bg } else { c.fill };

        let inner = egui::Frame::new()
            .fill(fill)
            .stroke(c.stroke)
            .corner_radius(egui::CornerRadius::same(radius))
            .inner_margin(Margin {
                left: pad_x as i8,
                right: pad_x as i8,
                top: pad_y as i8,
                bottom: pad_y as i8,
            })
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    content(ui)
                })
                .inner
            });

        pop_item(ui);

        let sense = if self.interactive {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.interact(inner.response.rect, id, sense);
        if ui.is_rect_visible(response.rect) && response.has_focus() {
            // `focus-visible:border-ring`: recolor the border, then the 3px
            // `ring-ring/50` outline.
            ui.painter().rect_stroke(
                response.rect,
                egui::CornerRadius::same(radius),
                Stroke::new(1.0, p.ring),
                egui::epaint::StrokeKind::Inside,
            );
            crate::utils::focus::paint_focus_ring(ui, &response, radius as f32, true);
        }

        InnerResponse::new(inner.inner, response)
    }
}

/// `ItemMedia` — the leading media slot. [`ItemMedia::icon`] draws the
/// `size-8 rounded-sm border bg-muted` icon box; [`ItemMedia::image`] the
/// `size-10 rounded-sm` image box.
#[derive(Clone, Debug)]
pub struct ItemMedia {
    icon: Option<Icon>,
    image: Option<ImageSource<'static>>,
    id_salt: Option<String>,
}

impl Default for ItemMedia {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemMedia {
    /// Creates an empty (transparent) media slot.
    pub fn new() -> Self {
        Self {
            icon: None,
            image: None,
            id_salt: None,
        }
    }

    /// The icon media variant: `size-8 rounded-sm border bg-muted` box with
    /// a `size-4` glyph.
    pub fn icon(icon: Icon) -> Self {
        Self {
            icon: Some(icon),
            image: None,
            id_salt: None,
        }
    }

    /// The image media variant: `size-10 rounded-sm` box, texture painted
    /// aspect-cover (URI sources degrade to a muted box until loaded).
    pub fn image(image: impl Into<ImageSource<'static>>) -> Self {
        Self {
            icon: None,
            image: Some(image.into()),
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the media slot.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // `rounded-sm` = 0.6 × --radius (no sm_u8 helper on RadiusTokens).
        let radius = theme.radius.sm().round() as u8;

        if let Some(icon) = self.icon {
            let (rect, response) =
                ui.allocate_exact_size(Vec2::splat(ITEM_MEDIA_ICON_BOX), Sense::hover());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                painter.rect_filled(rect, radius, p.muted);
                painter.rect_stroke(
                    rect,
                    radius,
                    Stroke::new(1.0, p.border),
                    egui::epaint::StrokeKind::Inside,
                );
                let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(ITEM_MEDIA_ICON));
                icon.paint(
                    painter,
                    icon_rect,
                    p.foreground,
                    2.0 * ITEM_MEDIA_ICON / 24.0,
                );
            }
            return response;
        }

        if let Some(source) = self.image.as_ref() {
            let size = match current_item(ui).map(|c| c.size).unwrap_or_default() {
                ItemSize::Xs => 24.0, // nova xs size-6
                ItemSize::Sm => 32.0, // nova sm size-8
                ItemSize::Default => ITEM_MEDIA_IMAGE_SIZE,
            };
            let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                painter.rect_filled(rect, radius, p.muted);
                if let Some(tex) = resolve_texture(ui, source, size) {
                    let uv = cover_uv(tex.size, rect.size());
                    painter.add(
                        egui::epaint::RectShape::filled(rect, radius, Color32::WHITE)
                            .with_texture(tex.id, uv),
                    );
                }
            }
            return response;
        }

        // Transparent media slot: no intrinsic size.
        ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1
    }
}

/// `ItemContent` — the flexible middle column (`flex-1 flex-col gap-1`).
/// It yields the width the item's [`ItemActions`] reserve (measured last
/// frame, self-correcting).
#[derive(Clone, Debug, Default)]
pub struct ItemContent {
    id_salt: Option<String>,
}

impl ItemContent {
    /// Creates an item content slot.
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
        // Reserve the width measured for trailing actions (two-frame).
        let reserved: f32 = current_item(ui)
            .map(|c| {
                ui.ctx()
                    .data_mut(|d| d.get_temp(c.id.with("actions_w")))
                    .unwrap_or(0.0)
            })
            .unwrap_or(0.0);
        let gap = current_item(ui)
            .map(|c| {
                let theme = ShadcnTheme::of_ui(ui);
                let size = c.size;
                item_geometry(theme.style, size).2
            })
            .unwrap_or(10.0);
        let w = (ui.available_width() - reserved - gap).max(0.0);
        let inner = ui.allocate_ui(Vec2::new(w, 0.0), |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0; // gap-1
                content(ui)
            })
            .inner
        });
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemTitle` — `text-sm font-medium leading-snug` (`flex w-fit
/// items-center gap-2`, so icons can be hosted next to the text via
/// [`ItemTitle::show_with`]).
#[derive(Clone, Debug)]
pub struct ItemTitle {
    text: String,
}

impl ItemTitle {
    /// Creates an item title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        self.show_with(ui, |_| {}).response
    }

    /// Renders the title with extra content beside the text (the source's
    /// `flex w-fit items-center gap-2` allows e.g. an icon or badge).
    pub fn show_with<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let size = crate::theme::tokens::type_scale::SM;
        let font = FontId::proportional(size);
        // `leading-snug` is set explicitly through the TextFormat.
        let format = egui::TextFormat {
            font_id: font,
            color: p.foreground,
            line_height: Some(size * TITLE_LINE_HEIGHT),
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = f32::INFINITY;
        let galley = ui.painter().layout_job(job);
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0; // gap-2
            let (rect, _) = ui.allocate_exact_size(galley.size(), Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter().galley(rect.min, galley, p.foreground);
            }
            content(ui)
        });
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemDescription` — `text-sm leading-normal text-muted-foreground`
/// (nova `group-data-[size=xs]/item:text-xs`), wraps.
#[derive(Clone, Debug)]
pub struct ItemDescription {
    text: String,
}

impl ItemDescription {
    /// Creates an item description.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // nova shrinks the description to `text-xs` inside xs items.
        let size = match current_item(ui).map(|c| c.size) {
            Some(ItemSize::Xs) if theme.style == Style::Nova => {
                crate::theme::tokens::type_scale::XS
            }
            _ => crate::theme::tokens::type_scale::SM,
        };
        let font = FontId::proportional(size);
        // `leading-normal` is set explicitly through the TextFormat.
        let format = egui::TextFormat {
            font_id: font,
            color: p.muted_foreground,
            line_height: Some(size * DESCRIPTION_LINE_HEIGHT),
            ..egui::TextFormat::default()
        };
        let mut job = egui::text::LayoutJob::default();
        job.append(&self.text, 0.0, format);
        job.wrap.max_width = ui.available_width().max(0.0);
        let galley = ui.painter().layout_job(job);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.muted_foreground);
        }
        response
    }
}

/// `ItemActions` — the trailing actions row (`gap-2`). Inside an [`Item`],
/// it reserves its width so [`ItemContent`] flexes around it; inside an
/// [`ItemHeader`]/[`ItemFooter`] it right-aligns (`justify-between`).
#[derive(Clone, Debug, Default)]
pub struct ItemActions {
    id_salt: Option<String>,
}

impl ItemActions {
    /// Creates an item actions slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the actions.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        if let Some(row) = current_row(ui) {
            // Right-align at the header/footer row's right edge.
            let region = Rect::from_min_max(
                Pos2::new(row.left, row.top),
                Pos2::new(row.right, row.top + 400.0),
            );
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(region)
                    .layout(egui::Layout::right_to_left(egui::Align::Min)),
            );
            let ret = content(&mut child);
            return InnerResponse::new(ret, child.response());
        }

        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0; // gap-2
            content(ui)
        });
        // Record the measured width so ItemContent can flex next frame.
        if let Some(ctx) = current_item(ui) {
            let w = inner.response.rect.width();
            ui.ctx()
                .data_mut(|d| d.insert_temp(ctx.id.with("actions_w"), w));
        }
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemHeader` — a full-width row above the main slots
/// (`items-center justify-between gap-2`).
#[derive(Clone, Debug, Default)]
pub struct ItemHeader {
    id_salt: Option<String>,
}

impl ItemHeader {
    /// Creates an item header row.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header row.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let cursor = ui.cursor();
        push_row(
            ui,
            RowCtx {
                left: cursor.left(),
                top: cursor.top(),
                right: cursor.left() + ui.available_width(),
            },
        );
        let inner = ui.horizontal(|ui| {
            ui.set_min_width(ui.available_width()); // basis-full
            ui.spacing_mut().item_spacing.x = 8.0;
            content(ui)
        });
        pop_row(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemFooter` — a full-width row below the main slots
/// (`items-center justify-between gap-2`).
#[derive(Clone, Debug, Default)]
pub struct ItemFooter {
    id_salt: Option<String>,
}

impl ItemFooter {
    /// Creates an item footer row.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the footer row.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let cursor = ui.cursor();
        push_row(
            ui,
            RowCtx {
                left: cursor.left(),
                top: cursor.top(),
                right: cursor.left() + ui.available_width(),
            },
        );
        let inner = ui.horizontal(|ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.x = 8.0;
            content(ui)
        });
        pop_row(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemGroup` — a vertical list of items with a per-size gap
/// (nova `gap-4` default, `gap-2.5` sm, `gap-2` xs).
#[derive(Clone, Debug, Default)]
pub struct ItemGroup {
    size: ItemSize,
    id_salt: Option<String>,
}

impl ItemGroup {
    /// Creates an item group.
    pub fn new() -> Self {
        Self {
            size: ItemSize::default(),
            id_salt: None,
        }
    }

    /// Sets the group size (items inherit it unless overridden).
    pub fn size(mut self, size: ItemSize) -> Self {
        self.size = size;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its items.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let g = group_gap(theme.style, self.size);
        push_group(ui, GroupCtx { size: self.size });
        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = g;
            content(ui)
        });
        pop_group(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `ItemSeparator` — a horizontal [`crate::components::separator::Separator`]
/// between items (nova `my-2`).
#[derive(Clone, Debug, Default)]
pub struct ItemSeparator {
    id_salt: Option<String>,
}

impl ItemSeparator {
    /// Creates an item separator.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the separator.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let nova = theme.style == Style::Nova;
        if nova {
            ui.add_space(8.0); // my-2
        }
        let mut sep = crate::components::separator::Separator::new();
        if let Some(salt) = self.id_salt.as_deref() {
            sep = sep.id_salt(salt);
        }
        let response = sep.show(ui);
        if nova {
            ui.add_space(8.0);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn geometry_matches_style_recipes() {
        // nova default/sm px-3 py-2.5 gap-2.5; xs px-2.5 py-2 gap-2.
        assert_eq!(
            item_geometry(Style::Nova, ItemSize::Default),
            (12.0, 10.0, 10.0)
        );
        assert_eq!(item_geometry(Style::Nova, ItemSize::Sm), (12.0, 10.0, 10.0));
        assert_eq!(item_geometry(Style::Nova, ItemSize::Xs), (10.0, 8.0, 8.0));
        // base default p-4 gap-4; sm px-4 py-3 gap-2.5.
        assert_eq!(
            item_geometry(Style::Vega, ItemSize::Default),
            (16.0, 16.0, 16.0)
        );
        assert_eq!(item_geometry(Style::Vega, ItemSize::Sm), (16.0, 12.0, 10.0));
        // group gaps: nova gap-4 / gap-2.5 / gap-2; the base group is a
        // plain flex column with no gap.
        assert_eq!(group_gap(Style::Nova, ItemSize::Default), 16.0);
        assert_eq!(group_gap(Style::Nova, ItemSize::Sm), 10.0);
        assert_eq!(group_gap(Style::Nova, ItemSize::Xs), 8.0);
        assert_eq!(group_gap(Style::Vega, ItemSize::Default), 0.0);
        assert_eq!(group_gap(Style::Vega, ItemSize::Sm), 0.0);
    }

    #[test]
    fn interactive_items_sense_clicks() {
        let ctx = egui::Context::default();
        let mut plain_senses_clicks = true;
        let mut interactive_senses_clicks = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let root = egui::Id::unique("item_interactive_test");
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                root,
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            Item::new().id_salt("plain_item").show(ui, |ui| {
                ItemTitle::new("Plain").show(ui);
            });
            Item::new()
                .interactive(true)
                .id_salt("click_item")
                .show(ui, |ui| {
                    ItemTitle::new("Clickable").show(ui);
                });
            let plain = ui.ctx().read_response(root.with("plain_item"));
            let clickable = ui.ctx().read_response(root.with("click_item"));
            if let Some(r) = plain {
                plain_senses_clicks = r.sense.senses_click();
            }
            if let Some(r) = clickable {
                interactive_senses_clicks = r.sense.senses_click();
            }
        });
        out.textures_delta.clear();
        assert!(!plain_senses_clicks);
        assert!(interactive_senses_clicks);
    }

    #[test]
    fn variants_resolve_colors() {
        let p = palette(Mode::Light);
        assert_eq!(
            resolve_item_colors(&p, ItemVariant::Default).fill,
            Color32::TRANSPARENT
        );
        assert_eq!(
            resolve_item_colors(&p, ItemVariant::Muted).fill,
            color_alpha(p.muted, 0.5)
        );
        let outline = resolve_item_colors(&p, ItemVariant::Outline);
        assert_eq!(outline.stroke.color, p.border);
        assert_eq!(outline.fill, Color32::TRANSPARENT);
    }

    #[test]
    fn media_constants_match_source() {
        assert_eq!(ITEM_MEDIA_ICON_BOX, 32.0); // size-8
        assert_eq!(ITEM_MEDIA_IMAGE_SIZE, 40.0); // size-10
        assert_eq!(ITEM_MEDIA_ICON, 16.0); // size-4
    }

    #[test]
    fn renders_items_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("item_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
                );
                ItemGroup::new().show(ui, |ui| {
                    Item::new().variant(ItemVariant::Outline).show(ui, |ui| {
                        ItemMedia::icon(Icon::User).show(ui);
                        ItemContent::new().show(ui, |ui| {
                            ItemTitle::new("Ada Lovelace").show(ui);
                            ItemDescription::new("Writes the first programs.").show(ui);
                        });
                        ItemActions::new().show(ui, |ui| {
                            ui.label("follow");
                        });
                    });
                    Item::new().variant(ItemVariant::Muted).show(ui, |ui| {
                        ItemContent::new().show(ui, |ui| {
                            ItemTitle::new("Plain muted").show(ui);
                        });
                    });
                    ItemSeparator::new().show(ui);
                    Item::new().show(ui, |ui| {
                        ItemHeader::new().show(ui, |ui| {
                            ItemTitle::new("Team settings").show(ui);
                            ItemActions::new().show(ui, |ui| {
                                ui.label("edit");
                            });
                        });
                        ItemContent::new().show(ui, |ui| {
                            ItemDescription::new("Invite members, manage roles.").show(ui);
                        });
                        ItemFooter::new().show(ui, |ui| {
                            ui.label("footer");
                        });
                    });
                });
            });
            out.textures_delta.clear();
        }
    }
}
