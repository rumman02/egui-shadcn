//! shadcn `button.tsx` — the primary interactive control.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/button.tsx`. Two recipes are
//! implemented and picked by [`crate::theme::ShadcnTheme::style`]:
//!
//! - **Base recipe** (`buttonVariants`, used by every non-nova style):
//!   `h-9 px-4`, `rounded-md`, `hover:bg-primary/90`, outline
//!   `border bg-background shadow-xs hover:bg-accent`, ghost
//!   `hover:bg-accent`, ...
//! - **Nova recipe** (`apps/v4/registry/styles/style-nova.css`,
//!   `.cn-button*`, the default style): `rounded-lg border border-transparent`
//!   with per-size radius caps and `h-8 px-2.5` default sizing, tinted
//!   destructive (`bg-destructive/10 text-destructive`), muted hovers for
//!   outline/ghost (`hover:bg-muted hover:text-foreground`), default hover
//!   `bg-primary/80`, secondary hover mixed with 5% foreground, and the
//!   `active:translate-y-px` pressed shift.
//!
//! Both recipes honor `focus-visible:border-ring`, `aria-invalid:*` states,
//! disabled opacity and button-group corner fusing.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Button::new("Save")
//!     .variant(ButtonVariant::Default)
//!     .size(ButtonSize::Default)
//!     .show(ui);
//!
//! if Button::new("Delete")
//!     .variant(ButtonVariant::Destructive)
//!     .icon_end(Icon::Trash)
//!     .show(ui)
//!     .clicked()
//! {
//!     // ...
//! }
//! ```

use crate::icons::Icon;
use crate::theme::{blend, color_alpha, Palette, ShadcnTheme, Style};
use crate::utils::focus;
use egui::{
    Align2, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget,
};

/// Visual style variants, mirroring the source `buttonVariants`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// `bg-primary text-primary-foreground hover:bg-primary/90`
    #[default]
    Default,
    /// `bg-destructive text-white hover:bg-destructive/90`
    Destructive,
    /// `border bg-background shadow-xs hover:bg-accent`
    Outline,
    /// `bg-secondary text-secondary-foreground hover:bg-secondary/80`
    Secondary,
    /// `hover:bg-accent hover:text-accent-foreground`
    Ghost,
    /// `text-primary underline-offset-4 hover:underline`
    Link,
}

/// Size presets, mirroring the source `buttonVariants.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// `h-9 px-4 py-2 gap-2` (36px tall).
    #[default]
    Default,
    /// `h-6 px-2 gap-1 text-xs` (24px tall).
    Xs,
    /// `h-8 px-3 gap-1.5` (32px tall).
    Sm,
    /// `h-10 px-6 gap-2` (40px tall).
    Lg,
    /// `size-9` icon-only square (36px).
    Icon,
    /// `size-6` icon-only square (24px).
    IconXs,
    /// `size-8` icon-only square (32px).
    IconSm,
    /// `size-10` icon-only square (40px).
    IconLg,
}

impl ButtonSize {
    /// Outer height in points (`h-9` etc.; for icon sizes the square side).
    pub fn height(self) -> f32 {
        match self {
            ButtonSize::Default | ButtonSize::Icon => crate::theme::tokens::spacing::CONTROL,
            ButtonSize::Xs | ButtonSize::IconXs => crate::theme::tokens::spacing::CONTROL_XS,
            ButtonSize::Sm | ButtonSize::IconSm => crate::theme::tokens::spacing::CONTROL_SM,
            ButtonSize::Lg | ButtonSize::IconLg => crate::theme::tokens::spacing::CONTROL_LG,
        }
    }

    /// Horizontal text padding in points (`px-4` etc.); 0 for icon sizes.
    pub fn padding_x(self) -> f32 {
        match self {
            ButtonSize::Default => crate::theme::tokens::spacing::LG,
            ButtonSize::Xs => crate::theme::tokens::spacing::SM,
            ButtonSize::Sm => crate::theme::tokens::spacing::MD,
            ButtonSize::Lg => crate::theme::tokens::spacing::XL,
            ButtonSize::Icon | ButtonSize::IconXs | ButtonSize::IconSm | ButtonSize::IconLg => 0.0,
        }
    }

    /// Gap between icon and label (`gap-2` etc.).
    pub fn gap(self) -> f32 {
        match self {
            ButtonSize::Xs => crate::theme::tokens::spacing::XS,
            ButtonSize::Sm => crate::theme::tokens::spacing::XS_HALF,
            _ => crate::theme::tokens::spacing::SM,
        }
    }

    /// Label size in points (`text-sm`, `text-xs`).
    pub fn text_size(self) -> f32 {
        match self {
            ButtonSize::Xs => crate::theme::tokens::type_scale::XS,
            _ => crate::theme::tokens::type_scale::SM,
        }
    }

    /// Inline icon size (`size-4`, `size-3` in `xs`).
    pub fn icon_size(self) -> f32 {
        match self {
            ButtonSize::Xs | ButtonSize::IconXs => crate::theme::tokens::spacing::MD,
            _ => crate::theme::tokens::spacing::LG,
        }
    }

    /// True for the square icon-only sizes.
    pub fn is_icon(self) -> bool {
        matches!(
            self,
            ButtonSize::Icon | ButtonSize::IconXs | ButtonSize::IconSm | ButtonSize::IconLg
        )
    }
}

/// Per-style size metrics: the [`ButtonSize`] methods hold the base recipe
/// (`buttonVariants`); these helpers branch on [`Style`] so the nova
/// overrides (`.cn-button-size-*`) apply wherever the active theme is known.
pub(crate) mod metrics {
    use super::ButtonSize;
    use crate::theme::{RadiusTokens, Style};

