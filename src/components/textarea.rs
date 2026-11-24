//! shadcn `textarea.tsx` — the multi-line text field.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/textarea.tsx` (17 lines).
//! The base recipe is one class string (`textarea.tsx:9`, quoted verbatim):
//!
//! ```text
//! flex field-sizing-content min-h-16 w-full rounded-md border border-input
//! bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow]
//! outline-none placeholder:text-muted-foreground focus-visible:border-ring
//! focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed
//! disabled:opacity-50 aria-invalid:border-destructive
//! aria-invalid:ring-destructive/20 md:text-sm dark:bg-input/30
//! dark:aria-invalid:ring-destructive/40
//! ```
//!
//! The nova override (`.cn-textarea`, `style-nova.css:1337-1339`) layers over
//! the base string (LAYER ruling): it restates the focus/invalid ring set,
//! changes radius (`rounded-lg`, 10px), padding (`px-2.5`, 10px) and the
//! transition (`transition-colors`), and adds three things — the dark invalid
//! border `dark:aria-invalid:border-destructive/50` (N5, settled), the
//! disabled fills (`disabled:bg-input/50 dark:disabled:bg-input/80`), and the
//! invalid rest ring (`aria-invalid:ring-3`). Everything the nova rule is
//! silent on keeps its base value, including `shadow-xs` and the
//! whole-element `disabled:opacity-50` (see the frozen spec §2.3 for the
//! per-property derivation).
//!
//! Behaviour notes (frozen spec `~/workspace/batch-b/specs/textarea-spec.md`,
//! verified final state — Batch B S10 PASS, all four style/mode configs):
//!
//! * Type size is `text-base ... md:text-sm`: 14px/20px governs at `md`+
//!   widths (≥768px) and 16px/24px below. The port renders the `md`+ size
//!   unconditionally — proposed deviation D5 (egui has no viewport
//!   breakpoints), awaiting Expert Reviewer approval.
//! * The box grows with content (`field-sizing-content`, spec §3.4): height
//!   is `max(64, 18 + 20 × N)` px for N visual rows at `md`+ widths (§3.2).
//!   The `rows` builder sets only the minimum visible row count (default 2,
//!   the HTML default); growth past it is content-sized, so deviation D3 is
//!   VOID for this build. Exact `field-sizing-content` edges (OPEN-3) stay
//!   OPEN as frozen (gate G5).
//! * Invalid + focus precedence is OPEN-2 and stays OPEN as frozen (gate
//!   G5); the interim expectation (invalid wins for border and ring
//!   colour) is implemented and labelled interim.
//! * Invalid split (N5, settled): under nova an invalid field shows the
//!   3px rest ring (`destructive/20` light, `/40` dark) even unfocused,
//!   and the nova dark invalid border is `destructive/50`; under the base
//!   recipe an invalid-but-unfocused field shows no ring (colour set, no
//!   width class) and the border is full `destructive` in both modes.
//! * The focus ring is 3px, `ring/50`, painted at the outer edge of the
//!   border box with zero gap (offset 0, spec §3.1); the focus border is
//!   the `ring` token. The ring shows on any egui focus — proposed
//!   deviation D1 (the approved button-D3 `:focus-visible` modality
//!   class).
//! * Disabled composition is group fade (ruling 7): the whole element
//!   renders, then composites once at 50% over the backdrop (CSS
//!   `opacity` semantics). The disabled border stroke is painted opaque
//!   with the group-fade precomposite, reading (242)/(242) light and
//!   (41)/(33) dark (ruling 11); the fill, text, ring, and shadow are
//!   halved at the single composition point, which is exact for those
//!   slots. The disabled fade is instant (`opacity` is in neither
//!   transition list, spec §6).
//! * `shadow-xs` survives under every style (LAYER: `.cn-textarea` is
//!   silent on shadow) and fades with the element when disabled (2.5%
//!   effective). There is no hover or pressed state (no `hover:*` /
//!   `active:*` class); the placeholder stays visible while focused.
//! * Animation (ruling 6, narrow reading): under the base recipe,
//!   `transition-[color,box-shadow]` covers text colour and `box-shadow`
//!   only — the ring fades, while border-colour and fill changes SNAP.
//!   Under nova, `transition-colors` animates the colours while the ring
//!   snaps. Duration/easing are OPEN-1 (interim 150ms,
//!   cubic-bezier(0.4, 0, 0.2, 1), Tailwind v4 framework defaults). The
//!   M3/M9 sampling gap is ACCEPTED (gate G3): the wiring is the
//!   verification basis, and no sampled snap-vs-fade proof is claimed.
//! * Selection colours: the spec freezes NO selection-colour rule (no
//!   `selection:*` class, gate G2) — whatever the theme renders for a
//!   textarea selection is acceptable for this line; it is no defect
//!   and no deviation.
//! * Keyboard (spec §7, native platform semantics): Escape is a no-op
//!   for a focused textarea (claimed in the `EventFilter`, TXT-INT-01);
//!   PageUp/PageDown move the caret by one viewport page — the fully
//!   visible rows in the content-sized box — and clamp at the document
//!   edges (TXT-INT-03); Tab/Shift+Tab traverse focus, never insert
//!   `\t`; triple-click selects the line/paragraph (spec §7): the
//!   component counts presses by pass-processing wall time, because
//!   egui 0.37's frame-time click counting + double-before-triple
//!   branch order make a batched triple-click select the word instead
//!   (TXT-INT-02). This repairs coalesced delivery; frame-time
//!   straddling (M1) still demotes — a documented environmental
//!   limitation, not a defect.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut bio = String::new();
//! Textarea::new(&mut bio)
//!     .placeholder("Tell us about yourself")
//!     .rows(4)
//!     .show(ui);
//! ```

use crate::components::attachment::mix;
use crate::theme::{blend, color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use crate::utils::animation::{transition_alpha, transition_color_to, TransitionSpec};
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Color32, CornerRadius, CursorIcon, Event, EventFilter, FontId, Frame, Key, Margin,
    PointerButton, Pos2, Response, Stroke, TextEdit, Ui, Vec2,
};

/// `min-h-16` — the smallest textarea height (64px, spec §3.1).
pub const TEXTAREA_MIN_HEIGHT: f32 = 64.0;

/// Row pitch at `md`+ widths: the `text-sm` 20px line height (spec §3.3).
/// Rendered at all widths per proposed deviation D5.
const ROW_HEIGHT: f32 = 20.0;

/// `border` — 1px on all sides, drawn inside the border box (spec §3.1).
const BORDER_WIDTH: f32 = 1.0;

/// Horizontal padding (`px-3` base recipe; nova `.cn-textarea` `px-2.5`).
pub(crate) fn padding_x(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0, // px-2.5
        _ => 12.0,           // px-3
    }
}

/// Corner radius (`rounded-md` base recipe; nova `.cn-textarea` `rounded-lg`).
pub(crate) fn corner_radius(style: Style, radius: &RadiusTokens) -> u8 {
    match style {
        Style::Nova => radius.lg_u8(),
        _ => radius.md_u8(),
    }
}

/// Text inset from the allocated rect edge, modelling the CSS border box.
///
/// The `TextEdit` frame's inner margin is the distance from the rect edge
/// to the text. In CSS the text sits behind the 1px border plus the
/// padding, so the margin is padding + 1px on every side (vertical: `py-2`
/// 8px + 1px = 9px). This makes the spec §3.2 arithmetic exact: content
/// width = box width − 2px (border) − 2 × padding-x, and height =
/// `max(64, 18 + 20 × N)` px.
fn content_margin(style: Style) -> Margin {
    Margin::symmetric(
        padding_x(style) as i8 + BORDER_WIDTH as i8,
        8 + BORDER_WIDTH as i8,
    )
}

/// Whether the 3px ring is shown (pure, unit-testable; spec §4).
///
/// The ring shows on focus in both styles. Under nova it ALSO shows at rest
/// whenever the field is invalid (`aria-invalid:ring-3`, added by
/// `.cn-textarea`); under base there is no `aria-invalid` ring-width class,
/// so an invalid-but-unfocused field shows no ring (the ring *colour* is
/// set, but with no width it paints nothing).
pub(crate) fn ring_visible(focused: bool, invalid: bool, style: Style) -> bool {
    focused || (invalid && matches!(style, Style::Nova))
}

