//! shadcn `checkbox.tsx` — a binary (or indeterminate) check control.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/checkbox.tsx` (31 lines).
//! Base recipe — Root class string, verbatim (`checkbox.tsx:16`):
//!
//! ```text
//! peer size-4 shrink-0 rounded-[4px] border border-input shadow-xs transition-shadow
//! outline-none focus-visible:border-ring focus-visible:ring-[3px]
//! focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50
//! aria-invalid:border-destructive aria-invalid:ring-destructive/20
//! data-[state=checked]:border-primary data-[state=checked]:bg-primary
//! data-[state=checked]:text-primary-foreground dark:bg-input/30
//! dark:aria-invalid:ring-destructive/40 dark:data-[state=checked]:bg-primary
//! ```
//!
//! Indicator (`checkbox.tsx:23`): `grid place-content-center text-current
//! transition-none`. Icon (`checkbox.tsx:25`): the Lucide `Check` icon in a
//! `size-3.5` (14px) box. The original supplies exactly one icon
//! (`CheckIcon`); no minus/dash glyph exists in the original source.
//!
//! Nova overrides layer over the base recipe under the settled LAYER ruling
//! (nova `.cn-*` rules layer over base classes; silence keeps the base
//! value). `.cn-checkbox` (`style-nova.css:283-285`), verbatim:
//!
//! ```text
//! @apply border-input dark:bg-input/30 data-checked:bg-primary
//! data-checked:text-primary-foreground dark:data-checked:bg-primary
//! data-checked:border-primary aria-invalid:aria-checked:border-primary
//! aria-invalid:border-destructive dark:aria-invalid:border-destructive/50
//! focus-visible:border-ring focus-visible:ring-ring/50
//! aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 flex
//! size-4 items-center justify-center rounded-[4px] border transition-colors
//! group-has-disabled/field:opacity-50 focus-visible:ring-3 aria-invalid:ring-3
//! group-has-[:focus-visible]/field-label:ring-0
//! group-has-[:focus-visible]/field-label:not-data-checked:border-input
//! group-has-[:focus-visible]/field-label:data-checked:border-primary;
//! ```
//!
//! `.cn-checkbox-aria` (`style-nova.css:287-289`) mirrors the same values
//! with `data-selected` / `data-focus-visible` / `data-invalid` variants
//! (where the rule is applied is OPEN-5; its values agree with
//! `.cn-checkbox` on every slot). `.cn-checkbox-indicator`
//! (`style-nova.css:291-293`) restates the 14px icon box
//! (`[&>svg]:size-3.5`).
//!
//! Nova deltas that matter here (frozen spec §§2–4): nova ADDS
//! `aria-invalid:ring-3`, so an invalid-but-unfocused nova checkbox SHOWS a
//! 3px destructive ring, while the base recipe (ring colour only, no width
//! rule unfocused) shows the destructive border with NO ring; nova sets the
//! dark invalid border to `destructive/50`; nova states the invalid+checked
//! border explicitly (`primary`); nova REPLACES the transition property
//! with `transition-colors`. Base `shadow-xs` survives under nova (nova is
//! silent on shadow for this slot). Geometry is identical in both styles:
//! 16px box, literal 4px radius, 1px border inside the box, 14px icon
//! centred, 3px ring OUTSIDE the border box (22px painted footprint).
//!
//! Animation (spec §6): the transition PROPERTY SETS are frozen; duration
//! and easing are the Tailwind v4 framework defaults (150ms,
//! cubic-bezier(0.4, 0, 0.2, 1)), labelled INTERIM (OPEN-8 — not
//! snapshot-verified, never presented as verified). Base:
//! `transition-shadow` — only the ring fades in/out; colours snap. Nova:
//! `transition-colors` — colours ease; the ring snaps. The indicator is
//! `transition-none` in both styles: the icon appears/disappears
//! instantly (its colour follows the Root text-colour leg under nova).
//! The 75ms cross-fade the previous build used has no snapshot basis and
//! is gone. Disabled opacity is instant in both styles (opacity is in
//! neither transition set).
//!
//! Indeterminate (spec §4.4, OPEN-2 CLOSED by S6 Work B, D3): the box
//! renders the UNCHECKED recipe, and the Indicator renders the `Check`
//! icon — the Radix `Presence` rule renders the Indicator whenever the
//! state is indeterminate, with the consumer's child (the shadcn
//! `CheckIcon`). The glyph paints in the inherited text colour
//! (`text-current`): the Root sets no text colour in this state, so it
//! inherits the ancestor text colour, the theme `foreground`. The lucide
//! minus glyph the previous build rendered was a port invention and
//! stays removed (D3 unexercised — no deviation is needed to render the
//! `CheckIcon`).
//!
//! Keyboard/click (spec §7, OPEN-1): S6 Work B settled the Radix
//! semantics from the public Radix Checkbox source (OPEN-1(a)–(h)
//! CLOSED). Space toggles on KEYUP, once per press, with no repeat
//! while held (the native `<button>` rule — Radix adds no Space
//! handler). egui fires Space on keydown, so this build intervenes
//! (see `space_keyup_fire`, the button BTN-INT-04 shape) to toggle on
//! keyup. Enter never toggles: the build suppresses Enter keydown
//! events before `interact` (see `suppress_enter_activation`,
//! CHK-INT-03), the Radix `preventDefault()` equivalent. A direct
//! click focuses AND toggles (CHK-INT-01): egui's `interact` never
//! grants focus on a pointer click, so the build requests focus on
//! `clicked()`, the button BTN-INT-01 shape. Label-click composition
//! (spec §7.3) is implemented (CHK-INT-04): the label side is settled
//! — a label click fires a click on the control AND focuses it. The
//! label requests focus on the checkbox's returned [`Response::id`]
//! and pushes a synthetic Space pair, which `interact` reports as a
//! click on the focused widget (the pair survives the Space-keyup
//! intervention: both halves arrive in one frame, so the hidden
//! keydown is re-created on that same frame). In the control-first
//! (shadcn) layout the label runs AFTER the checkbox, so the
//! checkbox consumes its own pending click at the top of `show`,
//! before `interact`; in label-first order the label already
//! consumed it and the call is a no-op. Bind labels with the
//! returned `Response::id` (scoped-widget id rule).
//!
//! Combined-state precedence (spec §4.6, OPEN-4) is UNESTABLISHED except
//! for the nova invalid+checked border (`primary`, explicit). The
//! precedence this build implements (checked wins the border over
//! invalid/focus; invalid wins the ring colour over focus) is
//! PROVISIONAL, labelled as such in [`resolve_checkbox_colors`] and
//! pinned by a test whose name says so. S5 must not treat it as the spec
//! value until OPEN-4 closes.
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

use crate::components::attachment::mix;
use crate::components::label::consume_pending_click;
use crate::icons::Icon;
use crate::theme::{blend, color_alpha, Palette, ShadcnTheme, Style};
use crate::utils::animation::{
    transition_alpha, transition_color_to, transition_progress, Easing, TransitionSpec,
};
use crate::utils::focus::paint_ring;
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Color32, CornerRadius, CursorIcon, Painter, Rect, Response, Sense, Stroke, Ui, Vec2, Widget,
};

/// `size-4` — the checkbox side length.
pub const CHECKBOX_SIZE: f32 = 16.0;
/// `rounded-[4px]` — a literal 4px radius, not the theme scale.
pub const CHECKBOX_RADIUS: u8 = 4;
/// `size-3.5` — the indicator icon size.
const INDICATOR_SIZE: f32 = 14.0;
/// `ring-[3px]` / `ring-3` — the focus/invalid ring width.
const RING_WIDTH: f32 = 3.0;

/// Resolved colors for one checkbox state (pure, unit-testable).
///
/// Every field is the value BEFORE the disabled element-opacity fade
/// (spec §4.3, applied in the paint path) and BEFORE the §6 transition
/// animation (also the paint path). The resolver owns the single-state
/// rules; combined-state cells marked PROVISIONAL below are OPEN-4
/// choices, not spec values.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CheckboxColors {
    /// Box fill: `primary` when checked; transparent (light) or `input/30`
    /// (dark) when unchecked.
    fill: Color32,
    /// 1px border stroke. See [`resolve_checkbox_colors`] for the
    /// (partly provisional) precedence.
    stroke: Stroke,
    /// Ring color (alpha already applied): `ring/50` when valid,
    /// `destructive/20` (light) / `destructive/40` (dark) when invalid.
    /// The focus+invalid ring colour is PROVISIONAL (OPEN-4).
    ring: Color32,
    /// Whether the ring paints: focused, or invalid under nova (the nova
    /// `aria-invalid:ring-3` rest ring, spec §4.5). A base invalid
    /// checkbox shows NO ring unfocused.
    ring_shown: bool,
    /// Indicator icon color when rendered: `primary-foreground` when
    /// checked, `foreground` when indeterminate (the `text-current`
    /// glyph inherits the ancestor text colour — the Root sets no text
    /// colour in the indeterminate state). The unchecked value
    /// (`foreground`) doubles as the leg start for the nova icon-colour
    /// transition; no icon renders for a plain unchecked box.
    icon: Color32,
    /// Whether `shadow-xs` paints. Always true: no rule in either recipe
    /// removes the shadow in any state (spec §4.7), including disabled
    /// (where it paints at the disabled element opacity).
    shadow_shown: bool,
}