    /// Outer height (`h-9` etc.; nova `h-8` default, `h-7` sm, `h-9` lg).
    pub(crate) fn height(size: ButtonSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ButtonSize::Xs | ButtonSize::IconXs => 24.0, // h-6 / size-6
                ButtonSize::Sm | ButtonSize::IconSm => 28.0, // h-7 / size-7
                ButtonSize::Default | ButtonSize::Icon => 32.0, // h-8 / size-8
                ButtonSize::Lg | ButtonSize::IconLg => 36.0, // h-9 / size-9
            },
            _ => size.height(),
        }
    }

    /// Base-recipe padding when the button has any icon (`has-[>svg]:px-*`).
    fn base_icon_padding_x(size: ButtonSize) -> f32 {
        match size {
            ButtonSize::Default => 12.0, // has-[>svg]:px-3
            ButtonSize::Xs => 6.0,       // has-[>svg]:px-1.5
            ButtonSize::Sm => 10.0,      // has-[>svg]:px-2.5
            ButtonSize::Lg => 16.0,      // has-[>svg]:px-4
            _ => 0.0,
        }
    }

    /// Left padding. The base recipe shrinks `px-*` whenever the button has
    /// any icon; nova shrinks `pl-*` only for an inline-start icon.
    pub(crate) fn padding_left(
        size: ButtonSize,
        style: Style,
        icon_start: bool,
        icon_end: bool,
    ) -> f32 {
        match style {
            Style::Nova => match size {
                ButtonSize::Xs => {
                    if icon_start {
                        6.0 // pl-1.5
                    } else {
                        8.0 // px-2
                    }
                }
                ButtonSize::Sm => {
                    if icon_start {
                        6.0 // pl-1.5
                    } else {
                        10.0 // px-2.5
                    }
                }
                ButtonSize::Default | ButtonSize::Lg => {
                    if icon_start {
                        8.0 // pl-2
                    } else {
                        10.0 // px-2.5
                    }
                }
                _ => 0.0,
            },
            _ => {
                if icon_start || icon_end {
                    base_icon_padding_x(size)
                } else {
                    size.padding_x()
                }
            }
        }
    }

    /// Right padding (`pr-*` for an inline-end icon under nova; the mirrored
    /// base-recipe rule).
    pub(crate) fn padding_right(
        size: ButtonSize,
        style: Style,
        icon_start: bool,
        icon_end: bool,
    ) -> f32 {
        match style {
            Style::Nova => match size {
                ButtonSize::Xs => {
                    if icon_end {
                        6.0 // pr-1.5
                    } else {
                        8.0 // px-2
                    }
                }
                ButtonSize::Sm => {
                    if icon_end {
                        6.0 // pr-1.5
                    } else {
                        10.0 // px-2.5
                    }
                }
                ButtonSize::Default | ButtonSize::Lg => {
                    if icon_end {
                        8.0 // pr-2
                    } else {
                        10.0 // px-2.5
                    }
                }
                _ => 0.0,
            },
            _ => {
                if icon_start || icon_end {
                    base_icon_padding_x(size)
                } else {
                    size.padding_x()
                }
            }
        }
    }

    /// Gap between icon and label (`gap-2` base; nova `gap-1.5` on
    /// default/lg, `gap-1` on xs/sm).
    pub(crate) fn gap(size: ButtonSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ButtonSize::Xs | ButtonSize::Sm => 4.0,      // gap-1
                ButtonSize::Default | ButtonSize::Lg => 6.0, // gap-1.5
                _ => 0.0,
            },
            _ => size.gap(),
        }
    }

    /// Label size in points (`text-sm`; nova xs `text-xs`, sm `text-[0.8rem]`).
    pub(crate) fn text_size(size: ButtonSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ButtonSize::Xs => 12.0, // text-xs
                ButtonSize::Sm => 12.8, // text-[0.8rem]
                _ => 14.0,              // text-sm
            },
            _ => size.text_size(),
        }
    }

    /// Inline icon size (`size-4` from `.cn-button`; nova xs `size-3`,
    /// sm `size-3.5`).
    pub(crate) fn icon_size(size: ButtonSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ButtonSize::Xs | ButtonSize::IconXs => 12.0, // size-3
                ButtonSize::Sm | ButtonSize::IconSm => 14.0, // size-3.5
                _ => 16.0,                                   // size-4
            },
            _ => size.icon_size(),
        }
    }

    /// Corner radius: `rounded-md` for the base recipe; nova uses
    /// `rounded-lg` with `min(var(--radius-md), Npx)` caps on the xs/sm
    /// sizes and forces `rounded-lg` inside button groups
    /// (`in-data-[slot=button-group]:rounded-lg`).
    pub(crate) fn radius(
        size: ButtonSize,
        style: Style,
        radius: &RadiusTokens,
        in_group: bool,
    ) -> u8 {
        match style {
            Style::Nova => {
                if !in_group {
                    match size {
                        ButtonSize::Xs | ButtonSize::IconXs => {
                            return (radius.md().round() as i32).clamp(0, 10) as u8;
                        }
                        ButtonSize::Sm | ButtonSize::IconSm => {
                            return (radius.md().round() as i32).clamp(0, 12) as u8;
                        }
                        _ => {}
                    }
                }
                radius.lg_u8()
            }
            _ => radius.md_u8(),
        }
    }
}

/// Resolved colors for one button state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Colors {
    fill: Color32,
    fg: Color32,
    stroke: Stroke,
    /// Focus ring color (alpha already applied).
    ring: Color32,
    /// Draw the label underline (link variant on hover).
    underline: bool,
}

