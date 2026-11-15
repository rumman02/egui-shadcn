//! shadcn typography recipes — text styles from the docs-page compositions.
//!
//! # Evidence status — read first
//!
//! shadcn/ui ships no typography component and no `ui/typography.tsx`; the
//! recipes below are docs-page compositions whose source files (the
//! `typography-*.tsx` examples and `typography.mdx`) are ABSENT from the
//! frozen snapshot (72 files searched, zero hits — frozen S0 spec §1.1).
//! NO recipe here is snapshot-verified. Every value carries an evidence
//! label from the frozen S0 spec (Batch A, signed 2026-11-15):
//!
//! | Label | Meaning |
//! |---|---|
//! | QUOTED | Quoted from two agreeing second-hand sources (the gaps-A report and this module's own recipe table as it stood pre-rebuild). The best available statement of the original, NOT a verified original value. |
//! | TAILWIND-DEFAULT | A Tailwind CSS v4 framework default (type scale, spacing scale, weight numbers). Framework behaviour, not snapshot evidence. |
//! | DERIVED | Arithmetic or a mechanical consequence of a labelled value; the derivation is shown. |
//! | VERIFIED | Read directly in a snapshot file (token values, the radius scale, the base-layer border rule, the absence of nova typography rules). |
//! | INVENTED | The [`CodeBlock`] recipe only: a documented porter's invention with no source recipe. |
//! | OPEN | Unverifiable from available inputs; listed below, none guessed. |
//!
//! Nova has NO typography rules (VERIFIED: all 441 `.cn-*` rules in
//! `style-nova.css` enumerated, none typography-related), so under the
//! settled LAYER ruling the base recipe IS the effective recipe under nova
//! too. Each element implements ONE recipe, not two; there is no style
//! branching in this module.
//!
//! # Recipes (QUOTED class strings; sizes/leading via TAILWIND-DEFAULT)
//!
//! | Element | Quoted recipe | Line height | Source margins (caller-applied, OPEN-2) |
//! |---|---|---|---|
//! | [`H1`] | `text-4xl font-extrabold tracking-tight` | 40px | — |
//! | [`H2`] | `text-3xl font-semibold tracking-tight border-b pb-2` | 36px | `mt-10` (40px), `first:mt-0` |
//! | [`H3`] | `text-2xl font-semibold tracking-tight` | 32px | `mt-8` (32px) |
//! | [`H4`] | `text-xl font-semibold tracking-tight` | 28px | `mt-8` (32px) |
//! | [`P`] | `text-base leading-7` | 28px | `mt-6` (24px), `first:mt-0` |
//! | [`Blockquote`] | `border-l-2 pl-6 italic` (+ `mt-6`) | 24px | `mt-6` (24px) |
//! | [`Ul`] / [`Ol`] | `my-6 ml-6 list-disc/list-decimal [&>li]:mt-2` | 24px | `my-6` (24px top and bottom) |
//! | [`InlineCode`] | `rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold` | 20px | — |
//! | [`CodeBlock`] | INVENTED: `rounded-md bg-muted p-4 font-mono` (no source recipe) | inherited 24px (OPEN-5, confirmed) | — |
//! | [`Lead`] | `text-xl text-muted-foreground` | 28px | — |
//! | [`Large`] | `text-lg font-semibold` | 28px | — |
//! | [`Small`] | `text-sm font-medium leading-none` | 14px | — |
//! | [`Muted`] | `text-sm text-muted-foreground` | 20px | — |
//! | [`TableCaption`] | `text-sm text-muted-foreground` | 20px | `mt-4` (16px) |
//!
//! Tailwind pairs every `text-*` size with a line height (`text-4xl` → 40px,
//! `text-3xl` → 36px, `text-2xl` → 32px, `text-xl`/`text-lg` → 28px,
//! `text-base` → 24px, `text-sm` → 20px) and the recipes override it where
//! needed (`leading-7` = 28px on paragraphs, `leading-none` = 14px on small
//! text); these are applied via egui's `line_height` so wrapped blocks match
//! the quoted row spacing. Block margins are *not* applied automatically —
//! immediate-mode UIs have no sibling knowledge, so callers space blocks
//! themselves with the source margins listed above (spec §4; `first:mt-0`
//! means a block first in its container takes no top margin).
//!
//! # Deviations and limitations (spec §11)
//!
//! * D1 (PROPOSED, Expert Reviewer approval pending): `tracking-tight`
//!   letter-spacing (-0.025em) is not rendered — egui has no letter-spacing
//!   control, so headings render at 0. Size, weight, and line height are
//!   unaffected.
//! * D2 (PROPOSED, Expert Reviewer approval pending): [`H1`] renders at
//!   SemiBold (600), the closest embedded weight, instead of the required
//!   ExtraBold (800) — no 800 face is embedded in the crate. The
//!   requirement (800) stays frozen.
//! * D7 (PROPOSED by this rebuild, numbering provisional — Coordinator /
//!   Expert Reviewer to confirm): [`InlineCode`] renders at Geist Mono
//!   Medium (500), the heaviest embedded mono face, instead of the required
//!   semibold (600) — no 600 mono face is embedded. Same class as D2; the
//!   alternative is embedding Geist Mono SemiBold in a future foundation
//!   change. The requirement (600) stays frozen.
//! * D3 (note): headings render in Inter (the sans family); whether the
//!   original uses `--font-heading` is OPEN-3. No visual difference in the
//!   default preset.
//! * D4 (limitation): heading levels, list semantics, and blockquote
//!   semantics have no egui runtime meaning; their visual consequences are
//!   fully specified above.
//! * D5 (limitation): `text-balance`, `scroll-m-20`, and sibling
//!   auto-margins are not portable to egui; headings wrap greedily and
//!   margins follow the caller-applied convention above.
//! * D6 (note): [`CodeBlock`] is an invention (INVENTED), judged against
//!   spec §2.4 only — never against the original, and never against the
//!   unrelated `.cn-code-block*` style rules or docs code-figure rules.
//!
//! # OPEN items (spec §12) — all left open, none guessed
//!
//! OPEN-1 (quoted recipes exact?), OPEN-2 (§4 margin values), OPEN-3
//! (heading font family), OPEN-4 (nested list behaviour), OPEN-6
//! (responsive variants) stay open. OPEN-5 (CodeBlock text metrics) was
//! CONFIRMED by the Coordinator at spec sign-off: size/leading/weight
//! inherit the surrounding text (16px/24px/400 in docs context), colour
//! inherits `foreground`.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! H1::new("Taxing Laughter: The Joke Tax Chronicles").show(ui);
//! P::new("Once upon a time...").show(ui);
//! Blockquote::new("After tax reform...").show(ui);
//! ```

