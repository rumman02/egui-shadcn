//! shadcn `toggle.tsx` — a two-state pressed/unpressed button.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/toggle.tsx` (`toggleVariants`):
//! `inline-flex items-center justify-center gap-2 rounded-md text-sm
//! font-medium` with `hover:bg-muted hover:text-muted-foreground`,
//! `data-[state=on]:bg-accent data-[state=on]:text-accent-foreground`, and
//! variant/size presets:
//!
//! | variant | classes |
//! |---|---|
//! | default | `bg-transparent` |
//! | outline | `border border-input bg-transparent shadow-xs hover:bg-accent` |
//!
//! | size | classes |
//! |---|---|
//! | default | `h-9 min-w-9 px-2` |
//! | sm | `h-8 min-w-8 px-1.5` |
//! | lg | `h-10 min-w-10 px-2.5` |
//!
//! The nova style (`.cn-toggle*`) tightens the geometry (`h-8/h-7/h-9`,
//! `px-2.5`, `gap-1`, `rounded-lg`, sm `rounded-[min(var(--radius-md),12px)]`
//! with a 12.8pt label and 14pt icon), moves the pressed fill to `bg-muted`
//! and the hover text to `text-foreground`, and — under the binding LAYER
//! ruling (Decision S-1, frozen spec §0) — KEEPS the outline `shadow-xs`:
//! no `.cn-toggle*` rule declares a shadow property, so silence keeps the
//! base value. The `metrics` module holds those overrides; the
//! [`ToggleSize`] methods keep the base recipe.
//!
//! Animation (frozen spec §6 as corrected by the narrow-reading ruling):
//! under base, `transition-[color,box-shadow]` animates text colour and
//! box-shadow (the focus ring) while border/fill snap; under nova,
//! `transition-all` animates every property. Timing is the interim
//! Tailwind v4 framework defaults (150ms, cubic-bezier(0.4,0,0.2,1)),
//! labelled INTERIM per OPEN-5.
//!
//! Disabled (spec §4.6) is a whole-element group fade: the enabled chrome
//! composites at 50% over the theme background — not a per-slot alpha
//! halving, which would double-fade the border-over-fill and glyph-over-fill
//! pixels. See `disabled_group_fade`.
//!
//! Inside a [`crate::components::toggle_group::ToggleGroup`] with `spacing 0`
//! (or a `ButtonGroup`), corners and borders fuse automatically via the
//! shared group protocol in `crate::components::button`; fusing is border
//! removal, not an inset — the fill paints the full rect and the fused
//! stroke omits the shared edge (BTG-VIS-01 pattern). Fused items drop the
//! outline shadow (`data-[spacing=0]:shadow-none`) and queue their focus
//! ring above siblings (`focus:z-10`, FG-3).
//!
//! The nova icon-inline padding reductions (spec §3.3) apply only when the
//! caller asserts the icon carries `data-icon="inline-start"` via
//! [`Toggle::icon_inline_start`]: what sets `data-icon` is OPEN-6 (nothing
//! in the snapshot sets it), so no trigger is inferred from icon presence.
//! The padding VALUES are frozen; the trigger mapping is the S1 proposal,
//! pending S8 ratification.
//!
//! Keyboard (spec §7, OPEN-2 CLOSED by S6 Work B from public Radix Toggle
//! source): Space toggles on KEYUP, Enter on keydown — the native
//! `<button>` asymmetry (Radix attaches only `onClick`, no key handlers).
//! egui reports a fake click on the keydown frame for both keys, so this
//! build intervenes for Space (see `space_keyup_fire`, the button
//! BTN-INT-04 shape); Enter is never touched. A direct click focuses AND
//! toggles (TOG-INT-02): egui's `interact` never grants focus on a pointer
//! click, so `show` requests focus explicitly on `clicked()` (the button
//! BTN-INT-01 shape); press-drag-off-release focuses nothing.
//! `:focus-visible` modality is deviation D1 (PROPOSED): the ring shows on
//! any focus. No ARIA tree exists in egui (D2, PROPOSED): pressed/invalid
//! state is exposed through the Rust API and rendered visually.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut bold = false;
//! Toggle::new(&mut bold)
//!     .text("Bold")
//!     .icon(Icon::Sparkles)
//!     .show(ui);
//! ```

use crate::components::attachment::mix;
use crate::components::button::{fuse_radius, group, GroupOrientation, GroupPosition};
use crate::icons::Icon;
use crate::theme::{blend, color_alpha, fonts, Palette, ShadcnTheme, Style};
use crate::utils::animation::{transition_alpha, transition_color_to, Easing, TransitionSpec};
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Align2, Color32, CornerRadius, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2, Widget,
};

/// Visual style variants, mirroring the source `toggleVariants.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleVariant {
    /// `bg-transparent` — no border, muted hover.
    #[default]
    Default,
    /// `border border-input bg-transparent shadow-xs hover:bg-accent`.
    Outline,
}

/// Size presets, mirroring the source `toggleVariants.size`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleSize {
    /// `h-9 min-w-9 px-2` (36px tall).
    #[default]
    Default,
    /// `h-8 min-w-8 px-1.5` (32px tall).
    Sm,
    /// `h-10 min-w-10 px-2.5` (40px tall).
    Lg,
}

impl ToggleSize {
    /// Outer height in points.
    pub fn height(self) -> f32 {
        match self {
            ToggleSize::Default => crate::theme::tokens::spacing::CONTROL,
            ToggleSize::Sm => crate::theme::tokens::spacing::CONTROL_SM,
            ToggleSize::Lg => crate::theme::tokens::spacing::CONTROL_LG,
        }
    }

    /// Minimum width in points (`min-w-9` etc.).
    pub fn min_width(self) -> f32 {
        self.height()
    }

    /// Horizontal padding in points (`px-2` etc.).
    pub fn padding_x(self) -> f32 {
        match self {
            ToggleSize::Default => crate::theme::tokens::spacing::SM,
            ToggleSize::Sm => crate::theme::tokens::spacing::XS_HALF,
            ToggleSize::Lg => crate::theme::tokens::spacing::SM_HALF,
        }
    }

    /// Gap between icon and label in the BASE recipe (`gap-2`). The paint
    /// path uses `metrics::gap` (8px base, 4px nova — spec §3.2, N6);
    /// this method keeps the base value only, like the other
    /// [`ToggleSize`] methods.
    pub fn gap(self) -> f32 {
        crate::theme::tokens::spacing::SM
    }

    /// Icon size in points (`size-4`).
    pub fn icon_size(self) -> f32 {
        crate::theme::tokens::spacing::LG
    }
}

/// Per-style size metrics: the [`ToggleSize`] methods hold the base recipe
/// (`toggleVariants`); these helpers branch on [`Style`] so the nova
/// overrides (`.cn-toggle`, `.cn-toggle-size-*`) apply wherever the active
/// theme is known.
pub(crate) mod metrics {
    use super::ToggleSize;
    use crate::theme::{RadiusTokens, Style};

