//! shadcn typography recipes — text styles from the docs
//! (`apps/v4/content/docs/components/radix/typography.mdx` and its example
//! files). shadcn ships no typography component; this module turns the class
//! recipes into egui text helpers.
//!
//! Tailwind pairs every `text-*` size with a line height (`text-4xl` → 40px,
//! `text-3xl` → 36px, `text-2xl` → 32px, `text-xl`/`text-lg` → 28px,
//! `text-base` → 24px, `text-sm` → 20px) and the recipes override it where
//! needed (`leading-7` = 28px on paragraphs, `leading-none` = 14px on small
//! text); these are applied via egui's `line_height` so wrapped blocks match
//! the source's row spacing. Block margins are *not* applied automatically —
//! immediate-mode UIs have no sibling knowledge, so callers space blocks
//! themselves with the source margins listed below.
//!
//! | Element | Source recipe | Line height | Source margins (caller-applied) |
//! |---|---|---|---|
//! | [`H1`] | `text-4xl font-extrabold tracking-tight` | 40px | — |
//! | [`H2`] | `text-3xl font-semibold tracking-tight border-b pb-2` | 36px | `mt-10` (40px), `first:mt-0` |
//! | [`H3`] | `text-2xl font-semibold tracking-tight` | 32px | `mt-8` (32px) |
//! | [`H4`] | `text-xl font-semibold tracking-tight` | 28px | `mt-8` (32px) |
//! | [`P`] | `text-base leading-7` | 28px | `mt-6` (24px), `first:mt-0` |
//! | [`Blockquote`] | `mt-6 border-l-2 pl-6 italic` | 24px | `mt-6` (24px) |
//! | [`Ul`] / [`Ol`] | `my-6 ml-6 list-disc/list-decimal [&>li]:mt-2` | 24px | `my-6` (24px top and bottom) |
//! | [`InlineCode`] | `rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold` | 20px | — |
//! | [`CodeBlock`] | no source recipe (docs prose styling; the port uses `rounded-md bg-muted p-4 font-mono`) | default | — |
//! | [`Lead`] | `text-xl text-muted-foreground` | 28px | — |
//! | [`Large`] | `text-lg font-semibold` | 28px | — |
//! | [`Small`] | `text-sm font-medium leading-none` | 14px | — |
//! | [`Muted`] | `text-sm text-muted-foreground` | 20px | — |
//! | [`TableCaption`] | `text-sm text-muted-foreground` | 20px | `mt-4` (16px) |
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! H1::new("Taxing Laughter: The Joke Tax Chronicles").show(ui);
//! P::new("Once upon a time...").show(ui);
//! Blockquote::new("After tax reform...").show(ui);
//! ```

use crate::theme::ShadcnTheme;
use egui::{Align2, Color32, FontId, Pos2, Rect, Response, Sense, Ui, Vec2, Widget};

/// Text size for each recipe, in points (Tailwind text-*).
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
/// Defaults from the Tailwind v4 type scale (`text-4xl` → 40, ...,
/// `text-sm` → 20) plus the recipe overrides (`leading-7` on [`P`],
/// `leading-none` on [`Small`]).
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

/// `pl-6` + `border-l-2` for blockquotes, in points.
pub(crate) const BLOCKQUOTE_PAD: f32 = 26.0;
/// `p-4` around code blocks.
pub(crate) const CODE_PAD: f32 = 16.0;
/// `ml-6` list indent.
pub(crate) const LIST_INDENT: f32 = 24.0;

/// The typographic style of one recipe, forwarded to the galley layout
/// ([`egui::text::TextFormat`]).
pub(crate) struct TextStyle {
    /// Font size in points.
    size: f32,
    /// Text color.
    color: Color32,
    /// Monospace face (`font-mono`).
    mono: bool,
    /// Tailwind line height in points, if the recipe sets one.
    line_height: Option<f32>,
    /// `italic`.
    italics: bool,
}