use crate::theme::fonts;
use crate::theme::ShadcnTheme;
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};

/// Text size for each recipe, in points (TAILWIND-DEFAULT `text-*` scale).
pub mod sizes {
    pub const H1: f32 = crate::theme::tokens::type_scale::X4L; // text-4xl
    pub const H2: f32 = crate::theme::tokens::type_scale::X3L; // text-3xl
    pub const H3: f32 = crate::theme::tokens::type_scale::X2L; // text-2xl
    pub const H4: f32 = crate::theme::tokens::type_scale::XL; // text-xl
    pub const LEAD: f32 = crate::theme::tokens::type_scale::XL; // text-xl
    pub const LARGE: f32 = crate::theme::tokens::type_scale::LG; // text-lg
    pub const BODY: f32 = crate::theme::tokens::type_scale::BASE; // text-base
    pub const SMALL: f32 = crate::theme::tokens::type_scale::SM; // text-sm
}

/// Tailwind line heights paired with each recipe, in points.
///
/// TAILWIND-DEFAULT pairings (`text-4xl` → 40, ..., `text-sm` → 20) plus the
/// QUOTED recipe overrides (`leading-7` on [`P`], `leading-none` on
/// [`Small`]).
pub mod line_heights {
    /// `text-4xl`.
    pub const H1: f32 = 40.0;
    /// `text-3xl`.
    pub const H2: f32 = 36.0;
    /// `text-2xl`.
    pub const H3: f32 = 32.0;
    /// `text-xl` (also `text-lg`).
    pub const H4: f32 = 28.0;
    /// `text-base`.
    pub const BODY: f32 = 24.0;
    /// `leading-7` — the paragraph override on `text-base`.
    pub const P: f32 = 28.0;
    /// `text-xl` — the lead recipe.
    pub const LEAD: f32 = 28.0;
    /// `text-lg`.
    pub const LARGE: f32 = 28.0;
    /// `leading-none` — the small-text override (`text-sm` = 14px).
    pub const SMALL_NONE: f32 = 14.0;
    /// `text-sm`.
    pub const SMALL: f32 = 20.0;
}

/// `pl-6` + `border-l-2` for blockquotes, in points (DERIVED: 24 + 2).
pub(crate) const BLOCKQUOTE_PAD: f32 = 26.0;
/// `p-4` around code blocks.
pub(crate) const CODE_PAD: f32 = 16.0;
/// `ml-6` list indent.
pub(crate) const LIST_INDENT: f32 = 24.0;
/// `[&>li]:mt-2` — 8px top margin on EVERY `li`, including the first.
pub(crate) const LIST_ITEM_GAP: f32 = 8.0;
/// `pb-2` — 8px padding between the H2 text block and its bottom rule.
pub(crate) const H2_BOTTOM_PAD: f32 = 8.0;
/// `border-b` — 1px bottom rule on H2.
pub(crate) const H2_BORDER_WIDTH: f32 = 1.0;

/// The typographic style of one recipe, forwarded to the galley layout
/// ([`egui::text::TextFormat`]).
pub(crate) struct TextStyle {
    /// Font (family + size), selecting the recipe's weight via the Phase 1
    /// font helpers — egui has no weight field, so each weight is a family.
    font: FontId,
    /// Text color.
    color: Color32,
    /// Tailwind line height in points, if the recipe sets one.
    line_height: Option<f32>,
    /// `italic`.
    italics: bool,
}

