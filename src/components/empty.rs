//! shadcn `empty.tsx` — a placeholder for empty states.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/empty.tsx`: a centered
//! column (`p-6`, gap-6, `text-center`) with `border-dashed` (visible when
//! [`Empty::bordered`] is on — the source's default has no border width, the
//! docs pages add `border`), an [`EmptyHeader`] holding an [`EmptyMedia`],
//! [`EmptyTitle`] and [`EmptyDescription`], and an [`EmptyContent`] slot for
//! actions. The nova style tightens the gap to `gap-4`, the media icon box
//! to `size-8` with a `size-4` icon and the title to `text-sm`.
//!
//! Scope notes: the source pads to 48px at the `md` breakpoint
//! (`md:p-12`); the port defaults to the fixed `p-6` = 24px, and
//! [`Empty::padding`] lets a caller opt into 48px (spec §3.1, D4 — egui
//! has no `md` breakpoint). [`EmptyFooter`] is an invented extension
//! (see its docs).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Empty::new().bordered(true).show(ui, |ui| {
//!     EmptyHeader::new().show(ui, |ui| {
//!         EmptyMedia::icon(Icon::Search).show(ui);
//!         EmptyTitle::new("No results found").show(ui);
//!         EmptyDescription::new("Try a different search term.").show(ui);
//!     });
//!     EmptyContent::new().show(ui, |ui| {
//!         Button::new("Clear filters").variant(ButtonVariant::Outline).show(ui);
//!     });
//! });
//! ```

use crate::icons::Icon;
use crate::theme::fonts;
use crate::theme::{Palette, ShadcnTheme, Style};
use egui::{Align, Color32, FontId, InnerResponse, Margin, Pos2, Rect, Shape, Stroke, Ui, Vec2};

/// Media variants, mirroring the source `emptyMediaVariants`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptyMediaVariant {
    /// `bg-transparent` — free-form media (images etc.).
    #[default]
    Default,
    /// `size-10 rounded-lg bg-muted text-foreground` icon box (nova: size-8).
    Icon,
}

/// The empty container's vertical gap (base `gap-6`, nova `gap-4`).
pub(crate) fn gap(style: Style) -> f32 {
    match style {
        Style::Nova => 16.0,
        _ => 24.0,
    }
}

/// The icon media box side (base `size-10`, nova `size-8`).
pub(crate) fn media_icon_box(style: Style) -> f32 {
    match style {
        Style::Nova => 32.0,
        _ => 40.0,
    }
}

/// The icon media glyph size (base `size-6`, nova `size-4`).
pub(crate) fn media_icon_size(style: Style) -> f32 {
    match style {
        Style::Nova => 16.0,
        _ => 24.0,
    }
}

/// The title size (base `text-lg`, nova `text-sm`).
pub(crate) fn title_size(style: Style) -> f32 {
    match style {
        Style::Nova => crate::theme::tokens::type_scale::SM,
        _ => crate::theme::tokens::type_scale::LG,
    }
}

/// The title font: `font-medium` (500) via the embedded Inter Medium face
/// (spec §2.4, §4 — REQUIRED, not a deviation).
pub(crate) fn title_font(style: Style) -> FontId {
    fonts::medium(title_size(style))
}

/// The title line-height (base `text-lg` 28px, nova `text-sm` 20px, spec §3.3).
pub(crate) fn title_line_height(style: Style) -> f32 {
    match style {
        Style::Nova => 20.0,
        _ => 28.0,
    }
}

/// The content slot's vertical gap (base `gap-4`, nova `gap-2.5`, spec §2.6).
pub(crate) fn content_gap(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0,
        _ => 16.0,
    }
}

/// The header's vertical gap (`gap-2`, spec §2.2).
pub(crate) const HEADER_GAP: f32 = 8.0;

/// The container padding when no override is set (`p-6` = 24px, spec §3.1).
pub const EMPTY_DEFAULT_PADDING: f32 = 24.0;

/// `max-w-sm` (24rem = 384px), shared by header/description/content.
pub const EMPTY_MAX_WIDTH: f32 = 384.0;

