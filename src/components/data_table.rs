//! shadcn `data-table` — the docs composition built on `ui/table.tsx`
//! (`data-table-demo.tsx` + the tasks/dashboard examples): column
//! descriptors, click-to-sort headers, a text filter toolbar, pagination
//! footer with a rows-per-page select, and row selection.
//!
//! The React version is a TanStack `useTable` wiring over the `Table`
//! primitives; here the row model is plain Rust:
//!
//! - [`DataTableColumn`] — header label, optional fixed width, a `cell`
//!   closure drawing the cell content, and an optional `sort_key` (which
//!   also marks the column sortable).
//! - [`DataTable`] — generic over the row type `T`. Sorting + page state
//!   persist per `id_salt`; the filter text, the column visibility and the
//!   selection are controlled (`&mut String` / `&mut Vec<bool>` /
//!   `&mut Vec<usize>`), like the source's `columnFilters` /
//!   `columnVisibility` / `rowSelection` state. When column visibility is
//!   enabled the toolbar gains the `Columns <ChevronDown>` dropdown
//!   (`DropdownMenuCheckboxItem` per hideable column).
//! - [`SortValue`] — the comparable extracted per row (`Text` / `Number`).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let rows = vec![Payment { status: "success".into(), email: "a@b.c".into(), amount: 316.0 }];
//! let mut filter = String::new();
//! let mut selected: Vec<usize> = Vec::new();
//!
//! DataTable::new(&rows)
//!     .id_salt("payments")
//!     .filter(&mut filter, |p, needle| p.email.contains(needle))
//!     .filter_placeholder("Filter emails...")
//!     .page_size(10)
//!     .selection(&mut selected)
//!     .column(
//!         DataTableColumn::new("status", "Status")
//!             .sort_key(|p| SortValue::Text(p.status.clone()))
//!             .cell(|ui, p| { ui.label(p.status.clone()); }),
//!     )
//!     .column(
//!         DataTableColumn::new("amount", "Amount")
//!             .align(DataTableCellAlign::End)
//!             .sort_key(|p| SortValue::Number(p.amount))
//!             .cell(|ui, p| { ui.label(format!("${:.2}", p.amount)); }),
//!     )
//!     .row_actions(|ui, p| { /* DropdownMenu etc. */ })
//!     .show(ui);
//! ```

use crate::components::table::{row_bg, TABLE_CELL_PADDING, TABLE_HEAD_HEIGHT};
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{FontId, Id, Pos2, Rect, Sense, Ui, Vec2};
use std::cmp::Ordering;

/// Sort direction for a column (`asc` / `desc`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SortDirection {
    /// Ascending.
    #[default]
    Ascending,
    /// Descending.
    Descending,
}

impl SortDirection {
    /// The other direction.
    pub fn toggle(self) -> Self {
        match self {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        }
    }
}

/// A comparable extracted from a row for sorting (`sortFn: text` /
/// `sortFn: alphanumeric` in the source).
#[derive(Clone, Debug, PartialEq)]
pub enum SortValue {
    /// String comparison.
    Text(String),
    /// Numeric comparison.
    Number(f64),
}

impl SortValue {
    /// Compares two values (text is case-sensitive, like `sortFn_text`).
    pub fn compare(&self, other: &SortValue) -> Ordering {
        match (self, other) {
            (SortValue::Text(a), SortValue::Text(b)) => a.cmp(b),
            (SortValue::Number(a), SortValue::Number(b)) => {
                a.partial_cmp(b).unwrap_or(Ordering::Equal)
            }
            // Numbers sort before text (deterministic for mixed columns).
            (SortValue::Number(_), SortValue::Text(_)) => Ordering::Less,
            (SortValue::Text(_), SortValue::Number(_)) => Ordering::Greater,
        }
    }
}

impl From<&str> for SortValue {
    fn from(s: &str) -> Self {
        SortValue::Text(s.to_owned())
    }
}

impl From<String> for SortValue {
    fn from(s: String) -> Self {
        SortValue::Text(s)
    }
}

impl From<f64> for SortValue {
    fn from(v: f64) -> Self {
        SortValue::Number(v)
    }
}

impl From<i64> for SortValue {
    fn from(v: i64) -> Self {
        SortValue::Number(v as f64)
    }
}

/// Cell content alignment inside a column (`text-right` for amounts).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataTableCellAlign {
    /// Left-aligned (default).
    #[default]
    Start,
    /// Right-aligned (`text-right`).
    End,
}

/// The cell renderer closure type.
pub type CellRenderer<'a, T> = Box<dyn Fn(&mut Ui, &T) + 'a>;

/// The sort-key extractor closure type.
pub type SortKeyFn<'a, T> = Box<dyn Fn(&T) -> SortValue + 'a>;

/// The filter predicate closure type.
pub type FilterFn<'a, T> = Box<dyn Fn(&T, &str) -> bool + 'a>;

/// One column descriptor.
pub struct DataTableColumn<'a, T> {
    /// Stable column id (`column.id`).
    pub id: String,
    /// Header label.
    pub header: String,
    /// Fixed column width; columns without one share the remaining space.
    pub width: Option<f32>,
    /// Cell alignment.
    pub align: DataTableCellAlign,
    /// The cell renderer.
    pub cell: Option<CellRenderer<'a, T>>,
    /// The sort extractor; when present the header becomes clickable.
    pub sort_key: Option<SortKeyFn<'a, T>>,
    /// Whether the column appears in the column-visibility dropdown
    /// (`enableHiding`, default `true`; the source's selection/actions
    /// display columns are `false` — those aren't [`DataTableColumn`]s here).
    pub hideable: bool,
}