impl TextStyle {
    /// A style with an explicit font (weight + face + size).
    fn new(font: FontId, color: Color32, line_height: Option<f32>) -> Self {
        Self {
            font,
            color,
            line_height,
            italics: false,
        }
    }

    /// Sets `italic`.
    fn italic(mut self) -> Self {
        self.italics = true;
        self
    }
}

/// Lays out a block of text wrapped to `wrap_width` in the given style.
pub(crate) fn galley_for(
    ui: &Ui,
    text: &str,
    style: TextStyle,
    wrap_width: f32,
) -> std::sync::Arc<egui::text::Galley> {
    let mut job =
        egui::text::LayoutJob::simple(text.to_owned(), style.font, style.color, wrap_width);
    job.sections[0].format.line_height = style.line_height;
    job.sections[0].format.italics = style.italics;
    ui.painter().layout_job(job)
}

/// Block-level heading text (left aligned, wraps at the available width).
fn heading(ui: &mut Ui, text: impl Into<String>, font: FontId, line_height: f32) -> Response {
    let theme = ShadcnTheme::of_ui(ui);
    let rich = egui::RichText::new(text.into())
        .font(font)
        .line_height(Some(line_height))
        .color(theme.palette.foreground);
    ui.add(egui::Label::new(rich))
}

/// `text-4xl font-extrabold tracking-tight` (QUOTED).
///
/// Weight 800 renders at SemiBold (600) — proposed deviation D2, no 800
/// face is embedded. Letter-spacing renders 0 — proposed deviation D1.
#[derive(Clone, Debug)]
pub struct H1 {
    text: String,
}

impl H1 {
    /// Creates an h1 heading.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: SemiBold 36px (D2 — required 800, closest available).
    pub(crate) fn font() -> FontId {
        fonts::semibold(sizes::H1)
    }

    /// Renders the heading.
    pub fn show(self, ui: &mut Ui) -> Response {
        heading(ui, self.text, Self::font(), line_heights::H1)
    }
}

impl Widget for H1 {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-3xl font-semibold tracking-tight border-b pb-2` (QUOTED).
///
/// Letter-spacing renders 0 — proposed deviation D1.
#[derive(Clone, Debug)]
pub struct H2 {
    text: String,
}

impl H2 {
    /// Creates an h2 heading.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: SemiBold 30px.
    pub(crate) fn font() -> FontId {
        fonts::semibold(sizes::H2)
    }

    /// Renders the heading with its full-width bottom border.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let style = TextStyle::new(
            Self::font(),
            theme.palette.foreground,
            Some(line_heights::H2),
        );
        let galley = galley_for(ui, &self.text, style, ui.available_width());
        let height = galley.size().y + H2_BOTTOM_PAD + H2_BORDER_WIDTH;
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.galley(rect.min, galley, Color32::WHITE);
            // `border-b`: 1px rule, `border` token, spanning the full
            // content width, centred in the last pixel row of the box.
            let rule_y = rect.bottom() - H2_BORDER_WIDTH / 2.0;
            painter.line_segment(
                [
                    Pos2::new(rect.left(), rule_y),
                    Pos2::new(rect.right(), rule_y),
                ],
                egui::Stroke::new(H2_BORDER_WIDTH, theme.palette.border),
            );
        }
        response
    }
}

impl Widget for H2 {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-2xl font-semibold tracking-tight` (QUOTED).
///
/// Letter-spacing renders 0 — proposed deviation D1.
#[derive(Clone, Debug)]
pub struct H3 {
    text: String,
}

impl H3 {
    /// Creates an h3 heading.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: SemiBold 24px.
    pub(crate) fn font() -> FontId {
        fonts::semibold(sizes::H3)
    }

    /// Renders the heading.
    pub fn show(self, ui: &mut Ui) -> Response {
        heading(ui, self.text, Self::font(), line_heights::H3)
    }
}

impl Widget for H3 {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-xl font-semibold tracking-tight` (QUOTED).
///
/// Letter-spacing renders 0 — proposed deviation D1.
#[derive(Clone, Debug)]
pub struct H4 {
    text: String,
}

impl H4 {
    /// Creates an h4 heading.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: SemiBold 20px.
    pub(crate) fn font() -> FontId {
        fonts::semibold(sizes::H4)
    }

    /// Renders the heading.
    pub fn show(self, ui: &mut Ui) -> Response {
        heading(ui, self.text, Self::font(), line_heights::H4)
    }
}

impl Widget for H4 {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-base leading-7` (QUOTED). Weight inherited (400).
#[derive(Clone, Debug)]
pub struct P {
    text: String,
}

impl P {
    /// Creates a body paragraph.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: Regular 16px.
    pub(crate) fn font() -> FontId {
        fonts::regular(sizes::BODY)
    }

    /// Renders the paragraph (wraps to the available width).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let style = TextStyle::new(
            Self::font(),
            theme.palette.foreground,
            Some(line_heights::P),
        );
        let galley = galley_for(ui, &self.text, style, ui.available_width());
        let size = galley.size();
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, Color32::WHITE);
        }
        response
    }
}