/// `text-sm/relaxed` line-height: 14px × 1.625 = 22.75px (spec §2.5,
/// §0 correction 1). The audit's 20.0 (`line_heights::SMALL`) is wrong:
/// the `/relaxed` modifier overrides the `text-sm` default line-height
/// with `--leading-relaxed` (1.625).
pub(crate) const DESCRIPTION_LINE_HEIGHT: f32 = 22.75;

/// `underline-offset-4`: a description link's underline sits 4px below the
/// text baseline (spec §2.5; same value as the badge link variant).
const LINK_UNDERLINE_OFFSET: f32 = 4.0;

/// One inline segment of an [`EmptyDescription`].
#[derive(Clone, Debug, PartialEq, Eq)]
enum DescriptionSegment {
    /// Plain description text (`text-muted-foreground`).
    Text(String),
    /// A direct-child link: underlined, 4px offset, `text-primary` on hover.
    Link(String),
}

/// Builds the description [`egui::text::LayoutJob`] for the given segments.
/// Plain segments paint in `base_color`; link segments in `link_color`. All
/// segments share the `text-sm/relaxed` metrics (14px, 22.75px line-height).
/// The caller sets `wrap.max_width`; `halign` is centred here.
fn description_job(
    segments: &[DescriptionSegment],
    base_color: Color32,
    link_color: Color32,
) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    for segment in segments {
        let (text, color) = match segment {
            DescriptionSegment::Text(text) => (text.as_str(), base_color),
            DescriptionSegment::Link(text) => (text.as_str(), link_color),
        };
        job.append(
            text,
            0.0,
            egui::TextFormat {
                font_id: FontId::proportional(crate::theme::tokens::type_scale::SM),
                color,
                line_height: Some(DESCRIPTION_LINE_HEIGHT),
                ..egui::TextFormat::default()
            },
        );
    }
    job.halign = Align::Center;
    job
}

/// Resolves a description link's colour: `text-primary` on hover, otherwise
/// `text-muted-foreground` (spec §2.5).
fn resolve_link_color(hovered: bool, p: &Palette) -> Color32 {
    if hovered {
        p.primary
    } else {
        p.muted_foreground
    }
}

/// One painted underline for a link run on a single galley row, plus the
/// pointer-hit rect for that run. Coordinates are relative to the galley's
/// origin (the paint position).
#[derive(Clone, Copy, Debug, PartialEq)]
struct LinkRun {
    /// Underline endpoints, 4px below the row baseline.
    from: Pos2,
    to: Pos2,
    /// Pointer-hit rect: the run's glyph span over the row's line box,
    /// expanded 2px vertically for forgiving hover.
    hit: Rect,
}

fn make_link_run(x0: f32, x1: f32, baseline: f32, row_top: f32, row_height: f32) -> LinkRun {
    let y = baseline + LINK_UNDERLINE_OFFSET;
    LinkRun {
        from: Pos2::new(x0, y),
        to: Pos2::new(x1, y),
        hit: Rect::from_min_max(
            Pos2::new(x0, row_top - 2.0),
            Pos2::new(x1, row_top + row_height + 2.0),
        ),
    }
}

/// Finds the underline runs for the link segments of a laid-out description
/// galley.
///
/// The glyphs of a [`egui::text::LayoutJob`] are laid out in char order
/// (one glyph per char, `epaint::text::Row`), so a link's glyphs are the
/// contiguous glyph range starting after the preceding segments' chars.
/// The scan verifies each glyph's `chr` against the expected text; if the
/// layout ever stops matching (a newline produces no glyph, a future
/// shaping change merges glyphs), the galley yields no runs rather than a
/// mis-painted underline. A link wrapping across rows underlines once per
/// visual row.
fn link_runs(galley: &egui::Galley, segments: &[DescriptionSegment]) -> Vec<LinkRun> {
    let mut expected: Vec<(char, bool)> = Vec::new();
    for segment in segments {
        let (text, is_link) = match segment {
            DescriptionSegment::Text(text) => (text.as_str(), false),
            DescriptionSegment::Link(text) => (text.as_str(), true),
        };
        expected.extend(text.chars().map(|c| (c, is_link)));
    }

    let mut runs = Vec::new();
    let mut idx = 0usize;
    for placed in &galley.rows {
        let row = &placed.row;
        let mut run: Option<(f32, f32)> = None; // (x0, baseline)
        let mut run_end_x = 0.0;
        for glyph in &row.glyphs {
            let Some(&(ch, is_link)) = expected.get(idx) else {
                // More glyphs than expected chars: give up on underlines.
                return Vec::new();
            };
            if glyph.chr != ch {
                return Vec::new();
            }
            idx += 1;
            if is_link {
                let baseline = placed.pos.y + glyph.pos.y;
                let x0 = placed.pos.x + glyph.pos.x;
                let x1 = x0 + glyph.advance_width;
                if run.is_none() {
                    run = Some((x0, baseline));
                }
                run_end_x = x1;
            } else if let Some((x0, baseline)) = run.take() {
                runs.push(make_link_run(
                    x0,
                    run_end_x,
                    baseline,
                    placed.pos.y,
                    row.size.y,
                ));
            }
        }
        if let Some((x0, baseline)) = run.take() {
            runs.push(make_link_run(
                x0,
                run_end_x,
                baseline,
                placed.pos.y,
                row.size.y,
            ));
        }
    }
    runs
}

