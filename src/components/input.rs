//! shadcn `input.tsx` — the single-line text field.
//!
//! Base recipe (`input.tsx:9-14`; class string at `input.tsx:10-12`):
//! ```text
//! h-9 w-full min-w-0 rounded-md border border-input bg-transparent px-3 py-1
//! text-base shadow-xs transition-[color,box-shadow] outline-none
//! selection:bg-primary selection:text-primary-foreground file:inline-flex
//! file:h-7 file:border-0 file:bg-transparent file:text-sm file:font-medium
//! file:text-foreground placeholder:text-muted-foreground
//! disabled:pointer-events-none disabled:cursor-not-allowed disabled:opacity-50
//! md:text-sm dark:bg-input/30
//! focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50
//! aria-invalid:border-destructive aria-invalid:ring-destructive/20
//! dark:aria-invalid:ring-destructive/40
//! ```
//!
//! Nova overrides (`.cn-input`, `style-nova.css:677-679`, inside `@apply` at
//! `:678`):
//! ```text
//! dark:bg-input/30 border-input focus-visible:border-ring
//! focus-visible:ring-ring/50 aria-invalid:ring-destructive/20
//! dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive
//! dark:aria-invalid:border-destructive/50 disabled:bg-input/50
//! dark:disabled:bg-input/80 h-8 rounded-lg border bg-transparent px-2.5 py-1
//! text-base transition-colors file:h-6 file:text-sm file:font-medium
//! focus-visible:ring-3 aria-invalid:ring-3 md:text-sm
//! ```
//!
//! LAYER: the nova rule layers over the base classes; silence keeps the base
//! value. `shadow-xs`, `outline-none`, the selection and placeholder colours,
//! `w-full`/`min-w-0`, and the disabled pointer/cursor/opacity rules all
//! survive under nova. Nova changes the geometry (`h-8`, `rounded-lg`,
//! `px-2.5`), adds the invalid rest ring (`aria-invalid:ring-3`) and the
//! disabled fills, narrows the dark invalid border to `destructive/50` (N5),
//! and replaces the transition with `transition-colors`.
//!
//! Geometry (border-box): 36px tall / 8px radius / 12px horizontal padding
//! under base, 32px / 10px / 10px under nova; the 1px border paints inside
//! the border box. In a group, non-first children drop their leading border
//! edge through a fused-stroke painter (the ring painted through a clip
//! that excludes the leading edge plus clipped corner cells; the fill
//! paints on the full rect — border removal, never a draw-rect inset).
//! There is no hover or active visual: hovering changes nothing, and
//! pressing adds nothing beyond the focus state the click itself causes.
//!
//! The underlying widget is a restyled [`egui::TextEdit`] whose theme frame
//! is replaced with an explicit [`egui::Frame`] carrying the resolved fill
//! and the padding: egui's default `TextEdit` frame paints an opaque
//! `text_edit_bg_color` fill and a state-dependent stroke (focused stroke
//! in the selection colour, `md` radius), which would defeat the
//! transparent light fill and the nova radius. The frame fill paints under
//! the glyphs (INP-VIS-02: the old post-`TextEdit` fill rect veiled them).
//! The rest of the shadcn frame (shadow, border, focus ring) is drawn
//! entirely by this component, on the border box, with the ring as a
//! 3px band immediately outside it. Hint text renders in
//! `muted-foreground` via the theme's weak text colour
//! (`placeholder:text-muted-foreground`); selection colours are overridden
//! per-widget to `primary` / `primary-foreground` (see OPEN-6 below),
//! rendering at light bg (23) / text (250) and dark bg (229) / text (23)
//! as re-derived from fresh captures (S10), and the caret renders in
//! `foreground` via the theme cursor stroke. Disabled compositing is
//! whole-element group fade (CSS `opacity` semantics): the interior slots
//! precomposite as the enabled render over the pre-opacity fill and fade
//! once over the backdrop, while the fill, ring, and shadow halve over
//! the backdrop alone — where halving IS the group fade (INP-VIS-01).
//!
//! Animation (§6) reverses by style: base `transition-[color,box-shadow]`
//! fades the ring and animates the text colour while border/fill colours
//! snap; nova `transition-colors` eases border/fill/text colours while the
//! ring snaps. Timing is the Tailwind v4 framework default (150ms,
//! cubic-bezier(0.4, 0, 0.2, 1)), labelled INTERIM per OPEN-2 — not
//! snapshot-verified. The disabled 50% compositing snaps in both styles:
//! opacity is in neither transition set, so the fade is applied after
//! animation.
//!
//! Keyboard and pointer behaviour follows the native single-line matrix
//! through `TextEdit`, verified in all four style/theme combinations
//! (S6/S10): clicks focus and place the caret; double-click selects the
//! word; triple-click selects all text (when the framework counts a
//! triple in a coalesced frame, the component restores the select-all
//! outcome — INP-INT-01); Tab never inserts a character; undo/redo and
//! word jumps match the native capability. Two observed behaviours are
//! NOT frozen spec rules: Escape and Enter clear focus (gate G4), and
//! pasted newline sequences map to exactly one space each (gate G5).
//! Under very slow rendering, click pairs straddling the framework's
//! 300ms frame-time window demote the click count before the component
//! can see a triple — a documented environmental limitation of the
//! framework's untimestamped events, not a component defect. IME
//! behaviour is unjudged (OPEN-10; no IME in the verification
//! environment).
//!
//! Proposed deviations (PROPOSED, never approved — Expert Reviewer decides):
//! D1 `:focus-visible` modality (ring on any focus, geometry unchanged),
//! D2 `data-slot="input"` dropped (no snapshot consumer), D3 the `md:text-sm`
//! size (14px) at all widths (no viewport breakpoints in egui), D8 text +
//! password types only — out-of-scope HTML types are inexpressible through
//! this API (no `type` builder) and render as plain text; `email`/`tel`/
//! `url`/`search` are visually identical to text on desktop. Limitations:
//! D4 `file:*` rules (no file-picker element), D6 ARIA (no screen-reader
//! tree; only the visual invalid consequence is implemented), D7 form
//! submission / native validation (no form model). D5 is an implementation
//! note: selection colours are per-element in the original; see OPEN-6.
//!
//! S1 decisions (frozen input spec §11): OPEN-3 — the FG-1 stretch channel
//! applies: `w-full` declares a *relative* width that resolves to the group
//! width inside a `w-fit` group, so it does not exempt the input; an
//! explicit `.width()` is a definite width and does exempt it. OPEN-5 — the
//! 160pt horizontal fallback is kept as the named [`INPUT_FALLBACK_WIDTH`]
//! port convention. OPEN-6 — selection colours are overridden per-widget:
//! the theme-level selection (`primary` at 35%, `ring` text) does not match
//! the spec, and the override is contained in the widget's own scope.
//! Focused+invalid precedence (invalid wins) is PROVISIONAL per OPEN-1.
//! No read-only builder is offered: egui ties read-only to buffer
//! mutability and exposes no builder flag for a caller-owned `String`. No
//! uncontrolled `defaultValue` wrapper is offered: `TextEdit` is inherently
//! caller-owned state and the spec does not require one.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut email = String::new();
//! let response = Input::new(&mut email)
//!     .placeholder("m@example.com")
//!     .show(ui);
//! if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
//!     // submit
//! }
//! ```

