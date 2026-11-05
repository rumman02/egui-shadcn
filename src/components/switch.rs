//! shadcn `switch.tsx` — a two-state on/off control.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/switch.tsx`:
//! `data-[size=default]:h-[1.15rem] data-[size=default]:w-8` (sm: `h-3.5
//! w-6`) rounded-full track, `data-[state=checked]:bg-primary
//! data-[state=unchecked]:bg-input dark:data-[state=unchecked]:bg-input/80`,
//! a `size-4` (sm: `size-3`) thumb that translates
//! `translate-x-[calc(100%-2px)]` when checked, and the shared focus ring
//! and `disabled:opacity-50` states.
//!
//! The thumb animates with `ctx.animate_value_with_time` (the source uses a
//! CSS `transition-transform`).
//!
//! Keyboard: Space or Enter toggles (egui activates focused widgets).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut enabled = false;
//! Switch::new(&mut enabled)
//!     .id_salt("airplane")
//!     .show(ui);
//! ```

use crate::components::button::group;
use crate::components::button::{fuse_radius, GroupOrientation, GroupPosition};
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use crate::utils::animation;
use egui::{Color32, CornerRadius, Pos2, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Track and thumb geometry per size (`data-size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SwitchSize {
    /// `h-[1.15rem] w-8` track, `size-4` thumb.
    #[default]
    Default,
    /// `h-3.5 w-6` track, `size-3` thumb.
    Sm,
}

impl SwitchSize {
    /// Track width in points.
    pub fn track_width(self) -> f32 {
        match self {
            SwitchSize::Default => 32.0,
            SwitchSize::Sm => 24.0,
        }
    }

    /// Track height in points (`1.15rem` = 18.4px).
    pub fn track_height(self) -> f32 {
        match self {
            SwitchSize::Default => 18.4,
            SwitchSize::Sm => 14.0,
        }
    }

    /// Thumb side in points (`size-4` / `size-3`).
    pub fn thumb_size(self) -> f32 {
        match self {
            SwitchSize::Default => 16.0,
            SwitchSize::Sm => 12.0,
        }
    }

    /// Horizontal thumb offset when checked
    /// (`translate-x-[calc(100%-2px)]` = thumb width minus 2px).
    pub fn checked_offset(self) -> f32 {
        self.thumb_size() - 2.0
    }
}

/// Resolved colors for one switch state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct SwitchColors {
    track: Color32,
    thumb: Color32,
    /// Focus ring color (alpha already applied).
    ring: Color32,
}

/// Mirrors the base recipe: track `data-[state=checked]:bg-primary
/// data-[state=unchecked]:bg-input dark:data-[state=unchecked]:bg-input/80`;
/// thumb `bg-background dark:data-[state=checked]:bg-primary-foreground
/// dark:data-[state=unchecked]:bg-foreground`.
pub(crate) fn resolve_switch_colors(
    p: &Palette,
    dark: bool,
    checked: bool,
    invalid: bool,
) -> SwitchColors {
    let track = if checked {
        p.primary
    } else if dark {
        color_alpha(p.input, 0.8)
    } else {
        p.input
    };
    let thumb = if dark {
        if checked {
            p.primary_foreground
        } else {
            p.foreground
        }
    } else {
        p.background
    };
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    SwitchColors { track, thumb, ring }
}

/// A shadcn switch. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Switch<'a> {
    checked: &'a mut bool,
    size: SwitchSize,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> Switch<'a> {
    /// Creates a switch bound to the given boolean.
    pub fn new(checked: &'a mut bool) -> Self {
        Self {
            checked,
            size: SwitchSize::Default,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Sets the size (`data-size`, default).
    pub fn size(mut self, size: SwitchSize) -> Self {
        self.size = size;
        self
    }

    /// Disables the switch (`disabled:opacity-50 disabled:cursor-not-allowed`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the switch aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so focus and thumb animation persist per
    /// instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the switch.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let size = self.size;

        // Auto-detected position inside a ButtonGroup-like group.
        let grp = group::next_child_position(ui);
        let position = grp.map(|(_, pos)| pos).unwrap_or(GroupPosition::Only);
        let orientation = grp.map(|(o, _)| o).unwrap_or(GroupOrientation::Horizontal);

        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_switch"),
        };

        let desired = Vec2::new(
            size.track_width(),
            size.track_height().max(size.thumb_size()),
        );
        let (_, rect) = ui.allocate_space(desired);
        let response = ui.interact(rect, id, sense);

        if response.clicked() && !self.disabled {
            *self.checked = !*self.checked;
        }

        if ui.is_rect_visible(rect) {
            let checked = *self.checked;
            let mut c = resolve_switch_colors(&p, dark, checked, self.invalid);
            let mut stroke = if self.invalid {
                Stroke::new(1.0, p.destructive)
            } else {
                Stroke::NONE
            };
            if self.disabled {
                // `disabled:opacity-50`
                c.track = color_alpha(c.track, 0.5);
                c.thumb = color_alpha(c.thumb, 0.5);
                stroke.color = color_alpha(stroke.color, 0.5);
            }

            let radius = size.track_height() / 2.0;
            let corner = CornerRadius::same(radius.round() as u8);
            let corner = fuse_radius(corner, orientation, position);

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

            painter.rect_filled(rect, corner, c.track);
            if stroke.width > 0.0 {
                painter.rect_stroke(rect, corner, stroke, egui::epaint::StrokeKind::Inside);
            }

            // Thumb: vertically centered, animated horizontal offset
            // (`transition-transform`, 150ms like the shadcn default).
            let target_x = if checked { size.checked_offset() } else { 0.0 };
            let offset = animation::animate_value(
                ui.ctx(),
                id.with("thumb_x"),
                target_x,
                animation::Duration::Default,
            );
            let thumb_center = Pos2::new(
                rect.min.x + offset + size.thumb_size() / 2.0,
                rect.center().y,
            );
            painter.circle_filled(thumb_center, size.thumb_size() / 2.0, c.thumb);

            // `focus-visible:border-ring focus-visible:ring-[3px]
            // focus-visible:ring-ring/50`
            if response.has_focus() && !self.disabled {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(painter, ring_rect, c.ring, 3.0, radius);
            }
        }

        response
    }
}

impl Widget for Switch<'_> {
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
    fn size_geometry_matches_source() {
        // default: w-8 (32) h-[1.15rem] (18.4), thumb size-4 (16)
        assert_eq!(SwitchSize::Default.track_width(), 32.0);
        assert_eq!(SwitchSize::Default.track_height(), 18.4);
        assert_eq!(SwitchSize::Default.thumb_size(), 16.0);
        // checked offset = calc(100% - 2px) = 14
        assert_eq!(SwitchSize::Default.checked_offset(), 14.0);
        // sm: w-6 h-3.5, thumb size-3, offset 10
        assert_eq!(SwitchSize::Sm.track_width(), 24.0);
        assert_eq!(SwitchSize::Sm.track_height(), 14.0);
        assert_eq!(SwitchSize::Sm.thumb_size(), 12.0);
        assert_eq!(SwitchSize::Sm.checked_offset(), 10.0);
    }

    #[test]
    fn track_and_thumb_colors() {
        let p = palette(Mode::Light);
        let c = resolve_switch_colors(&p, false, false, false);
        assert_eq!(c.track, p.input);
        assert_eq!(c.thumb, p.background);
        let on = resolve_switch_colors(&p, false, true, false);
        assert_eq!(on.track, p.primary);
        assert_eq!(on.thumb, p.background);
        let dark = palette(Mode::Dark);
        let d = resolve_switch_colors(&dark, true, false, false);
        assert_eq!(d.track, color_alpha(dark.input, 0.8));
        assert_eq!(d.thumb, dark.foreground);
        let d_on = resolve_switch_colors(&dark, true, true, false);
        assert_eq!(d_on.track, dark.primary);
        assert_eq!(d_on.thumb, dark.primary_foreground);
    }

    #[test]
    fn invalid_ring_uses_destructive() {
        let p = palette(Mode::Light);
        let c = resolve_switch_colors(&p, false, false, true);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
        let dark = palette(Mode::Dark);
        let c = resolve_switch_colors(&dark, true, false, true);
        assert_eq!(c.ring, color_alpha(dark.destructive, 0.4));
    }

    #[test]
    fn builders_keep_fields() {
        let mut checked = false;
        let sw = Switch::new(&mut checked)
            .size(SwitchSize::Sm)
            .disabled(true)
            .invalid(true)
            .id_salt("sw");
        assert_eq!(sw.size, SwitchSize::Sm);
        assert!(sw.disabled);
        assert!(sw.invalid);
        assert_eq!(sw.id_salt.as_deref(), Some("sw"));
    }
}