/// Resolved frame colors for one textarea state (pure, unit-testable).
///
/// Returns the fill, border, and ring colors BEFORE the whole-element
/// `disabled:opacity-50` is applied: callers animate these targets and then
/// apply [`apply_element_opacity`] instantly (spec §6: `opacity` is in
/// neither transition list, so the disabled fade is instant, never
/// animated) — except the border, which uses the group-fade precomposite
/// [`disabled_border_color`] instead (TXT-VIS-02: a halved-alpha stroke
/// over the faded fill double-applies the fade at the border pixel).
///
/// Precedence (OPEN-2, INTERIM): invalid wins over focus for both the
/// border and the ring colour. The compiled cascade order is not in the
/// snapshot; OPEN-2 stays OPEN as frozen (gate G5).
///
/// N5 (settled): under nova in dark mode the invalid border is
/// `destructive/50` (`dark:aria-invalid:border-destructive/50`,
/// `style-nova.css:1338`); every other invalid border is full `destructive`.
pub(crate) fn resolve_textarea_colors(
    p: &Palette,
    dark: bool,
    focused: bool,
    invalid: bool,
    disabled: bool,
    style: Style,
) -> (Color32, Color32, Color32) {
    let fill = if disabled {
        // Nova adds disabled fills; the base recipe keeps its resting fill.
        // Both are halved by the surviving `disabled:opacity-50` in
        // `apply_element_opacity`, giving the spec §4 effective values
        // (nova 25%/40%, base dark 15%, base light transparent).
        match style {
            Style::Nova => color_alpha(p.input, if dark { 0.8 } else { 0.5 }),
            _ => {
                if dark {
                    color_alpha(p.input, 0.3)
                } else {
                    Color32::TRANSPARENT
                }
            }
        }
    } else if dark {
        color_alpha(p.input, 0.3)
    } else {
        Color32::TRANSPARENT
    };
    let stroke_color = if invalid {
        // N5: nova dark invalid border is `destructive/50`.
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
    (fill, stroke_color, ring)
}

/// Applies the whole-element `disabled:opacity-50` (spec §4).
///
/// The original fades fill, border, text, ring, and shadow together at 50%;
/// egui has no element-level opacity, so every painted color is halved at
/// this single composition point — except the border stroke, which uses
/// [`disabled_border_color`] (TXT-VIS-02). Applied instantly, never
/// animated (spec §6). Derivation for the disabled fills: nova light
/// `input/50` × 0.5 = 25% effective, nova dark `input/80` × 0.5 = 40%,
/// base dark `input/30` × 0.5 = 15%.
pub(crate) fn apply_element_opacity(color: Color32, disabled: bool) -> Color32 {
    if disabled {
        color_alpha(color, 0.5)
    } else {
        color
    }
}

/// Group-fade precomposite for the disabled border stroke (TXT-VIS-02).
///
/// CSS `opacity: 0.5` composites the whole element once over the backdrop
/// (ruling 7: the spec §4 "whole element renders at 50% opacity" is
/// group-fade language). A halved-alpha stroke painted over the
/// already-faded fill double-applies the fade at the border pixel:
/// nova-light reads (239) instead of the group-fade (242). The disabled
/// stroke is therefore painted OPAQUE with the precomputed composite —
/// `0.5 * (stroke over fill over backdrop) + 0.5 * backdrop` — following
/// the toggle line's `disabled_group_fade` shape. Closing values, all
/// four configs (ruling 11): (242)/(242) light, (41)/(33) dark. `fill`
/// is the animated pre-opacity fill (the disabled-state fill target,
/// e.g. nova `input/50`);
/// `backdrop` is `p.background`, exact on the gallery page. On other
/// backdrops the theme background is the fade target, a recorded
/// limitation (same class as toggle). Only the stroke uses this path: the
/// fill paints over the backdrop alone (halving is exact there), and the
/// ring and shadow sit outside the rect over the backdrop, where halving
/// is likewise exact.
fn disabled_border_color(stroke: Color32, fill: Color32, backdrop: Color32) -> Color32 {
    mix(backdrop, blend(blend(backdrop, fill), stroke), 0.5)
}

/// Paints the textarea's `shadow-xs` through the LAYER resolution API.
///
/// `.cn-textarea` is silent on shadow, so the base `shadow-xs` survives
/// under every style (LAYER ruling §3.4). When disabled, the shadow fades
/// with the element (spec §4 derivation: 50% of the 5% alpha, 2.5%
/// effective): the layer alphas scale on the stack, following the button
/// `paint_outline_shadow` shape.
fn paint_textarea_shadow(
    painter: &egui::Painter,
    rect: egui::Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::Textarea).layers();
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
            ShadowSlot::Textarea,
            dark,
        );
    }
}

/// Multi-click window for triple-click detection (TXT-INT-02): mirrors
/// egui's `InputOptions::max_double_click_delay` default (0.3s,
/// `input_state/mod.rs:116`).
const MULTI_CLICK_WINDOW: std::time::Duration = std::time::Duration::from_millis(300);

/// Multi-click radius for triple-click detection (TXT-INT-02): mirrors
/// egui's `InputOptions::max_click_dist` default (6px,
/// `input_state/mod.rs:114`).
const MULTI_CLICK_DIST: f32 = 6.0;

/// Wall-clock multi-click sequence state (TXT-INT-02).
///
/// egui 0.37 counts clicks in FRAME time (`PointerState::begin_pass`
/// stamps one time per pass; `last_click_time` updates only on
/// release), and `TextCursorState::pointer_interaction` checks
/// `double_clicked()` before `triple_clicked()`. When two or more
/// clicks batch into one pass — the norm under slow frames, and
/// whenever the event loop wakes to a queued triple-click — the
/// pass's releases carry counts 1, 2 AND 3, so `double_clicked()` is
/// true and the word branch shadows the line branch: the
/// triple-click deterministically selects the word. Counting the
/// presses by pass-processing wall time repairs that coalesced case
/// (M2): presses arriving in one pass chain with ~zero gaps, so the
/// component selects the paragraph itself. It does NOT repair
/// frame-time straddling (M1): when the click-bearing passes are
/// more than 300 ms apart in processing time, this counter demotes
/// exactly like the framework's, because events carry no timestamps
/// and pass-processing time diverges from physical press time by the
/// render latency. M1 is a documented environmental limitation (see
/// the input INP-INT-01 report §1/§6 for the shared analysis); on a
/// fast machine the straddling residue vanishes.
#[derive(Clone, Copy, Default)]
struct TripleClickSeq {
    /// Presses in the current sequence (resets on any gap).
    count: u32,
    /// The last counted press (wall-clock time + position).
    last_press: Option<(std::time::Instant, Pos2)>,
    /// A triple-click whose release has not been seen yet: the
    /// release frame re-applies the line selection, covering the
    /// press/release frame split where the framework's demoted
    /// double branch would otherwise overwrite it with the word.
    armed: Option<(std::time::Instant, Pos2)>,
}

/// Selects the paragraph around the stored click cursor (TXT-INT-02).
///
/// Called after `TextEdit::show` when a triple-click is detected: the
/// framework has already placed the caret (or the word selection) at
/// the click position, so the paragraph is read off the stored cursor
/// — the secondary end sits safely inside the clicked line (the word
/// start after a double-click branch, or the caret itself). Writes
/// back through the documented load-modify-store pattern, under the
/// widget id. No-ops when no cursor state is stored.
fn select_clicked_line(ctx: &egui::Context, id: egui::Id, galley: &egui::text::Galley) {
    let Some(mut state) = egui::widgets::text_edit::TextEditState::load(ctx, id) else {
        return;
    };
    let Some(range) = state.cursor.range(galley) else {
        return;
    };
    let cursor = range.secondary;
    let begin = galley.cursor_begin_of_paragraph(&cursor);
    let end = galley.cursor_end_of_paragraph(&cursor);
    state
        .cursor
        .set_char_range(Some(egui::text_selection::CCursorRange::two(begin, end)));
    state.store(ctx, id);
}

/// A shadcn textarea. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Textarea<'a> {
    text: &'a mut String,
    placeholder: Option<String>,
    disabled: bool,
    invalid: bool,
    rows: usize,
    width: Option<f32>,
    char_limit: usize,
    id_salt: Option<String>,
}