impl<'a, T> DataTableColumn<'a, T> {
    /// Creates a column with the given id and header label.
    pub fn new(id: impl Into<String>, header: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            width: None,
            align: DataTableCellAlign::Start,
            cell: None,
            sort_key: None,
            hideable: true,
        }
    }

    /// Fixes the column width (the last flexible column absorbs the rest).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Right-aligns the cell content (`text-right`).
    pub fn align(mut self, align: DataTableCellAlign) -> Self {
        self.align = align;
        self
    }

    /// Marks the column sortable with the value to sort by.
    pub fn sort_key(mut self, key: impl Fn(&T) -> SortValue + 'a) -> Self {
        self.sort_key = Some(Box::new(key));
        self
    }

    /// Excludes the column from the column-visibility dropdown
    /// (`enableHiding: false`).
    pub fn hideable(mut self, hideable: bool) -> Self {
        self.hideable = hideable;
        self
    }

    /// Sets the cell renderer.
    pub fn cell(mut self, cell: impl Fn(&mut Ui, &T) + 'a) -> Self {
        self.cell = Some(Box::new(cell));
        self
    }
}

/// Filter the given candidate row indices by the predicate, unit-testable.
pub(crate) fn filter_indices<T>(
    rows: &[T],
    candidates: &[usize],
    pred: &dyn Fn(&T, &str) -> bool,
    needle: &str,
) -> Vec<usize> {
    if needle.trim().is_empty() {
        return candidates.to_vec();
    }
    let needle = needle.trim();
    candidates
        .iter()
        .copied()
        .filter(|&i| rows.get(i).is_some_and(|r| pred(r, needle)))
        .collect()
}

/// Sort the given candidate row indices by a column's sort key, unit-testable.
pub(crate) fn sort_indices<T>(
    rows: &[T],
    candidates: &[usize],
    key: &dyn Fn(&T) -> SortValue,
    dir: SortDirection,
) -> Vec<usize> {
    let mut v = candidates.to_vec();
    v.sort_by(|&a, &b| {
        let ord = key(&rows[a]).compare(&key(&rows[b]));
        match dir {
            SortDirection::Ascending => ord,
            SortDirection::Descending => ord.reverse(),
        }
    });
    v
}

/// Page count for a total row count, unit-testable.
pub(crate) fn page_count(total: usize, page_size: usize) -> usize {
    if page_size == 0 {
        return 1;
    }
    total.div_ceil(page_size).max(1)
}

/// The `(start, end)` row range of a page, clamped to the data, unit-testable.
pub(crate) fn page_range(total: usize, page: usize, page_size: usize) -> (usize, usize) {
    if page_size == 0 || total == 0 {
        return (0, 0);
    }
    let pages = page_count(total, page_size);
    let page = page.min(pages - 1);
    let start = page * page_size;
    let end = (start + page_size).min(total);
    (start.min(total), end)
}

/// The header select-all checkbox state for a page, unit-testable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectAll {
    /// Nothing on the page is selected.
    None,
    /// Some rows are selected (`"indeterminate"` in the source).
    Some,
    /// Every page row is selected.
    All,
}

pub(crate) fn select_all_state(page_selected: usize, page_len: usize) -> SelectAll {
    if page_len == 0 || page_selected == 0 {
        SelectAll::None
    } else if page_selected >= page_len {
        SelectAll::All
    } else {
        SelectAll::Some
    }
}

/// Add/remove a set of page rows from the selection, unit-testable.
pub(crate) fn toggle_all(selected: &mut Vec<usize>, page_indices: &[usize], select: bool) {
    if select {
        for &i in page_indices {
            if !selected.contains(&i) {
                selected.push(i);
            }
        }
    } else {
        selected.retain(|i| !page_indices.contains(i));
    }
}

/// Persisted per-table state (sorting + pagination), keyed by `id_salt`.
#[derive(Clone, Debug, Default)]
struct TableState {
    sort_col: Option<usize>,
    sort_dir: SortDirection,
    page: usize,
    page_size: usize,
    last_filter: String,
}

fn table_state_id(id: Id) -> Id {
    id.with("table_state")
}

fn read_state(ctx: &egui::Context, id: Id) -> TableState {
    ctx.data_mut(|d| d.get_temp::<TableState>(table_state_id(id)))
        .unwrap_or_default()
}

fn write_state(ctx: &egui::Context, id: Id, state: TableState) {
    ctx.data_mut(|d| d.insert_temp(table_state_id(id), state));
}

/// Response of [`DataTable::show`].
#[derive(Clone, Copy, Debug, Default)]
pub struct DataTableResponse {
    /// Rows selected among the filtered rows (controlled selection).
    pub selected_count: usize,
    /// Rows matching the filter.
    pub filtered_count: usize,
    /// Total row count.
    pub total_count: usize,
    /// The active page (0-based), after clamping.
    pub page: usize,
}

/// The data table. See the [module docs](self) for an example.
pub struct DataTable<'a, T> {
    id_salt: Option<String>,
    rows: &'a [T],
    columns: Vec<DataTableColumn<'a, T>>,
    filter: Option<&'a mut String>,
    filter_fn: Option<FilterFn<'a, T>>,
    filter_placeholder: String,
    visibility: Option<&'a mut Vec<bool>>,
    selection: Option<&'a mut Vec<usize>>,
    actions: Option<CellRenderer<'a, T>>,
    default_page_size: Option<usize>,
    page_size_options: Vec<usize>,
}

