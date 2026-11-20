//! shadcn `table.tsx` — a compound table layout.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/table.tsx`: `Table` wraps
//! `TableHeader` / `TableBody` / `TableFooter` sections of `TableRow`s made
//! of `TableHead` (`h-10 px-2 font-medium`) and `TableCell` (`p-2`) cells,
//! ending with an optional `TableCaption` (`mt-4 text-sm
//! text-muted-foreground`). Rows have `border-b`, `hover:bg-muted/50`,
//! `has-aria-expanded:bg-muted/50` and `data-[state=selected]:bg-muted`;
//! the footer is `border-t bg-muted/50 font-medium`; the body's last row
//! drops its border.
//!
//! Columns: give [`TableHead`]/[`TableCell`] an explicit `.width(f32)` to
//! align columns across rows; cells without a width take the remaining
//! space (make the last cell flexible, like `w-auto` vs `w-1/4`).
//!
//! The source wraps the table in `overflow-x-auto`; for wide tables wrap
//! this component in
//! [`ScrollArea`](crate::components::ScrollArea)::new().horizontal(true).
//! Rows only sense hover by default (they are not buttons in the source);
//! call [`TableRow::interactive`] to make rows clickable for
//! click-to-select recipes. Expanded rows render `bg-muted/50` via
//! [`TableRow::expanded`]; cells holding a checkbox take `pr-0` and a 2px
//! downward shift via [`TableCell::checkbox_cell`] /
//! [`TableHead::checkbox_cell`]. The caption is the typography
//! [`TableCaption`](crate::components::typography::TableCaption) slot (the
//! source adds `mt-4`, so give it 16px of space above).
//!
//! Known gap: the body/footer's last-row border drop (`border-0`) is
//! learned over two frames, so the very first frame after a resize paints
//! the final row's bottom border one frame late. Header rows always keep
//! their bottom borders (spec §4.5, N9) — the tracker applies to
//! body/footer sections only.
//!
//! The `transition-colors` 150ms fade (§6) renders as an instant recolour:
//! egui has no per-property CSS transitions, so row backgrounds are set per
//! frame (documented limitation D2, not a defect).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Table::new().show(ui, |ui| {
//!     TableHeader::new().show(ui, |ui| {
//!         TableRow::new().show(ui, |ui| {
//!             TableHead::new("Invoice").show(ui);
//!             TableHead::new("Amount").width(100.0).show(ui);
//!         });
//!     });
//!     TableBody::new().show(ui, |ui| {
//!         TableRow::new().show(ui, |ui| {
//!             TableCell::text("INV-001").show_text(ui);
//!             TableCell::text("$250.00").width(100.0).show_text(ui);
//!         });
//!     });
//!     typography::TableCaption::new("A list of invoices.").show(ui);
//! });
//! ```

use crate::theme::{color_alpha, fonts, Palette, ShadcnTheme};
use egui::{Color32, FontId, InnerResponse, Pos2, Rect, Response, Sense, Ui, Vec2};

/// The row background for its state: `data-[state=selected]:bg-muted`,
/// `hover:bg-muted/50` (pure, unit-testable). Selected wins over hover —
/// it is the opaque fill while hover is half-strength (spec §4.3–§4.4).
/// Expanded rows render the same `bg-muted/50` value as hover
/// (`has-aria-expanded:bg-muted/50`, spec §4.4), so callers pass
/// `hovered || expanded` for the `hovered` argument.
pub(crate) fn row_bg(p: &Palette, selected: bool, hovered: bool) -> Color32 {
    if selected {
        p.muted
    } else if hovered {
        color_alpha(p.muted, 0.5)
    } else {
        Color32::TRANSPARENT
    }
}

/// Cell inner margins: `p-2` (8px all sides) by default. A cell holding a
/// checkbox (`[&:has([role=checkbox])]:pr-0`, spec §2.6–2.7) takes no right
/// padding, and its content shifts 2px down (`translate-y-[2px]`); the
/// bottom margin shrinks by 2px so the cell's total height is unchanged —
/// a transform moves the content without affecting layout.
pub(crate) fn cell_margins(checkbox_cell: bool) -> egui::Margin {
    if checkbox_cell {
        egui::Margin {
            left: TABLE_CELL_PADDING as i8,
            right: 0,
            top: TABLE_CELL_PADDING as i8 + 2,
            bottom: TABLE_CELL_PADDING as i8 - 2,
        }
    } else {
        egui::Margin::same(TABLE_CELL_PADDING as i8)
    }
}