impl<'a> Textarea<'a> {
    /// Creates a textarea bound to the given string.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            disabled: false,
            invalid: false,
            rows: 2,
            width: None,
            char_limit: usize::MAX,
            id_salt: None,
        }
    }

    /// Shows faint hint text when the field is empty
    /// (`placeholder:text-muted-foreground`).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Disables the field (`disabled:opacity-50 disabled:cursor-not-allowed`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the field aria-invalid (`aria-invalid:border-destructive
    /// aria-invalid:ring-destructive/20`, plus the nova rest ring and the
    /// nova dark `/50` border — spec §4).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets the minimum visible row count (the native `rows` attribute; the
    /// default 2 is the HTML default). The box grows with content beyond
    /// this (`field-sizing-content`, spec §3.4); `rows` only sets the
    /// floor, and the 64px minimum still binds below it. How `rows`
    /// interacts with content sizing in the original is OPEN-3; treating
    /// it as a minimum is the S1 decision.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    /// Sets an explicit width. The default is `w-full` (the available width
    /// in vertical layouts; 200pt inside horizontal rows, an S1 decision —
    /// the spec states `w-full` but egui rows have no containing block).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a maximum character count (the native `maxLength` attribute,
    /// spec §8). Unlimited by default.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = limit;
        self
    }

    /// Sets a stable id salt so state (focus, cursor) persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the textarea.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let style = theme.style;

        let width = match self.width {
            Some(w) => w,
            None => {
                if ui.layout().is_vertical() {
                    ui.available_width()
                } else {
                    200.0
                }
            }
        };

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_textarea"),
        };

        // Transition timing is OPEN-1: the Phase 1 motion defaults (150ms,
        // cubic-bezier(0.4, 0, 0.2, 1)) are Tailwind v4 framework defaults,
        // labelled INTERIM — not snapshot-verified truth.
        let spec = TransitionSpec::default();
        // Ruling 6 (transition narrow reading): under the base recipe,
        // `transition-[color,box-shadow]` covers the CSS `color` (text
        // colour) and `box-shadow` (ring/shadow) properties only —
        // `border-color` and `background-color` are distinct properties,
        // so the border and fill SNAP under base. Under nova,
        // `transition-colors` animates the colours while the ring snaps.
        // The zero-duration spec is the snap mechanism (`TransitionSpec`
        // snaps instantly at `duration_secs <= 0.0`); the animator ids
        // stay the same, so a style switch mid-leg settles instantly
        // instead of stranding stale animator state.
        let color_spec = if matches!(style, Style::Nova) {
            spec
        } else {
            TransitionSpec::secs(0.0, spec.easing)
        };
        let ctx = ui.ctx().clone();

        let is_empty = self.text.is_empty();
        // Text colour animates (theme-switch recolouring); the disabled
        // fade applies instantly afterwards (spec §6). The inherited text
        // colour maps to the `foreground` slot (spec §5.1); the light value
        // follows themes.ts (0.145) per the coordinator's light-token
        // divergence note (OPEN-4, settled for judging).
        let text_color = apply_element_opacity(
            transition_color_to(&ctx, id.with("text"), p.foreground, spec).color,
            self.disabled,
        );
        let corner = CornerRadius::same(corner_radius(style, &theme.radius));
        // The fill is resolved BEFORE the TextEdit runs: it does not depend
        // on focus (spec §4 — only the border and ring do), and the
        // TextEdit's frame carries it so it paints UNDER the glyphs
        // (TXT-VIS-01). The default frame's opaque `p.input` fill showed
        // through the transparent resting fill, and the old post-TextEdit
        // `rect_filled` veiled the glyphs; the frame fill is painted by the
        // atom layout before the text atoms, so neither can happen.
        let (fill_target, _, _) =
            resolve_textarea_colors(&p, dark, false, self.invalid, self.disabled, style);
        // Keep the animated pre-opacity fill: the frame carries the faded
        // fill, while the disabled border precomposite (TXT-VIS-02) needs
        // the un-faded animated value. Same animator id, same target; the
        // leg eases under nova and snaps under base (ruling 6, via
        // `color_spec` above).
        let fill_animated =
            transition_color_to(&ctx, id.with("fill"), fill_target, color_spec).color;
        let fill = apply_element_opacity(fill_animated, self.disabled);
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let edit = TextEdit::multiline(self.text)
            .id_salt(id)
            .font(font.clone())
            .text_color(text_color)
            .desired_width(width)
            .desired_rows(self.rows)
            .min_size(Vec2::new(width, TEXTAREA_MIN_HEIGHT))
            .char_limit(self.char_limit)
            // TXT-INT-01: spec §7 requires Escape to be a no-op for a
            // focused textarea. egui's `Focus::begin_pass` clears focus
            // on unmodified Escape unless the focused widget's event
            // filter matches the event, so claim Escape in the filter —
            // the same shape `button.rs` uses for its focus-lock filter.
            // The multiline defaults are restated in full (arrows
            // captured, Tab NOT captured: Tab is focus traversal, never
            // `\t`, spec §7). The TextEdit event loop has no Escape
            // action, so the key becomes a true no-op: focus, caret, and
            // text unchanged.
            .event_filter(EventFilter {
                tab: false,
                horizontal_arrows: true,
                vertical_arrows: true,
                escape: true,
            })
            // Explicit frame REPLACES the default TextEdit frame (opaque
            // `text_edit_bg_color` fill + state-dependent stroke): the
            // fill is the resolved component fill, the stroke is NONE (the
            // component paints its own border below), and the padding is
            // baked into `inner_margin` — with an explicit frame the
            // builder's `.margin()` is ignored, so the text inset from the
            // rect edge is exactly `content_margin`, as before.
            .frame(Frame {
                inner_margin: content_margin(style),
                corner_radius: corner,
                fill,
                ..Frame::NONE
            });
        let edit = if let Some(placeholder) = self.placeholder.as_deref() {
            if is_empty {
                edit.hint_text(placeholder.to_owned())
            } else {
                edit
            }
        } else {
            edit
        };
        let edit = if self.disabled {
            edit.interactive(false)
        } else {
            edit
        };

        // Body lines at Tailwind `text-sm` 20px line height (spec §3.3, D5):
        // widen the layouter's row height inside a scoped Ui so no sibling
        // is affected. The multiline `TextEdit` sizes from its galley, so
        // the box grows and shrinks with content (§3.4) against the §3.2
        // formula; `desired_rows` only sets the floor.
        let row_height = ui.fonts_mut(|f| f.row_height(&font));
        let extra = (ROW_HEIGHT - row_height).max(0.0);
        let out = ui.scope(|ui| {
            ui.spacing_mut().extra_text_line_spacing = extra;
            edit.show(ui)
        });
        let output = out.inner;
        let response = output.response.response;
        let rect = response.rect;
        // Hoisted for the TXT-INT-02 block below: both post-`show`
        // handlers need the galley (paging moves rows, triple-click
        // selects a paragraph), and `output.galley` can only move out
        // once. The TXT-INT-03 block below is otherwise unchanged.
        let galley = output.galley;

        // TXT-INT-03: PageUp/PageDown paging (spec §7). egui 0.37 has NO
        // framework handling for these keys in `TextEdit` (only the key
        // definitions exist), so a focused textarea must move the caret
        // itself. After `TextEdit::show` the events are still visible in
        // `ui.input` (the builder only filters a copy), and the moved
        // caret is written back through the documented `TextEditState`
        // load-modify-store pattern, under `response.id` — the widget id
        // `show()` computed, NOT the local `id` salt (the two differ).
        //
        // One page = the fully visible text rows in the current box (the
        // box is content-sized, so every row is always visible and there
        // is no scroll offset: the spec's "scrolls" leg is vacuous). The
        // inset is `content_margin`'s vertical margin (8px `py-2` + 1px
        // border); on the §3.2 ladder this yields 64px→2, 78px→3,
        // 98px→4, 118px→5 rows. Row moves reuse the galley's own
        // `cursor_up/down_one_row`, threading `h_pos` exactly as the
        // arrow-key path does, and clamp idempotently at the document
        // edges. Shift extends the selection (the anchor stays put); any
        // ctrl/alt/command modifier disables paging. The events are left
        // in the input list: nothing else consumes them, matching the
        // arrow-key treatment.
        if !self.disabled && response.has_focus() {
            let moves: Vec<(bool, bool)> = ui.input(|i| {
                i.events
                    .iter()
                    .filter_map(|event| {
                        if let Event::Key {
                            key,
                            pressed,
                            modifiers,
                            ..
                        } = event
                        {
                            let paging = matches!(key, Key::PageDown | Key::PageUp);
                            let plain = !modifiers.ctrl
                                && !modifiers.alt
                                && !modifiers.command
                                && !modifiers.mac_cmd;
                            if *pressed && paging && plain {
                                return Some((matches!(key, Key::PageDown), modifiers.shift));
                            }
                        }
                        None
                    })
                    .collect()
            });
            if !moves.is_empty() {
                let mut state = output.state;
                let inset = content_margin(style).top as f32;
                let page_rows = ((rect.height() - 2.0 * inset) / ROW_HEIGHT)
                    .floor()
                    .max(1.0) as usize;
                if let Some(mut range) = state.cursor.range(&galley) {
                    for (down, shift) in moves {
                        let mut cursor = range.primary;
                        let mut h_pos = range.h_pos;
                        for _ in 0..page_rows {
                            (cursor, h_pos) = if down {
                                galley.cursor_down_one_row(&cursor, h_pos)
                            } else {
                                galley.cursor_up_one_row(&cursor, h_pos)
                            };
                        }
                        range.primary = cursor;
                        range.h_pos = h_pos;
                        if !shift {
                            range.secondary = cursor;
                        }
                    }
                    state.cursor.set_char_range(Some(range));
                    state.store(&ctx, response.id);
                }
            }
        }

        // TXT-INT-02: triple-click line selection (spec §7). egui 0.37
        // implements the line path (`select_line_at`), but its
        // frame-time click counting + double-before-triple branch
        // order make a batched triple-click deterministically select
        // the word (see `TripleClickSeq`). The component therefore
        // counts modifier-free primary presses by pass-processing
        // wall time and selects the paragraph itself, repairing the
        // coalesced case; straddled passes still demote (M1 — see
        // `TripleClickSeq`). When the framework's own
        // count already produced the line (fast machine, one click
        // per pass), the override writes the identical range and is
        // a no-op — including for the double/triple drag anchor,
        // which `set_char_range` only clears when the sorted range
        // changes. Single-click, double-click, drag, shift-click
        // extension, and keys are untouched: only a third press
        // inside the window + radius acts, and the armed release
        // re-application is consumed by the first release frame.
        if !self.disabled && response.has_focus() {
            let mut seq = ctx.data_mut(|data| {
                data.get_temp::<TripleClickSeq>(response.id.with("tclick_seq"))
                    .unwrap_or_default()
            });
            let now = std::time::Instant::now();
            // Expire a stale armed triple: it may only re-apply on
            // the release frame immediately following the third
            // press, never on a later unrelated release.
            if seq
                .armed
                .is_some_and(|(armed_at, _)| now.duration_since(armed_at) >= MULTI_CLICK_WINDOW)
            {
                seq.armed = None;
            }
            let mut apply_line = false;
            ui.input(|i| {
                for event in &i.events {
                    match event {
                        Event::Key { .. } | Event::Text(_) => {
                            // Any keyboard activity breaks a click
                            // sequence and invalidates a pending
                            // re-application. (A Shift press also
                            // keeps shift-click extension entirely in
                            // the framework's hands.)
                            seq.count = 0;
                            seq.last_press = None;
                            seq.armed = None;
                        }
                        Event::PointerButton {
                            pos,
                            button: PointerButton::Primary,
                            pressed,
                            modifiers,
                        } => {
                            if *pressed {
                                if !modifiers.is_none() || !rect.contains(*pos) {
                                    seq.count = 0;
                                    seq.last_press = None;
                                    seq.armed = None;
                                } else {
                                    let chained = seq.last_press.is_some_and(|(at, at_pos)| {
                                        now.duration_since(at) < MULTI_CLICK_WINDOW
                                            && pos.distance(at_pos) <= MULTI_CLICK_DIST
                                    });
                                    seq.count = if chained { seq.count + 1 } else { 1 };
                                    seq.last_press = Some((now, *pos));
                                    if seq.count >= 3 {
                                        apply_line = true;
                                        seq.armed = Some((now, *pos));
                                    }
                                }
                            } else if let Some((armed_at, armed_pos)) = seq.armed {
                                // The release frame: re-apply the line
                                // selection if this release completes
                                // the armed triple (same window +
                                // radius). The armed state is consumed
                                // either way — one re-application max.
                                if now.duration_since(armed_at) < MULTI_CLICK_WINDOW
                                    && pos.distance(armed_pos) <= MULTI_CLICK_DIST
                                {
                                    apply_line = true;
                                }
                                seq.armed = None;
                            }
                        }
                        _ => {}
                    }
                }
            });
            if apply_line && !galley.rows.is_empty() {
                select_clicked_line(&ctx, response.id, &galley);
            }
            ctx.data_mut(|data| data.insert_temp(response.id.with("tclick_seq"), seq));
        }

        // `disabled:cursor-not-allowed` (spec §4; D7 implementation note).
        if self.disabled && response.hovered() {
            ui.set_cursor_icon(CursorIcon::NotAllowed);
        }

        if ui.is_rect_visible(rect) {
            let focused = response.has_focus() && !self.disabled;
            let (_, stroke_target, ring_target) =
                resolve_textarea_colors(&p, dark, focused, self.invalid, self.disabled, style);
            // Animation follows ruling 6 (the narrow reading of the base
            // `transition-[color,box-shadow]`): the border and fill snap
            // under base (zero-duration `color_spec`), the ring fades
            // under base, and the text colour animates under both styles
            // (the `color` property). Under nova `transition-colors`
            // animates the colours while the ring snaps. (The fill
            // animator runs before the TextEdit — see above — so the
            // frame carries the animated fill; only border/ring resolve
            // here, where focus is known.)
            let stroke_animated =
                transition_color_to(&ctx, id.with("border"), stroke_target, color_spec).color;
            // TXT-VIS-02: when disabled, the stroke precomposites the
            // group fade (opaque) instead of halving its alpha over the
            // faded fill. Enabled rendering is unchanged.
            let stroke_color = if self.disabled {
                disabled_border_color(stroke_animated, fill_animated, p.background)
            } else {
                stroke_animated
            };
            let show_ring = ring_visible(focused, self.invalid, style);
            let (ring_color, ring_alpha) = if matches!(style, Style::Nova) {
                // `transition-colors` does not cover `box-shadow`: the ring
                // snaps under nova.
                (ring_target, if show_ring { 1.0 } else { 0.0 })
            } else {
                (
                    transition_color_to(&ctx, id.with("ring"), ring_target, spec).color,
                    transition_alpha(&ctx, id.with("ring_alpha"), 0.0, 1.0, show_ring, spec).alpha,
                )
            };
            let ring_color =
                apply_element_opacity(color_alpha(ring_color, ring_alpha), self.disabled);

            let painter = ui.painter();

            paint_textarea_shadow(painter, rect, corner, style, dark, self.disabled);
            // No fill paint here: the fill is the TextEdit frame's fill
            // (resolved above) and already sits under the glyphs. Painting
            // it again would double-composite it over the text (TXT-VIS-01
            // veil). The border and ring paint after the TextEdit: the
            // border sits at the rect edge and the ring outside it, so
            // neither overlaps the glyphs.
            painter.rect_stroke(
                rect,
                corner,
                Stroke::new(BORDER_WIDTH, stroke_color),
                egui::epaint::StrokeKind::Inside,
            );

            // Ring: 3px, immediately outside the border box (offset 0,
            // spec §3.1) — painted on the unexpanded rect, since
            // `paint_ring` strokes outside it.
            if ring_alpha > 0.0 {
                crate::utils::focus::paint_ring(painter, rect, ring_color, 3.0, corner.nw as f32);
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use crate::utils::shadow::{resolve_shadow, ShadowSize, ShadowSlot};
    use egui::{Pos2, Rect};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    /// Shows a textarea in a fixed 400×400 Ui and returns its rect.
    /// `setup` receives the textarea builder for per-test configuration.
    fn show_rect(
        ctx: &egui::Context,
        text: &mut String,
        setup: impl FnOnce(Textarea<'_>) -> Textarea<'_>,
    ) -> Rect {
        let mut setup = Some(setup);
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("textarea_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            rect = setup.take().unwrap()(Textarea::new(text).id_salt("probe"))
                .show(ui)
                .rect;
        });
        out.textures_delta.clear();
        rect
    }

    #[test]
    fn textarea_metrics_follow_style() {
        assert_eq!(padding_x(Style::Nova), 10.0); // px-2.5
        assert_eq!(padding_x(Style::Vega), 12.0); // px-3
        let theme = Preset::default().build(Mode::Light);
        assert_eq!(corner_radius(Style::Nova, &theme.radius), 10); // rounded-lg
        assert_eq!(corner_radius(Style::Vega, &theme.radius), 8); // rounded-md
    }

    #[test]
    fn textarea_is_at_least_64pt_tall() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from("line");
        let rect = show_rect(&ctx, &mut text, |ta| ta);
        assert_eq!(rect.height(), TEXTAREA_MIN_HEIGHT, "min-h-16 binds");
    }

    #[test]
    fn frame_colors_follow_state_all_configs() {
        let light = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        // Resting: transparent light fill, `input/30` dark fill, `input` border.
        for style in [Style::Nova, Style::Vega] {
            let (fill, stroke, ring) =
                resolve_textarea_colors(&light, false, false, false, false, style);
            assert_eq!(fill, Color32::TRANSPARENT, "light resting fill {style:?}");
            assert_eq!(stroke, light.input, "resting border {style:?}");
            assert_eq!(
                ring,
                color_alpha(light.ring, 0.5),
                "focus ring color {style:?}"
            );
            let (fill, stroke, _) =
                resolve_textarea_colors(&dark, true, false, false, false, style);
            assert_eq!(
                fill,
                color_alpha(dark.input, 0.3),
                "dark resting fill {style:?}"
            );
            assert_eq!(stroke, dark.input, "dark resting border {style:?}");
        }
        // Focus: border `ring`, ring `ring/50`, both styles, both modes.
        let (_, stroke, ring) =
            resolve_textarea_colors(&light, false, true, false, false, Style::Nova);
        assert_eq!(stroke, light.ring);
        assert_eq!(ring, color_alpha(light.ring, 0.5));
        // Invalid light: full `destructive` border, ring `destructive/20`.
        let (_, stroke, ring) =
            resolve_textarea_colors(&light, false, false, true, false, Style::Nova);
        assert_eq!(stroke, light.destructive);
        assert_eq!(ring, color_alpha(light.destructive, 0.2));
        // Invalid dark base: full `destructive` border, ring `/40`.
        let (_, stroke, ring) =
            resolve_textarea_colors(&dark, true, false, true, false, Style::Vega);
        assert_eq!(stroke, dark.destructive);
        assert_eq!(ring, color_alpha(dark.destructive, 0.4));
        // N5: invalid dark nova border is `destructive/50`.
        let (_, stroke, ring) =
            resolve_textarea_colors(&dark, true, false, true, false, Style::Nova);
        assert_eq!(stroke, color_alpha(dark.destructive, 0.5), "N5 /50 border");
        assert_eq!(ring, color_alpha(dark.destructive, 0.4));
    }

    #[test]
    fn invalid_wins_over_focus_provisional_open2_interim() {
        // OPEN-2 interim expectation: invalid wins for both border and ring
        // colour when invalid + focused. Test name records the interim
        // status; S4/S5 own the settlement.
        let light = palette(Mode::Light);
        let (_, stroke, ring) =
            resolve_textarea_colors(&light, false, true, true, false, Style::Nova);
        assert_eq!(stroke, light.destructive, "invalid border wins (interim)");
        assert_eq!(
            ring,
            color_alpha(light.destructive, 0.2),
            "invalid ring wins (interim)"
        );
        let dark = palette(Mode::Dark);
        let (_, stroke, ring) =
            resolve_textarea_colors(&dark, true, true, true, false, Style::Nova);
        assert_eq!(stroke, color_alpha(dark.destructive, 0.5), "N5 + interim");
        assert_eq!(ring, color_alpha(dark.destructive, 0.4));
    }

    #[test]
    fn nova_invalid_rest_ring_vs_base_focus_gated() {
        // Nova: invalid shows the ring even unfocused (`aria-invalid:ring-3`).
        assert!(ring_visible(false, true, Style::Nova));
        // Base: invalid unfocused shows NO ring (colour set, no width).
        assert!(!ring_visible(false, true, Style::Vega));
        // Focus shows the ring in both styles; resting valid shows none.
        assert!(ring_visible(true, false, Style::Nova));
        assert!(ring_visible(true, false, Style::Vega));
        assert!(!ring_visible(false, false, Style::Nova));
        assert!(!ring_visible(false, false, Style::Vega));
    }

    /// Asserts two colors agree within one level per channel: the disabled
    /// composition applies `color_alpha` twice (fill alpha, then the 50%
    /// element opacity), and premultiplied rounding can differ by 1 level
    /// from a single application of the product alpha. A 1-level delta is
    /// invisible and inside the harness PAE threshold.
    fn assert_color_near(left: Color32, right: Color32) {
        for (l, r) in [
            (left.r(), right.r()),
            (left.g(), right.g()),
            (left.b(), right.b()),
            (left.a(), right.a()),
        ] {
            assert!(
                l.abs_diff(r) <= 1,
                "colors differ by more than 1 level: {left:?} vs {right:?}"
            );
        }
    }

    #[test]
    fn disabled_composition_matches_section_4_derivation() {
        let light = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        // Nova light: fill `input/50` × 0.5 element opacity = 25% effective.
        let (fill, stroke, _) =
            resolve_textarea_colors(&light, false, false, false, true, Style::Nova);
        assert_color_near(
            apply_element_opacity(fill, true),
            color_alpha(light.input, 0.25),
        );
        assert_eq!(
            apply_element_opacity(stroke, true),
            color_alpha(light.input, 0.5)
        );
        // Nova dark: fill `input/80` × 0.5 = 40% effective.
        let (fill, _, _) = resolve_textarea_colors(&dark, true, false, false, true, Style::Nova);
        assert_color_near(
            apply_element_opacity(fill, true),
            color_alpha(dark.input, 0.4),
        );
        // Base dark: fill `input/30` × 0.5 = 15% effective.
        let (fill, _, _) = resolve_textarea_colors(&dark, true, false, false, true, Style::Vega);
        assert_color_near(
            apply_element_opacity(fill, true),
            color_alpha(dark.input, 0.15),
        );
        // Base light: transparent fill stays transparent; border halves.
        let (fill, stroke, _) =
            resolve_textarea_colors(&light, false, false, false, true, Style::Vega);
        assert_eq!(apply_element_opacity(fill, true), Color32::TRANSPARENT);
        assert_eq!(
            apply_element_opacity(stroke, true),
            color_alpha(light.input, 0.5)
        );
        // Disabled + invalid under nova: the rest ring is likewise faded.
        let (_, _, ring) = resolve_textarea_colors(&light, false, false, true, true, Style::Nova);
        assert_color_near(
            apply_element_opacity(ring, true),
            color_alpha(light.destructive, 0.1),
        );
        // Not disabled: the composition is the identity.
        assert_eq!(apply_element_opacity(light.input, false), light.input);
    }

    #[test]
    fn disabled_border_group_fade_matches_differential_model() {
        // TXT-VIS-02: the disabled border stroke must be the group-fade
        // precomposite — 0.5 * (stroke over fill over backdrop) + 0.5 *
        // backdrop — not a halved-alpha stroke over the faded fill.
        // Synthetic opaque case, exact arithmetic: backdrop 200, fill 50,
        // stroke 100. Composite = 100 (opaque stroke); group fade = 150.
        let backdrop = Color32::from_rgb(200, 200, 200);
        let fill = Color32::from_rgb(50, 50, 50);
        let stroke = Color32::from_rgb(100, 100, 100);
        assert_eq!(
            disabled_border_color(stroke, fill, backdrop),
            Color32::from_rgb(150, 150, 150)
        );
        // Transparent fill: the precomposite fades the stroke over the
        // backdrop alone.
        assert_eq!(
            disabled_border_color(stroke, Color32::TRANSPARENT, backdrop),
            Color32::from_rgb(150, 150, 150)
        );
        // Translucent stroke (dark `input` token shape): the precomposite
        // equals the differential formula — composite the enabled stack,
        // then fade once. This pins the wiring, not just the arithmetic.
        let translucent = color_alpha(Color32::from_rgb(255, 255, 255), 0.15);
        let fill_composite = blend(backdrop, fill);
        assert_eq!(
            disabled_border_color(translucent, fill, backdrop),
            mix(backdrop, blend(fill_composite, translucent), 0.5)
        );
        // Real palette values: the four closing values from the TXT-VIS-01
        // amendment (group fade governs; the (28) slot model is
        // superseded). The nova-light (242) is the discriminating slot:
        // the old per-slot path read (239).
        let light = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        for (p, dark_mode, style, expected) in [
            (&light, false, Style::Nova, 242u8),
            (&light, false, Style::Vega, 242u8),
            (&dark, true, Style::Nova, 41u8),
            (&dark, true, Style::Vega, 33u8),
        ] {
            let (fill, stroke, _) =
                resolve_textarea_colors(p, dark_mode, false, false, true, style);
            assert_color_near(
                disabled_border_color(stroke, fill, p.background),
                Color32::from_rgb(expected, expected, expected),
            );
        }
        // The precomposite is opaque: what sits beneath the stroke cannot
        // double-apply the fade.
        let (fill, stroke, _) =
            resolve_textarea_colors(&light, false, false, false, true, Style::Nova);
        assert_eq!(
            disabled_border_color(stroke, fill, light.background).a(),
            255
        );
    }

    #[test]
    fn disabled_border_shape_carries_group_fade_precomposite() {
        // TXT-VIS-02 regression pin (shape-level): a disabled textarea's
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
                let preset = Preset {
                    style,
                    ..Preset::default()
                };
                let ctx = egui::Context::default();
                preset.build(mode).install(&ctx);
                let p = preset.build(mode).palette;
                let mut text = String::new();
                let mut rect = Rect::NOTHING;
                let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("textarea_border_pin"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                    );
                    rect = Textarea::new(&mut text)
                        .id_salt("border-pin")
                        .disabled(true)
                        .show(ui)
                        .rect;
                });
                let mut shapes: Vec<Shape> = Vec::new();
                for clipped in &out.shapes {
                    flatten(&clipped.shape, &mut shapes);
                }
                out.textures_delta.clear();
                // The border: the 1px stroke on exactly the widget rect.
                // (The frame fill rect has no stroke; shadow rects sit on
                // expanded rects.)
                let border = shapes.iter().find_map(|s| match s {
                    Shape::Rect(r) if r.rect == rect && r.stroke.width == BORDER_WIDTH => {
                        Some(r.stroke.color)
                    }
                    _ => None,
                });
                let border = border.expect("disabled border stroke present");
                let (fill, stroke, _) =
                    resolve_textarea_colors(&p, mode.is_dark(), false, false, true, style);
                assert_eq!(
                    border,
                    disabled_border_color(stroke, fill, p.background),
                    "border stroke is the group-fade precomposite ({mode:?}/{style:?})"
                );
            }
        }
    }

    #[test]
    fn ruling6_base_border_and_fill_snap_while_ring_fades() {
        // TXT-ANM-01 regression pin (shape-level, controlled time): under
        // the base recipe the border and fill SNAP on change (ruling 6 —
        // the narrow reading of `transition-[color,box-shadow]`), while
        // the ring fades; under nova the colours ease and the ring snaps.
        // Real state changes (focus, then a theme switch) are driven
        // through `show()` at controlled times and the emitted shapes
        // inspected — resolver tests alone cannot catch a paint-wiring
        // regression (TXT-VIS-01 lesson).
        use egui::epaint::Shape;

        fn flatten(shape: &Shape, acc: &mut Vec<Shape>) {
            match shape {
                Shape::Vec(v) => v.iter().for_each(|s| flatten(s, acc)),
                s => acc.push(s.clone()),
            }
        }

        // One frame at `time` with a stable scope id (focus must survive
        // across frames). Returns the widget rect, the response id, and
        // the flattened shapes.
        fn frame(
            ctx: &egui::Context,
            time: f64,
            text: &mut String,
        ) -> (Rect, egui::Id, Vec<Shape>) {
            let mut rect = Rect::NOTHING;
            let mut id = egui::Id::unique("ruling6-pin-unset");
            let mut out = ctx.run_ui(
                egui::RawInput {
                    time: Some(time),
                    ..Default::default()
                },
                |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        egui::Id::unique("textarea_ruling6_pin"),
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                    );
                    let response = Textarea::new(text).id_salt("ruling6-pin").show(ui);
                    rect = response.rect;
                    id = response.id;
                },
            );
            let mut shapes = Vec::new();
            for clipped in &out.shapes {
                flatten(&clipped.shape, &mut shapes);
            }
            out.textures_delta.clear();
            (rect, id, shapes)
        }

        fn dump(shapes: &[Shape]) -> String {
            shapes
                .iter()
                .filter_map(|s| match s {
                    Shape::Rect(r) => Some(format!(
                        "rect={:?} fill={:?} stroke={:?}",
                        r.rect, r.fill, r.stroke
                    )),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        }

        fn border_color(shapes: &[Shape], rect: Rect) -> Color32 {
            shapes
                .iter()
                .find_map(|s| match s {
                    Shape::Rect(r) if r.rect == rect && r.stroke.width == BORDER_WIDTH => {
                        Some(r.stroke.color)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("no border stroke on {rect:?}:\n{}", dump(shapes)))
        }

        fn ring_color(shapes: &[Shape], rect: Rect) -> Option<Color32> {
            shapes.iter().find_map(|s| match s {
                Shape::Rect(r) if r.rect == rect && r.stroke.width == 3.0 => Some(r.stroke.color),
                _ => None,
            })
        }

        fn fill_color(shapes: &[Shape], rect: Rect) -> Option<Color32> {
            shapes.iter().find_map(|s| match s {
                Shape::Rect(r) if r.rect == rect && r.stroke.width == 0.0 => Some(r.fill),
                _ => None,
            })
        }

        for style in [Style::Vega, Style::Nova] {
            let preset = Preset {
                style,
                ..Preset::default()
            };
            let ctx = egui::Context::default();
            preset.build(Mode::Light).install(&ctx);
            let light = preset.build(Mode::Light).palette;
            let dark = preset.build(Mode::Dark).palette;
            let mut text = String::new();

            // Frame A (t=0): settled, unfocused, light. Border is the
            // resting `input`; no ring.
            let (rect, id, shapes) = frame(&ctx, 0.0, &mut text);
            assert_eq!(
                border_color(&shapes, rect),
                light.input,
                "resting border {style:?}"
            );
            assert_eq!(ring_color(&shapes, rect), None, "no rest ring {style:?}");

            // Focus the widget; frame B (t=0.05) is the flip frame.
            ctx.memory_mut(|m| m.request_focus(id));
            let (_, _, shapes) = frame(&ctx, 0.05, &mut text);
            if matches!(style, Style::Vega) {
                // Base: the border SNAPS to `ring` on the flip frame.
                assert_eq!(
                    border_color(&shapes, rect),
                    light.ring,
                    "base border snaps on focus {style:?}"
                );
                // The ring leg has just started: no ring yet.
                assert_eq!(
                    ring_color(&shapes, rect),
                    None,
                    "ring leg starts at 0 {style:?}"
                );
            } else {
                // Nova: the border leg has just started — still the old
                // target, NOT snapped.
                assert_eq!(
                    border_color(&shapes, rect),
                    light.input,
                    "nova border eases, no snap {style:?}"
                );
                // Nova: the ring SNAPS on the flip frame.
                let ring = ring_color(&shapes, rect).expect("nova ring present on flip frame");
                assert_eq!(
                    ring,
                    color_alpha(light.ring, 0.5),
                    "nova ring snaps {style:?}"
                );
            }

            // Frame C (t=0.10): mid-leg.
            let (_, _, shapes) = frame(&ctx, 0.10, &mut text);
            if matches!(style, Style::Vega) {
                assert_eq!(
                    border_color(&shapes, rect),
                    light.ring,
                    "base border stays snapped {style:?}"
                );
                // The ring FADES: strictly between 0 and its full alpha.
                let ring = ring_color(&shapes, rect).expect("base ring mid-leg");
                let full = color_alpha(light.ring, 0.5).a();
                assert!(
                    ring.a() > 0 && ring.a() < full,
                    "base ring fades mid-leg: alpha {} vs full {full} {style:?}",
                    ring.a()
                );
            } else {
                // The border is a mid-leg mix: neither endpoint.
                let border = border_color(&shapes, rect);
                assert_ne!(border, light.input, "nova border moved {style:?}");
                assert_ne!(border, light.ring, "nova border not yet landed {style:?}");
                // The ring stays at full strength (snapped).
                let ring = ring_color(&shapes, rect).expect("nova ring mid-leg");
                assert_eq!(
                    ring,
                    color_alpha(light.ring, 0.5),
                    "nova ring stays {style:?}"
                );
            }

            // Settle frame (t=0.25): the focus leg (started t=0.05, 150ms)
            // has landed in both styles. The theme switch below must
            // re-target from a SETTLED leg, or the flip frame shows the
            // mid-flight displayed value instead of the old target.
            let (_, _, shapes) = frame(&ctx, 0.25, &mut text);
            assert_eq!(
                border_color(&shapes, rect),
                light.ring,
                "focus leg settled {style:?}"
            );
            let ring = ring_color(&shapes, rect).expect("ring settled");
            assert_eq!(
                ring,
                color_alpha(light.ring, 0.5),
                "ring settled at full strength {style:?}"
            );

            // Theme switch to dark; frame D (t=0.30) is the flip frame
            // for the fill (transparent -> `input/30`) and the border
            // (`ring` light -> `ring` dark).
            preset.build(Mode::Dark).install(&ctx);
            let (_, _, shapes) = frame(&ctx, 0.30, &mut text);
            let dark_fill = color_alpha(dark.input, 0.3);
            if matches!(style, Style::Vega) {
                assert_eq!(
                    fill_color(&shapes, rect),
                    Some(dark_fill),
                    "base fill snaps on theme switch {style:?}:\n{}",
                    dump(&shapes)
                );
                assert_eq!(
                    border_color(&shapes, rect),
                    dark.ring,
                    "base border snaps on theme switch {style:?}"
                );
            } else {
                // Nova: the fill leg has just started — still the old
                // (transparent) target; a missing fill rect paints
                // nothing, which is the same transparent render.
                assert!(
                    fill_color(&shapes, rect).is_none_or(|c| c == Color32::TRANSPARENT),
                    "nova fill eases, no snap {style:?}:\n{}",
                    dump(&shapes)
                );
                assert_eq!(
                    border_color(&shapes, rect),
                    light.ring,
                    "nova border still on old target {style:?}"
                );
            }

            // Frame E (t=0.35): mid-leg for the theme-switch legs.
            let (_, _, shapes) = frame(&ctx, 0.35, &mut text);
            if matches!(style, Style::Vega) {
                assert_eq!(
                    fill_color(&shapes, rect),
                    Some(dark_fill),
                    "base fill stays snapped {style:?}"
                );
                assert_eq!(
                    border_color(&shapes, rect),
                    dark.ring,
                    "base border stays snapped {style:?}"
                );
            } else {
                // The fill is a mid-leg mix: strictly between transparent
                // and the dark target.
                let fill = fill_color(&shapes, rect).expect("nova fill mid-leg");
                assert!(
                    fill.a() > 0 && fill.a() < dark_fill.a(),
                    "nova fill eases mid-leg: alpha {} vs target {} {style:?}",
                    fill.a(),
                    dark_fill.a()
                );
                let border = border_color(&shapes, rect);
                assert_ne!(border, light.ring, "nova border moved {style:?}");
                assert_ne!(border, dark.ring, "nova border not yet landed {style:?}");
            }
        }
    }

    #[test]
    fn shadow_xs_survives_under_nova_via_layer_resolution() {
        // `.cn-textarea` is silent on shadow (LAYER ruling §3.4): the base
        // `shadow-xs` survives under every style.
        for style in [Style::Nova, Style::Vega] {
            assert_eq!(
                resolve_shadow(ShadowSize::Xs, style, ShadowSlot::Textarea),
                ShadowSize::Xs,
                "shadow-xs survives under {style:?}"
            );
        }
    }

    #[test]
    fn content_box_heights_follow_section_3_2() {
        // §3.2: height = max(64, 18 + 20 × N) px at md+ widths.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let cases = [
            ("", 64.0),
            ("one", 64.0),
            ("one\ntwo", 64.0),
            ("one\ntwo\nthree", 78.0),
            ("one\ntwo\nthree\nfour", 98.0),
            ("one\ntwo\nthree\nfour\nfive", 118.0),
        ];
        for (text, expected) in cases {
            let mut text = text.to_owned();
            let rect = show_rect(&ctx, &mut text, |ta| ta);
            assert_eq!(
                rect.height(),
                expected,
                "§3.2 height for {text:?}: got {}, want {expected}",
                rect.height()
            );
        }
    }

    #[test]
    fn box_grows_and_shrinks_with_content() {
        // `field-sizing-content` (§3.4): the box grows as lines are added
        // and shrinks back toward the 64px floor as they are removed. Same
        // context + same id across passes, so no stale state may stick.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from("one\ntwo\nthree\nfour\nfive");
        let grown = show_rect(&ctx, &mut text, |ta| ta);
        assert_eq!(grown.height(), 118.0, "grew to 5 rows");
        text = String::from("one");
        let shrunk = show_rect(&ctx, &mut text, |ta| ta);
        assert_eq!(shrunk.height(), 64.0, "shrank back to the floor");
    }

    #[test]
    fn single_size_rendering_d5_proposed() {
        // D5 (PROPOSED): the `md`+ size (14px, 20px rows) renders at ALL
        // widths, including below the 768px `md` breakpoint. A 300px-wide
        // Ui is below `md`, yet 3 rows must still measure 78px.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from("one\ntwo\nthree");
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("textarea_d5_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 400.0))),
            );
            rect = Textarea::new(&mut text).id_salt("d5").show(ui).rect;
        });
        out.textures_delta.clear();
        assert_eq!(rect.height(), 78.0, "D5: 20px rows below md too");
    }

    #[test]
    fn wrapping_grows_the_box() {
        // Long text wraps at the content-box edge and the box grows per
        // row (§3.4): height stays on the 18 + 20×N ladder.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from(
            "alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu nu xi omicron",
        );
        let rect = show_rect(&ctx, &mut text, |ta| ta.width(200.0));
        assert!(
            rect.height() >= 78.0,
            "wrapped text grows past the floor: {}",
            rect.height()
        );
        assert_eq!(
            (rect.height() - 18.0) % ROW_HEIGHT,
            0.0,
            "height stays on the §3.2 ladder: {}",
            rect.height()
        );
    }

    #[test]
    fn rows_sets_the_minimum_visible_height() {
        // `rows` is a floor (OPEN-3a S1 decision): an empty 5-row textarea
        // measures 118px even with no content.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::new();
        let rect = show_rect(&ctx, &mut text, |ta| ta.rows(5));
        assert_eq!(rect.height(), 118.0, "rows(5) floor");
    }

    #[test]
    fn frame_fill_is_resolved_and_paints_under_glyphs() {
        // TXT-VIS-01 regression pin (shape-level): the TextEdit frame must
        // carry the RESOLVED fill — not the theme's `text_edit_bg_color`
        // (`p.input`) — and the fill must paint BEFORE the glyphs. The
        // resolver unit tests passed while the render was wrong, so this
        // test inspects the emitted shapes instead of resolver values.
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
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("textarea_frame_pin"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                rect = Textarea::new(&mut text).id_salt("pin").show(ui).rect;
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
            // Mechanism A: no rect over the widget paints the raw theme
            // `text_edit_bg_color` fill.
            for s in &shapes {
                if let Shape::Rect(r) = s {
                    assert!(
                        r.rect != rect || r.fill != p.input,
                        "no raw text_edit_bg_color fill over the widget ({mode:?})"
                    );
                }
            }
            // Mechanism B: the fill paints before the first glyph.
            if let Some(text_idx) = shapes.iter().position(|s| matches!(s, Shape::Text(_))) {
                assert!(fill_idx < text_idx, "fill paints under glyphs ({mode:?})");
            }
        }
    }

    #[test]
    fn escape_is_noop_for_focused_textarea_txt_int_01() {
        // TXT-INT-01 regression pin: spec §7 requires Escape to be a
        // no-op for a focused textarea — focus AND text unchanged.
        // egui's `Focus::begin_pass` clears focus on unmodified Escape
        // unless the focused widget's event filter matches the event,
        // so the pin drives real frames through `show()` with a real
        // Escape event in the RawInput: it FAILS on the stock TextEdit
        // filter (`escape: false`) and passes with the fix. A Tab frame
        // guards the other half of the filter: Tab must still move
        // focus away (spec §7 traversal, never `\t`).
        fn frame(
            ctx: &egui::Context,
            input: egui::RawInput,
            text: &mut String,
        ) -> (egui::Id, bool) {
            // One frame: show the textarea plus a focusable button (the
            // Tab target). Returns the textarea id + whether it holds
            // focus during this frame.
            let mut id = egui::Id::unique("textarea_escape_pin_unset");
            let mut focused = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("textarea_escape_pin"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                let response = Textarea::new(text).id_salt("escape-pin").show(ui);
                id = response.id;
                focused = response.has_focus();
                let _ = ui.button("tab-target");
            });
            out.textures_delta.clear();
            (id, focused)
        }

        fn key_event(key: egui::Key, pressed: bool) -> egui::RawInput {
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }],
                ..Default::default()
            }
        }

        for style in [Style::Vega, Style::Nova] {
            let preset = Preset {
                style,
                ..Preset::default()
            };
            let ctx = egui::Context::default();
            preset.build(Mode::Light).install(&ctx);
            let mut text = String::from("hello");

            // Frame A: unfocused. Capture the widget id.
            let (id, focused) = frame(&ctx, egui::RawInput::default(), &mut text);
            assert!(!focused, "unfocused at rest {style:?}");

            // Focus the widget; frame B installs the focus-lock filter
            // (`set_focus_lock_filter` requires focus held last frame).
            ctx.memory_mut(|m| m.request_focus(id));
            let (_, focused) = frame(&ctx, egui::RawInput::default(), &mut text);
            assert!(focused, "focused after request {style:?}");

            // Frame C: unmodified Escape press. Focus AND text unchanged.
            let (_, focused) = frame(&ctx, key_event(egui::Key::Escape, true), &mut text);
            assert!(focused, "Escape must not clear focus {style:?}");
            assert_eq!(text, "hello", "Escape must not change text {style:?}");

            // Frame D: the Escape release changes nothing either.
            let (_, focused) = frame(&ctx, key_event(egui::Key::Escape, false), &mut text);
            assert!(focused, "Escape release must not clear focus {style:?}");

            // Frame E: Tab. Focus moves at end-of-pass, so the move is
            // observed in the following frame.
            let _ = frame(&ctx, key_event(egui::Key::Tab, true), &mut text);
            let (_, focused) = frame(&ctx, egui::RawInput::default(), &mut text);
            assert!(
                !focused,
                "Tab must still move focus away {style:?} (filter keeps tab: false)"
            );
            assert_ne!(
                ctx.memory(|m| m.focused()),
                Some(id),
                "focus left the textarea after Tab {style:?}"
            );
        }
    }

    #[test]
    fn page_up_down_move_caret_by_page_txt_int_03() {
        // TXT-INT-03 regression pin: spec §7 requires PageUp/PageDown to
        // move the caret by one viewport page (the "scrolls" leg is
        // vacuous for the content-sized box, where every row is always
        // visible). egui 0.37 has NO PageUp/PageDown handling in
        // `TextEdit`, so the component moves the cursor itself. The pin
        // drives real frames through `show()` with real key events and
        // proves movement BEHAVIOURALLY (a typed character lands at the
        // moved caret), mirroring the S6 repro: Ctrl+Home, PageDown,
        // type `M` (must land on a later line), PageUp, type `N` (must
        // land back at the start). It FAILS on the stock TextEdit (no
        // movement) and passes with the fix. Shift+PageDown must extend
        // the selection; paging at the document edges must clamp
        // without error; paging itself must never change the text.
        use egui::widgets::text_edit::TextEditState;

        fn frame(
            ctx: &egui::Context,
            input: egui::RawInput,
            text: &mut String,
        ) -> (egui::Id, bool) {
            // One frame: show the textarea. Returns the widget id +
            // whether it holds focus during this frame.
            let mut id = egui::Id::unique("textarea_page_pin_unset");
            let mut focused = false;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("textarea_page_pin"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                let response = Textarea::new(text).id_salt("page-pin").show(ui);
                id = response.id;
                focused = response.has_focus();
            });
            out.textures_delta.clear();
            (id, focused)
        }

        fn key_event(key: egui::Key, pressed: bool, modifiers: egui::Modifiers) -> egui::RawInput {
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed,
                    repeat: false,
                    modifiers,
                }],
                ..Default::default()
            }
        }

        fn text_event(s: &str) -> egui::RawInput {
            egui::RawInput {
                events: vec![egui::Event::Text(s.to_owned())],
                ..Default::default()
            }
        }

        // Document-start/end keys are platform-specific: on macOS egui
        // treats Ctrl as the Emacs modifier, so Ctrl+Home/Ctrl+End are
        // ignored (move_single_cursor returns early for any Ctrl+key
        // that is not an Emacs key); the macOS equivalents are Cmd+Up /
        // Cmd+Down (ArrowUp/ArrowDown + COMMAND move to the document
        // start/end). Elsewhere they are Ctrl+Home / Ctrl+End.
        fn doc_start() -> (egui::Key, egui::Modifiers) {
            if cfg!(target_os = "macos") {
                (egui::Key::ArrowUp, egui::Modifiers::COMMAND)
            } else {
                (egui::Key::Home, egui::Modifiers::CTRL)
            }
        }

        fn doc_end() -> (egui::Key, egui::Modifiers) {
            if cfg!(target_os = "macos") {
                (egui::Key::ArrowDown, egui::Modifiers::COMMAND)
            } else {
                (egui::Key::End, egui::Modifiers::CTRL)
            }
        }

        fn cursor_index(ctx: &egui::Context, id: egui::Id) -> usize {
            TextEditState::load(ctx, id)
                .and_then(|s| s.cursor.char_range())
                .map(|r| r.primary.index.0)
                .expect("stored cursor range present")
        }

        fn selection(ctx: &egui::Context, id: egui::Id) -> (usize, usize) {
            let range = TextEditState::load(ctx, id)
                .and_then(|s| s.cursor.char_range())
                .expect("stored cursor range present");
            let [a, b] = range.sorted_cursors();
            (a.index.0, b.index.0)
        }

        for style in [Style::Vega, Style::Nova] {
            let preset = Preset {
                style,
                ..Preset::default()
            };
            let ctx = egui::Context::default();
            preset.build(Mode::Light).install(&ctx);
            let mut text = String::from("p1\np2\np3\np4\np5");

            // Frame A: unfocused. Capture the widget id.
            let (id, focused) = frame(&ctx, egui::RawInput::default(), &mut text);
            assert!(!focused, "unfocused at rest {style:?}");

            // Focus the widget.
            ctx.memory_mut(|m| m.request_focus(id));
            let (_, focused) = frame(&ctx, egui::RawInput::default(), &mut text);
            assert!(focused, "focused after request {style:?}");

            // Document start: caret to the start.
            let (doc_key, doc_modifiers) = doc_start();
            let _ = frame(&ctx, key_event(doc_key, true, doc_modifiers), &mut text);
            assert_eq!(cursor_index(&ctx, id), 0, "doc-start to start {style:?}");

            // PageDown: one page = the 5 visible rows in the 118px box.
            // The box is content-sized, so the viewport IS the whole
            // document: 5 row-moves from row 0 reach the last row and
            // the galley's edge clamp lands at the document end (14).
            // The defect file's acceptance criterion is observed
            // movement, not a specific distance.
            let _ = frame(
                &ctx,
                key_event(egui::Key::PageDown, true, egui::Modifiers::NONE),
                &mut text,
            );
            assert_eq!(cursor_index(&ctx, id), 14, "PageDown one page {style:?}");
            assert_eq!(
                text, "p1\np2\np3\np4\np5",
                "paging changes no text {style:?}"
            );

            // Typing lands at the moved caret: `M` at the end, on the
            // last line.
            let _ = frame(&ctx, text_event("M"), &mut text);
            assert_eq!(
                text, "p1\np2\np3\np4\np5M",
                "M lands at the end of the last line {style:?}"
            );

            // PageUp: back to the start.
            let _ = frame(
                &ctx,
                key_event(egui::Key::PageUp, true, egui::Modifiers::NONE),
                &mut text,
            );
            assert_eq!(cursor_index(&ctx, id), 0, "PageUp back to start {style:?}");

            // Typing lands at the start: `N` before `p1`.
            let _ = frame(&ctx, text_event("N"), &mut text);
            assert_eq!(
                text, "Np1\np2\np3\np4\np5M",
                "N lands at the start {style:?}"
            );

            // PageUp at the top clamps: no movement, no text change.
            let _ = frame(
                &ctx,
                key_event(egui::Key::PageUp, true, egui::Modifiers::NONE),
                &mut text,
            );
            assert_eq!(cursor_index(&ctx, id), 0, "PageUp clamps at top {style:?}");
            assert_eq!(
                text, "Np1\np2\np3\np4\np5M",
                "clamped paging changes no text {style:?}"
            );

            // Document end, then PageDown at the bottom clamps.
            let (doc_key, doc_modifiers) = doc_end();
            let _ = frame(&ctx, key_event(doc_key, true, doc_modifiers), &mut text);
            assert_eq!(cursor_index(&ctx, id), 16, "doc-end to end {style:?}");
            let _ = frame(
                &ctx,
                key_event(egui::Key::PageDown, true, egui::Modifiers::NONE),
                &mut text,
            );
            assert_eq!(
                cursor_index(&ctx, id),
                16,
                "PageDown clamps at end {style:?}"
            );

            // Shift+PageDown extends the selection from the start to
            // the document end (mirrors Shift+ArrowDown selection
            // semantics): sorted range 0..16 in the 16-char text.
            let (doc_key, doc_modifiers) = doc_start();
            let _ = frame(&ctx, key_event(doc_key, true, doc_modifiers), &mut text);
            let _ = frame(
                &ctx,
                key_event(egui::Key::PageDown, true, egui::Modifiers::SHIFT),
                &mut text,
            );
            assert_eq!(
                selection(&ctx, id),
                (0, 16),
                "Shift+PageDown selects one page {style:?}"
            );
        }
    }

    #[test]
    fn builders_keep_fields() {
        let mut text = String::new();
        let ta = Textarea::new(&mut text)
            .placeholder("Bio")
            .disabled(true)
            .invalid(true)
            .rows(6)
            .width(320.0)
            .char_limit(280)
            .id_salt("bio");
        assert_eq!(ta.placeholder.as_deref(), Some("Bio"));
        assert!(ta.disabled);
        assert!(ta.invalid);
        assert_eq!(ta.rows, 6);
        assert_eq!(ta.width, Some(320.0));
        assert_eq!(ta.char_limit, 280);
        assert_eq!(ta.id_salt.as_deref(), Some("bio"));
        // Default rows is 2 (the HTML default; §3.2 empty height binds).
        let mut t2 = String::new();
        assert_eq!(Textarea::new(&mut t2).rows, 2);
        assert_eq!(Textarea::new(&mut t2).char_limit, usize::MAX);
        // rows() clamps to at least one row.
        let mut t3 = String::new();
        assert_eq!(Textarea::new(&mut t3).rows(0).rows, 1);
    }

    #[test]
    fn triple_click_selects_the_line_txt_int_02() {
        // TXT-INT-02 regression pin: spec §7 requires triple-click to
        // select the line/paragraph under the cursor. egui 0.37 counts
        // clicks in FRAME time and `pointer_interaction` checks
        // `double_clicked()` before `triple_clicked()`, so a
        // triple-click batched into one pass (slow frames / queued
        // events) deterministically selects the WORD: the pass's
        // releases carry counts 1, 2 AND 3, `double_clicked()` is
        // true, and the word branch shadows the line branch. The
        // component counts presses by pass-processing wall time and
        // selects the line itself (coalesced case; straddled passes
        // still demote — M1, see `TripleClickSeq`). The pin drives real pointer frames
        // through `show()` and proves the selection BEHAVIOURALLY
        // (typing over it replaces the whole line). It FAILS on the
        // stock behaviour (word selected) and passes with the fix.
        // Double-click word selection, single-click caret placement,
        // cross-frame triples, slow-click separation, and the
        // last-line case are pinned alongside: the fix must change
        // ONLY the triple-click row.
        use egui::widgets::text_edit::TextEditState;

        fn frame(
            ctx: &egui::Context,
            input: egui::RawInput,
            text: &mut String,
        ) -> (egui::Id, bool, Rect) {
            let mut id = egui::Id::unique("textarea_tclick_pin_unset");
            let mut focused = false;
            let mut rect = Rect::NOTHING;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("textarea_tclick_pin"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                let response = Textarea::new(text).id_salt("tclick-pin").show(ui);
                id = response.id;
                focused = response.has_focus();
                rect = response.rect;
            });
            out.textures_delta.clear();
            (id, focused, rect)
        }

        fn pointer_input(time: f64, events: Vec<egui::Event>) -> egui::RawInput {
            egui::RawInput {
                time: Some(time),
                events,
                ..Default::default()
            }
        }

        fn click_at(pos: Pos2) -> Vec<egui::Event> {
            // One full click: move (hover/focus), press, release.
            vec![
                egui::Event::PointerMoved(pos),
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ]
        }

        fn selection(ctx: &egui::Context, id: egui::Id) -> (usize, usize) {
            let range = TextEditState::load(ctx, id)
                .and_then(|s| s.cursor.char_range())
                .expect("stored cursor range present");
            let [a, b] = range.sorted_cursors();
            (a.index.0, b.index.0)
        }

        fn text_event(time: f64, s: &str) -> egui::RawInput {
            egui::RawInput {
                time: Some(time),
                events: vec![egui::Event::Text(s.to_owned())],
                ..Default::default()
            }
        }

        // Three lines, two words each: word and line selections
        // differ, so the pin discriminates them.
        // "alpha beta\ngamma delta\nepsilon zeta"
        //  0         11          23           35
        const TEXT: &str = "alpha beta\ngamma delta\nepsilon zeta";

        for style in [Style::Vega, Style::Nova] {
            let preset = Preset {
                style,
                ..Preset::default()
            };
            // Text starts at rect.min + content margin (padding-x +
            // 1px border horizontally, 9px vertically); rows are
            // 20px. These offsets land inside "gamma" (row 1) and
            // "epsilon" (row 2) respectively.
            let margin_x = padding_x(style) + 1.0;

            // --- Batched triple-click, focused box (the VM case):
            // all three clicks in ONE pass. The framework selects
            // the word here; the fix must select the line. ---
            {
                let ctx = egui::Context::default();
                preset.build(Mode::Light).install(&ctx);
                let mut text = String::from(TEXT);
                let (id, _, rect) = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let pos = Pos2::new(rect.left() + margin_x + 14.0, rect.top() + 9.0 + 30.0);
                ctx.memory_mut(|m| m.request_focus(id));
                let (_, focused, _) = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                assert!(focused, "focused after request {style:?}");
                let mut events = Vec::new();
                for _ in 0..3 {
                    events.extend(click_at(pos));
                }
                let _ = frame(&ctx, pointer_input(0.05, events), &mut text);
                assert_eq!(
                    selection(&ctx, id),
                    (11, 22),
                    "batched triple-click selects the line {style:?}"
                );
                let _ = frame(&ctx, text_event(0.10, "X"), &mut text);
                assert_eq!(
                    text, "alpha beta\nX\nepsilon zeta",
                    "typing replaces the whole line {style:?}"
                );
            }

            // --- Batched triple-click, UNFOCUSED box: the first
            // click focuses; the line must still win. ---
            {
                let ctx = egui::Context::default();
                preset.build(Mode::Light).install(&ctx);
                let mut text = String::from(TEXT);
                let (id, _, rect) = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let pos = Pos2::new(rect.left() + margin_x + 14.0, rect.top() + 9.0 + 30.0);
                let mut events = Vec::new();
                for _ in 0..3 {
                    events.extend(click_at(pos));
                }
                let (_, focused, _) = frame(&ctx, pointer_input(0.05, events), &mut text);
                assert!(focused, "click focuses the box {style:?}");
                assert_eq!(
                    selection(&ctx, id),
                    (11, 22),
                    "unfocused batched triple-click selects the line {style:?}"
                );
            }

            // --- Double-click control: the word, unchanged. ---
            {
                let ctx = egui::Context::default();
                preset.build(Mode::Light).install(&ctx);
                let mut text = String::from(TEXT);
                let (id, _, rect) = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let pos = Pos2::new(rect.left() + margin_x + 14.0, rect.top() + 9.0 + 30.0);
                ctx.memory_mut(|m| m.request_focus(id));
                let _ = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let mut events = Vec::new();
                for _ in 0..2 {
                    events.extend(click_at(pos));
                }
                let _ = frame(&ctx, pointer_input(0.05, events), &mut text);
                assert_eq!(
                    selection(&ctx, id),
                    (11, 16),
                    "double-click still selects the word {style:?}"
                );
                let _ = frame(&ctx, text_event(0.10, "Y"), &mut text);
                assert_eq!(
                    text, "alpha beta\nY delta\nepsilon zeta",
                    "typing replaces the word only {style:?}"
                );
            }

            // --- Single-click control: caret only, no selection. ---
            {
                let ctx = egui::Context::default();
                preset.build(Mode::Light).install(&ctx);
                let mut text = String::from(TEXT);
                let (id, _, rect) = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let pos = Pos2::new(rect.left() + margin_x + 14.0, rect.top() + 9.0 + 30.0);
                ctx.memory_mut(|m| m.request_focus(id));
                let _ = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let _ = frame(&ctx, pointer_input(0.05, click_at(pos)), &mut text);
                let (a, b) = selection(&ctx, id);
                assert_eq!(a, b, "single click selects nothing {style:?}");
                assert!(
                    (11..=16).contains(&a),
                    "caret lands inside the clicked word {style:?}: {a}"
                );
            }

            // --- Cross-frame triple-click (the fast-machine case):
            // one click per pass; the wall-clock counter chains
            // them into a triple. ---
            {
                let ctx = egui::Context::default();
                preset.build(Mode::Light).install(&ctx);
                let mut text = String::from(TEXT);
                let (id, _, rect) = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let pos = Pos2::new(rect.left() + margin_x + 14.0, rect.top() + 9.0 + 30.0);
                ctx.memory_mut(|m| m.request_focus(id));
                let _ = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                for (i, t) in [0.05, 0.10, 0.15].into_iter().enumerate() {
                    let _ = frame(&ctx, pointer_input(t, click_at(pos)), &mut text);
                    if i < 2 {
                        // No line selection before the third press.
                        assert_ne!(
                            selection(&ctx, id),
                            (11, 22),
                            "no line selection before the third click {style:?}"
                        );
                    }
                }
                assert_eq!(
                    selection(&ctx, id),
                    (11, 22),
                    "cross-frame triple-click selects the line {style:?}"
                );
            }

            // --- Slow clicks: a 350ms real-time gap breaks the
            // sequence, so the two fast clicks after it select the
            // WORD, not the line. Frame times advance past the
            // window too, so the framework agrees. ---
            {
                let ctx = egui::Context::default();
                preset.build(Mode::Light).install(&ctx);
                let mut text = String::from(TEXT);
                let (id, _, rect) = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let pos = Pos2::new(rect.left() + margin_x + 14.0, rect.top() + 9.0 + 30.0);
                ctx.memory_mut(|m| m.request_focus(id));
                let _ = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let _ = frame(&ctx, pointer_input(0.05, click_at(pos)), &mut text);
                std::thread::sleep(std::time::Duration::from_millis(350));
                let _ = frame(&ctx, pointer_input(0.45, click_at(pos)), &mut text);
                let _ = frame(&ctx, pointer_input(0.50, click_at(pos)), &mut text);
                assert_eq!(
                    selection(&ctx, id),
                    (11, 16),
                    "slow clicks select the word, not the line {style:?}"
                );
            }

            // --- Last-line triple-click: no trailing newline; the
            // line still selects whole. ---
            {
                let ctx = egui::Context::default();
                preset.build(Mode::Light).install(&ctx);
                let mut text = String::from(TEXT);
                let (id, _, rect) = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let pos = Pos2::new(rect.left() + margin_x + 30.0, rect.top() + 9.0 + 50.0);
                ctx.memory_mut(|m| m.request_focus(id));
                let _ = frame(&ctx, pointer_input(0.0, vec![]), &mut text);
                let mut events = Vec::new();
                for _ in 0..3 {
                    events.extend(click_at(pos));
                }
                let _ = frame(&ctx, pointer_input(0.05, events), &mut text);
                assert_eq!(
                    selection(&ctx, id),
                    (23, 35),
                    "last-line triple-click selects the line {style:?}"
                );
                let _ = frame(&ctx, text_event(0.10, "W"), &mut text);
                assert_eq!(
                    text, "alpha beta\ngamma delta\nW",
                    "typing replaces the last line {style:?}"
                );
            }
        }
    }
}