impl TextStyle {
    /// A proportional-face style with a Tailwind line height.
    fn new(size: f32, color: Color32, line_height: Option<f32>) -> Self {
        Self {
            size,
            color,
            mono: false,
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
    let font = if style.mono {
        FontId::monospace(style.size)
    } else {
        FontId::proportional(style.size)
    };
    let mut job = egui::text::LayoutJob::simple(text.to_owned(), font, style.color, wrap_width);
    job.sections[0].format.line_height = style.line_height;
    job.sections[0].format.italics = style.italics;
    ui.painter().layout_job(job)
}

/// Block-level heading text (left aligned, no wrap needed).
fn heading(ui: &mut Ui, text: impl Into<String>, size: f32, line_height: f32) -> Response {
    let theme = ShadcnTheme::of_ui(ui);
    let rich = egui::RichText::new(text.into())
        .size(size)
        .line_height(Some(line_height))
        .color(theme.palette.foreground);
    ui.add(egui::Label::new(rich))
}

/// `text-4xl font-extrabold tracking-tight`.
#[derive(Clone, Debug)]
pub struct H1 {
    text: String,
}

impl H1 {
    /// Creates an h1 heading.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the heading.
    pub fn show(self, ui: &mut Ui) -> Response {
        heading(ui, self.text, sizes::H1, line_heights::H1)
    }
}

impl Widget for H1 {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-3xl font-semibold tracking-tight border-b pb-2`.
#[derive(Clone, Debug)]
pub struct H2 {
    text: String,
}

impl H2 {
    /// Creates an h2 heading.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the heading with its full-width bottom border.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let style = TextStyle::new(sizes::H2, theme.palette.foreground, Some(line_heights::H2));
        let galley = galley_for(ui, &self.text, style, ui.available_width());
        let pb = 8.0; // pb-2
        let height = galley.size().y + pb + 1.0;
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.galley(rect.min, galley, Color32::WHITE);
            // `border-b`
            painter.line_segment(
                [rect.left_bottom(), rect.right_bottom()],
                egui::Stroke::new(1.0, theme.palette.border),
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

/// `text-2xl font-semibold tracking-tight`.
#[derive(Clone, Debug)]
pub struct H3 {
    text: String,
}

impl H3 {
    /// Creates an h3 heading.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the heading.
    pub fn show(self, ui: &mut Ui) -> Response {
        heading(ui, self.text, sizes::H3, line_heights::H3)
    }
}

impl Widget for H3 {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-xl font-semibold tracking-tight`.
#[derive(Clone, Debug)]
pub struct H4 {
    text: String,
}

impl H4 {
    /// Creates an h4 heading.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the heading.
    pub fn show(self, ui: &mut Ui) -> Response {
        heading(ui, self.text, sizes::H4, line_heights::H4)
    }
}

impl Widget for H4 {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `text-base leading-7`.
#[derive(Clone, Debug)]
pub struct P {
    text: String,
}

impl P {
    /// Creates a body paragraph.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the paragraph (wraps to the available width).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let style = TextStyle::new(sizes::BODY, theme.palette.foreground, Some(line_heights::P));
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

/// `mt-6 border-l-2 pl-6 italic`.
#[derive(Clone, Debug)]
pub struct Blockquote {
    text: String,
}

impl Blockquote {
    /// Creates a blockquote.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the italic quote with its 2px left border.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let style = TextStyle::new(
            sizes::BODY,
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

/// `ml-6 list-disc [&>li]:mt-2` (and the `list-decimal` variant).
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

    /// Renders the list.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let fg = theme.palette.foreground;
        let marker_kind = self.marker;
        ui.indent(LIST_INDENT as i8, |ui| {
            let painter = ui.painter().clone();
            ui.spacing_mut().item_spacing.y = 8.0; // [&>li]:mt-2
            for (i, item) in self.items.iter().enumerate() {
                ui.horizontal(|ui| {
                    match marker_kind {
                        ListMarker::Disc => {
                            // A filled dot on the text mid-line.
                            let style = TextStyle::new(sizes::BODY, fg, Some(line_heights::BODY));
                            let galley = galley_for(ui, item, style, f32::INFINITY);
                            let h = galley.size().y;
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::new(6.0, h), Sense::hover());
                            painter.circle_filled(
                                Pos2::new(rect.center().x, rect.min.y + h * 0.5),
                                2.5,
                                fg,
                            );
                            ui.label(
                                egui::RichText::new(item.clone())
                                    .size(sizes::BODY)
                                    .line_height(Some(line_heights::BODY))
                                    .color(fg),
                            );
                        }
                        ListMarker::Decimal => {
                            ui.label(
                                egui::RichText::new(format!("{}. ", i + 1))
                                    .size(sizes::BODY)
                                    .line_height(Some(line_heights::BODY))
                                    .color(fg),
                            );
                            ui.label(
                                egui::RichText::new(item.clone())
                                    .size(sizes::BODY)
                                    .line_height(Some(line_heights::BODY))
                                    .color(fg),
                            );
                        }
                    }
                });
            }
        })
        .response
    }
}

impl Widget for Ul {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `ml-6 list-decimal [&>li]:mt-2` — an ordered list.
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

/// `rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold`.
///
/// Plain Tailwind `rounded` is 4px — it is not remapped by shadcn's
/// `--radius-*` block.
#[derive(Clone, Debug)]
pub struct InlineCode {
    text: String,
}

impl InlineCode {
    /// Creates inline code.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the code chip.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let size = sizes::SMALL;
        let font = FontId::monospace(size);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), p.foreground);
        // px-[0.3rem] py-[0.2rem] → 4.8 / 3.2 points.
        let width = galley.size().x + 9.6;
        let height = line_heights::SMALL + 6.4;
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            painter.rect_filled(rect, 4, p.muted); // plain `rounded` = 4px
            painter.text(
                rect.center(),
                Align2::CENTER_CENTER,
                self.text,
                font,
                p.foreground,
            );
        }
        response
    }
}

