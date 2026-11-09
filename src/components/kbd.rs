//! shadcn `kbd.tsx` — a small keyboard-key chip.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/kbd.tsx`:
//! `h-5 min-w-5 rounded-sm bg-muted px-1 text-xs font-medium
//! text-muted-foreground`, with a [`KbdGroup`] wrapper that lays out
//! several keys with `gap-1`. The `[&_svg:not([class*='size-'])]:size-3`
//! slot is exposed as [`Kbd::icon`] (a 12px glyph separated by `gap-1`).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Kbd::new("K").show(ui);
//! Kbd::new("M").icon(Icon::ChevronLeft).show(ui);
//! KbdGroup::new().show(ui, |ui| {
//!     Kbd::new("⌘").show(ui);
//!     Kbd::new("⇧").show(ui);
//!     Kbd::new("P").show(ui);
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{Palette, ShadcnTheme};
use egui::{Align2, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};

/// The chip background/text colors (`bg-muted text-muted-foreground`).
pub(crate) fn colors(p: &Palette) -> (Color32, Color32) {
    (p.muted, p.muted_foreground)
}

/// Chip width for a text width: `px-1` padding, `min-w-5` floor.
pub(crate) fn chip_width(text_width: f32) -> f32 {
    (text_width + 2.0 * crate::theme::tokens::spacing::XS)
        .max(crate::theme::tokens::spacing::XS * 5.0)
}

/// A single keyboard key chip. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Kbd {
    label: String,
    icon: Option<Icon>,
    id_salt: Option<String>,
}

impl Kbd {
    /// Creates a key chip for the given key name.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            id_salt: None,
        }
    }

    /// Shows a 12px icon before the label (`[&_svg:not([class*='size-'])]:
    /// size-3` slot, separated by `gap-1`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets a stable id salt (Kbd keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the key chip.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let (bg, fg) = colors(&p);

        let text_size = crate::theme::tokens::type_scale::XS;
        let font = FontId::proportional(text_size);
        let galley = ui
            .painter()
            .layout_no_wrap(self.label.clone(), font.clone(), Color32::WHITE);

        // h-5 min-w-5 px-1 → 20px tall, at least 20px wide, 4px side padding.
        let height = crate::theme::tokens::spacing::XS * 5.0;
        let icon_size = 12.0; // [&_svg:not([class*='size-'])]:size-3
        let icon_gap = crate::theme::tokens::spacing::XS; // gap-1
        let has_icon = self.icon.is_some();
        let content_w = galley.size().x + if has_icon { icon_size + icon_gap } else { 0.0 };
        let width = chip_width(content_w);
        let (rect, response) = match self.id_salt.as_deref() {
            Some(salt) => {
                let id = ui.scope_id().with(salt);
                let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
                (rect, ui.interact(rect, id, Sense::hover()))
            }
            None => ui.allocate_exact_size(Vec2::new(width, height), Sense::hover()),
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // `rounded-sm`
            painter.rect_filled(
                rect,
                CornerRadius::same(theme.radius.sm().round() as u8),
                bg,
            );
            // Content: [icon] gap [label], centered.
            let center = rect.center();
            let mut cursor = center.x - content_w * 0.5;
            if let Some(icon) = self.icon {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, fg, 2.0 * icon_size / 24.0);
                cursor += icon_size + icon_gap;
            }
            painter.text(
                Pos2::new(cursor + galley.size().x * 0.5, center.y),
                Align2::CENTER_CENTER,
                self.label,
                font,
                fg,
            );
        }

        response
    }
}

impl Widget for Kbd {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Lays out several [`Kbd`] chips with `gap-1` spacing, like the source
/// `KbdGroup` (`inline-flex items-center gap-1`).
#[derive(Clone, Copy, Debug, Default)]
pub struct KbdGroup;

impl KbdGroup {
    /// Creates an empty group.
    pub fn new() -> Self {
        Self
    }

    /// Renders the group around its children.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.horizontal(|ui| {
            // `gap-1`
            ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::XS;
            content(ui)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    #[test]
    fn chip_uses_muted_tokens() {
        let p = Preset::default().build(Mode::Light).palette;
        let (bg, fg) = colors(&p);
        assert_eq!(bg, p.muted);
        assert_eq!(fg, p.muted_foreground);
    }

    #[test]
    fn kbd_builder_fields() {
        let k = Kbd::new("⌘").icon(Icon::ChevronRight).id_salt("meta");
        assert_eq!(k.label, "⌘");
        assert!(k.icon.is_some());
        assert_eq!(k.id_salt.as_deref(), Some("meta"));
    }

    #[test]
    fn group_is_a_unit_struct() {
        let _ = KbdGroup::new();
    }

    #[test]
    fn chip_width_respects_min_w_5() {
        // h-5 min-w-5 px-1: short keys are 20x20, longer labels get px-1.
        assert_eq!(chip_width(0.0), 20.0);
        assert_eq!(chip_width(10.0), 20.0);
        assert_eq!(chip_width(30.0), 38.0);
        // 20px height = 5 * gap-1
        assert_eq!(crate::theme::tokens::spacing::XS * 5.0, 20.0);
    }
}