    /// Outer height (`h-9/h-8/h-10` base; nova `h-8/h-7/h-9`).
    pub(crate) fn height(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ToggleSize::Default => 32.0, // h-8
                ToggleSize::Sm => 28.0,      // h-7
                ToggleSize::Lg => 36.0,      // h-9
            },
            _ => size.height(),
        }
    }

    /// Minimum width (`min-w-9/min-w-8/min-w-10` base; nova
    /// `min-w-8/min-w-7/min-w-9`). Tracks [`height`] in every size/style
    /// (spec §3.1).
    pub(crate) fn min_width(size: ToggleSize, style: Style) -> f32 {
        height(size, style)
    }

    /// Horizontal padding per side: the size `px-*` values, with the nova
    /// icon-inline reductions (spec §3.3, N16) applied per side when the
    /// corresponding icon trigger holds. Base has no icon-inline
    /// adjustment: both sides keep the size padding whatever the triggers.
    pub(crate) fn padding_sides(
        size: ToggleSize,
        style: Style,
        icon_inline_start: bool,
        icon_inline_end: bool,
    ) -> (f32, f32) {
        match style {
            Style::Nova => {
                let inline = match size {
                    ToggleSize::Sm => 6.0, // pl-1.5 / pr-1.5
                    _ => 8.0,              // pl-2 / pr-2
                };
                (
                    if icon_inline_start { inline } else { 10.0 },
                    if icon_inline_end { inline } else { 10.0 },
                )
            }
            _ => {
                let px = size.padding_x();
                (px, px)
            }
        }
    }

    /// Gap between icon and label (`gap-2` base; nova `gap-1`).
    pub(crate) fn gap(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => 4.0, // gap-1
            _ => size.gap(),
        }
    }

    /// Icon size (`size-4`; nova sm `size-3.5`).
    pub(crate) fn icon_size(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ToggleSize::Sm => 14.0, // size-3.5
                _ => 16.0,              // size-4
            },
            _ => size.icon_size(),
        }
    }

    /// Label size in points (`text-sm`; nova sm `text-[0.8rem]`).
    pub(crate) fn text_size(size: ToggleSize, style: Style) -> f32 {
        match style {
            Style::Nova => match size {
                ToggleSize::Sm => 12.8, // text-[0.8rem]
                _ => 14.0,              // text-sm
            },
            _ => 14.0, // text-sm
        }
    }

    /// Corner radius: `rounded-md` base; nova `rounded-lg` with the sm size
    /// on `rounded-[min(var(--radius-md),12px)]`.
    pub(crate) fn radius(radius: &RadiusTokens, size: ToggleSize, style: Style) -> u8 {
        match style {
            Style::Nova => match size {
                ToggleSize::Sm => radius.md().min(12.0).round() as u8,
                _ => radius.lg_u8(),
            },
            _ => radius.md_u8(),
        }
    }
}

/// Resolved colors for one toggle state (pure, unit-testable). These are
/// the ENABLED rule values: the disabled group fade (§4.6) applies after
/// animation in the paint path, never here.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ToggleColors {
    fill: Color32,
    fg: Color32,
    stroke: Stroke,
    /// Focus ring color (alpha already applied). Visibility is
    /// focus-gated in the paint path — the color alone never shows a ring.
    ring: Color32,
}

/// The four boolean state inputs to `resolve_toggle_colors` (spec §4).
/// Grouped so the resolver stays within the argument-count lint and call
/// sites name the states they set.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ToggleState {
    pub(crate) hovered: bool,
    pub(crate) pressed: bool,
    pub(crate) focused: bool,
    pub(crate) invalid: bool,
}

/// Mirrors the base recipe: `hover:bg-muted hover:text-muted-foreground`,
/// `data-[state=on]:bg-accent data-[state=on]:text-accent-foreground`,
/// outline `border border-input shadow-xs`. Nova (`aria-pressed:bg-muted
/// data-[state=on]:bg-muted hover:text-foreground`) keeps the pressed text
/// `accent-foreground`, moves the pressed fill to `muted` and the hover
/// text to `foreground`, and branches the outline hover fill to `muted`
/// (spec §4.2). Base outline hover is `accent`/`accent-foreground`
/// (SETTLED, spec §4.2) — the shared-string hover rules do not apply to
/// the outline variant under the OPEN-7 merge reading.
pub(crate) fn resolve_toggle_colors(
    variant: ToggleVariant,
    p: &Palette,
    dark: bool,
    state: ToggleState,
    style: Style,
) -> ToggleColors {
    let ToggleState {
        hovered,
        pressed,
        focused,
        invalid,
    } = state;
    let (fill, fg) = if pressed {
        // PROVISIONAL (OPEN-3): pressed wins over hover. The snapshot
        // cannot settle pressed+hover precedence (equal-specificity
        // selectors, generated-CSS order unestablishable); S9 corrects
        // this ordering if OPEN-3 settles the other way.
        let fill = match style {
            Style::Nova => p.muted, // data-[state=on]:bg-muted
            _ => p.accent,          // data-[state=on]:bg-accent
        };
        (fill, p.accent_foreground)
    } else if hovered {
        let fill = match (variant, style) {
            // Base outline hover: `hover:bg-accent` (variant string wins
            // over the shared `hover:bg-muted` under OPEN-7).
            (ToggleVariant::Outline, s) if !matches!(s, Style::Nova) => p.accent,
            // Nova outline hover: `hover:bg-muted` (explicit in
            // `.cn-toggle-variant-outline`); default hover: `hover:bg-muted`
            // (shared string, surviving under nova by silence).
            _ => p.muted,
        };
        let fg = match style {
            Style::Nova => p.foreground, // hover:text-foreground
            _ => match variant {
                ToggleVariant::Outline => p.accent_foreground,
                ToggleVariant::Default => p.muted_foreground,
            },
        };
        (fill, fg)
    } else {
        (Color32::TRANSPARENT, p.foreground)
    };
    let mut stroke = match variant {
        ToggleVariant::Default => Stroke::NONE,
        // `border-input`: the `input` slot in BOTH themes (spec §2.2/§5).
        ToggleVariant::Outline => Stroke::new(1.0, p.input),
    };
    if invalid && stroke.width > 0.0 {
        // Invalid border: full `destructive` in BOTH styles and BOTH
        // themes — no dark `/50` variant exists for toggle (spec §0
        // correction 1; N5 does not apply). The default variant has no
        // border width, so no visible invalid border is added.
        stroke.color = p.destructive;
    }
    // PROVISIONAL (OPEN-4): invalid wins over focus for the border colour.
    if focused && !invalid && stroke.width > 0.0 {
        stroke.color = p.ring; // `focus-visible:border-ring`
    }
    // PROVISIONAL (OPEN-4): invalid wins over focus for the ring colour.
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    ToggleColors {
        fill,
        fg,
        stroke,
        ring,
    }
}

/// Whether the outline variant paints its `shadow-xs`: standalone toggles
/// do, in ALL styles (Decision S-1 — nova silence keeps the base value);
/// items fused into a group (`data-[spacing=0]`) render `shadow-none` in
/// both styles (toggle-group spec §2.2).
pub(crate) fn outline_has_shadow(position: GroupPosition) -> bool {
    position == GroupPosition::Only
}

/// Per-property transition specs (frozen spec §6 as corrected by the
/// narrow-reading ruling; timing INTERIM per OPEN-5 — the Tailwind v4
/// framework defaults, not snapshot truth).
#[derive(Clone, Copy, Debug)]
struct AnimSpecs {
    /// Text colour: animates in both styles (`color` is in the base
    /// transition list; `transition-all` under nova).
    text: TransitionSpec,
    /// Fill + border colour: SNAP under base (neither `background-color`
    /// nor `border-color` is in `transition-[color,box-shadow]`); animate
    /// under nova (`transition-all`).
    fill_border: TransitionSpec,
    /// Ring (a `box-shadow` layer): animates in both styles.
    ring: TransitionSpec,
}

fn anim_specs(style: Style) -> AnimSpecs {
    // A zero-duration spec snaps instantly while keeping the animator
    // state fresh across style switches (input/checkbox precedent).
    let snap = TransitionSpec::secs(0.0, Easing::EaseInOut);
    let live = TransitionSpec::default(); // INTERIM per OPEN-5
    match style {
        Style::Nova => AnimSpecs {
            text: live,
            fill_border: live,
            ring: live,
        },
        _ => AnimSpecs {
            text: live,
            fill_border: snap,
            ring: live,
        },
    }
}

/// Opaque RGB the fill animator tracks: the current fill rule's RGB, with
/// the transparent resting endpoint replaced by the hover endpoint's RGB.
/// The transparent endpoint has no defined hue in the OKLab path (it would
/// fade through black); the alpha leg carries the actual transparency, so
/// the RGB value is invisible at rest, the hover leg is a pure fade, and
/// theme-switch recolours of the target animate under nova via endpoint
/// tracking. (Keyboard-only press from rest fades through the hover hue
/// at low alpha — recorded for S7; the settled values are exact.)
fn fill_rgb_target(variant: ToggleVariant, p: &Palette, pressed: bool, style: Style) -> Color32 {
    if pressed {
        match style {
            Style::Nova => p.muted,
            _ => p.accent,
        }
    } else {
        // Hover endpoint (also the resting state's would-be RGB).
        match (variant, style) {
            (ToggleVariant::Outline, s) if !matches!(s, Style::Nova) => p.accent,
            _ => p.muted,
        }
    }
}

