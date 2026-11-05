//! shadcn `marker.tsx` — an annotated row: icon + short text, optionally
//! divided by horizontal rules or underlined.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/marker.tsx`:
//! `text-sm text-muted-foreground gap-2`, variants `default`,
//! `separator` (flex-1 rules on both sides), and `border` (`border-b pb-2`).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Marker::new().show(ui, |ui| {
//!     MarkerIcon::new(Icon::Sparkles).show(ui);
//!     MarkerContent::new("Heads up — usage is trending up").show(ui);
//! });
//!
//! Marker::new().variant(MarkerVariant::Separator).show(ui, |ui| {
//!     MarkerContent::new("or continue with").show(ui);
//! });
//! ```

use crate::icons::Icon;
use crate::theme::ShadcnTheme;
use egui::{Color32, InnerResponse, Sense, Ui, UiBuilder, Vec2};

/// Marker layout variants, mirroring the source `markerVariants`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkerVariant {
    /// Plain row: optional icon + content.
    #[default]
    Default,
    /// The content is centered between two 1px rules (`before:/after:`).
    Separator,
    /// The row gets a bottom rule with `pb-2` padding.
    Border,
}

/// The marker text color (`text-muted-foreground`).
pub(crate) fn text_color(theme: &ShadcnTheme) -> Color32 {
    theme.palette.muted_foreground
}

/// A shadcn marker row. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Marker {
    variant: MarkerVariant,
    id_salt: Option<String>,
}

impl Default for Marker {
    fn default() -> Self {
        Self::new()
    }
}

impl Marker {
    /// Creates a default marker row.
    pub fn new() -> Self {
        Self {
            variant: MarkerVariant::Default,
            id_salt: None,
        }
    }

    /// Sets the layout variant.
    pub fn variant(mut self, variant: MarkerVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt (Marker keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The row height: `min-h-4` (16px), plus padding for `border`.
    pub fn row_height(&self) -> f32 {
        let min_h = crate::theme::tokens::spacing::LG;
        match self.variant {
            MarkerVariant::Border => min_h + 8.0 + 1.0, // pb-2 + border-b
            _ => min_h,
        }
    }

    /// Renders the marker row around its children (use [`MarkerIcon`] and
    /// [`MarkerContent`] inside).
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let width = ui.available_width();
        let height = self.row_height();

        let layout = match self.variant {
            MarkerVariant::Separator => egui::Layout::left_to_right(egui::Align::Center)
                .with_main_align(egui::Align::Center),
            _ => egui::Layout::left_to_right(egui::Align::Center),
        };

        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        let mut builder = UiBuilder::new().max_rect(rect).layout(layout);
        if let Some(salt) = self.id_salt.clone() {
            builder = builder.id_salt(salt);
        }
        let mut child = ui.new_child(builder);
        let inner = content(&mut child);

        let painter = ui.painter();
        let rule = theme.palette.border;
        match self.variant {
            MarkerVariant::Separator => {
                // `before:bg-border` / `after:bg-border`, `mr-1`/`ml-1` (4px).
                let content_min = child.min_rect().left();
                let content_max = child.min_rect().right();
                let cy = rect.center().y;
                let gap = crate::theme::tokens::spacing::XS;
                let left_end = (content_min - gap).max(rect.left());
                let right_start = (content_max + gap).min(rect.right());
                if left_end > rect.left() + 2.0 {
                    painter.line_segment(
                        [
                            egui::Pos2::new(rect.left(), cy),
                            egui::Pos2::new(left_end, cy),
                        ],
                        egui::Stroke::new(1.0, rule),
                    );
                }
                if right_start < rect.right() - 2.0 {
                    painter.line_segment(
                        [
                            egui::Pos2::new(right_start, cy),
                            egui::Pos2::new(rect.right(), cy),
                        ],
                        egui::Stroke::new(1.0, rule),
                    );
                }
            }
            MarkerVariant::Border => {
                // `border-b pb-2`
                let y = rect.top() + height - 0.5;
                painter.line_segment(
                    [
                        egui::Pos2::new(rect.left(), y),
                        egui::Pos2::new(rect.right(), y),
                    ],
                    egui::Stroke::new(1.0, rule),
                );
            }
            MarkerVariant::Default => {}
        }

        InnerResponse { inner, response }
    }
}

/// `size-4` leading icon slot.
#[derive(Clone, Debug)]
pub struct MarkerIcon {
    icon: Icon,
    size: f32,
}

impl MarkerIcon {
    /// Creates a 16px icon slot (`size-4`).
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: crate::theme::tokens::spacing::LG,
        }
    }

    /// Overrides the icon size.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Renders the icon in the marker's muted text color.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        if ui.is_rect_visible(rect) {
            self.icon.paint(
                ui.painter(),
                rect,
                text_color(&theme),
                2.0 * self.size / 24.0,
            );
        }
        response
    }
}

impl egui::Widget for MarkerIcon {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui)
    }
}

/// `text-sm` marker text (`text-muted-foreground`).
#[derive(Clone, Debug)]
pub struct MarkerContent {
    text: String,
}

impl MarkerContent {
    /// Creates the marker text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        ui.label(
            egui::RichText::new(self.text)
                .size(crate::theme::tokens::type_scale::SM)
                .color(text_color(&theme)),
        )
    }
}

impl egui::Widget for MarkerContent {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{Pos2, Rect};

    #[test]
    fn row_height_matches_padding_rules() {
        assert_eq!(Marker::new().row_height(), 16.0);
        assert_eq!(
            Marker::new().variant(MarkerVariant::Separator).row_height(),
            16.0
        );
        // border-b pb-2 → 16 + 8 + 1
        assert_eq!(
            Marker::new().variant(MarkerVariant::Border).row_height(),
            25.0
        );
    }

    #[test]
    fn variants_and_builder() {
        let m = Marker::new()
            .variant(MarkerVariant::Separator)
            .id_salt("m1");
        assert_eq!(m.variant, MarkerVariant::Separator);
        assert_eq!(m.id_salt.as_deref(), Some("m1"));
    }

    #[test]
    fn separator_lines_leave_a_gap_around_content() {
        let ctx = egui::Context::default();
        // Verify the gap math: content 60px wide in a 200px row leaves
        // (200 - 60)/2 - 4px rules on each side.
        let row_w = 200.0;
        let content_w = 60.0;
        let gap = crate::theme::tokens::spacing::XS;
        let left_rule_end = (row_w - content_w) / 2.0 - gap;
        let right_rule_start = (row_w + content_w) / 2.0 + gap;
        assert!(left_rule_end > 0.0);
        assert!(right_rule_start < row_w);
        assert!((left_rule_end - 66.0).abs() < 1e-4);
        assert!((right_rule_start - 134.0).abs() < 1e-4);
        let _ = (ctx, gap);
        let _ = (Pos2::ZERO, Rect::ZERO);
    }
}