impl Widget for InlineCode {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// A fenced code block. The docs style their `pre` through prose CSS rather
/// than a component recipe, so this port's `rounded-md bg-muted p-4
/// font-mono` treatment is the porter's invention, matching the visual style
/// of the docs code blocks.
#[derive(Clone, Debug)]
pub struct CodeBlock {
    text: String,
}

impl CodeBlock {
    /// Creates a code block; newlines become lines.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the block (wraps long lines to the available width).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let size = sizes::SMALL;
        let font = FontId::monospace(size);
        let max_text_w = (ui.available_width() - 2.0 * CODE_PAD).max(0.0);
        let galley = ui
            .painter()
            .layout(self.text.clone(), font.clone(), p.foreground, max_text_w);
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

/// `text-xl text-muted-foreground` — an intro paragraph.
#[derive(Clone, Debug)]
pub struct Lead {
    text: String,
}

impl Lead {
    /// Creates lead text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the lead (wraps).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let style = TextStyle::new(
            sizes::LEAD,
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

/// `text-lg font-semibold`.
#[derive(Clone, Debug)]
pub struct Large {
    text: String,
}

impl Large {
    /// Creates large text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        ui.label(
            egui::RichText::new(self.text)
                .size(sizes::LARGE)
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

/// `text-sm font-medium leading-none`.
#[derive(Clone, Debug)]
pub struct Small {
    text: String,
}

impl Small {
    /// Creates small text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        ui.label(
            egui::RichText::new(self.text)
                .size(sizes::SMALL)
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

/// `text-sm text-muted-foreground`.
#[derive(Clone, Debug)]
pub struct Muted {
    text: String,
}

impl Muted {
    /// Creates muted text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        ui.label(
            egui::RichText::new(self.text)
                .size(sizes::SMALL)
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

/// `text-sm text-muted-foreground` — a table caption.
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
            let style = TextStyle::new(16.0, Color32::WHITE, Some(24.0));
            let galley = galley_for(ui, &long, style, 100.0);
            assert!(galley.size().y > 40.0, "long text should wrap to many rows");
            let style = TextStyle::new(16.0, Color32::WHITE, Some(24.0));
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
            let style = TextStyle::new(16.0, Color32::WHITE, Some(28.0));
            let one = galley_for(ui, "hi", style, f32::INFINITY);
            assert!(
                (one.size().y - 28.0).abs() < 0.5,
                "one row at line-height 28 should be 28px, got {}",
                one.size().y
            );
            let style = TextStyle::new(16.0, Color32::WHITE, Some(28.0));
            let wrapped = galley_for(ui, &"word ".repeat(30), style, 120.0);
            let rows = wrapped.rows.len() as f32;
            assert!(rows > 1.0);
            assert!(
                (wrapped.size().y - rows * 28.0).abs() < 0.5,
                "each row must honor the 28px line height, got {}",
                wrapped.size().y
            );
            let style = TextStyle::new(16.0, Color32::WHITE, None).italic();
            let italic = galley_for(ui, "quote", style, 200.0);
            assert!(italic.size().y > 0.0);
        });
        out.textures_delta.clear();
    }
}
