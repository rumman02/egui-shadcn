//! shadcn `badge.tsx` — a small pill label for statuses and counts.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/badge.tsx`:
//! `rounded-full px-2 py-0.5 text-xs font-medium`, six variants, plus the
//! nova overrides (`.cn-badge*` in `style-nova.css`): fixed `h-5` height,
//! `gap-1` icon slots (`[&>svg]:size-3`), a tinted destructive variant
//! (`bg-destructive/10 text-destructive dark:bg-destructive/20`) and
//! unprefixed hovers on ghost (`hover:bg-muted hover:text-muted-foreground
//! dark:hover:bg-muted/50`) and link (`hover:underline`). The remaining nova
//! hovers are `[a]:hover`-prefixed (anchor-only) and correctly stay no-ops
//! for a static span.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Badge::new("Default").show(ui);
//! Badge::new("Secondary").variant(BadgeVariant::Secondary).show(ui);
//! Badge::new("Destructive").variant(BadgeVariant::Destructive).show(ui);
//! Badge::new("New").icon_start(Icon::Plus).show(ui);
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

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
    /// Draw the label underline (nova link variant on hover).
    underline: bool,
}

/// Resolves variant colors for the current style, palette and hover state.
///
/// Mirrors the base recipe (`bg-destructive text-white
/// dark:bg-destructive/60`) and the nova overrides: the tinted destructive
/// (`bg-destructive/10 text-destructive dark:bg-destructive/20`) and the
/// unprefixed ghost hover (`hover:bg-muted hover:text-muted-foreground
/// dark:hover:bg-muted/50`). The base link hover and all `[a]:hover` rules
/// are anchor-only, so they stay no-ops for a static span.
pub(crate) fn resolve_colors(
    variant: BadgeVariant,
    p: &Palette,
    dark: bool,
    hovered: bool,
    style: Style,
) -> Colors {
    let nova = style == Style::Nova;
    match variant {
        BadgeVariant::Default => Colors {
            fill: p.primary,
            fg: p.primary_foreground,
            stroke: Stroke::NONE,
            underline: false,
        },
        BadgeVariant::Secondary => Colors {
            fill: p.secondary,
            fg: p.secondary_foreground,
            stroke: Stroke::NONE,
            underline: false,
        },
        BadgeVariant::Destructive => {
            if nova {
                // `bg-destructive/10 text-destructive dark:bg-destructive/20`
                // (the `[a]:hover` tint bump is anchor-only).
                Colors {
                    fill: color_alpha(p.destructive, if dark { 0.2 } else { 0.1 }),
                    fg: p.destructive,
                    stroke: Stroke::NONE,
                    underline: false,
                }
            } else {
                Colors {
                    fill: if dark {
                        color_alpha(p.destructive, 0.6)
                    } else {
                        p.destructive
                    },
                    fg: Color32::WHITE,
                    stroke: Stroke::NONE,
                    underline: false,
                }
            }
        }
        BadgeVariant::Outline => Colors {
            fill: Color32::TRANSPARENT,
            fg: p.foreground,
            stroke: Stroke::new(1.0, p.border),
            underline: false,
        },
        BadgeVariant::Ghost => {
            let (fill, fg) = if nova && hovered {
                // Unprefixed `hover:bg-muted hover:text-muted-foreground
                // dark:hover:bg-muted/50`.
                (
                    if dark {
                        color_alpha(p.muted, 0.5)
                    } else {
                        p.muted
                    },
                    p.muted_foreground,
                )
            } else {
                (Color32::TRANSPARENT, p.foreground)
            };
            Colors {
                fill,
                fg,
                stroke: Stroke::NONE,
                underline: false,
            }
        }
        BadgeVariant::Link => Colors {
            fill: Color32::TRANSPARENT,
            fg: p.primary,
            stroke: Stroke::NONE,
            // Unprefixed nova `hover:underline`; the base `[a&]:hover` is
            // anchor-only.
            underline: hovered && nova,
        },
    }
}