impl Widget for P {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `border-l-2 pl-6 italic` (+ `mt-6`, caller-applied) (QUOTED). Size and
/// leading inherited (`text-base`, 16px/24px); weight inherited (400).
#[derive(Clone, Debug)]
pub struct Blockquote {
    text: String,
}

impl Blockquote {
    /// Creates a blockquote.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: Regular 16px.
    pub(crate) fn font() -> FontId {
        fonts::regular(sizes::BODY)
    }

    /// Renders the italic quote with its 2px left border.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let style = TextStyle::new(
            Self::font(),
            theme.palette.foreground,
            Some(line_heights::BODY),
        )
        .italic();
        let galley = galley_for(
            ui,
            &self.text,
            style,
            (ui.available_width() - BLOCKQUOTE_PAD).max(0.0),
        );
        let width = (ui.available_width()).max(BLOCKQUOTE_PAD);
        let height = galley.size().y.max(crate::theme::tokens::type_scale::BASE);
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            // `border-l-2`
            painter.rect_filled(
                Rect::from_min_size(rect.min, Vec2::new(2.0, height)),
                0,
                theme.palette.border,
            );
            // `pl-6 italic`
            painter.galley(
                rect.min + Vec2::new(BLOCKQUOTE_PAD, 0.0),
                galley,
                Color32::WHITE,
            );
        }
        response
    }
}

impl Widget for Blockquote {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A list marker kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListMarker {
    /// `list-disc` — filled round bullets.
    Disc,
    /// `list-decimal` — `1.`, `2.`, ...
    Decimal,
}

/// `my-6 ml-6 list-disc [&>li]:mt-2` (and the `list-decimal` variant)
/// (QUOTED). Size/leading/weight inherited (`text-base`, 16px/24px/400).
/// The `my-6` block margins are caller-applied (spec §4).
///
/// ```ignore
/// Ul::new(&["Shipping", "Delivery", "Returns"]).show(ui);
/// Ol::new(&["First", "Second", "Third"]).show(ui);
/// ```
#[derive(Clone, Debug)]
pub struct Ul {
    items: Vec<String>,
    marker: ListMarker,
}

impl Ul {
    /// Creates an unordered (disc) list.
    pub fn new(items: &[&str]) -> Self {
        Self {
            items: items.iter().map(|s| (*s).to_owned()).collect(),
            marker: ListMarker::Disc,
        }
    }

    /// Switches to decimal numbering (the `Ol` recipe).
    pub fn decimal(mut self) -> Self {
        self.marker = ListMarker::Decimal;
        self
    }

    /// The recipe font: Regular 16px.
    pub(crate) fn font() -> FontId {
        fonts::regular(sizes::BODY)
    }

    /// Renders the list.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let fg = theme.palette.foreground;
        let marker_kind = self.marker;
        // `ml-6`: apply the 24px indent by an explicit offset. Do NOT
        // use `ui.indent` here: its first argument is an id salt, not
        // an amount — the actual indent is `spacing().indent` (18px
        // under the gallery theme), so the 24px constant never
        // reached the layout (defect TYP-VIS-02), and `indent_dyn`
        // paints an indent guide line the list recipe does not
        // contain (defect TYP-VIS-03). This mirrors `indent_dyn`'s
        // child-Ui construction minus the guide paint.
        let mut child_rect = ui.available_rect_before_wrap();
        child_rect.min.x += LIST_INDENT;
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .id_salt("typography_list")
                .max_rect(child_rect),
        );
        let painter = child_ui.painter().clone();
        // `[&>li]:mt-2` selects EVERY direct child, including the
        // first: the first item sits 8px below the list's top edge,
        // and consecutive items are 8px apart.
        child_ui.add_space(LIST_ITEM_GAP);
        child_ui.spacing_mut().item_spacing.y = LIST_ITEM_GAP;
        for (i, item) in self.items.iter().enumerate() {
            // Do NOT use `ui.horizontal` here: it seeds the row
            // height to `spacing().interact_size.y` (36px under the
            // crate theme), so each item row allocates 36px instead
            // of its 24px content (defect TYP-VIS-01, same mechanism
            // as KBD-VIS-01). Seed the row with zero height instead:
            // each child's frame is then exactly the child's own
            // height and the row is the tallest child. Direction
            // handling mirrors `Ui::horizontal`.
            let layout = if child_ui.layout().prefer_right_to_left() {
                egui::Layout::right_to_left(egui::Align::Center)
            } else {
                egui::Layout::left_to_right(egui::Align::Center)
            };
            let initial_size = Vec2::new(child_ui.available_size_before_wrap().x, 0.0);
            child_ui.allocate_ui_with_layout(initial_size, layout, |ui| {
                match marker_kind {
                    ListMarker::Disc => {
                        // A filled dot on the text mid-line.
                        let style = TextStyle::new(Self::font(), fg, Some(line_heights::BODY));
                        let galley = galley_for(ui, item, style, f32::INFINITY);
                        let h = galley.size().y;
                        let (rect, _) = ui.allocate_exact_size(Vec2::new(6.0, h), Sense::hover());
                        painter.circle_filled(
                            Pos2::new(rect.center().x, rect.min.y + h * 0.5),
                            2.5,
                            fg,
                        );
                        ui.label(
                            egui::RichText::new(item.clone())
                                .font(Self::font())
                                .line_height(Some(line_heights::BODY))
                                .color(fg),
                        );
                    }
                    ListMarker::Decimal => {
                        ui.label(
                            egui::RichText::new(format!("{}. ", i + 1))
                                .font(Self::font())
                                .line_height(Some(line_heights::BODY))
                                .color(fg),
                        );
                        ui.label(
                            egui::RichText::new(item.clone())
                                .font(Self::font())
                                .line_height(Some(line_heights::BODY))
                                .color(fg),
                        );
                    }
                }
            });
        }
        ui.allocate_rect(child_ui.min_rect(), Sense::hover())
    }
}

