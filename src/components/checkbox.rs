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
//! Indeterminate (spec §4.4, OPEN-2, D3): the box renders the UNCHECKED
//! recipe. The icon is UNESTABLISHED — this build renders NO icon for the
//! indeterminate state (PROVISIONAL, pending the S6 Work B / S4
//! settlement of OPEN-2). The lucide minus glyph the previous build
//! rendered was a port invention and has been removed; rendering one
//! needs deviation approval (D3).
//!
//! Keyboard/click (spec §7, OPEN-1): ALL Radix interaction semantics are
//! unestablished (no Radix source in the snapshot). This build uses
//! egui-natural activation ([`Sense::click`] + [`Response::clicked`],
//! which covers pointer clicks and egui's focused-widget Space/Enter
//! activation) as IMPLEMENTATION, not spec truth. Label-click composition
//! (spec §7.3): the label side is settled — a label click fires a click
//! on the control AND focuses it. The checkbox needs no forwarding code
//! of its own: the label requests focus on the checkbox's returned
//! [`Response::id`] and pushes a synthetic Space pair, which `interact`
//! reports as a click on the focused widget. Bind labels with the
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
    /// checked. The unchecked value (`foreground`) is the leg start for
    /// the nova icon-colour transition; no icon renders unchecked.
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
/// indeterminate icon is UNESTABLISHED (spec §4.4, OPEN-2): this build
/// renders NO icon for the indeterminate state — PROVISIONAL, pending
/// the S6 Work B / S4 settlement. The minus glyph the previous build
/// rendered was a port invention (the original supplies only
/// `CheckIcon`); rendering one needs deviation approval (D3). When both
/// `checked` and `indeterminate` are set, the checked rendering wins
/// (spec E5; OPEN-3 proposal, pending coordinator ratification at S8).
fn indicator_icon(checked: bool, indeterminate: bool) -> Option<Icon> {
    // OPEN-2: no icon renders for the indeterminate state (provisional).
    // The flag is accepted so the E5 combination stays explicit at the
    // call site; it alters no rendered slot in this build.
    let _ = indeterminate;
    checked.then_some(Icon::Check)
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
    /// PROVISIONAL rendering (spec §4.4, OPEN-2): the box renders the
    /// unchecked recipe with NO icon. The minus glyph the previous build
    /// rendered was a port invention and has been removed; whether the
    /// Radix Indicator renders (and what glyph) in this state is
    /// unestablished until S6 Work B / S4 settle OPEN-2. When the bound
    /// boolean is `true`, the checked rendering wins (spec E5; OPEN-3
    /// proposal, pending coordinator ratification).
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
        let mut response = ui.interact(rect, id, sense);

        if self.disabled {
            // `disabled:cursor-not-allowed` (spec §2.1; D6: S6 records
            // the observed cursor behaviour).
            response = response.on_hover_cursor(CursorIcon::NotAllowed);
        }

        // egui-natural activation (OPEN-1 implementation, not spec
        // truth): pointer clicks and egui's focused-widget Space/Enter
        // activation both arrive here as `clicked()`, including the
        // synthetic Space pair a bound label forwards (spec §7.3).
        let clicked = response.clicked() && !self.disabled;
        if clicked {
            *self.checked = !*self.checked;
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
    fn indicator_icon_provisional_pending_open2() {
        // PROVISIONAL (spec §4.4, OPEN-2): checked renders `Check`; the
        // indeterminate state renders NO icon until S6 Work B / S4
        // settle OPEN-2. The E5 combination (checked + indeterminate)
        // renders the checked icon (checked wins; OPEN-3 proposal).
        assert_eq!(indicator_icon(true, false), Some(Icon::Check));
        assert_eq!(indicator_icon(false, false), None);
        assert_eq!(indicator_icon(false, true), None);
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
    fn indeterminate_unchecked_renders_like_unchecked() {
        // Spec §4.4: the indeterminate box renders the unchecked recipe,
        // and (provisional, OPEN-2) no icon — so the two renders must be
        // shape-identical.
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
        assert_eq!(
            format!("{:?}", plain.shapes),
            format!("{:?}", mixed.shapes),
            "indeterminate changed the unchecked render"
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
        // settled label rule relies on.
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
