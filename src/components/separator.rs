//! shadcn `separator.tsx` — a 1px visual divider.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/separator.tsx`:
//! `bg-border`, horizontal `h-px w-full`, vertical `w-px h-full`.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Separator::new().show(ui);                 // full-width horizontal line
//! Separator::new().vertical().length(24.0).show(ui); // 24pt tall vertical line
//! ```

use crate::theme::ShadcnTheme;
use egui::{Color32, Response, Sense, Ui, Vec2, Widget};

/// The divider direction, mirroring the source `orientation` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SeparatorOrientation {
    /// `h-px w-full`.
    #[default]
    Horizontal,
    /// `w-px h-full`.
    Vertical,
}

/// Resolves the separator fill (`bg-border`).
pub(crate) fn fill_color(palette: &crate::theme::Palette) -> Color32 {
    palette.border
}

/// A shadcn separator. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Separator {
    orientation: SeparatorOrientation,
    length: Option<f32>,
    decorative: bool,
    id_salt: Option<String>,
}

impl Default for Separator {
    fn default() -> Self {
        Self::new()
    }
}

impl Separator {
    /// Creates a horizontal separator.
    pub fn new() -> Self {
        Self {
            orientation: SeparatorOrientation::Horizontal,
            length: None,
            decorative: true,
            id_salt: None,
        }
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
    pub fn length(mut self, length: f32) -> Self {
        self.length = Some(length);
        self
    }

    /// Marks the separator as decorative (source default `true`; affects
    /// accessibility semantics in HTML, no visual difference).
    pub fn decorative(mut self, decorative: bool) -> Self {
        self.decorative = decorative;
        self
    }

    /// Sets a stable id salt (Separator keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The size this separator will occupy given the current ui.
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

    /// Renders the separator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let color = fill_color(&theme.palette);
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
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect};

    #[test]
    fn sizes_match_source() {
        let ctx = egui::Context::default();
        let ui = &mut egui::Ui::new(
            ctx,
            egui::Id::unique("sep_test"),
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 200.0))),
        );
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
    fn fill_is_the_border_token() {
        let p = Preset::default().build(Mode::Light).palette;
        assert_eq!(fill_color(&p), p.border);
    }

    #[test]
    fn orientation_builder() {
        let s = Separator::new().vertical(false);
        assert_eq!(s.orientation, SeparatorOrientation::Horizontal);
        let s = Separator::new().vertical(true);
        assert_eq!(s.orientation, SeparatorOrientation::Vertical);
        assert!(s.is_decorative());
        assert!(!Separator::new().decorative(false).is_decorative());
    }
}
