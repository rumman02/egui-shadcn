//! shadcn `slider.tsx` — a draggable value selector (Batch B S1 rebuild).
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/slider.tsx` (62 lines), one
//! `Slider` component wrapping Radix `SliderPrimitive.Root` with `Track`,
//! `Range`, and one `Thumb` per value. The frozen S0 spec
//! (`~/workspace/batch-b/specs/slider-spec.md`) is the target; section
//! citations below are spec sections.
//!
//! Base recipe — exact class strings (spec §2.1):
//!
//! ```text
//! Root:  relative flex w-full touch-none items-center select-none
//!         data-[disabled]:opacity-50 data-[orientation=vertical]:h-full
//!         data-[orientation=vertical]:min-h-44
//!         data-[orientation=vertical]:w-auto data-[orientation=vertical]:flex-col
//!         (slider.tsx:33)
//! Track: relative grow overflow-hidden rounded-full bg-muted
//!         data-[orientation=horizontal]:h-1.5
//!         data-[orientation=horizontal]:w-full
//!         data-[orientation=vertical]:h-full data-[orientation=vertical]:w-1.5
//!         (slider.tsx:41)
//! Range: absolute bg-primary data-[orientation=horizontal]:h-full
//!         data-[orientation=vertical]:w-full (slider.tsx:47)
//! Thumb: block size-4 shrink-0 rounded-full border border-primary bg-white
//!         shadow-sm ring-ring/50 transition-[color,box-shadow] hover:ring-4
//!         focus-visible:ring-4 focus-visible:outline-hidden
//!         disabled:pointer-events-none disabled:opacity-50 (slider.tsx:55)
//! ```
//!
//! Nova overrides (`.style-nova` scope, `style-nova.css:1228-1242`, spec
//! §2.2) layer over the base recipe; silence keeps the base value (LAYER
//! ruling). The nova thumb rule never mentions shadow or `disabled:` rules,
//! so the base `shadow-sm` and disabled rules survive under nova (spec
//! §2.4 traps 1–2). Effective values (spec §2.3): track 6px → 4px, thumb
//! 16px → 12px, thumb border `primary` → `ring`, ring 4px → 3px, nova-only
//! `active:ring-3`, nova-only 28px hit area (`after:-inset-2`), vertical
//! min-height 176px → 160px. Thumb fill is literal white in BOTH modes and
//! both styles (spec §0.3.1 — neither recipe has a `dark:` thumb variant).
//!
//! # Value model — the N1 invariant (spec §7, P0)
//!
//! Two domains, never mixed: ABSOLUTE `value` in [`min`, `max`] (state, API,
//! events, keyboard math — everywhere except the paint/hit-test boundary)
//! and FRACTION in [0, 1] (only at the paint boundary, value → thumb
//! position, and the hit-test boundary, pointer → value). The keyboard path
//! contains NO fraction: it reads an absolute value, adds/subtracts an
//! absolute step, clamps to absolute bounds, and writes an absolute value.
//! Every fraction holder is named `fraction`; every absolute holder is named
//! `value`; the two domains never share a helper. The internal setters are
//! [`Slider::set_value_absolute`] (absolute domain) and
//! [`Slider::set_value_from_fraction`] (fraction domain, converting
//! explicitly through [`fraction_to_value`]).
//!
//! # Geometry (spec §3)
//!
//! The thumb CENTRE travels the full Track length: at `min` it sits on the
//! Track's start edge, at `max` on the end edge, overhanging by half the
//! thumb diameter at the extremes. Vertical sliders grow bottom-up. The
//! Range spans from the Track start to the thumb centre (single value) or
//! between the outermost thumb centres (multi-value), clipped to the
//! Track's rounded ends. Vertical sliders stretch to the offered height
//! (`h-full`) with a 176px/160px floor (spec §3.2).
//!
//! # States (spec §4)
//!
//! Resting / hover / focus-visible / active rings per the state matrix; the
//! active ring exists ONLY under nova. Disabled renders the WHOLE control at
//! exactly 50% opacity, uniformly — the thumb is never faded twice (spec
//! §4.4, N8). No invalid state exists for slider (spec §4.6).
//!
//! # Animation (spec §6, interim per OPEN-11)
//!
//! Only the thumb's `transition-[color,box-shadow]` exists, read NARROWLY
//! (ruling 6): `color` is text colour only, so the track, Range, thumb
//! fill and thumb stroke SNAP on restyle; only the ring (box-shadow)
//! animates, over the Tailwind v4 framework defaults (150ms,
//! `cubic-bezier(0.4, 0, 0.2, 1)`); thumb position and Range extent snap.
//! The 150ms/easing values are framework defaults, NOT snapshot evidence.
//!
//! # Keyboard and pointer (spec §8 — working assumptions, OPEN items)
//!
//! Arrow keys step by the absolute step (Right/Up +, Left/Down −, both
//! orientations, W-3); Shift+Arrow moves 10 × step, exactly like
//! PageUp/PageDown (Radix `isSkipKey`, SLD-INT-01 — S6 Work B correction
//! of W-3; other modifiers do not change the step size); Home/End act
//! on the FIRST/LAST value index (not the focused thumb) through
//! snap-then-clamp: Home → `min`, End → snapped `max` (W-4 CORRECTED
//! by S6 Work B, OPEN-4 CLOSED, gate G7); PageUp/PageDown move 10 × step (W-2, one named
//! place); thumbs clamp to their neighbours
//! without pushing or swapping (W-5); track presses move the nearest thumb
//! and begin a drag (W-6a/W-6b); every thumb is a Tab stop in value order
//! (W-9). The track itself is not a Tab stop (SLD-INT-05): it paints no
//! ring and handles no keys, so only thumb ids are focusable. Pointerdown on a thumb focuses it, and every user-driven value
//! change re-focuses the changed thumb (Radix focus model, SLD-INT-03),
//! so arrow keys act on the thumb just clicked or dragged. Default step
//! is 1 absolute (W-1 CORRECTED by S6 Work B, OPEN-1 CLOSED, one named
//! constant).
//! S6 Work B confirms or corrects each assumption from Radix source.
//!
//! # Platform limits (spec §11, proposed deviations)
//!
//! D1 no ARIA tree, D2 `:focus-visible` modality (ring on any focus), D3
//! `data-slot` attributes dropped — all proposed, reviewer-decided.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut volume = 40.0;
//! Slider::new(&mut volume).min(0.0).max(100.0).show(ui);
//!
//! let mut bounds = (20.0, 80.0);
//! Slider::range(&mut bounds).min(0.0).max(100.0).show(ui);
//!
//! Slider::new(&mut volume).orientation(GroupOrientation::Vertical).show(ui);
//! ```

use crate::components::button::GroupOrientation;
use crate::theme::{blend, color_alpha, Palette, ShadcnTheme, Style};
use crate::utils::animation::{transition_alpha, TransitionSpec};
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Color32, CornerRadius, Event, EventFilter, Key, Painter, Pos2, Rect, Response, Sense, Stroke,
    Ui, Vec2,
};

/// `size-4` — the (base) thumb diameter; nova renders `size-3` (12pt).
pub const SLIDER_THUMB_SIZE: f32 = 16.0;
/// `h-1.5` — the (base) track thickness; nova renders `h-1` (4pt).
pub const SLIDER_TRACK_HEIGHT: f32 = 6.0;
/// Extra room so the focus/hover ring is not clipped.
const WIDGET_WIDTH: f32 = 24.0;

/// Default step as an ABSOLUTE value (working assumption W-1, corrected
/// by S6 Work B, spec §8.1/OPEN-1 CLOSED): when the caller supplies no
/// `step`, the step size is 1, regardless of the domain span (Radix
/// `step` = 1 absolute at every era, 2020–2026; gate G5). ONE named
/// place.
const DEFAULT_STEP: f32 = 1.0;

/// PageUp/PageDown magnitude as a multiple of the step (working assumption
/// W-2, spec §8.1/OPEN-2): a large step is 10 × `step`. ONE named place.
const PAGE_STEP_MULTIPLIER: f32 = 10.0;

/// Thumb diameter per style (`size-4` base, nova `size-3`).
pub(crate) fn thumb_size(style: Style) -> f32 {
    match style {
        Style::Nova => 12.0,
        _ => SLIDER_THUMB_SIZE,
    }
}

/// Track thickness per style (`h-1.5` base, nova `h-1`).
pub(crate) fn track_height(style: Style) -> f32 {
    match style {
        Style::Nova => 4.0,
        _ => SLIDER_TRACK_HEIGHT,
    }
}

/// Thumb halo width per style (`ring-4` base, nova `ring-3`).
pub(crate) fn ring_width(style: Style) -> f32 {
    match style {
        Style::Nova => 3.0,
        _ => 4.0,
    }
}

/// Radius passed to `circle_stroke` for the thumb halo (SLD-VIS-01).
/// egui 0.37 renders a `circle_stroke` band entirely OUTSIDE the given
/// radius ([r, r + width] — verified empirically from captures), so the
/// ring must be passed the thumb radius itself for the band to start at
/// the border-box edge (Tailwind outset `box-shadow` semantics: nova
/// [6, 9], base [8, 12]). Passing `radius + width / 2.0` (the
/// centred-stroke assumption) floats the band half a width outward.
pub(crate) fn ring_stroke_radius(style: Style) -> f32 {
    thumb_size(style) / 2.0
}

/// Thumb hit-area side per style (spec §3.5): under nova the `::after`
/// pseudo-element (`after:-inset-2`) extends 8px beyond the 12px thumb box,
/// giving a 28px grab area; under base the hit area is the thumb box itself.
/// Do NOT apply the 28px area under base (spec §2.4 trap 4).
pub(crate) fn thumb_hit_size(style: Style) -> f32 {
    match style {
        Style::Nova => 28.0,
        _ => thumb_size(style),
    }
}

/// Reserved column height for `data-[orientation=vertical]`
/// (base `min-h-44` = 176, nova `min-h-40` = 160). This is a MINIMUM floor
/// (spec §3.2): the slider stretches to the offered height (`h-full`).
pub(crate) fn vertical_min_height(style: Style) -> f32 {
    match style {
        Style::Nova => 160.0,
        _ => 176.0,
    }
}

/// What values the slider controls.
#[derive(Debug)]
pub enum SliderValue<'a> {
    /// One thumb.
    Single(&'a mut f32),
    /// Two thumbs (lo must stay <= hi).
    Range(&'a mut (f32, f32)),
}

impl<'a> SliderValue<'a> {
    fn count(&self) -> usize {
        match self {
            SliderValue::Single(_) => 1,
            SliderValue::Range(_) => 2,
        }
    }
}

/// Resolved colors for one slider state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct SliderColors {
    track: Color32,
    range: Color32,
    thumb_fill: Color32,
    thumb_stroke: Stroke,
    /// Focus/hover ring color (alpha already applied).
    ring: Color32,
}

impl SliderColors {
    /// Fades every painted colour by `alpha`, applied EXACTLY ONCE (spec
    /// §4.4, N8): the disabled fade is a single whole-control application.
    /// Callers must never fade an already-faded set again — the thumb at
    /// 25% is the forbidden N8 shape.
    fn faded(self, alpha: f32) -> Self {
        Self {
            track: color_alpha(self.track, alpha),
            range: color_alpha(self.range, alpha),
            thumb_fill: color_alpha(self.thumb_fill, alpha),
            thumb_stroke: Stroke::new(
                self.thumb_stroke.width,
                color_alpha(self.thumb_stroke.color, alpha),
            ),
            ring: color_alpha(self.ring, alpha),
        }
    }
}

/// Opaque group-fade composite for a disabled slot that paints OVER
/// another faded layer (spec §4.4, N8; SLD-VIS-02).
///
/// CSS `opacity-50` composites the whole subtree first and fades it once.
/// Painting the faded slot colour translucently over the already-faded
/// Track lets the Track contribute twice (the N8 double fade): the dark
/// disabled Range read 127 instead of 120, and the thumb fill let the
/// Track show through as a ~46-level band. Compositing the already-faded
/// slot colour opaquely against the panel background yields the
/// composite-then-fade result (within a rounding level), because the slot
/// is opaque in the enabled render: `blend(backdrop, faded_slot)`.
///
/// The backdrop is the panel background (`Palette::background`): exact
/// wherever the slider paints over the panel, approximate over any other
/// surface — the same limitation the checkbox/radio-group/input group-fade
/// fixes carry (recorded for the wave pack).
fn disabled_group_fade(faded_slot: Color32, backdrop: Color32) -> Color32 {
    blend(backdrop, faded_slot)
}

/// Mirrors the base recipe: track `bg-muted`, range `bg-primary`, thumb
/// `border border-primary bg-white`; nova replaces the thumb border with
/// `border-ring`. The thumb fill is literal white in BOTH modes (spec
/// §0.3.1): neither recipe carries a `dark:` thumb variant, so this
/// resolver takes no `dark` parameter.
pub(crate) fn resolve_slider_colors(p: &Palette, style: Style) -> SliderColors {
    let thumb_stroke_color = match style {
        Style::Nova => p.ring,
        _ => p.primary,
    };
    SliderColors {
        track: p.muted,
        range: p.primary,
        thumb_fill: Color32::WHITE,
        thumb_stroke: Stroke::new(1.0, thumb_stroke_color),
        ring: color_alpha(p.ring, 0.5),
    }
}

/// Paints the thumb's `shadow-sm` through the LAYER resolution API (spec
/// §5.3). The base recipe assigns `shadow-sm` and nova is silent on the
/// thumb shadow, so [`resolve_shadow`] returns `Sm` under every style. When
/// disabled, the §4.4 uniform fade scales the layer alphas once — the
/// shadow is never skipped and never double-faded (the button
/// `paint_outline_shadow` pattern).
fn paint_thumb_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Sm, style, ShadowSlot::SliderThumb).layers();
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
            ShadowSize::Sm,
            style,
            ShadowSlot::SliderThumb,
            dark,
        );
    }
}

/// Whether the thumb shows its ring (spec §4.1/§4.3, pure and unit-tested).
/// Base: hover or keyboard focus only — an active (dragged) thumb renders
/// as resting when the pointer leaves the hit area mid-drag, because the
/// base recipe has no `active:` rule. Nova adds `active:ring-3`, so a
/// dragged thumb keeps its ring. Disabled thumbs never show a ring.
pub(crate) fn thumb_ring_visible(
    style: Style,
    hovered: bool,
    dragged: bool,
    focused: bool,
    disabled: bool,
) -> bool {
    if disabled {
        return false;
    }
    match style {
        Style::Nova => hovered || dragged || focused,
        _ => hovered || focused,
    }
}