/// Head font: `font-medium` (500) via the embedded Inter Medium face
/// (spec §2.6, §5.2).
pub(crate) fn head_font() -> FontId {
    fonts::medium(crate::theme::tokens::type_scale::SM)
}

/// Cell text font: medium in footer sections — footer cells inherit
/// `font-medium` from the `tfoot` (spec §2.4) — regular elsewhere.
pub(crate) fn cell_text_font(medium: bool) -> FontId {
    if medium {
        head_font()
    } else {
        FontId::proportional(crate::theme::tokens::type_scale::SM)
    }
}

/// `TableHead` height (`h-10` = 40px).
pub const TABLE_HEAD_HEIGHT: f32 = 40.0;

/// Cell/Head horizontal padding (`px-2` head, `p-2` cell).
pub const TABLE_CELL_PADDING: f32 = 8.0;

/// Row tracking so the body/footer's last row can drop its `border-b`
/// (`[&_tr:last-child]:border-0`). Total is learned across frames, like the
/// button-group position plumbing.
mod tracker {
    use egui::{Context, Id, Ui};

    #[derive(Clone, Copy, Debug)]
    struct RowsState {
        frame: u64,
        drawn: usize,
        total: Option<usize>,
    }

    fn states_id() -> Id {
        Id::unique("shadcn_table_row_states")
    }

    fn write(ctx: &Context, id: Id, state: Option<RowsState>) {
        ctx.data_mut(|d| {
            let mut all: Vec<(Id, RowsState)> = d.get_temp(states_id()).unwrap_or_default();
            if let Some(state) = state {
                if let Some(slot) = all.iter_mut().find(|(i, _)| *i == id) {
                    slot.1 = state;
                } else {
                    all.push((id, state));
                }
            } else {
                all.retain(|(i, _)| *i != id);
            }
            d.insert_temp(states_id(), all);
        });
    }

    fn read(ctx: &Context, id: Id) -> Option<RowsState> {
        ctx.data_mut(|d| {
            d.get_temp::<Vec<(Id, RowsState)>>(states_id())
                .and_then(|all| all.into_iter().find(|(i, _)| *i == id).map(|(_, s)| s))
        })
    }

    /// A row container starts; resets the counter for the current frame.
    pub(crate) fn begin(ui: &Ui, id: Id) {
        let ctx = ui.ctx().clone();
        let frame = ctx.cumulative_frame_nr();
        let mut state = read(&ctx, id).unwrap_or(RowsState {
            frame,
            drawn: 0,
            total: None,
        });
        if state.frame != frame {
            state.frame = frame;
            state.drawn = 0;
        }
        write(&ctx, id, Some(state));
    }

    /// The row container ends; records the row count for the next frame.
    pub(crate) fn end(ui: &Ui, id: Id) {
        let ctx = ui.ctx().clone();
        if let Some(mut state) = read(&ctx, id) {
            state.total = Some(state.drawn);
            write(&ctx, id, Some(state));
        }
    }

    /// Returns this row's index and whether it is the container's last row.
    pub(crate) fn next_index(ui: &Ui, id: Id) -> (usize, bool) {
        let ctx = ui.ctx().clone();
        let mut state = match read(&ctx, id) {
            Some(s) if s.frame == ctx.cumulative_frame_nr() => s,
            _ => RowsState {
                frame: ctx.cumulative_frame_nr(),
                drawn: 0,
                total: None,
            },
        };
        let index = state.drawn;
        state.drawn += 1;
        let is_last = state.total == Some(index + 1);
        write(&ctx, id, Some(state));
        (index, is_last)
    }
}

/// Section context for rows: last-row tracking plus the per-section rules
/// rows and cells need (spec §2.4, §4.5).
#[derive(Clone, Copy, Debug)]
struct SectionCtx {
    tracker_id: egui::Id,
    /// Only body/footer sections drop their last row's bottom border;
    /// header rows keep theirs (`[&_tr]:border-b`, no exception — N9).
    drops_last_border: bool,
    /// Footer cells inherit `font-medium` from the `tfoot` (spec §2.4).
    medium_text: bool,
}

