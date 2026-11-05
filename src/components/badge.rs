//! shadcn `badge.tsx` — a small pill label for statuses and counts.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/badge.tsx`:
//! `rounded-full px-2 py-0.5 text-xs font-medium`, six variants.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Badge::new("Default").show(ui);
//! Badge::new("Secondary").variant(BadgeVariant::Secondary).show(ui);
//! Badge::new("Destructive").variant(BadgeVariant::Destructive).show(ui);
//! ```

use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{Align2, Color32, FontId, Response, Sense, Stroke, Ui, Vec2, Widget};

/// Visual style variants, mirroring the source `badgeVariants`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// `bg-primary text-primary-foreground`
    #[default]
    Default,
    /// `bg-secondary text-secondary-foreground`
    Secondary,
    /// `bg-destructive text-white dark:bg-destructive/60`
    Destructive,
    /// `border-border text-foreground`
    Outline,
    /// Transparent, inherits the foreground color.
    Ghost,
    /// `text-primary underline-offset-4`
    Link,
}

/// Resolved colors for one badge state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Colors {
    fill: Color32,
    fg: Color32,
    stroke: Stroke,
}

/// Resolves variant colors for the current palette and mode.
pub(crate) fn resolve_colors(variant: BadgeVariant, p: &Palette, dark: bool) -> Colors {
    match variant {
        BadgeVariant::Default => Colors {
            fill: p.primary,
            fg: p.primary_foreground,
            stroke: Stroke::NONE,
        },
        BadgeVariant::Secondary => Colors {
            fill: p.secondary,
            fg: p.secondary_foreground,
            stroke: Stroke::NONE,
        },
        BadgeVariant::Destructive => Colors {
            fill: if dark {
                color_alpha(p.destructive, 0.6)
            } else {
                p.destructive
            },
            fg: Color32::WHITE,
            stroke: Stroke::NONE,
        },
        BadgeVariant::Outline => Colors {
            fill: Color32::TRANSPARENT,
            fg: p.foreground,
            stroke: Stroke::new(1.0, p.border),
        },
        BadgeVariant::Ghost => Colors {
            fill: Color32::TRANSPARENT,
            fg: p.foreground,
            stroke: Stroke::NONE,
        },
        BadgeVariant::Link => Colors {
            fill: Color32::TRANSPARENT,
            fg: p.primary,
            stroke: Stroke::NONE,
        },
    }
}

/// A shadcn badge. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Badge {
    label: String,
    variant: BadgeVariant,
    disabled: bool,
    id_salt: Option<String>,
}

impl Badge {
    /// Creates a badge with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::default(),
            disabled: false,
            id_salt: None,
        }
    }

    /// Sets the visual variant (default: [`BadgeVariant::Default`]).
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Renders at 50% opacity, like a disabled tag.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt (Badge keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the badge (hover is a no-op: a static span in the source).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();

        let text_size = crate::theme::tokens::type_scale::XS;
        let font = FontId::proportional(text_size);
        let galley = ui
            .painter()
            .layout_no_wrap(self.label.clone(), font.clone(), Color32::WHITE);
        // px-2 py-0.5 → 8px horizontal, 2px vertical padding.
        let desired = Vec2::new(
            galley.size().x + 2.0 * crate::theme::tokens::spacing::SM,
            galley.size().y + 2.0 * crate::theme::tokens::spacing::HALF,
        );

        let (rect, response) = match self.id_salt.as_deref() {
            Some(salt) => {
                let id = ui.scope_id().with(salt);
                let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
                (rect, ui.interact(rect, id, Sense::hover()))
            }
            None => ui.allocate_exact_size(desired, Sense::hover()),
        };

        if ui.is_rect_visible(rect) {
            let mut c = resolve_colors(self.variant, &p, dark);
            if self.disabled {
                c.fill = color_alpha(c.fill, 0.5);
                c.fg = color_alpha(c.fg, 0.5);
                c.stroke.color = color_alpha(c.stroke.color, 0.5);
            }
            let painter = ui.painter();
            // `rounded-full` — the badge is a pill.
            painter.rect_filled(rect, theme.radius.full_u8(), c.fill);
            if c.stroke.width > 0.0 {
                painter.rect_stroke(
                    rect,
                    theme.radius.full_u8(),
                    c.stroke,
                    egui::epaint::StrokeKind::Inside,
                );
            }
            painter.text(rect.center(), Align2::CENTER_CENTER, self.label, font, c.fg);
        }

        response
    }
}

impl Widget for Badge {
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
    fn default_uses_primary() {
        let p = palette(Mode::Light);
        let c = resolve_colors(BadgeVariant::Default, &p, false);
        assert_eq!(c.fill, p.primary);
        assert_eq!(c.fg, p.primary_foreground);
        assert_eq!(c.stroke.width, 0.0);
    }

    #[test]
    fn destructive_is_tinted_in_dark_mode() {
        let p = palette(Mode::Light);
        let c = resolve_colors(BadgeVariant::Destructive, &p, false);
        assert_eq!(c.fill, p.destructive);
        assert_eq!(c.fg, Color32::WHITE);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(BadgeVariant::Destructive, &dark, true);
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.6));
    }

    #[test]
    fn outline_has_border_others_transparent() {
        let p = palette(Mode::Light);
        let o = resolve_colors(BadgeVariant::Outline, &p, false);
        assert_eq!(o.stroke.color, p.border);
        assert_eq!(o.fill, Color32::TRANSPARENT);
        assert_eq!(o.fg, p.foreground);
        for v in [
            BadgeVariant::Default,
            BadgeVariant::Secondary,
            BadgeVariant::Ghost,
            BadgeVariant::Link,
        ] {
            assert_eq!(resolve_colors(v, &p, false).stroke.width, 0.0);
        }
        assert_eq!(
            resolve_colors(BadgeVariant::Ghost, &p, false).fg,
            p.foreground
        );
        assert_eq!(resolve_colors(BadgeVariant::Link, &p, false).fg, p.primary);
    }
}
