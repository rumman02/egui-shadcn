//! shadcn `radio-group.tsx` — mutually exclusive option selector.
//!
//! Rebuilt to the frozen Batch B S0 spec (`~/workspace/batch-b/specs/
//! radio-group-spec.md`, Coordinator-signed 2026-11-16) plus two binding
//! coordinator rulings: the N2 / OPEN-5 Option C decision
//! (`~/workspace/batch-b/reports/radio-group-n2-coordinator-note.md`) and
//! the narrow `transition-[color,box-shadow]` reading
//! (`~/workspace/batch-b/reports/transition-color-narrow-reading-coordinator-note.md`).
//!
//! The original is `RadioGroup` + `RadioGroupItem` wrapping Radix
//! primitives (`radio-group.tsx`, 44 lines). The group recipe is
//! `grid gap-3` (`radio-group.tsx:15`); the item recipe is
//! `aspect-square size-4 shrink-0 rounded-full border border-input
//! text-primary shadow-xs transition-[color,box-shadow] outline-none
//! focus-visible:border-ring focus-visible:ring-[3px]
//! focus-visible:ring-ring/50 disabled:cursor-not-allowed
//! disabled:opacity-50 aria-invalid:border-destructive
//! aria-invalid:ring-destructive/20 dark:bg-input/30
//! dark:aria-invalid:ring-destructive/40` (`radio-group.tsx:29`); the
//! indicator is an 8px `fill-primary` dot (`:34-39`).
//!
//! Nova deltas (`.cn-radio-group` / `.cn-radio-group-item`,
//! `style-nova.css:968-975`): the group gaps at `gap-2` (8px); the checked
//! item is a solid primary disc (`data-checked:bg-primary
//! data-checked:border-primary`) whose dot is `bg-primary-foreground`
//! (`style-nova.css:980-986`); invalid items show a 3px rest ring
//! (`aria-invalid:ring-3`) and a `destructive/50` border in dark mode;
//! inside a focused field-label card the ring is suppressed and the
//! border forced (`group-has-[:focus-visible]/field-label:*` rules).
//! Under the settled LAYER ruling, everything nova is silent on —
//! `shadow-xs`, the transition, `outline-none`, the `disabled:*` rules —
//! keeps its base value.
//!
//! Labels (N2, spec §8.3 + coordinator ruling): the original renders NO
//! labels — callers compose a separate `Label`. The Rust [`RadioGroup`]
//! API promises visible labels (`.option("...")`), so the group renders
//! each option's label by composing the frozen [`Label`] component
//! (Option C, proposed deviation D5): an 8px circle–label gap, 16px
//! rows, label text per the frozen label spec, and click-forwarding per
//! the settled label-click rule (a label click fires a click on the
//! item AND focuses it; direct clicks fire exactly once). The 8px gap /
//! 16px row values are a documented coordinator invention — the original
//! states no inline-label rule (OPEN-10, moot for the group renderer).
//! An empty label string renders the circle only. The standalone
//! [`RadioGroupItem`] takes no label and is unaffected.
//!
//! Animation (spec §6 AS RULED): `transition-[color,box-shadow]` reads
//! narrowly — the CSS `color` property (text colour) and `box-shadow`
//! (the ring) animate; border-colour and fill changes SNAP. The ring
//! fades in/out and eases between colours on the interim 150ms
//! framework-default timing (OPEN-6); everything else is instant.
//! Opacity is in neither transition list, so the disabled fade is
//! instant.
//!
//! Keyboard/click (spec §7, settled by S6 Work B from Radix source):
//! roving tabindex — one Tab stop per group (zero when all items are
//! disabled), Tab entry focuses the checked item (else the first
//! focusable item) and does NOT check it, and selection follows focus
//! ONLY for arrow-key moves. All four arrows navigate (Left/Up =
//! previous, Right/Down = next), wrapping at the edges (`loop = true`
//! by default; no orientation is set, so the filter never engages)
//! and skipping disabled items, uniformly for vertical and horizontal
//! groups (gate G6 amendment). Home/PageUp move focus to the first
//! focusable item and End/PageDown to the last, skipping disabled
//! items, WITHOUT changing the selection (focus-only; settled OPEN-2),
//! and the stored tab stop follows the move. Space activates the focused item on
//! keyup, once per press, and Enter never activates it (RAD-INT-04/05, settled
//! OPEN-3); clicks select and
//! never uncheck, but do not focus yet (RAD-INT-06). Focus-visible
//! modality is proposed deviation D1 (the ring shows on any focus;
//! egui cannot distinguish keyboard from pointer focus).
//!
//! Focus+invalid border precedence is deliberately NOT frozen by the
//! spec (§4 precedence note): this build keeps invalid-wins as
//! explicitly PROVISIONAL; S4/S5 own the cell.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut plan = Some(0);
//! RadioGroup::new(&mut plan)
//!     .option("Default")
//!     .option("Comfortable")
//!     .option_disabled("Compact")
//!     .id_salt("plan")
//!     .show(ui);
//! ```

use crate::components::attachment::mix;
use crate::components::button::GroupOrientation;
use crate::components::label::{label_font, Label};
use crate::theme::tokens::type_scale;
use crate::theme::{blend, color_alpha, Palette, ShadcnTheme, Style};
use crate::utils::animation::{transition_alpha, transition_color_to, TransitionSpec};
use crate::utils::focus::paint_ring;
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Align, Color32, Context, CornerRadius, Id, Layout, Painter, Pos2, Rect, Response, Sense,
    Stroke, Ui, UiBuilder, Vec2, Widget,
};

/// `size-4` — the radio item side length.
pub const RADIO_SIZE: f32 = 16.0;
/// `size-2` — the selected indicator dot diameter.
pub const RADIO_DOT_SIZE: f32 = 8.0;
/// Circle–label gap inside a group row: the frozen D5 invented layout
/// value (coordinator ruling — the original states no inline-label rule).
pub const RADIO_LABEL_GAP: f32 = 8.0;

/// `focus-visible:ring-[3px]` / `aria-invalid:ring-3` — the ring band
/// width (spec §3.5).
const RING_WIDTH: f32 = 3.0;
/// Circle radius: `rounded-full` is half the 16px box (spec §3.1). The
/// theme radius scale does not apply to this component.
const RADIO_RADIUS: f32 = RADIO_SIZE / 2.0;
/// Label line box: `text-sm` 14px `leading-none` (frozen label spec §3.1).
/// The 14px line centres against the 16px row (1px inset top/bottom).
const LABEL_LINE_H: f32 = type_scale::SM;

/// The gap between items in the group: base `gap-3` (12), nova `gap-2` (8).
fn gap(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0,
        _ => crate::theme::tokens::spacing::MD,
    }
}

/// The §6 transition timing: Tailwind v4 framework defaults (150ms,
/// `cubic-bezier(0.4, 0, 0.2, 1)`), INTERIM per OPEN-6 — an explicitly
/// unverified framework-default hypothesis, not snapshot truth. S7
/// judges timing only after OPEN-6 closes.
fn interim_spec() -> TransitionSpec {
    TransitionSpec::default()
}

/// One selectable option inside a [`RadioGroup`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioOption {
    /// The visible label, rendered beside the circle (8px gap, 16px row
    /// — the frozen D5 layout). An empty string renders the circle only.
    pub label: String,
    /// `disabled` — the option cannot be selected.
    pub disabled: bool,
}

impl RadioOption {
    /// Creates an enabled option with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
        }
    }

    /// Creates a disabled option with the given label.
    pub fn disabled(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: true,
        }
    }
}

/// The state inputs to [`resolve_radio_colors`] (one bundle keeps the
/// resolver signature small and the state-matrix tests readable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct RadioState {
    checked: bool,
    invalid: bool,
    focused: bool,
    disabled: bool,
    /// The D6 field-label suppression flag ([`RadioGroup::suppress_focus_indication`]).
    /// Only meaningful under nova; inert under the base recipe.
    suppressed: bool,
}

/// Resolved colors for one radio item state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct RadioColors {
    fill: Color32,
    stroke: Stroke,
    /// Focus/invalid ring color at full visibility (alpha already applied).
    ring: Color32,
    /// Whether the ring is visible per the §4 state table, before the §6
    /// visibility leg ([`animate_ring`]).
    ring_shown: bool,
    /// Indicator dot color (`Color32::TRANSPARENT` when unchecked).
    dot: Color32,
    /// `shadow-xs` paints in every state (spec §2.2 — no rule excludes
    /// any state, and nova is silent so it survives under LAYER).
    shadow_shown: bool,
}

/// Mirrors the base recipe: `border-input dark:bg-input/30`, indicator
/// `fill-primary`, `aria-invalid:border-destructive
/// aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40`.
///
/// Style deltas (nova recipe): `data-checked:bg-primary
/// data-checked:border-primary` with a `bg-primary-foreground` dot (solid
/// disc), `dark:aria-invalid:border-destructive/50`, and
/// `aria-invalid:aria-checked:border-primary` (a checked item keeps its
/// primary border even when invalid), plus `focus-visible:border-ring`
/// and the invalid rest ring (`aria-invalid:ring-3`).
///
/// Precedence (spec §4 note): the focus+invalid border cell is NOT
/// frozen — this resolver keeps invalid-wins as explicitly PROVISIONAL.
/// The frozen cells are: invalid+checked (nova, unfocused) keeps the
/// primary border; focus-visible (valid, any checked state) takes the
/// `ring` border; suppression (nova) forces the border and kills the
/// ring unconditionally.
pub(crate) fn resolve_radio_colors(
    p: &Palette,
    dark: bool,
    style: Style,
    state: RadioState,
) -> RadioColors {
    let nova = style == Style::Nova;
    let suppressed_nova = state.suppressed && nova;
    let fill = if nova && state.checked {
        p.primary
    } else if dark {
        color_alpha(p.input, 0.3)
    } else {
        Color32::TRANSPARENT
    };
    let stroke_color = if suppressed_nova {
        // D6: the field-label card's `group-has-*` rules force the border
        // (`ring-0` + forced border); suppression wins over invalid and
        // focus alike (spec §2.4 / §4 last row).
        if state.checked {
            p.primary
        } else {
            p.input
        }
    } else if state.invalid {
        // PROVISIONAL precedence (spec §4 note): invalid wins over focus.
        // Only the unfocused cells below are frozen; S4/S5 own the
        // focus+invalid cell.
        if nova && state.checked {
            // `aria-invalid:aria-checked:border-primary` (frozen unfocused).
            p.primary
        } else if nova && dark {
            // `dark:aria-invalid:border-destructive/50` (N5).
            color_alpha(p.destructive, 0.5)
        } else {
            // `aria-invalid:border-destructive`.
            p.destructive
        }
    } else if state.focused {
        // `focus-visible:border-ring` (valid, any checked state — §4 focus row).
        p.ring
    } else if nova && state.checked {
        // `data-checked:border-primary`.
        p.primary
    } else {
        p.input
    };
    let ring = if state.invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    // The ring shows on focus (a disabled item cannot take focus, so the
    // focus leg is gated on `!disabled`) or — nova only — at rest when
    // invalid (`aria-invalid:ring-3`). The invalid rest ring is a state
    // rule, not a focus rule, so it shows on a disabled invalid item too
    // (faded to 50% at paint, spec §4 disabled row: "Ring: as above").
    // Suppression kills the ring unconditionally under nova.
    let ring_shown =
        !suppressed_nova && ((state.focused && !state.disabled) || (state.invalid && nova));
    RadioColors {
        fill,
        stroke: Stroke::new(1.0, stroke_color),
        ring,
        ring_shown,
        dot: if state.checked {
            if nova {
                p.primary_foreground
            } else {
                p.primary
            }
        } else {
            Color32::TRANSPARENT
        },
        shadow_shown: true,
    }
}

/// The animated ring paint color for one item frame (§6 AS RULED: the
/// `box-shadow` leg animates; border/fill/dot snap and never pass
/// through this helper).
///
/// The ring's visibility fades (`transition_alpha`) and its colour eases
/// between the focus and invalid endpoints (`transition_color_to`), both
/// on the interim spec (OPEN-6). Callers invoke this EVERY frame for
/// every item — even when the ring is hidden and even when the item
/// rect is off-screen — so the animator state stays fresh: a first call
/// for an id returns the target immediately (no fade-in on mount), and a
/// leg only runs when `ring_shown` flips after the id has been driven.
/// Skipping the call while hidden would make the ring pop in at full
/// strength on focus.
///
/// Returns the fully transparent color when the ring is settled-hidden;
/// [`paint_radio`] skips the ring then.
fn animate_ring(ctx: &Context, id: Id, colors: &RadioColors) -> Color32 {
    let spec = interim_spec();
    let visibility = transition_alpha(
        ctx,
        id.with("ring_visibility"),
        0.0,
        1.0,
        colors.ring_shown,
        spec,
    );
    let colour = transition_color_to(ctx, id.with("ring_colour"), colors.ring, spec);
    color_alpha(colour.color, visibility.alpha)
}

/// Paints the item's `shadow-xs` through the LAYER resolution API
/// (`ShadowSlot::RadioItem` is silent in all 8 style files, so the base
/// value survives under every style). When disabled, the shadow fades
/// with the element's target opacity: the layer alphas scale on the
/// stack (same shape as the button's `paint_outline_shadow`).
fn paint_radio_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::RadioItem).layers();
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
            ShadowSlot::RadioItem,
            dark,
        );
    }
}

