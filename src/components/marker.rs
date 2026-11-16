//! shadcn `marker.tsx` — an annotated row: icon + short text, optionally
//! divided by horizontal rules (`separator`) or underlined (`border`).
//!
//! Rebuilt to the frozen S0 spec (Batch A, coordinator-signed 2026-11-15).
//! Source: `apps/v4/registry/new-york-v4/ui/marker.tsx` (68 lines), nova
//! recipe `style-nova.css`, tokens `globals.css`.
//!
//! Shared base (`marker.tsx:7`):
//! `group/marker relative flex min-h-4 w-full items-center gap-2 text-left
//! text-sm text-muted-foreground [&_svg:not([class*='size-'])]:size-4
//! [a]:underline [a]:underline-offset-3 [a]:hover:text-foreground`
//!
//! Variants (`markerVariants`, `marker.tsx:10-15`):
//! - `default`: `""` (empty, `:11`).
//! - `separator` (`:12-13`): `before:mr-1 before:h-px before:min-w-0
//!   before:flex-1 before:bg-border after:ml-1 after:h-px after:min-w-0
//!   after:flex-1 after:bg-border`.
//! - `border` (`:14`): `border-b border-border pb-2`.
//!
//! Nova (`style-nova.css:1577-1595`) restates every slot: `.cn-marker`
//! (`:1577-1579`), `.cn-marker-variant-separator` (`:1581-1583`),
//! `.cn-marker-variant-border` (`:1585-1587`), `.cn-marker-icon`
//! (`:1589-1591`, omits `shrink-0`), `.cn-marker-content` (`:1593-1595`,
//! omits `min-w-0`/`wrap-break-word`). No `.cn-marker-variant-default` rule
//! exists. Under the LAYER ruling (nova layers over base; silence keeps the
//! base value) the effective nova values are IDENTICAL to base in every slot
//! (spec §2.3), so one geometry and one colour rule serve both recipes.
//!
//! Settled audit items (spec §§4-5, 11):
//! - N5 row-height rule: row height = `max(16px, tallest flex item)`.
//!   `min-h-4` is a FLOOR, not a fixed height — all three variants grow.
//!   The floor binds on the TOTAL row height (`border-box`): the `border`
//!   variant totals `max(16px, tallest + 9px)`, so an empty border row is
//!   16px (MRK-VIS-01 ruling, S5 §3).
//! - N12 gap rule: `gap-2` = 8px, set explicitly between every pair of
//!   adjacent flex items, all variants, both recipes.
//! - M-P2-2: generic composition in the content slot
//!   ([`MarkerContent::show_custom`]) and link styling (§7.2) via
//!   [`MarkerLink`].
//!
//! Deviations / limitations / notes (spec §13):
//! - D1 `asChild`/Slot: LIMITATION — no egui analogue; not implemented.
//! - D2 link activation: PROPOSED DEVIATION (Expert Reviewer approval: ____)
//!   — links render with §7.2 visual styling; clicks surface via the
//!   returned [`egui::Response`]; navigation is caller-owned.
//! - D3 `aria-hidden`: LIMITATION — no ARIA tree in egui.
//! - D4 underline rendering: NOTE — closest egui equivalent (1px line, text
//!   colour, 3px below the baseline); S5 judges.
//! - D5 `relative`: NOTE — no visual effect, no egui equivalent needed.
//! - D6 `group/marker`: IMPLEMENTATION NOTE — the variant is known directly
//!   in Rust; separator-only content rules apply when the variant is
//!   [`MarkerVariant::Separator`].
//!
//! OPEN items (spec §14, all left open by this rebuild):
//! - O1: exact `text-sm` line-height (20px Tailwind default used; the RULE is
//!   frozen, the numeric input is unconfirmed).
//! - O2: browser-default underline rendering details.
//! - O3: Radix `Slot` merge semantics (see D1).
//! - O4: extreme narrow-container separator behaviour.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Marker::new().show(ui, |ui| {
//!     MarkerIcon::new(Icon::Sparkles).show(ui);
//!     MarkerContent::new("Heads up — usage is trending up").show(ui);
//! });
//!
//! Marker::new().variant(MarkerVariant::Separator).show(ui, |ui| {
//!     MarkerContent::new("or continue with").show(ui);
//! });
//! ```

use crate::icons::Icon;
use crate::theme::fonts::regular;
use crate::theme::tokens::{spacing, type_scale};
use crate::theme::ShadcnTheme;
use egui::text::LayoutJob;
use egui::{
    Align, Color32, CursorIcon, InnerResponse, Pos2, Rect, Sense, Stroke, Ui, UiBuilder, Vec2,
};

/// `text-sm` line box height: the Tailwind default `20px` for `14px` text
/// (spec §4.1, OPEN O1 — the rule is frozen, the numeric input is the
/// framework default, unconfirmed against a rendered original).
pub(crate) const TEXT_LINE_HEIGHT: f32 = 20.0;

/// `underline-offset-3`: the underline sits 3px below the text baseline
/// (spec §7.2; rendering details OPEN O2, closest equivalent per D4).
pub(crate) const UNDERLINE_OFFSET: f32 = 3.0;

/// `h-px` / `border-b`: rule thickness, 1px (spec §§5-6).
pub(crate) const RULE_THICKNESS: f32 = 1.0;

/// Marker layout variants, mirroring the source `markerVariants`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MarkerVariant {
    /// Plain row: optional icon + content.
    #[default]
    Default,
    /// The content is centered between two 1px rules (`before:/after:`).
    Separator,
    /// The row gets a bottom rule with `pb-2` padding.
    Border,
}

/// The marker text color (`text-muted-foreground`).
pub(crate) fn text_color(theme: &ShadcnTheme) -> Color32 {
    theme.palette.muted_foreground
}

/// Total row height for a variant given the tallest flex item's height
/// (raw, unfloored): `min-h-4` is a floor on the TOTAL row height, not on
/// the content box. Under the `border-box` model the snapshot's Tailwind
/// import imposes (Preflight), the `border` variant's `pb-2` (8px) +
/// `border-b` (1px) fit INSIDE the 16px minimum when the content is short,
/// so the empty border row totals 16px (MRK-VIS-01 ruling, S5 §3). For
/// content heights ≥ 7px this is exactly `content + 9` (spec §6).
pub(crate) fn total_row_height(variant: MarkerVariant, tallest_item: f32) -> f32 {
    match variant {
        MarkerVariant::Border => (tallest_item + spacing::SM + RULE_THICKNESS).max(spacing::LG),
        _ => tallest_item.max(spacing::LG),
    }
}

