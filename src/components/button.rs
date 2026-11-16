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
//! disabled opacity and button-group corner fusing. State changes animate
//! through the shared transition system (`transition-all`, 150ms with the
//! Tailwind default easing, frozen spec §7): fills, text, borders and the
//! focus ring fade, and the nova pressed shift animates instead of snapping.
//!
//! Keyboard behaviour follows the native `<button>` (frozen spec §8):
//! clicking a button moves focus to it, Enter activates on keydown, Space
//! activates on keyup while showing the pressed state, and Escape and
//! arrow keys are no-ops. The focus ring shows whenever the button holds
//! focus (deviation D3: egui cannot reproduce the browser's
//! `:focus-visible` heuristic). See [`Button::show`] for the full
//! interaction contract.
//!
//! Group plumbing ([`group`], Foundation Loop FG-1/2/3): inside a
//! `ButtonGroup`, a text button stretches to the group's measured cross
//! size in vertical groups (FG-1), the last slot-carrying child gets its
//! trailing corners forced to `rounded-lg` under nova (FG-2, the
//! `[data-slot]` selector semantics), and a focused member's ring paints
//! above following siblings (FG-3, `z-10`). All three channels are inert
//! outside a group: standalone buttons render exactly as before.
//! Popup triggers should set `.has_popup(true)` (suppresses the pressed
//! shift, the `haspopup` exclusion) and `.expanded(true)` while open (the
//! nova `aria-expanded` fills). Icons accept an explicit size via the
//! `icon_*_sized` setters, which wins over the size-derived default, and
//! every icon counts toward the content width (frozen spec §4).
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
use crate::utils::animation::{
    mix_oklab, transition_alpha, transition_color_to, transition_progress, TransitionSpec,
};
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Align2, Color32, CornerRadius, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget,
};

/// Visual style variants, mirroring the source `buttonVariants`.
///
/// The class strings quoted on each variant are the base recipe; the nova
/// style (the default) overrides several fills and hovers — tinted
/// destructive, `hover:bg-primary/80`, muted outline/ghost hovers — as
/// described in the [module docs](self).
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
///
/// The quoted class strings are the base recipe; the nova style overrides
/// heights, paddings, gaps, text and icon sizes at render time (frozen
/// spec §3). Use [`Button::content_size`] for the size a button actually
/// renders at under the active theme.
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

/// Base-recipe size metrics.
///
/// These methods return the base-recipe (`buttonVariants`) values; the
/// nova style overrides them at render time (see the [`ButtonSize`] docs).
/// For the rendered size under the active theme, use
/// [`Button::content_size`].
impl ButtonSize {
    /// Outer height in points for the base recipe (`h-9` etc.; for icon
    /// sizes the square side). The nova style renders different heights.
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

/// Resolves variant colors for the current style, palette, hover state,
/// expanded state and mode.
///
/// Mirrors the base recipe (`dark:bg-destructive/60`, `dark:bg-input/30`,
/// `hover:bg-accent dark:hover:bg-accent/50`, ...) and the nova overrides
/// (`hover:bg-primary/90` vs `hover:bg-primary/80`, tinted
/// `bg-destructive/10 text-destructive`, `hover:bg-muted` outline/ghost
/// hovers, ...), plus the nova `aria-expanded:*` fills for
/// outline/secondary/ghost (frozen spec §5.7; the base recipe declares no
/// expanded rule). When a button is both hovered and expanded, the hover
/// fill wins: the cascade precedence is spec OPEN item O6b, still
/// unresolved after S5/S10 with hover-wins standing, and the choice is
/// recorded here.
pub(crate) fn resolve_colors(
    variant: ButtonVariant,
    p: &Palette,
    dark: bool,
    hovered: bool,
    expanded: bool,
    style: Style,
) -> Colors {
    let (mut fill, mut fg, stroke, ring, underline) = match variant {
        ButtonVariant::Default => {
            // Base `hover:bg-primary/90`; nova `hover:bg-primary/80`.
            let hover_alpha = if style == Style::Nova { 0.8 } else { 0.9 };
            (
                if hovered {
                    color_alpha(p.primary, hover_alpha)
                } else {
                    p.primary
                },
                p.primary_foreground,
                Stroke::NONE,
                color_alpha(p.ring, 0.5),
                false,
            )
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
                (
                    fill,
                    p.destructive,
                    Stroke::NONE,
                    color_alpha(p.destructive, if dark { 0.4 } else { 0.2 }),
                    false,
                )
            } else {
                let fill = if dark {
                    color_alpha(p.destructive, if hovered { 0.9 } else { 0.6 })
                } else if hovered {
                    color_alpha(p.destructive, 0.9)
                } else {
                    p.destructive
                };
                (
                    fill,
                    Color32::WHITE,
                    Stroke::NONE,
                    color_alpha(p.destructive, if dark { 0.4 } else { 0.2 }),
                    false,
                )
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
                    // `dark:hover:bg-input/50`;
                    // `hover:text-accent-foreground` has no dark qualifier,
                    // so the text stays accent-foreground (spec §2.3).
                    (color_alpha(p.input, 0.5), p.accent_foreground)
                } else {
                    (p.accent, p.accent_foreground)
                }
            } else if dark {
                (color_alpha(p.input, 0.3), p.foreground)
            } else {
                (p.background, p.foreground)
            };
            (
                fill,
                fg,
                Stroke::new(1.0, if dark { p.input } else { p.border }),
                color_alpha(p.ring, 0.5),
                false,
            )
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
            (
                fill,
                p.secondary_foreground,
                Stroke::NONE,
                color_alpha(p.ring, 0.5),
                false,
            )
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
            (fill, fg, Stroke::NONE, color_alpha(p.ring, 0.5), false)
        }
        ButtonVariant::Link => (
            Color32::TRANSPARENT,
            p.primary,
            Stroke::NONE,
            color_alpha(p.ring, 0.5),
            hovered,
        ),
    };
    // Nova `aria-expanded:*` fills (spec §5.7). Hover wins when both apply
    // (spec OPEN O6 judgment call — see the fn docs).
    if expanded && !hovered && style == Style::Nova {
        match variant {
            ButtonVariant::Outline | ButtonVariant::Ghost => {
                fill = p.muted;
                fg = p.foreground;
            }
            ButtonVariant::Secondary => {
                fill = p.secondary;
                fg = p.secondary_foreground;
            }
            _ => {}
        }
    }
    Colors {
        fill,
        fg,
        stroke,
        ring,
        underline,
    }
}

/// Mixes two colors at `t` (0.0..1.0) for `transition-all` (spec §7).
/// Opaque endpoints mix in OKLab; an endpoint with zero alpha fades by
/// ramping the other endpoint's alpha at a constant hue, because mixing
/// toward transparent in OKLab dips through black (a fully transparent
/// endpoint has no defined hue). An endpoint with partial alpha mixes in
/// premultiplied sRGB space instead: unmultiplied OKLab would sweep the
/// endpoint's full lightness while alpha rises, so a translucent white
/// fading toward an opaque dark colour bulges brighter than both
/// endpoints when composited (BTN-FLK-03). Lerping the stored
/// premultiplied channels keeps the composited value linear in `t`, hence
/// monotonic and within the endpoint range for any backdrop.
pub(crate) fn mix_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    if t == 0.0 {
        return a;
    }
    if t == 1.0 {
        return b;
    }
    if a.a() == 0 {
        return color_alpha(b, t);
    }
    if b.a() == 0 {
        return color_alpha(a, 1.0 - t);
    }
    if a.a() < 255 || b.a() < 255 {
        // Premultiplied lerp: egui stores premultiplied sRGBA, so the
        // stored channels and alpha lerp together and the composited
        // result is exactly linear in `t`.
        let lerp = |x: u8, y: u8| -> u8 { ((x as f32 + (y as f32 - x as f32) * t) + 0.5) as u8 };
        return Color32::from_rgba_premultiplied(
            lerp(a.r(), b.r()),
            lerp(a.g(), b.g()),
            lerp(a.b(), b.b()),
            lerp(a.a(), b.a()),
        );
    }
    mix_oklab(a, b, t)
}

/// Mixes two resolved color sets at `t`, for `transition-all` (spec §7).
/// Stroke width is taken from `a` (hover and expanded never change a
/// variant's border width); the underline flag is taken from `a` and the
/// caller overrides it with the target hover state, since underlines snap
/// in CSS (text-decoration is not animatable).
pub(crate) fn mix_colors(a: Colors, b: Colors, t: f32) -> Colors {
    Colors {
        fill: mix_color(a.fill, b.fill, t),
        fg: mix_color(a.fg, b.fg, t),
        stroke: Stroke::new(a.stroke.width, mix_color(a.stroke.color, b.stroke.color, t)),
        ring: mix_color(a.ring, b.ring, t),
        underline: a.underline,
    }
}

/// Wraps one resolved endpoint set's fill/foreground in the endpoint-change
/// animator ([`transition_color_to`]), for `transition-all` over theme/style
/// switches (spec §7 scope note, BTN-FLK-01).
///
/// Only fill and foreground are wrapped: the stroke field is discarded by
/// the border computation below (which wraps its own rest/focus endpoints),
/// and the ring field feeds the ring target (wrapped once, below) — wrapping
/// either here too would run two legs on the same colour. Callers pass a
/// distinct sub-`id` per endpoint set; the helper takes `fill`/`fg` sub-ids
/// under it.
///
/// The legs run on the resolved endpoints — whose targets never cross the
/// transparent boundary between themes (opaque in both, transparent in
/// both, or translucent-but-defined in both; pinned by the test
/// `endpoint_slots_never_cross_the_transparent_boundary`) — so the helper's
/// documented caveat holds, and state mixing below blends leg values with
/// the boundary-safe [`mix_color`]. Wrapping the final mixed colour instead
/// (the original FLK-01 shape) ran OKLab legs across the boundary on every
/// state fade: a ghost hover leg from a transparent `from` dips through
/// black (BTN-FLK-02).
fn theme_animate_endpoint(
    ctx: &egui::Context,
    id: egui::Id,
    c: Colors,
    spec: TransitionSpec,
) -> Colors {
    Colors {
        fill: transition_color_to(ctx, id.with("fill"), c.fill, spec).color,
        fg: transition_color_to(ctx, id.with("fg"), c.fg, spec).color,
        stroke: c.stroke,
        ring: c.ring,
        underline: c.underline,
    }
}

/// Fades every channel of a resolved color set by `opacity`
/// (`disabled:opacity-50` composites the whole element, spec §5.5).
pub(crate) fn fade_colors(c: Colors, opacity: f32) -> Colors {
    Colors {
        fill: color_alpha(c.fill, opacity),
        fg: color_alpha(c.fg, opacity),
        stroke: Stroke::new(c.stroke.width, color_alpha(c.stroke.color, opacity)),
        ring: color_alpha(c.ring, opacity),
        underline: c.underline,
    }
}

/// Resolves the border stroke for the current state (pure, unit-testable).
///
/// Rest stroke: the outline variant's 1px border; under nova every variant
/// carries the 1px transparent border (`.cn-button` `border
/// border-transparent`), so there is always a border to recolor; under the
/// base recipe non-outline variants have no border at all. Focus recolors
/// the border to `ring` (nova destructive: `destructive/40`); invalid
/// recolors it to `destructive` (nova dark: `destructive/50`). Invalid wins
/// over focus when both apply (spec §5.4; spec OPEN item O6a, closed
/// conformant by S5).
pub(crate) fn stroke_for(
    variant: ButtonVariant,
    p: &Palette,
    dark: bool,
    style: Style,
    focused: bool,
    invalid: bool,
) -> Stroke {
    let nova = style == Style::Nova;
    let mut stroke = if variant == ButtonVariant::Outline {
        Stroke::new(1.0, if dark { p.input } else { p.border })
    } else if nova {
        Stroke::new(1.0, Color32::TRANSPARENT)
    } else {
        Stroke::NONE
    };
    if focused && stroke.width > 0.0 {
        // `focus-visible:border-ring`; nova destructive uses
        // `focus-visible:border-destructive/40`.
        stroke.color = if nova && variant == ButtonVariant::Destructive {
            color_alpha(p.destructive, 0.4)
        } else {
            p.ring
        };
    }
    if invalid {
        // `aria-invalid:border-destructive
        // dark:aria-invalid:border-destructive/50` — nova has a border on
        // every variant; the base recipe only recolors the outline.
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
    stroke
}

/// Whether the nova `active:translate-y-px` pressed shift applies at all:
/// nova only, suppressed for popup triggers (the `haspopup` exclusion, spec
/// §5.3) and for disabled buttons (no `:active` without pointer events).
pub(crate) fn active_shift_applies(style: Style, has_popup: bool, disabled: bool) -> bool {
    style == Style::Nova && !has_popup && !disabled
}

/// Whether the 3px ring is shown: on focus in both recipes (spec §5.4), or —
/// under nova only — whenever the button is invalid (`aria-invalid:ring-3`,
/// spec §5.6, which applies whether or not the button is focused). The base
/// recipe has no non-focus ring-width rule for invalid, so a base invalid
/// button shows no ring until focused.
fn ring_shown(focused: bool, invalid: bool, nova: bool) -> bool {
    focused || (invalid && nova)
}

/// Space-keyup activation (spec §8, BTN-INT-04).
///
/// Native `<button>` fires on Space KEYUP; egui's `interact` reports a
/// fake click whenever the focused widget sees `key_pressed(Space)`, so
/// without intervention Space fires on keydown (and re-fires on every
/// autorepeat). This helper, called before `interact` while the button
/// holds focus, hides this frame's Space keydown events from `interact`,
/// remembers that Space was pressed here ("armed"), and re-creates the
/// keydown on the keyup frame so `interact` reports the click then.
///
/// Only the event list `interact` reads is touched: `keys_down` is
/// computed in `begin_pass`, before widgets run, so `key_down(Space)` —
/// and therefore the `:active`-while-held rendering — is unaffected.
/// Enter events are never touched (Enter fires on keydown, per spec).
/// The synthetic keydown on the keyup frame can only fake-click the
/// focused widget, and copies the release event's physical key and
/// modifiers. A Space released without a preceding keydown on this
/// button (focus arrived mid-hold) does not fire; a stale armed flag
/// is cleared on the first frame the button is shown with Space up.
fn space_keyup_fire(ctx: &egui::Context, id: egui::Id) {
    let armed_id = id.with("space_armed");
    let mut armed = ctx
        .data_mut(|data| data.get_temp::<bool>(armed_id))
        .unwrap_or(false);
    ctx.input_mut(|input| {
        let space_pressed = input.events.iter().any(|event| {
            matches!(
                event,
                egui::Event::Key {
                    key: egui::Key::Space,
                    pressed: true,
                    ..
                }
            )
        });
        let release = input.events.iter().find_map(|event| match event {
            egui::Event::Key {
                key: egui::Key::Space,
                pressed: false,
                physical_key,
                modifiers,
                ..
            } => Some((*physical_key, *modifiers)),
            _ => None,
        });
        if space_pressed {
            input.events.retain(|event| {
                !matches!(
                    event,
                    egui::Event::Key {
                        key: egui::Key::Space,
                        pressed: true,
                        ..
                    }
                )
            });
            armed = true;
        }
        if let Some((physical_key, modifiers)) = release {
            if armed {
                input.events.push(egui::Event::Key {
                    key: egui::Key::Space,
                    physical_key,
                    pressed: true,
                    repeat: false,
                    modifiers,
                });
            }
            armed = false;
        } else if !input.key_down(egui::Key::Space) {
            armed = false;
        }
    });
    ctx.data_mut(|data| data.insert_temp(armed_id, armed));
}

/// Shrinks every corner radius by 1px (saturating): the padding-box corners
/// of a `bg-clip-padding` fill sit 1px inside the border-box corners.
fn shrink_corner(corner: CornerRadius) -> CornerRadius {
    CornerRadius {
        nw: corner.nw.saturating_sub(1),
        ne: corner.ne.saturating_sub(1),
        sw: corner.sw.saturating_sub(1),
        se: corner.se.saturating_sub(1),
    }
}

/// Paints the outline variant's `shadow-xs` through the LAYER resolution
/// API (spec §2.3). When disabled, the shadow fades with the element's
/// target opacity: the layer alphas scale on the stack (no allocation) and
/// the cache keys on the faded spec, so the animated opacity fade never
/// churns shadow textures.
fn paint_outline_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::ButtonOutline).layers();
        // Tailwind shadows have at most two layers; the stack buffer covers
        // every size in the scale.
        let mut faded = [ShadowLayer {
            offset: [0.0, 0.0],
            blur: 0.0,
            spread: 0.0,
            alpha: 0.0,
        }; 2];
        let n = layers.len().min(2);
        for (i, layer) in layers.iter().take(2).enumerate() {
            faded[i] = ShadowLayer {
                alpha: layer.alpha * 0.5,
                ..*layer
            };
        }
        paint_shadow_layers(painter, rect, corner, &faded[..n], dark);
    } else {
        paint_shadow_resolved(
            painter,
            rect,
            corner,
            ShadowSize::Xs,
            style,
            ShadowSlot::ButtonOutline,
            dark,
        );
    }
}