/// Whole-element group fade for one interior slot under
/// `disabled:opacity-50` (spec §4.3; RAD-VIS-01): the slot paints OPAQUE
/// with `0.5 * (slot over fill over backdrop) + 0.5 * backdrop`, where
/// `fill` is the resolved (un-faded) fill and `backdrop` is
/// `p.background`. Follows the CHK-VIS-01 (`disabled_group_fade` in
/// `checkbox.rs`) / INP-VIS-01 shape.
///
/// Per-slot alpha halving double-fades the overlap pixels: a halved dot
/// over a halved fill read (194) nova-light where group fade requires
/// 252, and a halved border over a halved fill read (88) where group
/// fade requires 139 (RAD-VIS-01). The fill itself keeps halving — it
/// paints over the backdrop alone, where halving IS the group fade —
/// and the ring keeps halving for the same reason (its band sits
/// outside the rect, over the backdrop alone; S5 confirmed the disabled
/// rings already match group fade exactly). The precomposite is exact
/// at every coverage: coverage `c` over the faded fill yields the group
/// fade of the enabled pixel at that coverage. The backdrop
/// approximation (exact on the gallery page, where the panel fill is
/// `p.background`) is the same recorded limitation as the
/// checkbox/toggle/input precedents: on another backdrop the overlap
/// pixels fade toward the theme background instead of the local fill.
fn disabled_group_fade(slot: Color32, fill: Color32, backdrop: Color32) -> Color32 {
    mix(backdrop, blend(blend(backdrop, fill), slot), 0.5)
}

/// Paints one radio item on the given rect (shared by group and standalone).
///
/// `ring_color` is the animated ring color from [`animate_ring`]
/// (transparent when the ring is hidden); every other slot paints its
/// resolved color directly — border, fill, and dot SNAP per the narrow
/// §6 ruling and never animate.
///
/// `backdrop` is the theme page background (`p.background`): when
/// disabled, the border and dot pre-composite the whole-element group
/// fade against it (see [`disabled_group_fade`]).
#[allow(clippy::too_many_arguments)] // one paint call; params are the resolved slots + context.
pub(crate) fn paint_radio(
    ui: &Ui,
    rect: Rect,
    colors: &RadioColors,
    ring_color: Color32,
    disabled: bool,
    dark: bool,
    style: Style,
    backdrop: Color32,
) {
    if !ui.is_rect_visible(rect) {
        return;
    }
    let painter = ui.painter();
    // `rounded-full`: the corner radius is half the 16px box (8px).
    let corner = CornerRadius::same(8);

    if colors.shadow_shown {
        paint_radio_shadow(painter, rect, corner, style, dark, disabled);
    }

    // `disabled:opacity-50` (spec §4.3, CSS `opacity` semantics): the
    // WHOLE element composites at 50% over the backdrop. The fill fades
    // by halving its alpha (it paints over the backdrop alone, where
    // halving IS the group fade), while the border and dot — which paint
    // OVER the fill — pre-composite OPAQUE via `disabled_group_fade`:
    // halving each slot double-fades the overlap pixels (RAD-VIS-01).
    // The ring keeps halving: it paints over the backdrop alone.
    let mut fill = colors.fill;
    let mut stroke = colors.stroke;
    let mut dot = colors.dot;
    let mut ring = ring_color;
    if disabled {
        fill = color_alpha(fill, 0.5);
        stroke.color = disabled_group_fade(colors.stroke.color, colors.fill, backdrop);
        dot = disabled_group_fade(colors.dot, colors.fill, backdrop);
        ring = color_alpha(ring, 0.5);
    }

    painter.rect_filled(rect, corner, fill);
    // 1px border INSIDE the 16px box (§3.1, border-box): epaint circle
    // strokes render OUTSIDE the given radius (`PathStroke::outside` in
    // `tessellate_circle`), so the stroke radius is the outer radius
    // minus the stroke width (7px), not the box radius (8px).
    painter.circle_stroke(rect.center(), RADIO_RADIUS - stroke.width, stroke);

    // `size-2` indicator dot. Presence is instant — no animation class
    // governs the indicator's mount/unmount (spec §6).
    if colors.dot != Color32::TRANSPARENT {
        painter.circle_filled(rect.center(), RADIO_DOT_SIZE / 2.0, dot);
    }

    // 3px ring band starting at the outer edge of the border box, on the
    // UNEXPANDED rect (`paint_ring` strokes outside; expanding first
    // would leave a 1px gap between border and ring).
    if ring.a() > 0 {
        paint_ring(painter, rect, ring, RING_WIDTH, RADIO_RADIUS);
    }
}

/// Width of one option label in the frozen label typography: 14px Inter
/// Medium 500, no wrap (the D5 row never wraps — spec §8.3 ruling).
fn measure_label_width(ui: &Ui, text: &str) -> f32 {
    ui.painter()
        .layout_no_wrap(text.to_owned(), label_font(), Color32::PLACEHOLDER)
        .size()
        .x
}

/// The label's rect inside its row: 8px right of the circle (the frozen
/// D5 gap), on a 14px line box centred against the 16px row (1px inset
/// top and bottom).
fn label_rect_in_row(row_rect: Rect, label_w: f32) -> Rect {
    Rect::from_min_size(
        Pos2::new(
            row_rect.min.x + RADIO_SIZE + RADIO_LABEL_GAP,
            row_rect.min.y + (RADIO_SIZE - LABEL_LINE_H) / 2.0,
        ),
        Vec2::new(label_w, LABEL_LINE_H),
    )
}

/// A single radio circle bound to a boolean (`RadioGroupItem`).
///
/// Clicking selects it (radios never uncheck themselves). Space
/// activates the focused item on keyup, once per press, and Enter
/// never activates it (settled OPEN-3, RAD-INT-04/05). For a group
/// of mutually exclusive options prefer
/// [`RadioGroup`].
#[derive(Debug)]
pub struct RadioGroupItem<'a> {
    checked: &'a mut bool,
    disabled: bool,
    invalid: bool,
    suppress_focus_indication: bool,
    id_salt: Option<String>,
}

impl<'a> RadioGroupItem<'a> {
    /// Creates a standalone radio item bound to the given boolean.
    pub fn new(checked: &'a mut bool) -> Self {
        Self {
            checked,
            disabled: false,
            invalid: false,
            suppress_focus_indication: false,
            id_salt: None,
        }
    }

    /// Disables the item (`disabled:opacity-50 disabled:cursor-not-allowed`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the item aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Suppresses the item's own focus indication (ring + border recolor)
    /// when the item sits inside a FieldLabel choice card that shows the
    /// focus indication itself. This is the item-side mechanism for the
    /// nova rules `group-has-[:focus-visible]/field-label:ring-0
    /// group-has-[:focus-visible]/field-label:not-data-checked:border-input
    /// group-has-[:focus-visible]/field-label:data-checked:border-primary`
    /// (`style-nova.css:972-974`; spec §2.4, D6): the composing card sets
    /// this flag each frame from its own focus-inside state. The field
    /// line owns the composition contract. Nova-only; a no-op under the
    /// base recipe. Suppression wins over invalid and focus alike (the
    /// literal rule reading: `ring-0` + forced border).
    pub fn suppress_focus_indication(mut self, suppress: bool) -> Self {
        self.suppress_focus_indication = suppress;
        self
    }

    /// Sets a stable id salt so focus state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the item.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_radio_item"),
        };
        // Reserve space without an auto-id widget so egui's built-in
        // spatial arrow navigation only sees the explicit item ids.
        let (_, rect) = ui.allocate_space(Vec2::splat(RADIO_SIZE));
        // Space activates on keyup, once per press (RAD-INT-05,
        // settled OPEN-3): hide the keydown from `interact` and
        // re-create it on the release pass.
        space_keyup_fire(ui.ctx(), id);
        // Enter never activates (RAD-INT-04, settled OPEN-3): hide
        // Enter keydowns from `interact` while this item holds focus,
        // so egui's fake primary click cannot fire. Pure suppression
        // — no re-fire, in any phase.
        suppress_enter_activation(ui.ctx(), ui.ctx().memory(|m| m.has_focus(id)));
        let mut response = ui.interact(rect, id, sense);

        if response.clicked() && !self.disabled {
            *self.checked = true;
            response.mark_changed();
        }

        let state = RadioState {
            checked: *self.checked,
            invalid: self.invalid,
            focused: response.has_focus(),
            disabled: self.disabled,
            suppressed: self.suppress_focus_indication,
        };
        let colors = resolve_radio_colors(&p, theme.mode.is_dark(), theme.style, state);
        let ring = animate_ring(ui.ctx(), id, &colors);
        paint_radio(
            ui,
            rect,
            &colors,
            ring,
            self.disabled,
            theme.mode.is_dark(),
            theme.style,
            p.background,
        );

        response
    }
}

impl Widget for RadioGroupItem<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A shadcn radio group. See the [module docs](self) for an example.
///
/// Each option renders its label beside its circle (the frozen D5
/// layout: 8px gap, 16px rows, label per the frozen label spec); an
/// empty label string renders the circle only.
#[derive(Debug)]
pub struct RadioGroup<'a> {
    selected: &'a mut Option<usize>,
    options: Vec<RadioOption>,
    orientation: GroupOrientation,
    disabled: bool,
    invalid: bool,
    suppress_focus_indication: bool,
    id_salt: Option<String>,
}

impl<'a> RadioGroup<'a> {
    /// Creates a group bound to the selected index (`None` = no selection).
    /// An out-of-range index selects nothing (Rust-only edge case, spec
    /// §9 — no original analogue); the value itself is left untouched.
    pub fn new(selected: &'a mut Option<usize>) -> Self {
        Self {
            selected,
            options: Vec::new(),
            orientation: GroupOrientation::Vertical,
            disabled: false,
            invalid: false,
            suppress_focus_indication: false,
            id_salt: None,
        }
    }

    /// Adds an enabled option (`RadioGroupItem` with a label, rendered
    /// beside the circle per the frozen D5 layout).
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.options.push(RadioOption::new(label));
        self
    }

    /// Adds a disabled option.
    pub fn option_disabled(mut self, label: impl Into<String>) -> Self {
        self.options.push(RadioOption::disabled(label));
        self
    }

    /// Adds several options at once.
    pub fn options(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.options
            .extend(labels.into_iter().map(RadioOption::new));
        self
    }

    /// Sets the layout orientation (`grid` is vertical by default; the group
    /// can also lay items out in a row). Horizontal layout is a Rust
    /// extension (spec §8.5, D4); the gap rule (12px base / 8px nova)
    /// applies unchanged.
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disables the whole group.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks every item aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Suppresses every item's own focus indication (ring + border
    /// recolor) when the group sits inside a FieldLabel choice card that
    /// shows the focus indication itself. See
    /// [`RadioGroupItem::suppress_focus_indication`] for the mechanism;
    /// the field line owns the composition contract. Nova-only; a no-op
    /// under the base recipe.
    pub fn suppress_focus_indication(mut self, suppress: bool) -> Self {
        self.suppress_focus_indication = suppress;
        self
    }

    /// Sets a stable id salt (used to derive per-item focus ids for the
    /// roving-focus behavior).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group with its options.
    pub fn show(self, ui: &mut Ui) -> egui::InnerResponse<()> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();

        let group_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_radio_group"),
        };

        let RadioGroup {
            selected,
            options,
            orientation,
            disabled,
            invalid,
            suppress_focus_indication,
            id_salt: _,
        } = self;

        let style = ItemStyle {
            p: &p,
            dark,
            style: theme.style,
            disabled,
            invalid,
            suppressed: suppress_focus_indication,
        };
        let mut run_items = |ui: &mut Ui| {
            render_items(ui, group_id, selected, &options, &style);
        };
        match orientation {
            GroupOrientation::Vertical => ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = gap(theme.style);
                run_items(ui);
            }),
            // NOT `ui.horizontal`: it seeds a 20px `interact_size.y`
            // frame (`horizontal_with_main_wrap_dyn`), which inflates
            // the group's response rect to 36px tall. A plain
            // `scope_builder` with a left-to-right layout — the same
            // shape `ui.vertical` uses — keeps the group exactly 16px
            // tall.
            GroupOrientation::Horizontal => ui.scope_builder(
                UiBuilder::new().layout(Layout::left_to_right(Align::Min)),
                |ui| {
                    ui.spacing_mut().item_spacing.x = gap(theme.style);
                    run_items(ui);
                },
            ),
        }
    }
}

/// Shared per-group style/state for item rendering.
struct ItemStyle<'a> {
    p: &'a Palette,
    dark: bool,
    style: Style,
    disabled: bool,
    invalid: bool,
    suppressed: bool,
}