/// Whole-element group fade for `disabled:opacity-50` (spec §4.6): the
/// enabled chrome composites at 50% over `backdrop` (the theme page
/// background — exact on the gallery page, where the panel fill is
/// `p.background`; on other backdrops the overlap pixels use the theme
/// background as the fade target, a recorded limitation).
///
/// Per-slot alpha halving would double-fade the overlap pixels (a halved
/// stroke over a halved fill reads 25% at the border band — the CHK-VIS-01
/// defect class). Instead: the fill fades by halving its alpha (it paints
/// over the backdrop alone — the shadow interior is masked, so nothing of
/// the toggle sits beneath it), while the stroke and text/icon colours are
/// pre-composited OPAQUE against the backdrop: `0.5 * composite + 0.5 *
/// backdrop`, where `composite` is the enabled stroke/glyph colour over
/// the enabled fill over the backdrop. Anti-aliased edge pixels stay exact
/// under this scheme: coverage `c` over the faded fill yields the group
/// fade of the enabled pixel. Returns `(fill, stroke_color, fg)`.
fn disabled_group_fade(
    fill: Color32,
    stroke_color: Color32,
    fg: Color32,
    backdrop: Color32,
) -> (Color32, Color32, Color32) {
    let faded_fill = color_alpha(fill, 0.5);
    let fill_composite = blend(backdrop, fill);
    let stroke = mix(backdrop, blend(fill_composite, stroke_color), 0.5);
    let fg = mix(backdrop, blend(fill_composite, fg), 0.5);
    (faded_fill, stroke, fg)
}

/// Pure content geometry: total content width and the child offsets,
/// computed from ONE gap value (spec §3.2, N6). `text_w` is 0 when there
/// is no non-empty text; a gap is counted only between present children.
/// The width computation and the paint path both consume this plan, so the
/// painted gap can never diverge from the measured gap.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ContentPlan {
    /// Icon + gaps + text, excluding padding.
    total_w: f32,
    /// Icon's left offset from the content start (0 when no icon).
    icon_offset: f32,
    /// Text's left offset from the content start.
    text_offset: f32,
}

fn content_plan(has_icon: bool, icon_w: f32, text_w: f32, gap: f32) -> ContentPlan {
    let has_text = text_w > 0.0;
    let gaps = if has_icon && has_text { gap } else { 0.0 };
    let icon_w = if has_icon { icon_w } else { 0.0 };
    ContentPlan {
        total_w: icon_w + gaps + text_w,
        icon_offset: 0.0,
        text_offset: icon_w + gaps,
    }
}

/// Content start offset from the rect's left edge: the left padding
/// plus half of any min-width slack, split evenly inside the content
/// box (the rect minus the per-side padding). With asymmetric padding
/// (N16 under nova) and an exactly-sized rect, the content starts
/// exactly at `pad_l`; symmetric cases reduce to centring (TOG-VIS-01).
fn content_cursor_x(rect_w: f32, pad_l: f32, pad_r: f32, total_w: f32) -> f32 {
    let slack = (rect_w - pad_l - pad_r - total_w).max(0.0);
    pad_l + slack * 0.5
}

/// `shadow-xs` for the outline variant, via the LAYER resolution
/// ([`ShadowSlot::ToggleOutline`]: Xs under all styles — nova is silent,
/// so the base value survives per Decision S-1). Paints in every state
/// including disabled: the disabled shadow is the same layers with halved
/// alphas (the §4.6 group fade over the backdrop-only shadow region).
/// Mirrors `paint_input_shadow`.
fn paint_toggle_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::ToggleOutline).layers();
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
            ShadowSlot::ToggleOutline,
            dark,
        );
    }
}

/// Paints the 1px border with the leading edge omitted for non-first group
/// children (toggle-group spec §2.2 border removal: `border-l-0` /
/// `border-t-0`), mirroring the foundation fusing fix in
/// [`crate::components::button::Button`] (`paint_fused_stroke`, BTG-VIS-01) and `paint_input_fused_stroke` —
/// both private, so the shape is repeated here: the full `Inside` ring is
/// painted through a clip that excludes the leading 1px column (horizontal
/// groups) or row (vertical groups), and once more clipped to each of the
/// two leading corner cells. The three clips tile the ring exactly: they
/// share only zero-area boundaries, and the renderer's scissor conversion
/// rounds every clip edge, so no pixel is painted twice — exact for
/// translucent strokes, where overpainting would double-blend. First/only
/// children take the plain `rect_stroke` path, identical to a standalone
/// toggle.
///
/// The fill paints on the FULL rect: border removal is achieved by omitting
/// the leading edge of the stroke, NOT by insetting the draw rect (the old
/// 1px inset left the shared column painted by neither member). The corner
/// cells MUST be painted as clipped copies of the ring shape, never as 1x1
/// fill rects: a 1x1 `rect_filled` does not rasterize in egui 0.37
/// (BTG-VIS-01 corner-cell failure).
fn paint_toggle_fused_stroke(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    stroke: Stroke,
    orientation: GroupOrientation,
    position: GroupPosition,
) {
    if !matches!(position, GroupPosition::Middle | GroupPosition::Last) {
        painter.rect_stroke(rect, corner, stroke, egui::epaint::StrokeKind::Inside);
        return;
    }
    let (clip, corners) = match orientation {
        GroupOrientation::Horizontal => (
            Rect::from_min_max(Pos2::new(rect.min.x + 1.0, rect.min.y), rect.max),
            [
                Rect::from_min_max(rect.min, Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0)),
                Rect::from_min_max(
                    Pos2::new(rect.min.x, rect.max.y - 1.0),
                    Pos2::new(rect.min.x + 1.0, rect.max.y),
                ),
            ],
        ),
        GroupOrientation::Vertical => (
            Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + 1.0), rect.max),
            [
                Rect::from_min_max(rect.min, Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0)),
                Rect::from_min_max(
                    Pos2::new(rect.max.x - 1.0, rect.min.y),
                    Pos2::new(rect.max.x, rect.min.y + 1.0),
                ),
            ],
        ),
    };
    painter.with_clip_rect(clip).rect_stroke(
        rect,
        corner,
        stroke,
        egui::epaint::StrokeKind::Inside,
    );
    // Corner pixels: the identical ring, clipped to each leading corner
    // cell. See the doc comment — 1x1 fill rects do not rasterize.
    for corner_clip in corners {
        painter.with_clip_rect(corner_clip).rect_stroke(
            rect,
            corner,
            stroke,
            egui::epaint::StrokeKind::Inside,
        );
    }
}

/// Space-keyup activation (spec §7, OPEN-2 closed by S6 Work B; TOG-INT-01;
/// the button BTN-INT-04 / switch SWI-INT-01 shape).
///
/// Native `<button>` toggles on Space KEYUP; egui's `interact` reports a
/// fake click whenever the focused widget sees `key_pressed(Space)`, so
/// without intervention Space toggles on keydown (and re-toggles on every
/// autorepeat). This helper, called before `interact` while the toggle
/// holds focus, hides this frame's Space keydown events from `interact`,
/// remembers that Space was pressed here ("armed"), and re-creates the
/// keydown on the keyup frame so `interact` reports the click then.
///
/// Only the event list `interact` reads is touched: `keys_down` is
/// computed in `begin_pass`, before widgets run, so `key_down(Space)` is
/// unaffected. Enter events are never touched (Enter toggles on keydown,
/// per spec). The synthetic keydown on the keyup frame can only fake-click
/// the focused widget, and copies the release event's physical key and
/// modifiers. A Space released without a preceding keydown on this
/// toggle (focus arrived mid-hold) does not toggle; a stale armed flag
/// is cleared on the first frame the toggle is shown with Space up.
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

/// A shadcn toggle. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Toggle<'a> {
    pressed: &'a mut bool,
    text: Option<String>,
    icon: Option<Icon>,
    variant: ToggleVariant,
    size: ToggleSize,
    padding_x_override: Option<f32>,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
    icon_inline_start: bool,
}