/// Resolves the checkbox recipe for one state.
///
/// Base recipe (`checkbox.tsx:16`): `border-input dark:bg-input/30`,
/// `data-[state=checked]:border-primary data-[state=checked]:bg-primary
/// data-[state=checked]:text-primary-foreground`,
/// `aria-invalid:border-destructive aria-invalid:ring-destructive/20
/// dark:aria-invalid:ring-destructive/40`,
/// `focus-visible:border-ring focus-visible:ring-[3px]
/// focus-visible:ring-ring/50`.
///
/// Nova deltas (`style-nova.css:283-285`): `dark:aria-invalid:border-destructive/50`,
/// `aria-invalid:ring-3` (invalid rest ring), and the explicit
/// `aria-invalid:aria-checked:border-primary` (invalid+checked border is
/// `primary` — FROZEN, spec §4.6).
///
/// `focused` is the RAW `has_focus()` value; the resolver applies the
/// disabled gate itself (a disabled control cannot hold `:focus-visible`,
/// so no focus border/ring shows when `disabled`).
///
/// Border precedence — FROZEN cells vs PROVISIONAL (OPEN-4) cells:
/// - checked wins over invalid under NOVA: FROZEN (explicit rule above).
/// - checked wins over invalid under BASE: PROVISIONAL (OPEN-4(a)).
/// - invalid wins over focus (both styles): PROVISIONAL (OPEN-4(b)).
/// - checked wins over focus (both styles): PROVISIONAL (OPEN-4(d)).
/// - Ring colour, invalid wins over focus (both styles): PROVISIONAL
///   (OPEN-4(c)).
///
/// The provisional choices keep the previous build's order (checked >
/// invalid > focused); they are implementation rationale, NOT snapshot
/// evidence, and S5 must not treat them as spec values until OPEN-4
/// closes.
pub(crate) fn resolve_checkbox_colors(
    p: &Palette,
    dark: bool,
    style: Style,
    checked: bool,
    invalid: bool,
    focused: bool,
    disabled: bool,
) -> CheckboxColors {
    let nova = style == Style::Nova;
    // A disabled control cannot hold `:focus-visible` (spec §4.3 +
    // OPEN-1 disabled rows): the focus border/ring never show disabled.
    // The nova invalid rest ring is NOT focus-gated, so it still shows
    // when disabled (at the disabled element opacity, §4.3).
    let focused = focused && !disabled;
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
    let stroke_color = if checked {
        // `data-[state=checked]:border-primary` / `data-checked:border-primary`.
        // FROZEN for nova invalid+checked; PROVISIONAL (OPEN-4) for base
        // invalid+checked and for focus+checked in both styles.
        p.primary
    } else if invalid {
        if nova && dark {
            // `dark:aria-invalid:border-destructive/50` (nova only; N5).
            color_alpha(p.destructive, 0.5)
        } else {
            p.destructive
        }
        // PROVISIONAL (OPEN-4(b)): invalid wins over focus for the border.
    } else if focused {
        // `focus-visible:border-ring`.
        p.ring
    } else {
        stroke_color
    };
    CheckboxColors {
        fill,
        stroke: Stroke::new(1.0, stroke_color),
        ring,
        ring_shown: focused || (invalid && nova),
        icon,
        shadow_shown: true,
    }
}

/// Which indicator icon renders, if any.
///
/// Checked renders the Lucide `Check` (`checkbox.tsx:25`). The
/// indeterminate state ALSO renders the `Check` (spec §4.4, OPEN-2
/// CLOSED by S6 Work B: the Radix Indicator `Presence` renders whenever
/// the state is indeterminate, with the consumer's child — the shadcn
/// `CheckIcon`; the glyph is `text-current`, resolved to the theme
/// `foreground` by [`resolve_checkbox_colors`]). The minus glyph the
/// previous build rendered was a port invention (the original supplies
/// only `CheckIcon`); D3 stays unexercised. When both `checked` and
/// `indeterminate` are set, the checked rendering wins (spec E5; OPEN-3
/// ratified at S8).
fn indicator_icon(checked: bool, indeterminate: bool) -> Option<Icon> {
    // OPEN-2 (settled): the Indicator renders for checked AND
    // indeterminate; only the plain unchecked box has no icon.
    (checked || indeterminate).then_some(Icon::Check)
}

/// Whole-element group fade for `disabled:opacity-50` (spec §4.3): the
/// enabled chrome composites at 50% over `backdrop` (the theme page
/// background — exact on the gallery page, where the panel fill is
/// `p.background`; on other backdrops the overlap pixels use the theme
/// background as the fade target, a recorded limitation, same as the
/// toggle S1 precedent).
///
/// Per-slot alpha halving would double-fade the overlap pixels (a halved
/// stroke over a halved fill reads 25% at the border band — the CHK-VIS-01
/// defect). Instead: the fill fades by halving its alpha (it paints over
/// the backdrop alone, where halving IS the group fade), while the stroke
/// and icon colours are pre-composited OPAQUE against the backdrop:
/// `0.5 * composite + 0.5 * backdrop`, where `composite` is the enabled
/// stroke/icon colour over the enabled fill over the backdrop.
/// Anti-aliased edge pixels stay exact under this scheme: coverage `c`
/// over the faded fill yields the group fade of the enabled pixel.
/// Returns `(fill, stroke_color, icon_color)`.
fn disabled_group_fade(
    fill: Color32,
    stroke_color: Color32,
    icon_color: Color32,
    backdrop: Color32,
) -> (Color32, Color32, Color32) {
    let faded_fill = color_alpha(fill, 0.5);
    let fill_composite = blend(backdrop, fill);
    let stroke = mix(backdrop, blend(fill_composite, stroke_color), 0.5);
    let icon = mix(backdrop, blend(fill_composite, icon_color), 0.5);
    (faded_fill, stroke, icon)
}

/// Paints the checkbox's `shadow-xs` through the LAYER resolution API
/// (spec §4.7; the base recipe declares `shadow-xs` and nova is silent
/// on shadow for this slot, so it survives under every style — only Vega
/// restates it, with the identical value). When disabled, the shadow
/// fades with the element's 50% opacity (spec §4.3) by halving the
/// resolved layers' alphas — the button `paint_outline_shadow` shape.
fn paint_checkbox_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::Checkbox).layers();
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
            ShadowSlot::Checkbox,
            dark,
        );
    }
}

/// Space-keyup activation (spec §7, OPEN-1(a) closed by S6 Work B;
/// CHK-INT-02; the button BTN-INT-04 shape, as in `switch.rs`).
///
/// Native `<button>` toggles on Space KEYUP, once per press, with no
/// repeat while held; Radix adds no Space handler, so the native rule
/// governs. egui's `interact` reports a fake click whenever the focused
/// widget sees `key_pressed(Space)`, so without intervention Space
/// toggles on keydown (and re-toggles on every autorepeat). This helper,
/// called before `interact` while the checkbox holds focus, hides this
/// frame's Space keydown events from `interact`, remembers that Space
/// was pressed here ("armed"), and re-creates the keydown on the keyup
/// frame so `interact` reports the click then.
///
/// Only the event list `interact` reads is touched: `keys_down` is
/// computed in `begin_pass`, before widgets run, so `key_down(Space)` is
/// unaffected. Enter events are never touched here (Enter suppression
/// is `suppress_enter_activation`, CHK-INT-03). The synthetic keydown on the keyup
/// frame can only fake-click the focused widget, and copies the release
/// event's physical key and modifiers. A Space released without a
/// preceding keydown on this checkbox (focus arrived mid-hold) does not
/// toggle; a stale armed flag is cleared on the first frame the
/// checkbox is shown with Space up.
///
/// The label-forwarded synthetic Space pair (spec §7.3) still activates
/// through this helper: both halves arrive in one frame, so the hidden
/// keydown is re-created on that same frame and `interact` reports the
/// click.
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

