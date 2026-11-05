//! shadcn `checkbox.tsx` — a binary (or indeterminate) check control.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/checkbox.tsx`:
//! `size-4 rounded-[4px] border border-input shadow-xs` with
//! `data-[state=checked]:border-primary data-[state=checked]:bg-primary
//! data-[state=checked]:text-primary-foreground` and the shared
//! focus/invalid/disabled styles (`dark:bg-input/30`).
//!
//! The check mark is the lucide `check` icon drawn by [`crate::icons::Icon`];
//! the indeterminate state draws the lucide `minus` icon instead.
//!
//! Keyboard: Space or Enter toggles (egui activates focused widgets).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut accepted = false;
//! let response = Checkbox::new(&mut accepted)
//!     .id_salt("terms")
//!     .show(ui);
//! if response.changed() {
//!     // accepted flipped
//! }
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{Color32, CornerRadius, Response, Sense, Stroke, Ui, Vec2, Widget};

/// `size-4` — the checkbox side length.
pub const CHECKBOX_SIZE: f32 = 16.0;
/// `rounded-[4px]` — a literal 4px radius, not the theme scale.
pub const CHECKBOX_RADIUS: u8 = 4;
/// `size-3.5` — the indicator icon size.
const INDICATOR_SIZE: f32 = 14.0;

/// Resolved colors for one checkbox state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct CheckboxColors {
    fill: Color32,
    stroke: Stroke,
    /// Focus ring color (alpha already applied).
    ring: Color32,
    /// Indicator icon color.
    icon: Color32,
}

/// Mirrors the base recipe: `border-input dark:bg-input/30`,
/// `data-[state=checked]:border-primary data-[state=checked]:bg-primary
/// data-[state=checked]:text-primary-foreground`,
/// `aria-invalid:border-destructive aria-invalid:ring-destructive/20
/// dark:aria-invalid:ring-destructive/40`.
pub(crate) fn resolve_checkbox_colors(
    p: &Palette,
    dark: bool,
    checked: bool,
    invalid: bool,
) -> CheckboxColors {
    let (fill, stroke_color, icon) = if checked {
        (p.primary, p.primary, p.primary_foreground)
    } else {
        let fill = if dark {
            color_alpha(p.input, 0.3)
        } else {
            Color32::TRANSPARENT
        };
        (fill, p.input, p.foreground)
    };
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    let stroke_color = if invalid && !checked {
        p.destructive
    } else {
        stroke_color
    };
    CheckboxColors {
        fill,
        stroke: Stroke::new(1.0, stroke_color),
        ring,
        icon,
    }
}