/// Allocates the full available width at the galley's height and returns the
/// centred paint origin, the allocated rect, and the response. Paints
/// nothing; the caller paints the galley (and any underlines) at the origin.
///
/// The callers lay the job out with `halign = Align::Center`, so the
/// galley's rect is centred on x = 0 (epaint: with
/// `LayoutJob::halign == Align::Center`, `galley.rect.center() == 0.0`).
/// Centring the galley therefore means putting the galley rect's centre on
/// the allocated rect's centre. Offsetting by
/// `(rect.width() - galley.size().x) * 0.5` instead assumes a left-aligned
/// galley whose rect starts at 0 and double-counts the centring: the ink
/// shifts left by half the galley width, so the widest row's right edge
/// lands on the container centre (EMP-01).
fn allocate_centered(ui: &mut Ui, galley: &egui::Galley) -> (Pos2, Rect, egui::Response) {
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), galley.size().y),
        egui::Sense::hover(),
    );
    let origin = Pos2::new(rect.center().x - galley.rect.center().x, rect.min.y);
    (origin, rect, response)
}

/// A shadcn empty state container. See the [module docs](self) for an example.
#[derive(Clone, Debug, Default)]
pub struct Empty {
    bordered: bool,
    fill_height: bool,
    padding: Option<f32>,
    id_salt: Option<String>,
}

impl Empty {
    /// Creates an empty state container.
    pub fn new() -> Self {
        Self {
            bordered: false,
            fill_height: false,
            padding: None,
            id_salt: None,
        }
    }

    /// Shows the dashed border (`border border-dashed` in the docs markup;
    /// the base component only sets the dashed style).
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Stretches to the available height (`flex-1 justify-center`).
    pub fn fill_height(mut self, fill_height: bool) -> Self {
        self.fill_height = fill_height;
        self
    }

    /// Overrides the container padding (`p-6` = 24px by default). Pass 48px
    /// for the `md:p-12` rule at viewport widths ≥ 768px (spec §3.1, D4):
    /// egui has no `md` breakpoint, so the caller opts in manually.
    /// Padding is in whole points.
    pub fn padding(mut self, padding: f32) -> Self {
        self.padding = Some(padding);
        self
    }