impl<'a, T> DataTable<'a, T> {
    /// Creates a data table over the given rows.
    pub fn new(rows: &'a [T]) -> Self {
        Self {
            id_salt: None,
            rows,
            columns: Vec::new(),
            filter: None,
            filter_fn: None,
            filter_placeholder: "Filter...".to_owned(),
            visibility: None,
            selection: None,
            actions: None,
            default_page_size: None,
            page_size_options: vec![10, 20, 30, 40, 50],
        }
    }

    /// Sets a stable id salt (sort + page state persist under it).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Adds a column.
    pub fn column(mut self, column: DataTableColumn<'a, T>) -> Self {
        self.columns.push(column);
        self
    }

    /// Adds several columns.
    pub fn columns(mut self, columns: impl IntoIterator<Item = DataTableColumn<'a, T>>) -> Self {
        self.columns.extend(columns);
        self
    }

    /// Enables the filter toolbar with a controlled filter string and the
    /// row predicate (mirrors `columnFilters` + `filterFn_includesString`).
    pub fn filter(mut self, value: &'a mut String, pred: impl Fn(&T, &str) -> bool + 'a) -> Self {
        self.filter = Some(value);
        self.filter_fn = Some(Box::new(pred));
        self
    }

    /// The toolbar input placeholder (default `"Filter..."`).
    pub fn filter_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.filter_placeholder = placeholder.into();
        self
    }

    /// Enables the `Columns <ChevronDown>` toolbar dropdown with a controlled
    /// visibility vector, index-matched to [`Self::columns`]
    /// (`columnVisibility` + `DropdownMenuCheckboxItem` per hideable column;
    /// `true` = visible).
    pub fn column_visibility(mut self, visibility: &'a mut Vec<bool>) -> Self {
        self.visibility = Some(visibility);
        self
    }

    /// Enables pagination with the default page size (mirrors
    /// `rowPaginationFeature`).
    pub fn page_size(mut self, size: usize) -> Self {
        self.default_page_size = Some(size.max(1));
        self
    }

    /// The rows-per-page select options (default 10/20/30/40/50).
    pub fn page_size_options(mut self, options: Vec<usize>) -> Self {
        self.page_size_options = options;
        self
    }

    /// Enables the selection checkbox column bound to the caller's selected
    /// original row indices (mirrors `rowSelectionFeature`).
    pub fn selection(mut self, selected: &'a mut Vec<usize>) -> Self {
        self.selection = Some(selected);
        self
    }

    /// Adds a trailing per-row actions column rendering the closure
    /// (the source's `MoreHorizontal` dropdown cell).
    pub fn row_actions(mut self, actions: impl Fn(&mut Ui, &T) + 'a) -> Self {
        self.actions = Some(Box::new(actions));
        self
    }

    /// Renders the table (toolbar, bordered table, footer).
    pub fn show(self, ui: &mut Ui) -> DataTableResponse {
        let Self {
            id_salt,
            rows,
            mut columns,
            filter,
            filter_fn,
            filter_placeholder,
            mut visibility,
            mut selection,
            actions,
            default_page_size,
            page_size_options,
        } = self;

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let id = match id_salt.as_deref() {
            Some(salt) => Id::unique(format!("shadcn_data_table_{salt}")),
            None => ui.auto_id_with("shadcn_data_table"),
        };
        let mut state = read_state(ui.ctx(), id);

        // One effective page size, persisted so the select keeps its value.
        if state.page_size == 0 {
            state.page_size = default_page_size.unwrap_or(0);
        }
        let paginated = state.page_size > 0;

        let filter_text: String = filter.as_deref().cloned().unwrap_or_default();
        // `autoResetPageIndex`: changing the filter jumps back to page 0.
        if state.last_filter != filter_text {
            state.page = 0;
            state.last_filter = filter_text.clone();
        }

        // --- row model: filter -> sort -> paginate -----------------------------
        let all: Vec<usize> = (0..rows.len()).collect();
        let filtered = match (&filter, &filter_fn) {
            (Some(_), Some(f)) => filter_indices(rows, &all, f.as_ref(), &filter_text),
            _ => all,
        };
        let filtered = if let Some(col) = state
            .sort_col
            .and_then(|i| columns.get_mut(i))
            .filter(|c| c.sort_key.is_some())
        {
            match col.sort_key.take() {
                Some(key) => {
                    let sorted = sort_indices(rows, &filtered, key.as_ref(), state.sort_dir);
                    col.sort_key = Some(key);
                    sorted
                }
                None => filtered,
            }
        } else {
            filtered
        };
        let pages = page_count(filtered.len(), state.page_size);
        state.page = state.page.min(pages - 1);
        let (start, end) = if paginated {
            page_range(filtered.len(), state.page, state.page_size)
        } else {
            (0, filtered.len())
        };
        let page_rows: Vec<usize> = filtered[start..end].to_vec();

        // --- column visibility --------------------------------------------------
        // `columnVisibility` state: index-matched to the columns, `true` =
        // visible. Synced back to the caller's vector after the toolbar.
        let mut vis: Vec<bool> = visibility.as_deref().cloned().unwrap_or_default();
        vis.resize(columns.len(), true);

        // --- column widths -----------------------------------------------------
        let total = ui.available_width();
        let selection_w = if selection.is_some() { 40.0 } else { 0.0 };
        let actions_w = if actions.is_some() { 56.0 } else { 0.0 };
        let visible = |ci: usize| vis.get(ci).copied().unwrap_or(true);
        let fixed: f32 = columns
            .iter()
            .enumerate()
            .filter(|(ci, _)| visible(*ci))
            .filter_map(|(_, c)| c.width)
            .sum::<f32>()
            + selection_w
            + actions_w;
        let flexible = columns
            .iter()
            .enumerate()
            .filter(|(ci, c)| visible(*ci) && c.width.is_none())
            .count()
            .max(1);
        let flex_w = ((total - fixed) / flexible as f32).max(40.0);

        let mut widths: Vec<f32> = Vec::with_capacity(columns.len() + 2);
        if selection.is_some() {
            widths.push(selection_w);
        }
        // Data-index -> widths index (`usize::MAX` = hidden).
        let mut display_of_column: Vec<usize> = Vec::with_capacity(columns.len());
        for (ci, c) in columns.iter().enumerate() {
            if !visible(ci) {
                display_of_column.push(usize::MAX);
                continue;
            }
            display_of_column.push(widths.len());
            widths.push(c.width.unwrap_or(flex_w).min(total));
        }
        if actions.is_some() {
            widths.push(actions_w);
        }
        let actions_w_idx = widths.len().saturating_sub(1);

        // --- toolbar ---------------------------------------------------------
        if filter.is_some() || visibility.is_some() {
            ui.add_space(crate::theme::tokens::spacing::LG); // py-4 band
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::SM;
                if let Some(f) = filter {
                    crate::components::input::Input::new(f)
                        .placeholder(filter_placeholder)
                        .width(384.0_f32.min(ui.available_width())) // max-w-sm
                        .show(ui);
                }
                if visibility.is_some() {
                    // `ml-auto`: the Columns button hugs the right edge.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        columns_menu(ui, id, &columns, &mut vis);
                    });
                }
            });
            ui.add_space(crate::theme::tokens::spacing::LG);
        }
        if let Some(dest) = &mut visibility {
            **dest = vis;
        }

        // --- table -------------------------------------------------------------
        let painter = ui.painter().clone();
        let border_idx = painter.add(egui::Shape::Noop);
        let table_top = ui.cursor().top();

        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;

            // Header row.
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                if selection.is_some() {
                    let w = widths[0];
                    header_select_cell(ui, id, selection.as_deref_mut(), &page_rows, w);
                }
                for (ci, col) in columns.iter().enumerate() {
                    let w_idx = display_of_column[ci];
                    if w_idx == usize::MAX {
                        continue;
                    }
                    let w = widths[w_idx];
                    header_sort_cell(ui, id, &mut state, ci, col, w, &theme);
                }
                if actions.is_some() {
                    let w = widths[actions_w_idx];
                    ui.allocate_space(Vec2::new(w, TABLE_HEAD_HEIGHT));
                }
            });

            // Body rows.
            if page_rows.is_empty() {
                // `h-24 text-center` "No results." row.
                let (_, rect) = ui.allocate_space(Vec2::new(total, 96.0));
                if ui.is_rect_visible(rect) {
                    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
                    let galley =
                        ui.painter()
                            .layout_no_wrap("No results.".to_owned(), font, p.foreground);
                    ui.painter().galley(
                        Pos2::new(
                            rect.center().x - galley.size().x * 0.5,
                            rect.center().y - galley.size().y * 0.5,
                        ),
                        galley,
                        p.foreground,
                    );
                }
            } else {
                for (pi, &ri) in page_rows.iter().enumerate() {
                    let is_selected = selection.as_deref().is_some_and(|s| s.contains(&ri));
                    let is_last = pi == page_rows.len() - 1;

                    // Row background reserved behind the cells
                    // (`hover:bg-muted/50`, `data-[state=selected]:bg-muted`).
                    let bg_idx = painter.add(egui::Shape::Noop);
                    let inner = ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        if selection.is_some() {
                            let w = widths[0];
                            if let Some(sel) = selection.as_deref_mut() {
                                selection_cell(ui, id, ri, sel, w, is_selected);
                            }
                        }
                        for (ci, col) in columns.iter().enumerate() {
                            let w_idx = display_of_column[ci];
                            if w_idx == usize::MAX {
                                continue;
                            }
                            let w = widths[w_idx];
                            body_cell(ui, row_ref(rows, ri), col, w);
                        }
                        if let Some(action) = &actions {
                            let w = widths[actions_w_idx];
                            actions_cell(ui, row_ref(rows, ri), action.as_ref(), w);
                        }
                    });

                    let rect = inner.response.rect;
                    let hovered = ui
                        .interact(rect, id.with(("row", ri)), Sense::hover())
                        .hovered();
                    painter.set(
                        bg_idx,
                        egui::Shape::rect_filled(rect, 0, row_bg(&p, is_selected, hovered)),
                    );
                    // `border-b`, dropped on the last row.
                    if !is_last {
                        painter.line_segment(
                            [
                                Pos2::new(rect.left(), rect.bottom() - 0.5),
                                Pos2::new(rect.right(), rect.bottom() - 0.5),
                            ],
                            egui::Stroke::new(1.0, p.border),
                        );
                    }
                }
            }
        });

        let table_rect = Rect::from_min_max(
            Pos2::new(ui.cursor().left(), table_top),
            Pos2::new(ui.cursor().left() + total, ui.cursor().top()),
        );
        // `overflow-hidden rounded-md border` wrapper.
        painter.set(
            border_idx,
            egui::Shape::rect_stroke(
                table_rect,
                theme.radius.md_u8(),
                egui::Stroke::new(1.0, p.border),
                egui::epaint::StrokeKind::Inside,
            ),
        );

        let selected_count = selection
            .as_deref()
            .map(|s| s.iter().filter(|i| filtered.contains(i)).count())
            .unwrap_or(0);

        // --- footer ----------------------------------------------------------
        if paginated || selection.is_some() {
            ui.add_space(crate::theme::tokens::spacing::LG); // py-4
            ui.horizontal(|ui| {
                if selection.is_some() {
                    // `text-sm text-muted-foreground` selected count.
                    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
                    let text = format!("{selected_count} of {} row(s) selected.", filtered.len());
                    let galley = ui.painter().layout_no_wrap(text, font, p.muted_foreground);
                    let size = galley.size();
                    ui.painter()
                        .galley(ui.cursor().min, galley, p.muted_foreground);
                    ui.advance_cursor_after_rect(Rect::from_min_size(
                        ui.cursor().min,
                        Vec2::new(size.x, size.y),
                    ));
                }

                if paginated {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        footer_controls(
                            ui,
                            id,
                            &mut state,
                            filtered.len(),
                            &page_size_options,
                            &p,
                            &theme,
                        );
                    });
                }
            });
        }

        let final_page = state.page;
        write_state(ui.ctx(), id, state);

        DataTableResponse {
            selected_count,
            filtered_count: filtered.len(),
            total_count: rows.len(),
            page: final_page,
        }
    }
}

