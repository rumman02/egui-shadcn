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
//! Behaviour notes (frozen spec `~/workspace/batch-b/specs/textarea-spec.md`):
//!
//! * Type size is `text-base ... md:text-sm`: 14px/20px governs at `md`+
//!   widths (≥768px) and 16px/24px below. The port renders the `md`+ size
//!   unconditionally — proposed deviation D5 (egui has no viewport
//!   breakpoints), awaiting Expert Reviewer approval.
//! * The box grows with content (`field-sizing-content`, spec §3.4): height
//!   is `max(64, 18 + 20 × N)` px for N visual rows at `md`+ widths (§3.2).
//!   The `rows` builder sets only the minimum visible row count (default 2,
//!   the HTML default); growth past it is content-sized, so deviation D3 is
//!   VOID for this build. Exact `field-sizing-content` edges (OPEN-3) are
//!   owned by S4/S5.
//! * Invalid + focus precedence is OPEN-2; the interim expectation (invalid
//!   wins for border and ring colour) is implemented and labelled interim.
//! * Transition duration/easing are OPEN-1; the Phase 1 motion defaults
//!   (150ms, cubic-bezier(0.4, 0, 0.2, 1)) are used and labelled INTERIM.
//! * The focus ring shows on any egui focus — proposed deviation D1 (the
//!   approved button-D3 `:focus-visible` modality class).
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

use crate::theme::{color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use crate::utils::animation::{transition_alpha, transition_color_to, TransitionSpec};
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Color32, CornerRadius, CursorIcon, FontId, Frame, Margin, Response, Stroke, TextEdit, Ui, Vec2,
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
/// animated).
///
/// Precedence (OPEN-2, INTERIM): invalid wins over focus for both the
/// border and the ring colour. The compiled cascade order is not in the
/// snapshot; S4/S5 own the settlement.
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
/// this single composition point. Applied instantly, never animated
/// (spec §6). Derivation for the disabled fills: nova light `input/50` ×
/// 0.5 = 25% effective, nova dark `input/80` × 0.5 = 40%, base dark
/// `input/30` × 0.5 = 15%.
pub(crate) fn apply_element_opacity(color: Color32, disabled: bool) -> Color32 {
    if disabled {
        color_alpha(color, 0.5)
    } else {
        color
    }
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
        let fill = apply_element_opacity(
            transition_color_to(&ctx, id.with("fill"), fill_target, spec).color,
            self.disabled,
        );
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let edit = TextEdit::multiline(self.text)
            .id_salt(id)
            .font(font.clone())
            .text_color(text_color)
            .desired_width(width)
            .desired_rows(self.rows)
            .min_size(Vec2::new(width, TEXTAREA_MIN_HEIGHT))
            .char_limit(self.char_limit)
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
        let response = out.inner.response.response;
        let rect = response.rect;

        // `disabled:cursor-not-allowed` (spec §4; D7 implementation note).
        if self.disabled && response.hovered() {
            ui.set_cursor_icon(CursorIcon::NotAllowed);
        }

        if ui.is_rect_visible(rect) {
            let focused = response.has_focus() && !self.disabled;
            let (_, stroke_target, ring_target) =
                resolve_textarea_colors(&p, dark, focused, self.invalid, self.disabled, style);
            // Animation follows the frozen spec §6 literally: under the
            // base recipe the `color` and `box-shadow` properties animate
            // (fill/border/ring), and under nova `transition-colors`
            // animates colours while the ring snaps. NOTE: the input spec
            // §6 reads the base property list more narrowly (border/fill
            // snap under base) — the two frozen specs disagree here; the
            // coordinator ruling is requested in the S1 report, and S9
            // re-wires this block if the ruling goes the other way. (The
            // fill animator runs before the TextEdit — see above — so the
            // frame carries the animated fill; only border/ring resolve
            // here, where focus is known.)
            let stroke_color = apply_element_opacity(
                transition_color_to(&ctx, id.with("border"), stroke_target, spec).color,
                self.disabled,
            );
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
}