    /// The effective container padding (the override, or the 24px default).
    pub(crate) fn padding_value(&self) -> f32 {
        self.padding.unwrap_or(EMPTY_DEFAULT_PADDING)
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the empty state around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let g = gap(theme.style);
        let radius = if theme.style == Style::Nova {
            theme.radius.xl_u8()
        } else {
            theme.radius.lg_u8()
        };

        let painter = ui.painter().clone();
        let border_idx = painter.add(Shape::Noop);

        let inner = egui::Frame::new()
            .inner_margin(Margin::same(self.padding_value() as i8))
            .show(ui, |ui| {
                if self.fill_height {
                    ui.set_min_height(ui.available_height());
                }
                ui.set_min_width(ui.available_width());
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = g;
                    content(ui)
                })
                .inner
            });

        if self.bordered {
            // Dashed border approximation: four dashed segments between the
            // rounded corners.
            let rect = inner.response.rect;
            let r = radius as f32 * 0.5;
            let stroke = egui::Stroke::new(1.0, p.border);
            let (d, gpat) = (6.0, 4.0);
            let mut shapes = Vec::new();
            Shape::dashed_line(
                &[
                    Pos2::new(rect.left() + r, rect.top()),
                    Pos2::new(rect.right() - r, rect.top()),
                ],
                stroke,
                d,
                gpat,
            )
            .into_iter()
            .for_each(|s| shapes.push(s));
            Shape::dashed_line(
                &[
                    Pos2::new(rect.right(), rect.top() + r),
                    Pos2::new(rect.right(), rect.bottom() - r),
                ],
                stroke,
                d,
                gpat,
            )
            .into_iter()
            .for_each(|s| shapes.push(s));
            Shape::dashed_line(
                &[
                    Pos2::new(rect.right() - r, rect.bottom()),
                    Pos2::new(rect.left() + r, rect.bottom()),
                ],
                stroke,
                d,
                gpat,
            )
            .into_iter()
            .for_each(|s| shapes.push(s));
            Shape::dashed_line(
                &[
                    Pos2::new(rect.left(), rect.bottom() - r),
                    Pos2::new(rect.left(), rect.top() + r),
                ],
                stroke,
                d,
                gpat,
            )
            .into_iter()
            .for_each(|s| shapes.push(s));
            painter.set(border_idx, Shape::Vec(shapes));
        } else {
            painter.set(border_idx, Shape::Noop);
        }

        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `EmptyHeader` — centered column (`gap-2 max-w-sm`).
#[derive(Clone, Debug, Default)]
pub struct EmptyHeader {
    id_salt: Option<String>,
}

impl EmptyHeader {
    /// Creates an empty header.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let inner = ui.allocate_ui(
            Vec2::new(EMPTY_MAX_WIDTH.min(ui.available_width()), 0.0),
            |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = HEADER_GAP;
                    content(ui)
                })
                .inner
            },
        );
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `EmptyMedia` — the media slot; [`EmptyMediaVariant::Icon`] draws a
/// `bg-muted` rounded box around the icon.
#[derive(Clone, Debug)]
pub struct EmptyMedia {
    variant: EmptyMediaVariant,
    icon: Option<Icon>,
    id_salt: Option<String>,
}

impl Default for EmptyMedia {
    fn default() -> Self {
        Self::new()
    }
}

impl EmptyMedia {
    /// Creates free-form media (paint your own content in [`Self::show`]).
    pub fn new() -> Self {
        Self {
            variant: EmptyMediaVariant::Default,
            icon: None,
            id_salt: None,
        }
    }

    /// Creates the icon media variant with the given lucide icon.
    pub fn icon(icon: Icon) -> Self {
        Self {
            variant: EmptyMediaVariant::Icon,
            icon: Some(icon),
            id_salt: None,
        }
    }

    /// Overrides the media variant.
    pub fn variant(mut self, variant: EmptyMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the media slot (the icon box for the icon variant).
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let response = if self.variant == EmptyMediaVariant::Icon {
            let box_size = media_icon_box(theme.style);
            let (rect, response) =
                ui.allocate_exact_size(Vec2::splat(box_size), egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                painter.rect_filled(rect, theme.radius.lg_u8(), p.muted);
                if let Some(icon) = self.icon {
                    let icon_size = media_icon_size(theme.style);
                    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
                    icon.paint(painter, icon_rect, p.foreground, 2.0 * icon_size / 24.0);
                }
            }
            response
        } else {
            // Free-form media has no intrinsic size; paint it yourself with
            // [`Self::show_with`].
            ui.allocate_exact_size(Vec2::ZERO, egui::Sense::hover()).1
        };
        // `mb-2` *below* the media (the source class is on the media div).
        ui.add_space(8.0);
        response
    }

    /// Renders free-form media content (images, custom glyphs) centered.
    pub fn show_with<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let inner = ui.allocate_ui(
            Vec2::new(EMPTY_MAX_WIDTH.min(ui.available_width()), 0.0),
            |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), content)
                    .inner
            },
        );
        // `mb-2` below the media.
        ui.add_space(8.0);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `EmptyTitle` — `text-lg font-medium tracking-tight` (nova `text-sm`);
/// line-heights 28px base / 20px nova (spec §2.4, §3.3).
#[derive(Clone, Debug)]
pub struct EmptyTitle {
    text: String,
}