/// Defers Space activation to keyup, once per press (RAD-INT-05,
/// settled OPEN-3: the native-button rule).
///
/// egui's `interact` reports a fake primary click for a focused
/// click-sense widget whenever `key_pressed(Space)` is true
/// (`egui-0.37.0/src/context.rs`, `FAKE_PRIMARY_CLICKED`), and
/// `key_pressed` counts every `pressed: true` key event INCLUDING
/// repeats (`input_state/mod.rs`, `num_presses`) — so a held Space
/// fires on keydown and re-fires on every autorepeat. This helper,
/// called before the widget's `interact`, hides Space keydown events
/// from the live event list (arming the widget) and re-creates a
/// single keydown when the release arrives, so the widget's
/// `clicked()` fires on the release pass only. The armed flag is per
/// widget id (`id.with("space_armed")`) in temp data; a release with
/// no preceding keydown fires nothing, and the flag clears whenever
/// Space is no longer down (`key_down` reads the begin-pass
/// `keys_down` snapshot, so stripping events mid-pass leaves it
/// intact during a hold).
///
/// Verbatim reuse of the `space_keyup_fire` shape in `button.rs`,
/// `switch.rs`, `checkbox.rs`, and `toggle.rs` (SWI-INT-01,
/// CHK-INT-02, TOG-INT-01) — the fifth copy; the duplication is a
/// known wave-pack observation, kept identical so the behaviour
/// cannot drift between components. The label-forwarding synthetic
/// Space pair (`label.rs`, `consume_pending_click`) survives this
/// handling by design: the pair carries its own release, so the
/// helper re-creates the keydown in the same pass — and in the group
/// path the pair is pushed mid-pass, after this helper runs, so
/// label clicks are unaffected either way.
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

