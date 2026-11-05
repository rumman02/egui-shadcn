//! Focus ring drawing, matching shadcn's `focus-visible:ring-[3px]
//! focus-visible:ring-ring/50 focus-visible:border-ring`.

use egui::{Color32, CornerRadius, Painter, Rect, Response, Stroke, Ui};

/// Paints the shadcn focus ring around `rect` when `response` is focused.
///
/// The ring is 3px of the theme `ring` color at 50% opacity, drawn just
/// outside the widget rect (CSS `outline`-style), with the widget border
/// recolored to the full-strength ring.
pub fn paint_focus_ring(ui: &Ui, response: &Response, radius: f32, focused: bool) {
    if !focused {
        return;
    }
    let theme = crate::theme::ShadcnTheme::of_ui(ui);
    let ring = crate::theme::color_alpha(theme.palette.ring, 0.5);
    let rect = response.rect.expand2(egui::vec2(1.0, 1.0));
    paint_ring(ui.painter(), rect, ring, 3.0, radius);
}

/// Paints a ring of `width` points inset/outset around `rect`.
pub fn paint_ring(painter: &Painter, rect: Rect, color: Color32, width: f32, radius: f32) {
    painter.rect_stroke(
        rect,
        CornerRadius::same(radius.round() as u8),
        Stroke::new(width, color),
        egui::epaint::StrokeKind::Outside,
    );
}

/// True when a widget should show its focus ring: it has keyboard focus, or
/// it was reached by keyboard navigation (`focus_visible` semantics).
pub fn is_focused(response: &Response) -> bool {
    response.has_focus()
}

/// Marks that a response should show the ring for this frame
/// (e.g. programmatic focus).
pub fn request_focus(ui: &Ui, response: &Response) {
    ui.memory_mut(|mem| mem.request_focus(response.id));
}