use crate::components::attachment::mix;
use crate::components::button::group;
use crate::components::button::{fuse_radius, GroupOrientation, GroupPosition};
use crate::theme::{blend, color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use crate::utils::animation::{transition_alpha, transition_color_to, Easing, TransitionSpec};
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::text::CCursorRange;
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Margin, Painter, Pos2, Rect, Response, Stroke,
    TextEdit, Ui, Vec2,
};

/// Height of the input in the base recipe (`h-9`). The nova style is 32pt
/// (`h-8`).
pub const INPUT_HEIGHT: f32 = crate::theme::tokens::spacing::CONTROL;

/// Fallback width inside horizontal rows, where no `w-full` containing block
/// supplies a width. A port convention, NOT snapshot evidence (frozen spec
/// §3.3, OPEN-5): the original declares `w-full` and the containing block
/// always supplies a width in CSS; an egui horizontal layout has no
/// containing-block width, so the port falls back to this value. Kept
/// unchanged by the S1 rebuild; S5 verified the gallery rendering (the
/// 160pt fallback renders exact in all four style/theme combinations).
pub const INPUT_FALLBACK_WIDTH: f32 = 160.0;

/// Outer height (`h-9` base recipe; nova `.cn-input` `h-8`).
pub(crate) fn input_height(style: Style) -> f32 {
    match style {
        Style::Nova => 32.0, // h-8
        _ => INPUT_HEIGHT,   // h-9
    }
}

/// Horizontal padding (`px-3` base recipe; nova `.cn-input` `px-2.5`).
pub(crate) fn padding_x(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0, // px-2.5
        _ => 12.0,           // px-3
    }
}

/// Corner radius (`rounded-md` base recipe; nova `.cn-input` `rounded-lg`).
pub(crate) fn corner_radius(style: Style, radius: &RadiusTokens) -> u8 {
    match style {
        Style::Nova => radius.lg_u8(),
        _ => radius.md_u8(),
    }
}

/// Resolved frame colors for one input state: the RULE values, before the
/// disabled 50% compositing and before animation. The fill rule is not a
/// field here: the paint site animates it as an opaque RGB endpoint plus a
/// rule alpha from [`input_fill_alpha`], which is the single source of truth
/// for the fill value. The paint site animates these targets and applies the
/// disabled fade after animation, because `opacity` is in neither style's
/// transition set (spec §6) — the fade must snap while the rule values ease
/// or snap per style.
#[derive(Clone, Copy, Debug)]
pub(crate) struct InputColors {
    stroke: Stroke,
    /// Focus/invalid ring color (alpha already applied).
    ring: Color32,
}

/// Fill rule alpha: the `input`-token alpha of the resting/disabled fill,
/// before the disabled 50% compositing. Light resting is transparent; dark
/// resting is `input/30` (`dark:bg-input/30`, both recipes). Nova adds
/// disabled fills (`disabled:bg-input/50`, `dark:disabled:bg-input/80`,
/// `style-nova.css:678`); the base recipe has no disabled fill rule, so its
/// resting fill stands and the 50% compositing alone produces the effective
/// value. Effective rendered fills (§4.6), as fractions of the token:
/// nova disabled 25% light / 40% dark, base disabled transparent light /
/// 15% dark. (The dark `input` token itself carries alpha — `oklch(1 0 0 /
/// 15%)` — so absolute alphas are token-alpha x rule-alpha x 0.5, as CSS
/// color-mix requires.)
fn input_fill_alpha(dark: bool, disabled: bool, style: Style) -> f32 {
    if disabled && matches!(style, Style::Nova) {
        if dark {
            0.8
        } else {
            0.5
        }
    } else if dark {
        0.3
    } else {
        0.0
    }
}

/// Resolves the border and ring colors for the current palette and state
/// (rule values; see [`InputColors`]). The fill rule is resolved separately
/// by [`input_fill_alpha`].
///
/// Mirrors the base recipe (`border-input`,
/// `focus-visible:border-ring focus-visible:ring-ring/50`,
/// `aria-invalid:border-destructive aria-invalid:ring-destructive/20
/// dark:aria-invalid:ring-destructive/40`) plus the nova delta
/// (`dark:aria-invalid:border-destructive/50`).
/// Focused+invalid precedence is invalid-wins, PROVISIONAL per OPEN-1: no
/// snapshot rule orders the two combinations, and the choice is pinned by a
/// test whose name says so.
pub(crate) fn resolve_input_colors(
    p: &Palette,
    dark: bool,
    focused: bool,
    invalid: bool,
    style: Style,
) -> InputColors {
    let stroke_color = if invalid {
        // N5 SETTLED: nova dark invalid border is `destructive/50`
        // (`dark:aria-invalid:border-destructive/50`, `style-nova.css:678`).
        // Light invalid and base dark invalid keep full alpha.
        if dark && matches!(style, Style::Nova) {
            color_alpha(p.destructive, 0.5)
        } else {
            p.destructive
        }
    } else if focused {
        p.ring
    } else {
        p.input
    };
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    InputColors {
        stroke: Stroke::new(1.0, stroke_color),
        ring,
    }
}

/// Whether the 3px ring paints: on focus in both styles, plus at rest when
/// invalid under nova (`aria-invalid:ring-3`, `style-nova.css:678`). The
/// base recipe gates ring WIDTH on focus (`focus-visible:ring-[3px]`), so an
/// invalid-but-unfocused base input shows the destructive border with no
/// ring (spec §4.5).
pub(crate) fn ring_visible(focused: bool, invalid: bool, style: Style) -> bool {
    focused || (invalid && matches!(style, Style::Nova))
}

/// Whole-element group fade for one interior slot under
/// `disabled:opacity-50` (spec §4.6; INP-VIS-01): the slot paints OPAQUE
/// with `0.5 * (slot over fill over backdrop) + 0.5 * backdrop`, where
/// `fill` is the animated pre-opacity (disabled-state) fill and `backdrop`
/// is `p.background`. Follows the CHK-VIS-01 (`disabled_group_fade` in
/// `checkbox.rs`) / TXT-VIS-02 (`disabled_border_color` in `textarea.rs`)
/// shape.
///
/// Per-slot alpha halving double-fades the overlap pixels: a halved stroke
/// over a halved fill read (239) nova-light where group fade requires
/// (242), and halved glyphs over the faded fill read (125)/(182) where
/// group fade requires (128)/(185). The fill itself keeps halving — it
/// paints over the backdrop alone, where halving IS the group fade — and
/// the ring and shadow keep halving for the same reason (they sit outside
/// the rect). The precomposite is exact at every coverage: coverage `c`
/// over the faded fill yields the group fade of the enabled pixel. The
/// backdrop approximation (exact on the gallery page, where the panel fill
/// is `p.background`) is the same recorded limitation as the
/// checkbox/toggle precedents.
fn disabled_group_fade(slot: Color32, fill: Color32, backdrop: Color32) -> Color32 {
    mix(backdrop, blend(blend(backdrop, fill), slot), 0.5)
}

