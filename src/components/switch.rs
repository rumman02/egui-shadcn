//! shadcn `switch.tsx` — a two-state on/off control.
//!
//! Built to the frozen Batch B spec (`~/workspace/batch-b/specs/switch-spec.md`,
//! coordinator-signed 2026-11-16). Source: `apps/v4/registry/new-york-v4/ui/switch.tsx`
//! (34 lines) plus the nova rules in `apps/v4/registry/styles/style-nova.css:1250-1264`.
//! No Radix source is in the snapshot, so no Radix default is guessed from the
//! snapshot itself: the keyboard rules were settled by S6 Work B from the
//! public Radix Switch source (OPEN-2/OPEN-5/OPEN-6 CLOSED, OPEN-3 partially
//! closed — the browser-heuristic half stays OPEN), and timing stays OPEN-1
//! (Tailwind framework defaults, labelled INTERIM).
//!
//! Base track class string (`switch.tsx:19`):
//!
//! ```text
//! peer group/switch inline-flex shrink-0 items-center rounded-full border
//! border-transparent shadow-xs transition-all outline-none
//! focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50
//! disabled:cursor-not-allowed disabled:opacity-50
//! data-[size=default]:h-[1.15rem] data-[size=default]:w-8
//! data-[size=sm]:h-3.5 data-[size=sm]:w-6 data-[state=checked]:bg-primary
//! data-[state=unchecked]:bg-input dark:data-[state=unchecked]:bg-input/80
//! ```
//!
//! Base thumb class string (`switch.tsx:27`):
//!
//! ```text
//! pointer-events-none block rounded-full bg-background ring-0 transition-transform
//! group-data-[size=default]/switch:size-4 group-data-[size=sm]/switch:size-3
//! data-[state=checked]:translate-x-[calc(100%-2px)]
//! data-[state=unchecked]:translate-x-0
//! dark:data-[state=checked]:bg-primary-foreground
//! dark:data-[state=unchecked]:bg-foreground
//! ```
//!
//! Nova deltas only (`.cn-switch`, `style-nova.css:1251`): the full invalid
//! recipe (`aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40
//! aria-invalid:border-destructive dark:aria-invalid:border-destructive/50
//! aria-invalid:ring-3`) plus the field-label suppression rule
//! (`group-has-[:focus-visible]/field-label:ring-0
//! group-has-[:focus-visible]/field-label:border-transparent`). Nova geometry is
//! identical to base in every dimension (the rule re-states `h-[18.4px] w-[32px]`
//! and `h-[14px] w-[24px]`). Nova is silent on shadow, transitions, disabled
//! rules, `outline-none`, and the flex/group/peer classes, so all of those survive
//! via LAYER (Phase 1 §5(b) ruling: a property the nova rule does not mention
//! keeps its base value). The `-aria` spellings (`.cn-switch-aria`, `:1255`;
//! `.cn-switch-thumb-aria`, `:1263`) carry the same values (spec OPEN-7).
//!
//! Geometry (spec §3, border-box, 1px border inside the outer box): default
//! track 32.0 × 18.4, thumb 16×16, travel 14.0px; sm track 24.0 × 14.0, thumb
//! 12×12, travel 10.0px. The thumb is flush with the inner border edge at both
//! extremes (the `−2px` in `calc(100%-2px)` is exactly the two 1px borders, not
//! a gap). The focus/invalid ring is a 3px band starting at the outer edge of
//! the border box. `shadow-xs` paints on the track only, in all states.
//!
//! Animation (spec §6): the track carries `transition-all` (fill, border colour,
//! and ring appearance all animate, including theme-switch recolour); the thumb
//! carries `transition-transform` ONLY (the slide animates, the fill snaps).
//! Duration and easing are spec OPEN-1: this build uses the Tailwind v4
//! framework defaults (150ms, `cubic-bezier(0.4, 0, 0.2, 1)`) via
//! [`TransitionSpec::default`], labelled INTERIM — they are framework defaults,
//! not snapshot-verified truth.
//!
//! Invalid (spec §2.6, N5 settled): the invalid recipe is nova-only. The base
//! recipe has ZERO invalid classes, so under the base recipe (any non-nova
//! style) `.invalid(true)` is accepted but paints no extra chrome (spec
//! OPEN-10, S1 decision RATIFIED at the S8 gate: snapshot-faithful, no
//! deviation needed). Under nova, invalid paints the destructive border
//! (full strength light, `/50` dark) and a 3px rest ring (`destructive/20`
//! light, `/40` dark) that is NOT focus-gated. Invalid+focus precedence is
//! spec OPEN-4; this build keeps invalid-wins provisionally (and
//! suppression-wins-over-invalid, also provisional) and records it as such.
//!
//! Disabled (spec §4/§5): `disabled:opacity-50` composites the whole element
//! at 50% via one opacity multiplier applied to every painted slot — fill,
//! thumb, border, ring, and shadow — per the frozen spec's own gloss
//! ("multiplies every painted slot including shadow and ring"). The S5
//! verdict stands on that text (ruling 7 item 3). The spec-language
//! divergence from the checkbox/slider group-fade reading of the same CSS
//! property is a wave-pack question for the Expert Reviewer, not resolved
//! here. A disabled switch takes no Tab stop, never activates, and shows the
//! `not-allowed` cursor.
//!
//! Keyboard and click (spec §7): `switch.tsx` attaches no handler; S6 Work B
//! settled the Radix semantics from the public Radix Switch source (OPEN-2
//! CLOSED, `@radix-ui/react-switch` 1.4.1, mechanism stable 2021→2026):
//! Space toggles on KEYUP, Enter on keydown, click toggles on the click
//! event, exactly one Tab stop, disabled blocks activation and takes no
//! stop, arrows/Escape/Home/End have no behaviour. egui fires Space on
//! keydown, so this build intervenes (see `space_keyup_fire`, the button
//! BTN-INT-04 shape) to toggle on keyup (SWI-INT-01, fixed in one cycle
//! and independently re-verified at S10). The ring paints on any egui focus
//! (proposed deviation D1, button D3 class). A direct click toggles but does
//! not focus the switch and shows no ring in egui (observation OBS-SWI-S6-1,
//! gate G6 — not a defect: OPEN-3's browser-heuristic half stays OPEN, and
//! the harmonization question goes to the Expert Reviewer). OPEN-5 CLOSED
//! (controlled `checked` + `onCheckedChange`, uncontrolled `defaultChecked`
//! defaulting to `false`); this port exposes the controlled form only.
//! OPEN-6 CLOSED (`asChild` supported upstream; limitation D3 stands).
//!
//! Deviations (all PROPOSED by the spec, none approved — the Expert Reviewer
//! rules at the wave pack): D1 `:focus-visible` modality (ring on any focus);
//! D2 ARIA limitation (note only); D3 `asChild` limitation (note only); D4
//! field-label focus suppression (see [`Switch::suppress_focus_indication`];
//! the mechanism was verified in S5/S6, approval still pending);
//! D5 `peer`/`group` implementation note (no deviation); D6 native form
//! submission limitation (note only).
//!
//! Verification (Batch B): S5 judged 40/40 checks PASS in all 4 configs; S6
//! filed one defect (SWI-INT-01, Space phase), fixed and independently
//! re-verified at S10 with 0 defects; S7 passed M1–M7 with two accepted
//! gaps (the M8 render floor and the theme cross-fade leg could not be
//! resolved by the environment — gate G2, no breach claimed either way).
//! The FLD-VIS-01 field-description cross-check on the demo's §7 cards is
//! PENDING until the field line lands its fix (gate G7).
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
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use crate::utils::animation::{
    transition_alpha, transition_color_to, transition_progress, TransitionSpec,
};
use crate::utils::focus::paint_ring;
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Color32, CornerRadius, CursorIcon, Painter, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2,
    Widget,
};