fn section_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_table_section_stack")
}

fn push_section(ui: &Ui, ctx: SectionCtx) {
    let mut stack: Vec<SectionCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(section_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx()
        .data_mut(|d| d.insert_temp(section_stack_id(), stack));
}

fn pop_section(ui: &Ui) {
    let mut stack: Vec<SectionCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(section_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(section_stack_id(), stack));
}

fn current_section(ui: &Ui) -> Option<SectionCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<SectionCtx>>(section_stack_id()))
        .and_then(|s| s.last().copied())
}

/// A shadcn table. See the [module docs](self) for an example.
#[derive(Clone, Debug, Default)]
pub struct Table {
    id_salt: Option<String>,
}

impl Table {
    /// Creates a table.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the table around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let inner = ui.vertical(|ui| {
            ui.set_min_width(ui.available_width()); // `w-full`
            ui.style_mut().visuals.override_text_color = Some(theme.palette.foreground);
            ui.spacing_mut().item_spacing.y = 0.0;
            content(ui)
        });
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `TableHeader` — a section whose rows keep their bottom borders.
#[derive(Clone, Debug, Default)]
pub struct TableHeader {
    id_salt: Option<String>,
}

impl TableHeader {
    /// Creates a table header section.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header section.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_table_header"),
        };
        tracker::begin(ui, id);
        push_section(
            ui,
            SectionCtx {
                tracker_id: id,
                drops_last_border: false,
                medium_text: false,
            },
        );
        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            content(ui)
        });
        pop_section(ui);
        tracker::end(ui, id);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `TableBody` — a section whose last row drops its bottom border.
#[derive(Clone, Debug, Default)]
pub struct TableBody {
    id_salt: Option<String>,
}

impl TableBody {
    /// Creates a table body section.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the body section.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_table_body"),
        };
        tracker::begin(ui, id);
        push_section(
            ui,
            SectionCtx {
                tracker_id: id,
                drops_last_border: true,
                medium_text: false,
            },
        );
        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            content(ui)
        });
        pop_section(ui);
        tracker::end(ui, id);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `TableFooter` — `border-t bg-muted/50 font-medium` full-width band.
#[derive(Clone, Debug, Default)]
pub struct TableFooter {
    id_salt: Option<String>,
}

impl TableFooter {
    /// Creates a table footer section.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the footer section with its muted band and top border.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_table_footer"),
        };

        let painter = ui.painter().clone();
        let band_idx = painter.add(egui::Shape::Noop);

        tracker::begin(ui, id);
        push_section(
            ui,
            SectionCtx {
                tracker_id: id,
                drops_last_border: true,
                medium_text: true,
            },
        );
        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            content(ui)
        });
        pop_section(ui);
        tracker::end(ui, id);

        // Full-width `bg-muted/50` band behind the rows, `border-t` on top.
        let band = Rect::from_min_max(
            Pos2::new(ui.cursor().left(), inner.response.rect.min.y),
            Pos2::new(
                ui.cursor().left() + ui.available_width(),
                inner.response.rect.max.y,
            ),
        );
        painter.set(
            band_idx,
            egui::Shape::Rect(egui::epaint::RectShape::filled(
                band,
                0,
                color_alpha(p.muted, 0.5),
            )),
        );
        painter.line_segment(
            [
                Pos2::new(band.left(), band.top() + 0.5),
                Pos2::new(band.right(), band.top() + 0.5),
            ],
            egui::Stroke::new(1.0, p.border),
        );

        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `TableRow` — `border-b hover:bg-muted/50
/// has-aria-expanded:bg-muted/50 data-[state=selected]:bg-muted`. Rows
/// report hover via the response; the source's rows are not buttons, so
/// they only sense hover unless [`TableRow::interactive`] opts them into
/// click events.
#[derive(Clone, Debug, Default)]
pub struct TableRow {
    selected: bool,
    expanded: bool,
    interactive: bool,
    id_salt: Option<String>,
}

impl TableRow {
    /// Creates a table row.
    pub fn new() -> Self {
        Self {
            selected: false,
            expanded: false,
            interactive: false,
            id_salt: None,
        }
    }