fn row_ref<T>(rows: &[T], index: usize) -> &T {
    &rows[index]
}

/// Header select-all checkbox cell.
fn header_select_cell(
    ui: &mut Ui,
    table_id: Id,
    selected: Option<&mut Vec<usize>>,
    page_rows: &[usize],
    width: f32,
) {
    ui.allocate_space(Vec2::new(width, TABLE_HEAD_HEIGHT));

    let page_selected = selected
        .as_deref()
        .map(|s| page_rows.iter().filter(|i| s.contains(i)).count())
        .unwrap_or(0);
    let state = select_all_state(page_selected, page_rows.len());
    let (checked, indeterminate) = match state {
        SelectAll::None => (false, false),
        SelectAll::Some => (false, true),
        SelectAll::All => (true, false),
    };

    let mut flag = checked;
    let salt = format!("{}_select_all", table_id.value());
    let response = egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(
            (width as i8 / 2 - 8).max(0),
            (TABLE_HEAD_HEIGHT as i8 / 2 - 8).max(0),
        ))
        .show(ui, |ui| {
            crate::components::checkbox::Checkbox::new(&mut flag)
                .indeterminate(indeterminate)
                .id_salt(salt)
                .show(ui)
        })
        .inner;
    if response.clicked() {
        if let Some(selected) = selected {
            toggle_all(selected, page_rows, !matches!(state, SelectAll::All));
        }
    }
}