/// Separator rule x-spans for a row of `row_width` with the content span
/// `[content_min, content_max]` (relative to the row's left edge). Returns
/// `(left_rule_end, right_rule_start)`; the rules run `[0, left_rule_end]`
/// and `[right_rule_start, row_width]`, each 12px from the content (8px flex
/// gap + 4px `mr-1`/`ml-1` margin, spec §5). Spans clamp to the row, so
/// full-width content yields no rules.
pub(crate) fn separator_rule_spans(
    row_width: f32,
    content_min: f32,
    content_max: f32,
) -> (f32, f32) {
    let gap = spacing::SM + spacing::XS;
    (
        (content_min - gap).clamp(0.0, row_width),
        (content_max + gap).clamp(0.0, row_width),
    )
}

/// Separator rule x-spans with NO children: the gap between the two rules is
/// 16px (8px flex gap + 4px + 4px margins), centred on the row (spec §5).
pub(crate) fn empty_separator_rule_spans(row_width: f32) -> (f32, f32) {
    let center = row_width / 2.0;
    let half_gap = (spacing::SM + 2.0 * spacing::XS) / 2.0;
    (
        (center - half_gap).max(0.0),
        (center + half_gap).min(row_width),
    )
}

/// Left inset that centres content of `content_width` in a row of
/// `row_width` (spec §5). Zero when the content fills (or overflows) the row.
pub(crate) fn separator_content_inset(row_width: f32, content_width: f32) -> f32 {
    ((row_width - content_width) / 2.0).max(0.0)
}

/// Centre y of the `border-b` rule: 8px (`pb-2`) below the content-box
/// bottom, plus half the 1px border (spec §6).
pub(crate) fn border_rule_center_y(content_bottom: f32) -> f32 {
    content_bottom + spacing::SM + RULE_THICKNESS / 2.0
}

/// Underline y for a glyph row: the baseline plus `underline-offset-3`
/// (spec §7.2; D4 closest equivalent).
pub(crate) fn underline_y(baseline_y: f32) -> f32 {
    baseline_y + UNDERLINE_OFFSET
}

/// Id for the variant context stack in `ctx.data` temp storage. The stack
/// lets content widgets (see [`MarkerContent`]) apply the separator-only
/// rules of the row they sit in (D6: the variant is known directly in Rust,
/// no `group/marker` mechanism needed).
fn variant_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_marker_variant_stack")
}

fn push_variant(ctx: &egui::Context, variant: MarkerVariant) {
    ctx.data_mut(|data| {
        let mut stack: Vec<MarkerVariant> = data.get_temp(variant_stack_id()).unwrap_or_default();
        stack.push(variant);
        data.insert_temp(variant_stack_id(), stack);
    });
}

fn pop_variant(ctx: &egui::Context) {
    ctx.data_mut(|data| {
        let mut stack: Vec<MarkerVariant> = data.get_temp(variant_stack_id()).unwrap_or_default();
        stack.pop();
        data.insert_temp(variant_stack_id(), stack);
    });
}

/// The variant of the innermost enclosing marker row, or `Default` when the
/// widget sits outside any marker row.
fn current_variant(ctx: &egui::Context) -> MarkerVariant {
    ctx.data_mut(|data| {
        data.get_temp::<Vec<MarkerVariant>>(variant_stack_id())
            .and_then(|stack| stack.last().copied())
            .unwrap_or_default()
    })
}

/// Restores the enclosing variant when the row's closure returns (or panics),
/// so nested markers render independently (spec §12).
struct VariantGuard {
    ctx: egui::Context,
}

impl Drop for VariantGuard {
    fn drop(&mut self) {
        pop_variant(&self.ctx);
    }
}

/// Lays out marker text: 14px Inter 400 in `color`, in a 20px line box (O1),
/// wrapping at the available width (`min-w-0` + `wrap-break-word`). Under a
/// separator row the lines centre (`text-center`, spec §3.3). egui centres
/// text within an explicit line height like CSS does, and breaks overlong
/// words at the wrap width, matching `wrap-break-word`.
fn layout_marker_text(
    ui: &Ui,
    text: &str,
    color: Color32,
    centered: bool,
) -> std::sync::Arc<egui::text::Galley> {
    let mut job = LayoutJob::simple(
        text.to_owned(),
        regular(type_scale::SM),
        color,
        ui.available_width(),
    );
    job.sections[0].format.line_height = Some(TEXT_LINE_HEIGHT);
    if centered {
        job.halign = Align::Center;
    }
    ui.painter().layout_job(job)
}

/// Paint position for a laid-out galley inside its allocated rect. Centred
/// galleys have a negative `rect.min.x` (rows are centred on the galley
/// origin), so the paint position must subtract the galley rect's min —
/// painting at the rect min would shift centred text left by half its width.
fn galley_paint_pos(rect_min: Pos2, galley: &egui::text::Galley) -> Pos2 {
    Pos2::new(rect_min.x - galley.rect.left(), rect_min.y)
}

/// A shadcn marker row. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Marker {
    variant: MarkerVariant,
    id_salt: Option<String>,
}

impl Default for Marker {
    fn default() -> Self {
        Self::new()
    }
}

impl Marker {
    /// Creates a default marker row.
    pub fn new() -> Self {
        Self {
            variant: MarkerVariant::Default,
            id_salt: None,
        }
    }