// ---------------------------------------------------------------------------
// Value-domain conversions (spec §7.1 — the ONLY two conversions).
// ---------------------------------------------------------------------------

/// Converts an ABSOLUTE `value` to a 0..1 `fraction` (paint boundary only).
/// `min`/`max` are the ORDERED bounds. A degenerate domain (`max <= min`)
/// defines the fraction as 0 (spec §10: no division by zero, no panic).
pub(crate) fn value_to_fraction(value: f32, min: f32, max: f32) -> f32 {
    let span = max - min;
    if span <= 0.0 || span.is_nan() {
        return 0.0;
    }
    ((value - min) / span).clamp(0.0, 1.0)
}

/// Converts a 0..1 `fraction` to an ABSOLUTE value (hit-test boundary only).
/// The fraction is clamped to [0, 1] first (spec §7.3 pointer pipeline
/// step 1). `min`/`max` are the ORDERED bounds.
pub(crate) fn fraction_to_value(fraction: f32, min: f32, max: f32) -> f32 {
    min + fraction.clamp(0.0, 1.0) * (max - min)
}

/// Snaps an ABSOLUTE `value` to the NEAREST step-grid point anchored at
/// `min` (spec §7.4): grid points are `min + k × step`. Round-half goes
/// away from zero (working assumption W-7, OPEN-7 — `f32::round`). A
/// non-positive or non-finite step means "no snapping" (spec §10 choice).
pub(crate) fn snap_value_to_grid(value: f32, min: f32, step: f32) -> f32 {
    if step <= 0.0 || !step.is_finite() {
        return value;
    }
    min + ((value - min) / step).round() * step
}

/// The grid point strictly above `value` (working assumption W-7, OPEN-7:
/// arrows from an off-grid value move to the adjacent grid point in the
/// pressed direction). For an on-grid value this is exactly `value + step`.
fn adjacent_grid_value_above(value: f32, min: f32, step: f32) -> f32 {
    min + (((value - min) / step).floor() + 1.0) * step
}

/// The grid point strictly below `value` (W-7). For an on-grid value this
/// is exactly `value - step`.
fn adjacent_grid_value_below(value: f32, min: f32, step: f32) -> f32 {
    min + (((value - min) / step).ceil() - 1.0) * step
}

/// Whether an ABSOLUTE `value` sits on the step grid anchored at `min`
/// (within float tolerance), for the §7.3 keyboard-pipeline snap rule.
fn is_on_grid(value: f32, min: f32, step: f32) -> bool {
    (snap_value_to_grid(value, min, step) - value).abs() <= step.abs() * 1e-4 + f32::EPSILON
}

/// Index of the thumb whose centre fraction is nearest `fraction`
/// (W-6a/W-6c, OPEN-6). Ties go to the lowest index (documented reading;
/// S6 confirms or corrects the tie-break). Empty input returns 0.
fn nearest_thumb_index(fractions: &[f32], fraction: f32) -> usize {
    let mut best = 0;
    let mut best_dist = f32::INFINITY;
    for (i, centre_fraction) in fractions.iter().enumerate() {
        let dist = (centre_fraction - fraction).abs();
        if dist < best_dist {
            best_dist = dist;
            best = i;
        }
    }
    best
}

/// A shadcn slider. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Slider<'a> {
    value: SliderValue<'a>,
    min: f32,
    max: f32,
    step: Option<f32>,
    orientation: GroupOrientation,
    disabled: bool,
    id_salt: Option<String>,
}

impl<'a> Slider<'a> {
    /// Creates a single-value slider bound to the given float.
    pub fn new(value: &'a mut f32) -> Self {
        Self {
            value: SliderValue::Single(value),
            min: 0.0,
            max: 100.0,
            step: None,
            orientation: GroupOrientation::Horizontal,
            disabled: false,
            id_salt: None,
        }
    }

    /// Creates a two-value (range) slider bound to the given pair
    /// (the source renders one thumb per value).
    pub fn range(value: &'a mut (f32, f32)) -> Self {
        Self {
            value: SliderValue::Range(value),
            min: 0.0,
            max: 100.0,
            step: None,
            orientation: GroupOrientation::Horizontal,
            disabled: false,
            id_salt: None,
        }
    }