/// Header cell with the sortable label. The source wraps the label + icon in
/// a ghost `Button` (`hover:bg-accent`), so the whole 40 px cell is the click
/// target and the crate's ghost hover fill (accent light / accent/50 dark)
/// is painted behind it.
fn header_sort_cell<T>(
    ui: &mut Ui,
    table_id: Id,
    state: &mut TableState,
    col_index: usize,
    col: &DataTableColumn<'_, T>,
    width: f32,
    theme: &ShadcnTheme,
) {
    let p = theme.palette;
    let rect = Rect::from_min_size(
        Pos2::new(ui.cursor().left(), ui.cursor().top()),
        Vec2::new(width, TABLE_HEAD_HEIGHT),
    );
    ui.allocate_space(Vec2::new(width, TABLE_HEAD_HEIGHT));

    let sorted = state.sort_col == Some(col_index);
    let sortable = col.sort_key.is_some();
    let response = if sortable {
        Some(ui.interact(rect, table_id.with(("sort", col_index)), Sense::click()))
    } else {
        None
    };
    if response.as_ref().is_some_and(|r| r.hovered()) {
        let dark = theme.mode.is_dark();
        ui.painter().rect_filled(
            rect,
            theme.radius.md_u8(), // ghost button rounded-md
            if dark {
                color_alpha(p.accent, 0.5)
            } else {
                p.accent
            },
        );
    }

    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let galley = ui
        .painter()
        .layout_no_wrap(col.header.clone(), font, p.foreground);

    let icon_w = if sortable { 18.0 } else { 0.0 };
    let content_w = galley.size().x + icon_w;
    let x = match col.align {
        DataTableCellAlign::Start => rect.left() + TABLE_CELL_PADDING,
        DataTableCellAlign::End => rect.right() - TABLE_CELL_PADDING - content_w,
    };
    let text_pos = Pos2::new(x, rect.center().y - galley.size().y * 0.5);
    let text_w = galley.size().x;
    ui.painter().galley(text_pos, galley, p.foreground);

    if sortable {
        let icon_rect = Rect::from_min_size(
            Pos2::new(text_pos.x + text_w + 4.0, rect.center().y - 8.0),
            Vec2::splat(16.0),
        );
        let icon_color = if sorted {
            p.foreground
        } else {
            p.muted_foreground
        };
        // The source rotates ArrowUpDown 180° for desc; the glyph is
        // symmetric, so the direction is shown with ArrowUp / ArrowDown.
        let icon = match (sorted, state.sort_dir) {
            (true, SortDirection::Ascending) => crate::icons::Icon::ArrowUp,
            (true, SortDirection::Descending) => crate::icons::Icon::ArrowDown,
            _ => crate::icons::Icon::ArrowUpDown,
        };
        icon.paint(ui.painter(), icon_rect, icon_color, 2.0);
        if let Some(response) = response {
            if response.clicked() {
                if sorted {
                    state.sort_dir = state.sort_dir.toggle();
                } else {
                    state.sort_col = Some(col_index);
                    state.sort_dir = SortDirection::Ascending;
                }
            }
        }
    }
}