    /// Sets the layout variant.
    pub fn variant(mut self, variant: MarkerVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt. The row stores last frame's measured content
    /// size under a stable id; without a salt the id is call-order-stable
    /// (fine for static layouts, but prefer a salt in dynamic lists).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The minimum row height: `min-h-4` (16px) for all variants. Under
    /// `border-box` the floor binds on the total row height, so the
    /// `border` variant's `pb-2` + `border-b` (9px) fit inside the 16px
    /// minimum for short content (MRK-VIS-01 ruling). The actual row
    /// height is `max(minimum, tallest flex item)` for `default`/
    /// `separator` and `max(minimum, tallest flex item + 9)` for
    /// `border` (N5); this returns only the floor.
    pub fn row_height(&self) -> f32 {
        spacing::LG
    }

    /// Renders the marker row around its children (use [`MarkerIcon`] and
    /// [`MarkerContent`] inside, or any widget — the closure is the generic
    /// composition slot, spec §11.2).
    ///
    /// Layout is one-frame-deferred: the row height (N5) and the separator's
    /// centring inset (§5) both depend on the content size, which is only
    /// known after the content closure runs. The measured size is stored
    /// under the row id and reused next frame, so settled frames are exact;
    /// the first frame (and the frame after a content-size change) uses the
    /// floor/zero-width measurement. Captures and tests must use settled
    /// frames.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let width = ui.available_width();

        // Stable id for this row's stored measurement.
        let row_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt).with("marker_measured_content"),
            None => ui.auto_id_with("marker_measured_content"),
        };
        // No measurement yet: zero size, so the first frame uses the floor
        // via `total_row_height` (16px for every variant, including
        // `border` — the floor binds on the total, MRK-VIS-01).
        let stored: Vec2 = ui
            .ctx()
            .data_mut(|data| data.get_temp(row_id))
            .unwrap_or(Vec2::ZERO);
        let row_h = total_row_height(self.variant, stored.y);

        let (row_rect, response) = ui.allocate_exact_size(Vec2::new(width, row_h), Sense::hover());

        // The content box excludes `pb-2` + `border-b` so children centre in
        // the same space the CSS flex row gives them.
        let content_rect = match self.variant {
            MarkerVariant::Border => Rect::from_min_max(
                row_rect.min,
                Pos2::new(
                    row_rect.max.x,
                    row_rect.max.y - spacing::SM - RULE_THICKNESS,
                ),
            ),
            _ => row_rect,
        };
        // Separator: offset the content box by the centring inset computed
        // from last frame's measured content width, so the content group sits
        // centred and the rules share the remaining space equally (§5).
        // Offsetting the box (rather than inserting space) keeps `min_rect`
        // content-only.
        let content_rect = match self.variant {
            MarkerVariant::Separator => {
                let inset = separator_content_inset(row_rect.width(), stored.x);
                Rect::from_min_max(
                    Pos2::new(row_rect.min.x + inset, content_rect.min.y),
                    content_rect.max,
                )
            }
            _ => content_rect,
        };

        // N12: `gap-2` set explicitly on the row's style — never rely on the
        // ambient `item_spacing`.
        let mut child_style = egui::Style::clone(ui.style());
        child_style.spacing.item_spacing.x = spacing::SM;
        let mut builder = UiBuilder::new()
            .max_rect(content_rect)
            .layout(egui::Layout::left_to_right(Align::Center))
            .style(child_style);
        if let Some(salt) = self.id_salt {
            builder = builder.id_salt(salt);
        }
        let mut child = ui.new_child(builder);

        push_variant(ui.ctx(), self.variant);
        let guard = VariantGuard {
            ctx: ui.ctx().clone(),
        };
        let before = child.min_rect();
        let inner = content(&mut child);
        let measured_rect = child.min_rect();
        drop(guard);

        // Persist this frame's measured content size for the next frame.
        ui.ctx().data_mut(|data| {
            data.insert_temp(row_id, measured_rect.size());
        });

        // No flex items at all: the child's `min_rect` is unchanged.
        let empty = measured_rect == before;
        let painter = ui.painter();
        let rule_color = theme.palette.border;
        match self.variant {
            MarkerVariant::Separator => {
                // `::before`/`::after` are flex items: 1px, `border` token,
                // vertically centred on the row's cross axis, 12px from the
                // content (8px flex gap + 4px margin); with no children the
                // gap between the two rules is 16px, centred (§5).
                let cy = row_rect.center().y;
                let (left_end, right_start) = if empty {
                    let (l, r) = empty_separator_rule_spans(row_rect.width());
                    (row_rect.left() + l, row_rect.left() + r)
                } else {
                    let (l, r) = separator_rule_spans(
                        row_rect.width(),
                        measured_rect.left() - row_rect.left(),
                        measured_rect.right() - row_rect.left(),
                    );
                    (row_rect.left() + l, row_rect.left() + r)
                };
                if left_end > row_rect.left() {
                    painter.line_segment(
                        [Pos2::new(row_rect.left(), cy), Pos2::new(left_end, cy)],
                        Stroke::new(RULE_THICKNESS, rule_color),
                    );
                }
                if right_start < row_rect.right() {
                    painter.line_segment(
                        [Pos2::new(right_start, cy), Pos2::new(row_rect.right(), cy)],
                        Stroke::new(RULE_THICKNESS, rule_color),
                    );
                }
            }
            MarkerVariant::Border => {
                // `border-b pb-2`: the rule spans the full row width, its
                // centre 8.5px below the content-box bottom (§6).
                let y = border_rule_center_y(content_rect.bottom());
                painter.line_segment(
                    [
                        Pos2::new(row_rect.left(), y),
                        Pos2::new(row_rect.right(), y),
                    ],
                    Stroke::new(RULE_THICKNESS, rule_color),
                );
            }
            MarkerVariant::Default => {}
        }

        InnerResponse { inner, response }
    }
}

/// `size-4` leading icon slot (`marker.tsx:43-51`).
///
/// `size-4 shrink-0 [&_svg:not([class*='size-'])]:size-4`, `aria-hidden`
/// (D3: no ARIA tree in egui). The box is exactly 16×16 and never shrinks
/// (exact-size allocation); the glyph inherits `muted-foreground` (no colour
/// class in the source). An icon with an explicit size keeps its own size
/// (the `[class*='size-']` exemption) via [`MarkerIcon::size`].
#[derive(Clone, Debug)]
pub struct MarkerIcon {
    icon: Icon,
    size: f32,
}

impl MarkerIcon {
    /// Creates a 16px icon slot (`size-4`).
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: spacing::LG,
        }
    }

    /// Overrides the icon size (the explicit-`size-*` SVG case).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Renders the icon in the marker's muted text color.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(self.size), Sense::hover());
        if ui.is_rect_visible(rect) {
            self.icon.paint(
                ui.painter(),
                rect,
                text_color(&theme),
                2.0 * self.size / 24.0,
            );
        }
        response
    }
}

impl egui::Widget for MarkerIcon {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui)
    }
}