/// Resolves variant colors for the current style, palette, hover state and
/// mode.
///
/// Mirrors the base recipe (`dark:bg-destructive/60`, `dark:bg-input/30`,
/// `hover:bg-accent dark:hover:bg-accent/50`, ...) and the nova overrides
/// (`hover:bg-primary/80`, tinted `bg-destructive/10 text-destructive`,
/// `hover:bg-muted` outline/ghost hovers, ...). Nova also always carries a
/// 1px transparent border (`.cn-button` `border border-transparent`);
/// [`Button::show`] adds it so `focus-visible:border-ring` and
/// `aria-invalid:border-destructive` have a border to recolor.
pub(crate) fn resolve_colors(
    variant: ButtonVariant,
    p: &Palette,
    dark: bool,
    hovered: bool,
    style: Style,
) -> Colors {
    match variant {
        ButtonVariant::Default => {
            // Base `hover:bg-primary/90`; nova `hover:bg-primary/80`.
            let hover_alpha = if style == Style::Nova { 0.8 } else { 0.9 };
            Colors {
                fill: if hovered {
                    color_alpha(p.primary, hover_alpha)
                } else {
                    p.primary
                },
                fg: p.primary_foreground,
                stroke: Stroke::NONE,
                ring: color_alpha(p.ring, 0.5),
                underline: false,
            }
        }
        ButtonVariant::Destructive => {
            if style == Style::Nova {
                // Tinted: `bg-destructive/10 text-destructive
                // hover:bg-destructive/20 dark:bg-destructive/20
                // dark:hover:bg-destructive/30`, with
                // `focus-visible:ring-destructive/20
                // dark:focus-visible:ring-destructive/40`.
                let fill = if dark {
                    color_alpha(p.destructive, if hovered { 0.3 } else { 0.2 })
                } else {
                    color_alpha(p.destructive, if hovered { 0.2 } else { 0.1 })
                };
                Colors {
                    fill,
                    fg: p.destructive,
                    stroke: Stroke::NONE,
                    ring: color_alpha(p.destructive, if dark { 0.4 } else { 0.2 }),
                    underline: false,
                }
            } else {
                let fill = if dark {
                    color_alpha(p.destructive, if hovered { 0.9 } else { 0.6 })
                } else if hovered {
                    color_alpha(p.destructive, 0.9)
                } else {
                    p.destructive
                };
                Colors {
                    fill,
                    fg: Color32::WHITE,
                    stroke: Stroke::NONE,
                    ring: color_alpha(p.destructive, if dark { 0.4 } else { 0.2 }),
                    underline: false,
                }
            }
        }
        ButtonVariant::Outline => {
            let (fill, fg) = if hovered {
                if style == Style::Nova {
                    // `hover:bg-muted hover:text-foreground` (nova); dark
                    // keeps `dark:hover:bg-input/50`.
                    if dark {
                        (color_alpha(p.input, 0.5), p.foreground)
                    } else {
                        (p.muted, p.foreground)
                    }
                } else if dark {
                    (color_alpha(p.input, 0.5), p.foreground)
                } else {
                    (p.accent, p.accent_foreground)
                }
            } else if dark {
                (color_alpha(p.input, 0.3), p.foreground)
            } else {
                (p.background, p.foreground)
            };
            Colors {
                fill,
                fg,
                stroke: Stroke::new(1.0, if dark { p.input } else { p.border }),
                ring: color_alpha(p.ring, 0.5),
                underline: false,
            }
        }
        ButtonVariant::Secondary => {
            let fill = if hovered {
                if style == Style::Nova {
                    // `hover:bg-[color-mix(in_oklch,var(--secondary),
                    // var(--foreground)_5%)]` ≈ 5% foreground over secondary.
                    blend(p.secondary, color_alpha(p.foreground, 0.05))
                } else {
                    color_alpha(p.secondary, 0.8)
                }
            } else {
                p.secondary
            };
            Colors {
                fill,
                fg: p.secondary_foreground,
                stroke: Stroke::NONE,
                ring: color_alpha(p.ring, 0.5),
                underline: false,
            }
        }
        ButtonVariant::Ghost => {
            let fill = if hovered {
                if style == Style::Nova {
                    // `hover:bg-muted dark:hover:bg-muted/50`.
                    if dark {
                        color_alpha(p.muted, 0.5)
                    } else {
                        p.muted
                    }
                } else if dark {
                    color_alpha(p.accent, 0.5)
                } else {
                    p.accent
                }
            } else {
                Color32::TRANSPARENT
            };
            let fg = if hovered && style != Style::Nova {
                p.accent_foreground
            } else {
                // Nova `hover:text-foreground` and the resting state both use
                // the inherited `foreground`.
                p.foreground
            };
            Colors {
                fill,
                fg,
                stroke: Stroke::NONE,
                ring: color_alpha(p.ring, 0.5),
                underline: false,
            }
        }
        ButtonVariant::Link => Colors {
            fill: Color32::TRANSPARENT,
            fg: p.primary,
            stroke: Stroke::NONE,
            ring: color_alpha(p.ring, 0.5),
            underline: hovered,
        },
    }
}

