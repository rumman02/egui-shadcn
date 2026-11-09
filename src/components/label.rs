//! shadcn `label.tsx` — a form label, optionally wired to a control.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/label.tsx`:
//! `flex items-center gap-2 text-sm leading-none font-medium select-none`,
//! 50% opacity when disabled. The flex row's leading icon slot is exposed
//! via [`Label::icon`].
//!
//! Like the Radix `<label>`, clicking a label that is bound with
//! [`Label::for_id`] moves keyboard focus to the control with that id.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Label::new("Email").for_id(email_id).show(ui);
//! Label::new("Profile").icon(Icon::User).show(ui);
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{Align2, Color32, FontId, Id, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};

/// Label text color: `text-foreground`, 50% alpha when disabled
/// (`group-data-[disabled=true]:opacity-50`).
pub(crate) fn text_color(p: &Palette, disabled: bool) -> Color32 {
    if disabled {
        color_alpha(p.foreground, 0.5)
    } else {
        p.foreground
    }
}

/// A shadcn label. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Label {
    text: String,
    icon: Option<Icon>,
    disabled: bool,
    for_id: Option<Id>,
    id_salt: Option<String>,
}

impl Label {
    /// Creates a label with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            disabled: false,
            for_id: None,
            id_salt: None,
        }
    }

    /// Shows a 16px icon before the text (the flex row's `[&>svg]` slot,
    /// separated by `gap-2`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Binds the label to a control id: clicking focuses that widget,
    /// like Radix `htmlFor`.
    pub fn for_id(mut self, id: Id) -> Self {
        self.for_id = Some(id);
        self
    }

    /// Disables the label (`opacity-50`, no pointer interaction).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt (Label keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the label.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let text_size = crate::theme::tokens::type_scale::SM;
        let font = FontId::proportional(text_size);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), Color32::WHITE);

        // `leading-none` → the text line box is the font size.
        let line_h = text_size;
        let icon_size = crate::theme::tokens::spacing::LG; // [&>svg]:size-4
        let icon_gap = crate::theme::tokens::spacing::SM; // gap-2
        let has_icon = self.icon.is_some();
        let icon_w = if has_icon { icon_size } else { 0.0 };
        let icon_gap_w = if has_icon { icon_gap } else { 0.0 };
        let desired = Vec2::new(
            icon_w + icon_gap_w + galley.size().x,
            line_h.max(if has_icon { icon_size } else { 0.0 }),
        );

        // Clickable when it can focus a control (`label[for]` semantics);
        // otherwise the text is not interactive (`select-none`).
        let interactive = self.for_id.is_some() && !self.disabled;
        let sense = if interactive {
            Sense::click()
        } else {
            Sense::hover()
        };

        let (rect, response) = {
            // Reserve space without registering an auto-id focusable
            // response, so egui's spatial arrow-key navigation only sees the
            // explicit widget id (checkbox.rs pattern).
            let (_, rect) = ui.allocate_space(desired);
            let id = match self.id_salt.as_deref() {
                Some(salt) => ui.scope_id().with(salt),
                None => ui.auto_id_with("shadcn_label"),
            };
            (rect, ui.interact(rect, id, sense))
        };

        if interactive && response.clicked() {
            if let Some(target) = self.for_id {
                ui.memory_mut(|mem| mem.request_focus(target));
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let color = text_color(&p, self.disabled);
            let center_y = rect.center().y;
            let mut cursor = rect.left();
            if let Some(icon) = self.icon {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center_y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, color, 2.0 * icon_size / 24.0);
                cursor += icon_size + icon_gap;
            }
            painter.text(
                Pos2::new(cursor + galley.size().x * 0.5, center_y),
                Align2::CENTER_CENTER,
                self.text,
                font,
                color,
            );
        }

        response
    }
}

impl Widget for Label {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    #[test]
    fn disabled_label_is_half_alpha() {
        let p = Preset::default().build(Mode::Light).palette;
        assert_eq!(text_color(&p, false), p.foreground);
        assert_eq!(text_color(&p, true), color_alpha(p.foreground, 0.5));
    }

    #[test]
    fn builder_fields() {
        let label = Label::new("Email")
            .icon(Icon::User)
            .for_id(Id::unique("email"))
            .disabled(true)
            .id_salt("email_label");
        assert_eq!(label.text, "Email");
        assert!(label.icon.is_some());
        assert!(label.disabled);
        assert_eq!(label.for_id, Some(Id::unique("email")));
        assert_eq!(label.id_salt.as_deref(), Some("email_label"));
    }
}