/// Track and thumb geometry per size (`data-size`, spec §3). Nova geometry is
/// identical to base in every dimension, so one geometry serves both recipes.
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

    /// Thumb travel when checked (`translate-x-[calc(100%-2px)]` = thumb
    /// width minus the two 1px borders, spec §3.2).
    pub fn checked_offset(self) -> f32 {
        self.thumb_size() - 2.0
    }

    /// Center of the thumb circle for a track rect and slide offset. The
    /// thumb sits flush against the inner border edge at both extremes:
    /// unchecked left edge 1.0, checked left edge `1.0 + travel` (spec §3.2
    /// edge table); vertically centered (1.2px clearance top/bottom at
    /// default size, 1.0px at sm).
    pub fn thumb_center(self, track: Rect, offset: f32) -> Pos2 {
        Pos2::new(
            track.min.x + 1.0 + offset + self.thumb_size() / 2.0,
            track.center().y,
        )
    }
}

/// Resolved colors for one switch state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct SwitchColors {
    track: Color32,
    thumb: Color32,
    /// Ring target color (alpha already applied): the invalid color when
    /// invalid under nova, else `ring/50`. Visibility is decided by
    /// [`ring_shown`]; this field is only the color the ring fades to.
    ring: Color32,
}

/// Resolves track, thumb, and ring-target colors (spec §4/§5).
///
/// Track: `data-[state=checked]:bg-primary`,
/// `data-[state=unchecked]:bg-input dark:data-[state=unchecked]:bg-input/80`.
/// Thumb: `bg-background` light, `dark:data-[state=unchecked]:bg-foreground` /
/// `dark:data-[state=checked]:bg-primary-foreground` dark. Ring target: the
/// nova invalid color (`destructive/20` light, `/40` dark) when invalid under
/// nova, else `ring/50`. Under the base recipe invalid has no ring rule
/// (OPEN-10), so the target stays `ring/50` there.
pub(crate) fn resolve_switch_colors(
    p: &Palette,
    dark: bool,
    style: Style,
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
    let ring = if invalid && style == Style::Nova {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    SwitchColors { track, thumb, ring }
}

/// The border color the track fades TOWARD when the border is shown (spec
/// §4). Invalid wins over focus (provisional, OPEN-4): under nova an invalid
/// switch keeps the destructive border even when focused. Under the base
/// recipe invalid has no border rule (OPEN-10), so the target is always
/// `ring`. The dark invalid border is `destructive/50` (N5 settled, spec
/// §2.6); light is full-strength `destructive`.
pub(crate) fn border_target_color(p: &Palette, dark: bool, style: Style, invalid: bool) -> Color32 {
    if invalid && style == Style::Nova {
        if dark {
            color_alpha(p.destructive, 0.5)
        } else {
            p.destructive
        }
    } else {
        p.ring
    }
}

/// Whether the 1px border is shown (fading in under `transition-all`):
/// focused, or invalid under nova (rest chrome, spec §2.6). The field-label
/// suppression (D4) forces the border transparent, but only under nova — the
/// suppression rule lives in `.cn-switch`, so the flag is a no-op under the
/// base recipe.
pub(crate) fn border_shown(focused: bool, invalid: bool, style: Style, suppressed: bool) -> bool {
    let nova = style == Style::Nova;
    !(suppressed && nova) && (focused || (invalid && nova))
}

/// Whether the 3px ring is shown: the same rule as [`border_shown`] — the
/// ring and the border recolor travel together (`focus-visible:border-ring` +
/// `focus-visible:ring-[3px]`, and the nova invalid rest ring, spec §2.6/§4).
/// Suppression+invalid precedence is folded into OPEN-4: provisionally the
/// suppression wins over the invalid ring too (the literal rule reading:
/// `ring-0` + `border-transparent` when the condition holds).
pub(crate) fn ring_shown(focused: bool, invalid: bool, style: Style, suppressed: bool) -> bool {
    border_shown(focused, invalid, style, suppressed)
}

/// Effective focus for chrome purposes: a disabled switch is not focusable,
/// so it never shows the focus border/ring (spec §4).
pub(crate) fn focus_visible(has_focus: bool, disabled: bool) -> bool {
    has_focus && !disabled
}

/// Paints the track's `shadow-xs` through the LAYER resolution API (spec
/// §3.3). The base recipe declares `shadow-xs` and no style's `.cn-switch`
/// rule overrides it, so it paints under every style (Phase 1 §5(b) LAYER
/// ruling). When disabled, the shadow fades with the element's target
/// opacity: the layer alphas scale on the stack (no allocation) and the cache
/// keys on the faded spec, so the animated opacity fade never churns shadow
/// textures (same shape as the button's `paint_outline_shadow`).
fn paint_switch_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::Switch).layers();
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
            ShadowSlot::Switch,
            dark,
        );
    }
}