/// A shadcn button. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Button {
    label: String,
    variant: ButtonVariant,
    size: ButtonSize,
    icon_start: Option<Icon>,
    icon_end: Option<Icon>,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl Button {
    /// Creates a button with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            icon_start: None,
            icon_end: None,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Sets the visual variant (default: [`ButtonVariant::Default`]).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size (default: [`ButtonSize::Default`]).
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Shows an icon before the label (`has-[>svg]` slot).
    pub fn icon_start(mut self, icon: Icon) -> Self {
        self.icon_start = Some(icon);
        self
    }

    /// Shows an icon after the label (`has-[>svg]` slot).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    /// Disables the button (`disabled:opacity-50 disabled:pointer-events-none`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the button aria-invalid (`aria-invalid:border-destructive
    /// aria-invalid:ring-destructive/20`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt (Button keeps no state, but a stable id helps
    /// focus order and tooling).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The widget's intrinsic size for the current theme, before any
    /// [`crate::components::button_group::ButtonGroup`] adjustments.
    pub fn content_size(&self, ui: &Ui) -> Vec2 {
        let style = ShadcnTheme::of_ui(ui).style;
        let size = self.size;
        if size.is_icon() {
            return Vec2::splat(metrics::height(size, style));
        }
        let font = FontId::proportional(metrics::text_size(size, style));
        let text_w = if self.label.is_empty() {
            0.0
        } else {
            ui.painter()
                .layout_no_wrap(self.label.clone(), font, Color32::WHITE)
                .size()
                .x
        };
        let has_start = self.icon_start.is_some();
        let has_end = self.icon_end.is_some();
        let icon_w = if has_start || has_end {
            metrics::icon_size(size, style)
        } else {
            0.0
        };
        let n_children = has_start as i32 + has_end as i32 + (!self.label.is_empty()) as i32;
        let gaps = (n_children - 1).max(0) as f32 * metrics::gap(size, style);
        Vec2::new(
            metrics::padding_left(size, style, has_start, has_end)
                + metrics::padding_right(size, style, has_start, has_end)
                + icon_w
                + gaps
                + text_w,
            metrics::height(size, style),
        )
    }

    /// Renders the button.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let style = theme.style;
        let nova = style == Style::Nova;
        let p = theme.palette;
        let size = self.size;

        // Auto-detected position inside a ButtonGroup (see button_group.rs).
        let group = group::next_child_position(ui);
        let position = group.map(|(_, pos)| pos).unwrap_or(GroupPosition::Only);
        let orientation = group
            .map(|(o, _)| o)
            .unwrap_or(GroupOrientation::Horizontal);
        let in_group = position != GroupPosition::Only;

        let desired = self.content_size(ui);

        // Reserve space without registering an auto-id focusable response, so
        // egui's spatial arrow-key navigation only sees the explicit widget id.
        let (_, rect) = ui.allocate_space(desired);
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_button"),
        };

        let response = if self.disabled {
            // Disabled at the egui level too: `enabled()` is false and egui
            // suppresses hover, clicks and focus. Interact through a disabled
            // child Ui; painting below keeps using the outer painter so the
            // child's disabled opacity doesn't double-fade the explicit
            // `disabled:opacity-50` color fade.
            let child = ui.new_child(egui::UiBuilder::new().disabled());
            child.interact(rect, id, Sense::click())
        } else {
            ui.interact(rect, id, Sense::click())
        };

        if ui.is_rect_visible(rect) {
            let radius = metrics::radius(size, style, &theme.radius, in_group);
            let corner = fuse_radius(CornerRadius::same(radius), orientation, position);

            // A disabled response is never hovered, so this stays false.
            let hovered = response.hovered();
            let mut c = resolve_colors(self.variant, &p, dark, hovered, style);

            // `aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40`
            if self.invalid {
                c.ring = color_alpha(p.destructive, if dark { 0.4 } else { 0.2 });
            }

            if self.disabled {
                // `disabled:opacity-50` — fade everything against the backdrop.
                c.fill = color_alpha(c.fill, 0.5);
                c.fg = color_alpha(c.fg, 0.5);
                c.stroke.color = color_alpha(c.stroke.color, 0.5);
            }

            let painter = ui.painter();

            // `shadow-xs` on the outline variant (base recipe only; the nova
            // outline has no shadow).
            if self.variant == ButtonVariant::Outline && !self.disabled && !nova {
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

            // Non-first children of a group overlap the previous border so the
            // shared edge shows one 1px divider (`border-l-0`).
            let mut draw_rect = match orientation {
                GroupOrientation::Horizontal
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(rect.min + Vec2::new(1.0, 0.0), rect.max)
                }
                GroupOrientation::Vertical
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(rect.min + Vec2::new(0.0, 1.0), rect.max)
                }
                _ => rect,
            };

            // `active:translate-y-px` (nova): the whole button shifts 1px
            // down while pressed.
            if nova && !self.disabled && response.is_pointer_button_down_on() {
                draw_rect.min.y += 1.0;
                draw_rect.max.y += 1.0;
            }

            painter.rect_filled(draw_rect, corner, c.fill);

            // Border: the base recipe draws only the outline variant's
            // border; nova `.cn-button` always carries a 1px transparent
            // border (`border border-transparent bg-clip-padding`), which
            // `focus-visible:border-ring` and `aria-invalid:border-destructive`
            // recolor.
            let mut stroke = c.stroke;
            if nova && stroke.width == 0.0 && !self.disabled {
                stroke = Stroke::new(1.0, Color32::TRANSPARENT);
            }
            if response.has_focus() && !self.disabled && stroke.width > 0.0 {
                // `focus-visible:border-ring`; nova destructive uses
                // `focus-visible:border-destructive/40`.
                stroke.color = if nova && self.variant == ButtonVariant::Destructive {
                    color_alpha(p.destructive, 0.4)
                } else {
                    p.ring
                };
            }
            if self.invalid && !self.disabled {
                // `aria-invalid:border-destructive
                // dark:aria-invalid:border-destructive/50` — nova has a border
                // on every variant; the base recipe only recolors the outline.
                if nova {
                    stroke = Stroke::new(
                        1.0,
                        if dark {
                            color_alpha(p.destructive, 0.5)
                        } else {
                            p.destructive
                        },
                    );
                } else if stroke.width > 0.0 {
                    stroke.color = p.destructive;
                }
            }
            if stroke.width > 0.0 {
                painter.rect_stroke(draw_rect, corner, stroke, egui::epaint::StrokeKind::Inside);
            }

            // Content: [icon] gap [label] gap [icon], centered.
            let font = FontId::proportional(metrics::text_size(size, style));
            let icon_size = metrics::icon_size(size, style);
            let gap = metrics::gap(size, style);
            let has_start = self.icon_start.is_some();
            let has_end = self.icon_end.is_some();
            let text_w = if self.label.is_empty() {
                0.0
            } else {
                painter
                    .layout_no_wrap(self.label.clone(), font.clone(), c.fg)
                    .size()
                    .x
            };
            let icons_w = (has_start as i32 + has_end as i32) as f32 * icon_size;
            let n_children = has_start as i32 + has_end as i32 + (!self.label.is_empty()) as i32;
            let gaps = (n_children - 1).max(0) as f32 * gap;
            let total_w = icons_w + gaps + text_w;

            let center = draw_rect.center();
            let mut cursor = center.x - total_w * 0.5;

            if let Some(icon) = self.icon_start {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, c.fg, 2.0 * icon_size / 24.0);
                cursor += icon_size + gap;
            }

            if !self.label.is_empty() {
                let text_center = Pos2::new(cursor + text_w * 0.5, center.y);
                painter.text(
                    text_center,
                    Align2::CENTER_CENTER,
                    self.label.clone(),
                    font,
                    c.fg,
                );
                if c.underline {
                    // `underline-offset-4`
                    let y = text_center.y + metrics::text_size(size, style) * 0.62 + 2.0;
                    painter.line_segment(
                        [Pos2::new(cursor, y), Pos2::new(cursor + text_w, y)],
                        Stroke::new(1.0, c.fg),
                    );
                }
                cursor += text_w + gap;
            }

            if let Some(icon) = self.icon_end {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, c.fg, 2.0 * icon_size / 24.0);
            }

            // `focus-visible:border-ring focus-visible:ring-[3px]
            // focus-visible:ring-ring/50` — 3px ring just outside the widget.
            if response.has_focus() && !self.disabled {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                focus::paint_ring(painter, ring_rect, c.ring, 3.0, radius as f32);
            }
        }

        response
    }
}