    /// Marks the row `data-state="selected"`.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Marks the row expanded (`has-aria-expanded:bg-muted/50`). The
    /// source defines the style rule only — expansion state is owned by the
    /// consumer (data-table, Batch G); this builder is the port mapping
    /// (spec §4.4, OPEN-2/OPEN-5: the `aria-expanded` placement is open,
    /// the requirement is frozen). Selected still wins over expanded.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Makes the row clickable: the returned response then reports
    /// `clicked()` (and click/pointer cursor) for the whole row. The source
    /// marks up clickable rows with links/buttons in the docs examples, so
    /// this stays opt-in and the default keeps hover-only sensing.
    pub fn interactive(mut self, interactive: bool) -> Self {
        self.interactive = interactive;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the row around its cells.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let painter = ui.painter().clone();
        let bg_idx = painter.add(egui::Shape::Noop);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_table_row"),
        };

        // Last row of a body/footer drops its bottom border; header rows
        // keep theirs (spec §4.5, N9 — the section's `drops_last_border`
        // flag, not the row's position, decides).
        let mut is_last = false;
        if let Some(section) = current_section(ui) {
            let (_, last) = tracker::next_index(ui, section.tracker_id);
            is_last = last && section.drops_last_border;
        }

        let inner = ui.horizontal(|ui| {
            ui.set_min_width(ui.available_width());
            ui.spacing_mut().item_spacing.x = 0.0;
            content(ui)
        });

        let rect = inner.response.rect;
        let sense = if self.interactive {
            Sense::click_and_drag()
        } else {
            Sense::hover()
        };
        let response = ui.interact(rect, id, sense);
        let hovered = response.hovered() && !self.selected;
        painter.set(
            bg_idx,
            egui::Shape::Rect(egui::epaint::RectShape::filled(
                rect,
                0,
                // Expanded rows render the same `bg-muted/50` as hover
                // (spec §4.4), so `expanded` ORs into the hovered arm.
                row_bg(&p, self.selected, hovered || self.expanded),
            )),
        );
        if !is_last {
            painter.line_segment(
                [
                    Pos2::new(rect.left(), rect.bottom() - 0.5),
                    Pos2::new(rect.right(), rect.bottom() - 0.5),
                ],
                egui::Stroke::new(1.0, p.border),
            );
        }

        InnerResponse::new(inner.inner, response)
    }
}

/// `TableHead` — `h-10 px-2 text-left align-middle font-medium
/// whitespace-nowrap text-foreground`.
#[derive(Clone, Debug, Default)]
pub struct TableHead {
    text: String,
    width: Option<f32>,
    checkbox_cell: bool,
    id_salt: Option<String>,
}

impl TableHead {
    /// Creates a header cell with the given label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            width: None,
            checkbox_cell: false,
            id_salt: None,
        }
    }

    /// Fixes the column width (headers without a width take the remaining
    /// space; give every non-last column a width to align rows).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Marks the cell as holding a checkbox (`[&:has([role=checkbox])]`):
    /// the cell takes no right padding (`pr-0`, spec §2.6). The checkbox
    /// itself is translated 2px down — that rule applies to content shown
    /// via [`Self::show_with`]; a text head has no checkbox to translate.
    pub fn checkbox_cell(mut self, checkbox_cell: bool) -> Self {
        self.checkbox_cell = checkbox_cell;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header cell.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let w = self.width.unwrap_or_else(|| ui.available_width());
        let font = head_font();
        let galley = ui.painter().layout_no_wrap(self.text, font, p.foreground);

        let cell_w = w.min(ui.available_width());
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(cell_w, TABLE_HEAD_HEIGHT), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(
                Pos2::new(
                    rect.left() + TABLE_CELL_PADDING,
                    rect.center().y - galley.size().y * 0.5,
                ),
                galley,
                p.foreground,
            );
        }
        response
    }

    /// Renders the header cell with arbitrary content (e.g. a select-all
    /// checkbox, spec §2.6/§9.6). The cell keeps the fixed `h-10` height;
    /// with `.checkbox_cell(true)` the content takes `pr-0` and shifts 2px
    /// down (`translate-y-[2px]`), matching the checkbox-cell rules.
    pub fn show_with<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let w = self.width.unwrap_or_else(|| ui.available_width());
        let cell_w = w.min(ui.available_width());
        let margins = cell_margins(self.checkbox_cell);
        // Allocate the exact 40px rect first: `allocate_ui_with_layout`
        // would shrink to the space actually used, which would break the
        // fixed `h-10` height for short content.
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(cell_w, TABLE_HEAD_HEIGHT), Sense::hover());
        let inner = ui.scope_builder(
            egui::UiBuilder::new()
                .max_rect(rect)
                .layout(egui::Layout::top_down(egui::Align::LEFT)),
            |ui| {
                egui::Frame::new().inner_margin(margins).show(ui, |ui| {
                    ui.set_min_width((cell_w - margins.leftf() - margins.rightf()).max(0.0));
                    ui.spacing_mut().item_spacing.y = 0.0;
                    content(ui)
                })
            },
        );
        InnerResponse::new(inner.inner.inner, response)
    }
}