impl EmptyTitle {
    /// Creates an empty title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let mut job = egui::text::LayoutJob::default();
        job.append(
            &self.text,
            0.0,
            egui::TextFormat {
                font_id: title_font(theme.style),
                color: p.foreground,
                line_height: Some(title_line_height(theme.style)),
                ..egui::TextFormat::default()
            },
        );
        job.halign = Align::Center;
        job.wrap.max_width = EMPTY_MAX_WIDTH.min(ui.available_width());
        let galley = ui.painter().layout_job(job);
        let (origin, rect, response) = allocate_centered(ui, &galley);
        if ui.is_rect_visible(rect) {
            ui.painter().galley(origin, galley, p.foreground);
        }
        response
    }
}

/// `EmptyDescription` — `text-sm/relaxed text-muted-foreground` (14px text,
/// 22.75px line height).
#[derive(Clone, Debug)]
pub struct EmptyDescription {
    segments: Vec<DescriptionSegment>,
}

impl EmptyDescription {
    /// Creates an empty description.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            segments: vec![DescriptionSegment::Text(text.into())],
        }
    }

    /// Appends a direct-child link: underlined, 4px underline offset, and
    /// `text-primary` on hover (spec §2.5). Use [`Self::text`] to continue
    /// with plain text after a link, so a link can sit mid-sentence.
    pub fn link(mut self, text: impl Into<String>) -> Self {
        self.segments.push(DescriptionSegment::Link(text.into()));
        self
    }

    /// Appends plain description text (after a link, or to extend the text).
    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.segments.push(DescriptionSegment::Text(text.into()));
        self
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let width = EMPTY_MAX_WIDTH.min(ui.available_width());
        // Pass 1: resting colours, to learn the link geometry. The layout
        // depends on colours only through the formats, so the recolour pass
        // below keeps the same geometry.
        let mut job = description_job(&self.segments, p.muted_foreground, p.muted_foreground);
        job.wrap.max_width = width;
        let galley = ui.painter().layout_job(job);
        let (origin, rect, response) = allocate_centered(ui, &galley);
        let runs = link_runs(&galley, &self.segments);
        let hovered = response.hover_pos().is_some_and(|pos| {
            runs.iter()
                .any(|run| run.hit.translate(origin.to_vec2()).contains(pos))
        });
        // Pass 2 (only when a link is hovered): recolour the links to
        // `text-primary`. Geometry is identical — only colours changed.
        let (galley, runs) = if hovered {
            let mut job = description_job(&self.segments, p.muted_foreground, p.primary);
            job.wrap.max_width = width;
            let galley = ui.painter().layout_job(job);
            let runs = link_runs(&galley, &self.segments);
            (galley, runs)
        } else {
            (galley, runs)
        };
        if ui.is_rect_visible(rect) {
            ui.painter().galley(origin, galley, p.muted_foreground);
            let link_color = resolve_link_color(hovered, &p);
            for run in &runs {
                ui.painter().line_segment(
                    [origin + run.from.to_vec2(), origin + run.to.to_vec2()],
                    Stroke::new(1.0, link_color),
                );
            }
        }
        response
    }
}

/// `EmptyContent` — the actions slot (`gap-4 max-w-sm`; nova `gap-2.5`).
#[derive(Clone, Debug, Default)]
pub struct EmptyContent {
    id_salt: Option<String>,
}

impl EmptyContent {
    /// Creates an empty content slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the content slot.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let g = content_gap(theme.style);
        let inner = ui.allocate_ui(
            Vec2::new(EMPTY_MAX_WIDTH.min(ui.available_width()), 0.0),
            |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = g;
                    content(ui)
                })
                .inner
            },
        );
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `EmptyFooter` — bottom slot mirroring EmptyContent. This slot is a Rust
/// port extension: `empty.tsx` exports no footer (its docs pages wrap
/// actions in the content slot), so prefer [`EmptyContent`] for new code.
#[derive(Clone, Debug, Default)]
pub struct EmptyFooter {
    id_salt: Option<String>,
}