/// The toolbar's `Columns <ChevronDown>` dropdown (data-table-demo.tsx
/// `ml-auto`): an outline Button trigger opening a dropdown of checkbox
/// items, one per hideable column (`checked` = visible, capitalized id).
fn columns_menu<T>(
    ui: &mut Ui,
    table_id: Id,
    columns: &[DataTableColumn<'_, T>],
    visibility: &mut [bool],
) {
    // Temp per-column flags so every CheckboxItem can hold a disjoint
    // `&mut bool` (the borrow checker needs separate allocations); synced
    // back into `visibility` after the menu renders.
    let mut flags: Vec<Box<bool>> = visibility.iter().map(|b| Box::new(*b)).collect();
    let hideable: Vec<bool> = columns.iter().map(|c| c.hideable).collect();
    crate::components::dropdown_menu::DropdownMenu::new("")
        .id_salt(format!("{}_columns", table_id.value()))
        .align(crate::components::dropdown_menu::MenuAlign::End)
        .trigger_widget(|ui| {
            crate::components::button::Button::new("Columns")
                .variant(crate::components::button::ButtonVariant::Outline)
                .icon_end(crate::icons::Icon::ChevronDown)
                .id_salt(format!("{}_columns_trigger", table_id.value()))
                .show(ui)
        })
        .show(ui, |menu| {
            for (ci, flag) in flags.iter_mut().enumerate() {
                if ci >= hideable.len() || !hideable[ci] {
                    continue;
                }
                menu.checkbox_item(crate::components::dropdown_menu::CheckboxItem::new(
                    capitalize(&columns[ci].id),
                    flag.as_mut(),
                ));
            }
        });
    for (ci, flag) in flags.iter().enumerate() {
        if ci < visibility.len() && ci < hideable.len() && hideable[ci] {
            visibility[ci] = **flag;
        }
    }
}

/// The source's `capitalize` class: uppercase the first letter of the
/// column id.
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

/// Body cell rendering a column's closure.
fn body_cell<T>(ui: &mut Ui, row: &T, col: &DataTableColumn<'_, T>, width: f32) {
    egui::Frame::new()
        .inner_margin(egui::Margin::same(TABLE_CELL_PADDING as i8))
        .show(ui, |ui| {
            ui.set_min_width((width - TABLE_CELL_PADDING * 2.0).max(0.0));
            ui.spacing_mut().item_spacing.y = 0.0;
            match col.align {
                DataTableCellAlign::Start => {
                    if let Some(cell) = &col.cell {
                        cell(ui, row);
                    }
                }
                DataTableCellAlign::End => {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if let Some(cell) = &col.cell {
                            cell(ui, row);
                        }
                    });
                }
            }
        });
}

/// Row selection checkbox cell.
fn selection_cell(
    ui: &mut Ui,
    table_id: Id,
    row_index: usize,
    selected: &mut Vec<usize>,
    width: f32,
    is_selected: bool,
) {
    let mut flag = is_selected;
    let response = egui::Frame::new()
        .inner_margin(egui::Margin::same(TABLE_CELL_PADDING as i8))
        .show(ui, |ui| {
            ui.set_width((width - TABLE_CELL_PADDING * 2.0).max(0.0));
            crate::components::checkbox::Checkbox::new(&mut flag)
                .id_salt(format!("{}_sel_{row_index}", table_id.value()))
                .show(ui)
        })
        .inner;
    if response.clicked() {
        if flag {
            if !selected.contains(&row_index) {
                selected.push(row_index);
            }
        } else {
            selected.retain(|i| *i != row_index);
        }
    }
}

/// Row actions cell (right-aligned).
fn actions_cell<T>(ui: &mut Ui, row: &T, action: &dyn Fn(&mut Ui, &T), width: f32) {
    egui::Frame::new()
        .inner_margin(egui::Margin::same(TABLE_CELL_PADDING as i8))
        .show(ui, |ui| {
            ui.set_width((width - TABLE_CELL_PADDING * 2.0).max(0.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                action(ui, row);
            });
        });
}