/// `TableCell` — `p-2 align-middle whitespace-nowrap`. Either set the text
/// with [`TableCell::text`] or show arbitrary content with [`Self::show`].
/// A cell holding a checkbox uses [`Self::checkbox_cell`].
#[derive(Clone, Debug, Default)]
pub struct TableCell {
    text: Option<String>,
    width: Option<f32>,
    checkbox_cell: bool,
    id_salt: Option<String>,
}

impl TableCell {
    /// Creates an empty cell (use [`Self::show`] for content).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a text cell.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: Some(text.into()),
            width: None,
            checkbox_cell: false,
            id_salt: None,
        }
    }

    /// Fixes the column width (see [`TableHead::width`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Marks the cell as holding a checkbox (`[&:has([role=checkbox])]`):
    /// the cell takes no right padding (`pr-0`) and its content shifts 2px
    /// down (`[&>[role=checkbox]]:translate-y-[2px]`, spec §2.7). The shift
    /// is relative to the cell's content origin; the cell's total height is
    /// unchanged (see [`cell_margins`]).
    pub fn checkbox_cell(mut self, checkbox_cell: bool) -> Self {
        self.checkbox_cell = checkbox_cell;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the cell with arbitrary content.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let w = self.width.unwrap_or_else(|| ui.available_width());
        let cell_w = w.min(ui.available_width());
        let margins = cell_margins(self.checkbox_cell);
        egui::Frame::new().inner_margin(margins).show(ui, |ui| {
            ui.set_min_width((cell_w - margins.leftf() - margins.rightf()).max(0.0));
            ui.spacing_mut().item_spacing.y = 0.0;
            content(ui)
        })
    }

    /// Renders a text cell.
    pub fn show_text(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let text = self.text.clone().unwrap_or_default();
        // Footer cells inherit `font-medium` from the `tfoot` (spec §2.4);
        // other cells render at regular weight. Text is not a checkbox, so
        // the 2px shift does not apply here — only the `pr-0` does.
        let medium = current_section(ui).map(|s| s.medium_text).unwrap_or(false);
        let font = cell_text_font(medium);
        let galley = ui.painter().layout_no_wrap(text, font, p.foreground);
        let w = self.width.unwrap_or_else(|| ui.available_width());
        let margins = cell_margins(self.checkbox_cell);
        let cell_w = (galley.size().x + margins.leftf() + margins.rightf())
            .max(w)
            .min(ui.available_width());
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(cell_w, galley.size().y + TABLE_CELL_PADDING * 2.0),
            Sense::hover(),
        );
        if ui.is_rect_visible(rect) {
            ui.painter().galley(
                Pos2::new(
                    rect.left() + margins.leftf(),
                    rect.center().y - galley.size().y * 0.5,
                ),
                galley,
                p.foreground,
            );
        }
        response
    }
}