    /// Sets the minimum value (source default `0`, `slider.tsx:11`).
    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    /// Sets the maximum value (source default `100`, `slider.tsx:12`).
    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    /// Snaps values to this increment while dragging and on Arrow keys
    /// (step grid anchored at `min`, spec §7.4). When unset, the step is
    /// the default step (1 absolute, W-1 corrected by S6 Work B,
    /// OPEN-1 CLOSED). Non-positive steps
    /// clamp to `f32::EPSILON`, which renders snapping a no-op in practice
    /// (spec §10 choice: clamp to a positive step, never panic).
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step.max(f32::EPSILON));
        self
    }

    /// Sets the layout orientation (`data-[orientation=vertical]` renders a
    /// tall column with the minimum at the bottom; default horizontal).
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disables the slider (`data-[disabled]:opacity-50`, spec §4.4: the
    /// whole control fades uniformly to 50% and ignores input).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt so focus and drag state persist per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The domain bounds in ascending order. A caller-supplied `min > max`
    /// is treated as the swapped domain (spec §10 choice: handle without
    /// panic; the original's behaviour is OPEN-14).
    fn ordered_bounds(&self) -> (f32, f32) {
        if self.min <= self.max {
            (self.min, self.max)
        } else {
            (self.max, self.min)
        }
    }

    /// The ABSOLUTE step size: the explicit step, or the default step
    /// (1 absolute, W-1 corrected by S6 Work B, OPEN-1 CLOSED,
    /// [`DEFAULT_STEP`]).
    fn step_size(&self) -> f32 {
        self.step.unwrap_or(DEFAULT_STEP)
    }

    fn read_value(&self, index: usize) -> f32 {
        match &self.value {
            SliderValue::Single(v) => **v,
            SliderValue::Range(pair) => {
                if index == 0 {
                    pair.0
                } else {
                    pair.1
                }
            }
        }
    }

    /// The keyboard pipeline (spec §7.3): computes and applies the new
    /// ABSOLUTE value for one key press on the focused thumb. This path
    /// contains NO fraction — it reads an absolute value, computes an
    /// absolute target per the §8.1 table, and commits it through
    /// [`Self::set_value_absolute`]. Unhandled keys and degenerate domains
    /// are no-ops. `changed` is set iff the committed value differs.
    /// Home/End are the exception to focused-thumb targeting: they act
    /// on the FIRST/LAST value index (gate G7, Work B OPEN-4), never the
    /// focused thumb, and End lands on the snapped max. Focus never
    /// moves in this function.
    ///
    /// `shift` is the Shift-modifier state at the time of the press
    /// (SLD-INT-01): Radix `isSkipKey` — `isPageKey || (shiftKey &&
    /// arrowKey)` — selects the 10 × step path. Only `shiftKey` is
    /// consulted (Radix checks no other modifier), so Ctrl/Alt/Meta
    /// +Arrow steps exactly like a plain arrow.
    fn keyboard_step(&mut self, index: usize, key: Key, shift: bool, changed: &mut bool) {
        let (lo, hi) = self.ordered_bounds();
        let span = hi - lo;
        if span <= 0.0 || span.is_nan() {
            return;
        }
        let step = self.step_size();
        let current_value = self.read_value(index);
        // Radix `isSkipKey` (OPEN-3, corrected by S6 Work B): page keys,
        // or Shift+arrow, take the 10 × step path. Shift+Arrow selects
        // the SAME path as the corresponding page key, including the
        // §7.3 off-grid snap rule below.
        let is_skip = matches!(key, Key::PageUp | Key::PageDown)
            || (shift
                && matches!(
                    key,
                    Key::ArrowRight | Key::ArrowUp | Key::ArrowLeft | Key::ArrowDown
                ));
        // S9-C2 (gate G7, Work B OPEN-4): Home/End target the FIRST/LAST
        // value index, NOT the focused thumb (Radix `onHomeKeyDown` ->
        // `updateValues(min, 0)`; `onEndKeyDown` -> `updateValues(max,
        // values.length - 1)`). All other keys act on the focused thumb.
        // Focus does not move: this function never touches focus.
        let target_index = match key {
            Key::Home => 0,
            Key::End => match &self.value {
                SliderValue::Single(_) => 0,
                SliderValue::Range(_) => 1,
            },
            _ => index,
        };
        let target_value = match key {
            Key::ArrowRight | Key::ArrowUp if is_skip => {
                current_value + PAGE_STEP_MULTIPLIER * step
            }
            Key::ArrowLeft | Key::ArrowDown if is_skip => {
                current_value - PAGE_STEP_MULTIPLIER * step
            }
            Key::ArrowRight | Key::ArrowUp => adjacent_grid_value_above(current_value, lo, step),
            Key::ArrowLeft | Key::ArrowDown => adjacent_grid_value_below(current_value, lo, step),
            Key::Home => lo,
            // End lands on the SNAPPED max: Radix `updateValues` snaps
            // then clamps (nearest grid point anchored at min, then the
            // bounds/neighbour clamp in `set_value_absolute`) — e.g.
            // 0–10 step 3 -> 9. Same snap + commit path as the
            // arrow/page keys. Home needs no snap: `min` is grid point
            // k=0 by construction.
            Key::End => snap_value_to_grid(hi, lo, step),
            // W-2 (OPEN-2): a large step is 10 × step, one named place.
            Key::PageUp => current_value + PAGE_STEP_MULTIPLIER * step,
            Key::PageDown => current_value - PAGE_STEP_MULTIPLIER * step,
            _ => return,
        };
        // §7.3 step 3: snap skip-key targets ONLY when the starting
        // value was off-grid. Arrow targets land on-grid by construction,
        // Home is a bound (never snapped), and End is pre-snapped above;
        // skip-key targets (page keys and Shift+arrows) snap to the
        // nearest grid point from an off-grid start.
        let target_value = if is_skip && !is_on_grid(current_value, lo, step) {
            snap_value_to_grid(target_value, lo, step)
        } else {
            target_value
        };
        self.set_value_absolute(target_index, target_value, changed);
    }

    /// Commits an ABSOLUTE `value` for one thumb: clamps to the ordered
    /// bounds, clamps against the neighbouring thumb (W-5: no passing, no
    /// pushing, no swapping), and writes the value, setting `changed` iff
    /// it differs. This setter takes the ABSOLUTE domain only (spec §7.2
    /// clause (d)); fraction callers convert first via
    /// [`fraction_to_value`] in [`Self::set_value_from_fraction`].
    fn set_value_absolute(&mut self, index: usize, value: f32, changed: &mut bool) {
        let (lo, hi) = self.ordered_bounds();
        let clamped_value = value.clamp(lo, hi);
        match &mut self.value {
            SliderValue::Single(v) => {
                if (**v - clamped_value).abs() > f32::EPSILON {
                    **v = clamped_value;
                    *changed = true;
                }
            }
            SliderValue::Range(pair) => {
                let neighbour_clamped = if index == 0 {
                    // lo may not pass hi
                    clamped_value.min(pair.1)
                } else {
                    clamped_value.max(pair.0)
                };
                // Re-clamp: a programmatically out-of-range neighbour must
                // never push the target outside the bounds.
                let target_value = neighbour_clamped.clamp(lo, hi);
                let current_value = if index == 0 { pair.0 } else { pair.1 };
                if (current_value - target_value).abs() > f32::EPSILON {
                    if index == 0 {
                        pair.0 = target_value;
                    } else {
                        pair.1 = target_value;
                    }
                    *changed = true;
                }
            }
        }
    }

    /// The pointer pipeline (spec §7.3): converts a 0..1 `fraction` to an
    /// ABSOLUTE value, snaps it to the step grid, and commits it through
    /// [`Self::set_value_absolute`]. The fraction is clamped to [0, 1]
    /// before conversion, so drags past the Track ends clamp to the bounds.
    fn set_value_from_fraction(&mut self, index: usize, fraction: f32, changed: &mut bool) {
        let (lo, hi) = self.ordered_bounds();
        let value = fraction_to_value(fraction, lo, hi);
        let snapped_value = snap_value_to_grid(value, lo, self.step_size());
        self.set_value_absolute(index, snapped_value, changed);
    }

    /// Renders the slider (spec §§2–4 geometry and states).
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();
        let style = theme.style;

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_slider"),
        };

        // §6 + ruling 6: the track, Range, thumb fill and thumb stroke
        // SNAP to the resolved endpoint colours — no transition class
        // covers them (`transition-[color,box-shadow]` reads narrowly:
        // `color` is text colour only; background/border colours snap).
        // Only the ring (box-shadow) animates, via `transition_alpha`
        // below (interim timing, OPEN-11). SLD-ANM-01: the old endpoint
        // animator eased these slots on restyle (including the border —
        // the S7 "border snaps" reading was under-sampled); resolving
        // directly makes them snap, as ruling 6 requires, and leaves
        // the ring ease untouched. The §4.4 uniform disabled fade is
        // then applied exactly once via `faded`.
        let transition_spec = TransitionSpec::default();
        let resolved = resolve_slider_colors(&p, style);
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        let mut c = resolved.faded(alpha);
        if self.disabled {
            // SLD-VIS-02: the Range and the thumb fill paint OVER the
            // faded Track, so they must be composited opaquely against
            // the panel background — translucent paint lets the Track
            // show through (double fade). The Track itself stays
            // translucent: it paints directly over the panel, so its
            // faded value is already the group composite (250/24, exact).
            // The 1px border stays translucent: it paints over the fill,
            // whose value is now uniform, so the border ink is uniform
            // too (and unchanged where it was already correct).
            c.range = disabled_group_fade(c.range, p.background);
            c.thumb_fill = disabled_group_fade(c.thumb_fill, p.background);
        }

        let vertical = self.orientation == GroupOrientation::Vertical;
        let thumb = thumb_size(style);
        let track = track_height(style);
        let hit_size = thumb_hit_size(style);
        let (lo, hi) = self.ordered_bounds();
        let (_, rect) = if vertical {
            // `data-[orientation=vertical]:h-full` with the `min-h-44` /
            // `min-h-40` floor (spec §3.2): stretch to the offered height,
            // never below the floor.
            let offered = ui.available_height();
            let height = if offered.is_finite() {
                offered.max(vertical_min_height(style))
            } else {
                vertical_min_height(style)
            };
            ui.allocate_space(Vec2::new(WIDGET_WIDTH, height))
        } else {
            let width = ui.available_width().max(thumb * 3.0);
            ui.allocate_space(Vec2::new(width, WIDGET_WIDTH))
        };
        let track_rect = if vertical {
            Rect::from_center_size(rect.center(), Vec2::new(track, rect.height()))
        } else {
            Rect::from_center_size(rect.center(), Vec2::new(rect.width(), track))
        };

        if !ui.is_rect_visible(rect) {
            return ui.interact(rect, id, Sense::hover());
        }

        let painter = ui.painter();

        // Axis helpers, in FRACTION space (spec §3.3): the thumb centre
        // travels the full Track length; vertical sliders grow bottom-up.
        let axis_len = if vertical {
            track_rect.height()
        } else {
            track_rect.width()
        };
        let pos_at = |fraction: f32| -> f32 {
            if vertical {
                track_rect.max.y - fraction * axis_len
            } else {
                track_rect.min.x + fraction * axis_len
            }
        };
        let thumb_center = |fraction: f32| -> Pos2 {
            if vertical {
                Pos2::new(track_rect.center().x, pos_at(fraction))
            } else {
                Pos2::new(pos_at(fraction), rect.center().y)
            }
        };

        // Track (`rounded-full bg-muted`) and range (`bg-primary`).
        painter.rect_filled(track_rect, CornerRadius::same(track as u8 / 2), c.track);
        let n = self.value.count();
        let values: Vec<f32> = (0..n).map(|i| self.read_value(i)).collect();
        let fractions: Vec<f32> = values
            .iter()
            .map(|value| value_to_fraction(*value, lo, hi))
            .collect();
        // §3.4: a single value fills from the Track start to the thumb
        // centre; multiple values fill between the outermost centres.
        let (range_start_fraction, range_end_fraction) = match fractions.as_slice() {
            [] => (0.0, 0.0),
            [only_fraction] => (0.0, *only_fraction),
            _ => (
                fractions.iter().fold(f32::INFINITY, |a, b| a.min(*b)),
                fractions.iter().fold(f32::NEG_INFINITY, |a, b| a.max(*b)),
            ),
        };
        if range_end_fraction > range_start_fraction {
            let range_rect = if vertical {
                Rect::from_min_max(
                    Pos2::new(track_rect.min.x, pos_at(range_end_fraction)),
                    Pos2::new(track_rect.max.x, pos_at(range_start_fraction)),
                )
            } else {
                Rect::from_min_max(
                    Pos2::new(pos_at(range_start_fraction), track_rect.min.y),
                    Pos2::new(pos_at(range_end_fraction), track_rect.max.y),
                )
            };
            painter.rect_filled(range_rect, CornerRadius::same(track as u8 / 2), c.range);
        }

        // Track interaction: the thumbs are interacted after the track, so
        // they win when the pointer is directly over them. A disabled
        // control takes no focus and no input (spec §8.4, W-10).
        //
        // SLD-INT-05: the track itself must NOT be focusable. The
        // enabled track used `Sense::drag()` (`DRAG | FOCUSABLE`), so
        // its id (`id.with("track")`) was a Tab stop — and a silent
        // one: nothing paints a ring for the track id, and the keyboard
        // block below reads thumb responses only, so arrows at a track
        // stop changed nothing anywhere. Tab order is the pass order
        // of focusable interacts (egui's give-to-next), and the track
        // is interacted before the thumbs, so each slider contributed
        // one gap stop BEFORE its thumbs — the gap S6 filed "after"
        // each slider's thumbs is the NEXT slider's track id (the
        // pin `tab_walk_lands_only_on_thumbs_no_gap_stops_sld_int_05`
        // names every landing and proves the identity and order).
        // `Sense::DRAG` is the documented non-focusable drag sense
        // (egui `sense.rs`: "Use `Sense::DRAG` if you don't want the
        // widget to be focusable"). Drag behaviour is unchanged: drag
        // sensing is independent of focusability, the INT-03 focus
        // grants target thumb ids (never the track id), and the
        // keyboard block reads thumb responses only. Removing the
        // track from the focus cache also removes it as an arrow-key
        // cone-search landing spot (the INT-04 silent-drop shape).
        let track_sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::DRAG
        };
        let track_response = ui.interact(rect, id.with("track"), track_sense);

        let mut changed = false;
        let pointer_fraction = |ui: &Ui| -> f32 {
            ui.input(|i| {
                i.pointer
                    .latest_pos()
                    .map(|pos| {
                        let along = if vertical {
                            track_rect.max.y - pos.y
                        } else {
                            pos.x - track_rect.min.x
                        };
                        along / axis_len.max(f32::EPSILON)
                    })
                    .unwrap_or(0.0)
            })
        };

        let mut thumb_responses: Vec<Response> = Vec::with_capacity(n);
        for (i, fraction) in fractions.iter().enumerate() {
            let thumb_rect = Rect::from_center_size(thumb_center(*fraction), Vec2::splat(thumb));
            // §3.5 hit area: the nova 28px grab area, the thumb box under
            // base. The painted thumb always uses `thumb_rect`.
            let hit_rect = Rect::from_center_size(thumb_rect.center(), Vec2::splat(hit_size));
            let sense = if self.disabled {
                Sense::hover()
            } else {
                Sense::drag()
            };
            let response = ui.interact(hit_rect, id.with("thumb").with(i), sense);
            if response.dragged() {
                // SLD-INT-03: pointerdown focuses the thumb (Radix
                // focus model: pointerdown focuses the thumb, and every
                // user-driven value change re-focuses the changed
                // thumb). egui 0.37's `interact` never grants focus on
                // a pointer press — it only surrenders focus when the
                // pointer presses elsewhere — so the widget must
                // request focus itself. The clicked-block shape used
                // by button/checkbox/toggle cannot work here: the
                // thumb uses `Sense::drag()`, which never reports
                // `clicked()`. For a drag-only widget `dragged()` is
                // true on the press frame itself (`interaction.rs`:
                // "just sensitive to drags, so we can mark it as
                // dragged right away"), so gating on `dragged()`
                // covers pointerdown AND every drag frame. Disabled
                // thumbs use `Sense::hover()`, which never reports a
                // drag, so no focus request can fire for them.
                response.request_focus();
                self.set_value_from_fraction(i, pointer_fraction(ui), &mut changed);
            }
            thumb_responses.push(response);
        }

        // Track press/drag (W-6a/W-6b, OPEN-6): on press, grab the nearest
        // thumb and move it to the pointer; while the drag continues, keep
        // moving that same thumb. The active thumb lives in temp data only
        // for the drag's duration and is cleared on release.
        let active_thumb_id = id.with("active_thumb");
        if !self.disabled && track_response.drag_started() {
            let fraction = pointer_fraction(ui);
            let nearest = nearest_thumb_index(&fractions, fraction);
            ui.ctx()
                .data_mut(|d| d.insert_temp(active_thumb_id, nearest));
            // SLD-INT-03: a track press grabs the nearest thumb — focus
            // it (Radix re-focus rule: the value change re-focuses the
            // changed thumb).
            thumb_responses[nearest].request_focus();
            self.set_value_from_fraction(nearest, fraction, &mut changed);
        } else if !self.disabled && track_response.dragged() {
            let fraction = pointer_fraction(ui);
            let active = ui.ctx().data_mut(|d| d.get_temp::<usize>(active_thumb_id));
            let nearest = active
                .filter(|index| *index < n)
                .unwrap_or_else(|| nearest_thumb_index(&fractions, fraction));
            // SLD-INT-03: re-focus the active thumb on every value
            // change during a track drag (Radix re-focus rule).
            thumb_responses[nearest].request_focus();
            self.set_value_from_fraction(nearest, fraction, &mut changed);
        }
        if track_response.drag_stopped() {
            // SLD-INT-03: re-focus the grabbed thumb on release. The
            // press frame moved the thumb under the pointer, but the
            // release frame's hit test uses the pre-jump rect, so
            // `get_response` surrenders the press-frame focus as a
            // "click elsewhere" during the thumb interacts above.
            // Re-requesting here — after the interacts, before paint
            // — restores it, so the ring stays after the pointer
            // leaves. Guarded on an actual pointer release so an
            // Escape-aborted drag (SLD-INT-07's territory) does not
            // re-focus: Escape clears focus in `begin_pass`, before
            // this pass runs.
            if !self.disabled && ui.input(|i| i.pointer.any_released()) {
                let active = ui.ctx().data_mut(|d| d.get_temp::<usize>(active_thumb_id));
                if let Some(index) = active.filter(|index| *index < n) {
                    thumb_responses[index].request_focus();
                }
            }
            ui.ctx()
                .data_mut(|d| d.remove_temp::<usize>(active_thumb_id));
        }

        // Thumbs: `size-4 rounded-full border border-primary bg-white
        // shadow-sm` (nova: `size-3 border-ring`, `ring-3`). Paint order
        // (spec §4.1): shadow beneath the fill; fill; 1px border inside the
        // box; ring halo outside the border box.
        for (i, response) in thumb_responses.iter().enumerate() {
            if !ui.is_rect_visible(response.rect) {
                continue;
            }
            let fraction = value_to_fraction(self.read_value(i), lo, hi);
            let thumb_rect = Rect::from_center_size(thumb_center(fraction), Vec2::splat(thumb));
            let center = thumb_rect.center();
            let radius = thumb / 2.0;
            paint_thumb_shadow(
                painter,
                thumb_rect,
                CornerRadius::same(thumb as u8 / 2),
                style,
                dark,
                self.disabled,
            );
            painter.circle_filled(center, radius, c.thumb_fill);
            painter.circle_stroke(center, radius - c.thumb_stroke.width / 2.0, c.thumb_stroke);
            // §6: the ring fades in/out over the framework-default timing
            // (interim, OPEN-11); position never animates.
            let ring_active = thumb_ring_visible(
                style,
                response.hovered(),
                response.dragged(),
                response.has_focus(),
                self.disabled,
            );
            let ring_alpha = transition_alpha(
                ui.ctx(),
                id.with("thumb").with(i).with("ring"),
                0.0,
                1.0,
                ring_active,
                transition_spec,
            )
            .alpha;
            if ring_alpha > 0.003 {
                let width = ring_width(style);
                // SLD-VIS-01: `circle_stroke` renders the band OUTSIDE the
                // given radius, so pass the thumb radius itself — the band
                // then starts at the border-box edge (see `ring_stroke_radius`).
                painter.circle_stroke(
                    center,
                    ring_stroke_radius(style),
                    Stroke::new(width, color_alpha(c.ring, ring_alpha)),
                );
            }
        }

        // Keyboard on the focused thumb (spec §8.1, absolute deltas only —
        // see `keyboard_step`). Every thumb is a Tab stop in value order
        // (W-9, OPEN-9).
        if !self.disabled {
            for (i, response) in thumb_responses.iter().enumerate() {
                if !response.has_focus() {
                    continue;
                }
                // Claim both arrow axes for the focused thumb
                // (SLD-INT-04). egui 0.37 `Focus::begin_pass` sets a
                // cardinal `focus_direction` on every unmodified arrow
                // press whose target widget's `EventFilter` does not
                // claim the arrow, and `Focus::end_pass` then moves
                // focus to the spatially nearest widget in that
                // direction — AFTER the UI pass already stepped the
                // value. So the first arrow stepped the right thumb,
                // the ring faded (focus moved), and later arrows acted
                // on whatever focus landed on (misroute) or on a
                // track/gap id with no key handling (silent drop).
                // Home/End/PageUp/Shift+arrows never set a focus
                // direction, which is why they kept the ring. The
                // filter latches only after two consecutive focused
                // frames (`Memory::set_focus_lock_filter`), so this
                // runs every frame the thumb has focus — the stock
                // `slider_core::keyboard_steps` pattern. Both axes are
                // claimed for both orientations: all four arrows step
                // in both orientations (spec §8.1, W-3). `tab` stays
                // unclaimed (Tab navigation, W-9) and `escape` stays
                // unclaimed (Escape clearing focus is SLD-INT-07's).
                ui.memory_mut(|mem| {
                    mem.set_focus_lock_filter(
                        response.id,
                        EventFilter {
                            horizontal_arrows: true,
                            vertical_arrows: true,
                            ..Default::default()
                        },
                    );
                });
                // Shift state for the Radix `isSkipKey` rule (SLD-INT-01):
                // read from the Key event itself, NOT from
                // `InputState::modifiers`. `modifiers` holds the
                // frame-END state (updated only from
                // `Event::ModifiersChanged`), so a whole synthetic
                // press — Shift down, arrow press/release, Shift up —
                // landing in one frame reads as "no modifiers" and
                // steps 1× (observed in the rendered after-phase
                // probes: only the first Shift+Arrow stepped 10×). The
                // Key event carries "the state of the modifier keys at
                // the time of the event" (egui 0.37 `Event::Key`), which
                // survives coalescing — the Radix `shiftKey`
                // equivalent. This scan is exactly consistent with
                // `key_pressed`, which `num_presses` implements by
                // scanning the same events vector for `Event::Key {
                // pressed: true, .. }`.
                let (pressed_key, shift) = ui.input(|inp| {
                    let pressed_key = [
                        Key::ArrowRight,
                        Key::ArrowUp,
                        Key::ArrowLeft,
                        Key::ArrowDown,
                        Key::Home,
                        Key::End,
                        Key::PageUp,
                        Key::PageDown,
                    ]
                    .into_iter()
                    .find(|key| inp.key_pressed(*key));
                    let shift = pressed_key.is_some_and(|key| {
                        inp.events.iter().any(|event| {
                            matches!(
                                event,
                                Event::Key {
                                    key: event_key,
                                    pressed: true,
                                    modifiers,
                                    ..
                                } if *event_key == key && modifiers.shift
                            )
                        })
                    });
                    (pressed_key, shift)
                });
                if let Some(key) = pressed_key {
                    self.keyboard_step(i, key, shift, &mut changed);
                }
            }
        }

        let mut response = track_response;
        if changed {
            response.mark_changed();
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Style;
    use crate::utils::shadow::{resolve_shadow, ShadowSize, ShadowSlot};

    fn palette(mode: crate::theme::Mode) -> Palette {
        crate::theme::Preset::default().build(mode).palette
    }

    #[test]
    fn track_and_thumb_use_source_tokens() {
        let p = palette(crate::theme::Mode::Light);
        let c = resolve_slider_colors(&p, Style::Vega);
        assert_eq!(c.track, p.muted);
        assert_eq!(c.range, p.primary);
        assert_eq!(c.thumb_fill, Color32::WHITE);
        assert_eq!(c.thumb_stroke.color, p.primary);
        assert_eq!(c.ring, color_alpha(p.ring, 0.5));
        // §0.3.1: the thumb fill is white in dark mode too — no `dark:`
        // thumb variant exists in either recipe.
        let dark = palette(crate::theme::Mode::Dark);
        let c = resolve_slider_colors(&dark, Style::Vega);
        assert_eq!(c.thumb_fill, Color32::WHITE);
        let c = resolve_slider_colors(&dark, Style::Nova);
        assert_eq!(c.thumb_fill, Color32::WHITE);
    }

    #[test]
    fn nova_uses_smaller_thumb_ring_border_and_thinner_track() {
        let p = palette(crate::theme::Mode::Light);
        // nova thumb `size-3 border border-ring bg-white`
        let c = resolve_slider_colors(&p, Style::Nova);
        assert_eq!(c.thumb_stroke.color, p.ring);
        // base keeps `border-primary`
        let base = resolve_slider_colors(&p, Style::Vega);
        assert_eq!(base.thumb_stroke.color, p.primary);
        // geometry: base size-4/h-1.5, nova size-3/h-1
        assert_eq!(thumb_size(Style::Vega), 16.0);
        assert_eq!(thumb_size(Style::Nova), 12.0);
        assert_eq!(track_height(Style::Vega), 6.0);
        assert_eq!(track_height(Style::Nova), 4.0);
        // halo: ring-4 base, ring-3 nova
        assert_eq!(ring_width(Style::Vega), 4.0);
        assert_eq!(ring_width(Style::Nova), 3.0);
        // vertical column: min-h-44 (176) base, min-h-40 (160) nova
        assert_eq!(vertical_min_height(Style::Vega), 176.0);
        assert_eq!(vertical_min_height(Style::Nova), 160.0);
    }

    #[test]
    fn hit_area_is_thumb_box_under_base_and_28px_under_nova() {
        // §3.5: `after:-inset-2` exists only in the nova thumb rule.
        assert_eq!(thumb_hit_size(Style::Vega), 16.0);
        assert_eq!(thumb_hit_size(Style::Nova), 28.0);
    }

    #[test]
    fn shadow_sm_survives_under_nova_via_layer_resolution() {
        // §2.4 trap 1: the nova thumb rule never mentions shadow.
        assert_eq!(
            resolve_shadow(ShadowSize::Sm, Style::Nova, ShadowSlot::SliderThumb),
            ShadowSize::Sm
        );
        assert_eq!(
            resolve_shadow(ShadowSize::Sm, Style::Vega, ShadowSlot::SliderThumb),
            ShadowSize::Sm
        );
        assert_eq!(ShadowSize::Sm.layers().len(), 2);
    }

    #[test]
    fn ring_rules_follow_the_state_matrix() {
        // Base: hover/focus ring, no active-only ring (§4.3).
        assert!(thumb_ring_visible(Style::Vega, true, false, false, false));
        assert!(thumb_ring_visible(Style::Vega, false, false, true, false));
        assert!(!thumb_ring_visible(Style::Vega, false, true, false, false));
        assert!(!thumb_ring_visible(Style::Vega, false, false, false, false));
        // Nova: `active:ring-3` keeps the ring while dragging (§4.3).
        assert!(thumb_ring_visible(Style::Nova, false, true, false, false));
        assert!(thumb_ring_visible(Style::Nova, true, false, false, false));
        assert!(thumb_ring_visible(Style::Nova, false, false, true, false));
        assert!(!thumb_ring_visible(Style::Nova, false, false, false, false));
        // Disabled: no ring in any state, either style (§4.4).
        assert!(!thumb_ring_visible(Style::Vega, true, true, true, true));
        assert!(!thumb_ring_visible(Style::Nova, true, true, true, true));
    }

    #[test]
    fn ring_stroke_radius_sits_on_the_border_box_edge_sld_vis_01() {
        // SLD-VIS-01: egui 0.37 renders a `circle_stroke` band entirely
        // OUTSIDE the given radius ([r, r + width]), so the stroke radius
        // must equal the thumb radius for the halo to start exactly at the
        // border-box edge (Tailwind outset `box-shadow` semantics). The
        // rendered-band evidence is in the S9 report; this pins the input.
        assert_eq!(ring_stroke_radius(Style::Nova), 6.0);
        assert_eq!(ring_stroke_radius(Style::Vega), 8.0);
        assert_eq!(
            ring_stroke_radius(Style::Nova),
            thumb_size(Style::Nova) / 2.0
        );
        assert_eq!(
            ring_stroke_radius(Style::Vega),
            thumb_size(Style::Vega) / 2.0
        );
        // The band's outer edge: radius + ring width (nova [6, 9], base [8, 12]).
        assert_eq!(
            ring_stroke_radius(Style::Nova) + ring_width(Style::Nova),
            9.0
        );
        assert_eq!(
            ring_stroke_radius(Style::Vega) + ring_width(Style::Vega),
            12.0
        );
    }

    #[test]
    fn disabled_fade_applies_once_to_every_part() {
        // §4.4 (N8): one 50% application to every colour; a second
        // application to the thumb (25% effective) is forbidden.
        let p = palette(crate::theme::Mode::Light);
        let faded = resolve_slider_colors(&p, Style::Vega).faded(0.5);
        assert_eq!(faded.thumb_fill, color_alpha(Color32::WHITE, 0.5));
        assert_eq!(faded.track, color_alpha(p.muted, 0.5));
        assert_eq!(faded.range, color_alpha(p.primary, 0.5));
        assert_eq!(faded.thumb_stroke.color, color_alpha(p.primary, 0.5));
        // The forbidden double fade would halve the alpha again.
        assert_ne!(
            faded.thumb_fill,
            color_alpha(color_alpha(Color32::WHITE, 0.5), 0.5)
        );
        // Fading by 1.0 (the enabled path) leaves colours untouched.
        assert_eq!(
            resolve_slider_colors(&p, Style::Vega).faded(1.0).thumb_fill,
            Color32::WHITE
        );
    }

    #[test]
    fn disabled_overpainted_slots_composite_opaquely_sld_vis_02() {
        // SLD-VIS-02: the Range and the thumb fill paint OVER the faded
        // Track, so their disabled colours must be the OPAQUE group-fade
        // composites against the panel background — translucent paint
        // lets the Track show through (dark Range 127 vs 120, fill band
        // ~46 levels). The rendered-band evidence is in the S9 report;
        // this pins the composite values.
        for mode in [crate::theme::Mode::Light, crate::theme::Mode::Dark] {
            let p = palette(mode);
            let bg = p.background;
            for slot in [p.primary, Color32::WHITE] {
                let faded = color_alpha(slot, 0.5);
                let composite = disabled_group_fade(faded, bg);
                // Opaque: the discriminating property vs the old
                // translucent paint (alpha 128).
                assert_eq!(composite.a(), 255);
                assert_ne!(composite, faded);
                // Within a rounding level of the direct 50/50 mix.
                let mix = |s: u8, b: u8| (0.5 * s as f32 + 0.5 * b as f32 + 0.5) as u8;
                for (got, want) in [
                    (composite.r(), mix(slot.r(), bg.r())),
                    (composite.g(), mix(slot.g(), bg.g())),
                    (composite.b(), mix(slot.b(), bg.b())),
                ] {
                    assert!((got as i16 - want as i16).abs() <= 1, "{got} vs {want}");
                }
            }
        }
        // Exact S5 group-fade composites for the default preset (the
        // values S5/S10 judge in captures): Range 139 light / 120 dark,
        // thumb fill 255 light / 133 dark.
        let light = palette(crate::theme::Mode::Light);
        let dark = palette(crate::theme::Mode::Dark);
        let lr = disabled_group_fade(color_alpha(light.primary, 0.5), light.background);
        assert_eq!((lr.r(), lr.g(), lr.b()), (139, 139, 139));
        let dr = disabled_group_fade(color_alpha(dark.primary, 0.5), dark.background);
        assert_eq!((dr.r(), dr.g(), dr.b()), (120, 120, 120));
        let lf = disabled_group_fade(color_alpha(Color32::WHITE, 0.5), light.background);
        assert_eq!((lf.r(), lf.g(), lf.b()), (255, 255, 255));
        let df = disabled_group_fade(color_alpha(Color32::WHITE, 0.5), dark.background);
        assert_eq!((df.r(), df.g(), df.b()), (133, 133, 133));
    }

    #[test]
    fn restyle_snaps_track_range_fill_and_stroke_sld_anm_01() {
        // SLD-ANM-01: on a theme-switch restyle the track, Range, thumb
        // fill and thumb stroke must resolve to the new theme's endpoint
        // colours IMMEDIATELY — no transition classes cover those parts
        // (spec §6 + ruling 6: `transition-[color,box-shadow]` covers
        // text colour + box-shadow only). Only the ring animates (via
        // `transition_alpha`, untouched here). The pin drives `show()`
        // through a light→dark flip and inspects the painted shapes on
        // the first post-flip frame AND one mid-leg frame later: every
        // slot must already equal the dark endpoint exactly. Under the
        // old endpoint animator the flip frame still showed the light
        // endpoint (leg start) and the mid-leg frame an OKLab
        // intermediate, so both asserts are red before the fix.
        use crate::theme::{Mode, Preset, ShadcnTheme};
        let ctx = egui::Context::default();
        Preset::default().install(&ctx); // light, default (nova) preset
        let mut v = 40.0f32;
        let show_at = |ctx: &egui::Context, v: &mut f32, time: f64| {
            let input = egui::RawInput {
                time: Some(time),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_anm01_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(v).min(0.0).max(100.0).id_salt("anm01").show(ui);
            });
            // No renderer consumes the texture deltas in a harness test;
            // dropping them unapplied panics (epaint TexturesDelta guard).
            out.textures_delta.clear();
            out
        };
        // Frame 1 (light): settle every slot at the light endpoints.
        let _ = show_at(&ctx, &mut v, 1.0);
        // Flip to dark, then probe the flip frame (t + 75ms: mid-leg for
        // a 150ms ease) and a settled frame (t + 150ms).
        ShadcnTheme::set_mode(&ctx, Mode::Dark);
        let dark = palette(Mode::Dark);
        let style = Preset::default().build(Mode::Dark).style;
        let want = resolve_slider_colors(&dark, style);
        for time in [1.075, 1.15] {
            let out = show_at(&ctx, &mut v, time);
            let mut track_fill = None;
            let mut range_fill = None;
            let mut thumb_fill = None;
            let mut stroke_color = None;
            for clipped in &out.shapes {
                match &clipped.shape {
                    egui::Shape::Rect(rs) => {
                        // Track: full-width bar; Range: same height, the
                        // 40% span from the Track start to the thumb.
                        // (Shadow layers are thumb-sized rects, never a
                        // full-width 4px bar.)
                        if (rs.rect.height() - track_height(style)).abs() < 0.01 {
                            if rs.rect.width() > 150.0 {
                                track_fill = Some(rs.fill);
                            } else if rs.rect.width() > 10.0 {
                                range_fill = Some(rs.fill);
                            }
                        }
                    }
                    egui::Shape::Circle(cs) => {
                        if (cs.radius - thumb_size(style) / 2.0).abs() < 0.01 {
                            thumb_fill = Some(cs.fill);
                        } else if (cs.radius - (thumb_size(style) / 2.0 - 0.5)).abs() < 0.01 {
                            stroke_color = Some(cs.stroke.color);
                        }
                    }
                    _ => {}
                }
            }
            assert_eq!(track_fill, Some(want.track), "t={time}: track must snap");
            assert_eq!(range_fill, Some(want.range), "t={time}: Range must snap");
            assert_eq!(
                thumb_fill,
                Some(want.thumb_fill),
                "t={time}: fill must snap"
            );
            assert_eq!(
                stroke_color,
                Some(want.thumb_stroke.color),
                "t={time}: stroke must snap"
            );
        }
    }

    #[test]
    fn consecutive_arrows_stay_on_the_focused_thumb_sld_int_04() {
        // SLD-INT-04 (horizontal axis): consecutive arrow presses must
        // keep acting on the focused thumb, with NO re-acquire between
        // presses. Mechanism (egui 0.37 `Focus::begin_pass` /
        // `end_pass`, `memory/mod.rs`): an unmodified arrow press on a
        // focused widget whose `EventFilter` does not claim the arrow
        // sets a cardinal `focus_direction`, and `end_pass` then moves
        // focus to the spatially nearest widget in that direction —
        // AFTER the UI pass already stepped the value. So the first
        // arrow steps the right thumb, the ring fades (focus moved),
        // and the next arrows act on whatever focus landed on
        // (misroute) or nowhere (drop). Home/End/PageUp/Shift+arrows
        // never set a focus direction, which is why S6 saw them keep
        // the ring. The fix claims both arrow axes via
        // `set_focus_lock_filter` (the stock `slider_core::keyboard_steps`
        // pattern), so `begin_pass` leaves focus alone. This pin drives
        // two side-by-side sliders: the right-hand one sits inside the
        // Right focus cone of the left-hand thumb, so an unclaimed
        // arrow demonstrably misroutes (the S6 probe-2 shape). Focus is
        // granted via `request_focus` (the rendered probes cover the
        // real Tab path); two settle frames let the filter latch
        // (`set_focus_lock_filter` requires two consecutive focused
        // frames). RED before the fix: focus moves to the right-hand
        // slider at the end of the first arrow pass, so the target
        // stalls at 51 and the neighbour steps instead.
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut a = 50.0f32;
        let mut b = 50.0f32;

        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame =
            |ctx: &egui::Context, a: &mut f32, b: &mut f32, time: f64, events: Vec<Event>| {
                let input = egui::RawInput {
                    time: Some(time),
                    events,
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ctx| {
                    let ui_a = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("slider_int04_a"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                    );
                    Slider::new(a)
                        .min(0.0)
                        .max(100.0)
                        .step(1.0)
                        .id_salt("int04-a")
                        .show(ui_a);
                    let ui_b = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("slider_int04_b"),
                        egui::UiBuilder::new().max_rect(Rect::from_min_size(
                            Pos2::new(300.0, 0.0),
                            Vec2::new(200.0, 24.0),
                        )),
                    );
                    Slider::new(b)
                        .min(0.0)
                        .max(100.0)
                        .step(1.0)
                        .id_salt("int04-b")
                        .show(ui_b);
                });
                out.textures_delta.clear();
            };

        // Frame 1: register both sliders.
        run_frame(&ctx, &mut a, &mut b, 1.0, vec![]);
        // Focus the left-hand thumb directly (the rendered probes cover
        // the Tab path); the mechanism under test is identical however
        // focus was gained. The thumb id chain mirrors `show()`: the Ui
        // scope id, the salt, "thumb", and the usize index.
        let thumb_a = egui::Id::unique("slider_int04_a")
            .with("int04-a")
            .with("thumb")
            .with(0usize);
        ctx.memory_mut(|m| m.request_focus(thumb_a));
        // Two settle frames: the focus-lock filter latches only after
        // two consecutive focused frames.
        run_frame(&ctx, &mut a, &mut b, 1.1, vec![]);
        run_frame(&ctx, &mut a, &mut b, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb_a));

        // Five consecutive ArrowRight presses, NO re-acquire between them.
        for k in 1..=5 {
            let t = 1.2 + f64::from(k) * 0.2;
            run_frame(
                &ctx,
                &mut a,
                &mut b,
                t,
                vec![key_event(Key::ArrowRight, true)],
            );
            assert_eq!(a, 50.0 + k as f32, "press {k}: target must step");
            assert_eq!(b, 50.0, "press {k}: neighbour must not move");
            assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(thumb_a),
                "press {k}: focus must stay on the thumb"
            );
            run_frame(
                &ctx,
                &mut a,
                &mut b,
                t + 0.1,
                vec![key_event(Key::ArrowRight, false)],
            );
        }

        // Tab still moves focus away (the filter claims arrows only).
        run_frame(&ctx, &mut a, &mut b, 2.5, vec![key_event(Key::Tab, true)]);
        assert_ne!(ctx.memory(|m| m.focused()), Some(thumb_a));
    }

    #[test]
    fn consecutive_vertical_arrows_stay_on_the_focused_thumb_sld_int_04() {
        // SLD-INT-04 (vertical axis): the same mechanism as the
        // horizontal pin, with the neighbour BELOW the target so an
        // unclaimed ArrowDown demonstrably misroutes. Pins the
        // `vertical_arrows` half of the filter: Up/Down step in both
        // orientations (spec §8.1, W-3), so both axes must be claimed.
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut a = 50.0f32;
        let mut b = 50.0f32;

        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame =
            |ctx: &egui::Context, a: &mut f32, b: &mut f32, time: f64, events: Vec<Event>| {
                let input = egui::RawInput {
                    time: Some(time),
                    events,
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ctx| {
                    let ui_a = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("slider_int04_v_a"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                    );
                    Slider::new(a)
                        .min(0.0)
                        .max(100.0)
                        .step(1.0)
                        .id_salt("int04-va")
                        .show(ui_a);
                    let ui_b = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("slider_int04_v_b"),
                        egui::UiBuilder::new().max_rect(Rect::from_min_size(
                            Pos2::new(0.0, 100.0),
                            Vec2::new(200.0, 24.0),
                        )),
                    );
                    Slider::new(b)
                        .min(0.0)
                        .max(100.0)
                        .step(1.0)
                        .id_salt("int04-vb")
                        .show(ui_b);
                });
                out.textures_delta.clear();
            };

        run_frame(&ctx, &mut a, &mut b, 1.0, vec![]);
        let thumb_a = egui::Id::unique("slider_int04_v_a")
            .with("int04-va")
            .with("thumb")
            .with(0usize);
        ctx.memory_mut(|m| m.request_focus(thumb_a));
        run_frame(&ctx, &mut a, &mut b, 1.1, vec![]);
        run_frame(&ctx, &mut a, &mut b, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb_a));

        // Five consecutive ArrowDown presses, NO re-acquire between them.
        for k in 1..=5 {
            let t = 1.2 + f64::from(k) * 0.2;
            run_frame(
                &ctx,
                &mut a,
                &mut b,
                t,
                vec![key_event(Key::ArrowDown, true)],
            );
            assert_eq!(a, 50.0 - k as f32, "press {k}: target must step");
            assert_eq!(b, 50.0, "press {k}: neighbour must not move");
            assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(thumb_a),
                "press {k}: focus must stay on the thumb"
            );
            run_frame(
                &ctx,
                &mut a,
                &mut b,
                t + 0.1,
                vec![key_event(Key::ArrowDown, false)],
            );
        }
    }

    #[test]
    fn click_focuses_the_thumb_sld_int_03() {
        // SLD-INT-03 (click): a pointer press on a thumb must focus it
        // (Radix: pointerdown focuses the thumb manually), so the ring
        // stays after the pointer leaves and arrow keys act on it.
        // Mechanism (egui 0.37 `interact`/`get_response`): `interact`
        // NEVER grants focus on a pointer click — it only surrenders
        // focus when the pointer presses elsewhere — so the widget
        // must call `request_focus` itself. The clicked-block shape
        // used by button/checkbox/toggle CANNOT work here: the thumb
        // uses `Sense::drag()`, which senses drags but not clicks, so
        // `clicked()` is never true for it (`interaction.rs`:
        // `potential_click_id` comes from `hits.click` only). The fix
        // requests focus on the drag path instead: for a drag-only
        // widget `dragged()` is true on the press frame itself
        // (`interaction.rs`: "just sensitive to drags, so we can mark
        // it as dragged right away"), so gating on `dragged()` covers
        // pointerdown AND every drag frame. This pin asserts focus on
        // the PRESS frame — before any release — which a clicked-only
        // shape could never satisfy. RED before the fix: focus stays
        // None after the press.
        use egui::{Event, Key, Modifiers, PointerButton};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut value = 50.0f32;

        let button = |pos: Pos2, pressed: bool| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        };
        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame = |ctx: &egui::Context, value: &mut f32, time: f64, events: Vec<Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_int03_click"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(value)
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .id_salt("int03-click")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        // Frame 1: register the slider. Thumb centre at value 50 is
        // (100, 12): the track spans the full 200px rect width.
        run_frame(&ctx, &mut value, 1.0, vec![]);
        let thumb = egui::Id::unique("slider_int03_click")
            .with("int03-click")
            .with("thumb")
            .with(0usize);
        assert_eq!(ctx.memory(|m| m.focused()), None);

        // Hover, then press: focus must land on the PRESS frame.
        run_frame(
            &ctx,
            &mut value,
            1.1,
            vec![Event::PointerMoved(Pos2::new(100.0, 12.0))],
        );
        run_frame(
            &ctx,
            &mut value,
            1.2,
            vec![button(Pos2::new(100.0, 12.0), true)],
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(thumb),
            "press on the thumb must focus it (pointerdown focus)"
        );
        // Release in place: focus stays, value unchanged (a click
        // grabs the thumb in place).
        run_frame(
            &ctx,
            &mut value,
            1.3,
            vec![button(Pos2::new(100.0, 12.0), false)],
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
        assert_eq!(value, 50.0);

        // Two settle frames: the INT-04 focus-lock filter latches only
        // after two consecutive focused frames.
        run_frame(&ctx, &mut value, 1.4, vec![]);
        run_frame(&ctx, &mut value, 1.5, vec![]);

        // Arrows now act on the clicked thumb, with NO re-acquire.
        run_frame(
            &ctx,
            &mut value,
            1.6,
            vec![key_event(Key::ArrowRight, true)],
        );
        assert_eq!(value, 51.0, "Right after click must step the thumb");
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
        run_frame(
            &ctx,
            &mut value,
            1.7,
            vec![key_event(Key::ArrowRight, false)],
        );
        run_frame(
            &ctx,
            &mut value,
            1.8,
            vec![key_event(Key::ArrowRight, true)],
        );
        assert_eq!(value, 52.0, "second Right must stay on the thumb");
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
    }

    #[test]
    fn drag_focuses_the_dragged_thumb_sld_int_03() {
        // SLD-INT-03 (drag): dragging a thumb must focus it, and the
        // focus must be present DURING the drag — before any release.
        // A clicked-block shape could never satisfy this: a drag
        // release carries no `Click` (`is_decidedly_dragging`), and
        // the drag-only thumb never reports `clicked()` at all (see
        // the click pin). RED before the fix: focus stays None
        // throughout the drag and the post-drag arrows go nowhere.
        use egui::{Event, Key, Modifiers, PointerButton};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut value = 50.0f32;

        let button = |pos: Pos2, pressed: bool| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        };
        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame = |ctx: &egui::Context, value: &mut f32, time: f64, events: Vec<Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_int03_drag"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(value)
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .id_salt("int03-drag")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run_frame(&ctx, &mut value, 1.0, vec![]);
        let thumb = egui::Id::unique("slider_int03_drag")
            .with("int03-drag")
            .with("thumb")
            .with(0usize);

        // Press on the thumb: focused on the press frame.
        run_frame(
            &ctx,
            &mut value,
            1.1,
            vec![button(Pos2::new(100.0, 12.0), true)],
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
        // Drag +30px: value moves to 65, focus present MID-DRAG
        // (before any release — the clicked-shape discriminator).
        run_frame(
            &ctx,
            &mut value,
            1.2,
            vec![Event::PointerMoved(Pos2::new(130.0, 12.0))],
        );
        assert!(
            (value - 65.0).abs() < 0.01,
            "drag must move to 65, got {value}"
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(thumb),
            "focus must be present during the drag"
        );
        // Release: focus stays on the dragged thumb.
        run_frame(
            &ctx,
            &mut value,
            1.3,
            vec![button(Pos2::new(130.0, 12.0), false)],
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
        assert!((value - 65.0).abs() < 0.01);

        // Two settle frames for the INT-04 filter latch.
        run_frame(&ctx, &mut value, 1.4, vec![]);
        run_frame(&ctx, &mut value, 1.5, vec![]);

        // Arrows act on the dragged thumb, with NO re-acquire.
        run_frame(
            &ctx,
            &mut value,
            1.6,
            vec![key_event(Key::ArrowRight, true)],
        );
        assert_eq!(value, 66.0, "Right after drag must step the thumb");
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
        run_frame(
            &ctx,
            &mut value,
            1.7,
            vec![key_event(Key::ArrowRight, false)],
        );
        run_frame(&ctx, &mut value, 1.8, vec![key_event(Key::ArrowLeft, true)]);
        assert_eq!(value, 65.0, "Left must stay on the dragged thumb");
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
    }

    #[test]
    fn track_press_focuses_the_nearest_thumb_sld_int_03() {
        // SLD-INT-03 (track press): pressing the track grabs the
        // nearest thumb and moves it to the pointer (W-6a); the Radix
        // re-focus rule (every user-driven value change re-focuses the
        // changed thumb) requires the grabbed thumb to end up focused.
        // Press at (150, 12): outside the thumb hit rect, on the track.
        // RED before the fix: the value moves to 75 but focus stays None.
        use egui::{Event, Key, Modifiers, PointerButton};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut value = 50.0f32;

        let button = |pos: Pos2, pressed: bool| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        };
        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame = |ctx: &egui::Context, value: &mut f32, time: f64, events: Vec<Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_int03_track"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(value)
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .id_salt("int03-track")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run_frame(&ctx, &mut value, 1.0, vec![]);
        let thumb = egui::Id::unique("slider_int03_track")
            .with("int03-track")
            .with("thumb")
            .with(0usize);

        run_frame(
            &ctx,
            &mut value,
            1.1,
            vec![button(Pos2::new(150.0, 12.0), true)],
        );
        assert!(
            (value - 75.0).abs() < 0.01,
            "track press must move to 75, got {value}"
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(thumb),
            "track press must focus the grabbed thumb"
        );
        run_frame(
            &ctx,
            &mut value,
            1.2,
            vec![button(Pos2::new(150.0, 12.0), false)],
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));

        run_frame(&ctx, &mut value, 1.3, vec![]);
        run_frame(&ctx, &mut value, 1.4, vec![]);
        run_frame(
            &ctx,
            &mut value,
            1.5,
            vec![key_event(Key::ArrowRight, true)],
        );
        assert_eq!(value, 76.0, "Right after track press must step");
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
    }

    #[test]
    fn disabled_thumb_never_focuses_on_pointer_press_sld_int_03() {
        // Disabled sliders take no focus and no input (spec §8.4,
        // W-10): the hover-only sense never reports a drag, so no
        // focus request fires. Press + release on the thumb centre:
        // focus stays None, the value is unchanged, arrows stay inert.
        use egui::{Event, Key, Modifiers, PointerButton};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut value = 50.0f32;

        let button = |pos: Pos2, pressed: bool| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::default(),
        };
        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame = |ctx: &egui::Context, value: &mut f32, time: f64, events: Vec<Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_int03_disabled"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(value)
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .disabled(true)
                    .id_salt("int03-disabled")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run_frame(&ctx, &mut value, 1.0, vec![]);
        run_frame(
            &ctx,
            &mut value,
            1.1,
            vec![button(Pos2::new(100.0, 12.0), true)],
        );
        assert_eq!(ctx.memory(|m| m.focused()), None);
        run_frame(
            &ctx,
            &mut value,
            1.2,
            vec![Event::PointerMoved(Pos2::new(130.0, 12.0))],
        );
        assert_eq!(ctx.memory(|m| m.focused()), None);
        run_frame(
            &ctx,
            &mut value,
            1.3,
            vec![button(Pos2::new(130.0, 12.0), false)],
        );
        assert_eq!(ctx.memory(|m| m.focused()), None);
        assert_eq!(value, 50.0);
        run_frame(
            &ctx,
            &mut value,
            1.4,
            vec![key_event(Key::ArrowRight, true)],
        );
        assert_eq!(value, 50.0, "arrows must stay inert when disabled");
    }

    #[test]
    fn tab_walk_lands_only_on_thumbs_no_gap_stops_sld_int_05() {
        // SLD-INT-05: every Tab stop in a slider must be a thumb with a
        // visible ring (spec §8.1/§8.5, W-9: every thumb is a Tab stop
        // in value order, no roving model). The track id must NOT be a
        // Tab stop: nothing paints a ring for it and no key handling
        // reads it, so a Tab landing on it is a silent gap stop that
        // shows nothing and does nothing. This pin Tab-walks one
        // single-thumb slider and one range slider and NAMES every
        // landing; on the unfixed tree the landings include the track
        // ids, which is the empirical proof of the gap-stop mechanism
        // (which id holds focus at a gap stop, and where it sits in
        // the Tab order). Focus is read after a settle frame: a wrap
        // landing is granted on the frame AFTER the Tab (the
        // give-to-next flag survives to the next pass), and a
        // Shift+Tab landing is frame-delayed by construction
        // (`id_next_frame`), so a read taken on the Tab frame itself
        // would see `None` at exactly those landings.
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut a = 50.0f32;
        let mut b = (30.0f32, 70.0f32);

        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame = |ctx: &egui::Context,
                         a: &mut f32,
                         b: &mut (f32, f32),
                         time: f64,
                         events: Vec<Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui_a = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_int05_a"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(a)
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .id_salt("int05-a")
                    .show(ui_a);
                let ui_b = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_int05_b"),
                    egui::UiBuilder::new().max_rect(Rect::from_min_size(
                        Pos2::new(0.0, 100.0),
                        Vec2::new(200.0, 24.0),
                    )),
                );
                Slider::range(b)
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .id_salt("int05-b")
                    .show(ui_b);
            });
            out.textures_delta.clear();
        };

        // Id chains mirror `show()`: scope id + salt + "track"/"thumb"
        // + the usize index.
        let scope_a = egui::Id::unique("slider_int05_a").with("int05-a");
        let scope_b = egui::Id::unique("slider_int05_b").with("int05-b");
        let track_a = scope_a.with("track");
        let thumb_a = scope_a.with("thumb").with(0usize);
        let track_b = scope_b.with("track");
        let thumb_b0 = scope_b.with("thumb").with(0usize);
        let thumb_b1 = scope_b.with("thumb").with(1usize);
        let name = |id: Option<egui::Id>| -> &'static str {
            match id {
                Some(x) if x == track_a => "track_a",
                Some(x) if x == thumb_a => "thumb_a",
                Some(x) if x == track_b => "track_b",
                Some(x) if x == thumb_b0 => "thumb_b0",
                Some(x) if x == thumb_b1 => "thumb_b1",
                Some(_) => "other",
                None => "none",
            }
        };

        // Frame 1: register both sliders. No focus yet.
        run_frame(&ctx, &mut a, &mut b, 1.0, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), None);

        // Six Tabs: two full cycles of the three thumb stops after the
        // fix (wrap included); on the unfixed tree the five-stop cycle
        // (two tracks + three thumbs) shows up inside six Tabs too.
        let mut landings: Vec<&'static str> = Vec::new();
        for k in 1..=6 {
            let t = 1.0 + f64::from(k) * 0.2;
            run_frame(&ctx, &mut a, &mut b, t, vec![key_event(Key::Tab, true)]);
            run_frame(
                &ctx,
                &mut a,
                &mut b,
                t + 0.1,
                vec![key_event(Key::Tab, false)],
            );
            landings.push(name(ctx.memory(|m| m.focused())));
        }
        assert_eq!(
            landings,
            ["thumb_a", "thumb_b0", "thumb_b1", "thumb_a", "thumb_b0", "thumb_b1"],
            "every Tab landing must be a thumb, in layout order, wrapping"
        );

        // Shift+Tab walks backwards through thumbs only: from thumb_b1
        // back to thumb_b0, thumb_a, then wrap to thumb_b1.
        let shift_tab = |pressed: bool| Event::Key {
            key: Key::Tab,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::SHIFT,
        };
        let mut back: Vec<&'static str> = Vec::new();
        for k in 1..=3 {
            let t = 2.5 + f64::from(k) * 0.2;
            run_frame(&ctx, &mut a, &mut b, t, vec![shift_tab(true)]);
            run_frame(&ctx, &mut a, &mut b, t + 0.1, vec![shift_tab(false)]);
            back.push(name(ctx.memory(|m| m.focused())));
        }
        assert_eq!(
            back,
            ["thumb_b0", "thumb_a", "thumb_b1"],
            "Shift+Tab must walk backwards through thumbs only"
        );
    }

    #[test]
    fn shift_arrows_step_ten_times_sld_int_01() {
        // SLD-INT-01: Shift+Arrow applies 10 × step (Radix `isSkipKey`:
        // `isPageKey || (shiftKey && arrowKey)` → multiplier 10), exactly
        // like PageUp/PageDown; plain arrows apply 1 × step; Ctrl/Alt
        // +Arrow applies 1 × step (Radix checks only `shiftKey`). RED
        // before the fix: Shift+Right from 1 lands on 2 — the modifier
        // was never read (`keyboard_step` took only a `Key`).
        //
        // The modifier reaches `show()` two ways, and this pin sends
        // both, mirroring a real backend: the Key event carries
        // `modifiers` (which `Focus::begin_pass` reads, so a modified
        // arrow never sets a focus direction, and which `show()` reads
        // for the `isSkipKey` rule — see the coalesced-frame pin
        // below), and an `Event::ModifiersChanged` updates
        // `InputState::modifiers` (which egui 0.37 updates ONLY from
        // `ModifiersChanged`, never from a Key event's own modifiers
        // field).
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut v = 1.0f32;

        let run_frame = |ctx: &egui::Context, v: &mut f32, time: f64, events: Vec<Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_int01"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(v)
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .id_salt("int01")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        // Frame 1: register the slider. Focus the thumb directly (the
        // rendered probes cover the Tab path); two settle frames so the
        // focus-lock filter latches (SLD-INT-04).
        run_frame(&ctx, &mut v, 1.0, vec![]);
        let thumb = egui::Id::unique("slider_int01")
            .with("int01")
            .with("thumb")
            .with(0usize);
        ctx.memory_mut(|m| m.request_focus(thumb));
        run_frame(&ctx, &mut v, 1.1, vec![]);
        run_frame(&ctx, &mut v, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));

        // Press `key` with `mods` held, assert the value + focus, release.
        let press = |ctx: &egui::Context,
                     v: &mut f32,
                     time: f64,
                     key: Key,
                     mods: Modifiers,
                     expect: f32,
                     what: &str| {
            run_frame(
                ctx,
                v,
                time,
                vec![
                    Event::ModifiersChanged(mods),
                    Event::Key {
                        key,
                        physical_key: None,
                        pressed: true,
                        repeat: false,
                        modifiers: mods,
                    },
                ],
            );
            assert_eq!(*v, expect, "{what}: value");
            assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(thumb),
                "{what}: focus must stay on the thumb"
            );
            run_frame(
                ctx,
                v,
                time + 0.1,
                vec![
                    Event::Key {
                        key,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: mods,
                    },
                    Event::ModifiersChanged(Modifiers::default()),
                ],
            );
        };

        let mut t = 1.2;
        // Shift+Arrow: 10 × step, all four directions, focus retained.
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowRight,
            Modifiers::SHIFT,
            11.0,
            "Shift+Right",
        );
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowLeft,
            Modifiers::SHIFT,
            1.0,
            "Shift+Left",
        );
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowUp,
            Modifiers::SHIFT,
            11.0,
            "Shift+Up",
        );
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowDown,
            Modifiers::SHIFT,
            1.0,
            "Shift+Down",
        );
        // Page keys: 10 × step, unchanged.
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::PageUp,
            Modifiers::default(),
            11.0,
            "PageUp",
        );
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::PageDown,
            Modifiers::default(),
            1.0,
            "PageDown",
        );
        // Ctrl/Alt+Arrow: 1 × step (Radix checks only `shiftKey`).
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowRight,
            Modifiers::CTRL,
            2.0,
            "Ctrl+Right",
        );
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowRight,
            Modifiers::ALT,
            3.0,
            "Alt+Right",
        );
        // Plain arrows: 1 × step, unchanged.
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowLeft,
            Modifiers::default(),
            2.0,
            "plain Left",
        );
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowRight,
            Modifiers::default(),
            3.0,
            "plain Right",
        );
        // Shift+Arrow clamps at the bounds, like the page keys.
        v = 95.0;
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowRight,
            Modifiers::SHIFT,
            100.0,
            "Shift+Right clamp",
        );
        v = 5.0;
        t += 0.2;
        press(
            &ctx,
            &mut v,
            t,
            Key::ArrowLeft,
            Modifiers::SHIFT,
            0.0,
            "Shift+Left clamp",
        );
    }

    #[test]
    fn shift_arrow_coalesced_frame_steps_ten_times_sld_int_01() {
        // SLD-INT-01 follow-up: the Shift state must come from the Key
        // event itself, not from `InputState::modifiers`. Under load a
        // whole synthetic press — Shift down, arrow press/release,
        // Shift up — can land in ONE frame (exactly what `xdotool key
        // shift+Right` produces at ~1 fps, and what the rendered
        // after-phase probes showed: only the first Shift+Arrow stepped
        // 10×, every later one stepped 1×, because frame-end
        // `modifiers` had already returned to "no modifiers"). The Key
        // event carries "the state of the modifier keys at the time of
        // the event" (egui 0.37 `Event::Key`), so it survives
        // coalescing — the Radix `shiftKey` equivalent. RED on the
        // `inp.modifiers.shift` reading: the burst below lands on 2.
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut v = 1.0f32;

        let run_frame = |ctx: &egui::Context, v: &mut f32, time: f64, events: Vec<Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_int01"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(v)
                    .min(0.0)
                    .max(100.0)
                    .step(1.0)
                    .id_salt("int01")
                    .show(ui);
            });
            out.textures_delta.clear();
        };

        run_frame(&ctx, &mut v, 1.0, vec![]);
        let thumb = egui::Id::unique("slider_int01")
            .with("int01")
            .with("thumb")
            .with(0usize);
        ctx.memory_mut(|m| m.request_focus(thumb));
        run_frame(&ctx, &mut v, 1.1, vec![]);
        run_frame(&ctx, &mut v, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));

        // One frame, the whole burst: Shift down, arrow press/release,
        // Shift up — frame-end modifiers are NONE, but the Key event
        // itself carried Shift.
        let key_ev = |key: Key, pressed: bool, mods: Modifiers| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: mods,
        };
        let burst = |key: Key| {
            vec![
                Event::ModifiersChanged(Modifiers::SHIFT),
                key_ev(key, true, Modifiers::SHIFT),
                key_ev(key, false, Modifiers::SHIFT),
                Event::ModifiersChanged(Modifiers::default()),
            ]
        };
        run_frame(&ctx, &mut v, 1.4, burst(Key::ArrowRight));
        assert_eq!(v, 11.0, "coalesced Shift+Right: value");
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(thumb),
            "coalesced Shift+Right: focus must stay on the thumb"
        );
        run_frame(&ctx, &mut v, 1.6, burst(Key::ArrowLeft));
        assert_eq!(v, 1.0, "coalesced Shift+Left: value");
    }

    #[test]
    fn value_fraction_conversions_round_trip_and_clamp() {
        assert_eq!(value_to_fraction(50.0, 0.0, 100.0), 0.5);
        assert_eq!(value_to_fraction(0.0, 0.0, 100.0), 0.0);
        assert_eq!(value_to_fraction(100.0, 0.0, 100.0), 1.0);
        assert_eq!(value_to_fraction(150.0, 0.0, 100.0), 1.0);
        assert_eq!(value_to_fraction(-50.0, 0.0, 100.0), 0.0);
        assert_eq!(value_to_fraction(30.0, 20.0, 40.0), 0.5);
        assert_eq!(fraction_to_value(0.5, 0.0, 100.0), 50.0);
        assert_eq!(fraction_to_value(0.0, 20.0, 40.0), 20.0);
        assert_eq!(fraction_to_value(1.5, 0.0, 100.0), 100.0);
        assert_eq!(fraction_to_value(-0.5, 0.0, 100.0), 0.0);
        // Degenerate domain: fraction defined as 0, no division by zero.
        assert_eq!(value_to_fraction(5.0, 5.0, 5.0), 0.0);
        assert_eq!(fraction_to_value(0.75, 5.0, 5.0), 5.0);
    }

    #[test]
    fn snapping_anchors_at_min_and_rounds_half_away_from_zero() {
        // §7.4: grid points are `min + k × step`, not multiples of step.
        assert_eq!(snap_value_to_grid(27.0, 20.0, 6.0), 26.0);
        assert_eq!(snap_value_to_grid(25.0, 0.0, 10.0), 30.0);
        assert_eq!(snap_value_to_grid(15.0, 0.0, 10.0), 20.0);
        assert_eq!(snap_value_to_grid(24.9, 0.0, 10.0), 20.0);
        assert_eq!(snap_value_to_grid(50.0, 0.0, 10.0), 50.0);
        // Non-positive step: no snapping (§10 choice).
        assert_eq!(snap_value_to_grid(27.3, 0.0, 0.0), 27.3);
        assert_eq!(snap_value_to_grid(27.3, 0.0, -2.0), 27.3);
    }

    // S9-C1 (gate G5): the default step is 1 ABSOLUTE (Work B correction
    // of W-1/OPEN-1), not 1% of the range. These pins drive the real
    // keyboard pipeline without an explicit `.step()`; the show()-driven
    // pin sends a real key event through `show()`.
    fn step_once_default(value: f32, min: f32, max: f32, key: Key) -> (f32, bool) {
        let mut v = value;
        let mut changed = false;
        {
            let mut slider = Slider::new(&mut v).min(min).max(max);
            slider.keyboard_step(0, key, false, &mut changed);
        }
        (v, changed)
    }

    #[test]
    fn default_step_is_one_absolute_on_0_50_s9_c1() {
        // Discriminating fixture: 1% of 50 is 0.5, but the default step
        // is 1 absolute. RED before the fix (step_size 0.5, 25 -> 25.5).
        let mut v = 0.0;
        let slider = Slider::new(&mut v).min(0.0).max(50.0);
        assert_eq!(slider.step_size(), 1.0);
        let (v, changed) = step_once_default(25.0, 0.0, 50.0, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 26.0);
    }

    #[test]
    fn default_step_is_one_absolute_nonzero_min_s9_c1() {
        // Discriminating fixture with a nonzero min: 1% of the 50-wide
        // span is 0.5; the default step is 1 absolute. RED before the fix.
        let mut v = 0.0;
        let slider = Slider::new(&mut v).min(10.0).max(60.0);
        assert_eq!(slider.step_size(), 1.0);
        let (v, changed) = step_once_default(30.0, 10.0, 60.0, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 31.0);
    }

    #[test]
    fn default_step_0_100_control_s9_c1() {
        // Control: on 0-100, 1% == 1, so this passes both before and
        // after the fix — it proves the control is non-discriminating,
        // which is why the fixtures above carry the verdict (and why no
        // demo target can discriminate: every no-step demo slider is
        // 0-100, gate G5).
        let mut v = 0.0;
        let slider = Slider::new(&mut v).min(0.0).max(100.0);
        assert_eq!(slider.step_size(), 1.0);
        let (v, changed) = step_once_default(50.0, 0.0, 100.0, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 51.0);
    }

    #[test]
    fn explicit_step_unchanged_s9_c1() {
        // Guard: an explicit `.step(5)` still steps 5, before and after.
        let mut v = 0.0;
        let slider = Slider::new(&mut v).min(0.0).max(50.0).step(5.0);
        assert_eq!(slider.step_size(), 5.0);
        let (v, changed) = step_once(25.0, 0.0, 50.0, 5.0, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 30.0);
    }

    #[test]
    fn default_step_show_path_steps_one_s9_c1() {
        // S9-C1 through the real `show()` path: a 0-50 slider with no
        // explicit step, one ArrowRight key event, 25 -> 26. RED before
        // the fix (25 -> 25.5). Mirrors the INT-01 pin's frame pattern
        // (register, request focus directly, two settle frames so the
        // focus-lock filter latches, SLD-INT-04).
        use egui::{Event, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut v = 25.0f32;

        let run_frame = |ctx: &egui::Context, v: &mut f32, time: f64, events: Vec<Event>| {
            let input = egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("slider_s9c1"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                );
                Slider::new(v).min(0.0).max(50.0).id_salt("s9c1").show(ui);
            });
            out.textures_delta.clear();
        };

        run_frame(&ctx, &mut v, 1.0, vec![]);
        let thumb = egui::Id::unique("slider_s9c1")
            .with("s9c1")
            .with("thumb")
            .with(0usize);
        ctx.memory_mut(|m| m.request_focus(thumb));
        run_frame(&ctx, &mut v, 1.1, vec![]);
        run_frame(&ctx, &mut v, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));

        run_frame(
            &ctx,
            &mut v,
            1.3,
            vec![
                Event::ModifiersChanged(Modifiers::default()),
                Event::Key {
                    key: Key::ArrowRight,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Modifiers::default(),
                },
            ],
        );
        assert_eq!(v, 26.0);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb));
    }

    #[test]
    fn set_value_absolute_takes_absolute_values_not_fractions() {
        // The N1 regression pin: passing the absolute value 51 must land
        // on 51, never clamp to `max` as a fraction would.
        let mut v = 50.0;
        let mut slider = Slider::new(&mut v).min(0.0).max(100.0).step(1.0);
        let mut changed = false;
        slider.set_value_absolute(0, 51.0, &mut changed);
        assert!(changed);
        assert_eq!(slider.read_value(0), 51.0);
        slider.set_value_absolute(0, 200.0, &mut changed);
        assert_eq!(slider.read_value(0), 100.0);
        slider.set_value_absolute(0, -20.0, &mut changed);
        assert_eq!(slider.read_value(0), 0.0);
    }

    #[test]
    fn set_value_from_fraction_snaps_and_clamps() {
        let mut v = 50.0;
        let mut slider = Slider::new(&mut v).min(0.0).max(100.0).step(10.0);
        let mut changed = false;
        slider.set_value_from_fraction(0, 0.54, &mut changed); // 54 -> snaps to 50
        assert!(!changed);
        assert_eq!(slider.read_value(0), 50.0);
        slider.set_value_from_fraction(0, 0.55, &mut changed); // 55 rounds to 60
        assert!(changed);
        assert_eq!(slider.read_value(0), 60.0);
        slider.set_value_from_fraction(0, 1.5, &mut changed); // clamps to max
        assert_eq!(slider.read_value(0), 100.0);
        slider.set_value_from_fraction(0, -0.5, &mut changed); // clamps to min
        assert_eq!(slider.read_value(0), 0.0);
        // Snapping anchors at `min`, not at 0 (§7.4).
        let mut v = 20.0;
        let mut slider = Slider::new(&mut v).min(20.0).max(40.0).step(6.0);
        let mut changed = false;
        slider.set_value_from_fraction(0, 0.35, &mut changed); // 27 -> 26
        assert_eq!(slider.read_value(0), 26.0);
    }

    #[test]
    fn range_thumbs_cannot_cross() {
        let mut pair = (20.0, 80.0);
        let mut slider = Slider::range(&mut pair).min(0.0).max(100.0);
        let mut changed = false;
        slider.set_value_from_fraction(0, 0.95, &mut changed); // lo tries to pass hi
        assert!(changed);
        assert_eq!((slider.read_value(0), slider.read_value(1)), (80.0, 80.0));
        slider.set_value_from_fraction(1, 0.05, &mut changed); // hi clamps to lo
        assert_eq!((slider.read_value(0), slider.read_value(1)), (80.0, 80.0));
    }

    // --- §7.6 discriminating fixtures: one test per row, absolute values. ---

    fn step_once(value: f32, min: f32, max: f32, step: f32, key: Key) -> (f32, bool) {
        step_once_with_shift(value, min, max, step, key, false)
    }

    fn step_once_with_shift(
        value: f32,
        min: f32,
        max: f32,
        step: f32,
        key: Key,
        shift: bool,
    ) -> (f32, bool) {
        let mut v = value;
        let mut changed = false;
        {
            let mut slider = Slider::new(&mut v).min(min).max(max).step(step);
            slider.keyboard_step(0, key, shift, &mut changed);
        }
        (v, changed)
    }

    #[test]
    fn shift_arrow_off_grid_follows_page_path_sld_int_01() {
        // Shift+Arrow selects the page-key path, including the §7.3
        // off-grid snap rule: 1.5 + 10 × 1 = 11.5 snaps to the nearest
        // grid point (12), and 21.5 − 10 × 1 = 11.5 snaps to 12. The
        // plain-arrow control keeps the adjacent-grid-point rule (W-7).
        let (v, changed) = step_once_with_shift(1.5, 0.0, 100.0, 1.0, Key::ArrowRight, true);
        assert!(changed);
        assert_eq!(v, 12.0);
        let (v, changed) = step_once_with_shift(21.5, 0.0, 100.0, 1.0, Key::ArrowLeft, true);
        assert!(changed);
        assert_eq!(v, 12.0);
        let (v, changed) = step_once(1.5, 0.0, 100.0, 1.0, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 2.0);
    }

    #[test]
    fn fixture_f1a_arrow_right_steps_up() {
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 51.0);
    }

    #[test]
    fn fixture_f1b_arrow_left_steps_down() {
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::ArrowLeft);
        assert!(changed);
        assert_eq!(v, 49.0);
    }

    #[test]
    fn fixture_f1c_home_jumps_to_min() {
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::Home);
        assert!(changed);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn fixture_f1d_end_jumps_to_max() {
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::End);
        assert!(changed);
        assert_eq!(v, 100.0);
    }

    #[test]
    fn fixture_f2a_nonzero_min_arrow_right() {
        let (v, changed) = step_once(25.0, 20.0, 40.0, 5.0, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 30.0);
    }

    #[test]
    fn fixture_f2b_nonzero_min_arrow_left() {
        let (v, changed) = step_once(25.0, 20.0, 40.0, 5.0, Key::ArrowLeft);
        assert!(changed);
        assert_eq!(v, 20.0);
    }

    #[test]
    fn fixture_f2c_nonzero_min_home_jumps_to_min_not_max() {
        // The discriminating Home case: N1-confused code clamps the
        // absolute target 20 to fraction 1.0 and jumps to `max` (40).
        let (v, changed) = step_once(25.0, 20.0, 40.0, 5.0, Key::Home);
        assert!(changed);
        assert_eq!(v, 20.0);
    }

    #[test]
    fn fixture_f2d_nonzero_min_end_jumps_to_max() {
        let (v, changed) = step_once(25.0, 20.0, 40.0, 5.0, Key::End);
        assert!(changed);
        assert_eq!(v, 40.0);
    }

    #[test]
    fn fixture_f3_fractional_step() {
        let (v, changed) = step_once(5.0, 0.0, 10.0, 0.5, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 5.5);
    }

    #[test]
    fn fixture_f4_negative_domain_arrow_left() {
        let (v, changed) = step_once(0.0, -50.0, 50.0, 10.0, Key::ArrowLeft);
        assert!(changed);
        assert_eq!(v, -10.0);
    }

    #[test]
    fn fixture_f5a_range_lo_thumb_steps_up() {
        let mut pair = (30.0, 70.0);
        let mut changed = false;
        {
            let mut slider = Slider::range(&mut pair).min(0.0).max(100.0).step(10.0);
            slider.keyboard_step(0, Key::ArrowRight, false, &mut changed);
        }
        assert!(changed);
        assert_eq!(pair, (40.0, 70.0));
    }

    #[test]
    fn fixture_f5b_stacked_range_thumb_clamps_to_neighbour() {
        // W-5: the lo thumb stops AT the hi value; no pushing, no passing.
        let mut pair = (70.0, 70.0);
        let mut changed = false;
        {
            let mut slider = Slider::range(&mut pair).min(0.0).max(100.0).step(10.0);
            slider.keyboard_step(0, Key::ArrowRight, false, &mut changed);
        }
        assert!(!changed);
        assert_eq!(pair, (70.0, 70.0));
    }

    #[test]
    fn f5b_stacked_clamp_through_show_path_sld_int_02() {
        // SLD-INT-02: the full S6 sequence through the real `show()`
        // path, with real focus machinery (`begin_pass` / `end_pass`).
        // The `keyboard_step`-level F5b fixture above passed even while
        // S6 observed [70,70] + Right -> [70,80] rendered — the value
        // model always clamped; the rendered defect was the key
        // landing on the WRONG thumb (focus migrated to the hi thumb
        // via the SLD-INT-04 mechanism, and the ring is
        // indistinguishable when the thumbs are stacked). This pin
        // drives focus + keys through `show()` and asserts BOTH the
        // values AND that focus stays on the lo thumb after every
        // press, so it discriminates the rendered defect class. Focus
        // is granted via `request_focus` (the rendered probes cover
        // the real Tab path); two settle frames let the focus-lock
        // filter latch.
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut pair = (30.0f32, 70.0f32);

        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame =
            |ctx: &egui::Context, pair: &mut (f32, f32), time: f64, events: Vec<Event>| {
                let input = egui::RawInput {
                    time: Some(time),
                    events,
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("slider_int02_f5b"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                    );
                    Slider::range(pair)
                        .min(0.0)
                        .max(100.0)
                        .step(10.0)
                        .id_salt("int02-ptr")
                        .show(ui);
                });
                out.textures_delta.clear();
            };

        // Frame 1: register the slider.
        run_frame(&ctx, &mut pair, 1.0, vec![]);
        // Focus the lo thumb directly. The thumb id chain mirrors
        // `show()`: the Ui scope id, the salt, "thumb", the index.
        let thumb_lo = egui::Id::unique("slider_int02_f5b")
            .with("int02-ptr")
            .with("thumb")
            .with(0usize);
        ctx.memory_mut(|m| m.request_focus(thumb_lo));
        // Two settle frames: the focus-lock filter latches only after
        // two consecutive focused frames.
        run_frame(&ctx, &mut pair, 1.1, vec![]);
        run_frame(&ctx, &mut pair, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb_lo));

        // The S6 chain: 4x Right stacks lo onto hi, the 5th Right must
        // clamp with NEITHER thumb moving. NO re-acquire between
        // presses (SLD-INT-04 composition).
        let expected = [
            (40.0, 70.0),
            (50.0, 70.0),
            (60.0, 70.0),
            (70.0, 70.0),
            (70.0, 70.0),
        ];
        for (k, want) in expected.iter().enumerate() {
            let t = 1.2 + (k as f64 + 1.0) * 0.2;
            run_frame(&ctx, &mut pair, t, vec![key_event(Key::ArrowRight, true)]);
            assert_eq!(pair, *want, "press {}: values must be {:?}", k + 1, want);
            assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(thumb_lo),
                "press {}: focus must stay on the lo thumb",
                k + 1
            );
            run_frame(
                &ctx,
                &mut pair,
                t + 0.1,
                vec![key_event(Key::ArrowRight, false)],
            );
        }
        // The hi thumb never moved across the whole sequence.
        assert_eq!(pair.1, 70.0);
    }

    #[test]
    fn f5b_symmetric_clamp_through_show_path_sld_int_02() {
        // SLD-INT-02, symmetric case: from the stacked state, ArrowLeft
        // on the HI thumb clamps at the lo neighbour — neither thumb
        // moves. Same `show()` + focus-machinery pattern as the F5b
        // pin above.
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut pair = (70.0f32, 70.0f32);

        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame =
            |ctx: &egui::Context, pair: &mut (f32, f32), time: f64, events: Vec<Event>| {
                let input = egui::RawInput {
                    time: Some(time),
                    events,
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("slider_int02_sym"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                    );
                    Slider::range(pair)
                        .min(0.0)
                        .max(100.0)
                        .step(10.0)
                        .id_salt("int02-sym")
                        .show(ui);
                });
                out.textures_delta.clear();
            };

        run_frame(&ctx, &mut pair, 1.0, vec![]);
        let thumb_hi = egui::Id::unique("slider_int02_sym")
            .with("int02-sym")
            .with("thumb")
            .with(1usize);
        ctx.memory_mut(|m| m.request_focus(thumb_hi));
        run_frame(&ctx, &mut pair, 1.1, vec![]);
        run_frame(&ctx, &mut pair, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb_hi));

        for k in 1..=2 {
            let t = 1.2 + f64::from(k) * 0.2;
            run_frame(&ctx, &mut pair, t, vec![key_event(Key::ArrowLeft, true)]);
            assert_eq!(pair, (70.0, 70.0), "press {k}: neither thumb may move");
            assert_eq!(
                ctx.memory(|m| m.focused()),
                Some(thumb_hi),
                "press {k}: focus must stay on the hi thumb"
            );
            run_frame(
                &ctx,
                &mut pair,
                t + 0.1,
                vec![key_event(Key::ArrowLeft, false)],
            );
        }
    }

    #[test]
    fn f5b_chain_and_symmetric_keyboard_step_sld_int_02() {
        // SLD-INT-02 at the `keyboard_step` level: the full F5a chain
        // plus the boundary press, the symmetric case, and the
        // one-sidedness guard (the hi thumb CAN still move right from
        // the stacked state — the clamp must not over-clamp).
        let mut pair = (30.0, 70.0);
        let mut changed = false;
        {
            let mut slider = Slider::range(&mut pair).min(0.0).max(100.0).step(10.0);
            let values = |slider: &Slider| (slider.read_value(0), slider.read_value(1));
            for want in [(40.0, 70.0), (50.0, 70.0), (60.0, 70.0), (70.0, 70.0)] {
                slider.keyboard_step(0, Key::ArrowRight, false, &mut changed);
                assert!(changed);
                assert_eq!(values(&slider), want);
            }
            // Boundary press: lo stays, hi must not move.
            changed = false;
            slider.keyboard_step(0, Key::ArrowRight, false, &mut changed);
            assert!(!changed);
            assert_eq!(values(&slider), (70.0, 70.0));
            // Symmetric: Left on hi from the stacked state moves nothing.
            slider.keyboard_step(1, Key::ArrowLeft, false, &mut changed);
            assert!(!changed);
            assert_eq!(values(&slider), (70.0, 70.0));
            // One-sidedness: Right on hi from the stacked state moves hi.
            slider.keyboard_step(1, Key::ArrowRight, false, &mut changed);
            assert!(changed);
            assert_eq!(values(&slider), (70.0, 80.0));
        }
        assert_eq!(pair, (70.0, 80.0));
    }

    #[test]
    fn single_thumb_arrow_steps_unchanged_sld_int_02() {
        // SLD-INT-02 guard: single-thumb behaviour is untouched by any
        // neighbour-clamp work (no neighbour exists to clamp against).
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::ArrowRight);
        assert!(changed);
        assert_eq!(v, 51.0);
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::ArrowLeft);
        assert!(changed);
        assert_eq!(v, 49.0);
    }

    #[test]
    fn home_targets_first_index_from_focused_hi_sld_c2() {
        // S9-C2 (gate G7, Work B OPEN-4): Home targets the FIRST value
        // index, NOT the focused thumb (Radix `onHomeKeyDown` ->
        // `updateValues(min, 0)`). Driven through the real `show()`
        // path with real focus machinery: focus the HI thumb, press
        // Home, and the LO thumb must move to min while hi is
        // unchanged and focus stays on hi. RED before the fix: Home
        // acted on the focused hi thumb, which then neighbour-clamped
        // to lo, yielding (30, 30).
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut pair = (30.0f32, 70.0f32);

        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame =
            |ctx: &egui::Context, pair: &mut (f32, f32), time: f64, events: Vec<Event>| {
                let input = egui::RawInput {
                    time: Some(time),
                    events,
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("slider_c2_home"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                    );
                    Slider::range(pair)
                        .min(0.0)
                        .max(100.0)
                        .step(10.0)
                        .id_salt("c2-home")
                        .show(ui);
                });
                out.textures_delta.clear();
            };

        run_frame(&ctx, &mut pair, 1.0, vec![]);
        let thumb_hi = egui::Id::unique("slider_c2_home")
            .with("c2-home")
            .with("thumb")
            .with(1usize);
        ctx.memory_mut(|m| m.request_focus(thumb_hi));
        run_frame(&ctx, &mut pair, 1.1, vec![]);
        run_frame(&ctx, &mut pair, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb_hi));

        run_frame(&ctx, &mut pair, 1.4, vec![key_event(Key::Home, true)]);
        assert_eq!(pair, (0.0, 70.0), "Home must move lo to min, hi unchanged");
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(thumb_hi),
            "Home must not move focus off the hi thumb"
        );
        run_frame(&ctx, &mut pair, 1.5, vec![key_event(Key::Home, false)]);
        assert_eq!(pair, (0.0, 70.0));
    }

    #[test]
    fn end_targets_last_index_from_focused_lo_sld_c2() {
        // S9-C2 (gate G7, Work B OPEN-4): End targets the LAST value
        // index, NOT the focused thumb (Radix `onEndKeyDown` ->
        // `updateValues(max, values.length - 1)`). Focus the LO thumb,
        // press End, and the HI thumb must move to the (on-grid) max
        // while lo is unchanged and focus stays on lo. RED before the
        // fix: End acted on the focused lo thumb, which then
        // neighbour-clamped to hi, yielding (70, 70).
        use egui::{Event, Key, Modifiers};

        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut pair = (30.0f32, 70.0f32);

        let key_event = |key: Key, pressed: bool| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::default(),
        };

        let run_frame =
            |ctx: &egui::Context, pair: &mut (f32, f32), time: f64, events: Vec<Event>| {
                let input = egui::RawInput {
                    time: Some(time),
                    events,
                    ..Default::default()
                };
                let mut out = ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("slider_c2_end"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 24.0))),
                    );
                    Slider::range(pair)
                        .min(0.0)
                        .max(100.0)
                        .step(10.0)
                        .id_salt("c2-end")
                        .show(ui);
                });
                out.textures_delta.clear();
            };

        run_frame(&ctx, &mut pair, 1.0, vec![]);
        let thumb_lo = egui::Id::unique("slider_c2_end")
            .with("c2-end")
            .with("thumb")
            .with(0usize);
        ctx.memory_mut(|m| m.request_focus(thumb_lo));
        run_frame(&ctx, &mut pair, 1.1, vec![]);
        run_frame(&ctx, &mut pair, 1.2, vec![]);
        assert_eq!(ctx.memory(|m| m.focused()), Some(thumb_lo));

        run_frame(&ctx, &mut pair, 1.4, vec![key_event(Key::End, true)]);
        assert_eq!(pair, (30.0, 100.0), "End must move hi to max, lo unchanged");
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(thumb_lo),
            "End must not move focus off the lo thumb"
        );
        run_frame(&ctx, &mut pair, 1.5, vec![key_event(Key::End, false)]);
        assert_eq!(pair, (30.0, 100.0));
    }

    #[test]
    fn end_snaps_off_grid_max_sld_c2() {
        // S9-C2 (gate G7): End lands on the SNAPPED max (Radix
        // `updateValues` snaps then clamps): 0-10 step 3 -> End = 9.
        // No demo target exists with an off-grid max, so this case is
        // pin-only. RED before the fix: End landed on the raw max
        // (10.0), and on a range it acted on the focused index.
        let (v, changed) = step_once(5.0, 0.0, 10.0, 3.0, Key::End);
        assert!(changed);
        assert_eq!(v, 9.0);
        let mut pair = (2.0, 8.0);
        let mut changed = false;
        {
            let mut slider = Slider::range(&mut pair).min(0.0).max(10.0).step(3.0);
            // Focused index is lo (0); End must still target hi (1).
            slider.keyboard_step(0, Key::End, false, &mut changed);
        }
        assert!(changed);
        assert_eq!(pair, (2.0, 9.0));
        // Home from the same off-grid domain lands on min (on-grid by
        // construction: the grid is anchored at min).
        let (v, changed) = step_once(5.0, 0.0, 10.0, 3.0, Key::Home);
        assert!(changed);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn home_end_targeting_keyboard_step_sld_c2() {
        // S9-C2 at the `keyboard_step` level: Home/End target the
        // first/last index regardless of which index is passed (the
        // focused thumb). The focused-thumb controls coincide with
        // the old behaviour and must keep passing.
        let mut pair = (30.0, 70.0);
        let mut changed = false;
        {
            let mut slider = Slider::range(&mut pair).min(0.0).max(100.0).step(10.0);
            let values = |slider: &Slider| (slider.read_value(0), slider.read_value(1));
            // Home with the HI thumb focused: lo moves, hi unchanged.
            slider.keyboard_step(1, Key::Home, false, &mut changed);
            assert!(changed);
            assert_eq!(values(&slider), (0.0, 70.0));
            // End with the LO thumb focused: hi moves, lo unchanged.
            slider.keyboard_step(0, Key::End, false, &mut changed);
            assert!(changed);
            assert_eq!(values(&slider), (0.0, 100.0));
            // Focused-thumb controls: Home on lo, End on hi.
            changed = false;
            slider.keyboard_step(0, Key::End, false, &mut changed);
            assert!(!changed);
            assert_eq!(values(&slider), (0.0, 100.0));
            changed = false;
            slider.keyboard_step(1, Key::Home, false, &mut changed);
            assert!(!changed);
            assert_eq!(values(&slider), (0.0, 100.0));
        }
        assert_eq!(pair, (0.0, 100.0));
    }

    #[test]
    fn home_end_single_thumb_guards_sld_c2() {
        // S9-C2 guards: single-thumb Home/End are unchanged (the
        // first/last index IS the only thumb). Frozen fixtures
        // F1c/F1d/F2c/F2d cover the on-grid cases; these guards pin
        // the no-op boundaries.
        let (v, changed) = step_once(0.0, 0.0, 100.0, 1.0, Key::Home);
        assert!(!changed);
        assert_eq!(v, 0.0);
        let (v, changed) = step_once(100.0, 0.0, 100.0, 1.0, Key::End);
        assert!(!changed);
        assert_eq!(v, 100.0);
        let (v, changed) = step_once(25.0, 20.0, 40.0, 5.0, Key::Home);
        assert!(changed);
        assert_eq!(v, 20.0);
        let (v, changed) = step_once(25.0, 20.0, 40.0, 5.0, Key::End);
        assert!(changed);
        assert_eq!(v, 40.0);
    }

    #[test]
    fn home_end_neighbour_boundaries_sld_c2() {
        // S9-C2 boundary cases: the neighbour clamp still applies
        // through the first/last-index targeting.
        let mut pair = (0.0, 70.0);
        let mut changed = false;
        {
            let mut slider = Slider::range(&mut pair).min(0.0).max(100.0).step(10.0);
            // Home with hi focused and lo already at min: no-op.
            // RED before the fix: Home acted on hi, collapsing the
            // pair to (0, 0).
            slider.keyboard_step(1, Key::Home, false, &mut changed);
            assert!(!changed);
            assert_eq!((slider.read_value(0), slider.read_value(1)), (0.0, 70.0));
        }
        let mut pair = (95.0, 95.0);
        let mut changed = false;
        {
            // Snapped max (90) sits BELOW lo (95): the neighbour clamp
            // wins and End is a no-op. Guard: passes before and after
            // (the old code also no-ops here, via the lo-side clamp).
            let mut slider = Slider::range(&mut pair).min(0.0).max(100.0).step(30.0);
            slider.keyboard_step(0, Key::End, false, &mut changed);
            assert!(!changed);
            assert_eq!((slider.read_value(0), slider.read_value(1)), (95.0, 95.0));
        }
    }

    #[test]
    fn fixture_f6a_arrow_right_at_max_clamps_without_wrap() {
        let (v, changed) = step_once(100.0, 0.0, 100.0, 1.0, Key::ArrowRight);
        assert!(!changed);
        assert_eq!(v, 100.0);
    }

    #[test]
    fn fixture_f6b_arrow_left_at_min_clamps_without_wrap() {
        let (v, changed) = step_once(0.0, 0.0, 100.0, 1.0, Key::ArrowLeft);
        assert!(!changed);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn arrow_up_down_match_arrow_right_left() {
        // W-3: same mapping in both orientations.
        let (v, _) = step_once(50.0, 0.0, 100.0, 1.0, Key::ArrowUp);
        assert_eq!(v, 51.0);
        let (v, _) = step_once(50.0, 0.0, 100.0, 1.0, Key::ArrowDown);
        assert_eq!(v, 49.0);
    }

    #[test]
    fn page_keys_move_ten_steps_working_assumption() {
        // W-2 (OPEN-2): magnitude 10 × step, one named place.
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::PageUp);
        assert!(changed);
        assert_eq!(v, 60.0);
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::PageDown);
        assert!(changed);
        assert_eq!(v, 40.0);
        let (v, _) = step_once(95.0, 0.0, 100.0, 1.0, Key::PageUp);
        assert_eq!(v, 100.0);
        let (v, _) = step_once(5.0, 0.0, 100.0, 1.0, Key::PageDown);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn off_grid_arrows_land_on_the_adjacent_grid_point() {
        // W-7 (OPEN-7): from 53 with step 10, Right goes to 60 (not 63)
        // and Left goes to 50 (not 43).
        let (v, _) = step_once(53.0, 0.0, 100.0, 10.0, Key::ArrowRight);
        assert_eq!(v, 60.0);
        let (v, _) = step_once(53.0, 0.0, 100.0, 10.0, Key::ArrowLeft);
        assert_eq!(v, 50.0);
    }

    #[test]
    fn keyboard_step_ignores_unhandled_keys() {
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::Enter);
        assert!(!changed);
        assert_eq!(v, 50.0);
        let (v, changed) = step_once(50.0, 0.0, 100.0, 1.0, Key::A);
        assert!(!changed);
        assert_eq!(v, 50.0);
    }

    #[test]
    fn degenerate_domains_never_panic() {
        // min == max: every key is a no-op, fraction is 0 (§10).
        let (v, changed) = step_once(5.0, 5.0, 5.0, 1.0, Key::ArrowRight);
        assert!(!changed);
        assert_eq!(v, 5.0);
        let mut v = 5.0;
        let mut slider = Slider::new(&mut v).min(5.0).max(5.0);
        let mut changed = false;
        slider.set_value_from_fraction(0, 0.75, &mut changed);
        assert_eq!(slider.read_value(0), 5.0);
        // min > max: bounds are ordered (documented §10 choice).
        let (v, _) = step_once(50.0, 100.0, 0.0, 1.0, Key::Home);
        assert_eq!(v, 0.0);
        let (v, _) = step_once(50.0, 100.0, 0.0, 1.0, Key::End);
        assert_eq!(v, 100.0);
    }

    #[test]
    fn non_positive_step_clamps_to_epsilon() {
        let mut v = 0.0;
        let s = Slider::new(&mut v).step(0.0);
        assert_eq!(s.step, Some(f32::EPSILON));
        let mut v = 0.0;
        let s = Slider::new(&mut v).step(-5.0);
        assert_eq!(s.step, Some(f32::EPSILON));
    }

    #[test]
    fn nearest_thumb_index_picks_the_closest_centre() {
        assert_eq!(nearest_thumb_index(&[0.2, 0.8], 0.25), 0);
        assert_eq!(nearest_thumb_index(&[0.2, 0.8], 0.75), 1);
        assert_eq!(nearest_thumb_index(&[0.5], 0.9), 0);
        // Ties go to the lowest index (documented W-6c reading).
        assert_eq!(nearest_thumb_index(&[0.3, 0.5], 0.4), 0);
        assert_eq!(nearest_thumb_index(&[], 0.4), 0);
    }

    #[test]
    fn builders_keep_fields() {
        let mut v = 0.0;
        let s = Slider::new(&mut v)
            .min(5.0)
            .max(15.0)
            .step(1.0)
            .orientation(GroupOrientation::Vertical)
            .disabled(true)
            .id_salt("s");
        assert_eq!(s.min, 5.0);
        assert_eq!(s.max, 15.0);
        assert_eq!(s.step, Some(1.0));
        assert_eq!(s.orientation, GroupOrientation::Vertical);
        assert!(s.disabled);
        assert_eq!(s.id_salt.as_deref(), Some("s"));
    }

    #[test]
    fn vertical_slider_stretches_to_offered_height() {
        // §3.2: `h-full` — the offered 400px wins over the floor.
        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut v = 40.0;
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("slider_vertical_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 400.0))),
            );
            rect = Slider::new(&mut v)
                .min(0.0)
                .max(100.0)
                .orientation(GroupOrientation::Vertical)
                .id_salt("vertical")
                .show(ui)
                .rect;
        });
        out.textures_delta.clear();
        assert!((rect.height() - 400.0).abs() < 0.5);
        assert!((rect.width() - 24.0).abs() < 0.5);
        assert_eq!(v, 40.0);
    }

    #[test]
    fn vertical_slider_floors_at_min_height() {
        // §3.2: below the floor, the slider is `min-h-40` (160, nova
        // default preset) tall and overflows its parent as CSS would.
        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut v = 40.0;
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("slider_vertical_floor_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 100.0))),
            );
            rect = Slider::new(&mut v)
                .min(0.0)
                .max(100.0)
                .orientation(GroupOrientation::Vertical)
                .id_salt("vertical-floor")
                .show(ui)
                .rect;
        });
        out.textures_delta.clear();
        assert!((rect.height() - vertical_min_height(Style::Nova)).abs() < 0.5);
        assert_eq!(v, 40.0);
    }
}
