//! shadcn `scroll-area.tsx` — a themed scrolling container.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/scroll-area.tsx`: a
//! `ScrollArea` root with a viewport and a `ScrollBar` whose thumb is
//! `rounded-full bg-border` and whose track is `w-2.5` (10px) with `p-px`
//! padding.
//!
//! The scrollbar look is applied through a *scoped* egui style
//! (`ScrollStyle::solid` + themed handle color), so the global style is
//! never permanently mutated — nested `egui::ScrollArea`s created inside
//! the closure inherit it.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ScrollArea::new().height(200.0).show(ui, |ui| {
//!     for i in 0..50 {
//!         ui.label(format!("Row {i}"));
//!     }
//! });
//! ```

use crate::theme::ShadcnTheme;
use egui::{InnerResponse, Ui};

/// The scrollbar track width (`w-2.5`).
pub const SCROLLBAR_WIDTH: f32 = 10.0;

/// A shadcn scroll area. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct ScrollArea {
    vertical: bool,
    horizontal: bool,
    height: Option<f32>,
    width: Option<f32>,
    id_salt: Option<String>,
}

impl Default for ScrollArea {
    fn default() -> Self {
        Self::new()
    }
}


impl ScrollArea {
    /// Creates a vertical scroll area (the source default orientation).
    pub fn new() -> Self {
        Self {
            vertical: true,
            horizontal: false,
            height: None,
            width: None,
            id_salt: None,
        }
    }

    /// Enables/disables the vertical scrollbar.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Enables the horizontal scrollbar (`orientation="horizontal"`).
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Caps the viewport height (default: available height).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Fixes the viewport width (default: available width).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a stable id salt so scroll position persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Builds the scoped scrollbar style.
    ///
    /// Track = `w-2.5` (10px) with `p-px` margins, always-visible solid
    /// thumb. The thumb color is applied separately through the scoped
    /// `visuals.extreme_bg_color` (what egui's solid handle paints with).
    pub(crate) fn themed_scroll_style() -> egui::style::ScrollStyle {
        let mut s = egui::style::ScrollStyle::solid();
        s.bar_width = SCROLLBAR_WIDTH; // w-2.5
        s.bar_inner_margin = 1.0; // p-px
        s.bar_outer_margin = 1.0;
        s.handle_min_length = 12.0;
        // Solid bars are always opaque; keep the handle visible.
        s.dormant_handle_opacity = 1.0;
        s.active_handle_opacity = 1.0;
        s.interact_handle_opacity = 1.0;
        s
    }

    /// Renders the scroll area around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let outer = ui.scope(|ui| {
            // Scoped style: only scroll areas created inside inherit it.
            ui.style_mut().spacing.scroll = Self::themed_scroll_style();
            // The solid handle color comes from `extreme_bg_color`; scope it
            // to the scrollbar token (`bg-border`).
            ui.style_mut().visuals.extreme_bg_color = p.border;

            let mut area = egui::ScrollArea::new([self.vertical, self.horizontal])
                .auto_shrink([false, false]);
            if let Some(h) = self.height {
                area = area.max_height(h);
            }
            if let Some(salt) = self.id_salt.as_deref() {
                area = area.id_salt(ui.scope_id().with(salt));
            }
            let inner = area.show(ui, |ui| {
                if let Some(w) = self.width {
                    ui.set_min_width(w);
                    ui.set_max_width(w);
                }
                content(ui)
            });
            // egui 0.37's ScrollAreaOutput has no Response; synthesize one
            // over the visible inner rect.
            let response = ui.allocate_rect(inner.inner_rect, egui::Sense::hover());
            InnerResponse::new(inner.inner, response)
        });
        outer.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect, Vec2};

    #[test]
    fn scrollbar_style_matches_source() {
        let s = ScrollArea::themed_scroll_style();
        assert_eq!(s.bar_width, 10.0); // w-2.5
        assert_eq!(s.bar_inner_margin, 1.0); // p-px
        assert!(!s.floating); // solid, always allocates space
        assert_eq!(s.dormant_handle_opacity, 1.0);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("scroll_area_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                ScrollArea::new()
                    .height(120.0)
                    .id_salt("render_scroll")
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                    });
                ScrollArea::new()
                    .horizontal(true)
                    .vertical(false)
                    .height(60.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for i in 0..30 {
                                ui.label(format!("Item {i}"));
                            }
                        });
                    });
            });
            out.textures_delta.clear();
        }
    }
}