/// `text-sm` marker text (`marker.tsx:57-62`).
///
/// `min-w-0 wrap-break-word` in all variants, plus `flex-none` + `text-center`
/// under `separator` (`group-data-[variant=separator]/marker:*`, applied via
/// the row variant — D6). Inherits 14px + `muted-foreground`; no weight class
/// is specified in either recipe, so the text renders at the inherited 400.
/// Text lays out in a 20px line box (O1: Tailwind `text-sm` default).
#[derive(Clone, Debug)]
pub struct MarkerContent {
    text: String,
}

impl MarkerContent {
    /// Creates the marker text.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let centered = current_variant(ui.ctx()) == MarkerVariant::Separator;
        let galley = layout_marker_text(ui, &self.text, text_color(&theme), centered);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(
                galley_paint_pos(rect.min, &galley),
                galley,
                text_color(&theme),
            );
        }
        response
    }

    /// Runs arbitrary content in the marker content slot (spec §11.2, M-P2-2
    /// composition half): the original `MarkerContent` accepts any span
    /// children, so the Rust slot accepts any widget via this closure. The
    /// slot provides an id scope inside the row; text-level slot rules
    /// (wrap, separator centre) apply to text rendered through
    /// [`MarkerContent::show`] and [`MarkerLink`], while arbitrary content
    /// manages its own layout.
    pub fn show_custom<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        ui.scope(content)
    }
}

impl egui::Widget for MarkerContent {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui)
    }
}

/// An anchor inside a marker (spec §7.2, M-P2-2 link half; D2 PROPOSED).
///
/// Renders 14px/20px text in `muted-foreground`, underlined at a 3px offset
/// (D4 closest equivalent), turning `foreground` instantly on hover — the
/// `[a]:underline [a]:underline-offset-3 [a]:hover:text-foreground` rules,
/// all variants, both recipes. The original has no separate link export
/// (anchors are plain children); this widget is the Rust spelling of an
/// anchor child. Reachable as `crate::components::marker::MarkerLink`.
///
/// Clicks surface via the returned [`egui::Response`] (`clicked()`);
/// navigation is caller-owned (D2 — deviation PROPOSED, awaiting Expert
/// Reviewer approval). The link is Tab-focusable (anchors are natively
/// focusable on the web, spec §10) with no painted focus ring (no focus
/// styling is defined in the source). Nothing animates (§9): the hover
/// colour change is instant.
#[derive(Clone, Debug)]
pub struct MarkerLink {
    text: String,
}

impl MarkerLink {
    /// Creates a marker link with the given label.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the link. Every galley row is underlined.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let centered = current_variant(ui.ctx()) == MarkerVariant::Separator;
        let galley = layout_marker_text(ui, &self.text, text_color(&theme), centered);
        let (rect, response) = ui.allocate_exact_size(galley.size(), Sense::click());
        let response = response.on_hover_cursor(CursorIcon::PointingHand);
        if ui.is_rect_visible(rect) {
            let color = if response.hovered() {
                theme.palette.foreground
            } else {
                text_color(&theme)
            };
            let paint_pos = galley_paint_pos(rect.min, &galley);
            let painter = ui.painter();
            painter.galley_with_override_text_color(paint_pos, galley.clone(), color);
            // Underline every row: 1px, text colour (`currentColor` — no
            // decoration-colour class in the source), 3px below the baseline
            // (`underline-offset-3`; D4 closest equivalent, O2 open).
            for row in &galley.rows {
                let glyphs = &row.row.glyphs;
                if glyphs.is_empty() {
                    continue;
                }
                let first = &glyphs[0];
                let last = &glyphs[glyphs.len() - 1];
                let y = underline_y(paint_pos.y + row.pos.y + first.pos.y);
                let x0 = paint_pos.x + row.pos.x + first.pos.x;
                let x1 = paint_pos.x + row.pos.x + last.pos.x + last.advance_width;
                painter.line_segment(
                    [Pos2::new(x0, y), Pos2::new(x1, y)],
                    Stroke::new(RULE_THICKNESS, color),
                );
            }
        }
        response
    }
}