impl Widget for Button {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Position of a child inside a [`crate::components::button_group::ButtonGroup`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GroupPosition {
    /// The only child.
    Only,
    /// The first of several children.
    First,
    /// Between the first and last children.
    Middle,
    /// The last of several children.
    Last,
}

/// Layout direction of a [`crate::components::button_group::ButtonGroup`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupOrientation {
    /// Children laid out left to right.
    #[default]
    Horizontal,
    /// Children stacked top to bottom.
    Vertical,
}

/// Applies the button-group corner fusing: horizontal `rounded-r-none` for
/// non-last children (First zeroes the right corners) and `rounded-l-none`
/// for non-first children (Last zeroes the left corners); mirrored
/// vertically.
///
/// egui `CornerRadius`: `nw` = top-left, `ne` = top-right, `sw` =
/// bottom-left, `se` = bottom-right.
pub(crate) fn fuse_radius(
    mut corner: CornerRadius,
    o: GroupOrientation,
    pos: GroupPosition,
) -> CornerRadius {
    match (o, pos) {
        (GroupOrientation::Horizontal, GroupPosition::First) => {
            corner.ne = 0;
            corner.se = 0;
        }
        (GroupOrientation::Horizontal, GroupPosition::Middle) => {
            corner.nw = 0;
            corner.sw = 0;
            corner.ne = 0;
            corner.se = 0;
        }
        (GroupOrientation::Horizontal, GroupPosition::Last) => {
            corner.nw = 0;
            corner.sw = 0;
        }
        (GroupOrientation::Vertical, GroupPosition::First) => {
            corner.sw = 0;
            corner.se = 0;
        }
        (GroupOrientation::Vertical, GroupPosition::Middle) => {
            corner.nw = 0;
            corner.ne = 0;
            corner.sw = 0;
            corner.se = 0;
        }
        (GroupOrientation::Vertical, GroupPosition::Last) => {
            corner.nw = 0;
            corner.ne = 0;
        }
        (_, GroupPosition::Only) => {}
    }
    corner
}

/// Shared layout plumbing used by
/// [`crate::components::button_group::ButtonGroup`] to tell grouped widgets
/// their position, so borders and corners fuse like the source's
/// `[&>*:not(:first-child)]:rounded-l-none` selectors.
pub(crate) mod group {
    use super::{GroupOrientation, GroupPosition};
    use egui::{Context, Id, Ui};

    #[derive(Clone, Copy, Debug)]
    struct GroupState {
        orientation: GroupOrientation,
        frame: u64,
        drawn: usize,
        total: Option<usize>,
    }

    fn states_id() -> Id {
        Id::unique("shadcn_group_states")
    }

    fn stack_id() -> Id {
        Id::unique("shadcn_group_stack")
    }

    fn write_state(ctx: &Context, id: Id, state: Option<GroupState>) {
        ctx.data_mut(|d| {
            let mut all: Vec<(Id, GroupState)> = d.get_temp(states_id()).unwrap_or_default();
            if let Some(state) = state {
                if let Some(slot) = all.iter_mut().find(|(i, _)| *i == id) {
                    slot.1 = state;
                } else {
                    all.push((id, state));
                }
            } else {
                all.retain(|(i, _)| *i != id);
            }
            d.insert_temp(states_id(), all);
        });
    }

    fn read_state(ctx: &Context, id: Id) -> Option<GroupState> {
        ctx.data_mut(|d| {
            d.get_temp::<Vec<(Id, GroupState)>>(states_id())
                .and_then(|all| all.into_iter().find(|(i, _)| *i == id).map(|(_, s)| s))
        })
    }

    /// A group starts laying out children; call before the child closure.
    pub(crate) fn begin(ui: &Ui, id: Id, orientation: GroupOrientation) {
        let ctx = ui.ctx().clone();
        let frame = ctx.cumulative_frame_nr();
        let mut state = read_state(&ctx, id).unwrap_or(GroupState {
            orientation,
            frame,
            drawn: 0,
            total: None,
        });
        if state.frame != frame {
            // New frame: keep last frame's `total` for position math.
            state.frame = frame;
            state.drawn = 0;
            state.orientation = orientation;
        }
        write_state(&ctx, id, Some(state));
        let mut stack: Vec<Id> = ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
        stack.push(id);
        ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
    }

    /// The group finished; call after the child closure. Records the child
    /// count so the next frame can compute First/Middle/Last.
    pub(crate) fn end(ui: &Ui, id: Id) {
        let ctx = ui.ctx().clone();
        if let Some(mut state) = read_state(&ctx, id) {
            state.total = Some(state.drawn);
            write_state(&ctx, id, Some(state));
        }
        let mut stack: Vec<Id> = ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
        stack.pop();
        ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
    }

