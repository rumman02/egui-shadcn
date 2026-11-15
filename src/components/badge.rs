//! shadcn `badge.tsx` — a small pill label for statuses and counts.
//!
//! Rebuilt to the frozen Batch A S0 spec (`batches-a-h/specs/badge-spec.md`;
//! all section references below are to that spec). The original is a single
//! `<span>` (`badge.tsx`, 47 lines): one shared class string plus six CVA
//! variant strings, merged by `cn()`; no Radix behaviour primitive except
//! `Slot` for `asChild` (spec §1, §9).
//!
//! Shared base string (`badge.tsx:7`):
//! `inline-flex w-fit shrink-0 items-center justify-center gap-1 overflow-hidden
//! rounded-full border border-transparent px-2 py-0.5 text-xs font-medium
//! whitespace-nowrap transition-[color,box-shadow] focus-visible:border-ring
//! focus-visible:ring-[3px] focus-visible:ring-ring/50 aria-invalid:border-destructive
//! aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40
//! [&>svg]:pointer-events-none [&>svg]:size-3`
//!
//! Nova layers over the base recipe (the LAYER ruling — silence keeps the base
//! value): `.cn-badge` (`style-nova.css:96`) restates only height (`h-5`), gap,
//! radius (`rounded-4xl`), border, padding, text size/weight, transition
//! (`transition-all`), per-side icon padding
//! (`has-data-[icon=...]:pl/pr-1.5`), and svg size (`[&>svg]:size-3!`);
//! everything else survives from base. The six `.cn-badge-variant-*` rules
//! (`style-nova.css:100-120`) restate the variant colours. The destructive
//! variant differs radically between recipes (base solid `bg-destructive
//! text-white`, nova tint `bg-destructive/10 text-destructive
//! dark:bg-destructive/20`) — intentional, not a defect (spec §2.3).
//!
//! Span badges are non-interactive: no Tab stop, no focus ring, no click
//! behaviour (spec §8). The only hover behaviour observable on span badges is
//! nova ghost (`hover:bg-muted hover:text-muted-foreground
//! dark:hover:bg-muted/50`) and nova link (`hover:underline`); every other
//! hover rule in both recipes is anchor-scoped (`[a&]:hover:` / `[a]:hover:`)
//! and has no egui expression (spec §11 D1). Badges have NO shadow in either
//! recipe (spec §2.3) and NO disabled state in the original (spec §5.5; the
//! Rust `.disabled()` builder is a documented extension, spec §11 D4).
//!
//! OPEN items (spec §12, all left open): O1 transition duration/easing — the
//! Phase 1 motion defaults (150ms + cubic-bezier(0.4, 0, 0.2, 1)) are used as
//! the INTERIM position, labelled as such; O2 `data-icon` source — the icon
//! slots map onto the per-side padding rules and the lone-icon width ambiguity
//! stays open; O3 Slot semantics — no egui analogue (D1); O4 font family —
//! Inter per Phase 1, not snapshot truth; O5 base height derivations
//! (22/18/6px) — implemented as derived; O6 invalid-ring visibility — no ring
//! is invented for invalid-unfocused; O7 nova `[a]:hover:` expansion — span
//! behaviour is unaffected under every reading.
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
use crate::theme::fonts;
use crate::theme::tokens::{spacing, type_scale, RadiusTokens};
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use crate::utils::animation::{
    mix_oklab, transition_alpha, transition_color_to, transition_progress, TransitionSpec,
};
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget};

/// `text-xs` paired line box (16px, Tailwind default): the content height of a
/// badge with text under the base recipe (spec §3.1 — a derivation, OPEN O5).
const LINE_BOX: f32 = 16.0;
/// `[&>svg]:size-3` — direct-child icon size, both recipes (spec §3.1).
const ICON_SIZE: f32 = 12.0;
/// `border` width, both recipes (spec §3.1).
const BORDER_WIDTH: f32 = 1.0;
/// Nova fixed height, `h-5` (spec §3.1).
const NOVA_HEIGHT: f32 = 20.0;
/// `underline-offset-4`: the link underline sits 4px below the text baseline
/// (spec §2.3).
const UNDERLINE_OFFSET: f32 = 4.0;

/// The label font: `text-xs font-medium` — Inter Medium 500 at 12px
/// (spec §3.1). The concrete family is OPEN O4; Inter is the Phase 1
/// embedding, not snapshot truth.
fn label_font() -> FontId {
    fonts::medium(type_scale::XS)
}