impl EmptyFooter {
    /// Creates an empty footer.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the footer.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let inner = ui.allocate_ui(
            Vec2::new(EMPTY_MAX_WIDTH.min(ui.available_width()), 0.0),
            |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    content(ui)
                })
                .inner
            },
        );
        InnerResponse::new(inner.inner, inner.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    #[test]
    fn geometry_matches_style_recipes() {
        // base gap-6 / p-6 md:p-12; nova gap-4 / p-6.
        assert_eq!(gap(Style::Vega), 24.0);
        assert_eq!(gap(Style::Nova), 16.0);
        // media icon size-10 vs nova size-8, glyph size-6 vs size-4.
        assert_eq!(media_icon_box(Style::Vega), 40.0);
        assert_eq!(media_icon_box(Style::Nova), 32.0);
        assert_eq!(media_icon_size(Style::Vega), 24.0);
        assert_eq!(media_icon_size(Style::Nova), 16.0);
        // title text-lg vs nova text-sm.
        assert_eq!(title_size(Style::Vega), 18.0);
        assert_eq!(title_size(Style::Nova), 14.0);
        // title line-heights: text-lg 28px, nova text-sm 20px (spec §3.3).
        assert_eq!(title_line_height(Style::Vega), 28.0);
        assert_eq!(title_line_height(Style::Nova), 20.0);
        // content gap-4 vs nova gap-2.5; header gap-2.
        assert_eq!(content_gap(Style::Vega), 16.0);
        assert_eq!(content_gap(Style::Nova), 10.0);
        assert_eq!(HEADER_GAP, 8.0);
        // container radius: rounded-lg 10px base, rounded-xl 14px nova.
        let theme = Preset::default().build(Mode::Light);
        assert_eq!(theme.radius.lg_u8(), 10);
        assert_eq!(theme.radius.xl_u8(), 14);
        // max-w-sm
        assert_eq!(EMPTY_MAX_WIDTH, 384.0);
        // default padding is p-6 = 24px (spec §3.1).
        assert_eq!(EMPTY_DEFAULT_PADDING, 24.0);
    }

    #[test]
    fn title_font_is_medium() {
        // Spec §2.4/§4: `font-medium` (500) in both styles — the embedded
        // Inter Medium face, not the regular face.
        for style in [Style::Vega, Style::Nova] {
            let font = title_font(style);
            assert_eq!(font.family, fonts::medium_family());
        }
        assert_eq!(title_font(Style::Vega).size, 18.0);
        assert_eq!(title_font(Style::Nova).size, 14.0);
    }

    #[test]
    fn description_line_height_is_22_75() {
        // Spec §0 correction 1: `text-sm/relaxed` = 14px × 1.625 = 22.75px,
        // not the audit's 20.0 (`line_heights::SMALL`).
        assert_eq!(DESCRIPTION_LINE_HEIGHT, 22.75);
        assert_eq!(DESCRIPTION_LINE_HEIGHT, 14.0 * 1.625);
        let p = Preset::default().build(Mode::Light).palette;
        let job = description_job(
            &[DescriptionSegment::Text("x".to_string())],
            p.muted_foreground,
            p.muted_foreground,
        );
        assert_eq!(job.sections.len(), 1);
        assert_eq!(job.sections[0].format.line_height, Some(22.75));
        assert_eq!(job.sections[0].format.font_id.size, 14.0);
        assert_eq!(job.sections[0].format.color, p.muted_foreground);
    }

    #[test]
    fn padding_default_and_override() {
        assert_eq!(Empty::new().padding_value(), 24.0);
        assert_eq!(Empty::new().padding(48.0).padding_value(), 48.0);
    }

    #[test]
    fn link_color_and_offset() {
        // Spec §2.5: links are `text-muted-foreground` at rest and
        // `text-primary` on hover, with a 4px underline offset.
        let p = Preset::default().build(Mode::Light).palette;
        assert_eq!(resolve_link_color(false, &p), p.muted_foreground);
        assert_eq!(resolve_link_color(true, &p), p.primary);
        assert_eq!(LINK_UNDERLINE_OFFSET, 4.0);
    }

    /// Lays out a description job in a headless context and returns its galley.
    fn layout_description(
        ctx: &egui::Context,
        segments: &[DescriptionSegment],
        wrap_width: f32,
    ) -> std::sync::Arc<egui::Galley> {
        let mut galley = None;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("empty_link_runs_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            );
            let p = Preset::default().build(Mode::Light).palette;
            let mut job = description_job(segments, p.muted_foreground, p.muted_foreground);
            job.wrap.max_width = wrap_width;
            galley = Some(ui.painter().layout_job(job));
        });
        out.textures_delta.clear();
        galley.expect("galley laid out")
    }

    #[test]
    fn link_runs_found_in_single_line_galley() {
        let ctx = egui::Context::default();
        let segments = vec![
            DescriptionSegment::Text("See the ".to_string()),
            DescriptionSegment::Link("docs".to_string()),
            DescriptionSegment::Text(" for details.".to_string()),
        ];
        let galley = layout_description(&ctx, &segments, 384.0);
        let runs = link_runs(&galley, &segments);
        assert_eq!(runs.len(), 1, "one underline run for one link on one row");
        let run = runs[0];
        // The run spans exactly the link's laid-out width.
        let mut link_w = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            let g = ui.painter().layout_no_wrap(
                "docs".to_string(),
                FontId::proportional(crate::theme::tokens::type_scale::SM),
                Color32::WHITE,
            );
            link_w = g.size().x;
        });
        out.textures_delta.clear();
        assert!((run.to.x - run.from.x - link_w).abs() < 1.0);
        // The underline sits 4px below the row baseline.
        let placed = &galley.rows[0];
        let baseline = placed.pos.y + placed.row.glyphs[0].pos.y;
        assert_eq!(run.from.y, baseline + LINK_UNDERLINE_OFFSET);
        assert_eq!(run.to.y, baseline + LINK_UNDERLINE_OFFSET);
        // The hit rect covers the link text.
        let mid = Pos2::new((run.from.x + run.to.x) * 0.5, baseline);
        assert!(run.hit.contains(mid));
    }

    #[test]
    fn link_wraps_to_two_runs() {
        let ctx = egui::Context::default();
        let segments = vec![
            DescriptionSegment::Text("See ".to_string()),
            DescriptionSegment::Link("documentation".to_string()),
            DescriptionSegment::Text(" here.".to_string()),
        ];
        let galley = layout_description(&ctx, &segments, 60.0);
        let runs = link_runs(&galley, &segments);
        assert_eq!(
            runs.len(),
            2,
            "a link wrapping across rows underlines once per visual row"
        );
        assert_ne!(runs[0].from.y, runs[1].from.y);
    }

    #[test]
    fn mb_2_space_sits_after_media() {
        // Spec §2.3: `mb-2` is an 8px gap BELOW the media, never before.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("empty_mb2_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            );
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            let media_rect = EmptyMedia::icon(Icon::Search).show(ui).rect;
            let label_rect = ui.label("after").rect;
            assert_eq!(label_rect.top() - media_rect.bottom(), 8.0);
            // Same for the free-form path: the space follows the content.
            let mut content_bottom = 0.0;
            EmptyMedia::new().show_with(ui, |ui| {
                content_bottom = ui.label("custom").rect.bottom();
            });
            let after_rect = ui.label("after free-form").rect;
            assert_eq!(after_rect.top() - content_bottom, 8.0);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn padding_changes_container_height() {
        // `Empty::padding(48)` adds 24px of padding per side over the 24px
        // default, so the container grows by exactly 48px (spec §3.1).
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut heights = [0.0; 2];
        for (i, padding) in [None, Some(48.0)].into_iter().enumerate() {
            let mut height = 0.0;
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("empty_padding_height_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
                );
                let mut empty = Empty::new();
                if let Some(padding) = padding {
                    empty = empty.padding(padding);
                }
                height = empty
                    .show(ui, |ui| {
                        EmptyTitle::new("T").show(ui);
                    })
                    .response
                    .rect
                    .height();
            });
            out.textures_delta.clear();
            heights[i] = height;
        }
        assert_eq!(heights[1] - heights[0], 48.0);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("empty_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
                );
                Empty::new().bordered(true).show(ui, |ui| {
                    EmptyHeader::new().show(ui, |ui| {
                        EmptyMedia::icon(Icon::Search).show(ui);
                        EmptyTitle::new("No results found").show(ui);
                        EmptyDescription::new("Try a different search term or clear the filters.")
                            .show(ui);
                    });
                    EmptyContent::new().show(ui, |ui| {
                        ui.label("actions");
                    });
                });
                Empty::new().show(ui, |ui| {
                    EmptyMedia::new().show(ui);
                    EmptyTitle::new("Bare empty").show(ui);
                });
                // A description with a mid-sentence link renders in both modes.
                Empty::new().padding(48.0).show(ui, |ui| {
                    EmptyHeader::new().show(ui, |ui| {
                        EmptyDescription::new("See the ")
                            .link("docs")
                            .text(" for details.")
                            .show(ui);
                    });
                });
            });
            out.textures_delta.clear();
        }
    }

    /// Ink centre of one galley row, painted at `origin`, in absolute coords.
    /// Uses the glyphs' logical rects (pos + advance width), which bound the
    /// ink to within a pixel of bearing on each side.
    fn row_ink_centre_x(origin: Pos2, galley: &egui::Galley, row_idx: usize) -> f32 {
        let placed = &galley.rows[row_idx];
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        for glyph in &placed.row.glyphs {
            let rect = glyph.logical_rect();
            min_x = min_x.min(origin.x + placed.pos.x + rect.min.x);
            max_x = max_x.max(origin.x + placed.pos.x + rect.max.x);
        }
        (min_x + max_x) * 0.5
    }

    #[test]
    fn allocate_centered_centres_title_ink_on_rect_centre() {
        // EMP-01: the title's ink centre must sit on the allocated rect's
        // centre. The pre-fix origin put the widest row's right edge on the
        // centre instead (ink centre off by half the galley width, 25–190px
        // in the gallery), so a 2px tolerance discriminates decisively.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("empty_allocate_centered_title_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            );
            // Title job, built exactly as `EmptyTitle::show` builds it.
            let theme = ShadcnTheme::of_ui(ui);
            let mut job = egui::text::LayoutJob::default();
            job.append(
                "No results found",
                0.0,
                egui::TextFormat {
                    font_id: title_font(theme.style),
                    color: Color32::BLACK,
                    line_height: Some(title_line_height(theme.style)),
                    ..egui::TextFormat::default()
                },
            );
            job.halign = Align::Center;
            job.wrap.max_width = EMPTY_MAX_WIDTH.min(ui.available_width());
            let galley = ui.painter().layout_job(job);
            let (origin, rect, _response) = allocate_centered(ui, &galley);
            assert_eq!(rect.width(), 400.0, "allocates the full available width");
            assert_eq!(galley.rows.len(), 1);
            let ink_centre = row_ink_centre_x(origin, &galley, 0);
            assert!(
                (ink_centre - rect.center().x).abs() < 2.0,
                "title ink centre {ink_centre} must sit on rect centre {} (origin.x = {}, galley width = {})",
                rect.center().x,
                origin.x,
                galley.size().x,
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn allocate_centered_centres_each_description_row() {
        // EMP-01, multi-row case: with `halign = Center` every row is
        // centred on the galley centre independently, so each row's ink
        // centre must sit on the rect centre — including the short second
        // row, which the defect showed centred on the first row's (shifted)
        // centre instead of the container's.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let segments = vec![
            DescriptionSegment::Text("See the ".to_string()),
            DescriptionSegment::Link("docs".to_string()),
            DescriptionSegment::Text(
                " for details about this empty state and how to fix it.".to_string(),
            ),
        ];
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("empty_allocate_centered_desc_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
            );
            // Description job, built exactly as `EmptyDescription::show`
            // builds its pass-1 job (wraps at 384, so it spans two rows).
            let p = Preset::default().build(Mode::Light).palette;
            let mut job = description_job(&segments, p.muted_foreground, p.muted_foreground);
            job.wrap.max_width = EMPTY_MAX_WIDTH.min(ui.available_width());
            let galley = ui.painter().layout_job(job);
            let (origin, rect, _response) = allocate_centered(ui, &galley);
            assert!(
                galley.rows.len() >= 2,
                "the description must wrap to exercise the multi-row case"
            );
            for i in 0..galley.rows.len() {
                let ink_centre = row_ink_centre_x(origin, &galley, i);
                assert!(
                    (ink_centre - rect.center().x).abs() < 2.0,
                    "row {i} ink centre {ink_centre} must sit on rect centre {}",
                    rect.center().x,
                );
            }
        });
        out.textures_delta.clear();
    }
}
