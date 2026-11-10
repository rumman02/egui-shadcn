//! shadcn `pagination.tsx` — page navigation links.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/pagination.tsx` (nova
//! overrides: content `gap-0.5`, ellipsis `size-8`). The nav centers its
//! row (`mx-auto w-full justify-center`). Links reuse the shadcn [`Button`]
//! exactly like the source does with `buttonVariants`: the active page uses
//! the `Outline` variant, the rest `Ghost` (page links default to
//! `size="icon"`). Previous/Next are ghost buttons with chevrons carrying
//! the source's padding overrides (`gap-1 px-2.5 sm:pl-2.5` / `sm:pr-2.5`;
//! nova adds `pl-1.5!` / `pr-1.5!`), rendered with the shared
//! [`crate::components::button`] metrics so heights follow the active style.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut page = 2usize;
//! Pagination::new().show(ui, |ui| {
//!     PaginationContent::new().show(ui, |ui| {
//!         PaginationItem::new().show(ui, |ui| {
//!             PaginationPrevious::new().disabled(page == 1).show(ui);
//!         });
//!         for p in 1..=3 {
//!             PaginationItem::new().show(ui, |ui| {
//!                 PaginationLink::new(p.to_string()).active(p == page).show(ui);
//!             });
//!         }
//!         PaginationItem::new().show(ui, |ui| {
//!             PaginationEllipsis::new().show(ui);
//!         });
//!         PaginationItem::new().show(ui, |ui| {
//!             PaginationNext::new().show(ui);
//!         });
//!     });
//! });
//! ```

use crate::components::button::{self, Button, ButtonSize, ButtonVariant};
use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme, Style};
use egui::{Align2, Color32, CornerRadius, FontId, Response, Sense, Ui, Vec2};

/// Ellipsis box size (nova `size-8` = 32px).
pub const PAGINATION_ELLIPSIS_SIZE: f32 = 32.0;

// ---------------------------------------------------------------------------
// Pagination (nav wrapper)
// ---------------------------------------------------------------------------

/// The `<nav aria-label="pagination">` wrapper: `mx-auto w-full
/// justify-center` — content is laid out centered.
#[derive(Clone, Debug, Default)]
pub struct Pagination {
    _private: (),
}

impl Pagination {
    /// Creates a pagination root.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the pagination with its content centered horizontally, like
    /// the source's `mx-auto w-full justify-center`.
    ///
    /// egui cannot measure the row up front, so the first frame flows
    /// normally while it is measured; every frame after positions the row
    /// with the previous frame's measured size (the same one-frame
    /// convergence as the tabs `flex-1` stretch).
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        // The id must be stable per call site: salt it with the cursor
        // position, which is constant while the surrounding layout is.
        let cursor = ui.cursor().min;
        let size_id = ui.scope_id().with(format!(
            "shadcn_pagination_center_{}_{}",
            cursor.x as i32, cursor.y as i32
        ));
        let prev = ui.ctx().data_mut(|d| d.get_temp::<Vec2>(size_id));
        let rect = prev.filter(|s| s.x > 0.0 && s.y > 0.0).map(|size| {
            let avail = ui.available_rect_before_wrap();
            egui::Align2::CENTER_TOP.align_size_within_rect(
                Vec2::new(size.x.min(avail.width()), size.y.max(1.0)),
                avail,
            )
        });
        match rect {
            Some(rect) => {
                let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
                content(&mut child);
                let measured = child.min_rect();
                ui.advance_cursor_after_rect(measured);
                child
                    .ctx()
                    .data_mut(|d| d.insert_temp(size_id, measured.size()));
            }
            None => {
                let inner = ui.scope(|child| content(child));
                ui.ctx()
                    .data_mut(|d| d.insert_temp(size_id, inner.response.rect.size()));
            }
        }
    }
}

// ---------------------------------------------------------------------------
// PaginationContent
// ---------------------------------------------------------------------------

/// The `<ul>` row: `flex flex-row items-center gap-1` (nova `gap-0.5`).
#[derive(Clone, Debug, Default)]
pub struct PaginationContent {
    gap: f32,
}