/// Left padding: `px-2` (8px), except 6px (`pl-1.5`) under nova when a start
/// icon is present (spec §3.1/§4; the `data-icon` mechanism is OPEN O2).
fn pad_left(has_start: bool, nova: bool) -> f32 {
    if nova && has_start {
        spacing::XS_HALF
    } else {
        spacing::SM
    }
}

/// Right padding: `px-2` (8px), except 6px (`pr-1.5`) under nova when an end
/// icon is present (spec §3.1/§4).
fn pad_right(has_end: bool, nova: bool) -> f32 {
    if nova && has_end {
        spacing::XS_HALF
    } else {
        spacing::SM
    }
}

/// Content width — the spec §4 flex-row sum: padding + item widths + gaps.
/// Every icon counts its full 12px and every gap between adjacent items
/// counts 4px; `text_w` is the rendered label width at 12px/medium.
///
/// An empty badge (no label, no icons) has no content: spec §10 sizes it at
/// padding + border only (18px wide in both recipes). The §4 tables all have
/// content, so the empty case is governed by §10's explicit statement.
fn content_width(text_w: f32, has_text: bool, has_start: bool, has_end: bool, nova: bool) -> f32 {
    let pad_l = pad_left(has_start, nova);
    let pad_r = pad_right(has_end, nova);
    if !has_text && !has_start && !has_end {
        return pad_l + pad_r + 2.0 * BORDER_WIDTH;
    }
    let icons_w = (has_start as u8 + has_end as u8) as f32 * ICON_SIZE;
    let n_items = has_start as u8 + has_end as u8 + has_text as u8;
    let gaps = (n_items - 1) as f32 * spacing::XS;
    pad_l + pad_r + icons_w + gaps + text_w
}

/// Badge height (spec §3.1). Nova fixes `h-5` (20px). The base height is
/// content-derived: line box (16px, O5 derivation) with text, icon height
/// (12px) for icon-only, nothing for empty — plus vertical padding (2+2)
/// plus border (1+1): 22/18/6px.
fn badge_height(has_text: bool, has_icon: bool, nova: bool) -> f32 {
    if nova {
        return NOVA_HEIGHT;
    }
    let content = if has_text {
        LINE_BOX
    } else if has_icon {
        ICON_SIZE
    } else {
        0.0
    };
    content + 2.0 * spacing::HALF + 2.0 * BORDER_WIDTH
}

/// Corner radius: `rounded-full` under base, `rounded-4xl` (26px at the
/// default `--radius`) under nova (spec §3.1). Both render as a pill at
/// badge heights; egui clamps the radius to half the rect height.
fn badge_radius(nova: bool, radius: &RadiusTokens) -> u8 {
    if nova {
        radius.x4l().round() as u8
    } else {
        radius.full_u8()
    }
}

/// Visual style variants, mirroring the source `badgeVariants` (spec §2).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    /// `bg-primary text-primary-foreground` (both recipes).
    #[default]
    Default,
    /// `bg-secondary text-secondary-foreground` (both recipes).
    Secondary,
    /// Base: `bg-destructive text-white dark:bg-destructive/60`. Nova:
    /// `bg-destructive/10 text-destructive dark:bg-destructive/20`.
    Destructive,
    /// `border-border text-foreground` (both recipes).
    Outline,
    /// Transparent, inherits the foreground color. Nova hover:
    /// `hover:bg-muted hover:text-muted-foreground dark:hover:bg-muted/50`.
    Ghost,
    /// `text-primary underline-offset-4`. Nova hover: `hover:underline`.
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

/// Resolves variant colors for the current style, palette and hover state
/// (spec §2.3, §5.2, §5.6).
///
/// Hover is the SPAN-badge hover state: under the base recipe no variant
/// changes on hover (all rules are anchor-scoped); under nova only ghost
/// (fill + text) and link (underline) change. When `invalid`, the border
/// becomes `destructive` at full alpha in both themes and fill/text are
/// unchanged (spec §5.6); no ring is resolved — the ring width comes only
/// from the `focus-visible` rule, which never fires for span badges, and
/// invalid-ring visibility is OPEN O6.
pub(crate) fn resolve_colors(
    variant: BadgeVariant,
    p: &Palette,
    dark: bool,
    hovered: bool,
    style: Style,
    invalid: bool,
) -> Colors {
    let nova = style == Style::Nova;
    let mut c = match variant {
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
            stroke: Stroke::new(BORDER_WIDTH, p.border),
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
    };
    if invalid {
        // `aria-invalid:border-destructive` (spec §5.6): border only, never
        // fill or text; full alpha in both themes (no dark override exists
        // for badge, unlike button).
        c.stroke = Stroke::new(BORDER_WIDTH, p.destructive);
    }
    c
}