/// A shadcn badge. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Badge {
    label: String,
    variant: BadgeVariant,
    icon_start: Option<Icon>,
    icon_end: Option<Icon>,
    disabled: bool,
    id_salt: Option<String>,
}

impl Badge {
    /// Creates a badge with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: BadgeVariant::default(),
            icon_start: None,
            icon_end: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Sets the visual variant (default: [`BadgeVariant::Default`]).
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows a 12px icon before the label (`[&>svg]:size-3` slot).
    pub fn icon_start(mut self, icon: Icon) -> Self {
        self.icon_start = Some(icon);
        self
    }

    /// Shows a 12px icon after the label (`[&>svg]:size-3` slot).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
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

    /// Renders the badge. Under nova, ghost and link badges respond to hover
    /// (`hover:bg-muted` / `hover:underline`).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();
        let nova = theme.style == Style::Nova;

        let text_size = crate::theme::tokens::type_scale::XS;
        let font = FontId::proportional(text_size);
        let galley = ui
            .painter()
            .layout_no_wrap(self.label.clone(), font.clone(), Color32::WHITE);

        // `[&>svg]:size-3` icons with `gap-1`.
        let icon_size = 12.0;
        let gap = crate::theme::tokens::spacing::XS;
        let has_start = self.icon_start.is_some();
        let has_end = self.icon_end.is_some();

        // px-2 py-0.5; nova `h-5` height with pl/pr-1.5 (6px) on the icon
        // sides (`has-data-[icon=inline-start]:pl-1.5`).
        let pad_x = crate::theme::tokens::spacing::SM; // px-2 = 8
        let pad_left = if nova && has_start { 6.0 } else { pad_x };
        let pad_right = if nova && has_end { 6.0 } else { pad_x };
        let icons_w = (has_start as i32 + has_end as i32) as f32 * icon_size;
        let n_children = has_start as i32 + has_end as i32 + (!self.label.is_empty()) as i32;
        let gaps = (n_children - 1).max(0) as f32 * gap;
        let width = pad_left + pad_right + icons_w + gaps + galley.size().x;
        let height = if nova {
            20.0 // h-5
        } else {
            galley.size().y.max(icon_size) + 2.0 * crate::theme::tokens::spacing::HALF
        };
        let desired = Vec2::new(width, height);

        let (rect, response) = match self.id_salt.as_deref() {
            Some(salt) => {
                let id = ui.scope_id().with(salt);
                let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
                (rect, ui.interact(rect, id, Sense::hover()))
            }
            None => ui.allocate_exact_size(desired, Sense::hover()),
        };

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() && !self.disabled;
            let mut c = resolve_colors(self.variant, &p, dark, hovered, theme.style);
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