impl<'a> Toggle<'a> {
    /// Creates a toggle bound to the given pressed state.
    pub fn new(pressed: &'a mut bool) -> Self {
        Self {
            pressed,
            text: None,
            icon: None,
            variant: ToggleVariant::Default,
            size: ToggleSize::Default,
            padding_x_override: None,
            disabled: false,
            invalid: false,
            id_salt: None,
            icon_inline_start: false,
        }
    }

    /// Sets the label text.
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = Some(text.into());
        self
    }

    /// Shows an icon before the label.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets the visual variant (default: [`ToggleVariant::Default`]).
    pub fn variant(mut self, variant: ToggleVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the size (default: [`ToggleSize::Default`]).
    pub fn size(mut self, size: ToggleSize) -> Self {
        self.size = size;
        self
    }

    /// Overrides the horizontal padding. `ToggleGroupItem` uses `px-3`.
    pub fn padding_x(mut self, padding_x: f32) -> Self {
        self.padding_x_override = Some(padding_x);
        self
    }

    /// Disables the toggle (`disabled:pointer-events-none disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the toggle aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so focus state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Asserts the icon carries `data-icon="inline-start"`, applying the
    /// nova icon-inline padding reduction on the icon side (spec §3.3).
    /// No-op under base and when no icon is set. What sets `data-icon` is
    /// OPEN-6 (nothing in the snapshot sets it); this builder is the
    /// S1-proposed trigger mapping, pending S8 ratification — no trigger
    /// is inferred from icon presence.
    pub fn icon_inline_start(mut self, icon_inline_start: bool) -> Self {
        self.icon_inline_start = icon_inline_start;
        self
    }

    /// Horizontal padding per side: the override wins when set (group
    /// items); otherwise the size padding with the nova icon-inline
    /// reduction on the icon side when the caller asserted the trigger.
    /// There is no inline-end icon slot, so the end trigger never fires.
    fn padding_sides(&self, style: Style) -> (f32, f32) {
        if let Some(px) = self.padding_x_override {
            (px, px)
        } else {
            metrics::padding_sides(
                self.size,
                style,
                self.icon_inline_start && self.icon.is_some(),
                false,
            )
        }
    }

    /// Content width for the current text/icon, before `min_width`, plus
    /// the text galley laid out once.
    ///
    /// The galley uses [`Color32::PLACEHOLDER`] (egui's delayed-color
    /// layout: the paint-time fallback color replaces it exactly), so
    /// [`Toggle::show`] can paint this same galley instead of laying the
    /// text out again with the resolved foreground color. The galley is
    /// `None` exactly when there is no non-empty text. Text renders at
    /// Inter Medium 500 (`font-medium`, spec §8.2).
    fn content_layout(&self, ui: &Ui, style: Style) -> (f32, Option<std::sync::Arc<egui::Galley>>) {
        let galley = match self.text.as_deref() {
            Some(t) if !t.is_empty() => Some(ui.painter().layout_no_wrap(
                t.to_owned(),
                fonts::medium(metrics::text_size(self.size, style)),
                Color32::PLACEHOLDER,
            )),
            _ => None,
        };
        let text_w = galley.as_ref().map_or(0.0, |g| g.size().x);
        let plan = content_plan(
            self.icon.is_some(),
            metrics::icon_size(self.size, style),
            text_w,
            metrics::gap(self.size, style),
        );
        let (pad_l, pad_r) = self.padding_sides(style);
        (pad_l + plan.total_w + pad_r, galley)
    }

    /// Renders the toggle.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let style = theme.style;
        let size = self.size;

        // Auto-detected position inside a group (ButtonGroup / ToggleGroup
        // with spacing 0): corners fuse like `rounded-l-none`.
        let grp = group::next_child_position(ui);
        let position = grp.map(|(_, pos)| pos).unwrap_or(GroupPosition::Only);
        let orientation = grp.map(|(o, _)| o).unwrap_or(GroupOrientation::Horizontal);

        let height = metrics::height(size, style);
        let (content_w, text_galley) = self.content_layout(ui, style);
        let desired = Vec2::new(content_w.max(metrics::min_width(size, style)), height);
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_toggle"),
        };
        let (_, rect) = ui.allocate_space(desired);
        // Space toggles on keyup, not keydown (spec §7, OPEN-2 closed by S6
        // Work B; TOG-INT-01): hide this frame's Space keydown from
        // `interact` and re-create it on the keyup frame. Only while
        // focused and enabled — an unfocused toggle must not consume
        // another widget's Space, and a disabled toggle never activates.
        if !self.disabled && ui.ctx().memory(|m| m.has_focus(id)) {
            space_keyup_fire(ui.ctx(), id);
        }
        let response = ui.interact(rect, id, sense);

        // Activation (spec §7, OPEN-2 closed by S6 Work B): Space toggles
        // on keyup (see `space_keyup_fire`), Enter on keydown, pointer
        // click on the click event — the native `<button>` asymmetry.
        // A direct click ALSO focuses the toggle (TOG-INT-02): egui's
        // `interact` never grants focus on a pointer click (it only
        // surrenders focus elsewhere), so request it explicitly on click
        // — the button BTN-INT-01 / checkbox CHK-INT-01 shape. Gating on
        // `clicked()` (the release frame) keeps press-drag-off-release a
        // no-op, and the disabled gate rides the existing binding, so a
        // disabled toggle takes neither toggle nor focus. Requesting
        // before painting lets `has_focus()` below see the focus on the
        // click frame itself, so the ring (D1: any focus) shows at once.
        if response.clicked() && !self.disabled {
            *self.pressed = !*self.pressed;
            response.request_focus();
        }

        if ui.is_rect_visible(rect) {
            let pressed = *self.pressed;
            let hovered = response.hovered() && !self.disabled;
            // D1 (PROPOSED): the ring shows on any focus — egui cannot
            // distinguish keyboard from pointer focus (`:focus-visible`
            // modality). Disabled toggles never show the ring.
            let focused = response.has_focus() && !self.disabled;
            let c = resolve_toggle_colors(
                self.variant,
                &p,
                dark,
                ToggleState {
                    hovered,
                    pressed,
                    focused,
                    invalid: self.invalid,
                },
                style,
            );
            let specs = anim_specs(style);

            // Text colour animates in both styles (ruling 6: `color` is
            // in the base transition list; `transition-all` under nova).
            // Timing is INTERIM per OPEN-5. First paint is settled (no
            // fade-in on mount).
            let mut fg = transition_color_to(ui.ctx(), id.with("fg"), c.fg, specs.text).color;

            // Fill: RGB endpoint animator + scalar alpha animator (the
            // input pattern). The fill crosses the transparent boundary,
            // so OKLab mixing from the resting endpoint would fade
            // through black; the alpha leg carries the transparency
            // instead. Under base both legs snap (ruling 6); under nova
            // both ease.
            let fill_rgb = transition_color_to(
                ui.ctx(),
                id.with("fill_rgb"),
                fill_rgb_target(self.variant, &p, pressed, style),
                specs.fill_border,
            )
            .color;
            let fill_alpha = transition_alpha(
                ui.ctx(),
                id.with("fill_alpha"),
                0.0,
                1.0,
                c.fill != Color32::TRANSPARENT,
                specs.fill_border,
            )
            .alpha;
            let mut fill = color_alpha(fill_rgb, fill_alpha);

            // Border colour: endpoint tracking, so focus / invalid /
            // theme flips animate under nova and snap under base. The
            // slot never crosses the transparent boundary, so
            // `transition_color_to`'s caveat does not apply. The default
            // variant has no border: no animator runs for it.
            let mut stroke_color = c.stroke.color;
            if c.stroke.width > 0.0 {
                stroke_color = transition_color_to(
                    ui.ctx(),
                    id.with("border"),
                    c.stroke.color,
                    specs.fill_border,
                )
                .color;
            }

            // Ring: a `box-shadow` layer — colour + visibility both ride
            // the ring spec, which animates in both styles (ruling 6).
            // The ring is focus-gated in both styles (spec §4.5): no rest
            // ring for invalid toggles.
            let mut ring_color =
                transition_color_to(ui.ctx(), id.with("ring"), c.ring, specs.ring).color;
            let ring_alpha = transition_alpha(
                ui.ctx(),
                id.with("ring_alpha"),
                0.0,
                1.0,
                focused,
                specs.ring,
            )
            .alpha;

            if self.disabled {
                // Group fade (spec §4.6), applied AFTER animation so it
                // snaps: opacity is in neither transition set. See
                // `disabled_group_fade`.
                let (f, s, g) = disabled_group_fade(fill, stroke_color, fg, p.background);
                fill = f;
                stroke_color = s;
                fg = g;
                ring_color = color_alpha(ring_color, 0.5);
            }

            let radius = metrics::radius(&theme.radius, size, style);
            let corner = fuse_radius(CornerRadius::same(radius), orientation, position);
            let painter = ui.painter();

            // `shadow-xs` on the outline variant in ALL styles (Decision
            // S-1); fused group items render `shadow-none`. Paints in
            // every state including disabled (halved layers).
            if self.variant == ToggleVariant::Outline && outline_has_shadow(position) {
                paint_toggle_shadow(painter, rect, corner, style, dark, self.disabled);
            }

            // Fill on the FULL rect; fusing is border removal, not an
            // inset (BTG-VIS-01): the old 1px `draw_rect` shift left the
            // shared column painted by neither member.
            if fill != Color32::TRANSPARENT {
                painter.rect_filled(rect, corner, fill);
            }
            if c.stroke.width > 0.0 {
                paint_toggle_fused_stroke(
                    painter,
                    rect,
                    corner,
                    Stroke::new(1.0, stroke_color),
                    orientation,
                    position,
                );
            }

            // Content: [icon] gap [text] in the content box — the rect
            // minus the per-side padding (N16: asymmetric under nova
            // with the icon-inline trigger). Any min-width slack is
            // split evenly inside the content box, so symmetric and
            // min-width cases stay centred while the asymmetric case
            // starts exactly at the left padding (TOG-VIS-01: centring
            // the block in the full rect split the 2/4px reduction
            // across both sides). Same plan (and the same single gap
            // value) as the width computation (N6).
            let icon_size = metrics::icon_size(size, style);
            let text = self.text.as_deref().unwrap_or("");
            let text_w = text_galley.as_ref().map_or(0.0, |g| g.size().x);
            let plan = content_plan(
                self.icon.is_some(),
                icon_size,
                text_w,
                metrics::gap(size, style),
            );
            let center = rect.center();
            let (pad_l, pad_r) = self.padding_sides(style);
            let cursor = rect.min.x + content_cursor_x(rect.width(), pad_l, pad_r, plan.total_w);

            if let Some(icon) = self.icon {
                let icon_rect = Rect::from_center_size(
                    Pos2::new(cursor + plan.icon_offset + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, icon_rect, fg, 2.0 * icon_size / 24.0);
            }
            if !text.is_empty() {
                if let Some(galley) = text_galley {
                    // Same paint math as `Painter::text`, reusing the galley.
                    let text_rect = Align2::CENTER_CENTER.anchor_size(
                        Pos2::new(cursor + plan.text_offset + text_w * 0.5, center.y),
                        galley.size(),
                    );
                    painter.galley(text_rect.min, galley, fg);
                }
            }

            // Focus ring: a 3px band OUTSIDE the border box, immediately
            // adjacent to the border, following the effective (possibly
            // fused) corner radius — painted on the unexpanded rect,
            // since the `Outside` stroke starts at the rect edge. In a
            // group the ring is queued (FG-3, `focus:z-10`): it paints in
            // a foreground layer above the abutting siblings.
            if ring_alpha > 0.0 {
                let ring_stroke = Stroke::new(3.0, color_alpha(ring_color, ring_alpha));
                if grp.is_some() {
                    group::queue_ring(ui, rect, corner, ring_stroke);
                } else {
                    painter.rect_stroke(
                        rect,
                        corner,
                        ring_stroke,
                        egui::epaint::StrokeKind::Outside,
                    );
                }
            }
        }

        response
    }
}