/// Mixes two colors at `t` (0.0..1.0) for the §7 transition. Opaque
/// endpoints mix in OKLab; an endpoint with zero alpha fades by ramping
/// the other endpoint's alpha at a constant hue, because mixing toward
/// transparent in OKLab dips through black (a fully transparent endpoint
/// has no defined hue). An endpoint with partial alpha mixes in
/// premultiplied sRGB space instead: unmultiplied OKLab would sweep the
/// endpoint's full lightness while alpha rises, so a translucent colour
/// fading toward an opaque one bulges brighter than both endpoints when
/// composited. Lerping the stored premultiplied channels keeps the
/// composited value linear in `t`, hence monotonic and within the
/// endpoint range for any backdrop. (Same boundary-safe rule as
/// `button::mix_color`, which pins the same cases.)
fn mix_color(a: Color32, b: Color32, t: f32) -> Color32 {
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

/// Mixes two resolved color sets at `t`, for the §7 transition. Stroke
/// width is taken from `a` (hover never changes a variant's border width);
/// the underline flag is taken from `a` and the caller overrides it with
/// the target hover state, since underlines snap in CSS (text-decoration
/// is not animatable).
fn mix_colors(a: Colors, b: Colors, t: f32) -> Colors {
    Colors {
        fill: mix_color(a.fill, b.fill, t),
        fg: mix_color(a.fg, b.fg, t),
        stroke: Stroke::new(a.stroke.width, mix_color(a.stroke.color, b.stroke.color, t)),
        underline: a.underline,
    }
}

/// Fades every channel of a resolved color set by `opacity` (the D4
/// disabled extension composites the whole element at 50%).
fn fade_colors(c: Colors, opacity: f32) -> Colors {
    Colors {
        fill: color_alpha(c.fill, opacity),
        fg: color_alpha(c.fg, opacity),
        stroke: Stroke::new(c.stroke.width, color_alpha(c.stroke.color, opacity)),
        underline: c.underline,
    }
}

/// The stroke colour fed to the endpoint animator. A zero-width stroke is
/// never painted, but its colour must still be hue-defined: feeding a
/// fully transparent target would run OKLab legs through black when the
/// border appears (invalid flip). The border token is the placeholder
/// hue; it is invisible while the width is 0.
fn stroke_anim_color(c: &Colors, border: Color32) -> Color32 {
    if c.stroke.width > 0.0 {
        c.stroke.color
    } else {
        border
    }
}

/// Wraps one resolved endpoint set's fill/foreground/stroke in the
/// endpoint-change animator ([`transition_color_to`]), for the §7
/// transition over theme/style switches: in the browser, a theme switch
/// recolouring a badge animates over the same duration as any other colour
/// change, because CSS transitions each property from its current computed
/// value toward the new one.
///
/// The legs run on the resolved endpoints — whose targets never cross the
/// transparent boundary between themes (opaque in both, transparent in
/// both, or translucent-but-defined in both; pinned by the test
/// `endpoint_slots_never_cross_the_transparent_boundary`) — so the
/// helper's documented caveat holds, and state mixing blends leg values
/// with the boundary-safe [`mix_color`].
fn theme_animate_endpoint(
    ctx: &egui::Context,
    id: egui::Id,
    c: Colors,
    border: Color32,
    spec: TransitionSpec,
) -> Colors {
    Colors {
        fill: transition_color_to(ctx, id.with("fill"), c.fill, spec).color,
        fg: transition_color_to(ctx, id.with("fg"), c.fg, spec).color,
        stroke: Stroke::new(
            c.stroke.width,
            transition_color_to(ctx, id.with("stroke"), stroke_anim_color(&c, border), spec).color,
        ),
        underline: c.underline,
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
    invalid: bool,
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
            invalid: false,
            id_salt: None,
        }
    }

    /// Sets the visual variant (default: [`BadgeVariant::Default`]).
    pub fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows a 12px icon before the label (the `[&>svg]:size-3` slot, spec
    /// §5.8). Under nova, a start icon also narrows the left padding to
    /// 6px (`has-data-[icon=inline-start]:pl-1.5`, spec §4).
    pub fn icon_start(mut self, icon: Icon) -> Self {
        self.icon_start = Some(icon);
        self
    }

    /// Shows a 12px icon after the label (the `[&>svg]:size-3` slot, spec
    /// §5.8). Under nova, an end icon also narrows the right padding to
    /// 6px (`has-data-[icon=inline-end]:pr-1.5`, spec §4).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    /// Renders at 50% opacity, like a disabled tag.
    ///
    /// RUST EXTENSION (spec §11 D4): the original defines NO disabled badge
    /// state (spec §5.5) — no `disabled` prop, no `disabled:*` class. This
    /// treatment must not be judged as a parity requirement. A disabled
    /// badge never shows hover state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the badge aria-invalid (`aria-invalid:border-destructive`,
    /// spec §5.6): the border becomes `destructive` at full alpha in both
    /// themes; fill and text are unchanged. No ring is painted: the ring
    /// width comes only from the `focus-visible` rule, which never fires
    /// for span badges, and whether an invalid-but-unfocused badge shows a
    /// ring is OPEN (spec §12 O6). The `aria-*` attribute itself has no
    /// egui runtime meaning (spec §11 D3); this builder is its visual
    /// analogue.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt (Badge keeps no state of its own; the id
    /// anchors the §7 transition animators).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the badge. Under nova, ghost and link badges respond to hover
    /// (`hover:bg-muted` / `hover:underline`); every other variant, and
    /// every variant under the base recipe, shows no hover change (spec
    /// §5.2). The badge is never a Tab stop and never takes focus (spec
    /// §8).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();
        let nova = theme.style == Style::Nova;

        // `whitespace-nowrap` (spec §3.2): the label never wraps; the badge
        // grows to fit the text.
        let galley =
            ui.painter()
                .layout_no_wrap(self.label.clone(), label_font(), Color32::PLACEHOLDER);
        let has_text = !self.label.is_empty();
        let has_start = self.icon_start.is_some();
        let has_end = self.icon_end.is_some();
        let width = content_width(galley.size().x, has_text, has_start, has_end, nova);
        let height = badge_height(has_text, has_start || has_end, nova);
        let desired = Vec2::new(width, height);

        let (_, rect) = ui.allocate_space(desired);
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_badge"),
        };
        // `Sense::hover()`: hover detection only — a span is not focusable
        // and never a Tab stop (spec §8, §11 D2).
        let response = ui.interact(rect, id, Sense::hover());

        if ui.is_rect_visible(rect) {
            // A disabled badge never shows hover state (D4 extension
            // behaviour).
            let hovered = response.hovered() && !self.disabled;
            // `transition-[color,box-shadow]` (base) / `transition-all`
            // (nova), spec §7. Duration/easing are the Phase 1 motion
            // defaults — the OPEN O1 interim position (150ms +
            // cubic-bezier(0.4, 0, 0.2, 1)), not snapshot-verified truth.
            // Every progress is driven on every visible frame so the first
            // hover animates from the resting state instead of snapping;
            // repaints are requested only while a transition runs.
            let t_spec = TransitionSpec::default();
            let ctx = ui.ctx();
            let t_hover = transition_progress(ctx, id.with("hover"), hovered, t_spec).t;
            let rest = theme_animate_endpoint(
                ctx,
                id.with("ep_rest"),
                resolve_colors(self.variant, &p, dark, false, theme.style, self.invalid),
                p.border,
                t_spec,
            );
            let hov = theme_animate_endpoint(
                ctx,
                id.with("ep_hover"),
                resolve_colors(self.variant, &p, dark, true, theme.style, self.invalid),
                p.border,
                t_spec,
            );
            let mut c = mix_colors(rest, hov, t_hover);
            // Underlines snap: text-decoration is not animatable.
            c.underline = self.variant == BadgeVariant::Link && hovered && nova;
            // Disabled fade (D4 extension) over the same transition.
            let opacity =
                transition_alpha(ctx, id.with("disabled"), 1.0, 0.5, self.disabled, t_spec).alpha;
            if opacity < 1.0 {
                c = fade_colors(c, opacity);
            }

            let painter = ui.painter();
            // No shadow: neither recipe declares one (spec §2.3).
            painter.rect_filled(rect, badge_radius(nova, &theme.radius), c.fill);
            if c.stroke.width > 0.0 {
                painter.rect_stroke(
                    rect,
                    badge_radius(nova, &theme.radius),
                    c.stroke,
                    egui::epaint::StrokeKind::Inside,
                );
            }

            // Content: [icon] gap [label] gap [icon], centred (spec §3.2).
            let center = rect.center();
            let inner_w = width - pad_left(has_start, nova) - pad_right(has_end, nova);
            let mut cursor = center.x - inner_w * 0.5;
            let gap = spacing::XS;
            if let Some(icon) = self.icon_start {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + ICON_SIZE * 0.5, center.y),
                    Vec2::splat(ICON_SIZE),
                );
                icon.paint(painter, r, c.fg, 2.0 * ICON_SIZE / 24.0);
                cursor += ICON_SIZE + gap;
            }
            if !self.label.is_empty() {
                let text_center = Pos2::new(cursor + galley.size().x * 0.5, center.y);
                // Same paint math as `Painter::text`, reusing the galley.
                let text_rect = Align2::CENTER_CENTER.anchor_size(text_center, galley.size());
                painter.galley(text_rect.min, galley.clone(), c.fg);
                if c.underline {
                    // `underline-offset-4` (spec §2.3): 4px below the text
                    // baseline. The baseline comes from the laid-out
                    // galley: the first glyph's baseline position relative
                    // to its row, plus the row's offset in the galley.
                    let baseline = galley.rows.first().and_then(|placed| {
                        placed
                            .row
                            .glyphs
                            .first()
                            .map(|glyph| placed.pos.y + glyph.pos.y)
                    });
                    if let Some(baseline) = baseline {
                        let y = text_rect.min.y + baseline + UNDERLINE_OFFSET;
                        painter.line_segment(
                            [Pos2::new(cursor, y), Pos2::new(cursor + galley.size().x, y)],
                            Stroke::new(1.0, c.fg),
                        );
                    }
                }
                cursor += galley.size().x + gap;
            }
            if let Some(icon) = self.icon_end {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + ICON_SIZE * 0.5, center.y),
                    Vec2::splat(ICON_SIZE),
                );
                icon.paint(painter, r, c.fg, 2.0 * ICON_SIZE / 24.0);
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

    const ALL_VARIANTS: [BadgeVariant; 6] = [
        BadgeVariant::Default,
        BadgeVariant::Secondary,
        BadgeVariant::Destructive,
        BadgeVariant::Outline,
        BadgeVariant::Ghost,
        BadgeVariant::Link,
    ];

    #[test]
    fn default_uses_primary() {
        let p = palette(Mode::Light);
        let c = resolve_colors(BadgeVariant::Default, &p, false, false, Style::Vega, false);
        assert_eq!(c.fill, p.primary);
        assert_eq!(c.fg, p.primary_foreground);
        assert_eq!(c.stroke.width, 0.0);
    }

    #[test]
    fn base_destructive_is_solid_and_tinted_in_dark_mode() {
        let p = palette(Mode::Light);
        let c = resolve_colors(
            BadgeVariant::Destructive,
            &p,
            false,
            false,
            Style::Vega,
            false,
        );
        assert_eq!(c.fill, p.destructive);
        assert_eq!(c.fg, Color32::WHITE);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(
            BadgeVariant::Destructive,
            &dark,
            true,
            false,
            Style::Vega,
            false,
        );
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.6));
    }

    #[test]
    fn nova_destructive_is_tinted() {
        // `bg-destructive/10 text-destructive dark:bg-destructive/20`.
        let p = palette(Mode::Light);
        let c = resolve_colors(
            BadgeVariant::Destructive,
            &p,
            false,
            false,
            Style::Nova,
            false,
        );
        assert_eq!(c.fill, color_alpha(p.destructive, 0.1));
        assert_eq!(c.fg, p.destructive);
        let dark = palette(Mode::Dark);
        let d = resolve_colors(
            BadgeVariant::Destructive,
            &dark,
            true,
            false,
            Style::Nova,
            false,
        );
        assert_eq!(d.fill, color_alpha(dark.destructive, 0.2));
        assert_eq!(d.fg, dark.destructive);
    }

    #[test]
    fn nova_ghost_hover_uses_muted() {
        let p = palette(Mode::Light);
        let rest = resolve_colors(BadgeVariant::Ghost, &p, false, false, Style::Nova, false);
        assert_eq!(rest.fill, Color32::TRANSPARENT);
        assert_eq!(rest.fg, p.foreground);
        let hov = resolve_colors(BadgeVariant::Ghost, &p, false, true, Style::Nova, false);
        assert_eq!(hov.fill, p.muted);
        assert_eq!(hov.fg, p.muted_foreground);
        // `dark:hover:bg-muted/50`
        let dark = palette(Mode::Dark);
        let hov_dark = resolve_colors(BadgeVariant::Ghost, &dark, true, true, Style::Nova, false);
        assert_eq!(hov_dark.fill, color_alpha(dark.muted, 0.5));
        // The base recipe's ghost hover is `[a&]:hover` (anchor-only): no-op.
        let base_hov = resolve_colors(BadgeVariant::Ghost, &p, false, true, Style::Vega, false);
        assert_eq!(base_hov.fill, Color32::TRANSPARENT);
    }

    #[test]
    fn link_underlines_on_nova_hover_only() {
        let p = palette(Mode::Light);
        let rest = resolve_colors(BadgeVariant::Link, &p, false, false, Style::Nova, false);
        assert!(!rest.underline);
        let hov = resolve_colors(BadgeVariant::Link, &p, false, true, Style::Nova, false);
        assert!(hov.underline);
        // Base link hover is anchor-only: no underline.
        let base_hov = resolve_colors(BadgeVariant::Link, &p, false, true, Style::Vega, false);
        assert!(!base_hov.underline);
    }

    #[test]
    fn outline_has_border_others_transparent() {
        let p = palette(Mode::Light);
        let o = resolve_colors(BadgeVariant::Outline, &p, false, false, Style::Vega, false);
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
                resolve_colors(v, &p, false, false, Style::Vega, false)
                    .stroke
                    .width,
                0.0
            );
        }
        assert_eq!(
            resolve_colors(BadgeVariant::Ghost, &p, false, false, Style::Vega, false).fg,
            p.foreground
        );
        assert_eq!(
            resolve_colors(BadgeVariant::Link, &p, false, false, Style::Vega, false).fg,
            p.primary
        );
    }

    #[test]
    fn resting_colors_match_spec_section_2_3() {
        // Every variant × recipe × mode, expectations written from the
        // spec §2.3 tables (not from the implementation).
        for mode in [Mode::Light, Mode::Dark] {
            let dark = mode.is_dark();
            let p = palette(mode);
            for style in [Style::Vega, Style::Nova] {
                let nova = style == Style::Nova;
                let c = resolve_colors(BadgeVariant::Default, &p, dark, false, style, false);
                assert_eq!(c.fill, p.primary);
                assert_eq!(c.fg, p.primary_foreground);
                assert_eq!(c.stroke.width, 0.0);
                assert!(!c.underline);

                let c = resolve_colors(BadgeVariant::Secondary, &p, dark, false, style, false);
                assert_eq!(c.fill, p.secondary);
                assert_eq!(c.fg, p.secondary_foreground);
                assert_eq!(c.stroke.width, 0.0);
                assert!(!c.underline);

                let c = resolve_colors(BadgeVariant::Destructive, &p, dark, false, style, false);
                if nova {
                    assert_eq!(
                        c.fill,
                        color_alpha(p.destructive, if dark { 0.2 } else { 0.1 })
                    );
                    assert_eq!(c.fg, p.destructive);
                } else {
                    assert_eq!(
                        c.fill,
                        if dark {
                            color_alpha(p.destructive, 0.6)
                        } else {
                            p.destructive
                        }
                    );
                    assert_eq!(c.fg, Color32::WHITE);
                }
                assert_eq!(c.stroke.width, 0.0);
                assert!(!c.underline);

                let c = resolve_colors(BadgeVariant::Outline, &p, dark, false, style, false);
                assert_eq!(c.fill, Color32::TRANSPARENT);
                assert_eq!(c.fg, p.foreground);
                assert_eq!(c.stroke.width, 1.0);
                assert_eq!(c.stroke.color, p.border);
                assert!(!c.underline);

                let c = resolve_colors(BadgeVariant::Ghost, &p, dark, false, style, false);
                assert_eq!(c.fill, Color32::TRANSPARENT);
                assert_eq!(c.fg, p.foreground);
                assert_eq!(c.stroke.width, 0.0);
                assert!(!c.underline);

                let c = resolve_colors(BadgeVariant::Link, &p, dark, false, style, false);
                assert_eq!(c.fill, Color32::TRANSPARENT);
                assert_eq!(c.fg, p.primary);
                assert_eq!(c.stroke.width, 0.0);
                assert!(!c.underline);
            }
        }
    }

    #[test]
    fn span_hover_changes_only_nova_ghost_and_link() {
        // Spec §5.2: every other hover rule is anchor-scoped, so a span
        // badge shows no hover change outside nova ghost/link.
        for mode in [Mode::Light, Mode::Dark] {
            let dark = mode.is_dark();
            let p = palette(mode);
            for style in [Style::Vega, Style::Nova] {
                let nova = style == Style::Nova;
                for v in ALL_VARIANTS {
                    let rest = resolve_colors(v, &p, dark, false, style, false);
                    let hov = resolve_colors(v, &p, dark, true, style, false);
                    match (nova, v) {
                        (true, BadgeVariant::Ghost) => {
                            assert_eq!(
                                hov.fill,
                                if dark {
                                    color_alpha(p.muted, 0.5)
                                } else {
                                    p.muted
                                }
                            );
                            assert_eq!(hov.fg, p.muted_foreground);
                            assert_eq!(hov.stroke.width, rest.stroke.width);
                            assert!(!hov.underline);
                        }
                        (true, BadgeVariant::Link) => {
                            assert_eq!(hov.fill, rest.fill);
                            assert_eq!(hov.fg, rest.fg);
                            assert_eq!(hov.stroke.width, rest.stroke.width);
                            assert!(hov.underline);
                            assert!(!rest.underline);
                        }
                        _ => {
                            assert_eq!(hov.fill, rest.fill);
                            assert_eq!(hov.fg, rest.fg);
                            assert_eq!(hov.stroke.width, rest.stroke.width);
                            assert_eq!(hov.stroke.color, rest.stroke.color);
                            assert_eq!(hov.underline, rest.underline);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn nova_ghost_dark_hover_uses_muted_not_accent() {
        let dark = palette(Mode::Dark);
        // The slot choice is observable: dark muted (0.269) != dark accent
        // (0.371, Position A).
        assert_ne!(dark.muted, dark.accent);
        let hov = resolve_colors(BadgeVariant::Ghost, &dark, true, true, Style::Nova, false);
        assert_eq!(hov.fill, color_alpha(dark.muted, 0.5));
        assert_eq!(hov.fg, dark.muted_foreground);
    }

    #[test]
    fn invalid_paints_destructive_border_only() {
        // Spec §5.6: border `destructive` at full alpha in both themes;
        // fill and text unchanged; no ring resolved (OPEN O6).
        for mode in [Mode::Light, Mode::Dark] {
            let dark = mode.is_dark();
            let p = palette(mode);
            for style in [Style::Vega, Style::Nova] {
                for v in ALL_VARIANTS {
                    let plain = resolve_colors(v, &p, dark, false, style, false);
                    let inv = resolve_colors(v, &p, dark, false, style, true);
                    assert_eq!(inv.fill, plain.fill);
                    assert_eq!(inv.fg, plain.fg);
                    assert_eq!(inv.stroke.width, 1.0);
                    assert_eq!(inv.stroke.color, p.destructive);
                    assert_eq!(inv.underline, plain.underline);
                    // Hovered + invalid: the same border.
                    let inv_hov = resolve_colors(v, &p, dark, true, style, true);
                    assert_eq!(inv_hov.stroke.width, 1.0);
                    assert_eq!(inv_hov.stroke.color, p.destructive);
                }
            }
        }
    }

    #[test]
    fn stroke_is_at_most_the_1px_border() {
        // The resolved surface carries only fill, foreground, a 0/1px
        // border stroke, and the underline flag: no ring width, no shadow
        // layer. Painted output (no shadow call in `show`) is S5's scope;
        // this pins the resolver's surface.
        for mode in [Mode::Light, Mode::Dark] {
            let dark = mode.is_dark();
            let p = palette(mode);
            for style in [Style::Vega, Style::Nova] {
                for v in ALL_VARIANTS {
                    for hovered in [false, true] {
                        for invalid in [false, true] {
                            let c = resolve_colors(v, &p, dark, hovered, style, invalid);
                            assert!(c.stroke.width == 0.0 || c.stroke.width == 1.0);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn width_follows_the_section_4_tables() {
        let t = 40.0;
        // Base recipe: padding 8/8 always, icon 12, gap 4.
        assert_eq!(content_width(t, true, false, false, false), t + 16.0);
        assert_eq!(content_width(t, true, true, false, false), t + 32.0);
        assert_eq!(content_width(t, true, false, true, false), t + 32.0);
        assert_eq!(content_width(t, true, true, true, false), t + 48.0);
        assert_eq!(content_width(0.0, false, true, false, false), 28.0);
        assert_eq!(content_width(0.0, false, false, true, false), 28.0);
        // Nova recipe: 6px on the icon side(s) only.
        assert_eq!(content_width(t, true, false, false, true), t + 16.0);
        assert_eq!(content_width(t, true, true, false, true), t + 30.0);
        assert_eq!(content_width(t, true, false, true, true), t + 30.0);
        assert_eq!(content_width(t, true, true, true, true), t + 44.0);
        assert_eq!(content_width(0.0, false, true, false, true), 26.0);
        assert_eq!(content_width(0.0, false, false, true, true), 26.0);
        // Empty: padding + border only (§10), both recipes.
        assert_eq!(content_width(0.0, false, false, false, false), 18.0);
        assert_eq!(content_width(0.0, false, false, false, true), 18.0);
        // No cap, no truncation: width grows 1:1 with text (§10 long text).
        assert_eq!(
            content_width(1000.0, true, false, false, false)
                - content_width(10.0, true, false, false, false),
            990.0
        );
    }

    #[test]
    fn height_follows_the_section_3_derivations() {
        // Base: 16 line box (O5) / 12 icon / 0 + 4 padding + 2 border.
        assert_eq!(badge_height(true, false, false), 22.0);
        assert_eq!(badge_height(true, true, false), 22.0);
        assert_eq!(badge_height(false, true, false), 18.0);
        assert_eq!(badge_height(false, false, false), 6.0);
        // Nova: fixed `h-5` whatever the content.
        for (has_text, has_icon) in [(true, false), (true, true), (false, true), (false, false)] {
            assert_eq!(badge_height(has_text, has_icon, true), 20.0);
        }
    }

    #[test]
    fn padding_is_per_side_under_nova_only() {
        assert_eq!(pad_left(false, false), 8.0);
        assert_eq!(pad_left(true, false), 8.0);
        assert_eq!(pad_left(false, true), 8.0);
        assert_eq!(pad_left(true, true), 6.0);
        assert_eq!(pad_right(false, false), 8.0);
        assert_eq!(pad_right(true, false), 8.0);
        assert_eq!(pad_right(false, true), 8.0);
        assert_eq!(pad_right(true, true), 6.0);
    }

    #[test]
    fn radius_is_pill_in_both_recipes() {
        let r = RadiusTokens::default(); // `--radius` 10px.
        assert_eq!(badge_radius(true, &r), 26); // `rounded-4xl` = 2.6 * 10.
        assert_eq!(badge_radius(false, &r), u8::MAX); // `rounded-full`.
        let scaled = RadiusTokens::new(8.0);
        assert_eq!(badge_radius(true, &scaled), 21); // 2.6 * 8 = 20.8 -> 21.
    }

    #[test]
    fn label_font_is_inter_medium_12() {
        let f = label_font();
        assert_eq!(f, fonts::medium(12.0));
        assert_eq!(f.family, fonts::medium_family());
        assert_eq!(f.size, 12.0);
    }

    #[test]
    fn endpoint_slots_never_cross_the_transparent_boundary() {
        // `transition_color_to` legs run on the resolved endpoints when the
        // theme switches; a leg crossing the transparent boundary dips
        // through black. Pin: for a fixed state, each animated slot is
        // transparent in both themes or in neither.
        for v in ALL_VARIANTS {
            for hovered in [false, true] {
                for invalid in [false, true] {
                    for style in [Style::Vega, Style::Nova] {
                        let light_p = palette(Mode::Light);
                        let dark_p = palette(Mode::Dark);
                        let l = resolve_colors(v, &light_p, false, hovered, style, invalid);
                        let d = resolve_colors(v, &dark_p, true, hovered, style, invalid);
                        assert_eq!(l.fill.a() == 0, d.fill.a() == 0);
                        assert_eq!(l.fg.a() == 0, d.fg.a() == 0);
                        assert_eq!(
                            stroke_anim_color(&l, light_p.border).a() == 0,
                            stroke_anim_color(&d, dark_p.border).a() == 0
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn mix_color_endpoints_exact_and_transparent_safe() {
        let a = Color32::from_rgb(10, 20, 30);
        let b = Color32::from_rgb(200, 100, 50);
        assert_eq!(mix_color(a, b, 0.0), a);
        assert_eq!(mix_color(a, b, 1.0), b);
        // Transparent -> opaque ramps alpha at a constant hue (no black dip).
        assert_eq!(mix_color(Color32::TRANSPARENT, b, 0.5), color_alpha(b, 0.5));
        assert_eq!(mix_color(a, Color32::TRANSPARENT, 0.5), color_alpha(a, 0.5));
    }

    #[test]
    fn builder_fields() {
        let b = Badge::new("x")
            .variant(BadgeVariant::Ghost)
            .icon_start(Icon::Plus)
            .icon_end(Icon::X)
            .disabled(true)
            .invalid(true)
            .id_salt("b");
        assert_eq!(b.variant, BadgeVariant::Ghost);
        assert!(b.icon_start.is_some() && b.icon_end.is_some());
        assert!(b.disabled);
        assert!(b.invalid);
        assert_eq!(b.id_salt.as_deref(), Some("b"));
    }
}