/// Paints `shadow-xs` (`0 1px 2px rgb(0 0 0 / 0.05)`, Tailwind v4 default),
/// which the base recipe carries unconditionally (`input.tsx:10`) and nova
/// keeps by silence (LAYER). The shadow paints in every state including
/// disabled — the audit's "skipped when disabled" reading is wrong: the
/// whole element composites at 50%, so the disabled shadow is the same
/// layers with halved alphas. Mirrors the disabled-fade shape of
/// `paint_outline_shadow` in `button.rs`, with [`ShadowSlot::Input`].
fn paint_input_shadow(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::Input).layers();
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
            ShadowSlot::Input,
            dark,
        );
    }
}

/// Paints the 1px border with the leading edge omitted for non-first group
/// children (button-group spec §3.1 border removal: `border-l-0` /
/// `border-t-0`), mirroring the foundation fusing fix in [`Button`]
/// (`paint_fused_stroke`, BTG-VIS-01) and `paint_text_fused_stroke` in
/// `button_group.rs` — both private, so the shape is repeated here: the full
/// `Inside` ring is painted through a clip that excludes the leading 1px
/// column (horizontal groups) or row (vertical groups), and once more
/// clipped to each of the two leading corner cells. The three clips tile the
/// ring exactly: they share only zero-area boundaries, and the renderer's
/// scissor conversion rounds every clip edge, so no pixel is painted twice
/// — exact for translucent strokes (the 50% disabled fade), where
/// overpainting would double-blend. First/only children take the plain
/// `rect_stroke` path, identical to a standalone input.
///
/// The fill paints on the FULL rect: border removal is achieved by omitting
/// the leading edge of the stroke, NOT by insetting the draw rect (the old
/// 1px inset left the shared column painted by neither member). The corner
/// cells MUST be painted as clipped copies of the ring shape, never as 1x1
/// fill rects: a 1x1 `rect_filled` does not rasterize in egui 0.37
/// (BTG-VIS-01 corner-cell failure).
fn paint_input_fused_stroke(
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

/// A shadcn text input. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Input<'a> {
    text: &'a mut String,
    placeholder: Option<String>,
    disabled: bool,
    invalid: bool,
    password: bool,
    width: Option<f32>,
    char_limit: Option<usize>,
    id_salt: Option<String>,
}