impl egui::Widget for MarkerLink {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Context, Id, Pos2, Rect, Vec2};

    /// Harness running the marker in a fresh `Context` with a stable Ui id,
    /// so stored measurements persist across frames exactly like production.
    struct Harness {
        ctx: Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            Self::with_mode(Mode::Light)
        }

        fn with_mode(mode: Mode) -> Self {
            let ctx = Context::default();
            Preset::default().build(mode).install(&ctx);
            Self { ctx, time: 0.0 }
        }

        fn step<T>(
            &mut self,
            width: f32,
            events: Vec<egui::Event>,
            f: impl FnOnce(&mut Ui) -> T,
        ) -> (T, egui::FullOutput) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 600.0))),
                ..Default::default()
            };
            let mut result = None;
            let mut f = Some(f);
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut Ui::new(
                    ctx.clone(),
                    Id::unique("marker_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 600.0))),
                );
                result = Some(f.take().expect("closure ran once")(ui));
            });
            out.textures_delta.clear();
            (result.expect("closure ran"), out)
        }

        fn run<T>(&mut self, f: impl FnOnce(&mut Ui) -> T) -> (T, egui::FullOutput) {
            self.step(400.0, Vec::new(), f)
        }
    }

    /// All painted line segments: `(endpoints, stroke)`.
    fn line_segments(out: &egui::FullOutput) -> Vec<([Pos2; 2], Stroke)> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::LineSegment { points, stroke } => Some((*points, *stroke)),
                _ => None,
            })
            .collect()
    }

    /// All painted text shapes: `(paint pos, galley, override color)`.
    fn texts(
        out: &egui::FullOutput,
    ) -> Vec<(Pos2, std::sync::Arc<egui::text::Galley>, Option<Color32>)> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Text(ts) => Some((ts.pos, ts.galley.clone(), ts.override_text_color)),
                _ => None,
            })
            .collect()
    }

    fn approx_eq(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn row_height_matches_padding_rules() {
        assert_eq!(Marker::new().row_height(), 16.0);
        assert_eq!(
            Marker::new().variant(MarkerVariant::Separator).row_height(),
            16.0
        );
        // `min-h-4` constrains the BORDER box (border-box, MRK-VIS-01
        // ruling): the 8px `pb-2` + 1px `border-b` fit INSIDE the 16px
        // minimum, so the border floor is 16px, not 25px.
        assert_eq!(
            Marker::new().variant(MarkerVariant::Border).row_height(),
            16.0
        );
    }

    #[test]
    fn variants_and_builder() {
        let m = Marker::new()
            .variant(MarkerVariant::Separator)
            .id_salt("m1");
        assert_eq!(m.variant, MarkerVariant::Separator);
        assert_eq!(m.id_salt.as_deref(), Some("m1"));
    }

    #[test]
    fn total_row_height_is_a_floor_on_the_total() {
        // `min-h-4` floors the TOTAL (border-box), never the content box.
        assert_eq!(total_row_height(MarkerVariant::Default, 0.0), 16.0);
        assert_eq!(total_row_height(MarkerVariant::Default, 16.0), 16.0);
        assert_eq!(total_row_height(MarkerVariant::Default, 20.0), 20.0);
        assert_eq!(total_row_height(MarkerVariant::Default, 40.0), 40.0);
    }

    #[test]
    fn total_row_height_per_variant() {
        assert_eq!(
            total_row_height(MarkerVariant::Default, 20.0),
            20.0,
            "default grows with content, no padding"
        );
        assert_eq!(
            total_row_height(MarkerVariant::Separator, 20.0),
            20.0,
            "separator grows with content, no padding"
        );
        assert_eq!(
            total_row_height(MarkerVariant::Border, 20.0),
            29.0,
            "border adds pb-2 + border-b"
        );
        assert_eq!(total_row_height(MarkerVariant::Border, 16.0), 25.0);
        // Border-box floor (MRK-VIS-01 ruling): `min-h-4` constrains the
        // total, so content shorter than 7px still totals 16px.
        assert_eq!(
            total_row_height(MarkerVariant::Border, 0.0),
            16.0,
            "empty border row: padding + rule fit inside the 16px floor"
        );
        assert_eq!(total_row_height(MarkerVariant::Border, 6.0), 16.0);
        assert_eq!(total_row_height(MarkerVariant::Border, 7.0), 16.0);
        assert_eq!(total_row_height(MarkerVariant::Border, 8.0), 17.0);
        assert_eq!(total_row_height(MarkerVariant::Default, 0.0), 16.0);
        assert_eq!(total_row_height(MarkerVariant::Separator, 0.0), 16.0);
    }

    #[test]
    fn border_row_heights_follow_border_box_floor() {
        // Discriminating harness test (MRK-VIS-01): empty = 16px total,
        // 20px text = 29px, 40px tall content = 49px — settled frames.
        let mut h = Harness::new();
        let mut empty_h = 0.0;
        for _ in 0..3 {
            let (resp, _) = h.run(|ui| {
                Marker::new()
                    .variant(MarkerVariant::Border)
                    .show(ui, |_| {})
            });
            empty_h = resp.response.rect.height();
        }
        assert_eq!(empty_h, 16.0, "empty border row totals 16px");

        let mut h = Harness::new();
        let mut text_h = 0.0;
        for _ in 0..3 {
            let (resp, _) = h.run(|ui| {
                Marker::new()
                    .variant(MarkerVariant::Border)
                    .show(ui, |ui| MarkerContent::new("Hello").show(ui))
            });
            text_h = resp.response.rect.height();
        }
        assert_eq!(text_h, 29.0, "20px content + 8px pb-2 + 1px border");

        let mut h = Harness::new();
        let mut tall_h = 0.0;
        for _ in 0..3 {
            let (resp, _) = h.run(|ui| {
                Marker::new().variant(MarkerVariant::Border).show(ui, |ui| {
                    ui.allocate_exact_size(Vec2::new(50.0, 40.0), Sense::hover());
                })
            });
            tall_h = resp.response.rect.height();
        }
        assert_eq!(tall_h, 49.0, "40px content + 8px pb-2 + 1px border");
    }

    #[test]
    fn separator_rule_spans_twelve_px_each_side() {
        // Content 60px wide, centred in a 200px row: 12px each side.
        assert_eq!(separator_rule_spans(200.0, 70.0, 130.0), (58.0, 142.0));
        // Content touching the left edge: the left rule vanishes.
        assert_eq!(separator_rule_spans(200.0, 0.0, 60.0), (0.0, 72.0));
        // Full-width content: no rules at all.
        assert_eq!(separator_rule_spans(200.0, 0.0, 200.0), (0.0, 200.0));
    }

    #[test]
    fn empty_separator_sixteen_px_gap() {
        let (l, r) = empty_separator_rule_spans(200.0);
        assert_eq!((l, r), (92.0, 108.0));
        assert_eq!(r - l, 16.0, "8px flex gap + 4px + 4px margins");
        // Degenerate narrow row clamps to the row edges.
        assert_eq!(empty_separator_rule_spans(10.0), (0.0, 10.0));
    }

    #[test]
    fn separator_content_inset_centres() {
        assert_eq!(separator_content_inset(200.0, 60.0), 70.0);
        assert_eq!(separator_content_inset(200.0, 200.0), 0.0);
        assert_eq!(separator_content_inset(100.0, 120.0), 0.0);
    }

    #[test]
    fn border_rule_center_y_is_content_bottom_plus_eight_point_five() {
        assert_eq!(border_rule_center_y(20.0), 28.5);
        assert_eq!(border_rule_center_y(0.0), 8.5);
    }

    #[test]
    fn underline_y_is_baseline_plus_three() {
        assert_eq!(underline_y(15.0), 18.0);
    }

    #[test]
    fn empty_row_height_all_variants() {
        for variant in [
            MarkerVariant::Default,
            MarkerVariant::Separator,
            MarkerVariant::Border,
        ] {
            let mut h = Harness::new();
            let expected = Marker::new().variant(variant).row_height();
            for _ in 0..3 {
                let (resp, _) =
                    h.run(|ui| Marker::new().variant(variant).show(ui, |_| {}).response);
                assert_eq!(resp.rect.height(), expected, "variant {variant:?}");
                assert_eq!(resp.rect.width(), 400.0, "w-full, variant {variant:?}");
            }
        }
    }

    #[test]
    fn icon_only_row_is_sixteen_px() {
        let mut h = Harness::new();
        let mut row_h = 0.0;
        let mut icon_size = Vec2::ZERO;
        for _ in 0..3 {
            let ((row, icon), _) = h.run(|ui| {
                let r = Marker::new().show(ui, |ui| MarkerIcon::new(Icon::Sparkles).show(ui).rect);
                (r.response.rect, r.inner)
            });
            row_h = row.height();
            icon_size = icon.size();
        }
        assert_eq!(row_h, 16.0);
        assert_eq!(icon_size, Vec2::splat(16.0), "size-4 box, never shrinks");
    }

    #[test]
    fn single_line_text_row_is_twenty_px_all_variants() {
        for variant in [
            MarkerVariant::Default,
            MarkerVariant::Separator,
            MarkerVariant::Border,
        ] {
            let mut h = Harness::new();
            let mut row_h = 0.0;
            let mut text_h = 0.0;
            for _ in 0..3 {
                let ((row, text), _) = h.run(|ui| {
                    let r = Marker::new()
                        .variant(variant)
                        .show(ui, |ui| MarkerContent::new("Hello").show(ui).rect);
                    (r.response.rect, r.inner)
                });
                row_h = row.height();
                text_h = text.height();
            }
            assert_eq!(text_h, 20.0, "20px line box, variant {variant:?}");
            let expected = total_row_height(variant, 20.0);
            assert_eq!(row_h, expected, "variant {variant:?}");
        }
    }

    #[test]
    fn tall_content_grows_all_variants() {
        for variant in [
            MarkerVariant::Default,
            MarkerVariant::Separator,
            MarkerVariant::Border,
        ] {
            let mut h = Harness::new();
            let mut row_h = 0.0;
            for _ in 0..3 {
                let (resp, _) = h.run(|ui| {
                    Marker::new().variant(variant).show(ui, |ui| {
                        ui.allocate_exact_size(Vec2::new(50.0, 40.0), Sense::hover());
                    })
                });
                row_h = resp.response.rect.height();
            }
            assert_eq!(
                row_h,
                total_row_height(variant, 40.0),
                "variant {variant:?}"
            );
        }
    }

    #[test]
    fn gap_is_eight_px_explicit_despite_hostile_ambient_spacing() {
        let mut h = Harness::new();
        let mut gap = 0.0;
        for _ in 0..3 {
            let ((icon, text), _) = h.run(|ui| {
                // Hostile ambient spacing: the row must NOT inherit it.
                ui.style_mut().spacing.item_spacing = Vec2::new(30.0, 30.0);
                Marker::new()
                    .show(ui, |ui| {
                        let icon = MarkerIcon::new(Icon::Sparkles).show(ui).rect;
                        let text = MarkerContent::new("Hello").show(ui).rect;
                        (icon, text)
                    })
                    .inner
            });
            gap = text.left() - icon.right();
        }
        assert!(
            approx_eq(gap, 8.0, 0.01),
            "gap-2 must be exactly 8px, got {gap}"
        );
    }

    #[test]
    fn separator_rules_twelve_px_from_centred_content() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                let theme = ShadcnTheme::of_ui(ui);
                let r = Marker::new()
                    .variant(MarkerVariant::Separator)
                    .show(ui, |ui| {
                        MarkerContent::new("or continue with").show(ui).rect
                    });
                (r.response.rect, r.inner, theme.palette.border)
            }));
        }
        let ((row, content, border), out) = last.unwrap();
        assert_eq!(row.height(), 20.0);
        // Content centred: equal margins left and right (within rounding).
        let left_margin = content.left() - row.left();
        let right_margin = row.right() - content.right();
        assert!(
            approx_eq(left_margin, right_margin, 1.0),
            "content must centre: left {left_margin}, right {right_margin}"
        );
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 2, "two rules");
        let (left_seg, left_stroke) = &segs[0];
        let (right_seg, right_stroke) = &segs[1];
        // 12px from the content on each side.
        assert!(approx_eq(left_seg[1].x, content.left() - 12.0, 1.0));
        assert!(approx_eq(right_seg[0].x, content.right() + 12.0, 1.0));
        assert_eq!(left_seg[0].x, row.left());
        assert_eq!(right_seg[1].x, row.right());
        // Vertically centred on the row, 1px, border token.
        for seg in [left_seg, right_seg] {
            assert_eq!(seg[0].y, seg[1].y);
            assert_eq!(seg[0].y, row.center().y);
        }
        assert_eq!(left_stroke.width, 1.0);
        assert_eq!(right_stroke.width, 1.0);
        assert_eq!(left_stroke.color, border);
        assert_eq!(right_stroke.color, border);
        // Rules share the remaining width equally.
        let left_len = left_seg[1].x - left_seg[0].x;
        let right_len = right_seg[1].x - right_seg[0].x;
        assert!(approx_eq(left_len, right_len, 1.0));
        // flex-none: the content keeps its natural width, not stretched.
        assert!(content.width() < row.width());
    }

    #[test]
    fn separator_empty_sixteen_px_gap_centred() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                Marker::new()
                    .variant(MarkerVariant::Separator)
                    .show(ui, |_| {})
                    .response
                    .rect
            }));
        }
        let (row, out) = last.unwrap();
        assert_eq!(row.height(), 16.0);
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].0[0].x, 0.0);
        assert_eq!(segs[0].0[1].x, 192.0);
        assert_eq!(segs[1].0[0].x, 208.0);
        assert_eq!(segs[1].0[1].x, 400.0);
        assert_eq!(segs[0].0[0].y, 8.0, "centred on the 16px row");
        assert_eq!(segs[1].0[0].y, 8.0);
    }

    #[test]
    fn separator_rules_stay_centred_on_grown_row() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                Marker::new()
                    .variant(MarkerVariant::Separator)
                    .show(ui, |ui| {
                        ui.allocate_exact_size(Vec2::new(50.0, 40.0), Sense::hover());
                    })
                    .response
                    .rect
            }));
        }
        let (row, out) = last.unwrap();
        assert_eq!(row.height(), 40.0);
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 2);
        for (seg, _) in &segs {
            assert_eq!(seg[0].y, 20.0, "centred on the grown row");
        }
    }

    #[test]
    fn border_rule_geometry() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                let theme = ShadcnTheme::of_ui(ui);
                let r = Marker::new()
                    .variant(MarkerVariant::Border)
                    .show(ui, |ui| MarkerContent::new("Hello").show(ui));
                (r.response.rect, theme.palette.border)
            }));
        }
        let ((row, border), out) = last.unwrap();
        assert_eq!(row.height(), 29.0, "20px content + 8px pb-2 + 1px border");
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 1, "one full-width rule");
        let (seg, stroke) = &segs[0];
        assert_eq!(seg[0].x, row.left());
        assert_eq!(seg[1].x, row.right());
        assert_eq!(seg[0].y, 28.5, "content bottom + 8.5");
        assert_eq!(seg[1].y, 28.5);
        assert_eq!(stroke.width, 1.0);
        assert_eq!(stroke.color, border);
    }

    #[test]
    fn border_empty_row_rule() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                Marker::new()
                    .variant(MarkerVariant::Border)
                    .show(ui, |_| {})
                    .response
                    .rect
            }));
        }
        let (row, out) = last.unwrap();
        assert_eq!(row.height(), 16.0, "empty border row totals 16px");
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 1);
        assert_eq!(
            segs[0].0[0].y, 15.5,
            "rule is the bottom row of the 16px row"
        );
    }

    #[test]
    fn separator_content_text_centres_but_default_stays_left() {
        let text = "Short\nA much longer second line";
        // Separator: each line centres within the content span.
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                Marker::new()
                    .variant(MarkerVariant::Separator)
                    .show(ui, |ui| MarkerContent::new(text).show(ui))
            }));
        }
        let (_, out) = last.unwrap();
        let ts = texts(&out);
        assert_eq!(ts.len(), 1);
        let (pos, galley, _) = &ts[0];
        assert_eq!(galley.rows.len(), 2);
        assert!(
            approx_eq(pos.x, 200.0, 1.0),
            "centred galley origin sits at the row centre, got {}",
            pos.x
        );
        for row in &galley.rows {
            let glyphs = &row.row.glyphs;
            let first = &glyphs[0];
            let last_g = &glyphs[glyphs.len() - 1];
            let span_min = pos.x + row.pos.x + first.pos.x;
            let span_max = pos.x + row.pos.x + last_g.pos.x + last_g.advance_width;
            let center = (span_min + span_max) / 2.0;
            assert!(
                approx_eq(center, pos.x, 1.0),
                "each line centres on the galley origin: {center} vs {}",
                pos.x
            );
        }

        // Default: lines share a left edge (no centring outside separator).
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| Marker::new().show(ui, |ui| MarkerContent::new(text).show(ui))));
        }
        let (_, out) = last.unwrap();
        let ts = texts(&out);
        assert_eq!(ts.len(), 1);
        let (pos, galley, _) = &ts[0];
        let mut lefts = Vec::new();
        for row in &galley.rows {
            let first = &row.row.glyphs[0];
            lefts.push(pos.x + row.pos.x + first.pos.x);
        }
        assert!(
            approx_eq(lefts[0], lefts[1], 0.01),
            "left-aligned lines share a left edge: {lefts:?}"
        );
    }

    #[test]
    fn long_text_wraps_and_grows_row() {
        // A sentence wraps at the available width.
        let mut h = Harness::new();
        let mut content_h = 0.0;
        for _ in 0..3 {
            let (rect, _) = h.step(120.0, Vec::new(), |ui| {
                Marker::new()
                    .show(ui, |ui| {
                        MarkerContent::new("This is a fairly long marker sentence that must wrap")
                            .show(ui)
                            .rect
                    })
                    .inner
            });
            content_h = rect.height();
        }
        assert!(content_h >= 40.0, "wrapped to 2+ rows, got {content_h}");
        assert_eq!(content_h % 20.0, 0.0, "whole 20px line boxes");

        // An unbroken word breaks mid-word (`wrap-break-word`).
        let mut h = Harness::new();
        let mut content_h = 0.0;
        for _ in 0..3 {
            let (rect, _) = h.step(120.0, Vec::new(), |ui| {
                Marker::new()
                    .show(ui, |ui| MarkerContent::new("w".repeat(40)).show(ui).rect)
                    .inner
            });
            content_h = rect.height();
        }
        assert!(
            content_h >= 40.0,
            "unbroken word must break and grow the row, got {content_h}"
        );
    }

    #[test]
    fn content_uses_muted_foreground_and_no_decorations() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                let theme = ShadcnTheme::of_ui(ui);
                Marker::new().show(ui, |ui| {
                    MarkerContent::new("Hello").show(ui);
                });
                theme.palette.muted_foreground
            }));
        }
        let (muted, out) = last.unwrap();
        let ts = texts(&out);
        assert_eq!(ts.len(), 1);
        let job_color = ts[0].1.job.sections[0].format.color;
        assert_eq!(job_color, muted);
        // No background, radius, or shadow: the ONLY shape is the text.
        assert_eq!(
            out.shapes.len(),
            1,
            "a plain text marker paints nothing but its text"
        );
    }

    #[test]
    fn link_resting_underline_geometry() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                let theme = ShadcnTheme::of_ui(ui);
                let r = Marker::new().show(ui, |ui| MarkerLink::new("Learn more").show(ui));
                (r.inner.rect, theme.palette.muted_foreground)
            }));
        }
        let ((link_rect, muted), out) = last.unwrap();
        assert_eq!(link_rect.height(), 20.0, "20px line box");
        let ts = texts(&out);
        assert_eq!(ts.len(), 1);
        let (pos, galley, override_color) = &ts[0];
        assert_eq!(*override_color, Some(muted), "resting colour");
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 1, "one underline for one row");
        let (seg, stroke) = &segs[0];
        assert_eq!(stroke.width, 1.0);
        assert_eq!(stroke.color, muted, "underline follows text colour");
        // Underline 3px below the row baseline, spanning the glyphs.
        let row = &galley.rows[0];
        let glyphs = &row.row.glyphs;
        let first = &glyphs[0];
        let last_g = &glyphs[glyphs.len() - 1];
        let baseline = pos.y + row.pos.y + first.pos.y;
        assert!(approx_eq(seg[0].y, baseline + 3.0, 0.01));
        assert!(approx_eq(seg[0].x, pos.x + row.pos.x + first.pos.x, 0.01));
        assert!(approx_eq(
            seg[1].x,
            pos.x + row.pos.x + last_g.pos.x + last_g.advance_width,
            0.01
        ));
    }

    #[test]
    fn link_hover_turns_foreground_instantly() {
        let mut h = Harness::new();
        // Settle, then capture the link rect.
        let mut link_rect = Rect::NOTHING;
        for _ in 0..2 {
            let (rect, _) = h.run(|ui| {
                Marker::new()
                    .show(ui, |ui| MarkerLink::new("Learn more").show(ui))
                    .inner
                    .rect
            });
            link_rect = rect;
        }
        // Hover the link: the SAME frame must already show `foreground`.
        let (foreground, out) = h.step(
            400.0,
            vec![egui::Event::PointerMoved(link_rect.center())],
            |ui| {
                let theme = ShadcnTheme::of_ui(ui);
                Marker::new().show(ui, |ui| MarkerLink::new("Learn more").show(ui));
                theme.palette.foreground
            },
        );
        let ts = texts(&out);
        assert_eq!(ts.len(), 1);
        assert_eq!(ts[0].2, Some(foreground), "hover colour applies instantly");
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 1);
        assert_eq!(
            segs[0].1.color, foreground,
            "underline follows hover colour"
        );
    }

    #[test]
    fn link_click_surfaces_via_response() {
        let mut h = Harness::new();
        let mut link_rect = Rect::NOTHING;
        for _ in 0..2 {
            let (rect, _) = h.run(|ui| {
                Marker::new()
                    .show(ui, |ui| MarkerLink::new("Learn more").show(ui))
                    .inner
                    .rect
            });
            link_rect = rect;
        }
        let center = link_rect.center();
        h.step(400.0, vec![egui::Event::PointerMoved(center)], |ui| {
            Marker::new().show(ui, |ui| MarkerLink::new("Learn more").show(ui));
        });
        h.step(
            400.0,
            vec![egui::Event::PointerButton {
                pos: center,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::default(),
            }],
            |ui| {
                Marker::new().show(ui, |ui| MarkerLink::new("Learn more").show(ui));
            },
        );
        let (clicked, _) = h.step(
            400.0,
            vec![egui::Event::PointerButton {
                pos: center,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::default(),
            }],
            |ui| {
                Marker::new()
                    .show(ui, |ui| MarkerLink::new("Learn more").show(ui))
                    .inner
                    .clicked()
            },
        );
        assert!(clicked, "D2: clicks surface via the Response");
    }

    #[test]
    fn link_multiline_underlines_every_row() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last =
                Some(h.run(|ui| Marker::new().show(ui, |ui| MarkerLink::new("One\nTwo").show(ui))));
        }
        let (_, out) = last.unwrap();
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 2, "one underline per row");
        assert!(
            approx_eq(segs[1].0[0].y - segs[0].0[0].y, 20.0, 0.01),
            "rows are one line box apart"
        );
    }

    #[test]
    fn link_focusable_with_no_painted_ring() {
        let mut h = Harness::new();
        let mut baseline_shapes = 0usize;
        for _ in 0..2 {
            let (_, out) = h.run(|ui| {
                Marker::new().show(ui, |ui| MarkerLink::new("Learn more").show(ui));
            });
            baseline_shapes = out.shapes.len();
        }
        // Tab to the link, then a plain frame: focus persists, no ring.
        h.step(
            400.0,
            vec![egui::Event::Key {
                key: egui::Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            |ui| {
                Marker::new().show(ui, |ui| MarkerLink::new("Learn more").show(ui));
            },
        );
        let (focused, out) = h.run(|ui| {
            Marker::new()
                .show(ui, |ui| MarkerLink::new("Learn more").show(ui))
                .inner
                .has_focus()
        });
        assert!(focused, "anchors are Tab-focusable (spec §10)");
        assert_eq!(
            out.shapes.len(),
            baseline_shapes,
            "no focus ring is painted: no focus styling is defined"
        );
    }

    #[test]
    fn nested_markers_render_independently() {
        let mut h = Harness::new();
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                Marker::new()
                    .variant(MarkerVariant::Separator)
                    .show(ui, |ui| {
                        Marker::new().show(ui, |ui| {
                            MarkerContent::new("Short\nA much longer second line").show(ui)
                        });
                    })
            }));
        }
        let (_, out) = last.unwrap();
        // The inner (default) text stays left-aligned: the variant context
        // stack popped correctly after the inner row.
        let ts = texts(&out);
        assert_eq!(ts.len(), 1);
        let (pos, galley, _) = &ts[0];
        let mut lefts = Vec::new();
        for row in &galley.rows {
            let first = &row.row.glyphs[0];
            lefts.push(pos.x + row.pos.x + first.pos.x);
        }
        assert!(
            approx_eq(lefts[0], lefts[1], 0.01),
            "inner default text left-aligned: {lefts:?}"
        );
    }

    #[test]
    fn show_custom_accepts_arbitrary_content() {
        let mut h = Harness::new();
        let mut row_h = 0.0;
        for _ in 0..3 {
            let (resp, _) = h.run(|ui| {
                Marker::new().show(ui, |ui| {
                    MarkerContent::show_custom(ui, |ui| {
                        ui.allocate_exact_size(Vec2::new(30.0, 32.0), Sense::hover());
                    });
                })
            });
            row_h = resp.response.rect.height();
        }
        assert_eq!(row_h, 32.0, "row grows with arbitrary slot content");
    }

    #[test]
    fn dark_mode_token_slots() {
        let mut h = Harness::with_mode(Mode::Dark);
        let mut last = None;
        for _ in 0..3 {
            last = Some(h.run(|ui| {
                let theme = ShadcnTheme::of_ui(ui);
                Marker::new()
                    .variant(MarkerVariant::Separator)
                    .show(ui, |ui| {
                        MarkerContent::new("Hello").show(ui);
                        MarkerLink::new("Link").show(ui);
                    });
                (
                    theme.palette.muted_foreground,
                    theme.palette.border,
                    theme.palette.foreground,
                )
            }));
        }
        let ((muted, border, _foreground), out) = last.unwrap();
        assert!(
            border.a() > 0 && border.a() < 255,
            "dark border stays translucent, got alpha {}",
            border.a()
        );
        let segs = line_segments(&out);
        assert_eq!(segs.len(), 3, "two rules + link underline");
        let rules: Vec<_> = segs
            .iter()
            .filter(|(_, stroke)| stroke.color == border)
            .collect();
        assert_eq!(rules.len(), 2, "both rules use the border token");
        let ts = texts(&out);
        assert_eq!(ts.len(), 2, "content + link");
        // Content text bakes `muted-foreground`; the link overrides with it.
        assert_eq!(ts[0].1.job.sections[0].format.color, muted);
        assert_eq!(ts[1].2, Some(muted));
        let link_underlines: Vec<_> = segs
            .iter()
            .filter(|(_, stroke)| stroke.color == muted)
            .collect();
        assert_eq!(
            link_underlines.len(),
            1,
            "link underline uses the text colour in dark mode too"
        );
    }
}