// The table caption (`mt-4 text-sm text-muted-foreground`) is
// [`crate::components::typography::TableCaption`] — the same slot with the
// same classes; use it after a `Table`.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn row_background_matches_source_states() {
        let p = palette(Mode::Light);
        assert_eq!(row_bg(&p, false, false), Color32::TRANSPARENT);
        assert_eq!(row_bg(&p, true, false), p.muted);
        assert_eq!(row_bg(&p, true, true), p.muted); // selected wins
        assert_eq!(row_bg(&p, false, true), color_alpha(p.muted, 0.5));
    }

    #[test]
    fn expanded_rows_render_the_hover_fill() {
        // N7 (spec §4.4): an expanded row renders `bg-muted/50` — the same
        // value as hover. `TableRow::show` passes `hovered || expanded` to
        // `row_bg`, so the expanded fill is the hover fill; the
        // painted-shape test below pins the wiring end-to-end. Selected
        // still wins over expanded (derived-by-value, §4.4).
        let p = palette(Mode::Light);
        assert_eq!(row_bg(&p, false, true), color_alpha(p.muted, 0.5));
        assert_eq!(row_bg(&p, true, true), p.muted);
    }

    #[test]
    fn head_geometry_matches_source() {
        // h-10 px-2
        assert_eq!(TABLE_HEAD_HEIGHT, 40.0);
        assert_eq!(TABLE_CELL_PADDING, 8.0);
    }

    #[test]
    fn head_and_footer_fonts_are_medium() {
        // `font-medium` (500) on the head (spec §2.6) and on footer cells
        // via inheritance (spec §2.4) — the embedded Inter Medium face.
        assert_eq!(head_font().family, fonts::medium_family());
        assert_eq!(head_font().size, crate::theme::tokens::type_scale::SM);
        assert_eq!(cell_text_font(true).family, fonts::medium_family());
        assert_eq!(
            cell_text_font(false).family,
            FontId::proportional(crate::theme::tokens::type_scale::SM).family
        );
    }

    #[test]
    fn checkbox_cell_padding_rules() {
        // `pr-0` + 2px downward content shift, total height unchanged
        // (spec §2.6–2.7, N8).
        let plain = cell_margins(false);
        assert_eq!(
            (plain.left, plain.right, plain.top, plain.bottom),
            (8, 8, 8, 8)
        );
        let checkbox = cell_margins(true);
        assert_eq!(
            (checkbox.left, checkbox.right, checkbox.top, checkbox.bottom),
            (8, 0, 10, 6)
        );
    }

    /// Flattens painted shapes into their line segments (recursing into
    /// `Shape::Vec`).
    fn line_segments(shapes: &[egui::epaint::ClippedShape]) -> Vec<([Pos2; 2], egui::Stroke)> {
        fn push(shape: &egui::Shape, out: &mut Vec<([Pos2; 2], egui::Stroke)>) {
            match shape {
                egui::Shape::LineSegment { points, stroke } => {
                    out.push((*points, *stroke));
                }
                egui::Shape::Vec(shapes) => {
                    for s in shapes {
                        push(s, out);
                    }
                }
                _ => {}
            }
        }
        let mut out = Vec::new();
        for c in shapes {
            push(&c.shape, &mut out);
        }
        out
    }

    /// Flattens painted shapes into their `RectShape`s (recursing into
    /// `Shape::Vec`).
    fn rect_shapes(shapes: &[egui::epaint::ClippedShape]) -> Vec<egui::epaint::RectShape> {
        fn push(shape: &egui::Shape, out: &mut Vec<egui::epaint::RectShape>) {
            match shape {
                egui::Shape::Rect(r) => out.push(r.clone()),
                egui::Shape::Vec(shapes) => {
                    for s in shapes {
                        push(s, out);
                    }
                }
                _ => {}
            }
        }
        let mut out = Vec::new();
        for c in shapes {
            push(&c.shape, &mut out);
        }
        out
    }

    #[test]
    fn header_rows_keep_border_body_last_row_drops_it() {
        // N9 (P1, spec §4.5) + N7 (spec §4.4): the header's last row — the
        // header/body divider — keeps its `border-b` from frame 2 onward,
        // while the body's last row drops its border; an expanded non-last
        // row keeps its border and paints `bg-muted/50`. The header-border
        // bug only shows from frame 2 (the tracker learns totals across
        // frames), so this test runs two frames and judges the second
        // frame's painted shapes.
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let p = palette(Mode::Light);
        let header_rect = std::cell::Cell::new(Rect::NOTHING);
        let body_first_rect = std::cell::Cell::new(Rect::NOTHING);
        let expanded_rect = std::cell::Cell::new(Rect::NOTHING);
        let body_last_rect = std::cell::Cell::new(Rect::NOTHING);
        let footer_rect = std::cell::Cell::new(Rect::NOTHING);
        let mut shapes = Vec::new();
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("table_n9_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                Table::new().show(ui, |ui| {
                    TableHeader::new().show(ui, |ui| {
                        let r = TableRow::new().show(ui, |ui| {
                            TableHead::new("Invoice").show(ui);
                            TableHead::new("Amount").width(90.0).show(ui);
                        });
                        header_rect.set(r.response.rect);
                    });
                    TableBody::new().show(ui, |ui| {
                        let r = TableRow::new().show(ui, |ui| {
                            TableCell::text("INV-001").show_text(ui);
                            TableCell::text("$250.00").width(90.0).show_text(ui);
                        });
                        body_first_rect.set(r.response.rect);
                        let r = TableRow::new().expanded(true).show(ui, |ui| {
                            TableCell::text("INV-002").show_text(ui);
                            TableCell::text("$125.00").width(90.0).show_text(ui);
                        });
                        expanded_rect.set(r.response.rect);
                        let r = TableRow::new().show(ui, |ui| {
                            TableCell::text("INV-003").show_text(ui);
                            TableCell::text("$300.00").width(90.0).show_text(ui);
                        });
                        body_last_rect.set(r.response.rect);
                    });
                    TableFooter::new().show(ui, |ui| {
                        let r = TableRow::new().show(ui, |ui| {
                            TableCell::text("Total").show_text(ui);
                            TableCell::text("$675.00").width(90.0).show_text(ui);
                        });
                        footer_rect.set(r.response.rect);
                    });
                });
            });
            shapes = std::mem::take(&mut out.shapes);
            out.textures_delta.clear();
        }
        let segments = line_segments(&shapes);
        // A 1px border line at a row's bottom edge, in the border token.
        let has_border_at = |rect: Rect| {
            let y = rect.bottom() - 0.5;
            segments.iter().any(|(pts, stroke)| {
                (pts[0].y - y).abs() < 0.01
                    && (pts[1].y - y).abs() < 0.01
                    && (pts[0].x - rect.left()).abs() < 0.01
                    && (pts[1].x - rect.right()).abs() < 0.01
                    && stroke.color == p.border
                    && (stroke.width - 1.0).abs() < 0.01
            })
        };
        assert!(
            has_border_at(header_rect.get()),
            "header's last row keeps its border-b (N9)"
        );
        assert!(
            has_border_at(body_first_rect.get()),
            "body non-last row keeps its border"
        );
        assert!(
            has_border_at(expanded_rect.get()),
            "expanded non-last row keeps its border"
        );
        assert!(
            !has_border_at(body_last_rect.get()),
            "body's last row drops its border"
        );
        assert!(
            !has_border_at(footer_rect.get()),
            "footer's last row drops its border"
        );
        // Footer band's `border-t` paints at the band top.
        let band_y = footer_rect.get().top() + 0.5;
        assert!(
            segments.iter().any(|(pts, stroke)| {
                (pts[0].y - band_y).abs() < 0.01
                    && (pts[1].y - band_y).abs() < 0.01
                    && stroke.color == p.border
            }),
            "footer top border present"
        );
        // Expanded row fill: `bg-muted/50` (N7).
        let rects = rect_shapes(&shapes);
        assert!(
            rects
                .iter()
                .any(|r| r.rect == expanded_rect.get() && r.fill == color_alpha(p.muted, 0.5)),
            "expanded row renders bg-muted/50"
        );
    }

    #[test]
    fn checkbox_cell_content_shifts_down_2px() {
        // N8 (spec §2.6–2.7): a checkbox cell's content sits 2px lower
        // than in a plain cell (`translate-y-[2px]`), with the same total
        // height — measured at the checkbox's allocated rect, in a body
        // cell and in a header cell (`show_with`).
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        for checkbox_cell in [false, true] {
            let mut flag = false;
            let cell_top = std::cell::Cell::new(0.0f32);
            let box_top = std::cell::Cell::new(0.0f32);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("table_checkbox_cell_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                let resp = TableCell::new()
                    .checkbox_cell(checkbox_cell)
                    .show(ui, |ui| {
                        crate::components::checkbox::Checkbox::new(&mut flag).show(ui)
                    });
                cell_top.set(resp.response.rect.min.y);
                box_top.set(resp.inner.rect.min.y);
            });
            out.textures_delta.clear();
            let expected = if checkbox_cell { 10.0 } else { 8.0 };
            assert!(
                (box_top.get() - cell_top.get() - expected).abs() < 0.01,
                "cell checkbox_cell={checkbox_cell}: checkbox top offset {} (want {expected})",
                box_top.get() - cell_top.get()
            );
        }
        // Header cell via `show_with`: same 2px shift inside the fixed
        // 40px head, for the checkbox case.
        let mut flag = false;
        let head_top = std::cell::Cell::new(0.0f32);
        let head_box_top = std::cell::Cell::new(0.0f32);
        let head_height = std::cell::Cell::new(0.0f32);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("table_checkbox_head_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            let resp = TableHead::new("").checkbox_cell(true).show_with(ui, |ui| {
                crate::components::checkbox::Checkbox::new(&mut flag).show(ui)
            });
            head_top.set(resp.response.rect.min.y);
            head_box_top.set(resp.inner.rect.min.y);
            head_height.set(resp.response.rect.height());
        });
        out.textures_delta.clear();
        assert!((head_height.get() - TABLE_HEAD_HEIGHT).abs() < 0.01);
        assert!(
            (head_box_top.get() - head_top.get() - 10.0).abs() < 0.01,
            "head checkbox cell: checkbox top offset {} (want 10.0)",
            head_box_top.get() - head_top.get()
        );
    }

    #[test]
    fn interactive_rows_sense_clicks() {
        // Default rows are hover-only (the source's rows are not buttons);
        // `.interactive(true)` opts in to click events.
        let ctx = egui::Context::default();
        let mut plain_clicked = false;
        let mut interactive_senses_clicks = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("table_sense_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            TableBody::new().show(ui, |ui| {
                let plain = TableRow::new().id_salt("plain_row").show(ui, |ui| {
                    TableCell::text("hover only").show_text(ui);
                });
                plain_clicked = plain.response.clicked();
                let interactive =
                    TableRow::new()
                        .interactive(true)
                        .id_salt("click_row")
                        .show(ui, |ui| {
                            TableCell::text("click me").show_text(ui);
                        });
                interactive_senses_clicks = interactive.response.sense.senses_click();
            });
        });
        out.textures_delta.clear();
        assert!(!plain_clicked);
        assert!(interactive_senses_clicks);
    }

    #[test]
    fn tracker_reports_last_row_after_two_frames() {
        let ctx = egui::Context::default();
        let id = egui::Id::unique("tracker_test");
        for frame in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("tracker_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                tracker::begin(ui, id);
                let (_, last0) = tracker::next_index(ui, id);
                let (_, last1) = tracker::next_index(ui, id);
                let (_, last2) = tracker::next_index(ui, id);
                tracker::end(ui, id);
                if frame >= 1 {
                    // Stable from the second frame on.
                    assert!(!last0);
                    assert!(!last1);
                    assert!(last2);
                }
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn renders_a_full_table_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("table_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
                );
                Table::new().show(ui, |ui| {
                    TableHeader::new().show(ui, |ui| {
                        TableRow::new().show(ui, |ui| {
                            TableHead::new("Invoice").show(ui);
                            TableHead::new("Status").width(100.0).show(ui);
                            TableHead::new("Amount").width(90.0).show(ui);
                        });
                    });
                    TableBody::new().show(ui, |ui| {
                        TableRow::new().show(ui, |ui| {
                            TableCell::text("INV-001").show_text(ui);
                            TableCell::text("Paid").width(100.0).show_text(ui);
                            TableCell::text("$250.00").width(90.0).show_text(ui);
                        });
                        TableRow::new().selected(true).show(ui, |ui| {
                            TableCell::text("INV-002").show_text(ui);
                            TableCell::text("Pending").width(100.0).show_text(ui);
                            TableCell::text("$125.00").width(90.0).show_text(ui);
                        });
                        TableRow::new().show(ui, |ui| {
                            TableCell::new().show(ui, |ui| {
                                ui.label(egui::RichText::new("INV-003").weak());
                            });
                            TableCell::text("Unpaid").width(100.0).show_text(ui);
                            TableCell::text("$300.00").width(90.0).show_text(ui);
                        });
                    });
                    TableFooter::new().show(ui, |ui| {
                        TableRow::new().show(ui, |ui| {
                            TableCell::text("Total").show_text(ui);
                            TableCell::text("").width(100.0).show_text(ui);
                            TableCell::text("$675.00").width(90.0).show_text(ui);
                        });
                    });
                    crate::components::typography::TableCaption::new("A list of recent invoices.")
                        .show(ui);
                });
            });
            out.textures_delta.clear();
        }
    }
}