impl<'a> Input<'a> {
    /// Creates an input bound to the given string.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            disabled: false,
            invalid: false,
            password: false,
            width: None,
            char_limit: None,
            id_salt: None,
        }
    }

    /// Shows faint hint text when the field is empty
    /// (`placeholder:text-muted-foreground`).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Disables the field (`disabled:opacity-50 disabled:pointer-events-none`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the field aria-invalid (`aria-invalid:border-destructive
    /// aria-invalid:ring-destructive/20`; under nova the ring also shows at
    /// rest, `aria-invalid:ring-3`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Masks the text (`type="password"`). The placeholder renders unmasked,
    /// as in the browser.
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Sets an explicit width. The default is `w-full` (the available width
    /// in vertical layouts; [`INPUT_FALLBACK_WIDTH`] inside horizontal
    /// rows). An explicit width is a definite width: it exempts the input
    /// from FG-1 stretch in vertical groups, the way a caller `className`
    /// width would win over the recipe in `cn()` order.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Limits the number of characters. The unit is Unicode scalar values
    /// (Rust `char`s), as counted by `egui::TextEdit::char_limit` — NOT the
    /// UTF-16 code units HTML `maxLength` counts, so astral-plane characters
    /// count as one here and two in the browser. Insertions (typing and
    /// paste) truncate at the limit.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    /// Sets a stable id salt so state (focus, cursor) persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the input.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let style = theme.style;

        let mut width = match self.width {
            Some(w) => w,
            None => {
                if ui.layout().is_vertical() {
                    ui.available_width()
                } else {
                    INPUT_FALLBACK_WIDTH
                }
            }
        };

        // Auto-detected position inside a ButtonGroup/InputGroup-like group.
        let group = group::next_child_position(ui);
        let position = group.map(|(_, pos)| pos).unwrap_or(GroupPosition::Only);
        let orientation = group
            .map(|(o, _)| o)
            .unwrap_or(GroupOrientation::Horizontal);

        // FG-1 (OPEN-3 decision, mirroring `Button::show`): in a vertical
        // group, a width-`auto` input stretches to the widest member's
        // width. `w-full` does NOT exempt the input — it declares a
        // relative width that resolves to the group width inside a `w-fit`
        // group, which is exactly the stretch outcome — but an explicit
        // `.width()` is a definite width and does. Horizontal targets never
        // stretch inputs. The in-group gate keeps a stale target from
        // stretching a standalone input, and `max` keeps the input from
        // shrinking below its natural width.
        if group.is_some() && self.width.is_none() {
            if let Some(target) = group::stretch_target(ui) {
                if target.orientation == GroupOrientation::Vertical {
                    width = width.max(target.cross);
                }
            }
        }

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input"),
        };

        // Text colour animates in BOTH styles: `color` is in the base
        // transition list (`transition-[color,box-shadow]`) and in nova's
        // `transition-colors`. The disabled 50% compositing is applied after
        // animation, so it snaps: opacity is in neither transition set.
        // Timing is INTERIM per OPEN-2 (framework defaults, not snapshot
        // truth). First paint is settled (no fade-in on mount).
        let animated_foreground = transition_color_to(
            ui.ctx(),
            id.with("text"),
            p.foreground,
            TransitionSpec::default(),
        )
        .color;
        // The disabled compositing for the glyph colour resolves after the
        // fill below: it needs the pre-opacity fill for the group-fade
        // precomposite (INP-VIS-01).

        // Spec §6: the transition PROPERTY SET differs by style, so the
        // spec each animator runs under differs by style too. Timing is
        // `TransitionSpec::default()` (150ms + EaseInOut, the Tailwind
        // v4 framework defaults), labelled INTERIM per OPEN-2. A
        // zero-duration spec snaps instantly, which is how the
        // non-transitioned property set is implemented: the animator
        // still runs every frame, so its state stays fresh across style
        // switches.
        let nova = matches!(style, Style::Nova);
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

        let radius = corner_radius(style, &theme.radius);
        let corner = fuse_radius(CornerRadius::same(radius), orientation, position);

        // Fill: the rule value is always an `input`-token tint, so the
        // RGB endpoint (opaque `p.input`, no transparent-boundary
        // caveat) rides the endpoint animator while the rule alpha
        // rides a scalar endpoint animator on the same duration — a
        // premultiplied cross-fade from transparent that `mix_oklab`
        // cannot do (a transparent endpoint contributes black). The
        // alpha leg is linear (egui's animator applies no easing) while
        // the RGB leg is eased; the difference is sub-threshold for
        // S7's duration-based budgets. Theme-switch recolours animate
        // under nova per the §6 scope note. The disabled 50%
        // compositing applies after animation, so it snaps.
        //
        // The fill is resolved BEFORE the TextEdit runs: it does not
        // depend on focus (spec §4 — only the border and ring do), and
        // the TextEdit's frame carries it so it paints UNDER the glyphs
        // (INP-VIS-02, following the TXT-VIS-01 fix shape in
        // `textarea.rs`). The old post-TextEdit `rect_filled` veiled the
        // glyphs; the frame fill is painted by the atom layout before
        // the text atoms, so it cannot veil them.
        let fill_rgb =
            transition_color_to(ui.ctx(), id.with("fill_rgb"), p.input, color_spec).color;
        let fill_alpha = ui.ctx().animate_value_with_time(
            id.with("fill_alpha"),
            input_fill_alpha(dark, self.disabled, style),
            color_spec.duration_secs,
        );
        // Kept before fading: the border, text, and hint precomposites
        // (INP-VIS-01) consume the un-faded animated value.
        let fill_animated = color_alpha(fill_rgb, fill_alpha);
        let mut fill = fill_animated;
        if self.disabled {
            fill = color_alpha(fill, 0.5);
        }

        // Disabled compositing (INP-VIS-01): the glyph colour precomposites
        // the group fade OPAQUE against the backdrop instead of halving its
        // alpha over the faded fill, which double-fades glyph cores (the
        // per-slot path read (125) nova-light where group fade requires
        // (128)). Enabled rendering is unchanged.
        let text_color = if self.disabled {
            disabled_group_fade(animated_foreground, fill_animated, p.background)
        } else {
            animated_foreground
        };

        let is_empty = self.text.is_empty();
        // 14px Inter Regular (400): `md:text-sm` at all widths (D3,
        // proposed) with no `font-medium` — only the N/A file button carries
        // weight 500. `FontId::proportional` resolves to the embedded Inter
        // 400 face.
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let edit = TextEdit::singleline(self.text)
            .id_salt(id)
            .font(font)
            .text_color(text_color)
            // Replace the theme frame with an explicit frame carrying the
            // resolved fill and the padding: egui's default `TextEdit`
            // frame paints an opaque fill and a state-dependent stroke,
            // which would defeat the transparent light fill and the nova
            // radius. The frame fill paints UNDER the glyphs (INP-VIS-02);
            // the rest of the shadcn frame (shadow, border, ring) is drawn
            // by this component, below. With an explicit frame the
            // builder's `.margin()` is ignored, so the padding is baked
            // into `inner_margin` — the text inset from the rect edge is
            // exactly `padding_x`, as before.
            .frame(Frame {
                inner_margin: Margin::symmetric(padding_x(style) as i8, 0),
                corner_radius: corner,
                fill,
                ..Frame::NONE
            })
            .desired_width(width)
            .min_size(Vec2::new(width, input_height(style)))
            .align(Align2::LEFT_CENTER)
            .password(self.password);
        let edit = if let Some(placeholder) = self.placeholder.as_deref() {
            if is_empty {
                edit.hint_text(placeholder.to_owned())
            } else {
                edit
            }
        } else {
            edit
        };
        let edit = if let Some(limit) = self.char_limit {
            edit.char_limit(limit)
        } else {
            edit
        };
        let edit = if self.disabled {
            edit.interactive(false)
        } else {
            edit
        };

        // Colors depend on focus, so resolve after the TextEdit ran. The
        // scoped spacing keeps the theme's `interact_size` floor (36pt) from
        // overriding the nova `h-8` (32pt) geometry. The scoped visuals
        // overrides are per-widget (OPEN-6): selection colours
        // (`selection:bg-primary selection:text-primary-foreground` — the
        // theme-level selection does not match) and, when disabled, the
        // hint colour precomposited at the 50% group fade
        // (`placeholder:text-muted-foreground` composited at
        // `disabled:opacity-50`; INP-VIS-01, like the text colour).
        let out = ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = input_height(style);
            let visuals = ui.visuals_mut();
            visuals.selection.bg_fill = p.primary;
            visuals.selection.stroke = Stroke::new(1.0, p.primary_foreground);
            if self.disabled {
                visuals.weak_text_color = Some(disabled_group_fade(
                    p.muted_foreground,
                    fill_animated,
                    p.background,
                ));
            }
            edit.show(ui)
        });
        let output = out.inner;
        let response = output.response.response;
        let rect = response.rect;

        // INP-INT-01: a triple-click whose three press/release pairs arrive
        // in a SINGLE frame (the normal case under slow rendering: the pairs
        // queue behind one long pass and drain together) is misread by
        // `TextEdit`. The frame's releases carry counts 1, 2, 3, so BOTH
        // `double_clicked()` and `triple_clicked()` are true (the
        // `button_*_clicked` predicates are frame-global `.any()` over the
        // release events), and `pointer_interaction` checks the double
        // branch FIRST — the word wins and the line (all text, for a
        // single-line input) is never selected. When the framework DID
        // count a triple, restore the triple outcome: select all, stored
        // back under the widget id (the documented `TextEditState`
        // mutate-and-store pattern; `TextEdit::show` already stored its
        // state, so this overwrite wins for the next frame). This changes
        // nothing on a fast machine (there the triple branch already ran
        // and the range is identical), never fires for a double (no
        // count-3 release), and keeps the framework's `!shift` gate
        // (shift-click extends the selection instead). It does NOT repair
        // count demotion from frame-time gap straddling: when the pairs
        // straddle the 300ms frame-time window the framework never counts
        // 3, and no per-event timestamp exists to recover the physical
        // click times — that residue is environmental (see the defect
        // file).
        if !self.disabled && response.triple_clicked() && !ui.input(|i| i.modifiers.shift) {
            let mut state = output.state;
            state
                .cursor
                .set_char_range(Some(CCursorRange::select_all(&output.galley)));
            state.store(ui.ctx(), response.id);
        }

        if ui.is_rect_visible(rect) {
            let focused = response.has_focus() && !self.disabled;
            let c = resolve_input_colors(&p, dark, focused, self.invalid, style);

            let painter = ui.painter();
            // `shadow-xs`, in every state (see `paint_input_shadow`).
            paint_input_shadow(painter, rect, corner, style, dark, self.disabled);

            // No fill paint here: the fill is the TextEdit frame's fill
            // (resolved above, before the TextEdit ran) and already sits
            // under the glyphs. Painting it again would double-composite
            // it over the text (the INP-VIS-02 veil). The border and ring
            // paint after the TextEdit: the border sits at the rect edge
            // and the ring outside it, so neither overlaps the glyphs.

            // Border: endpoint tracking, so focus / invalid / theme flips
            // animate under nova and snap under base. The slot never crosses
            // the transparent boundary, so `transition_color_to`'s caveat
            // does not apply. Disabled fade after animation.
            let stroke_animated =
                transition_color_to(ui.ctx(), id.with("border"), c.stroke.color, color_spec).color;
            // INP-VIS-01: when disabled, the stroke precomposites the
            // group fade (opaque) instead of halving its alpha over the
            // faded fill. Enabled rendering is unchanged.
            let stroke_color = if self.disabled {
                disabled_group_fade(stroke_animated, fill_animated, p.background)
            } else {
                stroke_animated
            };
            paint_input_fused_stroke(
                painter,
                rect,
                corner,
                Stroke::new(1.0, stroke_color),
                orientation,
                position,
            );

            // Ring: a 3px band OUTSIDE the border box, immediately adjacent
            // to the border, following the effective (possibly fused)
            // corner radius (§3.5) — painted on the unexpanded rect, since
            // the `Outside` stroke starts at the rect edge. Colour +
            // visibility both ride the SHADOW spec: the ring is a
            // `box-shadow` layer, so it fades under base and snaps under
            // nova. In a group the ring is queued (FG-3, button-group spec
            // §4 `focus-visible:z-10`): it paints in a foreground layer
            // above the abutting siblings. Disabled fade after animation.
            let ring_shown = ring_visible(focused, self.invalid, style);
            let mut ring_color =
                transition_color_to(ui.ctx(), id.with("ring"), c.ring, shadow_spec).color;
            let ring_alpha = transition_alpha(
                ui.ctx(),
                id.with("ring_alpha"),
                0.0,
                1.0,
                ring_shown,
                shadow_spec,
            )
            .alpha;
            if self.disabled {
                ring_color = color_alpha(ring_color, 0.5);
            }
            if ring_alpha > 0.0 {
                let ring_stroke = Stroke::new(3.0, color_alpha(ring_color, ring_alpha));
                if group.is_some() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Pos2;

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    fn test_ui(ctx: &egui::Context, salt: &'static str, width: f32) -> egui::Ui {
        egui::Ui::new(
            ctx.clone(),
            egui::Id::unique(salt),
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 400.0))),
        )
    }

    #[test]
    fn input_height_follows_style() {
        // Base recipe `h-9`; nova `.cn-input` `h-8`.
        assert_eq!(input_height(Style::Vega), 36.0);
        assert_eq!(input_height(Style::Nova), 32.0);
        assert_eq!(INPUT_HEIGHT, 36.0);
        assert_eq!(padding_x(Style::Nova), 10.0);
        assert_eq!(padding_x(Style::Vega), 12.0);
        assert_eq!(INPUT_FALLBACK_WIDTH, 160.0);
    }

    #[test]
    fn input_is_control_height_and_text_is_vertically_centered() {
        // The default preset is nova: `h-8` (32pt).
        let (rect_h, rect_w) = {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            let mut text = String::from("hello");
            let mut h = 0.0;
            let mut w = 0.0;
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let mut ui = test_ui(ctx, "input_test", 400.0);
                let r = Input::new(&mut text).show(&mut ui);
                h = r.rect.height();
                w = r.rect.width();
            });
            out.textures_delta.clear();
            (h, w)
        };
        assert!((rect_h - 32.0).abs() < 1.0, "nova h-8: {}", rect_h);
        assert!((rect_w - 400.0).abs() < 1.0, "w-full: {}", rect_w);
        // A non-nova (base recipe) theme keeps `h-9`.
        let ctx = egui::Context::default();
        Preset::default().with_style(Style::Vega).install(&ctx);
        let mut text = String::new();
        let mut base_h = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "input_test_base", 400.0);
            base_h = Input::new(&mut text).show(&mut ui).rect.height();
        });
        out.textures_delta.clear();
        assert!((base_h - INPUT_HEIGHT).abs() < 1.0, "base h-9: {}", base_h);
    }

    #[test]
    fn frame_colors_follow_state() {
        // Rule values (before disabled compositing and animation).
        let p = palette(Mode::Light);
        assert_eq!(input_fill_alpha(false, false, Style::Vega), 0.0);
        let c = resolve_input_colors(&p, false, false, false, Style::Vega);
        assert_eq!(c.stroke.color, p.input);
        let f = resolve_input_colors(&p, false, true, false, Style::Vega);
        assert_eq!(f.stroke.color, p.ring);
        assert_eq!(f.ring, color_alpha(p.ring, 0.5));
        let i = resolve_input_colors(&p, false, false, true, Style::Vega);
        assert_eq!(i.stroke.color, p.destructive);
        assert_eq!(i.ring, color_alpha(p.destructive, 0.2));
        let dark = palette(Mode::Dark);
        assert_eq!(input_fill_alpha(true, false, Style::Vega), 0.3);
        let di = resolve_input_colors(&dark, true, false, true, Style::Vega);
        assert_eq!(di.ring, color_alpha(dark.destructive, 0.4));
    }

    #[test]
    fn frame_fill_is_resolved_and_paints_under_glyphs() {
        // INP-VIS-02 regression pin (shape-level): the TextEdit frame must
        // carry the RESOLVED fill — not a post-TextEdit rect over the
        // glyphs — and the fill must paint BEFORE the glyphs. The resolver
        // unit tests passed while the render was wrong, so this test
        // inspects the emitted shapes instead of resolver values (the
        // TXT-VIS-01 pin shape in `textarea.rs`).
        use egui::epaint::Shape;
        fn flatten(shape: &Shape, acc: &mut Vec<Shape>) {
            match shape {
                Shape::Vec(v) => v.iter().for_each(|s| flatten(s, acc)),
                s => acc.push(s.clone()),
            }
        }
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let p = palette(mode);
            let expected_fill = if mode.is_dark() {
                color_alpha(p.input, 0.3)
            } else {
                Color32::TRANSPARENT
            };
            let mut text = String::from("hello");
            let mut rect = Rect::NOTHING;
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let mut ui = test_ui(ctx, "input_frame_pin", 400.0);
                rect = Input::new(&mut text).id_salt("pin").show(&mut ui).rect;
            });
            let mut shapes: Vec<Shape> = Vec::new();
            for clipped in &out.shapes {
                flatten(&clipped.shape, &mut shapes);
            }
            out.textures_delta.clear();
            // The frame fill rect: same rect as the widget, no stroke.
            let fill_idx = shapes.iter().position(
                |s| matches!(s, Shape::Rect(r) if r.rect == rect && r.stroke == Stroke::NONE),
            );
            let fill_idx = fill_idx.expect("frame fill rect present");
            match &shapes[fill_idx] {
                Shape::Rect(r) => assert_eq!(
                    r.fill, expected_fill,
                    "frame fill is the resolved fill ({mode:?})"
                ),
                _ => unreachable!(),
            }
            // No rect over the widget paints the raw theme `input` token:
            // the fill is the rule value (`input/30` dark), never opaque.
            for s in &shapes {
                if let Shape::Rect(r) = s {
                    assert!(
                        r.rect != rect || r.fill != p.input,
                        "no raw input-token fill over the widget ({mode:?})"
                    );
                }
            }
            // The fill paints before the first glyph.
            if let Some(text_idx) = shapes.iter().position(|s| matches!(s, Shape::Text(_))) {
                assert!(fill_idx < text_idx, "fill paints under glyphs ({mode:?})");
            }
            // Exactly one stroke-less fill rect covers the widget: the old
            // post-TextEdit `rect_filled` would be a second one.
            let fill_rects = shapes
                .iter()
                .filter(
                    |s| matches!(s, Shape::Rect(r) if r.rect == rect && r.stroke == Stroke::NONE),
                )
                .count();
            assert_eq!(fill_rects, 1, "one fill rect over the widget ({mode:?})");
        }
    }

    #[test]
    fn nova_dark_invalid_border_is_destructive_at_half_alpha() {
        // N5 SETTLED: nova dark invalid border is `destructive/50`
        // (`dark:aria-invalid:border-destructive/50`, `style-nova.css:678`).
        // Light invalid and base dark invalid keep full alpha.
        let light = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        let nd = resolve_input_colors(&dark, true, false, true, Style::Nova);
        assert_eq!(nd.stroke.color, color_alpha(dark.destructive, 0.5));
        let nl = resolve_input_colors(&light, false, false, true, Style::Nova);
        assert_eq!(nl.stroke.color, light.destructive);
        let bd = resolve_input_colors(&dark, true, false, true, Style::Vega);
        assert_eq!(bd.stroke.color, dark.destructive);
        let bl = resolve_input_colors(&light, false, false, true, Style::Vega);
        assert_eq!(bl.stroke.color, light.destructive);
    }

    #[test]
    fn disabled_fills_follow_disabled_compositing() {
        // Rule alphas (BEFORE the 50% element compositing): nova
        // `disabled:bg-input/50 dark:disabled:bg-input/80`; the base recipe
        // has no disabled fill rule, so its resting fill stands.
        let p = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        assert_eq!(input_fill_alpha(false, true, Style::Nova), 0.5);
        assert_eq!(input_fill_alpha(true, true, Style::Nova), 0.8);
        assert_eq!(input_fill_alpha(false, true, Style::Vega), 0.0);
        assert_eq!(input_fill_alpha(true, true, Style::Vega), 0.3);
        // Effective rendered fills: the rule value composited at 50% (§4.6)
        // — the paint site applies `color_alpha(fill, 0.5)` AFTER animation.
        // Asserted relative to the token's own alpha: the dark `input`
        // token itself carries alpha (`oklch(1 0 0 / 15%)`), so absolute
        // alphas are token-alpha x rule-alpha x 0.5, as CSS color-mix
        // requires. Alpha asserted within one level: double premultiplied
        // rounding can differ by a level from the single-step value.
        let eff_nd = color_alpha(color_alpha(p.input, 0.5), 0.5);
        assert!((eff_nd.a() as f32 - p.input.a() as f32 * 0.25).abs() <= 1.0);
        let eff_ndd = color_alpha(color_alpha(dark.input, 0.8), 0.5);
        assert!((eff_ndd.a() as f32 - dark.input.a() as f32 * 0.4).abs() <= 1.0);
        let eff_bdd = color_alpha(color_alpha(dark.input, 0.3), 0.5);
        assert!((eff_bdd.a() as f32 - dark.input.a() as f32 * 0.15).abs() <= 1.0);
        assert_eq!(
            color_alpha(color_alpha(p.input, 0.0), 0.5),
            Color32::TRANSPARENT
        );
        // Stroke rule values are unfaded at resolve time; the paint site
        // fades them after animation.
        let nd = resolve_input_colors(&p, false, false, false, Style::Nova);
        assert_eq!(nd.stroke.color, p.input);
    }

    fn assert_color_near(actual: Color32, expected: Color32) {
        // ±1 per channel: premultiplied rounding can differ by a level from
        // the single-step derivation.
        for (a, e) in [
            (actual.r(), expected.r()),
            (actual.g(), expected.g()),
            (actual.b(), expected.b()),
        ] {
            assert!(
                (a as i32 - e as i32).abs() <= 1,
                "channel {a} vs {e} (actual {actual:?}, expected {expected:?})"
            );
        }
    }

    #[test]
    fn disabled_group_fade_matches_differential_model() {
        // INP-VIS-01: every interior slot that paints over the fill must be
        // the group-fade precomposite — 0.5 * (slot over fill over
        // backdrop) + 0.5 * backdrop — not a halved-alpha slot over the
        // faded fill. Synthetic opaque case, exact arithmetic: backdrop
        // 200, fill 50, slot 100. Composite = 100 (opaque slot); group
        // fade = 150.
        let backdrop = Color32::from_rgb(200, 200, 200);
        let fill = Color32::from_rgb(50, 50, 50);
        let slot = Color32::from_rgb(100, 100, 100);
        assert_eq!(
            disabled_group_fade(slot, fill, backdrop),
            Color32::from_rgb(150, 150, 150)
        );
        // Transparent fill: the precomposite fades the slot over the
        // backdrop alone.
        assert_eq!(
            disabled_group_fade(slot, Color32::TRANSPARENT, backdrop),
            Color32::from_rgb(150, 150, 150)
        );
        // Translucent slot (dark `input` token shape): the precomposite
        // equals the differential formula — composite the enabled stack,
        // then fade once. This pins the wiring, not just the arithmetic.
        let translucent = color_alpha(Color32::from_rgb(255, 255, 255), 0.15);
        let fill_composite = blend(backdrop, fill);
        assert_eq!(
            disabled_group_fade(translucent, fill, backdrop),
            mix(backdrop, blend(fill_composite, translucent), 0.5)
        );
        // Real palette values: the four closing border values from the
        // defect file / ruling 11 (group fade governs; the nova-dark (41)
        // uses the disabled-state fill, per the TXT-VIS-01 amendment).
        // The nova-light (242) is the discriminating slot: the old
        // per-slot path read (239).
        let light = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        for (p, dark_mode, style, expected) in [
            (&light, false, Style::Nova, 242u8),
            (&light, false, Style::Vega, 242u8),
            (&dark, true, Style::Nova, 41u8),
            (&dark, true, Style::Vega, 33u8),
        ] {
            let fill_rule = color_alpha(p.input, input_fill_alpha(dark_mode, true, style));
            let stroke = resolve_input_colors(p, dark_mode, false, false, style)
                .stroke
                .color;
            assert_color_near(
                disabled_group_fade(stroke, fill_rule, p.background),
                Color32::from_rgb(expected, expected, expected),
            );
        }
        // Invalid+disabled borders at the defect file's group-fade values.
        for (p, dark_mode, style, expected) in [
            (&light, false, Style::Nova, (243u8, 128u8, 133u8)),
            (&dark, true, Style::Nova, (78u8, 40u8, 40u8)),
            (&dark, true, Style::Vega, (132u8, 55u8, 56u8)),
        ] {
            let fill_rule = color_alpha(p.input, input_fill_alpha(dark_mode, true, style));
            let stroke = resolve_input_colors(p, dark_mode, false, true, style)
                .stroke
                .color;
            assert_color_near(
                disabled_group_fade(stroke, fill_rule, p.background),
                Color32::from_rgb(expected.0, expected.1, expected.2),
            );
        }
        // Glyph slots: an opaque slot fades against the backdrop alone —
        // the precomposite equals `mix(backdrop, slot, 0.5)` whatever the
        // fill beneath it is. Numeric values: light text 127.5 -> 128
        // (halves round away from zero), dark text 130, light placeholder
        // 185, dark placeholder 85.5 -> 86 — all within ±1 of the S5
        // derivation (128/130, 185/86). (FND-D1: light foreground is now
        // `oklch(0% 0 0)`, so the text composite moved 132 -> 128.)
        for (p, dark_mode, expected_text, expected_hint) in
            [(&light, false, 128u8, 185u8), (&dark, true, 130u8, 86u8)]
        {
            let fill_rule = color_alpha(p.input, input_fill_alpha(dark_mode, true, Style::Nova));
            assert_eq!(
                disabled_group_fade(p.foreground, fill_rule, p.background),
                mix(p.background, p.foreground, 0.5)
            );
            assert_color_near(
                disabled_group_fade(p.foreground, fill_rule, p.background),
                Color32::from_rgb(expected_text, expected_text, expected_text),
            );
            assert_color_near(
                disabled_group_fade(p.muted_foreground, fill_rule, p.background),
                Color32::from_rgb(expected_hint, expected_hint, expected_hint),
            );
        }
        // The precomposite is opaque: what sits beneath the slot cannot
        // double-apply the fade.
        let fill_rule = color_alpha(light.input, input_fill_alpha(false, true, Style::Nova));
        let stroke = resolve_input_colors(&light, false, false, false, Style::Nova)
            .stroke
            .color;
        assert_eq!(
            disabled_group_fade(stroke, fill_rule, light.background).a(),
            255
        );
    }

    #[test]
    fn disabled_border_shape_carries_group_fade_precomposite() {
        // INP-VIS-01 regression pin (shape-level): a disabled input's
        // emitted border stroke must carry the group-fade precomposite
        // colour, in all 4 configs — not the halved-alpha stroke the old
        // path painted. Resolver unit tests alone cannot catch a paint
        // wiring regression (TXT-VIS-01 lesson), so this inspects the
        // emitted shapes.
        use egui::epaint::Shape;
        fn flatten(shape: &Shape, acc: &mut Vec<Shape>) {
            match shape {
                Shape::Vec(v) => v.iter().for_each(|s| flatten(s, acc)),
                s => acc.push(s.clone()),
            }
        }
        for mode in [Mode::Light, Mode::Dark] {
            for style in [Style::Nova, Style::Vega] {
                let preset = Preset::default().with_style(style);
                let ctx = egui::Context::default();
                preset.build(mode).install(&ctx);
                let p = preset.build(mode).palette;
                let mut text = String::new();
                let mut rect = Rect::NOTHING;
                let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                    let mut ui = test_ui(ctx, "input_border_pin", 400.0);
                    rect = Input::new(&mut text)
                        .id_salt("border-pin")
                        .disabled(true)
                        .show(&mut ui)
                        .rect;
                });
                let mut shapes: Vec<Shape> = Vec::new();
                for clipped in &out.shapes {
                    flatten(&clipped.shape, &mut shapes);
                }
                out.textures_delta.clear();
                // The border: the 1px stroke on exactly the widget rect.
                // (The frame fill rect has no stroke; shadow rects sit on
                // expanded rects; no ring paints on a disabled input.)
                let border = shapes.iter().find_map(|s| match s {
                    Shape::Rect(r) if r.rect == rect && r.stroke.width == 1.0 => {
                        Some(r.stroke.color)
                    }
                    _ => None,
                });
                let border = border.expect("disabled border stroke present");
                let fill_rule = color_alpha(p.input, input_fill_alpha(mode.is_dark(), true, style));
                let stroke = resolve_input_colors(&p, mode.is_dark(), false, false, style)
                    .stroke
                    .color;
                assert_eq!(
                    border,
                    disabled_group_fade(stroke, fill_rule, p.background),
                    "border stroke is the group-fade precomposite ({mode:?}/{style:?})"
                );
            }
        }
    }

    #[test]
    fn ring_visibility_follows_focus_and_nova_invalid() {
        for style in [Style::Nova, Style::Vega] {
            assert!(ring_visible(true, false, style));
            assert!(ring_visible(true, true, style));
            assert!(!ring_visible(false, false, style));
        }
        // Nova adds the invalid rest ring; base gates ring width on focus.
        assert!(ring_visible(false, true, Style::Nova));
        assert!(!ring_visible(false, true, Style::Vega));
    }

    #[test]
    fn focused_invalid_precedence_is_provisional_pending_open1() {
        // PROVISIONAL (OPEN-1): invalid wins over focus for border + ring.
        // No snapshot rule orders the two combinations; S5 judges the
        // focused+invalid combination against this provisional choice.
        let p = palette(Mode::Light);
        let c = resolve_input_colors(&p, false, true, true, Style::Vega);
        assert_eq!(c.stroke.color, p.destructive);
        assert_eq!(c.ring, color_alpha(p.destructive, 0.2));
    }

    #[test]
    fn fg1_vertical_group_input_stretches_to_widest_member() {
        // OPEN-3: `w-full` does not exempt the input from FG-1 stretch.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let id = egui::Id::unique("input_fg1_vertical");
        let mut text = String::new();
        let mut w = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "input_fg1_vertical_ui", 100.0);
            let ui = &mut ui;
            group::begin(ui, id, GroupOrientation::Vertical);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Vertical,
                    cross: 300.0,
                },
            );
            w = Input::new(&mut text).show(ui).rect.width();
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
        });
        out.textures_delta.clear();
        assert!((w - 300.0).abs() < 0.01, "stretched width {w}, want 300");
    }

    #[test]
    fn fg1_horizontal_target_never_stretches_input() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let id = egui::Id::unique("input_fg1_horizontal");
        let mut text = String::new();
        let mut w = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "input_fg1_horizontal_ui", 100.0);
            let ui = &mut ui;
            group::begin(ui, id, GroupOrientation::Horizontal);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Horizontal,
                    cross: 300.0,
                },
            );
            w = Input::new(&mut text).show(ui).rect.width();
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
        });
        out.textures_delta.clear();
        assert!((w - 100.0).abs() < 0.01, "width {w}, want 100");
    }

    #[test]
    fn fg1_explicit_width_is_exempt_from_stretch() {
        // An explicit `.width()` is a definite width: it wins over stretch.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let id = egui::Id::unique("input_fg1_explicit");
        let mut text = String::new();
        let mut w = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "input_fg1_explicit_ui", 100.0);
            let ui = &mut ui;
            group::begin(ui, id, GroupOrientation::Vertical);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Vertical,
                    cross: 300.0,
                },
            );
            w = Input::new(&mut text).width(120.0).show(ui).rect.width();
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
        });
        out.textures_delta.clear();
        assert!((w - 120.0).abs() < 0.01, "width {w}, want 120");
    }

    #[test]
    fn builders_keep_fields() {
        let mut text = String::new();
        let input = Input::new(&mut text)
            .placeholder("Email")
            .disabled(true)
            .invalid(true)
            .password(true)
            .width(240.0)
            .char_limit(10)
            .id_salt("email");
        assert_eq!(input.placeholder.as_deref(), Some("Email"));
        assert!(input.disabled);
        assert!(input.invalid);
        assert!(input.password);
        assert_eq!(input.width, Some(240.0));
        assert_eq!(input.char_limit, Some(10));
        assert_eq!(input.id_salt.as_deref(), Some("email"));
    }

    #[test]
    fn frame_placeholder_hint_is_set() {
        // The hint renders only while the text is empty.
        let mut text = String::new();
        let input = Input::new(&mut text).placeholder("Email");
        assert!(input.placeholder.is_some());
    }

    #[test]
    fn default_senses_are_hover_only_when_disabled() {
        // Disabled inputs must not react to clicks.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from("abc");
        let mut focused = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "input_disabled_test", 400.0);
            focused = Input::new(&mut text)
                .disabled(true)
                .show(&mut ui)
                .has_focus();
        });
        out.textures_delta.clear();
        assert!(!focused);
    }

    // INP-INT-01 pins: drive the REAL `Input::show` + `TextEdit` through
    // `ctx.run_ui` with crafted pointer-event frames. The click position
    // lands inside the first word ("Hello"); word selection is chars
    // 0..5, select-all is 0..14. A settle pass follows every click frame:
    // focus requested by the click lands on the next pass, and the
    // stored selection is only kept once the widget has focus.
    fn click_input(time: f64, pos: Pos2, pairs: usize) -> egui::RawInput {
        let mut input = egui::RawInput {
            time: Some(time),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            ..Default::default()
        };
        input.events.push(egui::Event::PointerMoved(pos));
        for _ in 0..pairs {
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            });
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
        input
    }

    fn settle_input(time: f64) -> egui::RawInput {
        egui::RawInput {
            time: Some(time),
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            ..Default::default()
        }
    }

    fn run_click_sequence(
        salt: &'static str,
        text: &mut String,
        pairs_per_frame: Vec<usize>,
    ) -> Option<egui::text::CCursorRange> {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut rect = Rect::NOTHING;
        let mut id = egui::Id::NULL;
        // Pass 1: layout + stable widget id.
        let mut out = ctx.run_ui(settle_input(1.0), |ctx| {
            let mut ui = test_ui(ctx, salt, 400.0);
            let r = Input::new(text).id_salt("pin").show(&mut ui);
            rect = r.rect;
            id = r.id;
        });
        out.textures_delta.clear();
        let pos = Pos2::new(rect.left() + 20.0, rect.center().y);
        // Click frames 100ms apart in frame time (fast-machine spacing);
        // the caller controls how the pairs distribute across frames.
        for (i, pairs) in pairs_per_frame.into_iter().enumerate() {
            let frame = click_input(1.1 + i as f64 * 0.1, pos, pairs);
            let mut out = ctx.run_ui(frame, |ctx| {
                let mut ui = test_ui(ctx, salt, 400.0);
                let _ = Input::new(text).id_salt("pin").show(&mut ui);
            });
            out.textures_delta.clear();
        }
        // Settle pass: focus lands, the stored selection persists.
        let mut out = ctx.run_ui(settle_input(2.0), |ctx| {
            let mut ui = test_ui(ctx, salt, 400.0);
            let _ = Input::new(text).id_salt("pin").show(&mut ui);
        });
        out.textures_delta.clear();
        TextEdit::load_state(&ctx, id).and_then(|s| s.cursor.char_range())
    }

    #[test]
    fn triple_click_coalesced_in_one_frame_selects_all() {
        // INP-INT-01 (M2): all three pairs in ONE frame — the normal
        // delivery under slow rendering. Before the fix the double
        // branch shadowed the triple and only the word was selected.
        let mut text = String::from("Hello, shadcn!");
        let range = run_click_sequence("input_tclick_coalesced", &mut text, vec![3]);
        assert_eq!(
            range,
            Some(CCursorRange::select_all_str("Hello, shadcn!")),
            "coalesced triple-click selects all text"
        );
    }

    #[test]
    fn triple_click_split_across_fast_frames_selects_all() {
        // Control: the framework path. Three frames with 100ms frame-time
        // gaps (a fast machine) select all WITHOUT the override — the
        // fix must not disturb this path.
        let mut text = String::from("Hello, shadcn!");
        let range = run_click_sequence("input_tclick_split", &mut text, vec![1, 1, 1]);
        assert_eq!(
            range,
            Some(CCursorRange::select_all_str("Hello, shadcn!")),
            "split triple-click with small frame-time gaps selects all text"
        );
    }

    #[test]
    fn double_click_in_one_frame_selects_word_only() {
        // Control: the override must not fire for a double — no count-3
        // release exists, so the word (and only the word) is selected.
        let mut text = String::from("Hello, shadcn!");
        let range = run_click_sequence("input_dclick_coalesced", &mut text, vec![2]);
        assert_eq!(
            range,
            Some(CCursorRange::two(
                egui::text::CCursor::new(0),
                egui::text::CCursor::new(5)
            )),
            "coalesced double-click selects the word only"
        );
    }

    #[test]
    fn single_click_places_caret_without_selection() {
        // Control: one pair places a caret (an empty range), no selection.
        let mut text = String::from("Hello, shadcn!");
        let range = run_click_sequence("input_click_single", &mut text, vec![1]);
        let range = range.expect("a caret range is stored");
        assert!(range.is_empty(), "single click selects nothing: {range:?}");
    }
}