    /// Called by widgets that participate in a group. Returns the group
    /// orientation and this widget's position, or `None` outside a group.
    /// Call at most once per widget per frame, before painting.
    pub fn next_child_position(ui: &Ui) -> Option<(GroupOrientation, GroupPosition)> {
        let ctx = ui.ctx().clone();
        let id: Id = *ctx
            .data_mut(|d| d.get_temp::<Vec<Id>>(stack_id()))?
            .last()?;
        let mut state = read_state(&ctx, id)?;
        if state.frame != ctx.cumulative_frame_nr() {
            return None;
        }
        let total = state.total;
        let index = state.drawn;
        state.drawn += 1;
        write_state(&ctx, id, Some(state));
        let position = match (index, total) {
            (_, Some(1)) => GroupPosition::Only,
            (0, _) => GroupPosition::First,
            (_, Some(n)) if index + 1 == n => GroupPosition::Last,
            _ => GroupPosition::Middle,
        };
        Some((state.orientation, position))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset, RadiusTokens};

    /// Any non-nova style uses the base recipe (`buttonVariants`).
    const BASE_STYLE: Style = Style::Vega;

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn size_geometry_matches_source() {
        // default h-9 px-4, xs h-6 px-2, sm h-8 px-3, lg h-10 px-6
        assert_eq!(ButtonSize::Default.height(), 36.0);
        assert_eq!(ButtonSize::Default.padding_x(), 16.0);
        assert_eq!(ButtonSize::Xs.height(), 24.0);
        assert_eq!(ButtonSize::Xs.padding_x(), 8.0);
        assert_eq!(ButtonSize::Xs.text_size(), 12.0);
        assert_eq!(ButtonSize::Sm.height(), 32.0);
        assert_eq!(ButtonSize::Sm.padding_x(), 12.0);
        assert_eq!(ButtonSize::Lg.height(), 40.0);
        assert_eq!(ButtonSize::Lg.padding_x(), 24.0);
        // icon 9x9=36 square, icon-xs 24, icon-sm 32, icon-lg 40
        assert_eq!(ButtonSize::Icon.height(), 36.0);
        assert_eq!(ButtonSize::IconXs.height(), 24.0);
        assert_eq!(ButtonSize::IconSm.height(), 32.0);
        assert_eq!(ButtonSize::IconLg.height(), 40.0);
        assert!(ButtonSize::Icon.is_icon());
        assert!(!ButtonSize::Default.is_icon());
        // inline icons: size-4, size-3 in xs
        assert_eq!(ButtonSize::Default.icon_size(), 16.0);
        assert_eq!(ButtonSize::Xs.icon_size(), 12.0);
    }

    #[test]
    fn base_recipe_icon_padding_shrinks_px() {
        // has-[>svg]:px-3 on default, px-2.5 sm, px-4 lg, px-1.5 xs
        assert_eq!(
            metrics::padding_left(ButtonSize::Default, BASE_STYLE, false, true),
            12.0
        );
        assert_eq!(
            metrics::padding_right(ButtonSize::Default, BASE_STYLE, true, false),
            12.0
        );
        assert_eq!(
            metrics::padding_left(ButtonSize::Sm, BASE_STYLE, false, true),
            10.0
        );
        assert_eq!(
            metrics::padding_left(ButtonSize::Lg, BASE_STYLE, false, true),
            16.0
        );
        assert_eq!(
            metrics::padding_left(ButtonSize::Xs, BASE_STYLE, true, true),
            6.0
        );
        // no icons keeps px-4
        assert_eq!(
            metrics::padding_left(ButtonSize::Default, BASE_STYLE, false, false),
            16.0
        );
    }

    #[test]
    fn nova_size_geometry_matches_source() {
        use crate::theme::Style;
        let nova = Style::Nova;
        // default h-8 px-2.5 gap-1.5; lg h-9 px-2.5 gap-1.5
        assert_eq!(metrics::height(ButtonSize::Default, nova), 32.0);
        assert_eq!(
            metrics::padding_left(ButtonSize::Default, nova, false, false),
            10.0
        );
        assert_eq!(metrics::gap(ButtonSize::Default, nova), 6.0);
        assert_eq!(metrics::height(ButtonSize::Lg, nova), 36.0);
        assert_eq!(
            metrics::padding_right(ButtonSize::Lg, nova, false, false),
            10.0
        );
        // sm h-7 px-2.5 gap-1 text-[0.8rem] icon size-3.5; xs h-6 px-2 gap-1
        assert_eq!(metrics::height(ButtonSize::Sm, nova), 28.0);
        assert_eq!(metrics::text_size(ButtonSize::Sm, nova), 12.8);
        assert_eq!(metrics::icon_size(ButtonSize::Sm, nova), 14.0);
        assert_eq!(metrics::gap(ButtonSize::Sm, nova), 4.0);
        assert_eq!(metrics::height(ButtonSize::Xs, nova), 24.0);
        assert_eq!(metrics::text_size(ButtonSize::Xs, nova), 12.0);
        assert_eq!(metrics::icon_size(ButtonSize::Xs, nova), 12.0);
        // icon squares: size-6 / size-7 / size-8 / size-9
        assert_eq!(metrics::height(ButtonSize::IconXs, nova), 24.0);
        assert_eq!(metrics::height(ButtonSize::IconSm, nova), 28.0);
        assert_eq!(metrics::height(ButtonSize::Icon, nova), 32.0);
        assert_eq!(metrics::height(ButtonSize::IconLg, nova), 36.0);
        // with icons: pl/pr-2 on default/lg, pl/pr-1.5 on xs/sm (per side)
        assert_eq!(
            metrics::padding_left(ButtonSize::Default, nova, true, false),
            8.0
        );
        assert_eq!(
            metrics::padding_right(ButtonSize::Default, nova, false, true),
            8.0
        );
        assert_eq!(
            metrics::padding_right(ButtonSize::Default, nova, false, false),
            10.0
        );
        assert_eq!(
            metrics::padding_left(ButtonSize::Xs, nova, true, false),
            6.0
        );
        assert_eq!(
            metrics::padding_left(ButtonSize::Xs, nova, false, true),
            8.0
        );
        assert_eq!(
            metrics::padding_left(ButtonSize::Sm, nova, true, false),
            6.0
        );
    }