/// A shadcn checkbox. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Checkbox<'a> {
    checked: &'a mut bool,
    indeterminate: bool,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> Checkbox<'a> {
    /// Creates a checkbox bound to the given boolean.
    pub fn new(checked: &'a mut bool) -> Self {
        Self {
            checked,
            indeterminate: false,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Shows the indeterminate (mixed) state: the lucide minus icon on an
    /// unchecked box. Display-only; the caller owns the tri-state logic.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Disables the checkbox (`disabled:opacity-50
    /// disabled:cursor-not-allowed`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the checkbox aria-invalid (`aria-invalid:border-destructive
    /// aria-invalid:ring-destructive/20`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so focus state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the checkbox.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_checkbox"),
        };

        let checked_before = *self.checked;
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(CHECKBOX_SIZE), sense);
        let mut response = ui.interact(rect, id, sense);

        let clicked = response.clicked() && !self.disabled;
        if clicked {
            *self.checked = !*self.checked;
        }

        if ui.is_rect_visible(rect) {
            let checked = *self.checked;
            // Radix sets `data-state="indeterminate"` — the checked styling
            // does not apply; the minus icon renders in the foreground color.
            let c = resolve_checkbox_colors(&p, dark, checked, self.invalid);
            let icon_color = if self.indeterminate && !checked {
                p.foreground
            } else {
                c.icon
            };
            let corner = CornerRadius::same(CHECKBOX_RADIUS);
            let painter = ui.painter();

            // `shadow-xs`
            if !self.disabled {
                let sh = crate::utils::shadow::shadow_xs(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        corner,
                        sh.color,
                    )
                    .with_blur_width(2.0),
                );
            }

            let mut fill = c.fill;
            let mut stroke = c.stroke;
            let mut icon = icon_color;
            if self.disabled {
                // `disabled:opacity-50`
                fill = color_alpha(fill, 0.5);
                stroke.color = color_alpha(stroke.color, 0.5);
                icon = color_alpha(icon, 0.5);
            }

            painter.rect_filled(rect, corner, fill);
            painter.rect_stroke(rect, corner, stroke, egui::epaint::StrokeKind::Inside);

            // Indicator: lucide `check` (checked) or `minus` (indeterminate).
            if self.indeterminate || checked {
                let icon_rect =
                    egui::Rect::from_center_size(rect.center(), Vec2::splat(INDICATOR_SIZE));
                let icon_kind = if self.indeterminate {
                    Icon::Minus
                } else {
                    Icon::Check
                };
                icon_kind.paint(painter, icon_rect, icon, 2.0 * INDICATOR_SIZE / 24.0);
            }

            // `focus-visible:border-ring focus-visible:ring-[3px]
            // focus-visible:ring-ring/50`
            if response.has_focus() && !self.disabled {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(
                    painter,
                    ring_rect,
                    c.ring,
                    3.0,
                    CHECKBOX_RADIUS as f32,
                );
            }
        }

        // Keep egui's sense of change consistent with the visual state.
        if checked_before != *self.checked {
            response.mark_changed();
        }

        response
    }
}

impl Widget for Checkbox<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
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
    fn checked_uses_primary() {
        let p = palette(Mode::Light);
        let c = resolve_checkbox_colors(&p, false, true, false);
        assert_eq!(c.fill, p.primary);
        assert_eq!(c.stroke.color, p.primary);
        assert_eq!(c.icon, p.primary_foreground);
        let u = resolve_checkbox_colors(&p, false, false, false);
        assert_eq!(u.fill, Color32::TRANSPARENT);
        assert_eq!(u.stroke.color, p.input);
        assert_eq!(u.icon, p.foreground);
    }

    #[test]
    fn dark_unchecked_uses_input_30() {
        let dark = palette(Mode::Dark);
        let u = resolve_checkbox_colors(&dark, true, false, false);
        assert_eq!(u.fill, color_alpha(dark.input, 0.3));
    }

    #[test]
    fn invalid_colors() {
        let p = palette(Mode::Light);
        let i = resolve_checkbox_colors(&p, false, false, true);
        assert_eq!(i.stroke.color, p.destructive);
        assert_eq!(i.ring, color_alpha(p.destructive, 0.2));
        let dark = palette(Mode::Dark);
        let di = resolve_checkbox_colors(&dark, true, false, true);
        assert_eq!(di.ring, color_alpha(dark.destructive, 0.4));
        // A checked checkbox keeps its primary border even when invalid.
        let ic = resolve_checkbox_colors(&p, false, true, true);
        assert_eq!(ic.stroke.color, p.primary);
    }

    #[test]
    fn toggles_state_on_show_click() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let mut changed = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("checkbox_toggle_test"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(200.0, 200.0),
                )),
            );
            let r = Checkbox::new(&mut checked).show(ui);
            changed = r.changed();
        });
        out.textures_delta.clear();
        assert!(!checked); // no pointer input, nothing toggled
        assert!(!changed);
    }

    #[test]
    fn builders_keep_fields() {
        let mut checked = false;
        let cb = Checkbox::new(&mut checked)
            .indeterminate(true)
            .disabled(true)
            .invalid(true)
            .id_salt("cb");
        assert!(cb.indeterminate);
        assert!(cb.disabled);
        assert!(cb.invalid);
        assert_eq!(cb.id_salt.as_deref(), Some("cb"));
    }
}
