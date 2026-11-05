//! shadcn `pagination.tsx` — page navigation links.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/pagination.tsx` (nova
//! overrides: content `gap-0.5`, ellipsis `size-8`). Links reuse the shadcn
//! [`Button`] exactly like the source does with `buttonVariants`: the active
//! page uses the `Outline` variant, the rest `Ghost` (page links default to
//! `size="icon"`), and Previous/Next are ghost buttons with chevrons.
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

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::icons::Icon;
use crate::theme::ShadcnTheme;
use egui::{Response, Sense, Ui, Vec2};

/// Ellipsis box size (nova `size-8` = 32px).
pub const PAGINATION_ELLIPSIS_SIZE: f32 = 32.0;

// ---------------------------------------------------------------------------
// Pagination (nav wrapper)
// ---------------------------------------------------------------------------

/// The `<nav aria-label="pagination">` wrapper — pure layout scope.
#[derive(Clone, Debug, Default)]
pub struct Pagination {
    _private: (),
}

impl Pagination {
    /// Creates a pagination root.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the pagination.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        content(ui);
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

/// "Previous" ghost button with a leading chevron (`gap-1 px-2.5`; rendered
/// with the default Button padding).
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
        let mut button = Button::new(self.label.clone())
            .variant(ButtonVariant::Ghost)
            .icon_start(Icon::ChevronLeft);
        if self.disabled {
            button = button.disabled(true);
        }
        if let Some(salt) = self.id_salt.as_deref() {
            button = button.id_salt(salt);
        }
        button.show(ui)
    }
}

impl Default for PaginationPrevious {
    fn default() -> Self {
        Self::new()
    }
}

/// "Next" ghost button with a trailing chevron (`gap-1 px-2.5`; rendered
/// with the default Button padding).
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
        let mut button = Button::new(self.label.clone())
            .variant(ButtonVariant::Ghost)
            .icon_end(Icon::ChevronRight);
        if self.disabled {
            button = button.disabled(true);
        }
        if let Some(salt) = self.id_salt.as_deref() {
            button = button.id_salt(salt);
        }
        button.show(ui)
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

    /// Paints the ellipsis glyph.
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
                theme.palette.muted_foreground,
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
    use egui::Pos2;

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
        assert_eq!(link.size.height(), 36.0);
        let wide = PaginationLink::new("…100").size(ButtonSize::Sm);
        assert_eq!(wide.size.height(), 32.0);
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
}