    #[test]
    fn nova_radius_uses_lg_with_size_caps() {
        use crate::theme::Style;
        let nova = Style::Nova;
        let r = RadiusTokens::new(10.0);
        // rounded-lg base; min(var(--radius-md), 10px) on xs,
        // min(var(--radius-md), 12px) on sm
        assert_eq!(metrics::radius(ButtonSize::Default, nova, &r, false), 10);
        assert_eq!(metrics::radius(ButtonSize::Lg, nova, &r, false), 10);
        assert_eq!(metrics::radius(ButtonSize::Icon, nova, &r, false), 10);
        assert_eq!(metrics::radius(ButtonSize::Xs, nova, &r, false), 8);
        assert_eq!(metrics::radius(ButtonSize::Sm, nova, &r, false), 8);
        assert_eq!(metrics::radius(ButtonSize::IconXs, nova, &r, false), 8);
        // button groups force rounded-lg on every size
        assert_eq!(metrics::radius(ButtonSize::Xs, nova, &r, true), 10);
        assert_eq!(metrics::radius(ButtonSize::Sm, nova, &r, true), 10);
        // base recipe stays rounded-md
        assert_eq!(
            metrics::radius(ButtonSize::Default, BASE_STYLE, &r, false),
            8
        );
        assert_eq!(metrics::radius(ButtonSize::Xs, BASE_STYLE, &r, false), 8);
    }

