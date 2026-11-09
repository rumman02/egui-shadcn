//! shadcn `table.tsx` — a compound table layout.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/table.tsx`: `Table` wraps
//! `TableHeader` / `TableBody` / `TableFooter` sections of `TableRow`s made
//! of `TableHead` (`h-10 px-2 font-medium`) and `TableCell` (`p-2`) cells,
//! ending with an optional `TableCaption` (`mt-4 text-sm
//! text-muted-foreground`). Rows have `border-b`, `hover:bg-muted/50` and
//! `data-[state=selected]:bg-muted`; the footer is `border-t bg-muted/50
//! font-medium`; the body's last row drops its border.
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
//! click-to-select recipes. The caption is the typography
//! [`TableCaption`](crate::components::typography::TableCaption) slot (the
//! source adds `mt-4`, so give it 16px of space above).
//!
//! Known gap: the body/footer's last-row border drop (`border-0`) is
//! learned over two frames, so the very first frame after a resize paints
//! the final row's bottom border one frame late.
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

use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{Color32, FontId, InnerResponse, Pos2, Rect, Response, Sense, Ui, Vec2};

/// The row background for its state: `data-[state=selected]:bg-muted`,
/// `hover:bg-muted/50` (pure, unit-testable).
pub(crate) fn row_bg(p: &Palette, selected: bool, hovered: bool) -> Color32 {
    if selected {
        p.muted
    } else if hovered {
        color_alpha(p.muted, 0.5)
    } else {
        Color32::TRANSPARENT
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

/// Section context for rows (body/footer membership and last-row tracking).
#[derive(Clone, Copy, Debug)]
struct SectionCtx {
    tracker_id: egui::Id,
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
        push_section(ui, SectionCtx { tracker_id: id });
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
        push_section(ui, SectionCtx { tracker_id: id });
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
        push_section(ui, SectionCtx { tracker_id: id });
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
/// data-[state=selected]:bg-muted`. Rows report hover via the response; the
/// source's rows are not buttons, so they only sense hover unless
/// [`TableRow::interactive`] opts them into click events.
#[derive(Clone, Debug, Default)]
pub struct TableRow {
    selected: bool,
    interactive: bool,
    id_salt: Option<String>,
}

impl TableRow {
    /// Creates a table row.
    pub fn new() -> Self {
        Self {
            selected: false,
            interactive: false,
            id_salt: None,
        }
    }

    /// Marks the row `data-state="selected"`.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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

        // Last row of a body/footer drops its bottom border.
        let mut is_last = false;
        if let Some(section) = current_section(ui) {
            let (_, last) = tracker::next_index(ui, section.tracker_id);
            is_last = last;
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
                row_bg(&p, self.selected, hovered),
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
    id_salt: Option<String>,
}

impl TableHead {
    /// Creates a header cell with the given label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            width: None,
            id_salt: None,
        }
    }

    /// Fixes the column width (headers without a width take the remaining
    /// space; give every non-last column a width to align rows).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
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
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
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
}

/// `TableCell` — `p-2 align-middle whitespace-nowrap`. Either set the text
/// with [`TableCell::text`] or show arbitrary content with [`Self::show`].
#[derive(Clone, Debug, Default)]
pub struct TableCell {
    text: Option<String>,
    width: Option<f32>,
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
            id_salt: None,
        }
    }

    /// Fixes the column width (see [`TableHead::width`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
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
        egui::Frame::new()
            .inner_margin(egui::Margin::same(TABLE_CELL_PADDING as i8))
            .show(ui, |ui| {
                ui.set_min_width((cell_w - TABLE_CELL_PADDING * 2.0).max(0.0));
                ui.spacing_mut().item_spacing.y = 0.0;
                content(ui)
            })
    }

    /// Renders a text cell.
    pub fn show_text(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let text = self.text.clone().unwrap_or_default();
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui.painter().layout_no_wrap(text, font, p.foreground);
        let w = self.width.unwrap_or_else(|| ui.available_width());
        let cell_w = (galley.size().x + TABLE_CELL_PADDING * 2.0)
            .max(w)
            .min(ui.available_width());
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(cell_w, galley.size().y + TABLE_CELL_PADDING * 2.0),
            Sense::hover(),
        );
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
    fn head_geometry_matches_source() {
        // h-10 px-2
        assert_eq!(TABLE_HEAD_HEIGHT, 40.0);
        assert_eq!(TABLE_CELL_PADDING, 8.0);
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