/// Enter suppression (spec §7, OPEN-1(b) closed by S6 Work B;
/// CHK-INT-03; the event-list shape of `space_keyup_fire`).
///
/// Radix Checkbox explicitly preventDefaults Enter: "According to WAI
/// ARIA, Checkboxes don't activate on enter keypress" — Enter must
/// NEVER toggle, on any edge. egui's `interact` reports a fake click
/// whenever the focused widget sees `key_pressed(Enter)` (the keydown
/// frame, including autorepeat), so without intervention Enter toggles
/// exactly like Space. This helper, called before `interact` while the
/// checkbox holds focus, hides this frame's Enter keydown events
/// (including repeats) from `interact` and synthesizes nothing: Enter
/// is a complete no-op for the focused checkbox — no toggle, no focus
/// change.
///
/// Only the event list `interact` reads is touched: `keys_down` is
/// computed in `begin_pass`, before widgets run, so `key_down(Enter)` is
/// unaffected. The release events are left in place — nothing in
/// `interact` acts on a key release. The gate in `show` (focused and
/// enabled) is what keeps Enter working for every other widget: only
/// the focused widget can fake-click via Enter, so hiding the events
/// only while this checkbox holds focus changes no other widget's Enter
/// handling.
fn suppress_enter_activation(ctx: &egui::Context) {
    ctx.input_mut(|input| {
        input.events.retain(|event| {
            !matches!(
                event,
                egui::Event::Key {
                    key: egui::Key::Enter,
                    pressed: true,
                    ..
                }
            )
        });
    });
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

    /// Shows the indeterminate (mixed) state. Display-only; the caller
    /// owns the tri-state logic, and clicking toggles the bound boolean
    /// (the indeterminate → next-state rule is OPEN-1).
    ///
    /// Rendering (spec §4.4, OPEN-2 settled by S6 Work B): the box
    /// renders the unchecked recipe with the `Check` icon in the
    /// inherited text colour (`text-current` → theme `foreground`). The
    /// minus glyph the previous build rendered was a port invention and
    /// stays removed (D3 unexercised). When the bound boolean is `true`,
    /// the checked rendering wins (spec E5; OPEN-3 ratified at S8).
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Disables the checkbox (`disabled:opacity-50
    /// disabled:cursor-not-allowed`): the whole element — fill, border,
    /// icon, ring, and shadow — composites at 50% opacity (spec §4.3).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the checkbox aria-invalid. Border `destructive` in both
    /// styles (nova dark: `destructive/50`); ring colour
    /// `destructive/20` (light) / `/40` (dark). Under nova the invalid
    /// ring is visible WITHOUT focus (`aria-invalid:ring-3`); under the
    /// base recipe it shows only when focused (spec §4.5).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so focus state persists per instance.
    /// Label bindings must capture the returned [`Response::id`]
    /// (scoped-widget id rule), not re-derive the id from the salt.
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
        // Reserve space without an auto-id widget so egui's built-in
        // spatial arrow navigation only sees the explicit widget id.
        let (_, rect) = ui.allocate_space(Vec2::splat(CHECKBOX_SIZE));
        // A bound label's click recorded on a previous pass is
        // forwarded here, before `interact` (spec §7.3; CHK-INT-04).
        // The label pushes the synthetic Space pair at the top of
        // `Label::show`, which runs AFTER the checkbox in the
        // control-first (shadcn) layout — so the pair arrived after
        // this checkbox's `interact` and died unseen, leaving focus
        // without activation. Consuming the pending click here makes
        // the activation reach the checkbox in either show order: in
        // label-first order the label already consumed the flag and
        // this is a no-op, so exactly one activation fires. The flag
        // is consumed whether or not the checkbox is enabled, gated on
        // the checkbox's own enabled state: a DISABLED checkbox drops
        // the flag silently (CHK-INT-05) instead of leaving it for the
        // label's own consume — the label's consume requests focus
        // unconditionally (gated only on the label's enabled state),
        // and in control-first order that request lands after this
        // checkbox's `interact`, the only thing that would clear it, so
        // the disabled pair's readout observes focus during the
        // forwarding pass. Dropping the flag here makes the disabled
        // pair do nothing, per spec §7.3 point 4.
        consume_pending_click(ui, id, !self.disabled);
        // Space toggles on keyup, not keydown (spec §7, OPEN-1(a)
        // closed by S6 Work B; CHK-INT-02): hide this frame's Space
        // keydown from `interact` and re-create it on the keyup frame.
        // Enter never toggles (spec §7, OPEN-1(b) closed by S6 Work B;
        // CHK-INT-03): hide this frame's Enter keydown from `interact`
        // and synthesize nothing. Only while focused and enabled — an
        // unfocused checkbox must not consume another widget's keys,
        // and a disabled checkbox never activates.
        if !self.disabled && ui.ctx().memory(|m| m.has_focus(id)) {
            space_keyup_fire(ui.ctx(), id);
            suppress_enter_activation(ui.ctx());
        }
        let mut response = ui.interact(rect, id, sense);

        if self.disabled {
            // `disabled:cursor-not-allowed` (spec §2.1; D6: S6 records
            // the observed cursor behaviour).
            response = response.on_hover_cursor(CursorIcon::NotAllowed);
        }

        // Activation (spec §7, OPEN-1(a)–(b) closed by S6 Work B):
        // Space toggles on keyup (see `space_keyup_fire`), pointer
        // clicks arrive here as `clicked()`, including the synthetic
        // Space pair a bound label forwards (spec §7.3). Enter never
        // reaches here: it is suppressed before `interact` (see
        // `suppress_enter_activation`, CHK-INT-03).
        let clicked = response.clicked() && !self.disabled;
        if clicked {
            *self.checked = !*self.checked;
            // Click focuses AND toggles (spec §7.2, OPEN-1(e) closed by
            // S6 Work B; CHK-INT-01; the button BTN-INT-01 / `TextEdit`
            // shape): egui's `interact` never grants focus on a pointer
            // click — `get_response` only SURRENDERS focus when the
            // pointer clicks elsewhere — so the widget requests focus
            // itself. Gated on `clicked` (the release frame), so a
            // press-drag-off-release focuses nothing, and on
            // `!disabled`: a disabled box takes neither toggle nor
            // focus. Requesting before painting lets `has_focus()` below
            // see the focus on the click frame itself. A fake click from
            // the label-forwarded Space pair re-requests focus the label
            // already granted — idempotent; the forwarded activation
            // reaches the checkbox via the pending-click consume above
            // (CHK-INT-04).
            response.request_focus();
        }

        if ui.is_rect_visible(rect) {
            let checked = *self.checked;
            let focused = response.has_focus();
            let nova = theme.style == Style::Nova;
            // Spec §6: the transition PROPERTY SET differs by style, so
            // the spec each animator runs under differs by style too.
            // Timing is `TransitionSpec::default()` (150ms + EaseInOut,
            // the Tailwind v4 framework defaults), labelled INTERIM per
            // OPEN-8 — not snapshot-verified. A zero-duration spec snaps
            // instantly (foundation-tested), which is how the
            // non-transitioned property set is implemented: the animator
            // still runs every frame, so its state stays fresh across
            // style switches.
            let color_spec = if nova {
                TransitionSpec::default()
            } else {
                TransitionSpec::secs(0.0, Easing::EaseInOut)
            };
            let shadow_spec = if nova {
                TransitionSpec::secs(0.0, Easing::EaseInOut)
            } else {
                TransitionSpec::default()
            };
            let off = resolve_checkbox_colors(
                &p,
                dark,
                theme.style,
                false,
                self.invalid,
                focused,
                self.disabled,
            );
            let on = resolve_checkbox_colors(
                &p,
                dark,
                theme.style,
                true,
                self.invalid,
                focused,
                self.disabled,
            );
            let current = if checked { on } else { off };
            // Fill leg: a boolean drive on `checked` plus a premultiplied
            // mix of the two endpoint fills. `transition_color_to` is
            // deliberately NOT used here: the light unchecked fill is
            // TRANSPARENT, and OKLab mixing from a transparent endpoint
            // contributes black; the premultiplied `mix` cross-fades
            // transparent → primary cleanly. (Consequence, recorded for
            // S7: a theme-switch recolour of the fill snaps — the drive
            // only tracks `checked` — where the button precedent animates
            // theme recolours. The spec §6 is silent on theme-switch
            // legs.)
            let t = transition_progress(ui.ctx(), id.with("fill"), checked, color_spec).t;
            let mut fill = mix(off.fill, on.fill, t);
            // Border and icon colours: endpoint tracking, so focus /
            // invalid / checked flips all animate under nova and snap
            // under base. Neither slot crosses the transparent boundary,
            // so `transition_color_to`'s caveat does not apply.
            let mut stroke_color = transition_color_to(
                ui.ctx(),
                id.with("border"),
                current.stroke.color,
                color_spec,
            )
            .color;
            let mut icon_color =
                transition_color_to(ui.ctx(), id.with("icon"), current.icon, color_spec).color;
            // Ring: colour + visibility both ride the SHADOW spec — the
            // ring is a `box-shadow` layer, so it fades under base
            // (`transition-shadow`) and snaps under nova
            // (`transition-colors` excludes box-shadow).
            let mut ring_color =
                transition_color_to(ui.ctx(), id.with("ring"), current.ring, shadow_spec).color;
            let ring_alpha = transition_alpha(
                ui.ctx(),
                id.with("ring_alpha"),
                0.0,
                1.0,
                current.ring_shown,
                shadow_spec,
            )
            .alpha;
            if self.disabled {
                // `disabled:opacity-50` — the WHOLE element composites at
                // 50% (spec §4.3, CSS `opacity` semantics), applied
                // instantly: opacity is in neither style's transition
                // set. Group fade via [`disabled_group_fade`], NOT
                // per-slot halving: halving each slot double-fades the
                // overlap pixels (CHK-VIS-01). The ring keeps halving:
                // it paints over the backdrop alone, where halving IS
                // the group fade (the shadow halo beneath it is
                // sub-1-level).
                (fill, stroke_color, icon_color) =
                    disabled_group_fade(fill, stroke_color, icon_color, p.background);
                ring_color = color_alpha(ring_color, 0.5);
            }
            let corner = CornerRadius::same(CHECKBOX_RADIUS);
            let painter = ui.painter();

            // `shadow-xs` in ALL states (spec §4.7), via the LAYER
            // resolution; faded when disabled (see `paint_checkbox_shadow`).
            if current.shadow_shown {
                paint_checkbox_shadow(painter, rect, corner, theme.style, dark, self.disabled);
            }

            painter.rect_filled(rect, corner, fill);
            painter.rect_stroke(
                rect,
                corner,
                Stroke::new(1.0, stroke_color),
                egui::epaint::StrokeKind::Inside,
            );

            // Indicator: `transition-none` — presence is instant in both
            // styles (spec §6); only the colour rides the nova leg above.
            if let Some(icon_kind) = indicator_icon(checked, self.indeterminate) {
                let icon_rect =
                    egui::Rect::from_center_size(rect.center(), Vec2::splat(INDICATOR_SIZE));
                icon_kind.paint(painter, icon_rect, icon_color, 2.0 * INDICATOR_SIZE / 24.0);
            }

            // Ring on the UNEXPANDED rect: `paint_ring` strokes OUTSIDE,
            // so the 3px band starts at the outer edge of the border box
            // (spec §3.3, 22px footprint). The previous build expanded
            // the rect by 1px first, leaving a 1px gap.
            if ring_alpha > 0.0 {
                paint_ring(
                    painter,
                    rect,
                    color_alpha(ring_color, ring_alpha),
                    RING_WIDTH,
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

    /// Non-nova style standing in for the base recipe in tests: the spec
    /// covers base + nova only, and non-nova styles render the base
    /// recipe (documented in the test module, as in the switch S1).
    const BASE_STYLE: Style = Style::Vega;

    fn colors(
        mode: Mode,
        style: Style,
        checked: bool,
        invalid: bool,
        focused: bool,
        disabled: bool,
    ) -> CheckboxColors {
        let p = palette(mode);
        resolve_checkbox_colors(
            &p,
            mode.is_dark(),
            style,
            checked,
            invalid,
            focused,
            disabled,
        )
    }

    #[test]
    fn checked_uses_primary() {
        for mode in [Mode::Light, Mode::Dark] {
            for style in [BASE_STYLE, Style::Nova] {
                let p = palette(mode);
                let c = colors(mode, style, true, false, false, false);
                assert_eq!(c.fill, p.primary);
                assert_eq!(c.stroke.color, p.primary);
                assert_eq!(c.icon, p.primary_foreground);
            }
        }
        let p = palette(Mode::Light);
        let u = colors(Mode::Light, BASE_STYLE, false, false, false, false);
        assert_eq!(u.fill, Color32::TRANSPARENT);
        assert_eq!(u.stroke.color, p.input);
        assert_eq!(u.icon, p.foreground);
    }

    #[test]
    fn dark_unchecked_uses_input_30() {
        let dark = palette(Mode::Dark);
        for style in [BASE_STYLE, Style::Nova] {
            let u = colors(Mode::Dark, style, false, false, false, false);
            assert_eq!(u.fill, color_alpha(dark.input, 0.3));
        }
    }

    #[test]
    fn focus_border_is_ring_when_unchecked() {
        // FROZEN (spec §4.2): `focus-visible:border-ring` on the valid
        // unchecked box, both styles, both modes; ring 3px `ring/50`.
        for mode in [Mode::Light, Mode::Dark] {
            for style in [BASE_STYLE, Style::Nova] {
                let p = palette(mode);
                let f = colors(mode, style, false, false, true, false);
                assert_eq!(f.stroke.color, p.ring);
                assert!(f.ring_shown);
                assert_eq!(f.ring, color_alpha(p.ring, 0.5));
            }
        }
        // Unfocused + valid: no ring in either style.
        for style in [BASE_STYLE, Style::Nova] {
            assert!(!colors(Mode::Light, style, false, false, false, false).ring_shown);
        }
    }

    #[test]
    fn precedence_choices_are_provisional_pending_open4() {
        // PROVISIONAL (spec §4.6, OPEN-4): these assertions pin the
        // BUILD's choices, not spec values. S5 must not treat them as
        // the spec until OPEN-4 closes; S9 corrects them if OPEN-4
        // settles otherwise.
        let p = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        // (a) Base invalid+checked border: checked wins (provisional).
        assert_eq!(
            colors(Mode::Light, BASE_STYLE, true, true, false, false)
                .stroke
                .color,
            p.primary
        );
        // (b) Focus+invalid border: invalid wins (provisional), both styles.
        for style in [BASE_STYLE, Style::Nova] {
            assert_eq!(
                colors(Mode::Light, style, false, true, true, false)
                    .stroke
                    .color,
                p.destructive
            );
        }
        // (d) Focus+checked border: checked wins (provisional), both styles.
        for style in [BASE_STYLE, Style::Nova] {
            assert_eq!(
                colors(Mode::Light, style, true, false, true, false)
                    .stroke
                    .color,
                p.primary
            );
        }
        // (c) Focus+invalid ring colour: invalid wins (provisional).
        assert_eq!(
            colors(Mode::Light, BASE_STYLE, false, true, true, false).ring,
            color_alpha(p.destructive, 0.2)
        );
        assert_eq!(
            colors(Mode::Dark, Style::Nova, false, true, true, false).ring,
            color_alpha(dark.destructive, 0.4)
        );
    }

    #[test]
    fn nova_invalid_checked_border_is_primary() {
        // FROZEN (spec §4.6): `aria-invalid:aria-checked:border-primary`
        // (`style-nova.css:284`) — the one explicit combined-state rule.
        for mode in [Mode::Light, Mode::Dark] {
            let p = palette(mode);
            assert_eq!(
                colors(mode, Style::Nova, true, true, false, false)
                    .stroke
                    .color,
                p.primary
            );
        }
    }

    #[test]
    fn nova_dark_invalid_border_fades() {
        let dark = palette(Mode::Dark);
        let base = colors(Mode::Dark, BASE_STYLE, false, true, false, false);
        assert_eq!(base.stroke.color, dark.destructive);
        let nova = colors(Mode::Dark, Style::Nova, false, true, false, false);
        // `dark:aria-invalid:border-destructive/50`
        assert_eq!(nova.stroke.color, color_alpha(dark.destructive, 0.5));
        // Light keeps the full destructive border in both styles.
        let p = palette(Mode::Light);
        assert_eq!(
            colors(Mode::Light, Style::Nova, false, true, false, false)
                .stroke
                .color,
            p.destructive
        );
    }

    #[test]
    fn invalid_ring_colours() {
        // FROZEN (spec §4.5): ring COLOUR is destructive-based in both
        // styles; only the unfocused WIDTH differs (see the visibility
        // matrix test below).
        let p = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        for style in [BASE_STYLE, Style::Nova] {
            assert_eq!(
                colors(Mode::Light, style, false, true, false, false).ring,
                color_alpha(p.destructive, 0.2)
            );
            assert_eq!(
                colors(Mode::Dark, style, false, true, false, false).ring,
                color_alpha(dark.destructive, 0.4)
            );
        }
    }

    #[test]
    fn ring_visibility_matrix() {
        // Spec §4.5: the base recipe has NO unfocused invalid ring
        // (colour only, no width rule); nova ADDS `aria-invalid:ring-3`,
        // so the invalid ring shows without focus under nova only.
        for mode in [Mode::Light, Mode::Dark] {
            // Invalid + unfocused: nova shows, base does not.
            assert!(colors(mode, Style::Nova, false, true, false, false).ring_shown);
            assert!(!colors(mode, BASE_STYLE, false, true, false, false).ring_shown);
            // Invalid + focused: both show (the focus rule supplies the
            // width under base).
            assert!(colors(mode, Style::Nova, false, true, true, false).ring_shown);
            assert!(colors(mode, BASE_STYLE, false, true, true, false).ring_shown);
            // Invalid + checked + unfocused: same split (checked does not
            // affect ring visibility).
            assert!(colors(mode, Style::Nova, true, true, false, false).ring_shown);
            assert!(!colors(mode, BASE_STYLE, true, true, false, false).ring_shown);
        }
        // Disabled: the focus ring never shows (a disabled control
        // cannot hold `:focus-visible`), but the nova invalid rest ring
        // still shows (it is not focus-gated; §4.3 fades it to 50%).
        assert!(!colors(Mode::Light, Style::Nova, false, false, true, true).ring_shown);
        assert!(!colors(Mode::Light, BASE_STYLE, false, false, true, true).ring_shown);
        assert!(colors(Mode::Light, Style::Nova, false, true, false, true).ring_shown);
        assert!(!colors(Mode::Light, BASE_STYLE, false, true, false, true).ring_shown);
    }

    #[test]
    fn shadow_shown_in_every_state() {
        // Spec §4.7: `shadow-xs` applies in every state, both styles,
        // both modes, including disabled and focused.
        for mode in [Mode::Light, Mode::Dark] {
            for style in [BASE_STYLE, Style::Nova] {
                for checked in [false, true] {
                    for invalid in [false, true] {
                        for focused in [false, true] {
                            for disabled in [false, true] {
                                assert!(
                                    colors(mode, style, checked, invalid, focused, disabled)
                                        .shadow_shown,
                                    "shadow dropped: {mode:?} {style:?} checked={checked} invalid={invalid} focused={focused} disabled={disabled}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn geometry_constants_match_spec() {
        // Spec §3: `size-4` box, literal 4px radius, `size-3.5` icon,
        // 3px ring.
        assert_eq!(CHECKBOX_SIZE, 16.0);
        assert_eq!(CHECKBOX_RADIUS, 4);
        assert_eq!(INDICATOR_SIZE, 14.0);
        assert_eq!(RING_WIDTH, 3.0);
    }

    #[test]
    fn indicator_icon_renders_check_for_indeterminate() {
        // Settled OPEN-2 (S6 Work B): the Radix Indicator renders the
        // `CheckIcon` for checked AND indeterminate; only the plain
        // unchecked box has no icon. The E5 combination (checked +
        // indeterminate) renders the checked icon (checked wins; OPEN-3
        // ratified at S8). CHK-VIS-02 regression pin.
        assert_eq!(indicator_icon(true, false), Some(Icon::Check));
        assert_eq!(indicator_icon(false, false), None);
        assert_eq!(indicator_icon(false, true), Some(Icon::Check));
        assert_eq!(indicator_icon(true, true), Some(Icon::Check));
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
    fn space_fires_on_keyup_not_keydown() {
        // CHK-INT-02: native `<button>` behaviour (spec §7, OPEN-1(a)
        // closed by S6 Work B) — Space toggles once, on keyup; no
        // toggle on keydown, no hold-repeat. Mirrors the switch pin
        // `space_fires_on_keyup_not_keydown` (SWI-INT-01), adapted to
        // the checkbox's `run_pass` harness.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let key_event = |key: egui::Key, pressed: bool, repeat: bool| egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let mut checked = false;

        // Frame 1: render, record id; then give the checkbox focus.
        let mut id = egui::Id::NULL;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            id = Checkbox::new(&mut checked).id_salt("keyup").show(ui).id;
        });
        ctx.memory_mut(|m| m.request_focus(id));
        assert!(ctx.memory(|m| m.has_focus(id)));

        let mut frame = |ctx: &egui::Context, input: egui::RawInput| {
            run_pass(ctx, input, |ui| {
                let _ = Checkbox::new(&mut checked).id_salt("keyup").show(ui);
            });
            checked
        };

        // Space keydown: no toggle. Autorepeat keydown: no toggle. Hold
        // frame with no events: no toggle.
        assert!(!frame(&ctx, key_event(egui::Key::Space, true, false)));
        assert!(!frame(&ctx, key_event(egui::Key::Space, true, true)));
        assert!(!frame(&ctx, egui::RawInput::default()));
        // Space keyup: exactly one toggle. The frame after: no toggle.
        assert!(frame(&ctx, key_event(egui::Key::Space, false, false)));
        assert!(frame(&ctx, egui::RawInput::default()));
        // A second press/release cycle toggles exactly once, on keyup.
        assert!(frame(&ctx, key_event(egui::Key::Space, true, false)));
        assert!(!frame(&ctx, key_event(egui::Key::Space, false, false)));
        // Focus is retained after keyboard activation.
        assert!(ctx.memory(|m| m.has_focus(id)));
        // The label-forwarded synthetic Space pair still activates
        // through the keyup handling (both halves in one frame).
        assert!(frame(&ctx, space_press_input()));

        // A disabled checkbox never toggles from Space, on either edge.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let _ = Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("keyup_dis")
                .show(ui);
        });
        let mut disabled_frame = |input: egui::RawInput| {
            run_pass(&ctx, input, |ui| {
                let _ = Checkbox::new(&mut checked)
                    .disabled(true)
                    .id_salt("keyup_dis")
                    .show(ui);
            });
            checked
        };
        assert!(!disabled_frame(key_event(egui::Key::Space, true, false)));
        assert!(!disabled_frame(key_event(egui::Key::Space, false, false)));
    }

    #[test]
    fn enter_never_toggles() {
        // CHK-INT-03: Radix Checkbox preventDefaults Enter (spec §7,
        // OPEN-1(b) closed by S6 Work B; WAI-ARIA: checkboxes don't
        // activate on Enter) — Enter never toggles, on any edge.
        // Mirrors the Space pin above, plus the Space path kept working.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let key_event = |key: egui::Key, pressed: bool, repeat: bool| egui::RawInput {
            events: vec![egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let mut checked = false;

        // Frame 1: render, record id; then give the checkbox focus.
        let mut id = egui::Id::NULL;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            id = Checkbox::new(&mut checked).id_salt("enter").show(ui).id;
        });
        ctx.memory_mut(|m| m.request_focus(id));
        assert!(ctx.memory(|m| m.has_focus(id)));

        let mut frame = |ctx: &egui::Context, input: egui::RawInput| {
            run_pass(ctx, input, |ui| {
                let _ = Checkbox::new(&mut checked).id_salt("enter").show(ui);
            });
            checked
        };

        // Enter keydown: no toggle. Autorepeat keydown: no toggle. Hold
        // frame with no events: no toggle. Enter keyup: no toggle. The
        // frame after: no toggle. A second press/release cycle: no toggle.
        assert!(!frame(&ctx, key_event(egui::Key::Enter, true, false)));
        assert!(!frame(&ctx, key_event(egui::Key::Enter, true, true)));
        assert!(!frame(&ctx, egui::RawInput::default()));
        assert!(!frame(&ctx, key_event(egui::Key::Enter, false, false)));
        assert!(!frame(&ctx, egui::RawInput::default()));
        assert!(!frame(&ctx, key_event(egui::Key::Enter, true, false)));
        assert!(!frame(&ctx, key_event(egui::Key::Enter, false, false)));
        // Focus is retained after Enter (it is a no-op, not a blur).
        assert!(ctx.memory(|m| m.has_focus(id)));
        // Space still toggles exactly once, on keyup (CHK-INT-02
        // unregressed by the Enter suppression).
        assert!(!frame(&ctx, key_event(egui::Key::Space, true, false)));
        assert!(frame(&ctx, key_event(egui::Key::Space, false, false)));
        // The label-forwarded synthetic Space pair still activates.
        assert!(!frame(&ctx, space_press_input()));

        // A disabled checkbox never toggles from Enter, on either edge.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let _ = Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("enter_dis")
                .show(ui);
        });
        let mut disabled_frame = |input: egui::RawInput| {
            run_pass(&ctx, input, |ui| {
                let _ = Checkbox::new(&mut checked)
                    .disabled(true)
                    .id_salt("enter_dis")
                    .show(ui);
            });
            checked
        };
        assert!(!disabled_frame(key_event(egui::Key::Enter, true, false)));
        assert!(!disabled_frame(key_event(egui::Key::Enter, false, false)));
    }

    #[test]
    fn click_focuses_and_toggles() {
        // CHK-INT-01: native `<button>` behaviour (spec §7.2, OPEN-1(e)
        // closed by S6 Work B) — a direct click focuses AND toggles.
        // Mirrors the button pin `click_moves_keyboard_focus_to_the_button`
        // (BTN-INT-01), adapted to the checkbox's `run_pass` harness,
        // plus the drag-off and post-click Space legs.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let click_event = |pos: egui::Pos2, pressed: bool| egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
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
        let mut checked = false;

        // Frame 1: render, record rect + id; nothing focused yet.
        let mut rect = egui::Rect::NOTHING;
        let mut id = egui::Id::NULL;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let r = Checkbox::new(&mut checked).id_salt("click").show(ui);
            rect = r.rect;
            id = r.id;
        });
        assert!(!ctx.memory(|m| m.has_focus(id)));
        let center = rect.center();

        // Press alone: no toggle, no focus (focus is requested on the
        // click — the release frame — so a press-drag-off-release
        // focuses nothing).
        run_pass(&ctx, click_event(center, true), |ui| {
            let _ = Checkbox::new(&mut checked).id_salt("click").show(ui);
        });
        assert!(!checked, "press alone must not toggle");
        assert!(
            !ctx.memory(|m| m.has_focus(id)),
            "press alone must not focus"
        );

        // Release on the box: the click toggles AND focuses, and the
        // click frame itself already sees focus (the request happens
        // before painting reads `has_focus()`).
        let mut clicked = false;
        let mut focused_on_click_frame = false;
        run_pass(&ctx, click_event(center, false), |ui| {
            let r = Checkbox::new(&mut checked).id_salt("click").show(ui);
            clicked = r.clicked();
            focused_on_click_frame = r.has_focus();
        });
        assert!(clicked, "press + release must register a click");
        assert!(checked, "click must toggle");
        assert!(focused_on_click_frame, "click frame must already see focus");
        assert!(
            ctx.memory(|m| m.has_focus(id)),
            "click must move keyboard focus to the box"
        );

        // Space after a click toggles (focus retained, CHK-INT-02
        // keyup path): keydown no toggle, keyup exactly one toggle.
        run_pass(&ctx, key_event(egui::Key::Space, true), |ui| {
            let _ = Checkbox::new(&mut checked).id_salt("click").show(ui);
        });
        assert!(checked, "Space keydown must not toggle");
        run_pass(&ctx, key_event(egui::Key::Space, false), |ui| {
            let _ = Checkbox::new(&mut checked).id_salt("click").show(ui);
        });
        assert!(!checked, "Space keyup after a click must toggle");
        assert!(ctx.memory(|m| m.has_focus(id)), "focus retained");

        // A second click toggles back and keeps focus.
        run_pass(&ctx, click_event(center, true), |ui| {
            let _ = Checkbox::new(&mut checked).id_salt("click").show(ui);
        });
        run_pass(&ctx, click_event(center, false), |ui| {
            let _ = Checkbox::new(&mut checked).id_salt("click").show(ui);
        });
        assert!(checked, "second click must toggle back");
        assert!(ctx.memory(|m| m.has_focus(id)), "focus retained");

        // Press-drag-off-release: no toggle, no focus. Fresh context so
        // no stale focus state leaks in. The move past `max_click_dist`
        // is what voids the click (egui `PointerState`).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let mut rect = egui::Rect::NOTHING;
        let mut id = egui::Id::NULL;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let r = Checkbox::new(&mut checked).id_salt("dragoff").show(ui);
            rect = r.rect;
            id = r.id;
        });
        let center = rect.center();
        let away = egui::pos2(100.0, 100.0);
        run_pass(&ctx, click_event(center, true), |ui| {
            let _ = Checkbox::new(&mut checked).id_salt("dragoff").show(ui);
        });
        run_pass(
            &ctx,
            egui::RawInput {
                events: vec![
                    egui::Event::PointerMoved(away),
                    egui::Event::PointerButton {
                        pos: away,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::default(),
                    },
                ],
                ..Default::default()
            },
            |ui| {
                let _ = Checkbox::new(&mut checked).id_salt("dragoff").show(ui);
            },
        );
        assert!(!checked, "drag-off release must not toggle");
        assert!(
            !ctx.memory(|m| m.has_focus(id)),
            "drag-off release must not focus"
        );

        // A disabled box takes neither toggle nor focus when clicked:
        // fresh context so no stale focus state leaks in.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let mut rect = egui::Rect::NOTHING;
        let mut id = egui::Id::NULL;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let r = Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("click_dis")
                .show(ui);
            rect = r.rect;
            id = r.id;
        });
        let center = rect.center();
        run_pass(&ctx, click_event(center, true), |ui| {
            let _ = Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("click_dis")
                .show(ui);
        });
        let mut clicked = false;
        run_pass(&ctx, click_event(center, false), |ui| {
            let r = Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("click_dis")
                .show(ui);
            clicked = r.clicked();
        });
        assert!(!clicked, "disabled box must not report a click");
        assert!(!checked, "disabled box must not toggle on click");
        assert!(
            !ctx.memory(|m| m.has_focus(id)),
            "disabled box must not take focus on click"
        );
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

    // ------------------------------------------------------------------
    // Render-level tests: same-context passes, compared by paint shapes.
    // ------------------------------------------------------------------

    /// Runs one pass showing a checkbox UI and returns the output. The
    /// `Ui` id is fixed so widget ids (and animator state) are stable
    /// across passes in one context.
    fn run_pass(
        ctx: &egui::Context,
        input: egui::RawInput,
        mut show: impl FnMut(&mut egui::Ui),
    ) -> egui::FullOutput {
        let mut out = ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("checkbox_render_test"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(200.0, 200.0),
                )),
            );
            show(ui);
        });
        // The shadow paint allocates textures; dropping an unapplied
        // `TexturesDelta` panics (debug assert). These tests compare
        // shapes only, so the deltas are intentionally discarded.
        out.textures_delta.clear();
        out
    }

    fn pointer_input(pos: egui::Pos2) -> egui::RawInput {
        egui::RawInput {
            events: vec![egui::Event::PointerMoved(pos)],
            ..Default::default()
        }
    }

    /// `Shape::Rect` entries in paint order.
    fn rect_shapes(out: &egui::FullOutput) -> Vec<&egui::epaint::RectShape> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) => Some(rs),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn hovered_renders_identically_to_resting() {
        // Spec §4.9: no `hover:*` class in either recipe — the checkbox
        // renders IDENTICALLY hovered and unhovered. The paint path reads
        // no hover state; this test pins that by comparing every paint
        // shape across two passes in one context (same texture cache, so
        // image shapes compare equal too).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let away = run_pass(&ctx, pointer_input(egui::pos2(100.0, 100.0)), |ui| {
            Checkbox::new(&mut checked).id_salt("hover").show(ui);
        });
        let hovered = run_pass(&ctx, pointer_input(egui::pos2(8.0, 8.0)), |ui| {
            Checkbox::new(&mut checked).id_salt("hover").show(ui);
        });
        assert_eq!(
            format!("{:?}", away.shapes),
            format!("{:?}", hovered.shapes),
            "hover changed the paint output"
        );
    }

    #[test]
    fn indeterminate_adds_only_the_icon_shape() {
        // Spec §4.4 + settled OPEN-2 (CHK-VIS-02): the indeterminate box
        // renders the unchecked recipe PLUS the `Check` icon — so the
        // mixed render must contain every unchecked shape in order, plus
        // exactly the icon's image shape. The icon colour is the
        // resolver's unchecked `foreground` value (pinned by the
        // resolver tests); the capture evidence judges the rendered
        // colour.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let plain = run_pass(&ctx, egui::RawInput::default(), |ui| {
            Checkbox::new(&mut checked).id_salt("ind").show(ui);
        });
        let mixed = run_pass(&ctx, egui::RawInput::default(), |ui| {
            Checkbox::new(&mut checked)
                .indeterminate(true)
                .id_salt("ind")
                .show(ui);
        });
        let plain_dbg: Vec<String> = plain
            .shapes
            .iter()
            .map(|s| format!("{:?}", s.shape))
            .collect();
        let mixed_dbg: Vec<String> = mixed
            .shapes
            .iter()
            .map(|s| format!("{:?}", s.shape))
            .collect();
        // Subsequence alignment: every unchecked shape appears in the
        // mixed render, in order; anything else is an added shape.
        let mut plain_idx = 0;
        let mut added: Vec<&String> = Vec::new();
        for shape in &mixed_dbg {
            if plain_idx < plain_dbg.len() && shape == &plain_dbg[plain_idx] {
                plain_idx += 1;
            } else {
                added.push(shape);
            }
        }
        assert_eq!(
            plain_idx,
            plain_dbg.len(),
            "indeterminate changed an unchecked shape"
        );
        assert_eq!(added.len(), 1, "indeterminate added shapes: {added:?}");
        assert!(
            added[0].starts_with("Mesh"),
            "added shape is not the icon mesh: {}",
            added[0]
        );
    }

    #[test]
    fn disabled_group_fade_matches_css_opacity() {
        // CHK-VIS-01: the disabled element composites as a GROUP at 50%
        // over the backdrop (spec §4.3, CSS `opacity` semantics). The
        // fill halves (it paints over the backdrop alone); the stroke
        // and icon pre-composite OPAQUE: 0.5 * (slot over fill over
        // backdrop) + 0.5 * backdrop. Hand-computed literals below
        // (premultiplied `mix` rounds halves away from zero, so
        // 252.5 -> 253 and 16.5 -> 17 — within the ±1 differential
        // tolerance the captures are judged by).
        let white = Color32::WHITE;
        let dark_bg = Color32::from_rgb(10, 10, 10);
        // Light checked: fill/stroke primary (23), icon
        // primary-foreground (250) over white.
        let (fill, stroke, icon) = disabled_group_fade(
            Color32::from_rgb(23, 23, 23),
            Color32::from_rgb(23, 23, 23),
            Color32::from_rgb(250, 250, 250),
            white,
        );
        assert_eq!(fill, color_alpha(Color32::from_rgb(23, 23, 23), 0.5));
        assert_eq!(stroke, Color32::from_rgb(139, 139, 139));
        assert_eq!(icon, Color32::from_rgb(253, 253, 253));
        // Dark checked: fill/stroke primary (229), icon
        // primary-foreground (23) over (10).
        let (fill, stroke, icon) = disabled_group_fade(
            Color32::from_rgb(229, 229, 229),
            Color32::from_rgb(229, 229, 229),
            Color32::from_rgb(23, 23, 23),
            dark_bg,
        );
        assert_eq!(fill, color_alpha(Color32::from_rgb(229, 229, 229), 0.5));
        assert_eq!(stroke, Color32::from_rgb(120, 120, 120));
        assert_eq!(icon, Color32::from_rgb(17, 17, 17));
        // Light unchecked: transparent fill, opaque `input` (229) stroke.
        let (fill, stroke, _) = disabled_group_fade(
            Color32::TRANSPARENT,
            Color32::from_rgb(229, 229, 229),
            Color32::from_rgb(10, 10, 10),
            white,
        );
        assert_eq!(fill, Color32::TRANSPARENT);
        assert_eq!(stroke, Color32::from_rgb(242, 242, 242));
        // Dark unchecked: `input/30` fill (white 15% at 30% ->
        // premultiplied (11,11,11,11)) under a white-15% `input`
        // stroke (premultiplied (38,38,38,38)) over (10): the enabled
        // border composite is (56), so the group fade is (33).
        let (fill, stroke, _) = disabled_group_fade(
            Color32::from_rgba_premultiplied(11, 11, 11, 11),
            Color32::from_rgba_premultiplied(38, 38, 38, 38),
            Color32::from_rgb(250, 250, 250),
            dark_bg,
        );
        assert_eq!(
            fill,
            color_alpha(Color32::from_rgba_premultiplied(11, 11, 11, 11), 0.5)
        );
        assert_eq!(stroke, Color32::from_rgb(33, 33, 33));
    }

    #[test]
    fn disabled_paint_uses_group_fade() {
        // Spec §4.3 (CHK-VIS-01): the RENDERED disabled stroke is the
        // opaque group-fade precomposite, not a halved alpha — while the
        // fill still halves (it paints over the backdrop alone). The
        // fade applies AFTER the animators, whose targets are identical
        // in the enabled/disabled passes, so single-pass comparisons
        // are stable.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let p = palette(Mode::Light);
        let mut checked = false;
        let enabled = run_pass(&ctx, egui::RawInput::default(), |ui| {
            Checkbox::new(&mut checked).id_salt("dis").show(ui);
        });
        let disabled = run_pass(&ctx, egui::RawInput::default(), |ui| {
            Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("dis")
                .show(ui);
        });
        let stroke_of = |out: &egui::FullOutput| {
            rect_shapes(out)
                .into_iter()
                .find(|rs| rs.stroke.width > 0.0)
                .expect("a stroked rect shape")
                .stroke
                .color
        };
        assert_eq!(stroke_of(&enabled), p.input);
        // Group fade of the (229) border over white: opaque (242).
        assert_eq!(stroke_of(&disabled), Color32::from_rgb(242, 242, 242));

        // Light checked, fresh context: the disabled border-over-fill
        // reads the opaque group fade (139), not the per-slot (81).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = true;
        let disabled = run_pass(&ctx, egui::RawInput::default(), |ui| {
            Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("dis_chk")
                .show(ui);
        });
        assert_eq!(stroke_of(&disabled), Color32::from_rgb(139, 139, 139));

        let ctx = egui::Context::default();
        Preset::default().build(Mode::Dark).install(&ctx);
        let dark = palette(Mode::Dark);
        let mut checked = false;
        let enabled = run_pass(&ctx, egui::RawInput::default(), |ui| {
            Checkbox::new(&mut checked).id_salt("dis").show(ui);
        });
        let disabled = run_pass(&ctx, egui::RawInput::default(), |ui| {
            Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("dis")
                .show(ui);
        });
        let fill_of = |out: &egui::FullOutput| {
            rect_shapes(out)
                .into_iter()
                .find(|rs| rs.stroke.width == 0.0)
                .expect("a filled rect shape")
                .fill
        };
        assert_eq!(fill_of(&enabled), color_alpha(dark.input, 0.3));
        assert_eq!(
            fill_of(&disabled),
            color_alpha(color_alpha(dark.input, 0.3), 0.5)
        );
        // Dark unchecked border: group fade of the (56) composite over
        // (10) is opaque (33).
        assert_eq!(stroke_of(&disabled), Color32::from_rgb(33, 33, 33));
    }

    #[test]
    fn label_forwarding_sequence_toggles_and_focuses() {
        // Spec §7.3, checkbox side: the label mechanism (label.rs
        // `consume_pending_click`) requests focus on the checkbox's
        // `Response::id` and pushes a synthetic Space pair; egui's
        // `interact` reports a fake click for the focused widget. The
        // checkbox must toggle AND hold focus. Keyboard semantics
        // themselves are OPEN-1; this pins the egui mechanism the
        // settled label rule relies on. NOTE: this test feeds the pair
        // directly, so it cannot see the show-order failure CHK-INT-04
        // (the pair dying when the label runs after the checkbox);
        // the ordering cover is
        // `label_click_in_control_first_order_toggles_and_focuses`,
        // which drives the real `Label` + `Checkbox` widgets.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let mut target: Option<egui::Id> = None;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let r = Checkbox::new(&mut checked).id_salt("fwd").show(ui);
            target = Some(r.id);
        });
        assert!(!checked);
        let target = target.expect("checkbox response id");
        ctx.memory_mut(|mem| mem.request_focus(target));
        let mut focused = false;
        let mut changed = false;
        run_pass(&ctx, space_press_input(), |ui| {
            let r = Checkbox::new(&mut checked).id_salt("fwd").show(ui);
            focused = r.has_focus();
            changed = r.changed();
        });
        assert!(checked, "forwarded activation did not toggle");
        assert!(changed, "forwarded activation did not mark changed");
        assert!(focused, "forwarded activation did not focus");
    }

    #[test]
    fn label_forwarding_sequence_does_not_fire_when_disabled() {
        // Spec §7.3.4: a label click on a disabled checkbox must not
        // toggle it and must not focus it.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let mut target: Option<egui::Id> = None;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let r = Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("fwd_dis")
                .show(ui);
            target = Some(r.id);
        });
        let target = target.expect("checkbox response id");
        ctx.memory_mut(|mem| mem.request_focus(target));
        let mut focused = false;
        let mut changed = false;
        run_pass(&ctx, space_press_input(), |ui| {
            let r = Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("fwd_dis")
                .show(ui);
            focused = r.has_focus();
            changed = r.changed();
        });
        assert!(!checked, "disabled checkbox toggled on forwarded input");
        assert!(!changed, "disabled checkbox marked changed");
        assert!(!focused, "disabled checkbox took focus");
    }

    #[test]
    fn label_click_in_control_first_order_toggles_and_focuses() {
        // CHK-INT-04: the demo §6 / shadcn layout shows the checkbox
        // BEFORE its bound label. The label's deferred forwarding
        // pushes the synthetic Space pair at the top of `Label::show`
        // — after the checkbox's `interact` already ran — so the pair
        // died unseen and only the focus half landed. The checkbox
        // consumes its own pending click at the top of `show`, before
        // `interact`, which makes the activation land in either show
        // order. Unlike the S1 synthetic tests above, this pin drives
        // the REAL `Label` and `Checkbox` widgets in control-first
        // order through real passes with real pointer events: it FAILS
        // on the pre-fix build (the ordering failure is invisible to a
        // directly-fed Space pair) and PASSES with the fix.
        use crate::components::label::Label;
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let click_event = |pos: egui::Pos2, pressed: bool| egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };
        let mut checked = false;
        let mut target = egui::Id::NULL;
        let mut label_rect = egui::Rect::NOTHING;

        // Frame 1: render the pair control-first, capture the label
        // rect and the checkbox id (same-pass captured-id binding, as
        // the demo does).
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let r = Checkbox::new(&mut checked).id_salt("lbl4").show(ui);
            target = r.id;
            label_rect = Label::new("Enable notifications")
                .for_id(r.id)
                .show(ui)
                .rect;
        });
        assert!(!checked);
        assert!(!ctx.memory(|m| m.has_focus(target)));
        let label_center = label_rect.center();

        // Press + release on the label: the click is recorded for the
        // next pass; nothing toggles on the click pass itself.
        let show_pair = |ctx: &egui::Context, input: egui::RawInput, checked: &mut bool| {
            run_pass(ctx, input, |ui| {
                let r = Checkbox::new(checked).id_salt("lbl4").show(ui);
                let _ = Label::new("Enable notifications").for_id(r.id).show(ui);
            });
        };
        show_pair(&ctx, click_event(label_center, true), &mut checked);
        assert!(!checked, "label press alone must not toggle");
        show_pair(&ctx, click_event(label_center, false), &mut checked);
        assert!(!checked, "label click must not toggle on the click pass");

        // Next pass: the forwarded activation lands — toggle AND focus.
        show_pair(&ctx, egui::RawInput::default(), &mut checked);
        assert!(checked, "forwarded label click must toggle");
        assert!(
            ctx.memory(|m| m.has_focus(target)),
            "forwarded label click must focus"
        );

        // A second label click toggles back, exactly once, with focus
        // retained.
        show_pair(&ctx, click_event(label_center, true), &mut checked);
        show_pair(&ctx, click_event(label_center, false), &mut checked);
        assert!(
            checked,
            "second label click must not toggle on the click pass"
        );
        show_pair(&ctx, egui::RawInput::default(), &mut checked);
        assert!(!checked, "second label click must toggle back");
        assert!(
            ctx.memory(|m| m.has_focus(target)),
            "focus retained after second label click"
        );
    }

    #[test]
    fn label_click_on_disabled_checkbox_in_control_first_order_does_nothing() {
        // CHK-INT-05: the demo §6 disabled pair shows the checkbox
        // BEFORE its bound label. A label click must neither toggle nor
        // focus the disabled checkbox (spec §7.3 point 4) on ANY pass —
        // the defect is a painted transient, so the pin reads focus
        // DURING the pass, after the label runs, exactly where the demo
        // readout reads it. Two legs, two mechanisms:
        //
        // 1. Click pass: the label's click handler used to request focus
        //    on the target immediately, before recording the deferred
        //    click. That request is unconditional w.r.t. the TARGET's
        //    disabled state (gated only on the label's own), so the
        //    readout painted "focused: true" for the click pass's
        //    frame — and under load one presented frame can linger for
        //    seconds. The deferred forwarding is now the sole focus
        //    granter; no focus is requested in the click pass.
        // 2. Forwarding pass: the label's deferred consume requests
        //    focus unconditionally too, and in control-first order that
        //    request lands AFTER the disabled checkbox's `interact`
        //    (the only thing that would clear it: `create_widget`
        //    calls `ignore_focus` for the non-focusable `Sense::hover()`
        //    widget), so the readout painted "true" for that frame as
        //    well. The disabled checkbox drops its own pending click at
        //    the top of `show`, before `interact`, so the label's later
        //    consume finds nothing.
        //
        // An end-of-pass read would NOT discriminate either leg: egui's
        // end-of-pass dead-man's-switch (`Focus::end_pass`: the
        // hover-sense widget's id is never in `used_ids`, and the focus
        // was gained on a previous pass) clears the focus before the
        // next pass on both builds. This pin drives the REAL `Label` +
        // disabled `Checkbox` in control-first order through real
        // passes with real pointer events, three click cycles. It FAILS
        // on the pre-fix build (both legs) and on the cycle-1 build
        // (click-pass leg only — the forwarding leg was fixed there),
        // and PASSES with the complete fix. The enabled positive
        // control proves the harness engages the forwarding path (a
        // missed click would make the disabled assertions below pass
        // silently).
        use crate::components::label::Label;
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let click_event = |pos: egui::Pos2, pressed: bool| egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        };

        // Positive control: the identical sequence against an ENABLED
        // pair MUST focus (during the forwarding pass) and toggle.
        {
            let mut checked = false;
            let mut target = egui::Id::NULL;
            let mut label_rect = egui::Rect::NOTHING;
            run_pass(&ctx, egui::RawInput::default(), |ui| {
                let r = Checkbox::new(&mut checked).id_salt("lbl5ctl").show(ui);
                target = r.id;
                label_rect = Label::new("Enabled control").for_id(r.id).show(ui).rect;
            });
            let center = label_rect.center();
            let show_pair = |ctx: &egui::Context, input: egui::RawInput, checked: &mut bool| {
                run_pass(ctx, input, |ui| {
                    let r = Checkbox::new(checked).id_salt("lbl5ctl").show(ui);
                    let _ = Label::new("Enabled control").for_id(r.id).show(ui);
                });
            };
            show_pair(&ctx, click_event(center, true), &mut checked);
            show_pair(&ctx, click_event(center, false), &mut checked);
            let mut focused_during_forward = false;
            run_pass(&ctx, egui::RawInput::default(), |ui| {
                let r = Checkbox::new(&mut checked).id_salt("lbl5ctl").show(ui);
                let _ = Label::new("Enabled control").for_id(r.id).show(ui);
                focused_during_forward = ui.ctx().memory(|m| m.has_focus(target));
            });
            assert!(checked, "control: enabled pair must toggle");
            assert!(
                focused_during_forward,
                "control: enabled pair must focus during the forwarding pass"
            );
        }

        // Disabled pair: three label-click cycles, both legs.
        let mut checked = false;
        let mut target = egui::Id::NULL;
        let mut label_rect = egui::Rect::NOTHING;
        run_pass(&ctx, egui::RawInput::default(), |ui| {
            let r = Checkbox::new(&mut checked)
                .disabled(true)
                .id_salt("lbl5")
                .show(ui);
            target = r.id;
            label_rect = Label::new("Disabled option").for_id(r.id).show(ui).rect;
        });
        assert!(!checked);
        assert!(!ctx.memory(|m| m.has_focus(target)));
        let label_center = label_rect.center();

        for cycle in 1..=3 {
            // Press pass.
            run_pass(&ctx, click_event(label_center, true), |ui| {
                let r = Checkbox::new(&mut checked)
                    .disabled(true)
                    .id_salt("lbl5")
                    .show(ui);
                let _ = Label::new("Disabled option").for_id(r.id).show(ui);
            });
            // Release (click) pass: capture clicked() and read focus
            // DURING the pass, after the label runs.
            let mut clicked = false;
            let mut focused_during_click = false;
            run_pass(&ctx, click_event(label_center, false), |ui| {
                let r = Checkbox::new(&mut checked)
                    .disabled(true)
                    .id_salt("lbl5")
                    .show(ui);
                let lr = Label::new("Disabled option").for_id(r.id).show(ui);
                clicked = lr.clicked();
                focused_during_click = ui.ctx().memory(|m| m.has_focus(target));
            });
            assert!(clicked, "cycle {cycle}: label click did not register");
            assert!(
                !focused_during_click,
                "cycle {cycle}: disabled checkbox took focus during the click pass"
            );
            assert!(
                !checked,
                "cycle {cycle}: disabled checkbox toggled on label click"
            );
            // Forwarding pass: read focus DURING the pass, after the
            // label runs — exactly where the demo readout reads it.
            let mut focused_during_pass = false;
            run_pass(&ctx, egui::RawInput::default(), |ui| {
                let r = Checkbox::new(&mut checked)
                    .disabled(true)
                    .id_salt("lbl5")
                    .show(ui);
                let _ = Label::new("Disabled option").for_id(r.id).show(ui);
                focused_during_pass = ui.ctx().memory(|m| m.has_focus(target));
            });
            assert!(
                !checked,
                "cycle {cycle}: disabled checkbox toggled on label click"
            );
            assert!(
                !focused_during_pass,
                "cycle {cycle}: disabled checkbox took focus during the forwarding pass"
            );
        }
    }

    /// The synthetic Space down+up pair the label forwarding mechanism
    /// pushes (label.rs `consume_pending_click`).
    fn space_press_input() -> egui::RawInput {
        egui::RawInput {
            events: vec![
                egui::Event::Key {
                    key: egui::Key::Space,
                    physical_key: Some(egui::Key::Space),
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::Key {
                    key: egui::Key::Space,
                    physical_key: Some(egui::Key::Space),
                    pressed: false,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        }
    }
}