    #[test]
    fn default_variant_uses_primary_with_hover_90() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Default, &p, false, false, BASE_STYLE);
        assert_eq!(c.fill, p.primary);
        assert_eq!(c.fg, p.primary_foreground);
        let h = resolve_colors(ButtonVariant::Default, &p, false, true, BASE_STYLE);
        assert_eq!(h.fill, color_alpha(p.primary, 0.9));
    }

    #[test]
    fn nova_default_hovers_primary_80() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Default, &p, false, false, Style::Nova);
        assert_eq!(c.fill, p.primary);
        assert_eq!(c.fg, p.primary_foreground);
        let h = resolve_colors(ButtonVariant::Default, &p, false, true, Style::Nova);
        assert_eq!(h.fill, color_alpha(p.primary, 0.8));
    }

    #[test]
    fn destructive_uses_white_text_and_tinted_dark_fill() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Destructive, &p, false, false, BASE_STYLE);
        assert_eq!(c.fill, p.destructive);
        assert_eq!(c.fg, Color32::WHITE);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(ButtonVariant::Destructive, &dark, true, false, BASE_STYLE);
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.6));
        let dh = resolve_colors(ButtonVariant::Destructive, &dark, true, true, BASE_STYLE);
        assert_eq!(dh.fill, color_alpha(dark.destructive, 0.9));
    }

    #[test]
    fn nova_destructive_is_tinted_with_destructive_text() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Destructive, &p, false, false, Style::Nova);
        assert_eq!(c.fill, color_alpha(p.destructive, 0.1));
        assert_eq!(c.fg, p.destructive);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
        let h = resolve_colors(ButtonVariant::Destructive, &p, false, true, Style::Nova);
        assert_eq!(h.fill, color_alpha(p.destructive, 0.2));
        let dark = palette(Mode::Dark);
        let d = resolve_colors(ButtonVariant::Destructive, &dark, true, false, Style::Nova);
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.2));
        assert_eq!(d.fg, dark.destructive);
        assert_eq!(d.ring, color_alpha(dark.destructive, 0.4));
        let dh = resolve_colors(ButtonVariant::Destructive, &dark, true, true, Style::Nova);
        assert_eq!(dh.fill, color_alpha(dark.destructive, 0.3));
    }

    #[test]
    fn outline_matches_source_states() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Outline, &p, false, false, BASE_STYLE);
        assert_eq!(c.fill, p.background);
        assert_eq!(c.stroke.color, p.border);
        let h = resolve_colors(ButtonVariant::Outline, &p, false, true, BASE_STYLE);
        assert_eq!(h.fill, p.accent);
        assert_eq!(h.fg, p.accent_foreground);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(ButtonVariant::Outline, &dark, true, false, BASE_STYLE);
        assert_eq!(d.fill, color_alpha(dark.input, 0.3));
        assert_eq!(d.stroke.color, dark.input);
        let dh = resolve_colors(ButtonVariant::Outline, &dark, true, true, BASE_STYLE);
        assert_eq!(dh.fill, color_alpha(dark.input, 0.5));
    }

    #[test]
    fn nova_outline_hovers_muted_with_foreground_text() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Outline, &p, false, false, Style::Nova);
        assert_eq!(c.fill, p.background);
        assert_eq!(c.stroke.color, p.border);
        assert_eq!(c.fg, p.foreground);
        let h = resolve_colors(ButtonVariant::Outline, &p, false, true, Style::Nova);
        assert_eq!(h.fill, p.muted);
        assert_eq!(h.fg, p.foreground);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(ButtonVariant::Outline, &dark, true, false, Style::Nova);
        assert_eq!(d.fill, color_alpha(dark.input, 0.3));
        assert_eq!(d.stroke.color, dark.input);
        let dh = resolve_colors(ButtonVariant::Outline, &dark, true, true, Style::Nova);
        assert_eq!(dh.fill, color_alpha(dark.input, 0.5));
        assert_eq!(dh.fg, dark.foreground);
    }

    #[test]
    fn secondary_hover_is_80_and_ghost_hover_uses_accent() {
        let p = palette(Mode::Light);
        let h = resolve_colors(ButtonVariant::Secondary, &p, false, true, BASE_STYLE);
        assert_eq!(h.fill, color_alpha(p.secondary, 0.8));
        let g = resolve_colors(ButtonVariant::Ghost, &p, false, true, BASE_STYLE);
        assert_eq!(g.fill, p.accent);
        assert_eq!(g.fg, p.accent_foreground);
        let dark = palette(Mode::Dark);
        let gd = resolve_colors(ButtonVariant::Ghost, &dark, true, true, BASE_STYLE);
        assert_eq!(gd.fill, color_alpha(dark.accent, 0.5));
        let g0 = resolve_colors(ButtonVariant::Ghost, &dark, true, false, BASE_STYLE);
        assert_eq!(g0.fill, Color32::TRANSPARENT);
        assert_eq!(g0.fg, dark.foreground);
    }

    #[test]
    fn nova_secondary_and_ghost_hover_muted() {
        let p = palette(Mode::Light);
        // color-mix(in oklch, var(--secondary), var(--foreground) 5%)
        let h = resolve_colors(ButtonVariant::Secondary, &p, false, true, Style::Nova);
        assert_eq!(h.fill, blend(p.secondary, color_alpha(p.foreground, 0.05)));
        let s = resolve_colors(ButtonVariant::Secondary, &p, false, false, Style::Nova);
        assert_eq!(s.fill, p.secondary);
        // ghost: hover:bg-muted hover:text-foreground dark:hover:bg-muted/50
        let g = resolve_colors(ButtonVariant::Ghost, &p, false, true, Style::Nova);
        assert_eq!(g.fill, p.muted);
        assert_eq!(g.fg, p.foreground);
        let dark = palette(Mode::Dark);
        let gd = resolve_colors(ButtonVariant::Ghost, &dark, true, true, Style::Nova);
        assert_eq!(gd.fill, color_alpha(dark.muted, 0.5));
        assert_eq!(gd.fg, dark.foreground);
        let g0 = resolve_colors(ButtonVariant::Ghost, &dark, true, false, Style::Nova);
        assert_eq!(g0.fill, Color32::TRANSPARENT);
        assert_eq!(g0.fg, dark.foreground);
    }

    #[test]
    fn link_underlines_on_hover() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Link, &p, false, false, BASE_STYLE);
        assert_eq!(c.fg, p.primary);
        assert!(!c.underline);
        let h = resolve_colors(ButtonVariant::Link, &p, false, true, BASE_STYLE);
        assert!(h.underline);
    }

    #[test]
    fn disabled_response_is_disabled_at_the_egui_level() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("button_disabled_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            );
            let enabled = Button::new("On").show(ui);
            assert!(enabled.enabled());
            let disabled = Button::new("Off").disabled(true).show(ui);
            assert!(!disabled.enabled());
            // No clicks or hover ever come through a disabled response.
            assert!(!disabled.clicked());
            assert!(!disabled.hovered());
            // id_salt and the fallback produce different, stable ids.
            assert_ne!(enabled.id, disabled.id);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn fuse_radius_horizontal() {
        let r = CornerRadius::same(8);
        let only = fuse_radius(r, GroupOrientation::Horizontal, GroupPosition::Only);
        assert_eq!(only, r);
        // First: `rounded-r-none` — right corners (ne, se) zeroed.
        let first = fuse_radius(r, GroupOrientation::Horizontal, GroupPosition::First);
        assert_eq!((first.nw, first.sw), (8, 8));
        assert_eq!((first.ne, first.se), (0, 0));
        let middle = fuse_radius(r, GroupOrientation::Horizontal, GroupPosition::Middle);
        assert_eq!((middle.nw, middle.ne, middle.sw, middle.se), (0, 0, 0, 0));
        // Last: `rounded-l-none` — left corners (nw, sw) zeroed.
        let last = fuse_radius(r, GroupOrientation::Horizontal, GroupPosition::Last);
        assert_eq!((last.ne, last.se), (8, 8));
        assert_eq!((last.nw, last.sw), (0, 0));
    }

    #[test]
    fn fuse_radius_vertical() {
        let r = CornerRadius::same(8);
        let first = fuse_radius(r, GroupOrientation::Vertical, GroupPosition::First);
        assert_eq!((first.nw, first.ne), (8, 8));
        assert_eq!((first.sw, first.se), (0, 0));
        let last = fuse_radius(r, GroupOrientation::Vertical, GroupPosition::Last);
        assert_eq!((last.nw, last.ne), (0, 0));
        assert_eq!((last.sw, last.se), (8, 8));
    }

    #[test]
    fn group_positions_are_assigned_in_order() {
        let ctx = egui::Context::default();
        let id = egui::Id::unique("g1");
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("group_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            )
        };

        // Frame 1: total unknown, so trailing positions default to Middle.
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            group::begin(ui, id, GroupOrientation::Horizontal);
            let first = group::next_child_position(ui).unwrap();
            let second = group::next_child_position(ui).unwrap();
            let third = group::next_child_position(ui).unwrap();
            group::end(ui, id);
            assert_eq!(first, (GroupOrientation::Horizontal, GroupPosition::First));
            assert_eq!(
                second,
                (GroupOrientation::Horizontal, GroupPosition::Middle)
            );
            assert_eq!(third, (GroupOrientation::Horizontal, GroupPosition::Middle));
        });
        out.textures_delta.clear();

        // Frame 2: the total from frame 1 marks the last child.
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            group::begin(ui, id, GroupOrientation::Horizontal);
            let p1 = group::next_child_position(ui).unwrap();
            let p2 = group::next_child_position(ui).unwrap();
            let p3 = group::next_child_position(ui).unwrap();
            group::end(ui, id);
            assert_eq!(p1.1, GroupPosition::First);
            assert_eq!(p2.1, GroupPosition::Middle);
            assert_eq!(p3.1, GroupPosition::Last);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn single_child_group_is_only() {
        let ctx = egui::Context::default();
        let id = egui::Id::unique("g2");
        for frame in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("group_single"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                group::begin(ui, id, GroupOrientation::Vertical);
                let (o, pos) = group::next_child_position(ui).unwrap();
                group::end(ui, id);
                assert_eq!(o, GroupOrientation::Vertical);
                if frame >= 1 {
                    // Stable from the second frame on (total from frame 1).
                    assert_eq!(pos, GroupPosition::Only);
                }
            });
            out.textures_delta.clear();
        }
    }
}