impl Widget for Ul {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `my-6 ml-6 list-decimal [&>li]:mt-2` — an ordered list (QUOTED).
#[derive(Clone, Debug)]
pub struct Ol {
    items: Vec<String>,
}

impl Ol {
    /// Creates an ordered (decimal) list.
    pub fn new(items: &[&str]) -> Self {
        Self {
            items: items.iter().map(|s| (*s).to_owned()).collect(),
        }
    }

    /// Renders the list.
    pub fn show(self, ui: &mut Ui) -> Response {
        Ul {
            items: self.items,
            marker: ListMarker::Decimal,
        }
        .show(ui)
    }
}

impl Widget for Ol {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold`
/// (QUOTED).
///
/// Plain Tailwind `rounded` is 4px — it is not remapped by shadcn's
/// `--radius-*` block. Weight 600 renders at Geist Mono Medium (500) —
/// proposed deviation D7 (provisional numbering), no 600 mono face is
/// embedded.
#[derive(Clone, Debug)]
pub struct InlineCode {
    text: String,
}

impl InlineCode {
    /// Creates inline code.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: Geist Mono Medium 14px (D7 — required 600, closest
    /// available mono face).
    pub(crate) fn font() -> FontId {
        fonts::mono_medium(sizes::SMALL)
    }

    /// Renders the code chip.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // Laid out once with the paint color: the same galley sizes the
        // chip and paints the text.
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), Self::font(), p.foreground);
        // px-[0.3rem] py-[0.2rem] → 4.8 / 3.2 points.
        let width = galley.size().x + 9.6;
        let height = line_heights::SMALL + 6.4;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, 4, p.muted); // plain `rounded` = 4px
                                                   // Same paint math as `Painter::text`, reusing the galley.
            let text_rect = Align2::CENTER_CENTER.anchor_size(rect.center(), galley.size());
            painter.galley(text_rect.min, galley, p.foreground);
        }
        response
    }
}

impl Widget for InlineCode {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A fenced code block — a documented porter's INVENTION (spec §2.4, note
/// D6). No source recipe exists; the invention is `rounded-md bg-muted p-4
/// font-mono`. Text metrics inherit the surrounding text (16px/24px/400,
/// `foreground`) per OPEN-5, CONFIRMED by the Coordinator at spec sign-off.
/// This element must NOT import values from the unrelated `.cn-code-block*`
/// style rules or the docs code-figure rules, and it uses the `muted` token,
/// not the `code` token.
#[derive(Clone, Debug)]
pub struct CodeBlock {
    text: String,
}

impl CodeBlock {
    /// Creates a code block; newlines become lines.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The invention's font: Geist Mono Regular 16px (inherited metrics,
    /// OPEN-5 confirmed).
    pub(crate) fn font() -> FontId {
        fonts::mono(sizes::BODY)
    }

    /// Renders the block (wraps long lines to the available width).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let style = TextStyle::new(Self::font(), p.foreground, Some(line_heights::BODY));
        let max_text_w = (ui.available_width() - 2.0 * CODE_PAD).max(0.0);
        let galley = galley_for(ui, &self.text, style, max_text_w);
        let width = ui.available_width();
        let height = galley.size().y + 2.0 * CODE_PAD;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, theme.radius.md_u8(), p.muted);
            painter.galley(rect.min + Vec2::splat(CODE_PAD), galley, Color32::WHITE);
        }
        response
    }
}