/// Space-keyup activation (spec §7, SWI-INT-01; the button BTN-INT-04 shape).
///
/// Native `<button>` toggles on Space KEYUP; egui's `interact` reports a
/// fake click whenever the focused widget sees `key_pressed(Space)`, so
/// without intervention Space toggles on keydown (and re-toggles on every
/// autorepeat). This helper, called before `interact` while the switch
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
/// switch (focus arrived mid-hold) does not toggle; a stale armed flag
/// is cleared on the first frame the switch is shown with Space up.
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

/// A shadcn switch. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Switch<'a> {
    checked: &'a mut bool,
    size: SwitchSize,
    disabled: bool,
    invalid: bool,
    suppress_focus_indication: bool,
    id_salt: Option<String>,
}

impl<'a> Switch<'a> {
    /// Creates a switch bound to the given boolean (controlled `&mut bool`;
    /// spec OPEN-5 CLOSED by S6 Work B — the uncontrolled `defaultChecked`
    /// form has no port equivalent).
    pub fn new(checked: &'a mut bool) -> Self {
        Self {
            checked,
            size: SwitchSize::Default,
            disabled: false,
            invalid: false,
            suppress_focus_indication: false,
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

    /// Marks the switch aria-invalid. Paints the nova invalid recipe (border
    /// and rest ring); under the base recipe it paints nothing extra (spec
    /// OPEN-10, ratified at the S8 gate).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Suppresses the switch's own focus indication (ring + border recolor)
    /// when the switch sits inside a FieldLabel choice card that shows the
    /// focus indication itself (proposed deviation D4, spec §4/§10). This is
    /// the switch-side mechanism for the nova rule
    /// `group-has-[:focus-visible]/field-label:ring-0
    /// group-has-[:focus-visible]/field-label:border-transparent`
    /// (`style-nova.css:1251`): the composing card sets this flag each frame
    /// from its own focus-inside state. The field line owns the composition
    /// contract. Nova-only; a no-op under the base recipe. Suppression+invalid
    /// precedence is provisional (OPEN-4): the suppression wins. The
    /// mechanism was verified against the demo's card compositions in S5/S6;
    /// the deviation itself awaits Expert Reviewer approval.
    pub fn suppress_focus_indication(mut self, suppress: bool) -> Self {
        self.suppress_focus_indication = suppress;
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
        let style = theme.style;
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
        // Space toggles on keyup, not keydown (spec §7, OPEN-2 closed by S6
        // Work B; SWI-INT-01): hide this frame's Space keydown from
        // `interact` and re-create it on the keyup frame. Only while
        // focused and enabled — an unfocused switch must not consume
        // another widget's Space, and a disabled switch never activates.
        if !self.disabled && ui.ctx().memory(|m| m.has_focus(id)) {
            space_keyup_fire(ui.ctx(), id);
        }
        let response = ui.interact(rect, id, sense);

        // Activation (spec §7, OPEN-2 closed by S6 Work B): Space toggles
        // on keyup (see `space_keyup_fire`), Enter on keydown, pointer
        // click on the click event — the native `<button>` asymmetry.
        if response.clicked() && !self.disabled {
            *self.checked = !*self.checked;
        }

        if ui.is_rect_visible(rect) {
            let checked = *self.checked;
            let focused = focus_visible(response.has_focus(), self.disabled);
            let c = resolve_switch_colors(&p, dark, style, checked, self.invalid);
            // INTERIM per spec OPEN-1: Tailwind v4 framework defaults
            // (150ms + cubic-bezier(0.4, 0, 0.2, 1)), not snapshot truth.
            let spec = TransitionSpec::default();
            let ctx = ui.ctx();

            // `disabled:opacity-50` composites the whole element (spec §4);
            // the opacity fades with the rest under `transition-all`.
            let opacity =
                transition_alpha(ctx, id.with("disabled"), 1.0, 0.5, self.disabled, spec).alpha;

            let radius = size.track_height() / 2.0;
            let corner = CornerRadius::same(radius.round() as u8);
            let corner = fuse_radius(corner, orientation, position);

            let painter = ui.painter();
            // `shadow-xs` in all states (spec §3.3), composited at 50% when
            // disabled.
            paint_switch_shadow(painter, rect, corner, style, dark, self.disabled);

            // Track fill cross-fades under `transition-all` (spec §6):
            // toggle, theme switch, and style switch all animate. The
            // endpoints never cross the transparent boundary (opaque, or
            // translucent-but-defined in both themes), so
            // `transition_color_to`'s caveat holds.
            let track = color_alpha(
                transition_color_to(ctx, id.with("track_fill"), c.track, spec).color,
                opacity,
            );
            painter.rect_filled(rect, corner, track);

            // Border: `border border-transparent` at rest; recolors to `ring`
            // on focus, to `destructive` when invalid under nova (provisional
            // invalid-wins, OPEN-4). The target is wrapped in the
            // endpoint-change animator so a theme switch animates the border
            // in either state; the alpha ramp below owns the focus/invalid
            // fade (button precedent, BTN-FLK-02: never leg from transparent
            // on every focus change).
            let border_target = transition_color_to(
                ctx,
                id.with("border_target"),
                border_target_color(&p, dark, style, self.invalid),
                spec,
            )
            .color;
            let t_border = transition_progress(
                ctx,
                id.with("border"),
                border_shown(focused, self.invalid, style, self.suppress_focus_indication),
                spec,
            )
            .t;
            let border_alpha = t_border * opacity;
            if border_alpha > 0.0 {
                painter.rect_stroke(
                    rect,
                    corner,
                    Stroke::new(1.0, color_alpha(border_target, border_alpha)),
                    egui::epaint::StrokeKind::Inside,
                );
            }

            // Thumb: `transition-transform` ONLY (spec §6) — the slide eases,
            // the fill snaps (no color transition on the thumb). First paint
            // is settled (no slide-in on mount); mid-flight retargets
            // continue from the current position.
            let t_slide = transition_progress(ctx, id.with("thumb"), checked, spec).t;
            let offset = t_slide * size.checked_offset();
            painter.circle_filled(
                size.thumb_center(rect, offset),
                size.thumb_size() / 2.0,
                color_alpha(c.thumb, opacity),
            );

            // Ring: a 3px band starting at the outer edge of the border box
            // (spec §3.3), fading in/out under `transition-all`. The target
            // is wrapped in the endpoint-change animator so the focus fade
            // stays a pure alpha ramp instead of chasing through OKLab from
            // transparent (button precedent).
            let ring_target = transition_color_to(ctx, id.with("ring_target"), c.ring, spec).color;
            let ring_alpha = transition_alpha(
                ctx,
                id.with("ring"),
                0.0,
                1.0,
                ring_shown(focused, self.invalid, style, self.suppress_focus_indication),
                spec,
            )
            .alpha
                * opacity;
            if ring_alpha > 0.0 {
                paint_ring(
                    painter,
                    rect,
                    color_alpha(ring_target, ring_alpha),
                    3.0,
                    radius,
                );
            }
        }

        // `disabled:cursor-not-allowed` (spec §4).
        if self.disabled {
            response.on_hover_cursor(CursorIcon::NotAllowed)
        } else {
            response
        }
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

    /// A non-nova style stands in for the base recipe in tests: the spec
    /// covers base + nova only, and non-nova styles render the base recipe.
    const BASE: Style = Style::Vega;

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
    fn thumb_edges_flush_with_inner_border() {
        // Spec §3.2 edge table: the thumb is flush with the inner border
        // edge at both extremes, in both sizes.
        for size in [SwitchSize::Default, SwitchSize::Sm] {
            let track = Rect::from_min_size(
                Pos2::ZERO,
                Vec2::new(size.track_width(), size.track_height()),
            );
            let thumb = size.thumb_size();
            let unchecked = size.thumb_center(track, 0.0);
            assert!((unchecked.x - thumb / 2.0 - 1.0).abs() < 1e-4);
            let checked = size.thumb_center(track, size.checked_offset());
            assert!((checked.x - thumb / 2.0 - 1.0 - size.checked_offset()).abs() < 1e-4);
            assert!((checked.x + thumb / 2.0 - (size.track_width() - 1.0)).abs() < 1e-4);
            // Vertical centering: 1.2px clearance at default, 1.0px at sm.
            let clearance = if size == SwitchSize::Default {
                1.2
            } else {
                1.0
            };
            assert!((unchecked.y - thumb / 2.0 - clearance).abs() < 1e-4);
        }
        // Exact edge positions from the spec table.
        let default = Rect::from_min_size(Pos2::ZERO, Vec2::new(32.0, 18.4));
        let c = SwitchSize::Default.thumb_center(default, 14.0);
        assert!((c.x - 16.0 - 7.0).abs() < 1e-4); // left edge 15.0
        let sm = Rect::from_min_size(Pos2::ZERO, Vec2::new(24.0, 14.0));
        let c = SwitchSize::Sm.thumb_center(sm, 10.0);
        assert!((c.x - 6.0 - 11.0).abs() < 1e-4); // left edge 11.0
    }

    #[test]
    fn track_and_thumb_colors_all_configs() {
        // All four configs (light/dark x base/nova): state colors are
        // identical across styles (nova re-states the same fills); only the
        // invalid recipe differs (see the invalid tests).
        for (mode, dark) in [(Mode::Light, false), (Mode::Dark, true)] {
            let p = palette(mode);
            for style in [BASE, Style::Nova] {
                let off = resolve_switch_colors(&p, dark, style, false, false);
                let on = resolve_switch_colors(&p, dark, style, true, false);
                assert_eq!(on.track, p.primary);
                assert_eq!(
                    on.thumb,
                    if dark {
                        p.primary_foreground
                    } else {
                        p.background
                    }
                );
                if dark {
                    assert_eq!(off.track, color_alpha(p.input, 0.8));
                    assert_eq!(off.thumb, p.foreground);
                } else {
                    assert_eq!(off.track, p.input);
                    assert_eq!(off.thumb, p.background);
                }
            }
        }
    }

    #[test]
    fn nova_invalid_recipe() {
        // Spec §2.6 (N5 settled): border destructive light, destructive/50
        // dark; rest ring destructive/20 light, /40 dark — shown unfocused.
        let p = palette(Mode::Light);
        assert_eq!(
            border_target_color(&p, false, Style::Nova, true),
            p.destructive
        );
        let c = resolve_switch_colors(&p, false, Style::Nova, false, true);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
        assert!(ring_shown(false, true, Style::Nova, false));
        assert!(border_shown(false, true, Style::Nova, false));

        let dark = palette(Mode::Dark);
        assert_eq!(
            border_target_color(&dark, true, Style::Nova, true),
            color_alpha(dark.destructive, 0.5)
        );
        let c = resolve_switch_colors(&dark, true, Style::Nova, false, true);
        assert_eq!(c.ring, color_alpha(dark.destructive, 0.4));
        assert!(ring_shown(false, true, Style::Nova, false));
        assert!(border_shown(false, true, Style::Nova, false));
    }

    #[test]
    fn base_invalid_paints_nothing() {
        // Spec OPEN-10: the base recipe has ZERO invalid classes, so an
        // invalid switch under the base recipe shows no invalid chrome —
        // only the focus chrome, when focused.
        for (mode, dark) in [(Mode::Light, false), (Mode::Dark, true)] {
            let p = palette(mode);
            assert_eq!(border_target_color(&p, dark, BASE, true), p.ring);
            let c = resolve_switch_colors(&p, dark, BASE, false, true);
            assert_eq!(c.ring, color_alpha(p.ring, 0.5));
            assert!(!ring_shown(false, true, BASE, false));
            assert!(!border_shown(false, true, BASE, false));
            // Focused + invalid under base: focus chrome only.
            assert_eq!(border_target_color(&p, dark, BASE, true), p.ring);
            assert!(ring_shown(true, true, BASE, false));
        }
    }

    #[test]
    fn invalid_wins_over_focus_provisional() {
        // Provisional per spec OPEN-4: under nova, invalid targets win when
        // both focus and invalid apply.
        let p = palette(Mode::Light);
        assert_eq!(
            border_target_color(&p, false, Style::Nova, true),
            p.destructive
        );
        let c = resolve_switch_colors(&p, false, Style::Nova, false, true);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
    }

    #[test]
    fn focus_ring_requires_focus() {
        // A valid switch shows its ring only when focused, in both styles.
        for style in [BASE, Style::Nova] {
            assert!(ring_shown(true, false, style, false));
            assert!(!ring_shown(false, false, style, false));
            assert!(border_shown(true, false, style, false));
            assert!(!border_shown(false, false, style, false));
        }
    }

    #[test]
    fn disabled_never_shows_focus_chrome() {
        // A disabled switch is not focusable (spec §4): even a stale focus
        // flag yields no focus chrome.
        assert!(!focus_visible(true, true));
        assert!(!focus_visible(false, true));
        assert!(focus_visible(true, false));
        assert!(!focus_visible(false, false));
    }

    #[test]
    fn field_label_suppression_is_nova_only() {
        // D4: suppression hides the ring/border under nova, and is a no-op
        // under the base recipe. Suppression+invalid: provisionally the
        // suppression wins (OPEN-4).
        assert!(!ring_shown(true, false, Style::Nova, true));
        assert!(!border_shown(true, false, Style::Nova, true));
        assert!(!ring_shown(false, true, Style::Nova, true));
        assert!(ring_shown(true, false, BASE, true));
        assert!(border_shown(true, false, BASE, true));
    }

    #[test]
    fn space_fires_on_keyup_not_keydown() {
        // SWI-INT-01: native `<button>` behaviour (spec §7, OPEN-2 closed
        // by S6 Work B) — Space toggles on keyup; Enter toggles on keydown.
        // Mirrors the button pin `space_fires_on_keyup_not_keydown`
        // (BTN-INT-04), adapted to the switch's `&mut bool` binding.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let make_ui = |ctx: &egui::Context| {
            egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("switch_space_keyup_test"),
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
        let mut checked = false;

        // Frame 1: render, record id; then give the switch keyboard focus.
        let mut id = egui::Id::NULL;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            id = Switch::new(&mut checked).show(ui).id;
        });
        out.textures_delta.clear();
        ctx.memory_mut(|m| m.request_focus(id));
        assert!(ctx.memory(|m| m.has_focus(id)));

        let mut frame = |ctx: &egui::Context, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut make_ui(ctx);
                let _ = Switch::new(&mut checked).show(ui);
            });
            out.textures_delta.clear();
            checked
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

        // A disabled switch never toggles from Space, on either edge.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut make_ui(ctx);
            let _ = Switch::new(&mut checked).disabled(true).show(ui);
        });
        out.textures_delta.clear();
        let mut disabled_frame = |input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut make_ui(ctx);
                let _ = Switch::new(&mut checked).disabled(true).show(ui);
            });
            out.textures_delta.clear();
            checked
        };
        assert!(!disabled_frame(key_event(egui::Key::Space, true)));
        assert!(!disabled_frame(key_event(egui::Key::Space, false)));
    }

    #[test]
    fn builders_keep_fields() {
        let mut checked = false;
        let sw = Switch::new(&mut checked)
            .size(SwitchSize::Sm)
            .disabled(true)
            .invalid(true)
            .suppress_focus_indication(true)
            .id_salt("sw");
        assert_eq!(sw.size, SwitchSize::Sm);
        assert!(sw.disabled);
        assert!(sw.invalid);
        assert!(sw.suppress_focus_indication);
        assert_eq!(sw.id_salt.as_deref(), Some("sw"));
    }
}