/// Footer right side: rows-per-page select, "Page x of y", first/prev/next/
/// last outline icon buttons (`data-table-pagination.tsx`). Rendered inside
/// a right-to-left layout, so controls are added rightmost first.
fn footer_controls(
    ui: &mut Ui,
    table_id: Id,
    state: &mut TableState,
    filtered_len: usize,
    options: &[usize],
    p: &Palette,
    theme: &ShadcnTheme,
) {
    let pages = page_count(filtered_len, state.page_size);
    let can_prev = state.page > 0;
    let can_next = state.page + 1 < pages;

    // First / last (added first = outermost / rightmost).
    let btn = |ui: &mut Ui, icon: crate::icons::Icon, enabled: bool, tag: &str| -> bool {
        let (_, rect) = ui.allocate_space(Vec2::splat(32.0));
        let response = ui.interact(rect, table_id.with(tag), Sense::click());
        let fg = if enabled {
            p.foreground
        } else {
            color_alpha(p.muted_foreground, 0.5)
        };
        if ui.is_rect_visible(rect) {
            let fill = if response.hovered() && enabled {
                p.accent
            } else {
                p.background
            };
            // `disabled:opacity-50` dims the whole button (fill + border),
            // keeping the `input` border color.
            let fill = if enabled {
                fill
            } else {
                color_alpha(fill, 0.5)
            };
            let stroke = if enabled {
                p.input
            } else {
                color_alpha(p.input, 0.5)
            };
            ui.painter().rect_filled(rect, theme.radius.md_u8(), fill);
            ui.painter().rect_stroke(
                rect,
                theme.radius.md_u8(),
                egui::Stroke::new(1.0, stroke),
                egui::epaint::StrokeKind::Inside,
            );
            icon.paint(
                ui.painter(),
                Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
                fg,
                2.0,
            );
        }
        response.clicked() && enabled
    };

    let went_last = btn(ui, crate::icons::Icon::ChevronsRight, can_next, "last");
    let went_next = btn(ui, crate::icons::Icon::ChevronRight, can_next, "next");
    let went_prev = btn(ui, crate::icons::Icon::ChevronLeft, can_prev, "prev");
    let went_first = btn(ui, crate::icons::Icon::ChevronsLeft, can_prev, "first");
    if went_last {
        state.page = pages - 1;
    }
    if went_next {
        state.page += 1;
    }
    if went_prev {
        state.page = state.page.saturating_sub(1);
    }
    if went_first {
        state.page = 0;
    }

    // "Page x of y" (text-sm font-medium, centered in w-[100px]).
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let pages_text = format!("Page {} of {}", state.page + 1, pages);
    let galley = ui.painter().layout_no_wrap(pages_text, font, p.foreground);
    let text_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(100.0, 32.0));
    ui.painter().galley(
        Pos2::new(
            text_rect.left() + (text_rect.width() - galley.size().x) * 0.5,
            text_rect.center().y - galley.size().y * 0.5,
        ),
        galley,
        p.foreground,
    );
    ui.advance_cursor_after_rect(text_rect);

    // Rows-per-page select (h-8 w-[70px] sm trigger + label).
    let mut page_size_value = Some(state.page_size.to_string());
    let sel = crate::components::select::Select::new(&mut page_size_value)
        .options(options.iter().map(|o| o.to_string()))
        .size(crate::components::select::SelectSize::Sm)
        .width(70.0)
        .id_salt(format!("{}_pagesize", table_id.value()))
        .show_seeded(ui);
    if let Some(v) = sel.value {
        if let Ok(n) = v.parse::<usize>() {
            if options.contains(&n) && n != state.page_size {
                state.page_size = n;
                state.page = 0;
            }
        }
    }
    // "Rows per page" label on the left of the select.
    let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
    let label = ui
        .painter()
        .layout_no_wrap("Rows per page".to_owned(), font, p.foreground);
    let label_rect = Rect::from_min_size(
        Pos2::new(
            ui.cursor().right() - label.size().x - 8.0,
            ui.cursor().top(),
        ),
        Vec2::new(label.size().x + 8.0, 32.0),
    );
    ui.painter().galley(
        Pos2::new(
            label_rect.right() - label.size().x,
            label_rect.center().y - label.size().y * 0.5,
        ),
        label,
        p.foreground,
    );
    ui.advance_cursor_after_rect(label_rect);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct Payment {
        status: String,
        email: String,
        amount: f64,
    }

    fn rows() -> Vec<Payment> {
        vec![
            Payment {
                status: "success".into(),
                email: "ken99@example.com".into(),
                amount: 316.0,
            },
            Payment {
                status: "success".into(),
                email: "Abe45@example.com".into(),
                amount: 242.0,
            },
            Payment {
                status: "processing".into(),
                email: "monserrat44@example.com".into(),
                amount: 837.0,
            },
            Payment {
                status: "failed".into(),
                email: "silas22@example.com".into(),
                amount: 874.0,
            },
            Payment {
                status: "pending".into(),
                email: "carmella@example.com".into(),
                amount: 721.0,
            },
        ]
    }

    fn email_pred(p: &Payment, needle: &str) -> bool {
        p.email.to_lowercase().contains(&needle.to_lowercase())
    }

    #[test]
    fn sort_value_orders_numbers_and_text() {
        assert_eq!(
            SortValue::Number(1.0).compare(&SortValue::Number(2.0)),
            Ordering::Less
        );
        assert_eq!(
            SortValue::Text("a".into()).compare(&SortValue::Text("b".into())),
            Ordering::Less
        );
        // NaN-safe.
        assert_eq!(
            SortValue::Number(f64::NAN).compare(&SortValue::Number(1.0)),
            Ordering::Equal
        );
    }

    #[test]
    fn filter_indices_matches_substrings_and_trims() {
        let rows = rows();
        let all: Vec<usize> = (0..rows.len()).collect();
        assert_eq!(filter_indices(&rows, &all, &email_pred, ""), all);
        assert_eq!(filter_indices(&rows, &all, &email_pred, "   "), all);
        let hits = filter_indices(&rows, &all, &email_pred, "abe");
        assert_eq!(hits, vec![1]);
        let hits = filter_indices(&rows, &all, &email_pred, "example.com");
        assert_eq!(hits, all);
    }

    #[test]
    fn sort_indices_toggles_direction() {
        let rows = rows();
        let all: Vec<usize> = (0..rows.len()).collect();
        let key = |p: &Payment| SortValue::Number(p.amount);
        let asc = sort_indices(&rows, &all, &key, SortDirection::Ascending);
        let desc = sort_indices(&rows, &all, &key, SortDirection::Descending);
        assert_eq!(asc, vec![1, 0, 4, 2, 3]); // 242, 316, 721, 837, 874
        assert_eq!(desc, vec![3, 2, 4, 0, 1]);
        // Text sort is case-sensitive like sortFn_text.
        let emails = sort_indices(
            &rows,
            &all,
            &|p: &Payment| SortValue::Text(p.email.clone()),
            SortDirection::Ascending,
        );
        assert_eq!(emails[0], 1); // "Abe45..." sorts before lowercase
    }

    #[test]
    fn pagination_math_is_clamped() {
        assert_eq!(page_count(0, 10), 1);
        assert_eq!(page_count(25, 10), 3);
        assert_eq!(page_count(25, 0), 1); // degenerate page size
        assert_eq!(page_range(25, 0, 10), (0, 10));
        assert_eq!(page_range(25, 2, 10), (20, 25));
        // Out-of-range page clamps to the last one.
        assert_eq!(page_range(25, 9, 10), (20, 25));
        assert_eq!(page_range(0, 0, 10), (0, 0));
    }

    #[test]
    fn select_all_states_match_tanstack_semantics() {
        assert_eq!(select_all_state(0, 5), SelectAll::None);
        assert_eq!(select_all_state(2, 5), SelectAll::Some);
        assert_eq!(select_all_state(5, 5), SelectAll::All);
        assert_eq!(select_all_state(3, 0), SelectAll::None); // empty page
    }

    #[test]
    fn toggle_all_adds_and_removes_page_rows() {
        let mut selected = vec![0, 7];
        toggle_all(&mut selected, &[0, 1, 2], true);
        assert_eq!(selected, vec![0, 7, 1, 2]);
        toggle_all(&mut selected, &[0, 1, 2], false);
        assert_eq!(selected, vec![7]);
    }

    #[test]
    fn sort_direction_toggles() {
        assert_eq!(SortDirection::Ascending.toggle(), SortDirection::Descending);
        assert_eq!(SortDirection::Descending.toggle(), SortDirection::Ascending);
    }

    #[test]
    fn renders_light_and_dark_with_all_features() {
        use crate::theme::{Mode, Preset};
        let rows = rows();
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("dt_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(700.0, 500.0))),
                );
                let mut filter = String::new();
                let mut selected: Vec<usize> = Vec::new();
                DataTable::new(&rows)
                    .id_salt("render")
                    .filter(&mut filter, email_pred)
                    .page_size(10)
                    .selection(&mut selected)
                    .column(
                        DataTableColumn::new("status", "Status")
                            .sort_key(|p: &Payment| SortValue::Text(p.status.clone()))
                            .cell(|ui, p| {
                                ui.label(p.status.clone());
                            }),
                    )
                    .column(
                        DataTableColumn::new("amount", "Amount")
                            .align(DataTableCellAlign::End)
                            .sort_key(|p: &Payment| SortValue::Number(p.amount))
                            .cell(|ui, p| {
                                ui.label(format!("${:.2}", p.amount));
                            }),
                    )
                    .row_actions(|ui, _p| {
                        ui.label("...");
                    })
                    .show(ui);
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn empty_filter_result_renders_no_results_row() {
        use crate::theme::{Mode, Preset};
        let rows = rows();
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("dt_empty_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(700.0, 500.0))),
            );
            let mut filter = "zzz-no-match".to_owned();
            let res = DataTable::new(&rows)
                .id_salt("empty")
                .filter(&mut filter, email_pred)
                .column(
                    DataTableColumn::new("status", "Status").cell(|ui, p: &Payment| {
                        ui.label(p.status.clone());
                    }),
                )
                .show(ui);
            assert_eq!(res.filtered_count, 0);
            assert_eq!(res.total_count, 5);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn column_visibility_hides_columns() {
        use crate::theme::{Mode, Preset};
        let rows = rows();
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("dt_vis_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(700.0, 500.0))),
            );
            // Hide "status", show "amount"; the row model is unaffected.
            let mut visibility = vec![false, true];
            let res = DataTable::new(&rows)
                .id_salt("vis")
                .column_visibility(&mut visibility)
                .column(
                    DataTableColumn::new("status", "Status").cell(|ui, p: &Payment| {
                        ui.label(p.status.clone());
                    }),
                )
                .column(
                    DataTableColumn::new("amount", "Amount").cell(|ui, p: &Payment| {
                        ui.label(format!("${:.2}", p.amount));
                    }),
                )
                .show(ui);
            assert_eq!(res.total_count, 5);
            assert_eq!(res.filtered_count, 5);
            // The caller's vector is synced back with the render state.
            assert_eq!(visibility, vec![false, true]);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn capitalize_uppercases_first_char() {
        assert_eq!(capitalize("status"), "Status");
        assert_eq!(capitalize("amountUSD"), "AmountUSD");
        assert_eq!(capitalize(""), "");
    }
}