impl Widget for Toggle<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset, Style};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn size_geometry_matches_source() {
        // default h-9 min-w-9 px-2, sm h-8 min-w-8 px-1.5, lg h-10 min-w-10 px-2.5
        assert_eq!(ToggleSize::Default.height(), 36.0);
        assert_eq!(ToggleSize::Default.min_width(), 36.0);
        assert_eq!(ToggleSize::Default.padding_x(), 8.0);
        assert_eq!(ToggleSize::Sm.height(), 32.0);
        assert_eq!(ToggleSize::Sm.min_width(), 32.0);
        assert_eq!(ToggleSize::Sm.padding_x(), 6.0);
        assert_eq!(ToggleSize::Lg.height(), 40.0);
        assert_eq!(ToggleSize::Lg.min_width(), 40.0);
        assert_eq!(ToggleSize::Lg.padding_x(), 10.0);
        // gap-2, icon size-4
        assert_eq!(ToggleSize::Default.gap(), 8.0);
        assert_eq!(ToggleSize::Default.icon_size(), 16.0);
    }

    #[test]
    fn colors_follow_state() {
        let p = palette(Mode::Light);
        // Off, resting: transparent, foreground, no border (default).
        let c = resolve_toggle_colors(
            ToggleVariant::Default,
            &p,
            false,
            ToggleState::default(),
            Style::Vega,
        );
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.fg, p.foreground);
        assert_eq!(c.stroke, Stroke::NONE);
        // Off, hovered, default variant: muted / muted-foreground.
        let c = resolve_toggle_colors(
            ToggleVariant::Default,
            &p,
            false,
            ToggleState {
                hovered: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.fill, p.muted);
        assert_eq!(c.fg, p.muted_foreground);
        // Off, hovered, OUTLINE variant, base: accent / accent-foreground
        // (SETTLED, spec §4.2 — the old code returned muted here).
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &p,
            false,
            ToggleState {
                hovered: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.fill, p.accent);
        assert_eq!(c.fg, p.accent_foreground);
        // Pressed: accent, regardless of hover (provisional, OPEN-3 —
        // pinned by name in `pressed_wins_over_hover_is_provisional`).
        let c = resolve_toggle_colors(
            ToggleVariant::Default,
            &p,
            false,
            ToggleState {
                hovered: true,
                pressed: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.fill, p.accent);
        assert_eq!(c.fg, p.accent_foreground);
        // Outline: `input` border in BOTH themes, transparent fill.
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &p,
            false,
            ToggleState::default(),
            Style::Vega,
        );
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.stroke.color, p.input);
        let dark = palette(Mode::Dark);
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &dark,
            true,
            ToggleState::default(),
            Style::Vega,
        );
        assert_eq!(c.stroke.color, dark.input);
        // Pressed outline keeps the border.
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &p,
            false,
            ToggleState {
                pressed: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.fill, p.accent);
        assert_eq!(c.stroke.color, p.input);
    }

    #[test]
    fn nova_pressed_and_hover_follow_the_nova_recipe() {
        let p = palette(Mode::Light);
        // Pressed: `data-[state=on]:bg-muted`, text stays accent-foreground.
        let c = resolve_toggle_colors(
            ToggleVariant::Default,
            &p,
            false,
            ToggleState {
                hovered: true,
                pressed: true,
                ..Default::default()
            },
            Style::Nova,
        );
        assert_eq!(c.fill, p.muted);
        assert_eq!(c.fg, p.accent_foreground);
        // Hover, default: bg stays muted, text moves to `hover:text-foreground`.
        let c = resolve_toggle_colors(
            ToggleVariant::Default,
            &p,
            false,
            ToggleState {
                hovered: true,
                ..Default::default()
            },
            Style::Nova,
        );
        assert_eq!(c.fill, p.muted);
        assert_eq!(c.fg, p.foreground);
        // Hover, outline, nova: `hover:bg-muted` + `hover:text-foreground`.
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &p,
            false,
            ToggleState {
                hovered: true,
                ..Default::default()
            },
            Style::Nova,
        );
        assert_eq!(c.fill, p.muted);
        assert_eq!(c.fg, p.foreground);
        // Base keeps muted / muted-foreground on default hover.
        let c = resolve_toggle_colors(
            ToggleVariant::Default,
            &p,
            false,
            ToggleState {
                hovered: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.fg, p.muted_foreground);
    }

    #[test]
    fn pressed_wins_over_hover_is_provisional_pending_open3() {
        // OPEN-3 is unfrozen: this ordering is the S1 implementation
        // choice, labelled provisional. S9 corrects it if OPEN-3 settles
        // the other way.
        let p = palette(Mode::Light);
        for style in [Style::Vega, Style::Nova] {
            let c = resolve_toggle_colors(
                ToggleVariant::Default,
                &p,
                false,
                ToggleState {
                    hovered: true,
                    pressed: true,
                    ..Default::default()
                },
                style,
            );
            let pressed_fill = if style == Style::Nova {
                p.muted
            } else {
                p.accent
            };
            assert_eq!(c.fill, pressed_fill);
            assert_eq!(c.fg, p.accent_foreground);
        }
    }

    #[test]
    fn invalid_wins_over_focus_is_provisional_pending_open4() {
        // OPEN-4 is unfrozen: this ordering is the S1 implementation
        // choice, labelled provisional. S9 corrects it if OPEN-4 settles
        // the other way.
        let p = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        // Invalid + focused, outline: destructive border + destructive
        // ring colour (light 20%).
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &p,
            false,
            ToggleState {
                focused: true,
                invalid: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.stroke.color, p.destructive);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
        // Dark: destructive ring colour at 40%.
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &dark,
            true,
            ToggleState {
                focused: true,
                invalid: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.ring, color_alpha(dark.destructive, 0.4));
        // Focused, valid: `ring` border + `ring/50` ring.
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &p,
            false,
            ToggleState {
                focused: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.stroke.color, p.ring);
        assert_eq!(c.ring, color_alpha(p.ring, 0.5));
    }

    #[test]
    fn invalid_border_only_where_a_border_exists() {
        // Default variant: no border width, so invalid adds no visible
        // border (spec §4.5). The ring COLOUR is still resolved (the
        // ring itself is focus-gated in the paint path).
        let p = palette(Mode::Light);
        let c = resolve_toggle_colors(
            ToggleVariant::Default,
            &p,
            false,
            ToggleState {
                invalid: true,
                ..Default::default()
            },
            Style::Vega,
        );
        assert_eq!(c.stroke, Stroke::NONE);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
    }

    #[test]
    fn invalid_border_is_full_destructive_in_dark_no_n5() {
        // N5 does NOT apply to toggle: no `destructive/50` anywhere in
        // the toggle rules (spec §0 correction 1). The dark invalid
        // border is full-alpha `destructive`.
        let dark = palette(Mode::Dark);
        let c = resolve_toggle_colors(
            ToggleVariant::Outline,
            &dark,
            true,
            ToggleState {
                invalid: true,
                ..Default::default()
            },
            Style::Nova,
        );
        assert_eq!(c.stroke.color, dark.destructive);
        assert_eq!(c.stroke.color.a(), 255);
    }

    #[test]
    fn nova_size_table_matches_recipe() {
        // nova default h-8 min-w-8 px-2.5, sm h-7 min-w-7 px-2.5, lg h-9 min-w-9 px-2.5
        assert_eq!(metrics::height(ToggleSize::Default, Style::Nova), 32.0);
        assert_eq!(metrics::height(ToggleSize::Sm, Style::Nova), 28.0);
        assert_eq!(metrics::height(ToggleSize::Lg, Style::Nova), 36.0);
        assert_eq!(metrics::min_width(ToggleSize::Default, Style::Nova), 32.0);
        assert_eq!(metrics::min_width(ToggleSize::Sm, Style::Nova), 28.0);
        assert_eq!(metrics::min_width(ToggleSize::Lg, Style::Nova), 36.0);
        assert_eq!(
            metrics::padding_sides(ToggleSize::Default, Style::Nova, false, false),
            (10.0, 10.0)
        );
        assert_eq!(
            metrics::padding_sides(ToggleSize::Sm, Style::Nova, false, false),
            (10.0, 10.0)
        );
        assert_eq!(
            metrics::padding_sides(ToggleSize::Lg, Style::Nova, false, false),
            (10.0, 10.0)
        );
        // nova gap-1, sm icon size-3.5, sm label text-[0.8rem]
        assert_eq!(metrics::gap(ToggleSize::Default, Style::Nova), 4.0);
        assert_eq!(metrics::icon_size(ToggleSize::Sm, Style::Nova), 14.0);
        assert_eq!(metrics::icon_size(ToggleSize::Default, Style::Nova), 16.0);
        assert_eq!(metrics::text_size(ToggleSize::Sm, Style::Nova), 12.8);
        assert_eq!(metrics::text_size(ToggleSize::Default, Style::Nova), 14.0);
        // base keeps the toggleVariants table
        assert_eq!(metrics::height(ToggleSize::Default, Style::Vega), 36.0);
        assert_eq!(metrics::height(ToggleSize::Sm, Style::Vega), 32.0);
        assert_eq!(metrics::height(ToggleSize::Lg, Style::Vega), 40.0);
        assert_eq!(metrics::min_width(ToggleSize::Default, Style::Vega), 36.0);
        assert_eq!(
            metrics::padding_sides(ToggleSize::Sm, Style::Vega, false, false),
            (6.0, 6.0)
        );
        assert_eq!(metrics::gap(ToggleSize::Default, Style::Vega), 8.0);
        assert_eq!(metrics::icon_size(ToggleSize::Sm, Style::Vega), 16.0);
        assert_eq!(metrics::text_size(ToggleSize::Sm, Style::Vega), 14.0);
    }

    #[test]
    fn nova_radius_is_lg_with_sm_cap() {
        let radius = crate::theme::RadiusTokens::new(10.0);
        assert_eq!(
            metrics::radius(&radius, ToggleSize::Default, Style::Nova),
            10
        ); // rounded-lg
           // sm: rounded-[min(var(--radius-md),12px)] = min(8, 12) = 8
        assert_eq!(metrics::radius(&radius, ToggleSize::Sm, Style::Nova), 8);
        assert_eq!(
            metrics::radius(&radius, ToggleSize::Default, Style::Vega),
            8
        );
        assert_eq!(metrics::radius(&radius, ToggleSize::Sm, Style::Vega), 8);
    }

    #[test]
    fn outline_shadow_survives_under_nova_s1() {
        use crate::components::button::GroupPosition;
        // Standalone: shadow-xs in every style (Decision S-1).
        assert!(outline_has_shadow(GroupPosition::Only));
        // Fused group items: data-[spacing=0]:shadow-none, both styles.
        assert!(!outline_has_shadow(GroupPosition::First));
        assert!(!outline_has_shadow(GroupPosition::Middle));
        assert!(!outline_has_shadow(GroupPosition::Last));
    }

    #[test]
    fn content_plan_uses_one_gap_value_n6() {
        // Icon + text: exactly one gap between the present children, at
        // the style's gap value; text starts after icon + gap.
        let plan = content_plan(true, 16.0, 40.0, 8.0);
        assert_eq!(plan.total_w, 64.0);
        assert_eq!(plan.icon_offset, 0.0);
        assert_eq!(plan.text_offset, 24.0);
        let plan = content_plan(true, 16.0, 40.0, 4.0);
        assert_eq!(plan.total_w, 60.0);
        assert_eq!(plan.text_offset, 20.0);
        // Single child: no gap. Empty text counts as no child.
        assert_eq!(content_plan(true, 16.0, 0.0, 8.0).total_w, 16.0);
        assert_eq!(content_plan(false, 16.0, 40.0, 8.0).total_w, 40.0);
        assert_eq!(content_plan(false, 16.0, 0.0, 8.0).total_w, 0.0);
    }

    #[test]
    fn icon_inline_padding_follows_the_frozen_table() {
        // Nova default/lg: 8px on the icon side, 10px on the other.
        assert_eq!(
            metrics::padding_sides(ToggleSize::Default, Style::Nova, true, false),
            (8.0, 10.0)
        );
        assert_eq!(
            metrics::padding_sides(ToggleSize::Default, Style::Nova, false, true),
            (10.0, 8.0)
        );
        assert_eq!(
            metrics::padding_sides(ToggleSize::Default, Style::Nova, true, true),
            (8.0, 8.0)
        );
        assert_eq!(
            metrics::padding_sides(ToggleSize::Lg, Style::Nova, true, false),
            (8.0, 10.0)
        );
        // Nova sm: 6px on the icon side.
        assert_eq!(
            metrics::padding_sides(ToggleSize::Sm, Style::Nova, true, true),
            (6.0, 6.0)
        );
        // No trigger: full size padding.
        assert_eq!(
            metrics::padding_sides(ToggleSize::Default, Style::Nova, false, false),
            (10.0, 10.0)
        );
        // Base: no icon-inline adjustment whatever the triggers.
        assert_eq!(
            metrics::padding_sides(ToggleSize::Default, Style::Vega, true, true),
            (8.0, 8.0)
        );
        assert_eq!(
            metrics::padding_sides(ToggleSize::Sm, Style::Vega, true, true),
            (6.0, 6.0)
        );
    }

    #[test]
    fn content_cursor_starts_at_left_padding_tog_vis_01() {
        // TOG-VIS-01: centring the content block in the full rect
        // painted the N16 icon-side padding as the average of the two
        // sides (9/8 instead of 8/6). The content box starts at pad_l;
        // min-width slack splits evenly inside it.
        let close = |v: f32, e: f32| assert!((v - e).abs() < 0.001, "{v} != {e}");
        // Asymmetric, exactly sized: the §3 rows (widths from the S5
        // readouts; total_w = width - pads). The centred formula would
        // give 9.0 / 8.0 here.
        close(content_cursor_x(96.6, 8.0, 10.0, 78.6), 8.0);
        close(content_cursor_x(87.6, 6.0, 10.0, 71.6), 6.0);
        // Symmetric, exactly sized: identical to centring.
        close(content_cursor_x(98.6, 10.0, 10.0, 78.6), 10.0);
        // Min-width slack splits evenly: centred overall.
        close(content_cursor_x(40.0, 10.0, 10.0, 16.0), 12.0);
        // Over-full rect (narrow container): no negative slack.
        close(content_cursor_x(30.0, 10.0, 10.0, 16.0), 10.0);
    }

    #[test]
    fn transition_specs_follow_ruling_6() {
        // Base: text + ring animate; fill/border snap.
        let base = anim_specs(Style::Vega);
        assert_eq!(base.text.duration_secs, 0.15);
        assert_eq!(base.text.easing, Easing::EaseInOut);
        assert_eq!(base.fill_border.duration_secs, 0.0);
        assert_eq!(base.ring.duration_secs, 0.15);
        // Nova (`transition-all`): everything animates.
        let nova = anim_specs(Style::Nova);
        assert_eq!(nova.text.duration_secs, 0.15);
        assert_eq!(nova.fill_border.duration_secs, 0.15);
        assert_eq!(nova.ring.duration_secs, 0.15);
        // Timing is INTERIM per OPEN-5 (framework defaults).
    }

    #[test]
    fn fill_rgb_target_tracks_the_opaque_endpoint() {
        let p = palette(Mode::Light);
        // Pressed: the pressed fill in each style.
        assert_eq!(
            fill_rgb_target(ToggleVariant::Default, &p, true, Style::Vega),
            p.accent
        );
        assert_eq!(
            fill_rgb_target(ToggleVariant::Default, &p, true, Style::Nova),
            p.muted
        );
        // Resting: the hover endpoint's RGB (invisible at alpha 0).
        assert_eq!(
            fill_rgb_target(ToggleVariant::Default, &p, false, Style::Vega),
            p.muted
        );
        assert_eq!(
            fill_rgb_target(ToggleVariant::Outline, &p, false, Style::Vega),
            p.accent
        );
        assert_eq!(
            fill_rgb_target(ToggleVariant::Outline, &p, false, Style::Nova),
            p.muted
        );
    }

    #[test]
    fn disabled_group_fade_matches_the_differential_model() {
        // Synthetic opaque case, exact arithmetic: backdrop 200, fill 50,
        // stroke 100, fg 80. Group fade = 0.5 * composite + 0.5 * 200.
        let backdrop = Color32::from_rgb(200, 200, 200);
        let fill = Color32::from_rgb(50, 50, 50);
        let stroke = Color32::from_rgb(100, 100, 100);
        let fg = Color32::from_rgb(80, 80, 80);
        let (f, s, g) = disabled_group_fade(fill, stroke, fg, backdrop);
        // Fill: halved alpha over the backdrop → 0.5*50 + 0.5*200 = 125.
        assert_eq!(blend(backdrop, f), Color32::from_rgb(125, 125, 125));
        // Stroke over fill: composite 100 → 0.5*100 + 0.5*200 = 150.
        assert_eq!(s, Color32::from_rgb(150, 150, 150));
        // Glyph over fill: composite 80 → 0.5*80 + 0.5*200 = 140.
        assert_eq!(g, Color32::from_rgb(140, 140, 140));
        // Transparent fill: the stroke precomposite fades over the
        // backdrop alone (composite = stroke over backdrop).
        let (f, s, _) = disabled_group_fade(Color32::TRANSPARENT, stroke, fg, backdrop);
        assert_eq!(f, Color32::TRANSPARENT);
        assert_eq!(s, Color32::from_rgb(150, 150, 150));
        // Translucent stroke (dark `input` token shape): the precomposite
        // equals the differential formula — composite the enabled stack,
        // then fade once. This pins the wiring, not just the arithmetic.
        let translucent = color_alpha(Color32::from_rgb(255, 255, 255), 0.15);
        let (f, s, g) = disabled_group_fade(fill, translucent, fg, backdrop);
        let fill_composite = blend(backdrop, fill);
        assert_eq!(blend(backdrop, f), mix(backdrop, fill_composite, 0.5));
        assert_eq!(s, mix(backdrop, blend(fill_composite, translucent), 0.5));
        assert_eq!(g, mix(backdrop, blend(fill_composite, fg), 0.5));
    }

    #[test]
    fn toggles_on_click() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut pressed = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("toggle_click_test"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(200.0, 200.0),
                )),
            );
            let r = Toggle::new(&mut pressed).text("Bold").show(ui);
            assert!(!r.clicked()); // no pointer input
        });
        out.textures_delta.clear();
        assert!(!pressed);
    }

    #[test]
    fn rendered_width_matches_plan() {
        // End-to-end pin of the width wiring: the allocated rect is the
        // content plan (one gap value) + side padding, floored at
        // min-width, at the fixed height — in whichever style the default
        // preset installs. The per-style VALUES are pinned by the metrics
        // tests; this pins that `show` consumes the same plan.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut pressed = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("toggle_width_test"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(400.0, 200.0),
                )),
            );
            let style = ShadcnTheme::of_ui(ui).style;
            let r = Toggle::new(&mut pressed)
                .text("Bold")
                .icon(Icon::Sparkles)
                .show(ui);
            let galley = ui.painter().layout_no_wrap(
                "Bold".to_owned(),
                fonts::medium(metrics::text_size(ToggleSize::Default, style)),
                Color32::PLACEHOLDER,
            );
            let plan = content_plan(
                true,
                metrics::icon_size(ToggleSize::Default, style),
                galley.size().x,
                metrics::gap(ToggleSize::Default, style),
            );
            let (pad_l, pad_r) = metrics::padding_sides(ToggleSize::Default, style, false, false);
            let expected =
                (pad_l + plan.total_w + pad_r).max(metrics::min_width(ToggleSize::Default, style));
            assert!((r.rect.width() - expected).abs() < 0.01);
            assert_eq!(r.rect.height(), metrics::height(ToggleSize::Default, style));
        });
        out.textures_delta.clear();
    }

    #[test]
    fn space_fires_on_keyup_not_keydown() {
        // TOG-INT-01: native `<button>` behaviour (spec §7, OPEN-2 closed
        // by S6 Work B) — Space toggles on keyup; Enter toggles on
        // keydown. Mirrors the switch pin `space_fires_on_keyup_not_keydown`
        // (SWI-INT-01), adapted to the toggle's `&mut bool` binding.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("toggle_space_keyup_test"),
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
        let mut pressed = false;

        // Frame 1: render, record id; then give the toggle keyboard focus.
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            id = Toggle::new(&mut pressed).text("Bold").show(ui).id;
        });
        out.textures_delta.clear();
        ctx.memory_mut(|m| m.request_focus(id));
        assert!(ctx.memory(|m| m.has_focus(id)));

        let mut frame = |ctx: &egui::Context, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut make_ui(ctx);
                let _ = Toggle::new(&mut pressed).text("Bold").show(ui);
            });
            out.textures_delta.clear();
            pressed
        };

        // Space keydown: no toggle. Autorepeat keydown: no toggle. Hold
        // frame with no events: no toggle.
        assert!(!frame(&ctx, key_event(egui::Key::Space, true)));
        let repeat_down = egui::RawInput {
            events: vec![egui::Event::Key {
                key: egui::Key::Space,
                physical_key: None,
                pressed: true,
                repeat: true,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        assert!(!frame(&ctx, repeat_down));
        assert!(!frame(&ctx, egui::RawInput::default()));
        // Space keyup: exactly one toggle. The frame after: no toggle.
        assert!(frame(&ctx, key_event(egui::Key::Space, false)));
        assert!(frame(&ctx, egui::RawInput::default()));
        // A second press/release cycle toggles exactly once, on keyup.
        assert!(frame(&ctx, key_event(egui::Key::Space, true)));
        assert!(!frame(&ctx, key_event(egui::Key::Space, false)));
        // Enter still toggles on keydown, and does not toggle on keyup.
        assert!(frame(&ctx, key_event(egui::Key::Enter, true)));
        assert!(frame(&ctx, key_event(egui::Key::Enter, false)));
        // Focus is retained after keyboard activation.
        assert!(ctx.memory(|m| m.has_focus(id)));

        // A disabled toggle never toggles from Space, on either edge.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut pressed = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Toggle::new(&mut pressed)
                .text("Bold")
                .disabled(true)
                .show(ui);
        });
        out.textures_delta.clear();
        let mut disabled_frame = |input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut make_ui(ctx);
                let _ = Toggle::new(&mut pressed)
                    .text("Bold")
                    .disabled(true)
                    .show(ui);
            });
            out.textures_delta.clear();
            pressed
        };
        assert!(!disabled_frame(key_event(egui::Key::Space, true)));
        assert!(!disabled_frame(key_event(egui::Key::Space, false)));
    }

    #[test]
    fn click_focuses_and_toggles() {
        // TOG-INT-02: native `<button>` behaviour (spec §7, settled by S6
        // Work B from public Radix Toggle source) — a direct click focuses
        // AND toggles. Mirrors the button pin
        // `click_moves_keyboard_focus_to_the_button` (BTN-INT-01) and the
        // checkbox pin (CHK-INT-01), adapted to the toggle's `&mut bool`
        // binding.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("toggle_click_focus_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
            )
        };
        let pointer_event = |pos: Pos2, pressed: bool| egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let move_event = |pos: Pos2| egui::RawInput {
            events: vec![egui::Event::PointerMoved(pos)],
            ..Default::default()
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
        let mut pressed = false;

        // Frame 1: render, record rect + id; nothing focused yet.
        let mut rect = Rect::NOTHING;
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let r = Toggle::new(&mut pressed).text("Bold").show(ui);
            rect = r.rect;
            id = r.id;
        });
        out.textures_delta.clear();
        assert!(!ctx.memory(|m| m.has_focus(id)));

        // Press alone: no toggle, no focus.
        let center = rect.center();
        let mut out = ctx.run_ui(pointer_event(center, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let r = Toggle::new(&mut pressed).text("Bold").show(ui);
            assert!(!r.clicked());
        });
        out.textures_delta.clear();
        assert!(!pressed);
        assert!(!ctx.memory(|m| m.has_focus(id)));

        // Release on the toggle: click + toggle + focus on the click frame.
        let mut clicked = false;
        let mut out = ctx.run_ui(pointer_event(center, false), |ctx| {
            let ui = &mut make_ui(ctx);
            clicked = Toggle::new(&mut pressed).text("Bold").show(ui).clicked();
        });
        out.textures_delta.clear();
        assert!(clicked, "press + release must register a click");
        assert!(pressed, "click must toggle");
        assert!(
            ctx.memory(|m| m.has_focus(id)),
            "click must move keyboard focus to the toggle"
        );

        // Space after a click: keydown no toggle (TOG-INT-01), keyup
        // exactly one toggle, focus retained.
        let mut out = ctx.run_ui(key_event(egui::Key::Space, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Toggle::new(&mut pressed).text("Bold").show(ui);
        });
        out.textures_delta.clear();
        assert!(pressed);
        let mut out = ctx.run_ui(key_event(egui::Key::Space, false), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Toggle::new(&mut pressed).text("Bold").show(ui);
        });
        out.textures_delta.clear();
        assert!(!pressed);
        assert!(ctx.memory(|m| m.has_focus(id)));

        // A second click toggles back, focus retained.
        let mut out = ctx.run_ui(pointer_event(center, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Toggle::new(&mut pressed).text("Bold").show(ui);
        });
        out.textures_delta.clear();
        let mut out = ctx.run_ui(pointer_event(center, false), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Toggle::new(&mut pressed).text("Bold").show(ui);
        });
        out.textures_delta.clear();
        assert!(pressed);
        assert!(ctx.memory(|m| m.has_focus(id)));

        // Press-drag-off-release: no toggle, no focus. Fresh context so
        // the first click's focus does not leak in.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut pressed = false;
        let mut rect = Rect::NOTHING;
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let r = Toggle::new(&mut pressed).text("Bold").show(ui);
            rect = r.rect;
            id = r.id;
        });
        out.textures_delta.clear();
        let center = rect.center();
        let far = center + Vec2::new(150.0, 0.0);
        let mut out = ctx.run_ui(pointer_event(center, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Toggle::new(&mut pressed).text("Bold").show(ui);
        });
        out.textures_delta.clear();
        let mut out = ctx.run_ui(move_event(far), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Toggle::new(&mut pressed).text("Bold").show(ui);
        });
        out.textures_delta.clear();
        let mut clicked = false;
        let mut out = ctx.run_ui(pointer_event(far, false), |ctx| {
            let ui = &mut make_ui(ctx);
            clicked = Toggle::new(&mut pressed).text("Bold").show(ui).clicked();
        });
        out.textures_delta.clear();
        assert!(!clicked, "drag-off release must not register a click");
        assert!(!pressed);
        assert!(!ctx.memory(|m| m.has_focus(id)));

        // A disabled toggle takes neither toggle nor focus when clicked.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut pressed = false;
        let mut rect = Rect::NOTHING;
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let r = Toggle::new(&mut pressed)
                .text("Bold")
                .disabled(true)
                .show(ui);
            rect = r.rect;
            id = r.id;
        });
        out.textures_delta.clear();
        let center = rect.center();
        let mut out = ctx.run_ui(pointer_event(center, true), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Toggle::new(&mut pressed)
                .text("Bold")
                .disabled(true)
                .show(ui);
        });
        out.textures_delta.clear();
        let mut clicked = false;
        let mut out = ctx.run_ui(pointer_event(center, false), |ctx| {
            let ui = &mut make_ui(ctx);
            clicked = Toggle::new(&mut pressed)
                .text("Bold")
                .disabled(true)
                .show(ui)
                .clicked();
        });
        out.textures_delta.clear();
        assert!(!clicked);
        assert!(!pressed);
        assert!(!ctx.memory(|m| m.has_focus(id)));
    }

    #[test]
    fn builders_keep_fields() {
        let mut pressed = false;
        let t = Toggle::new(&mut pressed)
            .text("Bold")
            .icon(Icon::Sparkles)
            .variant(ToggleVariant::Outline)
            .size(ToggleSize::Sm)
            .padding_x(12.0)
            .disabled(true)
            .invalid(true)
            .id_salt("t")
            .icon_inline_start(true);
        assert_eq!(t.text.as_deref(), Some("Bold"));
        assert!(t.icon.is_some());
        assert_eq!(t.variant, ToggleVariant::Outline);
        assert_eq!(t.size, ToggleSize::Sm);
        assert_eq!(t.padding_x_override, Some(12.0));
        assert!(t.disabled);
        assert!(t.invalid);
        assert_eq!(t.id_salt.as_deref(), Some("t"));
        assert!(t.icon_inline_start);
    }
}