            // Content: [icon] gap [label] gap [icon], centered.
            let center = rect.center();
            let mut cursor = center.x - (icons_w + gaps + galley.size().x) * 0.5;
            if let Some(icon) = self.icon_start {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, c.fg, 2.0 * icon_size / 24.0);
                cursor += icon_size + gap;
            }
            if !self.label.is_empty() {
                let text_center = Pos2::new(cursor + galley.size().x * 0.5, center.y);
                painter.text(
                    text_center,
                    Align2::CENTER_CENTER,
                    self.label.clone(),
                    font,
                    c.fg,
                );
                if c.underline {
                    // `underline-offset-4` approximation (button.rs pattern).
                    let y = text_center.y + text_size * 0.62 + 2.0;
                    painter.line_segment(
                        [Pos2::new(cursor, y), Pos2::new(cursor + galley.size().x, y)],
                        Stroke::new(1.0, c.fg),
                    );
                }
                cursor += galley.size().x + gap;
            }
            if let Some(icon) = self.icon_end {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, c.fg, 2.0 * icon_size / 24.0);
            }
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
        let c = resolve_colors(BadgeVariant::Default, &p, false, false, Style::Vega);
        assert_eq!(c.fill, p.primary);
        assert_eq!(c.fg, p.primary_foreground);
        assert_eq!(c.stroke.width, 0.0);
    }

    #[test]
    fn base_destructive_is_solid_and_tinted_in_dark_mode() {
        let p = palette(Mode::Light);
        let c = resolve_colors(BadgeVariant::Destructive, &p, false, false, Style::Vega);
        assert_eq!(c.fill, p.destructive);
        assert_eq!(c.fg, Color32::WHITE);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(BadgeVariant::Destructive, &dark, true, false, Style::Vega);
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.6));
    }

    #[test]
    fn nova_destructive_is_tinted() {
        // `bg-destructive/10 text-destructive dark:bg-destructive/20`.
        let p = palette(Mode::Light);
        let c = resolve_colors(BadgeVariant::Destructive, &p, false, false, Style::Nova);
        assert_eq!(c.fill, color_alpha(p.destructive, 0.1));
        assert_eq!(c.fg, p.destructive);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(BadgeVariant::Destructive, &dark, true, false, Style::Nova);
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.2));
        assert_eq!(d.fg, dark.destructive);
    }

    #[test]
    fn nova_ghost_hover_uses_muted() {
        let p = palette(Mode::Light);
        let rest = resolve_colors(BadgeVariant::Ghost, &p, false, false, Style::Nova);
        assert_eq!(rest.fill, Color32::TRANSPARENT);
        assert_eq!(rest.fg, p.foreground);
        let hov = resolve_colors(BadgeVariant::Ghost, &p, false, true, Style::Nova);
        assert_eq!(hov.fill, p.muted);
        assert_eq!(hov.fg, p.muted_foreground);
        // `dark:hover:bg-muted/50`
        let dark = palette(Mode::Dark);
        let hov_dark = resolve_colors(BadgeVariant::Ghost, &dark, true, true, Style::Nova);
        assert_eq!(hov_dark.fill, color_alpha(dark.muted, 0.5));
        // The base recipe's ghost hover is `[a&]:hover` (anchor-only): no-op.
        let base_hov = resolve_colors(BadgeVariant::Ghost, &p, false, true, Style::Vega);
        assert_eq!(base_hov.fill, Color32::TRANSPARENT);
    }

    #[test]
    fn link_underlines_on_nova_hover_only() {
        let p = palette(Mode::Light);
        let rest = resolve_colors(BadgeVariant::Link, &p, false, false, Style::Nova);
        assert!(!rest.underline);
        let hov = resolve_colors(BadgeVariant::Link, &p, false, true, Style::Nova);
        assert!(hov.underline);
        // Base link hover is anchor-only: no underline.
        let base_hov = resolve_colors(BadgeVariant::Link, &p, false, true, Style::Vega);
        assert!(!base_hov.underline);
    }

    #[test]
    fn outline_has_border_others_transparent() {
        let p = palette(Mode::Light);
        let o = resolve_colors(BadgeVariant::Outline, &p, false, false, Style::Vega);
        assert_eq!(o.stroke.color, p.border);
        assert_eq!(o.fill, Color32::TRANSPARENT);
        assert_eq!(o.fg, p.foreground);
        for v in [
            BadgeVariant::Default,
            BadgeVariant::Secondary,
            BadgeVariant::Ghost,
            BadgeVariant::Link,
        ] {
            assert_eq!(
                resolve_colors(v, &p, false, false, Style::Vega)
                    .stroke
                    .width,
                0.0
            );
        }
        assert_eq!(
            resolve_colors(BadgeVariant::Ghost, &p, false, false, Style::Vega).fg,
            p.foreground
        );
        assert_eq!(
            resolve_colors(BadgeVariant::Link, &p, false, false, Style::Vega).fg,
            p.primary
        );
    }

    #[test]
    fn builder_fields() {
        let b = Badge::new("x")
            .variant(BadgeVariant::Ghost)
            .icon_start(Icon::Plus)
            .icon_end(Icon::X)
            .disabled(true)
            .id_salt("b");
        assert_eq!(b.variant, BadgeVariant::Ghost);
        assert!(b.icon_start.is_some() && b.icon_end.is_some());
        assert!(b.disabled);
        assert_eq!(b.id_salt.as_deref(), Some("b"));
    }
}