/// Suppresses Enter activation (RAD-INT-04, settled OPEN-3: WAI-ARIA
/// radio groups don't activate items on Enter — the Radix trigger's
/// `onKeyDown` calls `preventDefault()` for Enter).
///
/// egui's `interact` reports a fake primary click for a focused
/// click-sense widget whenever `key_pressed(Enter)` is true
/// (`egui-0.37.0/src/context.rs:1761`, `FAKE_PRIMARY_CLICKED`), and
/// `key_pressed` counts every `pressed: true` key event in the LIVE
/// event list (`input_state/mod.rs:767-785`, `num_presses`) — so
/// removing Enter keydowns from the list before the widget's
/// `interact` makes the fake click impossible. Unlike
/// `space_keyup_fire` there is no re-fire: Enter must NEVER
/// activate, in any phase (keydown, repeat, or keyup).
///
/// The strip is gated on the widget holding focus (`has_focus`):
/// the fake click requires focus, so stripping only while focused
/// removes exactly the activation path and never eats an Enter meant
/// for another widget later in the pass. Enter releases are left
/// untouched (releases never fake-click), `Key::NumpadEnter` is
/// untouched (`create_widget` checks `Key::Enter` only), and the
/// label-forwarding synthetic Space pair carries no Enter event and
/// is unaffected.
fn suppress_enter_activation(ctx: &egui::Context, has_focus: bool) {
    if !has_focus {
        return;
    }
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

fn render_items(
    ui: &mut Ui,
    group_id: egui::Id,
    selected: &mut Option<usize>,
    options: &[RadioOption],
    style: &ItemStyle<'_>,
) {
    // Space activates on keyup, once per press (RAD-INT-05, settled
    // OPEN-3), for every item in the group: one group-level call
    // hides the keydown from all of the items' `interact`s and
    // re-creates it on the release pass, when only the focused item
    // can see it. The label-forwarding synthetic Space pair is pushed
    // mid-pass, during the item loop below, so it is unaffected.
    space_keyup_fire(ui.ctx(), group_id);
    let ItemStyle {
        p,
        dark,
        style,
        disabled,
        invalid,
        suppressed,
    } = *style;
    // Roving tabindex (RAD-INT-01; settled OPEN-2/OPEN-4): exactly one
    // item per group is Tab-reachable. Entry focuses the first of
    // [checked item, current tab stop, first focusable item]; the
    // stored stop only matters while the group has no selection (once
    // an item is checked it is the stop). A disabled item is never the
    // stop; a group with no enabled item has no stop at all.
    let first_enabled = options.iter().position(|o| !disabled && !o.disabled);
    let stored_tab_stop: Option<usize> =
        ui.ctx().data_mut(|d| d.get_temp(group_id.with("tab_stop")));
    let tab_stop = match *selected {
        Some(i) if i < options.len() && !disabled && !options[i].disabled => Some(i),
        _ => stored_tab_stop
            .filter(|&i| i < options.len() && !disabled && !options[i].disabled)
            .or(first_enabled),
    };
    // Whether focus is inside this group at pass start. When it is, the
    // focused item is the group's sole Tab stop (Radix
    // `roving-focus-group.tsx:306`: every item `onFocus` updates the
    // current tab stop), so Tab leaves the group directly. When focus
    // is outside, the entry candidate above (checked → stored → first)
    // is the sole stop, and Tab entry lands on it. The jump keys
    // (RAD-INT-03) are the first mechanism to separate focus from the
    // checked item; without this distinction Tab from a jumped-to item
    // would visit the checked item first (two focusable items).
    let focus_in_group = ui
        .ctx()
        .memory(|m| m.focused())
        .is_some_and(|id| (0..options.len()).any(|i| group_id.with(i) == id));
    let mut responses: Vec<Response> = Vec::with_capacity(options.len());
    for (i, option) in options.iter().enumerate() {
        let item_disabled = disabled || option.disabled;
        let item_id = group_id.with(i);
        // Row layout (frozen D5): a 16px row; the label, when present,
        // sits 8px right of the circle on a centred 14px line box. The
        // text is measured first so the row allocates its exact size —
        // no measure pass, no centred-parent trap.
        let has_label = !option.label.is_empty();
        let label_w = if has_label {
            measure_label_width(ui, &option.label)
        } else {
            0.0
        };
        let row_width = RADIO_SIZE
            + if has_label {
                RADIO_LABEL_GAP + label_w
            } else {
                0.0
            };
        let (_, row_rect) = ui.allocate_space(Vec2::new(row_width, RADIO_SIZE));
        let circle_rect = Rect::from_min_size(row_rect.min, Vec2::splat(RADIO_SIZE));

        // The label runs BEFORE the circle interact (label-then-control
        // order): the Label's deferred click-forwarding pushes its
        // synthetic Space pair into this pass, and the circle's
        // `interact` below must run after it to see the fake click. The
        // label lives in a child Ui at its exact rect so the row keeps
        // its 16px height (a `ui.horizontal` row would allocate 36px).
        if has_label {
            let label_rect = label_rect_in_row(row_rect, label_w);
            let mut child = ui.new_child(UiBuilder::new().max_rect(label_rect).id_salt(item_id));
            Label::new(option.label.as_str())
                .for_id(item_id)
                .disabled(item_disabled)
                .show(&mut child);
        }

        // Roving tabindex: exactly one item per group is focusable, so
        // Tab visits one stop per group. While focus is inside the
        // group, the FOCUSED item is the sole stop (see
        // `focus_in_group` above); while focus is outside, the entry
        // candidate (`tab_stop`) is. The other enabled items stay
        // clickable (`Sense::CLICK`, no FOCUSABLE bit) but are invisible
        // to Tab and to egui's spatial arrow navigation; arrows are
        // handled component-level below. An item that already holds
        // focus (label-forwarded focus is requested earlier in this
        // same pass, before this interact) MUST stay focusable:
        // egui's `create_widget` calls `ignore_focus` on a
        // non-focusable widget and clears focus it holds — so the
        // focus-holder is focusable unconditionally, which also covers
        // the label-click pass where the snapshot above still says
        // focus is outside (a one-pass transient also present in the
        // INT-01 model; the next pass restores a single stop). Disabled
        // items are not focusable (`Sense::hover`).
        let holds_focus = ui.ctx().memory(|m| m.has_focus(item_id));
        let sense = if item_disabled {
            Sense::hover()
        } else if holds_focus || (!focus_in_group && tab_stop == Some(i)) {
            Sense::click()
        } else {
            Sense::CLICK
        };
        // Enter never activates (RAD-INT-04, settled OPEN-3): hide
        // Enter keydowns from `interact` while this item holds focus,
        // so egui's fake primary click cannot fire. Pure suppression
        // — no re-fire. `holds_focus` is read after the label runs,
        // so label-forwarded focus is covered too.
        suppress_enter_activation(ui.ctx(), holds_focus);
        let response = ui.interact(circle_rect, item_id, sense);
        let is_selected = *selected == Some(i);
        let state = RadioState {
            checked: is_selected,
            invalid,
            focused: response.has_focus(),
            disabled: item_disabled,
            suppressed,
        };
        let colors = resolve_radio_colors(p, dark, style, state);
        // The ring animator runs every frame, even when the ring is
        // hidden (see `animate_ring`).
        let ring = animate_ring(ui.ctx(), item_id, &colors);
        paint_radio(
            ui,
            circle_rect,
            &colors,
            ring,
            item_disabled,
            dark,
            style,
            p.background,
        );

        // Clicking selects (radios never uncheck).
        if response.clicked() && !item_disabled && *selected != Some(i) {
            *selected = Some(i);
        }
        responses.push(response);
    }

    // Roving tabindex, arrow leg (RAD-INT-01/02; settled OPEN-1/OPEN-4):
    // selection follows focus ONLY for arrow-key-driven moves. Tab
    // entry, click-focus, and programmatic focus never check — there
    // is deliberately no focus-gain follower anymore (the old
    // `last_focused` mechanism is gone). All four arrows navigate
    // (Left/Up = previous, Right/Down = next) with wrap-around
    // (`loop = true` by default; shadcn sets no `orientation`, so the
    // orientation filter never engages) and disabled items skipped,
    // uniformly for vertical and horizontal groups alike (gate G6
    // amendment — no orientation gating). Each unmodified keydown
    // steps to the next/previous enabled item, checks it, and moves
    // the tab stop with it. A group with no other enabled item
    // (single-item group, or all others disabled) is a no-op (spec
    // §9), but the key is still consumed: the group owns unmodified
    // arrows while focus is inside it, so egui's spatial move is
    // cancelled even when nothing moves (else `end_pass` would carry
    // focus out of the group). Modifier+arrow receives NO handling
    // (gate G5, HEAD Radix behaviour). Home/End and PageUp/PageDown
    // are handled by the jump leg below (RAD-INT-03, settled OPEN-2);
    // Space is handled by `space_keyup_fire` at the top of this
    // function (RAD-INT-05, settled OPEN-3: keyup, once per press);
    // Enter is suppressed per item before `interact` (RAD-INT-04,
    // settled OPEN-3: never activates); Escape is untouched
    // (RAD-INT-07 owns it).
    let focused_id = ui.ctx().memory(|m| m.focused());
    let focused_idx =
        focused_id.and_then(|id| (0..options.len()).find(|&i| group_id.with(i) == id));
    if let Some(current) = focused_idx {
        let steps: Vec<isize> = ui.input(|input| {
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } if !modifiers.any() => match key {
                        egui::Key::ArrowDown | egui::Key::ArrowRight => Some(1),
                        egui::Key::ArrowUp | egui::Key::ArrowLeft => Some(-1),
                        _ => None,
                    },
                    _ => None,
                })
                .collect()
        });
        let mut target = current;
        let mut moved = false;
        for &step in steps.iter() {
            // Step ±1, skipping disabled items, wrapping at the
            // edges. The search is bounded by the group length, so a
            // group with no other enabled item finds only the current
            // item and does not move.
            let n = options.len() as isize;
            let mut next = target as isize;
            let mut found = false;
            for _ in 0..options.len() {
                next = (next + step).rem_euclid(n);
                if !(disabled || options[next as usize].disabled) {
                    found = true;
                    break;
                }
            }
            if found && next as usize != target {
                target = next as usize;
                moved = true;
            }
        }
        if !steps.is_empty() {
            if moved {
                *selected = Some(target);
                responses[target].mark_changed();
                ui.ctx()
                    .data_mut(|d| d.insert_temp(group_id.with("tab_stop"), target));
            }
            ui.ctx().memory_mut(|m| {
                if moved {
                    m.request_focus(group_id.with(target));
                }
                // Cancel egui's spatial arrow move for this pass:
                // `begin_pass` already set the cardinal direction from
                // the key event, and `end_pass` would otherwise move
                // focus AGAIN, starting from the newly focused item —
                // or, when nothing moved, out of the group entirely.
                m.move_focus(egui::FocusDirection::None);
            });
        }
    }

    // Roving tabindex, jump leg (RAD-INT-03; settled OPEN-2):
    // Home/PageUp move focus to the FIRST focusable item, End/PageDown
    // to the LAST, skipping disabled items — and the move is
    // FOCUS-ONLY. These keys are not in Radix `ARROW_KEYS`, so no
    // synthetic click fires and selection is unchanged. This leg
    // deliberately never writes `*selected` and never calls
    // `mark_changed()`; there is no focus-gain follower, so focus
    // landing on an item checks nothing by itself. The stored tab stop
    // DOES follow the move (Radix `roving-focus-group.tsx:306`: every
    // item `onFocus` updates the current tab stop) — it only takes
    // effect while the group has no selection, since the checked item
    // wins the tab-stop computation at entry. Uniform for vertical and
    // horizontal groups alike (gate G6 amendment — no orientation
    // gating, same as the arrow leg). A jump key is consumed even when
    // nothing moves (already at the edge, or a single-item group), so
    // the spatial direction is cancelled whenever a jump keydown was
    // collected while focus is in the group. Modifier+jump receives NO
    // handling (G5 spirit). Focus is re-read here, after the arrow
    // leg, so a combined arrow+jump burst in one pass applies
    // sequentially (arrow first, then jump from the new position),
    // matching Radix's per-event semantics.
    let focused_id = ui.ctx().memory(|m| m.focused());
    let focused_idx =
        focused_id.and_then(|id| (0..options.len()).find(|&i| group_id.with(i) == id));
    if let Some(current) = focused_idx {
        let jumps: Vec<bool> = ui.input(|input| {
            input
                .events
                .iter()
                .filter_map(|event| match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } if !modifiers.any() => match key {
                        egui::Key::Home | egui::Key::PageUp => Some(false),
                        egui::Key::End | egui::Key::PageDown => Some(true),
                        _ => None,
                    },
                    _ => None,
                })
                .collect()
        });
        let last_enabled = options.iter().rposition(|o| !disabled && !o.disabled);
        let mut target = current;
        for &to_last in jumps.iter() {
            let dest = if to_last { last_enabled } else { first_enabled };
            if let Some(dest) = dest {
                target = dest;
            }
        }
        if !jumps.is_empty() {
            let moved = target != current;
            if moved {
                ui.ctx()
                    .data_mut(|d| d.insert_temp(group_id.with("tab_stop"), target));
            }
            ui.ctx().memory_mut(|m| {
                if moved {
                    m.request_focus(group_id.with(target));
                }
                // Cancel any spatial move for this pass, mirroring the
                // arrow leg: the group owns unmodified jump keys while
                // focus is inside it.
                m.move_focus(egui::FocusDirection::None);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Key, Pos2};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    fn state(
        checked: bool,
        invalid: bool,
        focused: bool,
        disabled: bool,
        suppressed: bool,
    ) -> RadioState {
        RadioState {
            checked,
            invalid,
            focused,
            disabled,
            suppressed,
        }
    }

    fn key_input(key: Key) -> egui::RawInput {
        key_input_mod(key, egui::Modifiers::default())
    }

    fn key_input_mod(key: Key, modifiers: egui::Modifiers) -> egui::RawInput {
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        });
        input
    }

    fn key_release_input(key: Key) -> egui::RawInput {
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: false,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        });
        input
    }

    fn key_repeat_input(key: Key) -> egui::RawInput {
        let mut input = egui::RawInput::default();
        input.events.push(egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: egui::Modifiers::default(),
        });
        input
    }

    /// Keydown + keyup in a single pass — the label-forwarding pair
    /// shape (`label.rs`, `consume_pending_click`).
    fn key_pair_input(key: Key) -> egui::RawInput {
        let mut input = egui::RawInput::default();
        for pressed in [true, false] {
            input.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
        }
        input
    }

    fn test_ctx() -> egui::Context {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        ctx
    }

    /// One headless pass over a fixed Ui (label S9 harness pattern). The
    /// Ui id and rect are fixed within a test so widget ids are stable
    /// across passes.
    fn run_ui(ctx: &egui::Context, input: egui::RawInput, mut f: impl FnMut(&mut Ui)) {
        let mut output = ctx.run_ui(input, |ctx| {
            let mut ui = Ui::new(
                ctx.clone(),
                Id::unique("radio_s1_test"),
                UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 800.0))),
            );
            f(&mut ui);
        });
        // No renderer consumes the texture uploads in a headless test;
        // dropping an unapplied delta panics, so clear it explicitly.
        output.textures_delta.clear();
    }

    fn click_event(pos: Pos2, pressed: bool) -> egui::RawInput {
        egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn checked_item_shows_primary_dot() {
        let p = palette(Mode::Light);
        // Base: transparent fill, `fill-primary` dot, `border-input`.
        let c = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(true, false, false, false, false),
        );
        assert_eq!(c.dot, p.primary);
        assert_eq!(c.stroke.color, p.input);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        let u = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(false, false, false, false, false),
        );
        assert_eq!(u.dot, Color32::TRANSPARENT);
        let dark_p = palette(Mode::Dark);
        let d = resolve_radio_colors(
            &dark_p,
            true,
            Style::Vega,
            state(false, false, false, false, false),
        );
        assert_eq!(d.fill, color_alpha(dark_p.input, 0.3));
        // Nova: solid primary disc with a `primary-foreground` dot.
        let n = resolve_radio_colors(
            &p,
            false,
            Style::Nova,
            state(true, false, false, false, false),
        );
        assert_eq!(n.fill, p.primary);
        assert_eq!(n.stroke.color, p.primary);
        assert_eq!(n.dot, p.primary_foreground);
        let dp = palette(Mode::Dark);
        let nd = resolve_radio_colors(
            &dp,
            true,
            Style::Nova,
            state(true, false, false, false, false),
        );
        assert_eq!(nd.fill, dp.primary);
        assert_eq!(nd.dot, dp.primary_foreground);
    }

    #[test]
    fn focused_border_uses_ring() {
        let p = palette(Mode::Light);
        let f = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(false, false, true, false, false),
        );
        assert_eq!(f.stroke.color, p.ring);
        let nf = resolve_radio_colors(
            &p,
            false,
            Style::Nova,
            state(false, false, true, false, false),
        );
        assert_eq!(nf.stroke.color, p.ring);
        // Focus-visible takes the `ring` border for ANY checked state
        // (§4 focus row); fill and dot stay per the checked row.
        let cf = resolve_radio_colors(
            &p,
            false,
            Style::Nova,
            state(true, false, true, false, false),
        );
        assert_eq!(cf.stroke.color, p.ring);
        assert_eq!(cf.fill, p.primary);
        assert_eq!(cf.dot, p.primary_foreground);
        let cf_base = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(true, false, true, false, false),
        );
        assert_eq!(cf_base.stroke.color, p.ring);
        assert_eq!(cf_base.dot, p.primary);
    }

    #[test]
    fn invalid_wins_over_focus_is_provisional_pending_spec_decision() {
        // Spec §4 precedence note: the focus+invalid border cell is NOT
        // frozen. This test pins the CURRENT resolution (invalid wins)
        // and its provisional status; S4/S5 own the cell, and if they
        // settle otherwise this test — and only this test — changes.
        let p = palette(Mode::Light);
        let fi = resolve_radio_colors(
            &p,
            false,
            Style::Nova,
            state(false, true, true, false, false),
        );
        assert_eq!(fi.stroke.color, p.destructive);
        let dark = palette(Mode::Dark);
        let dn = resolve_radio_colors(
            &dark,
            true,
            Style::Nova,
            state(false, true, true, false, false),
        );
        assert_eq!(dn.stroke.color, color_alpha(dark.destructive, 0.5));
    }

    #[test]
    fn nova_invalid_checked_keeps_primary() {
        let p = palette(Mode::Light);
        // Nova `aria-invalid:aria-checked:border-primary` (frozen unfocused cell).
        let ic = resolve_radio_colors(
            &p,
            false,
            Style::Nova,
            state(true, true, false, false, false),
        );
        assert_eq!(ic.stroke.color, p.primary);
        assert_eq!(ic.fill, p.primary);
        assert_eq!(ic.dot, p.primary_foreground);
        // Base fades nothing: a checked invalid item still gets the
        // destructive border (`aria-invalid:border-destructive`).
        let bc = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(true, true, false, false, false),
        );
        assert_eq!(bc.stroke.color, p.destructive);
        let dark = palette(Mode::Dark);
        let dn = resolve_radio_colors(
            &dark,
            true,
            Style::Nova,
            state(false, true, false, false, false),
        );
        // `dark:aria-invalid:border-destructive/50`
        assert_eq!(dn.stroke.color, color_alpha(dark.destructive, 0.5));
    }

    #[test]
    fn invalid_item_uses_destructive() {
        let p = palette(Mode::Light);
        let c = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(false, true, false, false, false),
        );
        assert_eq!(c.stroke.color, p.destructive);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
        let dark = palette(Mode::Dark);
        let c = resolve_radio_colors(
            &dark,
            true,
            Style::Vega,
            state(false, true, false, false, false),
        );
        assert_eq!(c.ring, color_alpha(dark.destructive, 0.4));
    }

    #[test]
    fn ring_visibility_matrix() {
        let p = palette(Mode::Light);
        let dark_p = palette(Mode::Dark);
        // Base invalid at rest: no ring (colour set, no width class).
        let base_rest = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(false, true, false, false, false),
        );
        assert!(!base_rest.ring_shown);
        // Nova invalid at rest: the 3px rest ring (`aria-invalid:ring-3`).
        let nova_rest = resolve_radio_colors(
            &p,
            false,
            Style::Nova,
            state(false, true, false, false, false),
        );
        assert!(nova_rest.ring_shown);
        assert_eq!(nova_rest.ring, color_alpha(p.destructive, 0.2));
        let nova_rest_dark = resolve_radio_colors(
            &dark_p,
            true,
            Style::Nova,
            state(false, true, false, false, false),
        );
        assert!(nova_rest_dark.ring_shown);
        assert_eq!(nova_rest_dark.ring, color_alpha(dark_p.destructive, 0.4));
        // Focus shows the ring in both styles (valid).
        for style in [Style::Vega, Style::Nova] {
            let f = resolve_radio_colors(&p, false, style, state(false, false, true, false, false));
            assert!(f.ring_shown);
            assert_eq!(f.ring, color_alpha(p.ring, 0.5));
        }
        // A disabled item cannot take focus: no focus ring.
        let df = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(false, false, true, true, false),
        );
        assert!(!df.ring_shown);
        // ...but the nova invalid rest ring is a state rule, so it shows
        // on a disabled invalid item too (faded to 50% at paint).
        let di = resolve_radio_colors(
            &p,
            false,
            Style::Nova,
            state(false, true, false, true, false),
        );
        assert!(di.ring_shown);
    }

    #[test]
    fn suppression_forces_border_and_kills_ring_nova_only() {
        let p = palette(Mode::Light);
        // Nova + suppression: border forced by checked state, ring gone —
        // even when focused and invalid.
        let s = resolve_radio_colors(
            &p,
            false,
            Style::Nova,
            state(false, true, true, false, true),
        );
        assert_eq!(s.stroke.color, p.input);
        assert!(!s.ring_shown);
        let sc = resolve_radio_colors(&p, false, Style::Nova, state(true, true, true, false, true));
        assert_eq!(sc.stroke.color, p.primary);
        assert_eq!(sc.fill, p.primary);
        assert!(!sc.ring_shown);
        // Base + suppression: inert — identical to unsuppressed.
        let b = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(false, true, true, false, true),
        );
        let b0 = resolve_radio_colors(
            &p,
            false,
            Style::Vega,
            state(false, true, true, false, false),
        );
        assert_eq!(b.stroke.color, b0.stroke.color);
        assert_eq!(b.ring_shown, b0.ring_shown);
        assert_eq!(b.fill, b0.fill);
    }

    #[test]
    fn shadow_shown_in_every_state() {
        // `shadow-xs` paints in all states (spec §2.2) under both styles.
        let light = palette(Mode::Light);
        let dark_p = palette(Mode::Dark);
        for style in [Style::Vega, Style::Nova] {
            for (dark, p) in [(false, &light), (true, &dark_p)] {
                for checked in [false, true] {
                    for invalid in [false, true] {
                        for focused in [false, true] {
                            for disabled in [false, true] {
                                for suppressed in [false, true] {
                                    let c = resolve_radio_colors(
                                        p,
                                        dark,
                                        style,
                                        state(checked, invalid, focused, disabled, suppressed),
                                    );
                                    assert!(
                                        c.shadow_shown,
                                        "shadow must show: {style:?} dark={dark} checked={checked} invalid={invalid} focused={focused} disabled={disabled} suppressed={suppressed}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn disabled_group_fade_rad_vis_01() {
        // RAD-VIS-01: a disabled item composites as a GROUP at 50% over
        // the backdrop (spec §4.3, CSS `opacity` semantics). The border
        // and dot — which paint OVER the fill — pre-composite OPAQUE:
        // `0.5 * (slot over fill over backdrop) + 0.5 * backdrop`.
        // Per-slot alpha halving double-faded the overlap pixels (dot
        // (194) / border (88) nova-light; group fade requires 252/139).
        // Hand-computed literals below (premultiplied `mix` rounds
        // halves away from zero, so 252.5 -> 253 and 16.5 -> 17 —
        // within the ±1 differential tolerance the captures are judged
        // by, as in the CHK-VIS-01 precedent). This pin supports the
        // fix; the differential capture re-run is the closing evidence.
        let white = Color32::WHITE;
        let dark_bg = Color32::from_rgb(10, 10, 10);
        // Light checked (nova): fill/stroke primary (23), dot
        // primary-foreground (250) over white.
        assert_eq!(
            disabled_group_fade(
                Color32::from_rgb(23, 23, 23),
                Color32::from_rgb(23, 23, 23),
                white
            ),
            Color32::from_rgb(139, 139, 139)
        );
        assert_eq!(
            disabled_group_fade(
                Color32::from_rgb(250, 250, 250),
                Color32::from_rgb(23, 23, 23),
                white
            ),
            Color32::from_rgb(253, 253, 253)
        );
        // Dark checked (nova): fill/stroke primary (229), dot
        // primary-foreground (23) over (10).
        assert_eq!(
            disabled_group_fade(
                Color32::from_rgb(229, 229, 229),
                Color32::from_rgb(229, 229, 229),
                dark_bg
            ),
            Color32::from_rgb(120, 120, 120)
        );
        assert_eq!(
            disabled_group_fade(
                Color32::from_rgb(23, 23, 23),
                Color32::from_rgb(229, 229, 229),
                dark_bg
            ),
            Color32::from_rgb(17, 17, 17)
        );
        // Dark checked (base shape): translucent `input/30` fill under
        // an opaque dot (229) — the fill composite feeds the slot, so
        // the dot still lands on the group fade of the opaque dot.
        let fill_30 = color_alpha(Color32::WHITE, 0.3);
        assert_eq!(
            disabled_group_fade(Color32::from_rgb(229, 229, 229), fill_30, dark_bg),
            Color32::from_rgb(120, 120, 120)
        );
        // Dark invalid+checked (base shape): opaque destructive stroke
        // over the translucent fill — full-coverage group fade. (The
        // S5 sampled pixel sat at AA coverage 0.95, so its differential
        // expectation (126, 53, 54) is the coverage-weighted value,
        // which this opaque precomposite reproduces exactly at that
        // coverage.)
        assert_eq!(
            disabled_group_fade(Color32::from_rgb(255, 100, 103), fill_30, dark_bg),
            Color32::from_rgb(133, 55, 57)
        );
        // Light unchecked: transparent fill — the stroke lands on the
        // group fade of the stroke over the backdrop alone (242), the
        // value S5 confirmed already matched.
        assert_eq!(
            disabled_group_fade(
                Color32::from_rgb(229, 229, 229),
                Color32::TRANSPARENT,
                white
            ),
            Color32::from_rgb(242, 242, 242)
        );
    }

    #[test]
    fn gap_matches_style() {
        // Base `gap-3` (12), nova `gap-2` (8).
        assert_eq!(gap(Style::Vega), 12.0);
        assert_eq!(gap(Style::Nova), 8.0);
    }

    #[test]
    fn label_row_geometry_gap_height_and_centring() {
        // The frozen D5 invented layout: 8px circle–label gap, 16px row,
        // 14px label line box centred (1px inset top and bottom).
        assert_eq!(RADIO_LABEL_GAP, 8.0);
        assert_eq!(RADIO_SIZE, 16.0);
        assert_eq!(LABEL_LINE_H, 14.0);
        let row = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(100.0, RADIO_SIZE));
        let lr = label_rect_in_row(row, 50.0);
        assert_eq!(lr.min.x, 10.0 + RADIO_SIZE + RADIO_LABEL_GAP);
        assert_eq!(lr.min.y, 21.0);
        assert_eq!(lr.size(), Vec2::new(50.0, LABEL_LINE_H));
    }

    #[test]
    fn option_labels_use_frozen_label_typography_and_colour() {
        // The D5 composition renders labels with the frozen label spec's
        // exact helpers: 14px Inter Medium 500, `foreground`, 50% when
        // disabled. The wiring (the Label widget is actually shown) is
        // proven by the rect and click tests; this pins the contract.
        assert_eq!(label_font(), crate::theme::fonts::medium(14.0));
        assert_eq!(label_font().size, 14.0);
        let p = palette(Mode::Light);
        assert_eq!(
            crate::components::label::text_color(&p, false),
            p.foreground
        );
        assert_eq!(
            crate::components::label::text_color(&p, true),
            color_alpha(p.foreground, 0.5)
        );
    }

    #[test]
    fn group_rect_matches_rows_and_gaps() {
        // Two labeled options, vertical, nova default: height is
        // 16 + 8 + 16 (rows + gap); width is the widest row
        // (16 + 8 + label width).
        let ctx = test_ctx();
        let mut selected = None;
        let mut rect = Rect::NOTHING;
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            rect = RadioGroup::new(&mut selected)
                .id_salt("geom")
                .option("Alpha")
                .option("Beta")
                .show(ui)
                .response
                .rect;
        });
        assert_eq!(rect.height(), 40.0);
        let mut widest = 0.0f32;
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            widest = measure_label_width(ui, "Alpha").max(measure_label_width(ui, "Beta"));
        });
        assert_eq!(rect.width(), RADIO_SIZE + RADIO_LABEL_GAP + widest);
    }

    #[test]
    fn empty_label_renders_circle_only() {
        // An empty label string: zero-width label, no gap — the row is
        // exactly the 16px circle.
        let ctx = test_ctx();
        let mut selected = None;
        let mut rect = Rect::NOTHING;
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            rect = RadioGroup::new(&mut selected)
                .id_salt("empty_label")
                .option("")
                .show(ui)
                .response
                .rect;
        });
        assert_eq!(rect.size(), Vec2::splat(RADIO_SIZE));
    }

    #[test]
    fn horizontal_group_gaps_match_style() {
        // Empty labels isolate the gap: two options in a row, nova
        // default → width 16 + 8 + 16, height 16.
        let ctx = test_ctx();
        let mut selected = None;
        let mut rect = Rect::NOTHING;
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            rect = RadioGroup::new(&mut selected)
                .id_salt("hgap")
                .orientation(GroupOrientation::Horizontal)
                .option("")
                .option("")
                .show(ui)
                .response
                .rect;
        });
        assert_eq!(rect.size(), Vec2::new(40.0, RADIO_SIZE));
    }

    #[test]
    fn out_of_range_selection_renders_no_dot() {
        // Rust-only edge case (spec §9): an out-of-range index selects
        // nothing — no dot anywhere, no panic, value untouched.
        let ctx = test_ctx();
        let mut selected = Some(99);
        let mut dots = 0usize;
        let mut output = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = Ui::new(
                ctx.clone(),
                Id::unique("radio_oor_test"),
                UiBuilder::new().max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            RadioGroup::new(&mut selected)
                .id_salt("oor")
                .option("A")
                .option("B")
                .show(&mut ui);
        });
        for s in &output.shapes {
            if let egui::Shape::Circle(cs) = &s.shape {
                if cs.stroke.is_empty() && cs.fill != Color32::TRANSPARENT {
                    dots += 1;
                }
            }
        }
        output.textures_delta.clear();
        assert_eq!(dots, 0, "no dot may render for an out-of-range index");
        assert_eq!(selected, Some(99));
    }

    #[test]
    fn label_click_selects_and_focuses_item() {
        // The settled label-click rule (§7.3): a label click fires a
        // click on the item AND focuses it — via the Label widget's
        // deferred forwarding, exactly as the label line verified it.
        let ctx = test_ctx();
        let selected = std::cell::RefCell::new(None);
        let group_rect = std::cell::Cell::new(Rect::NOTHING);
        let item_id = std::cell::Cell::new(Id::NULL);

        let render = |ui: &mut Ui| {
            item_id.set(ui.scope_id().with("radio_label_click").with(0usize));
            let mut guard = selected.borrow_mut();
            let resp = RadioGroup::new(&mut guard)
                .id_salt("radio_label_click")
                .option("Alpha")
                .show(ui);
            group_rect.set(resp.response.rect);
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| {
            render(ui);
        });
        assert!(!ctx.memory(|m| m.has_focus(item_id.get())));

        // Click the centre of row 0's label: x = 24 + half the label
        // width, y = 8 (the 14px line box centres on the 16px row).
        let rect = group_rect.get();
        let center = Pos2::new(
            rect.min.x
                + RADIO_SIZE
                + RADIO_LABEL_GAP
                + (rect.width() - RADIO_SIZE - RADIO_LABEL_GAP) / 2.0,
            rect.min.y + RADIO_SIZE / 2.0,
        );
        run_ui(&ctx, click_event(center, true), |ui| {
            render(ui);
        });
        run_ui(&ctx, click_event(center, false), |ui| {
            render(ui);
        });
        // Deferred pass: no click in flight, the forwarded focus +
        // synthetic Space reach the item.
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            render(ui);
        });
        assert!(
            ctx.memory(|m| m.has_focus(item_id.get())),
            "label click must focus the item"
        );
        assert_eq!(
            *selected.borrow(),
            Some(0),
            "label click must select the item"
        );
    }

    #[test]
    fn direct_click_selects_exactly_once() {
        // A direct click on a circle selects it; no stale forwarded
        // click bounces the selection back on later passes (the
        // label/circle rects are disjoint, so each click fires once).
        let ctx = test_ctx();
        let selected = std::cell::RefCell::new(Some(1));
        let group_rect = std::cell::Cell::new(Rect::NOTHING);

        let render = |ui: &mut Ui| {
            let mut guard = selected.borrow_mut();
            let resp = RadioGroup::new(&mut guard)
                .id_salt("radio_direct_click")
                .option("A")
                .option("Longer label")
                .show(ui);
            group_rect.set(resp.response.rect);
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| {
            render(ui);
        });
        // Click the centre of row 0's circle.
        let rect = group_rect.get();
        let center = rect.min + Vec2::new(RADIO_SIZE / 2.0, RADIO_SIZE / 2.0);
        run_ui(&ctx, click_event(center, true), |ui| {
            render(ui);
        });
        run_ui(&ctx, click_event(center, false), |ui| {
            render(ui);
        });
        assert_eq!(*selected.borrow(), Some(0));
        // Settle passes: the selection stays — no second activation.
        for _ in 0..3 {
            run_ui(&ctx, egui::RawInput::default(), |ui| {
                render(ui);
            });
        }
        assert_eq!(*selected.borrow(), Some(0));
    }

    #[test]
    fn disabled_label_and_circle_clicks_do_nothing() {
        // A disabled option: neither its label nor its circle responds
        // to clicks — no selection, no focus.
        let ctx = test_ctx();
        let selected = std::cell::RefCell::new(None);
        let group_rect = std::cell::Cell::new(Rect::NOTHING);
        let item_id = std::cell::Cell::new(Id::NULL);

        let render = |ui: &mut Ui| {
            item_id.set(ui.scope_id().with("radio_disabled_click").with(0usize));
            let mut guard = selected.borrow_mut();
            let resp = RadioGroup::new(&mut guard)
                .id_salt("radio_disabled_click")
                .option_disabled("Unavailable")
                .show(ui);
            group_rect.set(resp.response.rect);
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| {
            render(ui);
        });
        let rect = group_rect.get();
        let label_center = Pos2::new(
            rect.min.x
                + RADIO_SIZE
                + RADIO_LABEL_GAP
                + (rect.width() - RADIO_SIZE - RADIO_LABEL_GAP) / 2.0,
            rect.min.y + RADIO_SIZE / 2.0,
        );
        let circle_center = rect.min + Vec2::new(RADIO_SIZE / 2.0, RADIO_SIZE / 2.0);
        for center in [label_center, circle_center] {
            run_ui(&ctx, click_event(center, true), |ui| {
                render(ui);
            });
            run_ui(&ctx, click_event(center, false), |ui| {
                render(ui);
            });
            run_ui(&ctx, egui::RawInput::default(), |ui| {
                render(ui);
            });
        }
        assert_eq!(*selected.borrow(), None);
        assert!(!ctx.memory(|m| m.has_focus(item_id.get())));
    }

    #[test]
    fn arrows_move_selection_and_focus() {
        // Settled OPEN-1/OPEN-4 (RAD-INT-01): the group handles Up/Down
        // itself — each arrow keydown moves focus one item and checks
        // it in the same pass. Selection follows arrow moves only.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_arrows_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_arrows")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        // Focus the selected item (roving focus starts on the selection).
        let base = egui::Id::unique("radio_arrows_test").with("radio_arrows");
        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        let arrow_input = key_input(Key::ArrowDown);

        // ArrowDown: the group's own arrow handling moves focus down
        // one item and checks it in the same pass.
        run(&ctx, &mut selected, arrow_input.clone());
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(1));

        run(&ctx, &mut selected, arrow_input);
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(2));

        // ArrowUp moves back up.
        let up_input = key_input(Key::ArrowUp);
        run(&ctx, &mut selected, up_input);
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn arrows_skip_disabled_options() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);
        let base = egui::Id::unique("radio_skip_test").with("radio_skip");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_skip_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_skip")
                    .option("A")
                    .option_disabled("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        let arrow_input = key_input(Key::ArrowDown);

        // The disabled item B is not focusable, so navigation lands on C.
        run(&ctx, &mut selected, arrow_input);
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(2));
    }

    #[test]
    fn arrows_wrap_at_edges_and_left_right_navigate() {
        // Settled OPEN-1 (RAD-INT-02): loop=true by default and no
        // orientation is set, so ALL FOUR arrows navigate (Left/Up =
        // previous, Right/Down = next), wrapping at both edges, with
        // selection following every move. Fails pre-fix: no wrap and
        // Left/Right unhandled.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);
        let base = egui::Id::unique("radio_wrap_test").with("radio_wrap");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_wrap_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_wrap")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };
        let press = |ctx: &egui::Context, selected: &mut Option<usize>, key: Key| {
            run(ctx, selected, key_input(key));
            run(ctx, selected, egui::RawInput::default());
        };
        let check = |ctx: &egui::Context, selected: &Option<usize>, want: usize| {
            assert_eq!(*selected, Some(want));
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(want)));
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        // Up from the first item wraps to the last.
        press(&ctx, &mut selected, Key::ArrowUp);
        check(&ctx, &selected, 2);
        // Down from the last item wraps to the first.
        press(&ctx, &mut selected, Key::ArrowDown);
        check(&ctx, &selected, 0);
        // Left/Right mid-group.
        press(&ctx, &mut selected, Key::ArrowDown);
        check(&ctx, &selected, 1);
        press(&ctx, &mut selected, Key::ArrowLeft);
        check(&ctx, &selected, 0);
        press(&ctx, &mut selected, Key::ArrowRight);
        check(&ctx, &selected, 1);
        press(&ctx, &mut selected, Key::ArrowRight);
        check(&ctx, &selected, 2);
        // Left from the first wraps to the last; Right from the last
        // wraps to the first.
        press(&ctx, &mut selected, Key::ArrowLeft);
        check(&ctx, &selected, 1);
        press(&ctx, &mut selected, Key::ArrowLeft);
        check(&ctx, &selected, 0);
        press(&ctx, &mut selected, Key::ArrowLeft);
        check(&ctx, &selected, 2);
        press(&ctx, &mut selected, Key::ArrowRight);
        check(&ctx, &selected, 0);
    }

    #[test]
    fn arrows_skip_disabled_across_wrap_boundary() {
        // Settled OPEN-1 (RAD-INT-02): disabled items are skipped in
        // both directions AND across the wrap boundary — wrapping from
        // the first item lands on the last ENABLED item, not on a
        // disabled neighbour of the edge.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);
        let base = egui::Id::unique("radio_wrapskip_test").with("radio_wrapskip");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_wrapskip_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_wrapskip")
                    .option("A")
                    .option_disabled("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };
        let press = |ctx: &egui::Context, selected: &mut Option<usize>, key: Key| {
            run(ctx, selected, key_input(key));
            run(ctx, selected, egui::RawInput::default());
        };
        let check = |ctx: &egui::Context, selected: &Option<usize>, want: usize| {
            assert_eq!(*selected, Some(want));
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(want)));
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        // Up from the first wraps past the disabled middle item to C.
        press(&ctx, &mut selected, Key::ArrowUp);
        check(&ctx, &selected, 2);
        // Down from the last wraps back to A.
        press(&ctx, &mut selected, Key::ArrowDown);
        check(&ctx, &selected, 0);
        // Left/Right wrap with the same skip.
        press(&ctx, &mut selected, Key::ArrowLeft);
        check(&ctx, &selected, 2);
        press(&ctx, &mut selected, Key::ArrowRight);
        check(&ctx, &selected, 0);
        // Down from A skips B mid-group (kept behaviour).
        press(&ctx, &mut selected, Key::ArrowDown);
        check(&ctx, &selected, 2);
    }

    #[test]
    fn horizontal_group_arrows_wrap_uniformly() {
        // G6 amendment (RAD-INT-02): the roving handler applies
        // uniformly regardless of orientation — the horizontal group
        // gets the same wrap + all-four-arrows model as vertical
        // groups. No orientation gating.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);
        let base = egui::Id::unique("radio_hwrap_test").with("radio_hwrap");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_hwrap_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_hwrap")
                    .orientation(GroupOrientation::Horizontal)
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };
        let press = |ctx: &egui::Context, selected: &mut Option<usize>, key: Key| {
            run(ctx, selected, key_input(key));
            run(ctx, selected, egui::RawInput::default());
        };
        let check = |ctx: &egui::Context, selected: &Option<usize>, want: usize| {
            assert_eq!(*selected, Some(want));
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(want)));
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        press(&ctx, &mut selected, Key::ArrowUp);
        check(&ctx, &selected, 2);
        press(&ctx, &mut selected, Key::ArrowDown);
        check(&ctx, &selected, 0);
        press(&ctx, &mut selected, Key::ArrowLeft);
        check(&ctx, &selected, 2);
        press(&ctx, &mut selected, Key::ArrowRight);
        check(&ctx, &selected, 0);
        press(&ctx, &mut selected, Key::ArrowRight);
        check(&ctx, &selected, 1);
        press(&ctx, &mut selected, Key::ArrowLeft);
        check(&ctx, &selected, 0);
    }

    #[test]
    fn single_item_group_arrow_is_noop() {
        // Spec §9 edge case (RAD-INT-02): a single-item group has
        // nowhere to move — an arrow key leaves focus and selection
        // unchanged (the group consumes the key; egui's spatial nav
        // must not carry focus out of the group).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = None;
        let base = egui::Id::unique("radio_single_test").with("radio_single");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_single_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_single")
                    .option("A")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        for key in [
            Key::ArrowDown,
            Key::ArrowUp,
            Key::ArrowLeft,
            Key::ArrowRight,
        ] {
            run(&ctx, &mut selected, key_input(key));
            run(&ctx, &mut selected, egui::RawInput::default());
            assert_eq!(selected, None);
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(0usize)));
        }
    }

    #[test]
    fn jump_keys_move_focus_only_to_first_and_last() {
        // Settled OPEN-2 (RAD-INT-03): Home/PageUp move focus to the
        // FIRST focusable item, End/PageDown to the LAST, and the move
        // is focus-only — selection is UNCHANGED (these keys are not
        // in Radix ARROW_KEYS, so no synthetic click fires). Edge
        // presses are no-op moves that keep focus in the group.
        // Fails pre-fix: all four keys are complete no-ops.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(1);
        let base = egui::Id::unique("radio_jump_test").with("radio_jump");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_jump_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_jump")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };
        let press = |ctx: &egui::Context, selected: &mut Option<usize>, key: Key| {
            run(ctx, selected, key_input(key));
            run(ctx, selected, egui::RawInput::default());
        };
        let check = |ctx: &egui::Context, selected: &Option<usize>, want: usize| {
            assert_eq!(*selected, Some(1));
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(want)));
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(1usize)));

        // Home/PageUp from the middle land on the first item.
        press(&ctx, &mut selected, Key::Home);
        check(&ctx, &selected, 0);
        press(&ctx, &mut selected, Key::ArrowDown);
        assert_eq!(selected, Some(1));
        press(&ctx, &mut selected, Key::PageUp);
        check(&ctx, &selected, 0);
        // Home from the first item is a no-op move (focus stays).
        press(&ctx, &mut selected, Key::Home);
        check(&ctx, &selected, 0);
        // End/PageDown from the first land on the last item.
        press(&ctx, &mut selected, Key::End);
        check(&ctx, &selected, 2);
        press(&ctx, &mut selected, Key::ArrowUp);
        assert_eq!(selected, Some(1));
        press(&ctx, &mut selected, Key::PageDown);
        check(&ctx, &selected, 2);
        // End from the last item is a no-op move (focus stays).
        press(&ctx, &mut selected, Key::End);
        check(&ctx, &selected, 2);
    }

    #[test]
    fn jump_keys_skip_disabled_items_including_edges() {
        // Settled OPEN-2 (RAD-INT-03): jump targets are the first/last
        // ENABLED item — disabled items are skipped even at the edges,
        // where no demo group exercises the case (rendered probes cover
        // the disabled-middle group only). Focus-only throughout.
        // Fails pre-fix: the keys do nothing at all.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(1);
        let base = egui::Id::unique("radio_jumpskip_test").with("radio_jumpskip");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_jumpskip_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_jumpskip")
                    .option_disabled("A")
                    .option("B")
                    .option("C")
                    .option_disabled("D")
                    .show(ui);
            });
            out.textures_delta.clear();
        };
        let press = |ctx: &egui::Context, selected: &mut Option<usize>, key: Key| {
            run(ctx, selected, key_input(key));
            run(ctx, selected, egui::RawInput::default());
        };
        let check = |ctx: &egui::Context, selected: &Option<usize>, want: usize| {
            assert_eq!(*selected, Some(1));
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(want)));
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(1usize)));

        // End skips the disabled last item, landing on C.
        press(&ctx, &mut selected, Key::End);
        check(&ctx, &selected, 2);
        // Home skips the disabled first item, landing on B.
        press(&ctx, &mut selected, Key::Home);
        check(&ctx, &selected, 1);
        // PageUp/PageDown honour the same edges.
        press(&ctx, &mut selected, Key::PageDown);
        check(&ctx, &selected, 2);
        press(&ctx, &mut selected, Key::PageUp);
        check(&ctx, &selected, 1);
    }

    #[test]
    fn jump_keys_update_stored_tab_stop() {
        // Radix roving-focus (`roving-focus-group.tsx:306`): the item
        // `onFocus` handler calls `onItemFocus(id)`, so ANY focus
        // landing — including a jump-key move — updates the current
        // tab stop. The stored stop only takes effect while the group
        // has no selection (the checked item wins at entry); the
        // rendered probes assert the re-entry behaviour, this pin
        // asserts the stored write itself. Fails pre-fix: no move, no
        // write.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected: Option<usize> = None;
        let base = egui::Id::unique("radio_jumpstop_test").with("radio_jumpstop");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_jumpstop_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_jumpstop")
                    .option("A")
                    .option("B")
                    .show(ui);
            });
            out.textures_delta.clear();
        };
        let stored =
            |ctx: &egui::Context| ctx.data_mut(|d| d.get_temp::<usize>(base.with("tab_stop")));

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));
        assert_eq!(stored(&ctx), None);

        // End moves focus to B and stores it as the tab stop.
        run(&ctx, &mut selected, key_input(Key::End));
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, None);
        assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(1usize)));
        assert_eq!(stored(&ctx), Some(1));

        // Home moves focus back to A and stores that.
        run(&ctx, &mut selected, key_input(Key::Home));
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, None);
        assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(0usize)));
        assert_eq!(stored(&ctx), Some(0));
    }

    #[test]
    fn single_item_group_jump_keys_are_noop() {
        // Spec §9 edge case (RAD-INT-03): a single-item group has
        // nowhere to jump — Home/End/PageUp/PageDown leave focus and
        // selection unchanged (the group consumes the key; focus must
        // not leak out). Passes red too: the keys are no-ops pre-fix
        // for the wrong reason (unhandled). The pin guards the
        // consume-without-move behaviour against regressions.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = None;
        let base = egui::Id::unique("radio_single_jump_test").with("radio_single_jump");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_single_jump_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_single_jump")
                    .option("A")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        for key in [Key::Home, Key::End, Key::PageUp, Key::PageDown] {
            run(&ctx, &mut selected, key_input(key));
            run(&ctx, &mut selected, egui::RawInput::default());
            assert_eq!(selected, None);
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(0usize)));
        }
    }

    #[test]
    fn horizontal_group_jump_keys_uniform() {
        // G6 amendment (RAD-INT-03): the roving handler never reads
        // orientation, so the horizontal group gets the same
        // focus-only jump model as vertical groups. Fails pre-fix.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(1);
        let base = egui::Id::unique("radio_hjump_test").with("radio_hjump");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_hjump_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_hjump")
                    .orientation(GroupOrientation::Horizontal)
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };
        let press = |ctx: &egui::Context, selected: &mut Option<usize>, key: Key| {
            run(ctx, selected, key_input(key));
            run(ctx, selected, egui::RawInput::default());
        };
        let check = |ctx: &egui::Context, selected: &Option<usize>, want: usize| {
            assert_eq!(*selected, Some(1));
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(want)));
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(1usize)));

        press(&ctx, &mut selected, Key::Home);
        check(&ctx, &selected, 0);
        press(&ctx, &mut selected, Key::End);
        check(&ctx, &selected, 2);
        press(&ctx, &mut selected, Key::PageUp);
        check(&ctx, &selected, 0);
        press(&ctx, &mut selected, Key::PageDown);
        check(&ctx, &selected, 2);
    }

    #[test]
    fn modified_jump_keys_unhandled() {
        // G5 spirit (RAD-INT-03): modified jump keys receive NO
        // handling — no movement, no selection change, focus stays.
        // Passes red too (unhandled pre-fix); the pin guards the
        // no-handling rule against a future over-eager handler.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(1);
        let base = egui::Id::unique("radio_modjump_test").with("radio_modjump");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_modjump_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_modjump")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(1usize)));

        for (key, mods) in [
            (Key::Home, egui::Modifiers::SHIFT),
            (Key::End, egui::Modifiers::CTRL),
            (Key::PageUp, egui::Modifiers::ALT),
            (Key::PageDown, egui::Modifiers::COMMAND),
        ] {
            run(&ctx, &mut selected, key_input_mod(key, mods));
            run(&ctx, &mut selected, egui::RawInput::default());
            assert_eq!(selected, Some(1));
            assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(1usize)));
        }
    }

    #[test]
    fn tab_leaves_group_after_jump_and_reentry_lands_on_checked() {
        // RAD-INT-03, sense rule: the jump keys are the first mechanism
        // to separate focus from the checked item, and Tab must still
        // visit exactly one stop per group. While focus is inside the
        // group, the FOCUSED item is the sole Tab stop (Radix
        // `onItemFocus`: any focus landing updates the current tab
        // stop), so Tab leaves the group directly; re-entry lands on
        // the CHECKED item (Radix entry order [active, current, ...]).
        // Fails pre-sense-fix: Tab from the jumped-to item lands on the
        // checked item first (two focusable items in the group).
        let ctx = test_ctx();
        let sel_a = std::cell::RefCell::new(Some(1));
        let sel_b = std::cell::RefCell::new(None);
        let ids = std::cell::Cell::new([Id::NULL; 5]);

        let render = |ui: &mut Ui| {
            let base = ui.scope_id();
            ids.set([
                base.with("jump_tab_a").with(0usize),
                base.with("jump_tab_a").with(1usize),
                base.with("jump_tab_a").with(2usize),
                base.with("jump_tab_b").with(0usize),
                base.with("jump_tab_b").with(1usize),
            ]);
            RadioGroup::new(&mut sel_a.borrow_mut())
                .id_salt("jump_tab_a")
                .option("A0")
                .option("A1")
                .option("A2")
                .show(ui);
            RadioGroup::new(&mut sel_b.borrow_mut())
                .id_salt("jump_tab_b")
                .option("B0")
                .option("B1")
                .show(ui);
        };

        let focused = |ctx: &egui::Context| ctx.memory(|m| m.focused());

        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        let [a0, a1, _a2, b0, _b1] = ids.get();
        ctx.memory_mut(|mem| mem.request_focus(a1));

        // Home: focus-only move to A0; selection stays on A1.
        run_ui(&ctx, key_input(Key::Home), |ui| render(ui));
        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(a0));
        assert_eq!(*sel_a.borrow(), Some(1));

        // Tab leaves the group directly to B's stop (first enabled).
        run_ui(&ctx, key_input(Key::Tab), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(b0));

        // Shift+Tab back in: re-entry lands on the CHECKED item.
        let shift = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        run_ui(&ctx, key_input_mod(Key::Tab, shift), |ui| render(ui));
        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(a1));
        assert_eq!(*sel_a.borrow(), Some(1));
    }

    #[test]
    fn tab_walk_visits_one_stop_per_group_and_never_checks() {
        // Roving tabindex (RAD-INT-01; settled OPEN-2/OPEN-4): Tab
        // visits exactly one stop per group — the checked item, else
        // the first focusable item — and never changes a selection.
        // The all-disabled group takes no stop. Re-entry lands on the
        // checked item. Selection moves only on the arrow key.
        let ctx = test_ctx();
        let sel_a = std::cell::RefCell::new(None);
        let sel_b = std::cell::RefCell::new(Some(1));
        let sel_c = std::cell::RefCell::new(None);
        let ids = std::cell::Cell::new([Id::NULL; 5]);

        let render = |ui: &mut Ui| {
            let base = ui.scope_id();
            ids.set([
                base.with("walk_a").with(0usize),
                base.with("walk_a").with(1usize),
                base.with("walk_b").with(0usize),
                base.with("walk_b").with(1usize),
                base.with("walk_c").with(0usize),
            ]);
            RadioGroup::new(&mut sel_a.borrow_mut())
                .id_salt("walk_a")
                .option("A0")
                .option("A1")
                .show(ui);
            RadioGroup::new(&mut sel_b.borrow_mut())
                .id_salt("walk_b")
                .option("B0")
                .option("B1")
                .show(ui);
            RadioGroup::new(&mut sel_c.borrow_mut())
                .id_salt("walk_c")
                .option_disabled("C0")
                .option_disabled("C1")
                .show(ui);
        };

        let focused = |ctx: &egui::Context| ctx.memory(|m| m.focused());

        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        let [a0, a1, _b0, b1, c0] = ids.get();
        assert_eq!(focused(&ctx), None);

        // Tab: entry at the first focusable item of the no-selection
        // group — focus without checking.
        run_ui(&ctx, key_input(Key::Tab), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(a0));
        assert_eq!(*sel_a.borrow(), None, "Tab entry must not check");
        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(a0));
        assert_eq!(*sel_a.borrow(), None);

        // Tab: exit to the next group's stop — its CHECKED item.
        run_ui(&ctx, key_input(Key::Tab), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(b1));
        assert_eq!(*sel_b.borrow(), Some(1));

        // Tab: the all-disabled group takes no stop; the cycle wraps
        // (the wrap lands on the next pass via `give_to_next`).
        run_ui(&ctx, key_input(Key::Tab), |ui| render(ui));
        assert_eq!(focused(&ctx), None);
        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(a0));
        assert_ne!(focused(&ctx), Some(c0));
        assert_eq!(*sel_c.borrow(), None);

        // ArrowDown: selection follows the arrow move only.
        run_ui(&ctx, key_input(Key::ArrowDown), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(a1));
        assert_eq!(*sel_a.borrow(), Some(1));

        // Tab out, Shift+Tab back in: re-entry lands on the checked
        // item (Shift+Tab is frame-delayed: needs a settle pass).
        run_ui(&ctx, key_input(Key::Tab), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(b1));
        let shift = egui::Modifiers {
            shift: true,
            ..Default::default()
        };
        run_ui(&ctx, key_input_mod(Key::Tab, shift), |ui| render(ui));
        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert_eq!(
            focused(&ctx),
            Some(a1),
            "re-entry lands on the checked item"
        );
        assert_eq!(*sel_a.borrow(), Some(1));

        // Shift+Tab from the first stop wraps to the last stop.
        run_ui(&ctx, key_input_mod(Key::Tab, shift), |ui| render(ui));
        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert_eq!(focused(&ctx), Some(b1));
    }

    #[test]
    fn label_click_on_non_tab_stop_item_focuses_it() {
        // The label-click rule (§7.3) on a NON-stop item: the label's
        // deferred forwarding requests focus before the circle's
        // interact runs, so the item must render focusable that pass
        // (the holds-focus clause) or egui's `ignore_focus` clears the
        // focus. Without the clause this test fails: the selection
        // lands but the focus does not.
        let ctx = test_ctx();
        let selected = std::cell::RefCell::new(Some(0));
        let group_rect = std::cell::Cell::new(Rect::NOTHING);
        let item_id = std::cell::Cell::new(Id::NULL);

        let render = |ui: &mut Ui| {
            item_id.set(ui.scope_id().with("radio_label_nonstop").with(2usize));
            let mut guard = selected.borrow_mut();
            let resp = RadioGroup::new(&mut guard)
                .id_salt("radio_label_nonstop")
                .option("AAA")
                .option("BBB")
                .option("CCC")
                .show(ui);
            group_rect.set(resp.response.rect);
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert!(!ctx.memory(|m| m.has_focus(item_id.get())));

        // Label centre of row 2. Rows are 16px with the style gap, so
        // the pitch is (height − 16) / 2; the equal-width labels span
        // x ∈ [24, rect.width()).
        let rect = group_rect.get();
        let pitch = (rect.height() - RADIO_SIZE) / 2.0;
        let center = Pos2::new(
            rect.min.x
                + RADIO_SIZE
                + RADIO_LABEL_GAP
                + (rect.width() - RADIO_SIZE - RADIO_LABEL_GAP) / 2.0,
            rect.min.y + 2.0 * pitch + RADIO_SIZE / 2.0,
        );
        run_ui(&ctx, click_event(center, true), |ui| render(ui));
        run_ui(&ctx, click_event(center, false), |ui| render(ui));
        // Deferred pass: the forwarded focus + synthetic Space reach
        // the item.
        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert!(
            ctx.memory(|m| m.has_focus(item_id.get())),
            "label click must focus the non-stop item"
        );
        assert_eq!(
            *selected.borrow(),
            Some(2),
            "label click must select the item"
        );
    }

    #[test]
    fn modifier_arrows_do_not_move_focus_or_selection() {
        // Gate G5 (HEAD Radix behaviour): arrows with modifiers
        // receive NO handling — neither the group's arrow leg nor
        // egui's spatial navigation (unmodified arrows only) moves.
        let ctx = test_ctx();
        let mut selected = Some(0);
        let base = egui::Id::unique("radio_mod_test").with("radio_mod");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_mod_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_mod")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(0usize)));

        for modifiers in [
            egui::Modifiers {
                shift: true,
                ..Default::default()
            },
            egui::Modifiers {
                ctrl: true,
                command: true,
                ..Default::default()
            },
            egui::Modifiers {
                alt: true,
                ..Default::default()
            },
        ] {
            run(
                &ctx,
                &mut selected,
                key_input_mod(Key::ArrowDown, modifiers),
            );
            run(&ctx, &mut selected, egui::RawInput::default());
            assert_eq!(selected, Some(0));
            assert!(
                ctx.memory(|m| m.has_focus(base.with(0usize))),
                "modifier+arrow must leave focus on item 0"
            );
        }
    }

    #[test]
    fn builders_keep_fields() {
        let mut selected = None;
        let group = RadioGroup::new(&mut selected)
            .option("A")
            .option_disabled("B")
            .options(["C", "D"])
            .orientation(GroupOrientation::Horizontal)
            .disabled(true)
            .invalid(true)
            .suppress_focus_indication(true)
            .id_salt("g");
        assert_eq!(group.options.len(), 4);
        assert!(group.options[1].disabled);
        assert!(!group.options[2].disabled);
        assert_eq!(group.orientation, GroupOrientation::Horizontal);
        assert!(group.disabled);
        assert!(group.invalid);
        assert!(group.suppress_focus_indication);
        assert_eq!(group.id_salt.as_deref(), Some("g"));

        let mut checked = false;
        let item = RadioGroupItem::new(&mut checked)
            .disabled(true)
            .invalid(true)
            .suppress_focus_indication(true)
            .id_salt("item");
        assert!(item.disabled);
        assert!(item.invalid);
        assert!(item.suppress_focus_indication);
        assert_eq!(item.id_salt.as_deref(), Some("item"));
    }

    #[test]
    fn item_is_sixteen_pt() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut checked = false;
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("radio_size_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
            );
            rect = RadioGroupItem::new(&mut checked).show(ui).rect;
        });
        out.textures_delta.clear();
        assert_eq!(rect.size(), Vec2::splat(RADIO_SIZE));
    }

    // --- Controlled-time animation tests (badge pin-test pattern). ---
    //
    // One frame at `time`, returning the painted shapes. The texture
    // delta is cleared before the output is dropped (epaint panics on
    // dropped unapplied deltas).

    fn run_item_frame(
        ctx: &egui::Context,
        time: f64,
        checked: &mut bool,
        id_salt: &str,
    ) -> Vec<egui::Shape> {
        let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0));
        let input = egui::RawInput {
            time: Some(time),
            screen_rect: Some(screen),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("radio_anim_test"),
                egui::UiBuilder::new().max_rect(screen),
            );
            RadioGroupItem::new(&mut *checked).id_salt(id_salt).show(ui);
        });
        let shapes = out.shapes.iter().map(|s| s.shape.clone()).collect();
        out.textures_delta.clear();
        shapes
    }

    /// The fill rect: the `RectShape` with an empty stroke (the ring
    /// carries the stroke; the shadow paints meshes).
    fn fill_rect_color(shapes: &[egui::Shape]) -> Option<Color32> {
        shapes.iter().find_map(|s| match s {
            egui::Shape::Rect(rs) if rs.stroke.is_empty() => Some(rs.fill),
            _ => None,
        })
    }

    /// The ring: the `RectShape` carrying a stroke.
    fn ring_stroke(shapes: &[egui::Shape]) -> Option<Stroke> {
        shapes.iter().find_map(|s| match s {
            egui::Shape::Rect(rs) if !rs.stroke.is_empty() => Some(rs.stroke),
            _ => None,
        })
    }

    /// The indicator dot: the filled circle (the border circle carries
    /// the stroke instead).
    fn dot_fill(shapes: &[egui::Shape]) -> Option<Color32> {
        shapes.iter().find_map(|s| match s {
            egui::Shape::Circle(cs) if cs.stroke.is_empty() => Some(cs.fill),
            _ => None,
        })
    }

    /// The border: the stroked circle.
    fn border_stroke(shapes: &[egui::Shape]) -> Option<Stroke> {
        shapes.iter().find_map(|s| match s {
            egui::Shape::Circle(cs) if !cs.stroke.is_empty() => Some(cs.stroke),
            _ => None,
        })
    }

    #[test]
    fn checked_change_snaps_fill_dot_and_border() {
        // Narrow §6 reading: border-colour and fill are NOT in the
        // `transition-[color,box-shadow]` list, so a checked change
        // lands on the endpoints in the very first frame — no leg.
        let ctx = test_ctx(); // nova default: the checked disc is opaque.
        let mut checked = false;
        let mut t = 0.0;
        for _ in 0..5 {
            let _ = run_item_frame(&ctx, t, &mut checked, "snap");
            t += 0.05;
        }
        checked = true;
        let first = run_item_frame(&ctx, t, &mut checked, "snap");
        let mut settled = vec![];
        for _ in 0..5 {
            settled = run_item_frame(&ctx, t + 0.3, &mut checked, "snap");
            t += 0.05;
        }
        assert_eq!(fill_rect_color(&first), fill_rect_color(&settled));
        assert_eq!(dot_fill(&first), dot_fill(&settled));
        assert_eq!(border_stroke(&first), border_stroke(&settled));
        // Discrimination: the endpoints are the nova checked values.
        let p = palette(Mode::Light);
        assert_eq!(fill_rect_color(&settled), Some(p.primary));
        assert_eq!(dot_fill(&settled), Some(p.primary_foreground));
        assert_eq!(border_stroke(&settled).map(|s| s.color), Some(p.primary));
    }

    #[test]
    fn ring_fades_on_focus_change() {
        // The `box-shadow` leg DOES animate (§6): the ring fades in over
        // the interim 150ms leg when focus arrives — no pop-in, no snap.
        let ctx = test_ctx();
        let mut checked = false;
        let item_id = egui::Id::unique("radio_anim_test").with("ringfade");
        // Settle unfocused: no ring shape at all.
        let mut t = 0.0;
        let mut shapes = vec![];
        for _ in 0..5 {
            shapes = run_item_frame(&ctx, t, &mut checked, "ringfade");
            t += 0.05;
        }
        assert_eq!(ring_stroke(&shapes), None, "no ring while unfocused");
        // Focus arrives: the leg starts at alpha 0 on the focus frame...
        ctx.memory_mut(|mem| mem.request_focus(item_id));
        let focus_frame = run_item_frame(&ctx, t, &mut checked, "ringfade");
        assert_eq!(
            ring_stroke(&focus_frame),
            None,
            "leg starts at alpha 0 (no pop-in)"
        );
        // ...runs through an intermediate alpha mid-leg (75ms into the
        // 150ms leg)...
        let mid = run_item_frame(&ctx, t + 0.075, &mut checked, "ringfade");
        let mid_alpha = ring_stroke(&mid).map(|s| s.color.a()).unwrap_or(0);
        // ...and lands exactly on the settled ring colour.
        let mut settled = vec![];
        for _ in 0..5 {
            settled = run_item_frame(&ctx, t + 0.3, &mut checked, "ringfade");
            t += 0.05;
        }
        let full_alpha = ring_stroke(&settled).map(|s| s.color.a()).unwrap_or(0);
        let p = palette(Mode::Light);
        assert_eq!(
            ring_stroke(&settled).map(|s| s.color),
            Some(color_alpha(p.ring, 0.5))
        );
        assert!(
            mid_alpha > 0 && mid_alpha < full_alpha,
            "mid-leg alpha {mid_alpha} must be strictly between 0 and {full_alpha}"
        );
    }

    #[test]
    fn space_activates_on_keyup_not_keydown_standalone() {
        // Settled OPEN-3 (RAD-INT-05): Space activation is the native
        // button click — it fires on KEYUP, once per press, with no
        // hold-repeat. The standalone item carries the verdict: it can
        // be focused-but-unchecked, which group items cannot (arrows
        // move selection with focus). Fails pre-fix: the item is
        // checked on the keydown pass itself.
        let ctx = test_ctx();
        let mut checked = false;
        let item_id = egui::Id::unique("radio_space_test").with("radio_space");

        let run = |ctx: &egui::Context, checked: &mut bool, input: egui::RawInput| -> bool {
            let mut changed = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_space_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                let resp = RadioGroupItem::new(checked).id_salt("radio_space").show(ui);
                changed = resp.changed();
            });
            out.textures_delta.clear();
            changed
        };

        run(&ctx, &mut checked, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(item_id));
        run(&ctx, &mut checked, egui::RawInput::default());
        assert!(ctx.memory(|m| m.has_focus(item_id)));

        // Keydown: must NOT fire.
        assert!(
            !run(&ctx, &mut checked, key_input(Key::Space)),
            "Space keydown must not fire"
        );
        assert!(!checked, "item must stay unchecked while Space is held");
        // Hold with autorepeat: still must NOT fire.
        assert!(
            !run(&ctx, &mut checked, key_repeat_input(Key::Space)),
            "Space repeat must not fire"
        );
        assert!(!checked);
        // Release: fires exactly once.
        assert!(
            run(&ctx, &mut checked, key_release_input(Key::Space)),
            "Space keyup must fire"
        );
        assert!(checked, "item must be checked after keyup");
        // Settle: no second fire.
        assert!(
            !run(&ctx, &mut checked, egui::RawInput::default()),
            "no second fire after release"
        );
        assert!(checked);
    }

    #[test]
    fn space_activates_on_keyup_not_keydown_group() {
        // Same settled OPEN-3 rule inside a group: after an INT-03
        // jump, focus can sit on an unchecked item, so the group path
        // must defer Space to keyup too. Focus is requested directly
        // on the middle item with no selection — the focus != checked
        // state. Fails pre-fix: selection moves on the keydown pass.
        let ctx = test_ctx();
        let mut selected: Option<usize> = None;
        let base = egui::Id::unique("radio_space_group_test").with("radio_space_group");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_space_group_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_space_group")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(1usize)));
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(1usize)));

        run(&ctx, &mut selected, key_input(Key::Space));
        assert_eq!(selected, None, "Space keydown must not select");
        run(&ctx, &mut selected, key_repeat_input(Key::Space));
        assert_eq!(selected, None, "Space repeat must not select");
        run(&ctx, &mut selected, key_release_input(Key::Space));
        assert_eq!(
            selected,
            Some(1),
            "Space keyup must select the focused item"
        );
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, Some(1));
    }

    #[test]
    fn space_keydown_and_keyup_same_pass_fires_once() {
        // Guard: the label-forwarding pair shape (keydown + keyup in
        // one pass) must still activate through the keyup deferral —
        // the helper hides the keydown and re-creates it when the
        // release arrives in the same pass. Passes both before and
        // after the fix; it guards the label-click path against the
        // deferral. (The real label pair is pushed mid-pass, after
        // the helper runs; this pin proves the helper's own pair
        // handling.)
        let ctx = test_ctx();
        let mut checked = false;
        let item_id = egui::Id::unique("radio_space_pair_test").with("radio_space_pair");

        let run = |ctx: &egui::Context, checked: &mut bool, input: egui::RawInput| -> bool {
            let mut changed = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_space_pair_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                let resp = RadioGroupItem::new(checked)
                    .id_salt("radio_space_pair")
                    .show(ui);
                changed = resp.changed();
            });
            out.textures_delta.clear();
            changed
        };

        run(&ctx, &mut checked, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(item_id));
        run(&ctx, &mut checked, egui::RawInput::default());
        assert!(ctx.memory(|m| m.has_focus(item_id)));

        assert!(
            run(&ctx, &mut checked, key_pair_input(Key::Space)),
            "same-pass Space pair must fire"
        );
        assert!(checked);
        assert!(
            !run(&ctx, &mut checked, egui::RawInput::default()),
            "no second fire after the pair"
        );
    }

    #[test]
    fn enter_does_nothing_on_focused_unchecked_standalone() {
        // Settled OPEN-3 (RAD-INT-04): Enter NEVER activates a radio
        // item (Radix `onKeyDown` preventDefault; WAI-ARIA: radio
        // groups don't activate items on enter keypress). The
        // standalone item carries the verdict: it can be
        // focused-but-unchecked, which group items cannot (arrows
        // move selection with focus). Fails pre-fix: Enter checks the
        // item on the keydown pass itself.
        let ctx = test_ctx();
        let mut checked = false;
        let item_id = egui::Id::unique("radio_enter_test").with("radio_enter");

        let run = |ctx: &egui::Context, checked: &mut bool, input: egui::RawInput| -> bool {
            let mut changed = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_enter_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                let resp = RadioGroupItem::new(checked).id_salt("radio_enter").show(ui);
                changed = resp.changed();
            });
            out.textures_delta.clear();
            changed
        };

        run(&ctx, &mut checked, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(item_id));
        run(&ctx, &mut checked, egui::RawInput::default());
        assert!(ctx.memory(|m| m.has_focus(item_id)));

        // Keydown: must NOT fire, focus retained.
        assert!(
            !run(&ctx, &mut checked, key_input(Key::Enter)),
            "Enter keydown must not fire"
        );
        assert!(!checked, "item must stay unchecked after Enter");
        assert!(
            ctx.memory(|m| m.has_focus(item_id)),
            "Enter must not move focus"
        );
        // Hold with autorepeat: still must NOT fire.
        assert!(
            !run(&ctx, &mut checked, key_repeat_input(Key::Enter)),
            "Enter repeat must not fire"
        );
        assert!(!checked);
        // Release: must NOT fire either (pure suppression, no re-fire).
        assert!(
            !run(&ctx, &mut checked, key_release_input(Key::Enter)),
            "Enter keyup must not fire"
        );
        assert!(!checked);
        // Settle: no delayed fire.
        assert!(
            !run(&ctx, &mut checked, egui::RawInput::default()),
            "no delayed fire after Enter"
        );
        assert!(!checked);
    }

    #[test]
    fn enter_does_nothing_on_focused_group_item() {
        // Same settled OPEN-3 rule inside a group: Enter on the
        // focused item must not move the selection. Focus is requested
        // directly on the middle item with no selection — the focus !=
        // checked state (reachable at runtime via an INT-03 jump).
        // Fails pre-fix: Enter selects the focused item on the
        // keydown pass.
        let ctx = test_ctx();
        let mut selected: Option<usize> = None;
        let base = egui::Id::unique("radio_enter_group_test").with("radio_enter_group");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_enter_group_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_enter_group")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(1usize)));
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(1usize)));

        run(&ctx, &mut selected, key_input(Key::Enter));
        assert_eq!(selected, None, "Enter keydown must not select");
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(base.with(1usize)),
            "Enter must not move focus"
        );
        run(&ctx, &mut selected, key_repeat_input(Key::Enter));
        assert_eq!(selected, None, "Enter repeat must not select");
        run(&ctx, &mut selected, key_release_input(Key::Enter));
        assert_eq!(selected, None, "Enter keyup must not select");
        run(&ctx, &mut selected, egui::RawInput::default());
        assert_eq!(selected, None);
    }

    #[test]
    fn enter_does_nothing_on_focused_checked_standalone() {
        // Guard: Enter on a CHECKED item is a no-op too — no uncheck,
        // no spurious `changed()`. Fails pre-fix on the `changed()`
        // assertion only: the fake click re-checks the item and marks
        // it changed.
        let ctx = test_ctx();
        let mut checked = true;
        let item_id = egui::Id::unique("radio_enter_checked_test").with("radio_enter_checked");

        let run = |ctx: &egui::Context, checked: &mut bool, input: egui::RawInput| -> bool {
            let mut changed = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_enter_checked_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                let resp = RadioGroupItem::new(checked)
                    .id_salt("radio_enter_checked")
                    .show(ui);
                changed = resp.changed();
            });
            out.textures_delta.clear();
            changed
        };

        run(&ctx, &mut checked, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(item_id));
        run(&ctx, &mut checked, egui::RawInput::default());
        assert!(ctx.memory(|m| m.has_focus(item_id)));

        assert!(
            !run(&ctx, &mut checked, key_input(Key::Enter)),
            "Enter must not report changed on a checked item"
        );
        assert!(checked, "Enter must not uncheck");
        assert!(
            !run(&ctx, &mut checked, key_release_input(Key::Enter)),
            "Enter keyup must not report changed"
        );
        assert!(checked);
    }

    #[test]
    fn space_keyup_fire_unaffected_by_enter() {
        // Guard: the Enter suppression must not disturb the Space
        // keyup deferral (RAD-INT-05) — an Enter press interleaved
        // with a Space press leaves the Space phase sequence intact:
        // Enter fires nothing, Space fires once on keyup, no second
        // fire. Fails pre-fix on the Enter assertion only.
        let ctx = test_ctx();
        let mut checked = false;
        let item_id = egui::Id::unique("radio_enter_space_test").with("radio_enter_space");

        let run = |ctx: &egui::Context, checked: &mut bool, input: egui::RawInput| -> bool {
            let mut changed = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_enter_space_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                let resp = RadioGroupItem::new(checked)
                    .id_salt("radio_enter_space")
                    .show(ui);
                changed = resp.changed();
            });
            out.textures_delta.clear();
            changed
        };

        run(&ctx, &mut checked, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(item_id));
        run(&ctx, &mut checked, egui::RawInput::default());
        assert!(ctx.memory(|m| m.has_focus(item_id)));

        assert!(
            !run(&ctx, &mut checked, key_input(Key::Enter)),
            "Enter keydown must not fire"
        );
        assert!(!checked);
        assert!(
            !run(&ctx, &mut checked, key_input(Key::Space)),
            "Space keydown must not fire"
        );
        assert!(!checked);
        assert!(
            run(&ctx, &mut checked, key_release_input(Key::Space)),
            "Space keyup must fire"
        );
        assert!(checked, "item must be checked after Space keyup");
        assert!(
            !run(&ctx, &mut checked, egui::RawInput::default()),
            "no second fire after release"
        );
        assert!(checked);
    }

    #[test]
    fn arrow_and_jump_moves_unchanged_by_space_fix() {
        // Guard: arrow + jump behaviour (INT-01/02/03) is untouched by
        // the Space deferral. Passes both before and after the fix.
        let ctx = test_ctx();
        let mut selected = Some(1);
        let base = egui::Id::unique("radio_space_arrow_test").with("radio_space_arrow");

        let run = |ctx: &egui::Context, selected: &mut Option<usize>, input: egui::RawInput| {
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("radio_space_arrow_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
                );
                RadioGroup::new(selected)
                    .id_salt("radio_space_arrow")
                    .option("A")
                    .option("B")
                    .option("C")
                    .show(ui);
            });
            out.textures_delta.clear();
        };
        let press = |ctx: &egui::Context, selected: &mut Option<usize>, key: Key| {
            run(ctx, selected, key_input(key));
            run(ctx, selected, egui::RawInput::default());
        };

        run(&ctx, &mut selected, egui::RawInput::default());
        ctx.memory_mut(|mem| mem.request_focus(base.with(1usize)));
        press(&ctx, &mut selected, Key::ArrowDown);
        assert_eq!(selected, Some(2));
        assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(2usize)));
        press(&ctx, &mut selected, Key::Home);
        assert_eq!(selected, Some(2), "jump must not change selection");
        assert_eq!(ctx.memory(|m| m.focused()), Some(base.with(0usize)));
    }
}