impl PaginationContent {
    /// Creates a pagination content row with the nova `gap-0.5`.
    pub fn new() -> Self {
        Self {
            gap: crate::theme::tokens::spacing::HALF,
        }
    }

    /// Overrides the gap between items.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Lays out the pagination items in a centered row.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = self.gap;
            content(ui);
        });
    }
}

// ---------------------------------------------------------------------------
// PaginationItem
// ---------------------------------------------------------------------------

/// A `<li>` slot — pure layout passthrough.
#[derive(Clone, Debug, Default)]
pub struct PaginationItem {
    _private: (),
}

impl PaginationItem {
    /// Creates a pagination item.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the item's control (a link, Previous/Next, or the ellipsis).
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        content(ui);
    }
}

// ---------------------------------------------------------------------------
// PaginationLink
// ---------------------------------------------------------------------------

/// A page link, styled through the shadcn button variants exactly like the
/// source: `buttonVariants({ variant: isActive ? "outline" : "ghost" })`,
/// default `size="icon"`.
#[derive(Debug)]
pub struct PaginationLink {
    label: String,
    active: bool,
    size: ButtonSize,
    disabled: bool,
    id_salt: Option<String>,
}

impl PaginationLink {
    /// Creates a page link with the given label (usually the page number).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            active: false,
            size: ButtonSize::Icon,
            disabled: false,
            id_salt: None,
        }
    }

    /// Marks this link as the current page (`aria-current="page"`; uses the
    /// `Outline` button variant).
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Overrides the button size (source default `size="icon"`).
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Disables the link (`disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt for focus persistence.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The button variant this link resolves to (source `buttonVariants`).
    pub(crate) fn resolved_variant(&self) -> ButtonVariant {
        if self.active {
            ButtonVariant::Outline
        } else {
            ButtonVariant::Ghost
        }
    }

    /// Renders the link.
    pub fn show(self, ui: &mut Ui) -> Response {
        let mut button = Button::new(self.label.clone())
            .variant(self.resolved_variant())
            .size(self.size);
        if let Some(salt) = self.id_salt.as_deref() {
            button = button.id_salt(salt);
        }
        button.show(ui)
    }
}

// ---------------------------------------------------------------------------
// PaginationPrevious / PaginationNext
// ---------------------------------------------------------------------------

/// Which side the chevron sits on for the pager buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PagerSide {
    /// `PaginationPrevious`: leading chevron, `sm:pl-2.5` (nova `pl-1.5!`).
    Previous,
    /// `PaginationNext`: trailing chevron, `sm:pr-2.5` (nova `pr-1.5!`).
    Next,
}

/// Height of the Previous/Next buttons (`size="default"`): nova `h-8`,
/// base recipe `h-9`. Same as [`crate::components::button::metrics`]'s
/// default-size height for the active style.
fn pager_height(style: Style) -> f32 {
    button::metrics::height(ButtonSize::Default, style)
}

/// `(left, right)` padding from the source `gap-1 px-2.5` plus the
/// `sm:pl-2.5` / `sm:pr-2.5` overrides (10px on the overridden side in the
/// base recipe) and the nova `pl-1.5!` / `pr-1.5!` (6px).
fn pager_padding(side: PagerSide, style: Style) -> (f32, f32) {
    let edge = if style == Style::Nova { 6.0 } else { 10.0 };
    match side {
        PagerSide::Previous => (edge, 10.0),
        PagerSide::Next => (10.0, edge),
    }
}

/// Icon-label gap: the source `gap-1` overrides the button's own gap in
/// every style.
const PAGER_GAP: f32 = crate::theme::tokens::spacing::XS;

/// Renders a ghost pager button with the source's size overrides. Mirrors
/// the [`Button`] ghost rendering but with the pagination-specific paddings,
/// which `Button`'s fixed per-size metrics cannot express.
fn pager_button(
    ui: &mut Ui,
    side: PagerSide,
    label: &str,
    icon: Icon,
    disabled: bool,
    id_salt: Option<&str>,
) -> Response {
    let theme = ShadcnTheme::of_ui(ui);
    let dark = theme.mode.is_dark();
    let style = theme.style;
    let nova = style == Style::Nova;
    let p = theme.palette;

    let height = pager_height(style);
    let (pad_left, pad_right) = pager_padding(side, style);
    let font = FontId::proportional(button::metrics::text_size(ButtonSize::Default, style));
    let icon_size = button::metrics::icon_size(ButtonSize::Default, style);
    let radius = button::metrics::radius(ButtonSize::Default, style, &theme.radius, false);
    let text_w = if label.is_empty() {
        0.0
    } else {
        ui.painter()
            .layout_no_wrap(label.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x
    };
    let desired = Vec2::new(
        pad_left + pad_right + icon_size + PAGER_GAP + text_w,
        height,
    );

    let (_, rect) = ui.allocate_space(desired);
    let id = match id_salt {
        Some(salt) => ui.scope_id().with(salt),
        None => ui.auto_id_with("shadcn_pager_button"),
    };
    let response = if disabled {
        // Disabled at the egui level too, like `Button` (no hover, clicks or
        // focus); painting below keeps the explicit `disabled:opacity-50`.
        let child = ui.new_child(egui::UiBuilder::new().disabled());
        child.interact(rect, id, Sense::click())
    } else {
        ui.interact(rect, id, Sense::click())
    };

    if ui.is_rect_visible(rect) {
        // Ghost variant colors (mirrors the `Button` Ghost arm).
        let hovered = response.hovered() && !disabled;
        let mut fill = if hovered {
            if nova {
                // `hover:bg-muted dark:hover:bg-muted/50`
                if dark {
                    color_alpha(p.muted, 0.5)
                } else {
                    p.muted
                }
            } else if dark {
                color_alpha(p.accent, 0.5)
            } else {
                p.accent
            }
        } else {
            Color32::TRANSPARENT
        };
        let mut fg = if hovered && !nova {
            p.accent_foreground
        } else {
            p.foreground
        };
        if disabled {
            fill = color_alpha(fill, 0.5);
            fg = color_alpha(fg, 0.5);
        }

        let painter = ui.painter();
        // `active:translate-y-px` (nova): the whole button shifts 1px down
        // while pressed.
        let mut draw_rect = rect;
        if nova && !disabled && response.is_pointer_button_down_on() {
            draw_rect.min.y += 1.0;
            draw_rect.max.y += 1.0;
        }
        painter.rect_filled(draw_rect, CornerRadius::same(radius), fill);

        // Content: [icon] gap [label] (Previous) or [label] gap [icon]
        // (Next), centered.
        let center = draw_rect.center();
        let total_w = icon_size
            + if label.is_empty() {
                0.0
            } else {
                PAGER_GAP + text_w
            };
        let mut cursor_x = center.x - total_w * 0.5;
        if side == PagerSide::Previous {
            let r = egui::Rect::from_center_size(
                egui::Pos2::new(cursor_x + icon_size * 0.5, center.y),
                Vec2::splat(icon_size),
            );
            icon.paint(painter, r, fg, 2.0 * icon_size / 24.0);
            cursor_x += icon_size + PAGER_GAP;
        }
        if !label.is_empty() {
            painter.text(
                egui::Pos2::new(cursor_x + text_w * 0.5, center.y),
                Align2::CENTER_CENTER,
                label,
                font,
                fg,
            );
            cursor_x += text_w + PAGER_GAP;
        }
        if side == PagerSide::Next {
            let r = egui::Rect::from_center_size(
                egui::Pos2::new(cursor_x + icon_size * 0.5, center.y),
                Vec2::splat(icon_size),
            );
            icon.paint(painter, r, fg, 2.0 * icon_size / 24.0);
        }

        // `focus-visible:ring-[3px] focus-visible:ring-ring/50`.
        if response.has_focus() && !disabled {
            crate::utils::focus::paint_ring(
                painter,
                rect.expand2(egui::vec2(1.0, 1.0)),
                color_alpha(p.ring, 0.5),
                3.0,
                radius as f32,
            );
        }
    }

    response
}

/// "Previous" ghost button with a leading chevron: `size="default"` +
/// `gap-1 px-2.5 sm:pl-2.5` (nova `pl-1.5!`), so it is 32px tall under nova
/// with a 6px left / 10px right padding and a 4px icon-label gap.
#[derive(Debug)]
pub struct PaginationPrevious {
    label: String,
    disabled: bool,
    id_salt: Option<String>,
}

impl PaginationPrevious {
    /// Creates the "Previous" button.
    pub fn new() -> Self {
        Self {
            label: "Previous".to_owned(),
            disabled: false,
            id_salt: None,
        }
    }

    /// Overrides the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Disables the button (e.g. on the first page).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the button.
    pub fn show(self, ui: &mut Ui) -> Response {
        pager_button(
            ui,
            PagerSide::Previous,
            &self.label,
            Icon::ChevronLeft,
            self.disabled,
            self.id_salt.as_deref(),
        )
    }
}

impl Default for PaginationPrevious {
    fn default() -> Self {
        Self::new()
    }
}

/// "Next" ghost button with a trailing chevron: `size="default"` + `gap-1
/// px-2.5 sm:pr-2.5` (nova `pr-1.5!`), mirroring [`PaginationPrevious`].
#[derive(Debug)]
pub struct PaginationNext {
    label: String,
    disabled: bool,
    id_salt: Option<String>,
}

impl PaginationNext {
    /// Creates the "Next" button.
    pub fn new() -> Self {
        Self {
            label: "Next".to_owned(),
            disabled: false,
            id_salt: None,
        }
    }

    /// Overrides the label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Disables the button (e.g. on the last page).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the button.
    pub fn show(self, ui: &mut Ui) -> Response {
        pager_button(
            ui,
            PagerSide::Next,
            &self.label,
            Icon::ChevronRight,
            self.disabled,
            self.id_salt.as_deref(),
        )
    }
}

impl Default for PaginationNext {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// PaginationEllipsis
// ---------------------------------------------------------------------------

/// The "more pages" slot: `MoreHorizontal` centered in a `size-8` box
/// (nova; the base style uses `size-9`).
#[derive(Clone, Debug, Default)]
pub struct PaginationEllipsis {
    _private: (),
}

impl PaginationEllipsis {
    /// Creates the ellipsis slot.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Paints the ellipsis glyph. The source ellipsis `<span>` sets no text
    /// color, so the glyph inherits `foreground` (unlike the breadcrumb
    /// ellipsis, whose ancestors set `text-muted-foreground`).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let size = PAGINATION_ELLIPSIS_SIZE;
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
        if ui.is_rect_visible(rect) {
            let icon_size = crate::theme::tokens::spacing::LG; // `size-4` glyph
            let icon_rect = egui::Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
            Icon::MoreHorizontal.paint(
                ui.painter(),
                icon_rect,
                theme.palette.foreground,
                2.0 * icon_size / 24.0,
            );
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Preset;
    use egui::{Pos2, Rect};

    #[test]
    fn active_link_uses_outline_and_others_ghost() {
        // Mirrors `buttonVariants({ variant: isActive ? "outline" : "ghost" })`.
        let active = PaginationLink::new("1").active(true);
        assert_eq!(active.resolved_variant(), ButtonVariant::Outline);
        let inactive = PaginationLink::new("1");
        assert_eq!(inactive.resolved_variant(), ButtonVariant::Ghost);
    }

    #[test]
    fn links_default_to_icon_size() {
        let link = PaginationLink::new("2");
        assert_eq!(link.size, ButtonSize::Icon);
        assert_eq!(link.size.height(), 36.0); // base recipe `size-9`
                                              // Nova `.cn-button-size-icon { size-8 }` shrinks the square via the
                                              // style-aware button metrics.
        assert_eq!(button::metrics::height(ButtonSize::Icon, Style::Nova), 32.0);
        let wide = PaginationLink::new("…100").size(ButtonSize::Sm);
        assert_eq!(wide.size.height(), 32.0);
    }

    #[test]
    fn pager_metrics_match_source() {
        // `size="default"` height: nova `h-8`, base recipe `h-9`.
        assert_eq!(pager_height(Style::Nova), 32.0);
        assert_eq!(pager_height(Style::Vega), 36.0);
        // `gap-1 px-2.5 sm:pl-2.5` / `sm:pr-2.5` (10px) with the nova
        // `pl-1.5!` / `pr-1.5!` (6px) edge override.
        assert_eq!(pager_padding(PagerSide::Previous, Style::Nova), (6.0, 10.0));
        assert_eq!(pager_padding(PagerSide::Next, Style::Nova), (10.0, 6.0));
        assert_eq!(
            pager_padding(PagerSide::Previous, Style::Vega),
            (10.0, 10.0)
        );
        assert_eq!(pager_padding(PagerSide::Next, Style::Vega), (10.0, 10.0));
        // The icon-label gap is `gap-1` in every style.
        assert_eq!(PAGER_GAP, 4.0);
    }

    #[test]
    fn ellipsis_matches_nova_size() {
        assert_eq!(PAGINATION_ELLIPSIS_SIZE, 32.0);
    }

    #[test]
    fn builders_keep_fields() {
        let link = PaginationLink::new("3")
            .active(true)
            .disabled(true)
            .id_salt("p3");
        assert!(link.active);
        assert!(link.disabled);
        assert_eq!(link.id_salt.as_deref(), Some("p3"));

        let prev = PaginationPrevious::new()
            .label("Prev")
            .disabled(true)
            .id_salt("prev");
        assert_eq!(prev.label, "Prev");
        assert!(prev.disabled);
        let next = PaginationNext::new().label("Nästa");
        assert_eq!(next.label, "Nästa");
        assert!(!next.disabled);
    }

    #[test]
    fn pagination_renders_without_panic() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("pagination_test_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(400.0, 200.0),
                )),
            );
            Pagination::new().show(ui, |ui| {
                PaginationContent::new().show(ui, |ui| {
                    PaginationItem::new().show(ui, |ui| {
                        PaginationPrevious::new().disabled(true).show(ui);
                    });
                    PaginationItem::new().show(ui, |ui| {
                        PaginationLink::new("1").show(ui);
                    });
                    PaginationItem::new().show(ui, |ui| {
                        PaginationLink::new("2").active(true).show(ui);
                    });
                    PaginationItem::new().show(ui, |ui| {
                        PaginationLink::new("3").show(ui);
                    });
                    PaginationItem::new().show(ui, |ui| {
                        PaginationEllipsis::new().show(ui);
                    });
                    PaginationItem::new().show(ui, |ui| {
                        PaginationNext::new().show(ui);
                    });
                });
            });
        });
        out.textures_delta.clear();
    }

    #[test]
    fn content_row_is_centered() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let width = 400.0;
        let mut row = Rect::ZERO;
        // Frame 1 flows plainly while measuring the row; frame 2 centers it
        // with that measured size.
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("pagination_center_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 200.0))),
                );
                Pagination::new().show(ui, |ui| {
                    PaginationContent::new().show(ui, |ui| {
                        PaginationItem::new().show(ui, |ui| {
                            PaginationLink::new("1").show(ui);
                        });
                        PaginationItem::new().show(ui, |ui| {
                            PaginationLink::new("2").active(true).show(ui);
                        });
                        PaginationItem::new().show(ui, |ui| {
                            PaginationNext::new().show(ui);
                        });
                    });
                    row = ui.min_rect();
                });
            });
            out.textures_delta.clear();
        }
        assert!(row.left() > 1.0, "the row is no longer left-aligned");
        let left_gap = row.left();
        let right_gap = width - row.right();
        assert!(
            (left_gap - right_gap).abs() < 1.5,
            "the row is not centered: left gap {left_gap}, right gap {right_gap}"
        );
    }
}