/// One icon slot: the icon plus an optional explicit size. An explicit size
/// wins over the size-derived default, mirroring an svg carrying its own
/// `size-*` class (frozen spec §5.8).
#[derive(Clone, Copy, Debug)]
struct IconSpec {
    icon: Icon,
    /// Explicit icon size in points, or `None` for the size-derived default.
    size: Option<f32>,
}

impl IconSpec {
    /// The painted/laid-out size: explicit, else the size-derived default.
    fn resolve(self, size: ButtonSize, style: Style) -> f32 {
        self.size.unwrap_or_else(|| metrics::icon_size(size, style))
    }
}

/// A shadcn button. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Button {
    label: String,
    variant: ButtonVariant,
    size: ButtonSize,
    icon_start: Option<IconSpec>,
    icon_end: Option<IconSpec>,
    disabled: bool,
    invalid: bool,
    expanded: bool,
    has_popup: bool,
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
            expanded: false,
            has_popup: false,
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
        self.icon_start = Some(IconSpec { icon, size: None });
        self
    }

    /// Shows an icon before the label with an explicit size, which wins over
    /// the size-derived default (an svg with its own `size-*` class).
    pub fn icon_start_sized(mut self, icon: Icon, size: f32) -> Self {
        self.icon_start = Some(IconSpec {
            icon,
            size: Some(size),
        });
        self
    }

    /// Shows an icon after the label (`has-[>svg]` slot).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(IconSpec { icon, size: None });
        self
    }

    /// Shows an icon after the label with an explicit size, which wins over
    /// the size-derived default (an svg with its own `size-*` class).
    pub fn icon_end_sized(mut self, icon: Icon, size: f32) -> Self {
        self.icon_end = Some(IconSpec {
            icon,
            size: Some(size),
        });
        self
    }

    /// Disables the button (`disabled:opacity-50 disabled:pointer-events-none`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the button aria-invalid (`aria-invalid:border-destructive
    /// aria-invalid:ring-destructive/20`). Under the nova style the
    /// invalid ring shows at rest as well as on focus (frozen spec §5.6);
    /// under other styles it shows on focus only. When a button is both
    /// invalid and focused, the invalid ring wins over the focus ring.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Marks the button expanded (`aria-expanded`), applying the nova
    /// expanded fills for the outline, secondary and ghost variants (frozen
    /// spec §5.7). Popup triggers set this while their popup is open.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Marks the button as a popup trigger (`aria-haspopup`), suppressing
    /// the nova pressed shift (frozen spec §5.3). Popup triggers set this
    /// so trigger buttons do not jump when pressed.
    pub fn has_popup(mut self, has_popup: bool) -> Self {
        self.has_popup = has_popup;
        self
    }

    /// Sets a stable id salt. The button keys its per-widget animation and
    /// keyboard state (transition legs, the Space keyup arm, the
    /// focus-lock filter) to its widget id, so a stable id keeps
    /// transitions continuous when the button moves in the layout; it also
    /// helps focus order and tooling.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The widget's intrinsic size for the current theme, before any
    /// [`crate::components::button_group::ButtonGroup`] adjustments.
    pub fn content_size(&self, ui: &Ui) -> Vec2 {
        self.content_layout(ui).0
    }

    /// The width of one flex row: padding + every icon + every gap + text
    /// (frozen spec §4). Pure, so the worked arithmetic is unit-testable
    /// without a `Ui`. Icon-only sizes are squares: the width is the height.
    fn content_width_for(
        size: ButtonSize,
        style: Style,
        text_w: f32,
        has_text: bool,
        icon_start_w: Option<f32>,
        icon_end_w: Option<f32>,
    ) -> f32 {
        if size.is_icon() {
            return metrics::height(size, style);
        }
        let has_start = icon_start_w.is_some();
        let has_end = icon_end_w.is_some();
        let icons_w = icon_start_w.unwrap_or(0.0) + icon_end_w.unwrap_or(0.0);
        let n_children = has_start as i32 + has_end as i32 + has_text as i32;
        let gaps = (n_children - 1).max(0) as f32 * metrics::gap(size, style);
        metrics::padding_left(size, style, has_start, has_end)
            + metrics::padding_right(size, style, has_start, has_end)
            + icons_w
            + gaps
            + text_w
    }

    /// Size plus the label galley, laid out once.
    ///
    /// The galley uses [`Color32::PLACEHOLDER`] (egui's delayed-color
    /// layout: the paint-time fallback color replaces it exactly), so
    /// [`Button::show`] can paint this same galley instead of laying the
    /// label out a second time with the resolved foreground color. The
    /// galley is `None` exactly when the label is empty.
    fn content_layout(&self, ui: &Ui) -> (Vec2, Option<std::sync::Arc<egui::Galley>>) {
        let style = ShadcnTheme::of_ui(ui).style;
        let size = self.size;
        // `font-medium`: button text is Inter Medium (500), spec §6.
        let font = crate::theme::fonts::medium(metrics::text_size(size, style));
        let galley = if self.label.is_empty() {
            None
        } else {
            Some(
                ui.painter()
                    .layout_no_wrap(self.label.clone(), font, Color32::PLACEHOLDER),
            )
        };
        let text_w = galley.as_ref().map_or(0.0, |g| g.size().x);
        let start_w = self.icon_start.map(|s| s.resolve(size, style));
        let end_w = self.icon_end.map(|s| s.resolve(size, style));
        (
            Vec2::new(
                Self::content_width_for(
                    size,
                    style,
                    text_w,
                    !self.label.is_empty(),
                    start_w,
                    end_w,
                ),
                metrics::height(size, style),
            ),
            galley,
        )
    }

    /// Renders the button and returns its [`Response`].
    ///
    /// Activation: [`Response::clicked`] is true on a pointer click, on
    /// Enter keydown, and on Space keyup — the native `<button>` timings
    /// (frozen spec §8). While Space is held the button shows the pressed
    /// state without firing; autorepeat never re-fires.
    ///
    /// Focus: clicking a button moves keyboard focus to it, so Space and
    /// Enter then activate it. The 3px focus ring shows whenever the
    /// button holds focus, regardless of how focus arrived (deviation D3
    /// from the browser's `:focus-visible` heuristic, which egui cannot
    /// reproduce). Escape and arrow keys are no-ops while a button holds
    /// focus; Tab and Shift+Tab move focus in layout order, skipping
    /// disabled buttons.
    ///
    /// Disabled buttons never take focus and never activate. Known
    /// limitation: clicking a disabled button clears focus held by
    /// another widget — an egui framework default (`SurrenderFocusOn`)
    /// that the library cannot scope per widget, so it differs from the
    /// native `pointer-events: none` behaviour.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let style = theme.style;
        let nova = style == Style::Nova;
        let p = theme.palette;
        let size = self.size;

        // Auto-detected position inside a ButtonGroup (see button_group.rs).
        // The `_ex` form also reports whether this child receives the nova
        // last-`[data-slot]` rule (FG-2).
        let group = group::next_child_position_ex(ui);
        let position = group.map(|(_, pos, _)| pos).unwrap_or(GroupPosition::Only);
        let orientation = group
            .map(|(o, _, _)| o)
            .unwrap_or(GroupOrientation::Horizontal);
        let receives_last_slot_rule = group.map(|(_, _, flag)| flag).unwrap_or(false);
        let in_group = position != GroupPosition::Only;
        // Buttons carry `data-slot="button"`, so every button in a group
        // registers its slot identity (FG-2) — after the position call, so
        // the registration lands on this child's index. No-op outside a
        // group.
        group::register_slot(ui, group::SlotIdentity::Button);

        let (mut desired, label_galley) = self.content_layout(ui);

        // FG-1: in a vertical group, a text button (no definite width in
        // either recipe) stretches to the group's measured cross size.
        // Icon buttons keep their fixed `size-*` width, and horizontal
        // targets never stretch buttons (definite heights). No target —
        // measure passes, standalone buttons — means current behaviour
        // exactly, and the in-group gate means a stale target can never
        // stretch a standalone button. The width never shrinks below the
        // natural width: the container cross size is by construction the
        // widest member, so `max` only guards a mismatched target.
        if group.is_some() && !size.is_icon() {
            if let Some(target) = group::stretch_target(ui) {
                if target.orientation == GroupOrientation::Vertical {
                    desired.x = desired.x.max(target.cross);
                }
            }
        }

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
            // Space fires on keyup, not keydown (spec §8, BTN-INT-04):
            // hide this frame's Space keydown from `interact` and
            // re-create it on the keyup frame. Only while focused — an
            // unfocused button must not consume another widget's Space.
            if ui.ctx().memory(|m| m.has_focus(id)) {
                space_keyup_fire(ui.ctx(), id);
            }
            ui.interact(rect, id, Sense::click())
        };

        // Native `<button>` behaviour (spec §8): a clicked button holds
        // keyboard focus afterwards, so Space/Enter then activate it.
        // egui's `interact` does not move focus on pointer press, so request
        // it explicitly on click — the pattern `DragValue`/`TextEdit` use.
        // The ring then shows per deviation D3 (any focus, regardless of how
        // it arrived). Requesting before painting lets `has_focus()` below
        // see the focus on the click frame itself. Disabled buttons never
        // take focus.
        if !self.disabled && response.clicked() {
            response.request_focus();
        }

        // Escape and arrow keys are no-ops on a focused button (spec §8,
        // BTN-INT-02/03): egui's default (`Focus::begin_pass`) clears focus
        // on unmodified Escape and moves focus spatially on unmodified
        // arrows, so claim both via the focus-lock filter while focused —
        // the pattern `accordion`/`tabs` use for arrows. Tab keeps the
        // built-in traversal. The claim is unconditional: the original
        // `button-group` is a presentational div with no roving tabindex,
        // so arrows are no-ops for group members too. Disabled buttons
        // never hold focus, so the gate is belt-and-braces.
        if !self.disabled && response.has_focus() {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    egui::EventFilter {
                        tab: false,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                    },
                );
            });
        }

        if ui.is_rect_visible(rect) {
            let radius = metrics::radius(size, style, &theme.radius, in_group);
            let mut corner = fuse_radius(CornerRadius::same(radius), orientation, position);
            // FG-2: the nova last-`[data-slot]` rule forces the trailing
            // corners to `rounded-lg!`, beating the fusing square corners
            // above. Nova-only; the base recipe has no such rule.
            if receives_last_slot_rule && nova {
                corner = force_last_slot_corners(corner, orientation, theme.radius.lg_u8());
            }

            // A disabled response is never hovered or focused, so both stay
            // false for disabled buttons.
            let hovered = response.hovered();
            let focused = response.has_focus();
            // `:active` while pressed: pointer held on the button, or Space /
            // Enter held while focused (native button behavior, spec §8).
            let pressed = response.is_pointer_button_down_on()
                || (focused
                    && ui.input(|i| i.key_down(egui::Key::Space) || i.key_down(egui::Key::Enter)));

            // `transition-all`: 150ms + cubic-bezier(0.4, 0, 0.2, 1), the
            // Tailwind v4 defaults (spec §7 / OPEN O1). Every progress is
            // driven on every visible frame so the first hover/focus/press
            // animates from the resting state instead of snapping; repaints
            // are requested only while a transition runs.
            let ctx = ui.ctx();
            let t_spec = TransitionSpec::default();
            let t_hover = transition_progress(ctx, id.with("hover"), hovered, t_spec).t;
            let t_expanded = transition_progress(ctx, id.with("expanded"), self.expanded, t_spec).t;
            // Endpoint-change animation (BTN-FLK-01, repaired BTN-FLK-02):
            // theme/style switches re-resolve the endpoint colours, so the
            // legs run on the endpoints (see `theme_animate_endpoint`) and
            // state mixing below blends leg values with the boundary-safe
            // `mix_color`. Hover/expanded fades therefore keep their
            // pre-FLK-01 shape — no OKLab leg ever chases a state fade.
            let rest = theme_animate_endpoint(
                ctx,
                id.with("ep_rest"),
                resolve_colors(self.variant, &p, dark, false, false, style),
                t_spec,
            );
            let hov = theme_animate_endpoint(
                ctx,
                id.with("ep_hov"),
                resolve_colors(self.variant, &p, dark, true, false, style),
                t_spec,
            );
            let exp = theme_animate_endpoint(
                ctx,
                id.with("ep_exp"),
                resolve_colors(self.variant, &p, dark, false, true, style),
                t_spec,
            );
            let hov_exp = theme_animate_endpoint(
                ctx,
                id.with("ep_hov_exp"),
                resolve_colors(self.variant, &p, dark, true, true, style),
                t_spec,
            );
            let mut c = mix_colors(
                mix_colors(rest, hov, t_hover),
                mix_colors(exp, hov_exp, t_hover),
                t_expanded,
            );
            // Underlines snap: text-decoration is not animatable in CSS.
            c.underline = self.variant == ButtonVariant::Link && hovered;

            // Disabled opacity fades with the rest (`transition-all` covers
            // opacity); the target is exact once settled.
            let opacity =
                transition_alpha(ctx, id.with("disabled"), 1.0, 0.5, self.disabled, t_spec).alpha;

            // Focus ring target: the invalid ring wins over the focus ring
            // (spec §5.4). The ring fades in/out with focus — and, under
            // nova, an invalid button shows its ring at rest too (spec
            // §5.6); the base recipe has no such rule.
            let ring_target = if self.invalid {
                color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
            } else {
                c.ring
            };
            // Endpoint-change animation for the ring target (BTN-FLK-01/02):
            // wrap the TARGET, before the alpha ramp below. The target only
            // changes on theme/style switches (the mixed ring colour is
            // hover/expanded independent), so the leg runs only then, and
            // the focus fade stays a pure alpha ramp instead of chasing
            // through OKLab from transparent.
            let ring_target =
                transition_color_to(ctx, id.with("theme_ring_target"), ring_target, t_spec).color;
            let ring_alpha = transition_alpha(
                ctx,
                id.with("ring"),
                0.0,
                1.0,
                ring_shown(focused, self.invalid, nova),
                t_spec,
            )
            .alpha;
            let ring_color = color_alpha(ring_target, ring_alpha * opacity);

            // Border color transitions between the resting stroke and the
            // focus/invalid stroke (widths never differ between the two:
            // `stroke_for` only recolors across focus/invalid). Both
            // endpoints are wrapped in the endpoint-change animator so a
            // theme switch animates the border in either state; the
            // endpoints are focus-independent values (the focus endpoint is
            // the colour the border WOULD have when focused, with the same
            // invalid-wins precedence), so the legs run only on theme/style
            // changes and the `t_border` mix below — boundary-safe
            // `mix_color` — owns the focus fade. Wrapping the
            // focus-dependent target instead would leg the border from
            // transparent on every focus change (BTN-FLK-02's mechanism).
            let rest_stroke = stroke_for(self.variant, &p, dark, style, false, false);
            let focus_stroke = stroke_for(self.variant, &p, dark, style, true, self.invalid);
            let rest_stroke_color =
                transition_color_to(ctx, id.with("theme_border_rest"), rest_stroke.color, t_spec)
                    .color;
            let focus_stroke_color = transition_color_to(
                ctx,
                id.with("theme_border_focus"),
                focus_stroke.color,
                t_spec,
            )
            .color;
            let t_border =
                transition_progress(ctx, id.with("border"), focused || self.invalid, t_spec).t;
            c.stroke = Stroke::new(
                focus_stroke.width,
                mix_color(rest_stroke_color, focus_stroke_color, t_border),
            );
            // `disabled:opacity-50` — fade everything against the backdrop.
            c = fade_colors(c, opacity);

            // `active:translate-y-px` (nova): the whole button — shadow,
            // body, ring — shifts 1px down while pressed, animated over the
            // transition duration (spec §5.3/§7, D7). Suppressed for popup
            // triggers (`haspopup` exclusion) and disabled buttons.
            let shift = if active_shift_applies(style, self.has_popup, self.disabled) {
                transition_progress(ctx, id.with("press"), pressed, t_spec).t
            } else {
                0.0
            };
            let render_rect = rect.translate(Vec2::new(0.0, shift));

            let painter = ui.painter();

            // `shadow-xs` on the outline variant: the base recipe declares
            // it and no style's `.cn-button-variant-outline` rule overrides
            // it, so it paints under every style (Phase 1 §5(b) LAYER
            // ruling).
            if self.variant == ButtonVariant::Outline {
                paint_outline_shadow(painter, render_rect, corner, style, dark, self.disabled);
            }

            // Non-first children of a group overlap the previous border so the
            // shared edge shows one 1px divider (`border-l-0`).
            let draw_rect = match orientation {
                GroupOrientation::Horizontal
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(render_rect.min + Vec2::new(1.0, 0.0), render_rect.max)
                }
                GroupOrientation::Vertical
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(render_rect.min + Vec2::new(0.0, 1.0), render_rect.max)
                }
                _ => render_rect,
            };

            // Fill: under nova, `bg-clip-padding` clips the background to
            // the padding box (spec D5) — inset 1px from the border box
            // except on the shared group edge, which has no border. The
            // base recipe clips to the border box, so the fill covers the
            // whole rect.
            let (fill_rect, fill_corner) = if nova {
                let mut fill_rect = draw_rect;
                let shared_left = matches!(orientation, GroupOrientation::Horizontal)
                    && matches!(position, GroupPosition::Middle | GroupPosition::Last);
                let shared_top = matches!(orientation, GroupOrientation::Vertical)
                    && matches!(position, GroupPosition::Middle | GroupPosition::Last);
                if !shared_left {
                    fill_rect.min.x += 1.0;
                }
                if !shared_top {
                    fill_rect.min.y += 1.0;
                }
                fill_rect.max.x -= 1.0;
                fill_rect.max.y -= 1.0;
                (fill_rect, shrink_corner(corner))
            } else {
                (draw_rect, corner)
            };
            painter.rect_filled(fill_rect, fill_corner, c.fill);

            if c.stroke.width > 0.0 {
                painter.rect_stroke(
                    draw_rect,
                    corner,
                    c.stroke,
                    egui::epaint::StrokeKind::Inside,
                );
            }

            // Content: [icon] gap [label] gap [icon], centered. Each icon
            // paints at its own resolved size (explicit size wins, §5.8);
            // the label galley from `content_layout` (laid out once) gives
            // the paint width.
            let gap = metrics::gap(size, style);
            let start = self.icon_start.map(|s| (s.icon, s.resolve(size, style)));
            let end = self.icon_end.map(|s| (s.icon, s.resolve(size, style)));
            let text_w = label_galley.as_ref().map_or(0.0, |g| g.size().x);
            let icons_w = start.map_or(0.0, |(_, w)| w) + end.map_or(0.0, |(_, w)| w);
            let n_children =
                start.is_some() as i32 + end.is_some() as i32 + (!self.label.is_empty()) as i32;
            let gaps = (n_children - 1).max(0) as f32 * gap;
            let total_w = icons_w + gaps + text_w;

            let center = draw_rect.center();
            let mut cursor = center.x - total_w * 0.5;

            if let Some((icon, icon_size)) = start {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, c.fg, 2.0 * icon_size / 24.0);
                cursor += icon_size + gap;
            }

            if !self.label.is_empty() {
                let text_center = Pos2::new(cursor + text_w * 0.5, center.y);
                if let Some(galley) = label_galley {
                    // Same paint math as `Painter::text`: anchor the galley
                    // at the anchor point, paint with the resolved color as
                    // the fallback that replaces the placeholder color.
                    let rect = Align2::CENTER_CENTER.anchor_size(text_center, galley.size());
                    painter.galley(rect.min, galley, c.fg);
                }
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

            if let Some((icon, icon_size)) = end {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, c.fg, 2.0 * icon_size / 24.0);
            }

            // `focus-visible:border-ring focus-visible:ring-[3px]
            // focus-visible:ring-ring/50` — 3px ring just outside the
            // widget, following the fused group corners (spec §5.4).
            // Painted while the fade has any alpha, so losing focus fades
            // the ring out instead of popping it. The stroke is drawn
            // outside the border box itself (no expansion): a CSS
            // box-shadow ring is adjacent to the border box, with no gap
            // row between ring and border (BTN-VIS-02).
            //
            // FG-3: inside a group the ring is deferred to `group::end`,
            // which paints it into a foreground layer above the following
            // siblings (`z-10`, button-group spec §4). Standalone buttons
            // paint inline, exactly as before.
            if ring_color.a() > 0 {
                if group.is_some() {
                    group::queue_ring(ui, render_rect, corner, Stroke::new(3.0, ring_color));
                } else {
                    painter.rect_stroke(
                        render_rect,
                        corner,
                        Stroke::new(3.0, ring_color),
                        egui::epaint::StrokeKind::Outside,
                    );
                }
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

/// Applies the nova last-`[data-slot]` rule (button-group spec §3.2, FG-2):
/// the last slot-carrying child's TRAILING corners are forced to
/// `rounded-lg` (`lg`, 10px at the snapshot radius) with `!` priority, so
/// the rule beats the fusing square-corner rule that also applies to a
/// non-last child. Leading corners are untouched (specified silence: no
/// first-child rule exists). The caller gates this on the nova style —
/// the base recipe has no such rule — and on the
/// [`group::next_child_position_ex`] flag.
pub(crate) fn force_last_slot_corners(
    mut corner: CornerRadius,
    o: GroupOrientation,
    lg: u8,
) -> CornerRadius {
    match o {
        GroupOrientation::Horizontal => {
            corner.ne = lg;
            corner.se = lg;
        }
        GroupOrientation::Vertical => {
            corner.sw = lg;
            corner.se = lg;
        }
    }
    corner
}

/// Shared layout plumbing used by
/// [`crate::components::button_group::ButtonGroup`] to tell grouped widgets
/// their position, so borders and corners fuse like the source's
/// `[&>*:not(:first-child)]:rounded-l-none` selectors.
///
/// The module also carries the three group↔member channels the frozen
/// button-group spec needs but a member cannot provide from outside
/// (Foundation Loop FG-1/2/3, 2026-11-16):
///
/// * **FG-1 stretch target** ([`group::StretchTarget`]): the group's
///   measured cross size, pushed for the real pass and read by members
///   whose cross size is `auto` (text buttons in vertical groups).
/// * **FG-2 slot identity** ([`group::SlotIdentity`]): children register
///   the `data-slot` they carry; the group computes which child index is
///   the last slot-carrying child and flags it for the nova
///   last-`[data-slot]` rule (spec §3.2).
/// * **FG-3 deferred focus rings**: a focused member's ring is queued
///   during the child pass and flushed by [`group::end`] into a
///   foreground layer, so it paints above following siblings (spec §4,
///   `z-10`).
pub(crate) mod group {
    use super::{GroupOrientation, GroupPosition};
    use egui::{Context, CornerRadius, Id, LayerId, Order, Rect, Stroke, Ui};

    #[derive(Clone, Debug)]
    struct GroupState {
        orientation: GroupOrientation,
        frame: u64,
        drawn: usize,
        total: Option<usize>,
        /// Slot identities registered by children during the current pass,
        /// in child order: (child index, identity). Reset by [`begin`],
        /// finalized by [`end`] into `last_slot_child`.
        slots: Vec<(usize, SlotIdentity)>,
        /// Child index of the last slot-carrying child, as computed by
        /// [`end`] from the just-finished pass — or as set explicitly by
        /// [`set_last_slot_child`] (the measure→real transfer). Read by
        /// the next pass to flag the child receiving the nova
        /// last-`[data-slot]` rule. `None` until a pass completes.
        last_slot_child: Option<usize>,
    }

    /// A `data-slot` identity a group child carries (button-group spec
    /// §3.2). Only the PRESENCE of a slot matters for the nova last-child
    /// rule — the selector is `[data-slot]` — but the variant records
    /// which slot, so the Batch F select-trigger interface (§13) can tell
    /// its members apart later. Children with no `data-slot` (plain
    /// `ButtonGroupText`, arbitrary elements) register nothing.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(crate) enum SlotIdentity {
        /// `data-slot="button"`.
        Button,
        /// `data-slot="button-group"` (a nested group, as a direct child).
        /// First constructed by the S1b wiring step in `button_group.rs`;
        /// exercised by tests meanwhile.
        #[allow(dead_code)]
        ButtonGroup,
        /// `data-slot="button-group-separator"`.
        /// First constructed by the S1b wiring step in `button_group.rs`;
        /// exercised by tests meanwhile.
        #[allow(dead_code)]
        ButtonGroupSeparator,
        /// `data-slot="select-trigger"` (Batch F interface).
        /// Not constructed until Batch F wires select triggers; the
        /// `allow` keeps the declared interface warning-free meanwhile.
        #[allow(dead_code)]
        SelectTrigger,
    }

    /// The same-frame `items-stretch` target for one group's real pass
    /// (button-group spec §5): the group's orientation plus the measured
    /// cross size (tallest member in a horizontal group, widest in a
    /// vertical group). Pushed immediately before the real pass and popped
    /// immediately after — it never survives the frame, so no stale
    /// maximum can pin a shrunken group open. `None` in measure passes
    /// (natural sizes) and outside any group (standalone sizing).
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct StretchTarget {
        /// The group's orientation; the cross size refers to the axis
        /// perpendicular to it (height in horizontal groups, width in
        /// vertical groups).
        pub(crate) orientation: GroupOrientation,
        /// The measured cross size in points.
        pub(crate) cross: f32,
    }

    /// A focus ring deferred to the end of its group (spec §4, D2).
    #[derive(Clone, Copy, Debug)]
    struct DeferredRing {
        rect: Rect,
        corner: CornerRadius,
        stroke: Stroke,
        clip_rect: Rect,
    }

    fn states_id() -> Id {
        Id::unique("shadcn_group_states")
    }

    fn stack_id() -> Id {
        Id::unique("shadcn_group_stack")
    }

    fn stretch_stack_id() -> Id {
        Id::unique("shadcn_group_stretch_stack")
    }

    fn rings_id() -> Id {
        Id::unique("shadcn_group_deferred_rings")
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
            slots: Vec::new(),
            last_slot_child: None,
        });
        if state.frame != frame {
            // New frame: keep last frame's `total` for position math.
            state.frame = frame;
            state.drawn = 0;
            state.orientation = orientation;
        }
        // A new pass starts a fresh slot-registration list. `last_slot_child`
        // is deliberately kept: it is the previous pass's result (or the
        // measure→real transfer) and is what this pass's children read.
        state.slots.clear();
        write_state(&ctx, id, Some(state));
        let mut stack: Vec<Id> = ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
        stack.push(id);
        ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
    }

    /// The group finished; call after the child closure. Records the child
    /// count so the next frame can compute First/Middle/Last, finalizes the
    /// last-slot-child index from this pass's slot registrations, and
    /// flushes any focus rings deferred by this group's children (FG-3).
    pub(crate) fn end(ui: &Ui, id: Id) {
        let ctx = ui.ctx().clone();
        if let Some(mut state) = read_state(&ctx, id) {
            state.total = Some(state.drawn);
            // Registrations arrive in child order, so the last one is the
            // last slot-carrying child — the `[data-slot]` selector's
            // target (spec §3.2). An empty list means no child carries a
            // slot: no child receives the rule.
            state.last_slot_child = state.slots.last().map(|(index, _)| *index);
            write_state(&ctx, id, Some(state));
        }
        let mut stack: Vec<Id> = ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
        stack.pop();
        ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
        flush_deferred_rings(&ctx, id);
    }

    /// Called by widgets that participate in a group. Returns the group
    /// orientation and this widget's position, or `None` outside a group.
    /// Call at most once per widget per frame, before painting.
    pub fn next_child_position(ui: &Ui) -> Option<(GroupOrientation, GroupPosition)> {
        next_child_position_ex(ui).map(|(o, pos, _)| (o, pos))
    }

    /// Like [`next_child_position`], plus whether this child receives the
    /// nova last-`[data-slot]` rule (button-group spec §3.2): true exactly
    /// when this child's index is the last slot-carrying child's index, as
    /// computed by the previous pass or set by [`set_last_slot_child`].
    /// The flag is style-independent — the caller applies the nova gate —
    /// and is only ever true inside a group.
    pub(crate) fn next_child_position_ex(
        ui: &Ui,
    ) -> Option<(GroupOrientation, GroupPosition, bool)> {
        let ctx = ui.ctx().clone();
        let id: Id = *ctx
            .data_mut(|d| d.get_temp::<Vec<Id>>(stack_id()))?
            .last()?;
        let mut state = read_state(&ctx, id)?;
        if state.frame != ctx.cumulative_frame_nr() {
            return None;
        }
        let total = state.total;
        let orientation = state.orientation;
        let index = state.drawn;
        state.drawn += 1;
        let receives_last_slot_rule = state.last_slot_child == Some(index);
        write_state(&ctx, id, Some(state));
        let position = match (index, total) {
            (_, Some(1)) => GroupPosition::Only,
            (0, _) => GroupPosition::First,
            (_, Some(n)) if index + 1 == n => GroupPosition::Last,
            _ => GroupPosition::Middle,
        };
        Some((orientation, position, receives_last_slot_rule))
    }

    /// Registers the calling child's `data-slot` identity with the
    /// innermost group, for the nova last-`[data-slot]` rule (FG-2).
    ///
    /// Call exactly once per slot-carrying child per pass, AFTER
    /// [`next_child_position`] / [`next_child_position_ex`] in the same
    /// `show` — the registration is attributed to the child index that
    /// call just consumed. Children with no `data-slot` (plain
    /// `ButtonGroupText`, arbitrary elements) must NOT call this: their
    /// absence from the list is what makes the selector semantics work
    /// (a trailing non-slot child does not prevent the last slot child
    /// from receiving the rule). No-op outside a group.
    ///
    /// A nested group registers [`SlotIdentity::ButtonGroup`] with its
    /// PARENT group: call this while the parent group is still the stack
    /// top, i.e. before the nested group's own [`begin`].
    pub(crate) fn register_slot(ui: &Ui, identity: SlotIdentity) {
        let ctx = ui.ctx().clone();
        let Some(id) = ctx
            .data_mut(|d| d.get_temp::<Vec<Id>>(stack_id()))
            .and_then(|stack| stack.last().copied())
        else {
            return;
        };
        let Some(mut state) = read_state(&ctx, id) else {
            return;
        };
        if state.frame != ctx.cumulative_frame_nr() {
            return;
        }
        // `drawn` was incremented by the position call; the child index is
        // one less. A registration with no consumed position is ignored.
        let Some(index) = state.drawn.checked_sub(1) else {
            return;
        };
        state.slots.push((index, identity));
        write_state(&ctx, id, Some(state));
    }

    /// The child index of the last slot-carrying child for the group `id`,
    /// as computed by [`end`] from the most recent completed pass under
    /// that id. `None` if no pass has completed or no child registered a
    /// slot. This is the measure-pass result the group transfers to the
    /// real pass via [`set_last_slot_child`].
    ///
    /// First called by the S1b wiring step in `button_group.rs`;
    /// exercised by tests meanwhile.
    #[allow(dead_code)]
    pub(crate) fn last_slot_child(ui: &Ui, id: Id) -> Option<usize> {
        read_state(ui.ctx(), id).and_then(|state| state.last_slot_child)
    }

    /// Sets the last-slot-child index the upcoming pass under `id` will
    /// flag (FG-2 measure→real transfer). Call after [`begin`] and before
    /// the child closure, with the value read from the measure pass's id
    /// via [`last_slot_child`]. The value is consumed by that pass's
    /// [`next_child_position_ex`] calls; [`end`] then recomputes it from
    /// the pass's own registrations (the same value, when the child list
    /// is unchanged). Pass `None` when no child carries a slot.
    ///
    /// First called by the S1b wiring step in `button_group.rs`;
    /// exercised by tests meanwhile.
    #[allow(dead_code)]
    pub(crate) fn set_last_slot_child(ui: &Ui, id: Id, index: Option<usize>) {
        let ctx = ui.ctx().clone();
        let mut state = read_state(&ctx, id).unwrap_or(GroupState {
            orientation: GroupOrientation::Horizontal,
            frame: ctx.cumulative_frame_nr(),
            drawn: 0,
            total: None,
            slots: Vec::new(),
            last_slot_child: None,
        });
        state.last_slot_child = index;
        write_state(&ctx, id, Some(state));
    }

    /// Pushes a stretch target for the innermost group's real pass (FG-1).
    /// The group pushes immediately before the real-pass closure and pops
    /// immediately after ([`pop_stretch_target`]); the stack discipline
    /// nests correctly for nested groups. Never push during a measure
    /// pass: members must measure their natural sizes there.
    ///
    /// First called by the S1b wiring step in `button_group.rs`;
    /// exercised by tests meanwhile.
    #[allow(dead_code)]
    pub(crate) fn push_stretch_target(ctx: &Context, target: StretchTarget) {
        ctx.data_mut(|d| {
            let mut stack: Vec<StretchTarget> = d.get_temp(stretch_stack_id()).unwrap_or_default();
            stack.push(target);
            d.insert_temp(stretch_stack_id(), stack);
        });
    }

    /// Pops the stretch target pushed by [`push_stretch_target`]. Every
    /// push must be balanced by exactly one pop in the same frame.
    ///
    /// First called by the S1b wiring step in `button_group.rs`;
    /// exercised by tests meanwhile.
    #[allow(dead_code)]
    pub(crate) fn pop_stretch_target(ctx: &Context) {
        ctx.data_mut(|d| {
            let mut stack: Vec<StretchTarget> = d.get_temp(stretch_stack_id()).unwrap_or_default();
            stack.pop();
            d.insert_temp(stretch_stack_id(), stack);
        });
    }

    /// The innermost active stretch target, if the caller is inside a
    /// group's real pass. `None` in measure passes and outside any group.
    ///
    /// **Batch B interface (input):** a direct-child input in a vertical
    /// group stretches to the widest member's width unless its own recipe
    /// declares a width (button-group spec §5). The input component reads
    /// this channel the same way [`Button::show`](super::Button::show)
    /// does: when the target's orientation is vertical, a width-`auto`
    /// input takes `target.cross` as its width (never shrinking below its
    /// natural width). Horizontal targets never stretch inputs (definite
    /// `h-*` heights, like buttons).
    pub(crate) fn stretch_target(ui: &Ui) -> Option<StretchTarget> {
        ui.ctx().data_mut(|d| {
            d.get_temp::<Vec<StretchTarget>>(stretch_stack_id())
                .and_then(|stack| stack.last().copied())
        })
    }

    /// Queues a focus ring to be painted when the innermost group ends
    /// (FG-3). The ring paints into a foreground layer, above the
    /// following siblings it abuts — the `z-10` rule (spec §4). No-op
    /// outside a group (the caller paints inline instead). The clip rect
    /// is captured from the calling `Ui`, so a ring inside a scrolled
    /// region stays clipped exactly as an inline paint would be.
    pub(crate) fn queue_ring(ui: &Ui, rect: Rect, corner: CornerRadius, stroke: Stroke) {
        let ctx = ui.ctx().clone();
        let Some(id) = ctx
            .data_mut(|d| d.get_temp::<Vec<Id>>(stack_id()))
            .and_then(|stack| stack.last().copied())
        else {
            return;
        };
        // Only queue for a live group pass; a stale frame means `begin`
        // was never called this frame and no `end` will flush the queue.
        // The frame number is captured BEFORE the `data_mut` below: any
        // `Context` call inside that closure would re-lock the context
        // while the write lock is held and deadlock (egui 0.37).
        let frame = ctx.cumulative_frame_nr();
        let live = read_state(&ctx, id).is_some_and(|state| state.frame == frame);
        if !live {
            return;
        }
        let ring = DeferredRing {
            rect,
            corner,
            stroke,
            clip_rect: ui.clip_rect(),
        };
        ctx.data_mut(|d| {
            let mut rings: Vec<(Id, u64, DeferredRing)> =
                d.get_temp(rings_id()).unwrap_or_default();
            rings.push((id, frame, ring));
            d.insert_temp(rings_id(), rings);
        });
    }

    /// Paints this group's queued rings from the current frame into a
    /// foreground layer, then drops every queued ring for this group
    /// (flushed or stale). Unflushed rings from other groups and older
    /// frames are pruned too, so a group that never calls `end` cannot
    /// leak queue entries across frames.
    ///
    /// `Order::Foreground` is load-bearing: within one `Order`, egui
    /// paints area-registered layers in area order and every other layer
    /// in hash-map order (egui 0.37 `GraphicLayers::drain`), so a
    /// `Middle` ring layer could land UNDER the siblings it must cover.
    /// `Foreground` paints after all `Middle` content by construction,
    /// while tooltips (`Order::Tooltip`) still paint above the rings.
    /// The layer is never registered as an `Area`, so it stays
    /// transparent to hit-testing.
    fn flush_deferred_rings(ctx: &Context, id: Id) {
        let frame = ctx.cumulative_frame_nr();
        let rings: Vec<(Id, u64, DeferredRing)> = ctx.data_mut(|d| {
            let rings: Vec<(Id, u64, DeferredRing)> = d.get_temp(rings_id()).unwrap_or_default();
            // Keep only other groups' current-frame rings; everything for
            // this group is consumed below, stale entries are dropped.
            let kept: Vec<(Id, u64, DeferredRing)> = rings
                .iter()
                .filter(|(gid, f, _)| *gid != id && *f == frame)
                .copied()
                .collect();
            d.insert_temp(rings_id(), kept);
            rings
        });
        let layer_id = LayerId::new(Order::Foreground, id.with("shadcn_group_ring"));
        for (gid, f, ring) in rings {
            if gid == id && f == frame {
                let painter = ctx.layer_painter(layer_id).with_clip_rect(ring.clip_rect);
                painter.rect_stroke(
                    ring.rect,
                    ring.corner,
                    ring.stroke,
                    egui::epaint::StrokeKind::Outside,
                );
            }
        }
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
        let c = resolve_colors(ButtonVariant::Default, &p, false, false, false, BASE_STYLE);
        assert_eq!(c.fill, p.primary);
        assert_eq!(c.fg, p.primary_foreground);
        let h = resolve_colors(ButtonVariant::Default, &p, false, true, false, BASE_STYLE);
        assert_eq!(h.fill, color_alpha(p.primary, 0.9));
    }

    #[test]
    fn nova_default_hovers_primary_80() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Default, &p, false, false, false, Style::Nova);
        assert_eq!(c.fill, p.primary);
        assert_eq!(c.fg, p.primary_foreground);
        let h = resolve_colors(ButtonVariant::Default, &p, false, true, false, Style::Nova);
        assert_eq!(h.fill, color_alpha(p.primary, 0.8));
    }

    #[test]
    fn destructive_uses_white_text_and_tinted_dark_fill() {
        let p = palette(Mode::Light);
        let c = resolve_colors(
            ButtonVariant::Destructive,
            &p,
            false,
            false,
            false,
            BASE_STYLE,
        );
        assert_eq!(c.fill, p.destructive);
        assert_eq!(c.fg, Color32::WHITE);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(
            ButtonVariant::Destructive,
            &dark,
            true,
            false,
            false,
            BASE_STYLE,
        );
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.6));
        let dh = resolve_colors(
            ButtonVariant::Destructive,
            &dark,
            true,
            true,
            false,
            BASE_STYLE,
        );
        assert_eq!(dh.fill, color_alpha(dark.destructive, 0.9));
    }

    #[test]
    fn nova_destructive_is_tinted_with_destructive_text() {
        let p = palette(Mode::Light);
        let c = resolve_colors(
            ButtonVariant::Destructive,
            &p,
            false,
            false,
            false,
            Style::Nova,
        );
        assert_eq!(c.fill, color_alpha(p.destructive, 0.1));
        assert_eq!(c.fg, p.destructive);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
        let h = resolve_colors(
            ButtonVariant::Destructive,
            &p,
            false,
            true,
            false,
            Style::Nova,
        );
        assert_eq!(h.fill, color_alpha(p.destructive, 0.2));
        let dark = palette(Mode::Dark);
        let d = resolve_colors(
            ButtonVariant::Destructive,
            &dark,
            true,
            false,
            false,
            Style::Nova,
        );
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.2));
        assert_eq!(d.fg, dark.destructive);
        assert_eq!(d.ring, color_alpha(dark.destructive, 0.4));
        let dh = resolve_colors(
            ButtonVariant::Destructive,
            &dark,
            true,
            true,
            false,
            Style::Nova,
        );
        assert_eq!(dh.fill, color_alpha(dark.destructive, 0.3));
    }

    #[test]
    fn outline_matches_source_states() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Outline, &p, false, false, false, BASE_STYLE);
        assert_eq!(c.fill, p.background);
        assert_eq!(c.stroke.color, p.border);
        let h = resolve_colors(ButtonVariant::Outline, &p, false, true, false, BASE_STYLE);
        assert_eq!(h.fill, p.accent);
        assert_eq!(h.fg, p.accent_foreground);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(
            ButtonVariant::Outline,
            &dark,
            true,
            false,
            false,
            BASE_STYLE,
        );
        assert_eq!(d.fill, color_alpha(dark.input, 0.3));
        assert_eq!(d.stroke.color, dark.input);
        let dh = resolve_colors(ButtonVariant::Outline, &dark, true, true, false, BASE_STYLE);
        assert_eq!(dh.fill, color_alpha(dark.input, 0.5));
    }

    #[test]
    fn nova_outline_hovers_muted_with_foreground_text() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Outline, &p, false, false, false, Style::Nova);
        assert_eq!(c.fill, p.background);
        assert_eq!(c.stroke.color, p.border);
        assert_eq!(c.fg, p.foreground);
        let h = resolve_colors(ButtonVariant::Outline, &p, false, true, false, Style::Nova);
        assert_eq!(h.fill, p.muted);
        assert_eq!(h.fg, p.foreground);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(
            ButtonVariant::Outline,
            &dark,
            true,
            false,
            false,
            Style::Nova,
        );
        assert_eq!(d.fill, color_alpha(dark.input, 0.3));
        assert_eq!(d.stroke.color, dark.input);
        let dh = resolve_colors(
            ButtonVariant::Outline,
            &dark,
            true,
            true,
            false,
            Style::Nova,
        );
        assert_eq!(dh.fill, color_alpha(dark.input, 0.5));
        assert_eq!(dh.fg, dark.foreground);
    }

    #[test]
    fn secondary_hover_is_80_and_ghost_hover_uses_accent() {
        let p = palette(Mode::Light);
        let h = resolve_colors(ButtonVariant::Secondary, &p, false, true, false, BASE_STYLE);
        assert_eq!(h.fill, color_alpha(p.secondary, 0.8));
        let g = resolve_colors(ButtonVariant::Ghost, &p, false, true, false, BASE_STYLE);
        assert_eq!(g.fill, p.accent);
        assert_eq!(g.fg, p.accent_foreground);
        let dark = palette(Mode::Dark);
        let gd = resolve_colors(ButtonVariant::Ghost, &dark, true, true, false, BASE_STYLE);
        assert_eq!(gd.fill, color_alpha(dark.accent, 0.5));
        let g0 = resolve_colors(ButtonVariant::Ghost, &dark, true, false, false, BASE_STYLE);
        assert_eq!(g0.fill, Color32::TRANSPARENT);
        assert_eq!(g0.fg, dark.foreground);
    }

    #[test]
    fn nova_secondary_and_ghost_hover_muted() {
        let p = palette(Mode::Light);
        // color-mix(in oklch, var(--secondary), var(--foreground) 5%)
        let h = resolve_colors(
            ButtonVariant::Secondary,
            &p,
            false,
            true,
            false,
            Style::Nova,
        );
        assert_eq!(h.fill, blend(p.secondary, color_alpha(p.foreground, 0.05)));
        let s = resolve_colors(
            ButtonVariant::Secondary,
            &p,
            false,
            false,
            false,
            Style::Nova,
        );
        assert_eq!(s.fill, p.secondary);
        // ghost: hover:bg-muted hover:text-foreground dark:hover:bg-muted/50
        let g = resolve_colors(ButtonVariant::Ghost, &p, false, true, false, Style::Nova);
        assert_eq!(g.fill, p.muted);
        assert_eq!(g.fg, p.foreground);
        let dark = palette(Mode::Dark);
        let gd = resolve_colors(ButtonVariant::Ghost, &dark, true, true, false, Style::Nova);
        assert_eq!(gd.fill, color_alpha(dark.muted, 0.5));
        assert_eq!(gd.fg, dark.foreground);
        let g0 = resolve_colors(ButtonVariant::Ghost, &dark, true, false, false, Style::Nova);
        assert_eq!(g0.fill, Color32::TRANSPARENT);
        assert_eq!(g0.fg, dark.foreground);
    }

    #[test]
    fn link_underlines_on_hover() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Link, &p, false, false, false, BASE_STYLE);
        assert_eq!(c.fg, p.primary);
        assert!(!c.underline);
        let h = resolve_colors(ButtonVariant::Link, &p, false, true, false, BASE_STYLE);
        assert!(h.underline);
    }

    #[test]
    fn two_icon_width_counts_every_icon_and_gap() {
        // Frozen spec §4 worked arithmetic, with a 100px label. NOTE: the
        // spec's base "2 icons, no label" total reads 60, but its own terms
        // sum to 64 (12 + 16 + 8 + 16 + 12); the formula wins and 64 is
        // asserted here (spec erratum, reported to the Coordinator).
        let text_w = 100.0;
        // Base default: pad 16/16 no icon, 12/12 with ≥1 icon; icon 16, gap 8.
        assert_eq!(
            Button::content_width_for(ButtonSize::Default, BASE_STYLE, text_w, true, None, None),
            132.0
        );
        assert_eq!(
            Button::content_width_for(
                ButtonSize::Default,
                BASE_STYLE,
                text_w,
                true,
                Some(16.0),
                None
            ),
            148.0
        );
        assert_eq!(
            Button::content_width_for(
                ButtonSize::Default,
                BASE_STYLE,
                text_w,
                true,
                Some(16.0),
                Some(16.0)
            ),
            172.0
        );
        assert_eq!(
            Button::content_width_for(
                ButtonSize::Default,
                BASE_STYLE,
                0.0,
                false,
                Some(16.0),
                Some(16.0)
            ),
            64.0
        );
        // Nova default: pad 10/10, icon-side 8; icon 16, gap 6.
        let nova = Style::Nova;
        assert_eq!(
            Button::content_width_for(ButtonSize::Default, nova, text_w, true, None, None),
            120.0
        );
        assert_eq!(
            Button::content_width_for(ButtonSize::Default, nova, text_w, true, Some(16.0), None),
            140.0
        );
        assert_eq!(
            Button::content_width_for(
                ButtonSize::Default,
                nova,
                text_w,
                true,
                Some(16.0),
                Some(16.0)
            ),
            160.0
        );
        // Nova xs: pad 8/6, icon 12, gap 4 → +16/+30/+44.
        assert_eq!(
            Button::content_width_for(ButtonSize::Xs, nova, text_w, true, None, None),
            116.0
        );
        assert_eq!(
            Button::content_width_for(ButtonSize::Xs, nova, text_w, true, Some(12.0), None),
            130.0
        );
        assert_eq!(
            Button::content_width_for(ButtonSize::Xs, nova, text_w, true, Some(12.0), Some(12.0)),
            144.0
        );
        // Nova sm: pad 10/6, icon 14, gap 4 → +20/+34/+48.
        assert_eq!(
            Button::content_width_for(ButtonSize::Sm, nova, text_w, true, None, None),
            120.0
        );
        assert_eq!(
            Button::content_width_for(ButtonSize::Sm, nova, text_w, true, Some(14.0), None),
            134.0
        );
        assert_eq!(
            Button::content_width_for(ButtonSize::Sm, nova, text_w, true, Some(14.0), Some(14.0)),
            148.0
        );
        // Nova lg matches nova default.
        assert_eq!(
            Button::content_width_for(ButtonSize::Lg, nova, text_w, true, Some(16.0), Some(16.0)),
            160.0
        );
        // Icon sizes are squares regardless of content.
        assert_eq!(
            Button::content_width_for(ButtonSize::Icon, nova, text_w, true, Some(16.0), None),
            32.0
        );
        assert_eq!(
            Button::content_width_for(ButtonSize::IconSm, BASE_STYLE, text_w, true, None, None),
            32.0
        );
    }

    #[test]
    fn explicit_icon_size_wins_over_default() {
        // Spec §5.8: an svg with its own `size-*` class keeps its size.
        let spec = IconSpec {
            icon: Icon::Plus,
            size: Some(20.0),
        };
        assert_eq!(spec.resolve(ButtonSize::Default, Style::Nova), 20.0);
        assert_eq!(spec.resolve(ButtonSize::Xs, BASE_STYLE), 20.0);
        let dflt = IconSpec {
            icon: Icon::Plus,
            size: None,
        };
        assert_eq!(dflt.resolve(ButtonSize::Default, Style::Nova), 16.0);
        assert_eq!(dflt.resolve(ButtonSize::Sm, Style::Nova), 14.0);
        assert_eq!(dflt.resolve(ButtonSize::Xs, BASE_STYLE), 12.0);
        // The width arithmetic uses the explicit size.
        assert_eq!(
            Button::content_width_for(
                ButtonSize::Default,
                Style::Nova,
                100.0,
                true,
                Some(20.0),
                None
            ),
            144.0 // 8 + 20 + 6 + 100 + 10
        );
    }

    #[test]
    fn base_recipe_gaps_and_icons_match_source() {
        // gap-2 default/lg, gap-1.5 sm, gap-1 xs; icons size-4, size-3 xs.
        assert_eq!(ButtonSize::Default.gap(), 8.0);
        assert_eq!(ButtonSize::Lg.gap(), 8.0);
        assert_eq!(ButtonSize::Sm.gap(), 6.0);
        assert_eq!(ButtonSize::Xs.gap(), 4.0);
        assert_eq!(ButtonSize::Sm.icon_size(), 16.0);
        assert_eq!(ButtonSize::Lg.icon_size(), 16.0);
        assert_eq!(ButtonSize::IconSm.icon_size(), 16.0);
        assert_eq!(ButtonSize::IconXs.icon_size(), 12.0);
        assert_eq!(metrics::text_size(ButtonSize::Sm, BASE_STYLE), 14.0);
        assert_eq!(metrics::text_size(ButtonSize::Lg, BASE_STYLE), 14.0);
        assert_eq!(metrics::icon_size(ButtonSize::IconSm, Style::Nova), 14.0);
        assert_eq!(metrics::icon_size(ButtonSize::Icon, Style::Nova), 16.0);
    }

    #[test]
    fn expanded_fills_match_nova_rules() {
        // Spec §5.7: nova aria-expanded fills for outline/secondary/ghost;
        // the base recipe declares none.
        let p = palette(Mode::Light);
        let o = resolve_colors(ButtonVariant::Outline, &p, false, false, true, Style::Nova);
        assert_eq!(o.fill, p.muted);
        assert_eq!(o.fg, p.foreground);
        let g = resolve_colors(ButtonVariant::Ghost, &p, false, false, true, Style::Nova);
        assert_eq!(g.fill, p.muted);
        assert_eq!(g.fg, p.foreground);
        let s = resolve_colors(
            ButtonVariant::Secondary,
            &p,
            false,
            false,
            true,
            Style::Nova,
        );
        assert_eq!(s.fill, p.secondary);
        assert_eq!(s.fg, p.secondary_foreground);
        // Dark: expanded fills have no dark qualifier.
        let dark = palette(Mode::Dark);
        let od = resolve_colors(
            ButtonVariant::Outline,
            &dark,
            true,
            false,
            true,
            Style::Nova,
        );
        assert_eq!(od.fill, dark.muted);
        assert_eq!(od.fg, dark.foreground);
        // Default/destructive/link declare no expanded rule.
        let d = resolve_colors(ButtonVariant::Default, &p, false, false, true, Style::Nova);
        assert_eq!(d.fill, p.primary);
        // Base recipe: expanded changes nothing.
        let b = resolve_colors(ButtonVariant::Outline, &p, false, false, true, BASE_STYLE);
        assert_eq!(b.fill, p.background);
        // Hover wins over expanded (OPEN O6 judgment call, documented in
        // resolve_colors): expanded + hovered == hovered.
        let both = resolve_colors(ButtonVariant::Outline, &p, false, true, true, Style::Nova);
        let hov = resolve_colors(ButtonVariant::Outline, &p, false, true, false, Style::Nova);
        assert_eq!(both.fill, hov.fill);
        assert_eq!(both.fg, hov.fg);
    }

    #[test]
    fn base_dark_outline_hover_uses_accent_foreground() {
        // Spec §2.3: `hover:text-accent-foreground` has no dark qualifier.
        let dark = palette(Mode::Dark);
        let h = resolve_colors(ButtonVariant::Outline, &dark, true, true, false, BASE_STYLE);
        assert_eq!(h.fill, color_alpha(dark.input, 0.5));
        assert_eq!(h.fg, dark.accent_foreground);
    }

    #[test]
    fn stroke_resolution_matches_spec() {
        let p = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        // Base non-outline: no border, even focused or invalid.
        let s = stroke_for(ButtonVariant::Default, &p, false, BASE_STYLE, false, false);
        assert_eq!(s.width, 0.0);
        let s = stroke_for(ButtonVariant::Default, &p, false, BASE_STYLE, true, false);
        assert_eq!(s.width, 0.0);
        let s = stroke_for(ButtonVariant::Default, &p, false, BASE_STYLE, false, true);
        assert_eq!(s.width, 0.0);
        // Nova: 1px transparent border at rest, ring color when focused.
        let s = stroke_for(ButtonVariant::Default, &p, false, Style::Nova, false, false);
        assert_eq!((s.width, s.color), (1.0, Color32::TRANSPARENT));
        let s = stroke_for(ButtonVariant::Default, &p, false, Style::Nova, true, false);
        assert_eq!((s.width, s.color), (1.0, p.ring));
        // Nova destructive focus border is destructive/40.
        let s = stroke_for(
            ButtonVariant::Destructive,
            &p,
            false,
            Style::Nova,
            true,
            false,
        );
        assert_eq!(s.color, color_alpha(p.destructive, 0.4));
        // Outline: 1px border/input border at rest.
        let s = stroke_for(ButtonVariant::Outline, &p, false, BASE_STYLE, false, false);
        assert_eq!((s.width, s.color), (1.0, p.border));
        let s = stroke_for(
            ButtonVariant::Outline,
            &dark,
            true,
            BASE_STYLE,
            false,
            false,
        );
        assert_eq!((s.width, s.color), (1.0, dark.input));
        // Invalid: destructive border; nova dark uses destructive/50; invalid
        // wins over focus.
        let s = stroke_for(ButtonVariant::Default, &p, false, Style::Nova, false, true);
        assert_eq!((s.width, s.color), (1.0, p.destructive));
        let s = stroke_for(
            ButtonVariant::Default,
            &dark,
            true,
            Style::Nova,
            false,
            true,
        );
        assert_eq!(
            (s.width, s.color),
            (1.0, color_alpha(dark.destructive, 0.5))
        );
        let s = stroke_for(ButtonVariant::Default, &p, false, Style::Nova, true, true);
        assert_eq!(s.color, p.destructive);
        let s = stroke_for(ButtonVariant::Outline, &p, false, BASE_STYLE, false, true);
        assert_eq!((s.width, s.color), (1.0, p.destructive));
    }

    #[test]
    fn outline_shadow_resolves_to_xs_under_all_styles() {
        // Spec §2.3 + LAYER ruling: base `shadow-xs` survives under every
        // style (no `.cn-button-variant-*` rule declares a shadow).
        for style in [
            Style::Nova,
            Style::Vega,
            Style::Maia,
            Style::Lyra,
            Style::Mira,
            Style::Luma,
            Style::Sera,
            Style::Rhea,
        ] {
            assert_eq!(
                resolve_shadow(ShadowSize::Xs, style, ShadowSlot::ButtonOutline),
                ShadowSize::Xs,
                "{style:?}"
            );
        }
    }

    #[test]
    fn disabled_fade_scales_every_channel() {
        let p = palette(Mode::Light);
        let c = resolve_colors(ButtonVariant::Outline, &p, false, false, false, BASE_STYLE);
        let faded = fade_colors(c, 0.5);
        assert_eq!(faded.fill, color_alpha(c.fill, 0.5));
        assert_eq!(faded.fg, color_alpha(c.fg, 0.5));
        assert_eq!(faded.stroke.color, color_alpha(c.stroke.color, 0.5));
        assert_eq!(faded.stroke.width, c.stroke.width);
        assert_eq!(faded.ring, color_alpha(c.ring, 0.5));
        assert_eq!(fade_colors(c, 1.0).fill, c.fill);
    }

    #[test]
    fn active_shift_truth_table() {
        // Spec §5.3: nova only, never for popup triggers or disabled.
        assert!(active_shift_applies(Style::Nova, false, false));
        assert!(!active_shift_applies(Style::Nova, true, false));
        assert!(!active_shift_applies(Style::Nova, false, true));
        assert!(!active_shift_applies(BASE_STYLE, false, false));
    }

    #[test]
    fn ring_visibility_truth_table() {
        // Spec §5.4: focus shows the ring in both recipes, invalid or not.
        assert!(ring_shown(true, false, false));
        assert!(ring_shown(true, false, true));
        assert!(ring_shown(true, true, false));
        assert!(ring_shown(true, true, true));
        // Spec §5.6: invalid at rest shows the ring under nova only.
        assert!(ring_shown(false, true, true));
        assert!(!ring_shown(false, true, false));
        // Resting valid buttons show no ring.
        assert!(!ring_shown(false, false, false));
        assert!(!ring_shown(false, false, true));
    }

    #[test]
    fn mix_color_handles_transparent_endpoints() {
        // Fading in from transparent holds the target hue and ramps alpha
        // (mixing toward transparent in OKLab would dip through black).
        let accent = Color32::from_rgb(200, 200, 200);
        let mid = mix_color(Color32::TRANSPARENT, accent, 0.5);
        assert_eq!(mid, color_alpha(accent, 0.5));
        let mid = mix_color(accent, Color32::TRANSPARENT, 0.5);
        assert_eq!(mid, color_alpha(accent, 0.5));
        // Opaque endpoints mix in OKLab with exact endpoints.
        assert_eq!(mix_color(accent, Color32::BLACK, 0.0), accent);
        assert_eq!(mix_color(accent, Color32::BLACK, 1.0), Color32::BLACK);
    }

    #[test]
    fn mix_color_never_overshoots_with_translucent_endpoints() {
        // BTN-FLK-03: the dark outline expanded fade mixes translucent
        // white (`input/30`, white at ~4.5% alpha) toward opaque muted.
        // In unmultiplied OKLab the composited value bulged to ~77,
        // brighter than both endpoints (20/38). The premultiplied mix
        // keeps every composited channel within the endpoint range and
        // monotonic in `t` (up to ±1 rounding noise), in both directions.
        fn assert_no_overshoot(a: Color32, b: Color32, backdrop: Color32) {
            let lo = blend(backdrop, a);
            let hi = blend(backdrop, b);
            let channels = |c: Color32| [c.r(), c.g(), c.b()];
            let mut prev: Option<[u8; 3]> = None;
            for i in 0..=20 {
                let t = i as f32 / 20.0;
                let mid = channels(blend(backdrop, mix_color(a, b, t)));
                let (l, h) = (channels(lo), channels(hi));
                for ch in 0..3 {
                    let (mn, mx) = (l[ch].min(h[ch]), l[ch].max(h[ch]));
                    assert!(
                        mid[ch].saturating_add(1) >= mn && mid[ch] <= mx.saturating_add(1),
                        "overshoot at t={t}: channel {ch} value {} outside [{mn}, {mx}]",
                        mid[ch]
                    );
                    if let Some(p) = prev {
                        if h[ch] >= l[ch] {
                            assert!(mid[ch].saturating_add(1) >= p[ch], "non-monotonic at t={t}");
                        } else {
                            assert!(mid[ch] <= p[ch].saturating_add(1), "non-monotonic at t={t}");
                        }
                    }
                }
                prev = Some(mid);
            }
        }
        let dark = palette(Mode::Dark);
        let rest = resolve_colors(
            ButtonVariant::Outline,
            &dark,
            true,
            false,
            false,
            Style::Nova,
        );
        let exp = resolve_colors(
            ButtonVariant::Outline,
            &dark,
            true,
            false,
            true,
            Style::Nova,
        );
        assert!(rest.fill.a() > 0 && rest.fill.a() < 255);
        assert_eq!(exp.fill.a(), 255);
        assert_no_overshoot(rest.fill, exp.fill, dark.background);
        assert_no_overshoot(exp.fill, rest.fill, dark.background);
        // A second translucent pair (opaque primary toward primary/80):
        // same guarantee, both directions.
        let d_rest = resolve_colors(
            ButtonVariant::Default,
            &dark,
            true,
            false,
            false,
            Style::Nova,
        );
        let d_hov = resolve_colors(
            ButtonVariant::Default,
            &dark,
            true,
            true,
            false,
            Style::Nova,
        );
        assert_no_overshoot(d_rest.fill, d_hov.fill, dark.background);
        assert_no_overshoot(d_hov.fill, d_rest.fill, dark.background);
        // Opaque pairs still mix in OKLab (perceptual midpoint, not the
        // premultiplied-sRGB midpoint).
        assert_eq!(
            mix_color(Color32::BLACK, Color32::WHITE, 0.5),
            mix_oklab(Color32::BLACK, Color32::WHITE, 0.5)
        );
    }

    #[test]
    fn mix_colors_endpoints_are_exact() {
        let p = palette(Mode::Light);
        let a = resolve_colors(ButtonVariant::Default, &p, false, false, false, BASE_STYLE);
        let b = resolve_colors(ButtonVariant::Default, &p, false, true, false, BASE_STYLE);
        let lo = mix_colors(a, b, 0.0);
        assert_eq!(lo.fill, a.fill);
        assert_eq!(lo.fg, a.fg);
        let hi = mix_colors(a, b, 1.0);
        assert_eq!(hi.fill, b.fill);
        assert_eq!(hi.fg, b.fg);
        assert_eq!(hi.stroke.width, a.stroke.width);
    }

    #[test]
    fn endpoint_slots_never_cross_the_transparent_boundary() {
        // BTN-FLK-02: every slot wrapped in `transition_color_to` must
        // satisfy the helper's documented caveat — its target never crosses
        // the transparent boundary between themes. A slot transparent in
        // one theme and opaque in the other would dip through black in the
        // OKLab leg. Checks every wrapped slot across both styles: the
        // endpoint-set fill/fg (all four state combinations), both border
        // endpoints (rest + focus, valid + invalid), and the ring target
        // (valid mixed ring + invalid destructive target).
        fn compatible(a: Color32, b: Color32) -> bool {
            (a.a() == 0) == (b.a() == 0)
        }
        let light = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        let variants = [
            ButtonVariant::Default,
            ButtonVariant::Destructive,
            ButtonVariant::Outline,
            ButtonVariant::Secondary,
            ButtonVariant::Ghost,
            ButtonVariant::Link,
        ];
        for style in [Style::Nova, BASE_STYLE] {
            for variant in variants {
                for (hovered, expanded) in
                    [(false, false), (true, false), (false, true), (true, true)]
                {
                    let a = resolve_colors(variant, &light, false, hovered, expanded, style);
                    let b = resolve_colors(variant, &dark, true, hovered, expanded, style);
                    assert!(
                        compatible(a.fill, b.fill),
                        "{variant:?}/{style:?} hovered={hovered} expanded={expanded}: fill crosses the boundary (light={a:?}, dark={b:?})"
                    );
                    assert!(
                        compatible(a.fg, b.fg),
                        "{variant:?}/{style:?} hovered={hovered} expanded={expanded}: fg crosses the boundary (light={a:?}, dark={b:?})"
                    );
                    assert!(
                        compatible(a.ring, b.ring),
                        "{variant:?}/{style:?} hovered={hovered} expanded={expanded}: ring crosses the boundary (light={a:?}, dark={b:?})"
                    );
                }
                for (focused, invalid) in [(false, false), (true, false), (true, true)] {
                    let a = stroke_for(variant, &light, false, style, focused, invalid);
                    let b = stroke_for(variant, &dark, true, style, focused, invalid);
                    assert!(
                        compatible(a.color, b.color),
                        "{variant:?}/{style:?} focused={focused} invalid={invalid}: border crosses the boundary (light={a:?}, dark={b:?})"
                    );
                }
                let inv_ring_light = color_alpha(light.destructive, 0.2);
                let inv_ring_dark = color_alpha(dark.destructive, 0.4);
                assert!(
                    compatible(inv_ring_light, inv_ring_dark),
                    "{variant:?}/{style:?}: invalid ring target crosses the boundary"
                );
            }
        }
    }

    #[test]
    fn new_builders_accept_values_and_render() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("button_new_builders_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            );
            let r = Button::new("Pop")
                .expanded(true)
                .has_popup(true)
                .icon_start_sized(Icon::Plus, 20.0)
                .icon_end_sized(Icon::Check, 12.0)
                .show(ui);
            assert!(r.enabled());
        });
        out.textures_delta.clear();
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
    fn click_moves_keyboard_focus_to_the_button() {
        // BTN-INT-01: native `<button>` behaviour (spec §8) — a clicked button
        // holds keyboard focus afterwards. Disabled buttons never take focus.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("button_click_focus_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            )
        };
        let click_event = |pos: Pos2, pressed: bool| egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };

        // Frame 1: render, record rect + id; nothing focused yet.
        let mut rect = Rect::NOTHING;
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let r = Button::new("Click").show(ui);
            rect = r.rect;
            id = r.id;
        });
        out.textures_delta.clear();
        assert!(!ctx.memory(|m| m.has_focus(id)));

        // Frames 2-3: press + release on the button = a click.
        let center = rect.center();
        let mut out = ctx.run_ui(click_event(center, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let _r = Button::new("Click").show(ui);
        });
        out.textures_delta.clear();
        let mut clicked = false;
        let mut out = ctx.run_ui(click_event(center, false), |ctx| {
            let ui = &mut make_ui(ctx);
            clicked = Button::new("Click").show(ui).clicked();
        });
        out.textures_delta.clear();
        assert!(clicked, "press + release must register a click");
        assert!(
            ctx.memory(|m| m.has_focus(id)),
            "click must move keyboard focus to the button"
        );

        // A disabled button takes no focus when clicked: fresh context so no
        // stale focus state leaks in.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut rect = Rect::NOTHING;
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let r = Button::new("Off").disabled(true).show(ui);
            rect = r.rect;
            id = r.id;
        });
        out.textures_delta.clear();
        let center = rect.center();
        let mut out = ctx.run_ui(click_event(center, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let _r = Button::new("Off").disabled(true).show(ui);
        });
        out.textures_delta.clear();
        let mut clicked = false;
        let mut out = ctx.run_ui(click_event(center, false), |ctx| {
            let ui = &mut make_ui(ctx);
            clicked = Button::new("Off").disabled(true).show(ui).clicked();
        });
        out.textures_delta.clear();
        assert!(!clicked, "disabled buttons never report clicks");
        assert!(
            !ctx.memory(|m| m.has_focus(id)),
            "disabled buttons must never take focus on click"
        );
    }

    #[test]
    fn space_fires_on_keyup_not_keydown() {
        // BTN-INT-04: native `<button>` behaviour (spec §8) — Space shows
        // `:active` while held and fires on keyup; Enter fires on keydown.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("button_space_keyup_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            )
        };
        let key_event = |key: egui::Key, pressed: bool| egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let frame = |ctx: &egui::Context, input: egui::RawInput| {
            let mut clicked = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut make_ui(ctx);
                clicked = Button::new("Space").show(ui).clicked();
            });
            out.textures_delta.clear();
            clicked
        };

        // Frame 1: render, record id; then give the button keyboard focus.
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            id = Button::new("Space").show(ui).id;
        });
        out.textures_delta.clear();
        ctx.memory_mut(|m| m.request_focus(id));
        assert!(ctx.memory(|m| m.has_focus(id)));

        // Space keydown: no fire. Repeat keydown (autorepeat): no fire.
        // Hold frame with no events: no fire.
        assert!(!frame(&ctx, key_event(egui::Key::Space, true)));
        assert!(!frame(&ctx, key_event(egui::Key::Space, true)));
        assert!(!frame(&ctx, egui::RawInput::default()));
        // Space keyup: exactly one fire. The frame after: no fire.
        assert!(frame(&ctx, key_event(egui::Key::Space, false)));
        assert!(!frame(&ctx, egui::RawInput::default()));
        // A second press/release cycle fires exactly once, on keyup.
        assert!(!frame(&ctx, key_event(egui::Key::Space, true)));
        assert!(frame(&ctx, key_event(egui::Key::Space, false)));
        // Enter still fires on keydown, and does not fire on keyup.
        assert!(frame(&ctx, key_event(egui::Key::Enter, true)));
        assert!(!frame(&ctx, key_event(egui::Key::Enter, false)));
        // Focus is retained after keyboard activation.
        assert!(ctx.memory(|m| m.has_focus(id)));

        // A disabled button never fires from Space, on either edge.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let _r = Button::new("Off").disabled(true).show(ui);
        });
        out.textures_delta.clear();
        let disabled_frame = |input: egui::RawInput| {
            let mut clicked = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut make_ui(ctx);
                clicked = Button::new("Off").disabled(true).show(ui).clicked();
            });
            out.textures_delta.clear();
            clicked
        };
        assert!(!disabled_frame(key_event(egui::Key::Space, true)));
        assert!(!disabled_frame(key_event(egui::Key::Space, false)));
    }

    #[test]
    fn escape_is_a_noop_on_a_focused_button() {
        // BTN-INT-02: spec §8 — Escape has no button behaviour; a focused
        // button keeps focus when Escape is pressed. egui's default
        // (`Focus::begin_pass`) clears focus on unmodified Escape unless the
        // focused widget claims it via the focus-lock filter.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("button_escape_noop_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            )
        };
        let key_event = |key: egui::Key, pressed: bool| egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let frame = |ctx: &egui::Context, input: egui::RawInput| {
            let mut clicked = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut make_ui(ctx);
                clicked = Button::new("Escape").show(ui).clicked();
            });
            out.textures_delta.clear();
            clicked
        };

        // Frame 1: render, record id; then give the button keyboard focus.
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            id = Button::new("Escape").show(ui).id;
        });
        out.textures_delta.clear();
        ctx.memory_mut(|m| m.request_focus(id));
        assert!(ctx.memory(|m| m.has_focus(id)));

        // A settled frame with no events: the button runs with focus, so the
        // focus-lock filter is set.
        assert!(!frame(&ctx, egui::RawInput::default()));
        assert!(ctx.memory(|m| m.has_focus(id)));

        // Escape keydown: no fire, focus retained. Escape keyup: same.
        assert!(!frame(&ctx, key_event(egui::Key::Escape, true)));
        assert!(
            ctx.memory(|m| m.has_focus(id)),
            "Escape keydown must not clear focus"
        );
        assert!(!frame(&ctx, key_event(egui::Key::Escape, false)));
        assert!(
            ctx.memory(|m| m.has_focus(id)),
            "Escape keyup must not clear focus"
        );
        // A second Escape press: still a no-op.
        assert!(!frame(&ctx, key_event(egui::Key::Escape, true)));
        assert!(ctx.memory(|m| m.has_focus(id)));
        assert!(!frame(&ctx, key_event(egui::Key::Escape, false)));
        assert!(ctx.memory(|m| m.has_focus(id)));

        // Space still fires on keyup after Escape: focus is genuinely held.
        assert!(!frame(&ctx, key_event(egui::Key::Space, true)));
        assert!(frame(&ctx, key_event(egui::Key::Space, false)));
        assert!(ctx.memory(|m| m.has_focus(id)));
    }

    #[test]
    fn arrow_keys_are_noops_on_a_focused_button() {
        // BTN-INT-03: spec §8 — arrow keys have no button behaviour; a
        // focused button keeps focus when an arrow key is pressed. egui's
        // default (`Focus::begin_pass` + `end_pass`) moves focus spatially
        // on unmodified arrows unless the focused widget claims them via
        // the focus-lock filter. The 2x2 grid gives every direction a
        // spatial neighbour, so each arrow would move focus without the
        // fix; Tab must still move focus (`tab: false` preserved).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("button_arrow_noop_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            )
        };
        let key_event = |key: egui::Key, pressed: bool| egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let frame = |ctx: &egui::Context, input: egui::RawInput| {
            let mut clicked = false;
            let mut ids = [egui::Id::NULL; 4];
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut make_ui(ctx);
                ui.horizontal(|ui| {
                    let r = Button::new("A").id_salt("arrow_a").show(ui);
                    ids[0] = r.id;
                    clicked |= r.clicked();
                    let r = Button::new("B").id_salt("arrow_b").show(ui);
                    ids[1] = r.id;
                    clicked |= r.clicked();
                });
                ui.horizontal(|ui| {
                    let r = Button::new("C").id_salt("arrow_c").show(ui);
                    ids[2] = r.id;
                    clicked |= r.clicked();
                    let r = Button::new("D").id_salt("arrow_d").show(ui);
                    ids[3] = r.id;
                    clicked |= r.clicked();
                });
            });
            out.textures_delta.clear();
            (ids, clicked)
        };

        // Frame 1: render, record ids; then give button A keyboard focus.
        let (ids, _) = frame(&ctx, egui::RawInput::default());
        let (a, b) = (ids[0], ids[1]);
        ctx.memory_mut(|m| m.request_focus(a));
        assert!(ctx.memory(|m| m.has_focus(a)));

        // A settled frame with no events: A runs with focus, so the
        // focus-lock filter is set.
        assert!(!frame(&ctx, egui::RawInput::default()).1);
        assert!(ctx.memory(|m| m.has_focus(a)));

        // ArrowRight / ArrowDown from A: no fire, focus stays on A (each
        // has a spatial neighbour — B right, C below — so without the
        // filter fix focus would move).
        for key in [egui::Key::ArrowRight, egui::Key::ArrowDown] {
            assert!(!frame(&ctx, key_event(key, true)).1);
            assert!(
                ctx.memory(|m| m.has_focus(a)),
                "{key:?} keydown must not move focus"
            );
            assert!(!frame(&ctx, key_event(key, false)).1);
            assert!(ctx.memory(|m| m.has_focus(a)));
        }
        // ArrowLeft / ArrowUp from A: no neighbour in that direction, but
        // still no-ops.
        for key in [egui::Key::ArrowLeft, egui::Key::ArrowUp] {
            assert!(!frame(&ctx, key_event(key, true)).1);
            assert!(ctx.memory(|m| m.has_focus(a)));
            assert!(!frame(&ctx, key_event(key, false)).1);
            assert!(ctx.memory(|m| m.has_focus(a)));
        }

        // Tab still moves focus to the next widget (B), then a settled
        // frame so B's filter is set before arrows are pressed.
        assert!(!frame(&ctx, key_event(egui::Key::Tab, true)).1);
        assert!(
            ctx.memory(|m| m.has_focus(b)),
            "Tab must still move focus to the next widget"
        );
        assert!(!frame(&ctx, egui::RawInput::default()).1);
        assert!(ctx.memory(|m| m.has_focus(b)));

        // ArrowLeft / ArrowDown from B: neighbours A and D; focus stays.
        for key in [egui::Key::ArrowLeft, egui::Key::ArrowDown] {
            assert!(!frame(&ctx, key_event(key, true)).1);
            assert!(
                ctx.memory(|m| m.has_focus(b)),
                "{key:?} keydown must not move focus"
            );
            assert!(!frame(&ctx, key_event(key, false)).1);
            assert!(ctx.memory(|m| m.has_focus(b)));
        }

        // Space still fires on keyup after arrows: focus is genuinely held.
        assert!(!frame(&ctx, key_event(egui::Key::Space, true)).1);
        assert!(frame(&ctx, key_event(egui::Key::Space, false)).1);
        assert!(ctx.memory(|m| m.has_focus(b)));
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

    // ------------------------------------------------------------------
    // FG-1/2/3: group plumbing channels (Foundation Loop, 2026-11-16).
    // ------------------------------------------------------------------

    fn fg_test_ui(ctx: &egui::Context, name: &str) -> egui::Ui {
        egui::Ui::new(
            ctx.clone(),
            egui::Id::unique(name),
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
        )
    }

    fn input_at(time: f64) -> egui::RawInput {
        egui::RawInput {
            time: Some(time),
            ..Default::default()
        }
    }

    #[test]
    fn vertical_group_text_button_stretches_to_target() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let id = egui::Id::unique("fg1_stretch");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg1_stretch_ui");
            group::begin(ui, id, GroupOrientation::Vertical);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Vertical,
                    cross: 200.0,
                },
            );
            let r = Button::new("Hi").show(ui);
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
            assert!(
                (r.rect.width() - 200.0).abs() < 0.01,
                "stretched width {}, want 200",
                r.rect.width()
            );
            assert!(
                (r.rect.height() - 32.0).abs() < 0.01,
                "nova default height {}, want 32",
                r.rect.height()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn vertical_group_icon_button_keeps_fixed_width() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let id = egui::Id::unique("fg1_icon");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg1_icon_ui");
            group::begin(ui, id, GroupOrientation::Vertical);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Vertical,
                    cross: 200.0,
                },
            );
            let r = Button::new("").size(ButtonSize::Icon).show(ui);
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
            assert!(
                (r.rect.width() - 32.0).abs() < 0.01,
                "icon width {}, want fixed 32",
                r.rect.width()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn button_without_stretch_target_keeps_natural_width() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let id = egui::Id::unique("fg1_no_target");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg1_no_target_ui");
            group::begin(ui, id, GroupOrientation::Vertical);
            // No stretch target pushed: measure-pass / standalone sizing.
            let natural = Button::new("Hi").content_size(ui).x;
            let r = Button::new("Hi").show(ui);
            group::end(ui, id);
            assert!(
                (r.rect.width() - natural).abs() < 0.01,
                "width {} != natural width {natural}",
                r.rect.width()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn horizontal_stretch_target_does_not_stretch_buttons() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let id = egui::Id::unique("fg1_horizontal");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg1_horizontal_ui");
            group::begin(ui, id, GroupOrientation::Horizontal);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Horizontal,
                    cross: 200.0,
                },
            );
            let natural = Button::new("Hi").content_size(ui);
            let r = Button::new("Hi").show(ui);
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
            assert!(
                (r.rect.width() - natural.x).abs() < 0.01,
                "width {} != natural width {}",
                r.rect.width(),
                natural.x
            );
            assert!(
                (r.rect.height() - natural.y).abs() < 0.01,
                "height {} != natural height {}",
                r.rect.height(),
                natural.y
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn stretch_target_smaller_than_natural_width_does_not_shrink() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let id = egui::Id::unique("fg1_no_shrink");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg1_no_shrink_ui");
            group::begin(ui, id, GroupOrientation::Vertical);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Vertical,
                    cross: 10.0,
                },
            );
            let natural = Button::new("Hi").content_size(ui).x;
            assert!(natural > 10.0);
            let r = Button::new("Hi").show(ui);
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
            assert!(
                (r.rect.width() - natural).abs() < 0.01,
                "width {} != natural width {natural}",
                r.rect.width()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn last_slot_corner_forcing() {
        let base = CornerRadius::same(8);
        let h = force_last_slot_corners(base, GroupOrientation::Horizontal, 10);
        assert_eq!((h.nw, h.ne, h.sw, h.se), (8, 10, 8, 10));
        let v = force_last_slot_corners(base, GroupOrientation::Vertical, 10);
        assert_eq!((v.nw, v.ne, v.sw, v.se), (8, 8, 10, 10));
        // Beats fused (zeroed) trailing corners; leading corners untouched.
        let fused = CornerRadius {
            nw: 10,
            ne: 0,
            sw: 10,
            se: 0,
        };
        let forced = force_last_slot_corners(fused, GroupOrientation::Horizontal, 10);
        assert_eq!(
            (forced.nw, forced.ne, forced.sw, forced.se),
            (10, 10, 10, 10)
        );
        let fused_v = CornerRadius {
            nw: 0,
            ne: 0,
            sw: 10,
            se: 10,
        };
        let forced_v = force_last_slot_corners(fused_v, GroupOrientation::Vertical, 10);
        assert_eq!(
            (forced_v.nw, forced_v.ne, forced_v.sw, forced_v.se),
            (0, 0, 10, 10)
        );
    }

    #[test]
    fn slot_registration_and_transfer_protocol() {
        let ctx = egui::Context::default();
        let measure_id = egui::Id::unique("fg2_measure");
        let id = egui::Id::unique("fg2_group");

        // Measure pass: [Button, non-slot, Separator] -> last slot index 2.
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_ui");
            group::begin(ui, measure_id, GroupOrientation::Horizontal);
            let _ = group::next_child_position(ui);
            group::register_slot(ui, group::SlotIdentity::Button);
            let _ = group::next_child_position(ui);
            let _ = group::next_child_position(ui);
            group::register_slot(ui, group::SlotIdentity::ButtonGroupSeparator);
            group::end(ui, measure_id);
            assert_eq!(group::last_slot_child(ui, measure_id), Some(2));
        });
        out.textures_delta.clear();

        // Real pass: the transfer flags only child 2.
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_ui");
            let last = group::last_slot_child(ui, measure_id);
            group::begin(ui, id, GroupOrientation::Horizontal);
            group::set_last_slot_child(ui, id, last);
            let (_, _, f0) = group::next_child_position_ex(ui).unwrap();
            let (_, _, f1) = group::next_child_position_ex(ui).unwrap();
            let (_, _, f2) = group::next_child_position_ex(ui).unwrap();
            group::end(ui, id);
            assert!(!f0 && !f1 && f2, "flags {f0} {f1} {f2}");
        });
        out.textures_delta.clear();

        // Trailing non-slot child: [Button, non-slot] -> last slot index 0,
        // and the non-last button still receives the rule (`!` semantics).
        let measure2 = egui::Id::unique("fg2_measure2");
        let id2 = egui::Id::unique("fg2_group2");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_ui");
            group::begin(ui, measure2, GroupOrientation::Horizontal);
            let _ = group::next_child_position(ui);
            group::register_slot(ui, group::SlotIdentity::Button);
            let _ = group::next_child_position(ui);
            group::end(ui, measure2);
            assert_eq!(group::last_slot_child(ui, measure2), Some(0));
        });
        out.textures_delta.clear();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_ui");
            let last = group::last_slot_child(ui, measure2);
            group::begin(ui, id2, GroupOrientation::Horizontal);
            group::set_last_slot_child(ui, id2, last);
            let (_, _, f0) = group::next_child_position_ex(ui).unwrap();
            let (_, _, f1) = group::next_child_position_ex(ui).unwrap();
            group::end(ui, id2);
            assert!(f0 && !f1, "flags {f0} {f1}");
        });
        out.textures_delta.clear();

        // No slot-carrying children: nothing is flagged.
        let measure3 = egui::Id::unique("fg2_measure3");
        let id3 = egui::Id::unique("fg2_group3");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_ui");
            group::begin(ui, measure3, GroupOrientation::Horizontal);
            let _ = group::next_child_position(ui);
            let _ = group::next_child_position(ui);
            group::end(ui, measure3);
            assert_eq!(group::last_slot_child(ui, measure3), None);
        });
        out.textures_delta.clear();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_ui");
            group::begin(ui, id3, GroupOrientation::Horizontal);
            group::set_last_slot_child(ui, id3, None);
            let (_, _, f0) = group::next_child_position_ex(ui).unwrap();
            let (_, _, f1) = group::next_child_position_ex(ui).unwrap();
            group::end(ui, id3);
            assert!(!f0 && !f1, "flags {f0} {f1}");
        });
        out.textures_delta.clear();

        // A registration with no consumed position is ignored.
        let id4 = egui::Id::unique("fg2_group4");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_ui");
            group::begin(ui, id4, GroupOrientation::Horizontal);
            group::register_slot(ui, group::SlotIdentity::Button);
            group::end(ui, id4);
            assert_eq!(group::last_slot_child(ui, id4), None);
        });
        out.textures_delta.clear();

        // A nested group registers with its parent while the parent is on
        // top of the stack, before its own `begin`.
        let outer = egui::Id::unique("fg2_outer");
        let inner = egui::Id::unique("fg2_inner");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_ui");
            group::begin(ui, outer, GroupOrientation::Horizontal);
            let _ = group::next_child_position(ui);
            group::register_slot(ui, group::SlotIdentity::ButtonGroup);
            group::begin(ui, inner, GroupOrientation::Horizontal);
            let _ = group::next_child_position(ui);
            group::register_slot(ui, group::SlotIdentity::Button);
            group::end(ui, inner);
            let _ = group::next_child_position(ui);
            group::end(ui, outer);
            assert_eq!(group::last_slot_child(ui, outer), Some(0));
            assert_eq!(group::last_slot_child(ui, inner), Some(0));
        });
        out.textures_delta.clear();
    }

    /// The border-stroke shapes (`Shape::Rect` with a 1px stroke and a
    /// transparent fill) in paint order, with their corner radii.
    fn border_strokes(out: &egui::FullOutput) -> Vec<(Rect, CornerRadius)> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.stroke.width == 1.0 && rs.fill == Color32::TRANSPARENT =>
                {
                    Some((rs.rect, rs.corner_radius))
                }
                _ => None,
            })
            .collect()
    }

    #[test]
    fn nova_button_with_last_slot_flag_paints_lg_trailing_corners() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let measure_id = egui::Id::unique("fg2_e2e_measure");
        let id = egui::Id::unique("fg2_e2e_group");

        // Measure pass: [Button, non-slot child].
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_e2e_ui");
            group::begin(ui, measure_id, GroupOrientation::Horizontal);
            let _ = Button::new("A").id_salt("fg2_e2e_a").show(ui);
            let _ = group::next_child_position(ui);
            group::end(ui, measure_id);
        });
        out.textures_delta.clear();

        // Real pass with the transfer: the non-last button receives the
        // rule, so its fused trailing corners come back as `rounded-lg`.
        let mut button_rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_e2e_ui");
            let last = group::last_slot_child(ui, measure_id);
            assert_eq!(last, Some(0));
            group::begin(ui, id, GroupOrientation::Horizontal);
            group::set_last_slot_child(ui, id, last);
            let r = Button::new("A").id_salt("fg2_e2e_a").show(ui);
            button_rect = r.rect;
            let _ = group::next_child_position(ui);
            group::end(ui, id);
        });
        out.textures_delta.clear();

        let strokes = border_strokes(&out);
        assert_eq!(
            strokes.len(),
            1,
            "expected exactly one border stroke, got {}",
            strokes.len()
        );
        let (rect, rounding) = strokes[0];
        assert!(
            (rect.min.x - button_rect.min.x).abs() < 0.01
                && (rect.min.y - button_rect.min.y).abs() < 0.01,
            "stroke rect {rect:?} should match the button rect {button_rect:?}"
        );
        assert_eq!(
            (rounding.nw, rounding.ne, rounding.sw, rounding.se),
            (10, 10, 10, 10),
            "trailing corners must be forced to rounded-lg by the flag"
        );
    }

    #[test]
    fn nova_button_without_last_slot_flag_keeps_fused_corners() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let id = egui::Id::unique("fg2_noflag_group");
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_noflag_ui");
            group::begin(ui, id, GroupOrientation::Horizontal);
            let _ = Button::new("A").id_salt("fg2_noflag_a").show(ui);
            let _ = group::next_child_position(ui);
            group::end(ui, id);
        });
        out.textures_delta.clear();

        let strokes = border_strokes(&out);
        assert_eq!(strokes.len(), 1);
        let (_, rounding) = strokes[0];
        assert_eq!(
            (rounding.nw, rounding.ne, rounding.sw, rounding.se),
            (10, 0, 10, 0),
            "without the flag, fusing zeroes the trailing corners"
        );
    }

    #[test]
    fn base_recipe_ignores_the_last_slot_flag() {
        let ctx = egui::Context::default();
        Preset {
            style: BASE_STYLE,
            ..Default::default()
        }
        .install(&ctx);
        let measure_id = egui::Id::unique("fg2_base_measure");
        let id = egui::Id::unique("fg2_base_group");

        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_base_ui");
            group::begin(ui, measure_id, GroupOrientation::Horizontal);
            let _ = Button::new("A").id_salt("fg2_base_a").show(ui);
            let _ = group::next_child_position(ui);
            group::end(ui, measure_id);
        });
        out.textures_delta.clear();

        let mut button_rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg2_base_ui");
            let last = group::last_slot_child(ui, measure_id);
            group::begin(ui, id, GroupOrientation::Horizontal);
            group::set_last_slot_child(ui, id, last);
            let r = Button::new("A").id_salt("fg2_base_a").show(ui);
            button_rect = r.rect;
            let _ = group::next_child_position(ui);
            group::end(ui, id);
        });
        out.textures_delta.clear();

        // Under the base recipe the Default variant has no border stroke,
        // so judge the fill shape: its rounding is the fused corner
        // exactly (no `bg-clip-padding` shrink outside nova).
        let fills: Vec<(Rect, CornerRadius)> = out
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.fill != Color32::TRANSPARENT && rs.stroke == Stroke::NONE =>
                {
                    Some((rs.rect, rs.corner_radius))
                }
                _ => None,
            })
            .collect();
        assert_eq!(
            fills.len(),
            1,
            "expected exactly one fill, got {}",
            fills.len()
        );
        let (rect, rounding) = fills[0];
        assert!(
            (rect.min.x - button_rect.min.x).abs() < 0.01
                && (rect.min.y - button_rect.min.y).abs() < 0.01,
            "fill rect {rect:?} should match the button rect {button_rect:?}"
        );
        assert_eq!(
            (rounding.nw, rounding.ne, rounding.sw, rounding.se),
            (8, 0, 8, 0),
            "the base recipe has no last-slot rule: fusing stands"
        );
    }

    #[test]
    fn group_focus_ring_paints_above_following_sibling() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let id = egui::Id::unique("fg3_group");
        let mut b1_id = egui::Id::NULL;
        let mut b2_rect = Rect::NOTHING;

        // Frame 1: render, record the first button's id and the second
        // button's rect.
        let mut out = ctx.run_ui(input_at(0.0), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg3_ui");
            group::begin(ui, id, GroupOrientation::Horizontal);
            let r1 = Button::new("One").id_salt("fg3_b1").show(ui);
            let r2 = Button::new("Two").id_salt("fg3_b2").show(ui);
            group::end(ui, id);
            b1_id = r1.id;
            b2_rect = r2.rect;
        });
        out.textures_delta.clear();

        // Focus the first button, then run two more frames with a
        // controlled clock so the 150ms ring fade completes.
        ctx.memory_mut(|m| m.request_focus(b1_id));
        let mut out = ctx.run_ui(input_at(0.05), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg3_ui");
            group::begin(ui, id, GroupOrientation::Horizontal);
            let _ = Button::new("One").id_salt("fg3_b1").show(ui);
            let _ = Button::new("Two").id_salt("fg3_b2").show(ui);
            group::end(ui, id);
        });
        out.textures_delta.clear();
        let mut out = ctx.run_ui(input_at(0.30), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg3_ui");
            group::begin(ui, id, GroupOrientation::Horizontal);
            let _ = Button::new("One").id_salt("fg3_b1").show(ui);
            let _ = Button::new("Two").id_salt("fg3_b2").show(ui);
            group::end(ui, id);
        });
        out.textures_delta.clear();

        // The ring: a 3px outside stroke. It must exist (the queue
        // flushed) and come after every shape the second button painted.
        let mut ring_idx = None;
        let mut sibling_max_idx = 0usize;
        let mut sibling_count = 0usize;
        for (i, clipped) in out.shapes.iter().enumerate() {
            match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.stroke.width == 3.0
                        && rs.stroke_kind == egui::epaint::StrokeKind::Outside =>
                {
                    ring_idx = Some(i);
                }
                egui::Shape::Rect(rs) if rs.rect.intersects(b2_rect) => {
                    sibling_max_idx = sibling_max_idx.max(i);
                    sibling_count += 1;
                }
                egui::Shape::Text(ts) if b2_rect.contains(ts.pos) => {
                    sibling_max_idx = sibling_max_idx.max(i);
                    sibling_count += 1;
                }
                _ => {}
            }
        }
        let ring_idx = ring_idx.expect("a deferred focus ring must be painted");
        assert!(
            sibling_count > 0,
            "the second button must have painted shapes to compare against"
        );
        assert!(
            ring_idx > sibling_max_idx,
            "ring shape at index {ring_idx} must paint after the sibling's last shape at {sibling_max_idx}"
        );
    }

    #[test]
    fn standalone_focus_ring_paints_inline_before_following_content() {
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Default::default()
        }
        .install(&ctx);
        let mut b_id = egui::Id::NULL;

        let mut out = ctx.run_ui(input_at(0.0), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg3_standalone_ui");
            let r = Button::new("Solo").id_salt("fg3_solo").show(ui);
            b_id = r.id;
        });
        out.textures_delta.clear();

        ctx.memory_mut(|m| m.request_focus(b_id));
        let mut out = ctx.run_ui(input_at(0.05), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg3_standalone_ui");
            let _ = Button::new("Solo").id_salt("fg3_solo").show(ui);
        });
        out.textures_delta.clear();
        let mut out = ctx.run_ui(input_at(0.30), |ctx| {
            let ui = &mut fg_test_ui(ctx, "fg3_standalone_ui");
            let r = Button::new("Solo").id_salt("fg3_solo").show(ui);
            // Content painted after the button, in the same Ui: the
            // inline ring must precede it in paint order, as before FG-3.
            ui.painter().rect_filled(
                Rect::from_min_size(r.rect.min + Vec2::new(0.0, 60.0), Vec2::new(40.0, 20.0)),
                CornerRadius::ZERO,
                Color32::RED,
            );
        });
        out.textures_delta.clear();

        let mut ring_idx = None;
        let mut marker_idx = None;
        for (i, clipped) in out.shapes.iter().enumerate() {
            if let egui::Shape::Rect(rs) = &clipped.shape {
                if rs.stroke.width == 3.0 && rs.stroke_kind == egui::epaint::StrokeKind::Outside {
                    ring_idx = Some(i);
                }
                if rs.fill == Color32::RED {
                    marker_idx = Some(i);
                }
            }
        }
        let ring_idx = ring_idx.expect("the standalone ring must be painted");
        let marker_idx = marker_idx.expect("the marker rect must be painted");
        assert!(
            ring_idx < marker_idx,
            "standalone ring at {ring_idx} must paint before following content at {marker_idx}"
        );
    }
}