impl Widget for CodeBlock {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-xl text-muted-foreground` — an intro paragraph (QUOTED). Weight
/// inherited (400).
#[derive(Clone, Debug)]
pub struct Lead {
    text: String,
}

impl Lead {
    /// Creates lead text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: Regular 20px.
    pub(crate) fn font() -> FontId {
        fonts::regular(sizes::LEAD)
    }

    /// Renders the lead (wraps).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let style = TextStyle::new(
            Self::font(),
            theme.palette.muted_foreground,
            Some(line_heights::LEAD),
        );
        let galley = galley_for(ui, &self.text, style, ui.available_width());
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, Color32::WHITE);
        }
        response
    }
}

impl Widget for Lead {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-lg font-semibold` (QUOTED).
#[derive(Clone, Debug)]
pub struct Large {
    text: String,
}

impl Large {
    /// Creates large text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: SemiBold 18px.
    pub(crate) fn font() -> FontId {
        fonts::semibold(sizes::LARGE)
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        ui.label(
            egui::RichText::new(self.text)
                .font(Self::font())
                .line_height(Some(line_heights::LARGE))
                .color(theme.palette.foreground),
        )
    }
}

impl Widget for Large {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-sm font-medium leading-none` (QUOTED).
#[derive(Clone, Debug)]
pub struct Small {
    text: String,
}

impl Small {
    /// Creates small text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: Medium 14px.
    pub(crate) fn font() -> FontId {
        fonts::medium(sizes::SMALL)
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        ui.label(
            egui::RichText::new(self.text)
                .font(Self::font())
                .line_height(Some(line_heights::SMALL_NONE))
                .color(theme.palette.foreground),
        )
    }
}

impl Widget for Small {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-sm text-muted-foreground` (QUOTED). Weight inherited (400).
#[derive(Clone, Debug)]
pub struct Muted {
    text: String,
}

impl Muted {
    /// Creates muted text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// The recipe font: Regular 14px.
    pub(crate) fn font() -> FontId {
        fonts::regular(sizes::SMALL)
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        ui.label(
            egui::RichText::new(self.text)
                .font(Self::font())
                .line_height(Some(line_heights::SMALL))
                .color(theme.palette.muted_foreground),
        )
    }
}

impl Widget for Muted {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-sm text-muted-foreground` — a table caption (QUOTED). Same recipe
/// as [`Muted`]; placement and alignment are the table's concern (spec §9).
#[derive(Clone, Debug)]
pub struct TableCaption {
    text: String,
}

impl TableCaption {
    /// Creates a table caption.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the caption.
    pub fn show(self, ui: &mut Ui) -> Response {
        Muted::new(self.text).show(ui)
    }
}

impl Widget for TableCaption {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Preset;
    use egui::FontFamily;

    #[test]
    fn sizes_match_the_class_recipes() {
        assert_eq!(sizes::H1, 36.0); // text-4xl
        assert_eq!(sizes::H2, 30.0); // text-3xl
        assert_eq!(sizes::H3, 24.0); // text-2xl
        assert_eq!(sizes::H4, 20.0); // text-xl
        assert_eq!(sizes::LEAD, 20.0); // text-xl
        assert_eq!(sizes::LARGE, 18.0); // text-lg
        assert_eq!(sizes::BODY, 16.0); // text-base
        assert_eq!(sizes::SMALL, 14.0); // text-sm
    }

    #[test]
    fn line_heights_match_the_tailwind_scale() {
        assert_eq!(line_heights::H1, 40.0); // text-4xl
        assert_eq!(line_heights::H2, 36.0); // text-3xl
        assert_eq!(line_heights::H3, 32.0); // text-2xl
        assert_eq!(line_heights::H4, 28.0); // text-xl
        assert_eq!(line_heights::BODY, 24.0); // text-base
        assert_eq!(line_heights::P, 28.0); // leading-7
        assert_eq!(line_heights::LEAD, 28.0); // text-xl
        assert_eq!(line_heights::LARGE, 28.0); // text-lg
        assert_eq!(line_heights::SMALL_NONE, 14.0); // leading-none on text-sm
        assert_eq!(line_heights::SMALL, 20.0); // text-sm
    }

    #[test]
    fn spacing_translations() {
        // border-l-2 + pl-6
        assert_eq!(BLOCKQUOTE_PAD, 26.0);
        // p-4
        assert_eq!(CODE_PAD, 16.0);
        // ml-6
        assert_eq!(LIST_INDENT, 24.0);
        // [&>li]:mt-2
        assert_eq!(LIST_ITEM_GAP, 8.0);
        // pb-2 + border-b
        assert_eq!(H2_BOTTOM_PAD, 8.0);
        assert_eq!(H2_BORDER_WIDTH, 1.0);
    }

    #[test]
    fn weights_match_the_spec() {
        // SemiBold (600): H1 per D2 (required 800), H2/H3/H4, Large.
        assert_eq!(H1::font(), fonts::semibold(36.0));
        assert_eq!(H2::font(), fonts::semibold(30.0));
        assert_eq!(H3::font(), fonts::semibold(24.0));
        assert_eq!(H4::font(), fonts::semibold(20.0));
        assert_eq!(Large::font(), fonts::semibold(18.0));
        assert_eq!(H1::font().family, fonts::semibold_family());
        // Medium (500): Small.
        assert_eq!(Small::font(), fonts::medium(14.0));
        assert_eq!(Small::font().family, fonts::medium_family());
        // Regular (400): P, Blockquote, Ul/Ol, Lead, Muted.
        assert_eq!(P::font(), fonts::regular(16.0));
        assert_eq!(Blockquote::font(), fonts::regular(16.0));
        assert_eq!(Ul::font(), fonts::regular(16.0));
        assert_eq!(Lead::font(), fonts::regular(20.0));
        assert_eq!(Muted::font(), fonts::regular(14.0));
        assert_eq!(P::font().family, FontFamily::Proportional);
        // Mono Medium (500): InlineCode per proposed D7 (required 600).
        assert_eq!(InlineCode::font(), fonts::mono_medium(14.0));
        assert_eq!(InlineCode::font().family, fonts::mono_medium_family());
        // Mono Regular (400): CodeBlock per OPEN-5 (inherited metrics).
        assert_eq!(CodeBlock::font(), fonts::mono(16.0));
        assert_eq!(CodeBlock::font().family, FontFamily::Monospace);
    }

    #[test]
    fn list_marker_variants() {
        let ul = Ul::new(&["a"]).decimal();
        assert_eq!(ul.marker, ListMarker::Decimal);
        let ul = Ul::new(&["a"]);
        assert_eq!(ul.marker, ListMarker::Disc);
        // Ol wraps Ul with the decimal marker.
        let ol = Ol::new(&["a", "b"]);
        assert_eq!(ol.items.len(), 2);
    }

    #[test]
    fn builders_keep_text() {
        assert_eq!(H1::new("T").text, "T");
        assert_eq!(
            CodeBlock::new("let x = 1;\nlet y = 2;")
                .text
                .lines()
                .count(),
            2
        );
        assert_eq!(Muted::new("m").text, "m");
    }

    /// Builds a bare `Ui` over a fresh context for `show()` geometry tests.
    fn test_ui(ctx: &egui::Context, id: &str, w: f32, h: f32) -> egui::Ui {
        egui::Ui::new(
            ctx.clone(),
            egui::Id::unique(id),
            egui::UiBuilder::new().max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(w, h))),
        )
    }

    #[test]
    fn h2_allocates_text_plus_padding_plus_rule() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "typo_h2", 400.0, 400.0);
            let response = H2::new("Title").show(&mut ui);
            // One 36px row + pb-2 (8px) + the 1px rule.
            assert!(
                (response.rect.height() - 45.0).abs() < 0.5,
                "H2 height should be 45px, got {}",
                response.rect.height()
            );
            assert!(
                (response.rect.width() - 400.0).abs() < 0.5,
                "H2 spans the full content width, got {}",
                response.rect.width()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn list_first_item_sits_8px_below_the_top_edge() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "typo_ul1", 400.0, 400.0);
            let one = Ul::new(&["a"]).show(&mut ui);
            // 8px `[&>li]:mt-2` + one 24px row.
            assert!(
                (one.rect.height() - 32.0).abs() < 0.5,
                "one-item list should be 32px tall, got {}",
                one.rect.height()
            );
            let mut ui = test_ui(ctx, "typo_ul2", 400.0, 400.0);
            let two = Ul::new(&["a", "b"]).show(&mut ui);
            // 8 + 24 + 8 + 24.
            assert!(
                (two.rect.height() - 64.0).abs() < 0.5,
                "two-item list should be 64px tall, got {}",
                two.rect.height()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn list_geometry_under_gallery_theme_spacing() {
        // Defects TYP-VIS-01/02: under the gallery theme spacing the
        // list must still allocate 8 + 3×24 + 2×8 = 96px with a real
        // 24px `ml-6` indent. The installed preset is essential: it
        // applies `interact_size.y` = 36px (which the old
        // `ui.horizontal` rows inherited, TYP-VIS-01) while
        // `spacing().indent` stays 18px (which the old `ui.indent`
        // call used instead of the 24px constant, TYP-VIS-02).
        // Default egui spacing would prove less (the typography S1
        // lesson: a test passing only because the test context uses
        // default spacing proves nothing).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "typo_list_theme", 400.0, 400.0);
            assert_eq!(
                ui.spacing().interact_size.y,
                crate::theme::tokens::spacing::CONTROL,
                "the gallery theme spacing must be in effect for this test to discriminate"
            );
            assert_eq!(
                ui.spacing().indent,
                18.0,
                "the gallery theme indent must be in effect for this test to discriminate"
            );
            let ul = Ul::new(&["a", "b", "c"]).show(&mut ui);
            // 8 + 3×24 + 2×8 = 96.
            assert!(
                (ul.rect.height() - 96.0).abs() < 0.5,
                "three-item list should be 96px tall, got {}",
                ul.rect.height()
            );
            // The test Ui starts at x=0, so the `ml-6` indent puts
            // the list rect's left edge at exactly 24px.
            assert!(
                (ul.rect.min.x - LIST_INDENT).abs() < 0.5,
                "list rect should start at the 24px indent, got x={}",
                ul.rect.min.x
            );
            let mut ui = test_ui(ctx, "typo_list_theme_ol", 400.0, 400.0);
            let ol = Ol::new(&["a", "b", "c"]).show(&mut ui);
            assert!(
                (ol.rect.height() - 96.0).abs() < 0.5,
                "three-item ordered list should be 96px tall, got {}",
                ol.rect.height()
            );
            assert!(
                (ol.rect.min.x - LIST_INDENT).abs() < 0.5,
                "ordered list rect should start at the 24px indent, got x={}",
                ol.rect.min.x
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn list_paints_nothing_left_of_the_indent() {
        // Defect TYP-VIS-03: the old `ui.indent` call painted an
        // indent guide vline at `child_rect.min − 0.5 × indent`
        // (x=9 under the gallery theme). No painted vertex may sit
        // left of the 24px indent edge (the disc starts at 24px;
        // the 1px tolerance covers anti-aliasing feathering).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "typo_list_paint", 400.0, 400.0);
            Ul::new(&["a", "b", "c"]).show(&mut ui);
        });
        // Clear before the assertions below: dropping `out` with an
        // unapplied delta panics, which would abort the test binary
        // instead of failing this test cleanly.
        out.textures_delta.clear();
        let primitives = ctx.tessellate(out.shapes.clone(), out.pixels_per_point);
        for primitive in &primitives {
            if let egui::epaint::Primitive::Mesh(mesh) = &primitive.primitive {
                for vertex in &mesh.vertices {
                    assert!(
                        vertex.pos.x >= LIST_INDENT - 1.0,
                        "painted vertex left of the indent at {:?}",
                        vertex.pos
                    );
                }
            }
        }
    }

    #[test]
    fn inline_code_chip_height_is_line_height_plus_padding() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "typo_code", 400.0, 400.0);
            let response = InlineCode::new("x").show(&mut ui);
            // 20px text-sm row + py-[0.2rem] (3.2px top and bottom).
            assert!(
                (response.rect.height() - 26.4).abs() < 0.5,
                "InlineCode height should be 26.4px, got {}",
                response.rect.height()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn codeblock_uses_inherited_text_metrics() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut test_ui(ctx, "typo_cb", 400.0, 400.0);
            // OPEN-5 confirmed: 16px mono at the inherited 24px leading.
            let style = TextStyle::new(CodeBlock::font(), Color32::WHITE, Some(24.0));
            let galley = galley_for(ui, "line one\nline two", style, 300.0);
            assert_eq!(galley.rows.len(), 2);
            assert!(
                (galley.size().y - 48.0).abs() < 0.5,
                "two CodeBlock rows at 24px leading should be 48px, got {}",
                galley.size().y
            );
            let mut ui = test_ui(ctx, "typo_cb_show", 400.0, 400.0);
            let response = CodeBlock::new("a\nb").show(&mut ui);
            // 48px of text + p-4 (16px top and bottom).
            assert!(
                (response.rect.height() - 80.0).abs() < 0.5,
                "CodeBlock height should be 80px, got {}",
                response.rect.height()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn blockquote_allocates_full_width_at_body_leading() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "typo_bq", 400.0, 400.0);
            let response = Blockquote::new("quote").show(&mut ui);
            assert!(
                (response.rect.height() - 24.0).abs() < 0.5,
                "one-row blockquote should be 24px tall, got {}",
                response.rect.height()
            );
            assert!(
                (response.rect.width() - 400.0).abs() < 0.5,
                "blockquote spans the full content width, got {}",
                response.rect.width()
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn galley_wrap_respects_width() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("typo_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 200.0))),
            );
            let long = "word ".repeat(60);
            let style = TextStyle::new(fonts::regular(16.0), Color32::WHITE, Some(24.0));
            let galley = galley_for(ui, &long, style, 100.0);
            assert!(galley.size().y > 40.0, "long text should wrap to many rows");
            let style = TextStyle::new(fonts::regular(16.0), Color32::WHITE, Some(24.0));
            let galley = galley_for(ui, &long, style, 1000.0);
            assert!(galley.size().y < 100.0);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn line_height_is_applied_to_the_galley() {
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("typo_lh_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            // One row at line-height 28 is exactly 28 tall; a wrapped block
            // is rows * line-height.
            let style = TextStyle::new(fonts::regular(16.0), Color32::WHITE, Some(28.0));
            let one = galley_for(ui, "hi", style, f32::INFINITY);
            assert!(
                (one.size().y - 28.0).abs() < 0.5,
                "one row at line-height 28 should be 28px, got {}",
                one.size().y
            );
            let style = TextStyle::new(fonts::regular(16.0), Color32::WHITE, Some(28.0));
            let wrapped = galley_for(ui, &"word ".repeat(30), style, 120.0);
            let rows = wrapped.rows.len() as f32;
            assert!(rows > 1.0);
            assert!(
                (wrapped.size().y - rows * 28.0).abs() < 0.5,
                "each row must honor the 28px line height, got {}",
                wrapped.size().y
            );
            let style = TextStyle::new(fonts::regular(16.0), Color32::WHITE, None).italic();
            let italic = galley_for(ui, "quote", style, 200.0);
            assert!(italic.size().y > 0.0);
        });
        out.textures_delta.clear();
    }
}
