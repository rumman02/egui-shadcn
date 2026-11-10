//! shadcn `calendar.tsx` — a month calendar (built on react-day-picker).
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/calendar.tsx` with the nova
//! recipe: `p-2 [--cell-size:--spacing(7)] [--cell-radius:var(--radius-md)]`
//! — an 8 px padded, 196 px wide month grid of 28 px day cells separated by
//! the `week: mt-2` 8 px row gap (also between the weekday header and the
//! first week).
//!
//! Visual mapping (nova): ghost nav chevrons flanking a centered
//! "November 2026" caption (or two bordered month/year dropdown buttons with
//! [`CaptionLayout::Dropdown`]); muted weekday header; day cells with
//! `hover:bg-accent`, today = `bg-accent`, selected single = `bg-primary`,
//! ranges = primary start/end squares with a continuous `bg-accent` middle
//! bar (`rounded-none`) that runs behind the rounded end squares; outside
//! days render in `text-muted-foreground`. Optional ISO week-number column
//! (`showWeekNumber`).
//!
//! The date math (leap years, month rollovers, weekdays, ISO week numbers)
//! is implemented in pure Rust inside this module — no new dependencies.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut date = None;
//! Calendar::single(&mut date).show(ui);
//!
//! let mut days = Vec::new();
//! Calendar::multiple(&mut days).week_starts_on(WeekStart::Monday).show(ui);
//!
//! let mut range = None;
//! Calendar::range(&mut range).show(ui);
//!
//! Calendar::single(&mut date)
//!     .caption_layout(CaptionLayout::Dropdown)
//!     .show_week_numbers(true)
//!     .show(ui);
//! ```

use crate::components::dropdown_menu::{DropdownMenu, MenuAlign, MenuItem};
use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme, Style};
use crate::utils::focus;
use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Key, Painter, Pos2, Rect, Response, Sense, Stroke,
    Ui, Vec2,
};

// ---------------------------------------------------------------------------
// Geometry constants (nova style CSS: .cn-calendar)
// ---------------------------------------------------------------------------

/// Day cell size: nova `[--cell-size:--spacing(7)]` = 28 px.
pub const CALENDAR_CELL_SIZE: f32 = 28.0;
/// Root padding: nova `p-2` = 8 px.
pub const CALENDAR_PADDING: f32 = 8.0;
/// Width of the 7-column day grid.
pub const CALENDAR_GRID_WIDTH: f32 = 7.0 * CALENDAR_CELL_SIZE;
/// Gap between the caption row and the grid (`month: gap-4`).
const CAPTION_GAP: f32 = 16.0;
/// Caption row height (`h-(--cell-size)`).
const CAPTION_HEIGHT: f32 = CALENDAR_CELL_SIZE;
/// Gap above every week row (`week: mt-2` = 8 px; also between the weekday
/// header and the first week).
const WEEK_GAP: f32 = 8.0;
/// Weekday header row height (a table row of `text-[0.8rem]` labels).
const WEEKDAY_ROW_HEIGHT: f32 = 20.0;
/// Nav chevron icon size (`size-4`).
const NAV_ICON_SIZE: f32 = 16.0;
/// Weekday label size (`text-[0.8rem]` = 12.8 px; no theme token).
const WEEKDAY_TEXT_SIZE: f32 = 12.8;
/// Day number size (`text-sm` via the ghost DayButton).
const DAY_TEXT_SIZE: f32 = crate::theme::tokens::type_scale::SM;
/// Caption dropdown paddings (nova `caption_label`: `pl-1.5 pr-1`).
const CAPTION_DROPDOWN_PADDING_LEFT: f32 = 6.0;
const CAPTION_DROPDOWN_PADDING_RIGHT: f32 = 4.0;
/// Gap between the caption dropdown label and its chevron (`gap-1`).
const CAPTION_DROPDOWN_GAP: f32 = 4.0;
/// Gap between the two caption dropdown buttons (`dropdowns: gap-1.5`).
const CAPTION_DROPDOWNS_GAP: f32 = 6.0;
/// Years offered by the year dropdown (`view year ± 12`).
const YEAR_DROPDOWN_RANGE: i32 = 12;
/// Dropdown menu widths for the caption month / year pickers.
const MONTH_MENU_WIDTH: f32 = 104.0;
const YEAR_MENU_WIDTH: f32 = 88.0;

const DAY_ID_KEY: &str = "shadcn_calendar_day";

/// Short weekday labels, indexed from Sunday (matches the source's two-letter
/// `Su Mo Tu …` header).
pub const WEEKDAY_SHORT: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

/// Full month names, indexed 0 = January.
pub const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Short month names ("Jan"), the source's `formatMonthDropdown`
/// (`toLocaleString(…, { month: "short" })`), used by the dropdown caption.
pub const MONTH_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

// ---------------------------------------------------------------------------
// Date math (pure Rust, no dependencies)
// ---------------------------------------------------------------------------

/// Days-per-era helpers: Howard Hinnant's `days_from_civil` /
/// `civil_from_days` (public domain), valid for the whole proleptic
/// Gregorian calendar.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = y as i64 - i64::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let m = m as i64;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = mp + if mp < 10 { 3 } else { -9 }; // [1, 12]
    ((y + i64::from(m <= 2)) as i32, m as u32, d as u32)
}

/// A lightweight proleptic-Gregorian date used by [`Calendar`] and
/// [`crate::components::date_picker::DatePicker`].
///
/// Ordering is chronological (year, then month, then day).
///
/// ```ignore
/// use egui_shadcn_ui::components::calendar::CalendarDate;
///
/// let d = CalendarDate::new(2026, 11, 10).unwrap();
/// assert_eq!(d.weekday(), 2); // Tuesday
/// assert_eq!(d.month_name(), "November");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarDate {
    /// Full year (e.g. `2026`).
    pub year: i32,
    /// Month, 1 = January … 12 = December.
    pub month: u32,
    /// Day of month, 1-based.
    pub day: u32,
}

impl CalendarDate {
    /// Creates a date, validating month/day against the month length.
    /// Returns `None` for out-of-range values (e.g. Feb 30).
    pub fn new(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) || day == 0 || day > Self::days_in_month(year, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// Days since 1970-01-01 (negative before the epoch).
    pub fn epoch_days(self) -> i64 {
        days_from_civil(self.year, self.month, self.day)
    }

    /// Builds a date from days since 1970-01-01.
    pub fn from_epoch_days(days: i64) -> Self {
        let (year, month, day) = civil_from_days(days);
        Self { year, month, day }
    }

    /// Builds a date from seconds since the Unix epoch (UTC — the date
    /// boundaries follow UTC, which is what egui's input clock provides).
    pub fn from_unix_secs(secs: f64) -> Self {
        Self::from_epoch_days((secs / 86_400.0).floor() as i64)
    }

    /// Today's date from the egui input clock (Unix seconds).
    pub fn today(ui: &Ui) -> Self {
        let secs = ui.input(|i| i.time);
        Self::from_unix_secs(secs)
    }

    /// Day of week, `0 = Sunday … 6 = Saturday` (1970-01-01 was a Thursday).
    pub fn weekday(self) -> u32 {
        (self.epoch_days().rem_euclid(7) as u32 + 4) % 7
    }

    /// Number of days in a month, leap-year aware.
    pub fn days_in_month(year: i32, month: u32) -> u32 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }

    /// Proleptic Gregorian leap-year rule.
    pub fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
    }

    /// The first day of this date's month.
    pub fn first_of_month(self) -> Self {
        Self { day: 1, ..self }
    }

    /// First day of the previous month (year rollover safe).
    pub fn prev_month(self) -> Self {
        Self::from_epoch_days(self.first_of_month().epoch_days() - 1).first_of_month()
    }

    /// First day of the next month (year rollover safe).
    pub fn next_month(self) -> Self {
        let first = self.first_of_month();
        Self::from_epoch_days(
            first.epoch_days() + i64::from(Self::days_in_month(first.year, first.month)),
        )
        .first_of_month()
    }

    /// Replaces the day, clamping into the month (Jan 31 → Feb 28).
    pub fn with_day_clamped(self, day: u32) -> Self {
        Self {
            day: day.min(Self::days_in_month(self.year, self.month)).max(1),
            ..self
        }
    }

    /// The next day.
    pub fn tomorrow(self) -> Self {
        Self::from_epoch_days(self.epoch_days() + 1)
    }

    /// The previous day.
    pub fn yesterday(self) -> Self {
        Self::from_epoch_days(self.epoch_days() - 1)
    }

    /// Full English month name ("November").
    pub fn month_name(self) -> &'static str {
        MONTH_NAMES[self.month.clamp(1, 12) as usize - 1]
    }

    /// date-fns `PPP` formatting: "November 10th, 2026" (used by the
    /// date-picker trigger).
    pub fn format_ppp(self) -> String {
        format!("{} {}, {}", self.month_name(), ordinal(self.day), self.year)
    }
}

/// English ordinal suffix for a day number (1st, 2nd, 22nd, 13th …).
fn ordinal(day: u32) -> String {
    let suffix = match day % 100 {
        11..=13 => "th",
        _ => match day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    format!("{day}{suffix}")
}

/// ISO 8601 week number (1–53) of the given day, as react-day-picker's
/// `weekNumber` computes it: the week number of the week's Thursday, which
/// always falls in the ISO year that owns the week. ISO week 1 is the week
/// containing January 4.
pub fn iso_week_number(day: CalendarDate) -> u32 {
    // Monday-based weekday: 0 = Monday … 6 = Sunday.
    let iso_weekday = i64::from((day.weekday() + 6) % 7);
    let thursday = day.epoch_days() + 3 - iso_weekday;
    let thursday_date = CalendarDate::from_epoch_days(thursday);
    let jan4 = days_from_civil(thursday_date.year, 1, 4);
    let jan4_iso_weekday = i64::from((CalendarDate::from_epoch_days(jan4).weekday() + 6) % 7);
    let week1_monday = jan4 - jan4_iso_weekday;
    ((thursday - week1_monday) / 7 + 1) as u32
}

/// Days from `day` back to the first day of its week (the react-day-picker
/// `Home` key target; `End` = first day + 6).
fn days_since_week_start(day: CalendarDate, week_start: WeekStart) -> i64 {
    i64::from((day.weekday() + 7 - week_start.index()) % 7)
}

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------

/// Which day a week starts on (`weekStartsOn` in react-day-picker).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WeekStart {
    /// Sunday first (the source default).
    #[default]
    Sunday,
    /// Monday first.
    Monday,
}

impl WeekStart {
    fn index(self) -> u32 {
        match self {
            WeekStart::Sunday => 0,
            WeekStart::Monday => 1,
        }
    }
}

/// Caption style (the source's `captionLayout` prop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaptionLayout {
    /// Centered "November 2026" label (the source default).
    #[default]
    Label,
    /// Two bordered dropdown buttons (month, year) opening [`DropdownMenu`]s,
    /// styled per the source's `dropdowns` / `dropdown_root` classes.
    Dropdown,
}

/// Pure single-mode selection: clicking the selected day deselects it
/// (react-day-picker's default when `required = false`). Returns whether the
/// selection changed.
pub fn apply_single(sel: &mut Option<CalendarDate>, day: CalendarDate) -> bool {
    if *sel == Some(day) {
        *sel = None;
    } else {
        *sel = Some(day);
    }
    true
}

/// Pure multiple-mode selection: toggles the day's membership (kept sorted).
pub fn apply_multiple(sel: &mut Vec<CalendarDate>, day: CalendarDate) -> bool {
    match sel.iter().position(|&d| d == day) {
        Some(pos) => {
            sel.remove(pos);
        }
        None => {
            sel.push(day);
            sel.sort();
        }
    }
    true
}

/// Pure range-mode selection: the first click starts a pending range, the
/// second completes it (clicking before the start restarts from that day),
/// the third click starts a new range.
pub fn apply_range(
    sel: &mut Option<(CalendarDate, CalendarDate)>,
    day: CalendarDate,
    pending: &mut bool,
) -> bool {
    match (*sel, *pending) {
        (Some((start, _)), true) => {
            if day >= start {
                *sel = Some((start, day));
                *pending = false;
            } else {
                // Restart the range from the earlier day.
                *sel = Some((day, day));
            }
        }
        _ => {
            *sel = Some((day, day));
            *pending = true;
        }
    }
    true
}

/// Storage for the selection, controlled by the caller.
#[derive(Debug)]
enum SelectionRef<'a> {
    Single(&'a mut Option<CalendarDate>),
    Multiple(&'a mut Vec<CalendarDate>),
    Range(&'a mut Option<(CalendarDate, CalendarDate)>),
}

/// Predicate deciding which days are disabled.
pub(crate) type DisabledFn<'a> = Box<dyn Fn(&CalendarDate) -> bool + 'a>;

/// Day cell visual role (mirrors the source's `data-range-*`, `data-today`…).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CellRole {
    Hover,
    Today,
    SelectedSingle,
    RangeStart,
    RangeEnd,
    RangeMiddle,
}

/// The day keyboard focus should land on for `initialFocus`: the selected
/// day (first for multiple, start for ranges) or today.
fn selection_day(sel: &SelectionRef<'_>) -> Option<CalendarDate> {
    match sel {
        SelectionRef::Single(opt) => **opt,
        SelectionRef::Multiple(v) => v.first().copied(),
        SelectionRef::Range(opt) => (**opt).map(|(start, _)| start),
    }
}

fn selection_role(sel: &SelectionRef<'_>, day: CalendarDate) -> Option<CellRole> {
    match sel {
        SelectionRef::Single(opt) => (**opt == Some(day)).then_some(CellRole::SelectedSingle),
        SelectionRef::Multiple(v) => v.contains(&day).then_some(CellRole::SelectedSingle),
        SelectionRef::Range(opt) => {
            let (start, end) = (**opt)?;
            if start == end {
                // Pending or one-day range: reads as a selected single day.
                (day == start).then_some(CellRole::SelectedSingle)
            } else if day == start {
                Some(CellRole::RangeStart)
            } else if day == end {
                Some(CellRole::RangeEnd)
            } else if start < day && day < end {
                Some(CellRole::RangeMiddle)
            } else {
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Widget
// ---------------------------------------------------------------------------

/// Per-instance state persisted in the egui context.
#[derive(Clone, Copy, Debug, Default)]
struct CalendarState {
    /// Currently displayed month as `(year, month)`; `None` until first shown.
    view_month: Option<(i32, u32)>,
    /// The day that keyboard navigation acts on.
    focused_day: Option<CalendarDate>,
    /// Range mode: a start has been clicked, awaiting the end.
    range_pending: bool,
    /// `initial_focus` was already applied once.
    initial_focus_done: bool,
    /// Caption dropdown layout: the month menu is open.
    month_menu_open: bool,
    /// Caption dropdown layout: the year menu is open.
    year_menu_open: bool,
}

/// What happened to a [`Calendar`] this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CalendarResponse {
    /// The selection was mutated (by click or keyboard) this frame.
    pub changed: bool,
    /// The day that was clicked (or keyboard-selected) this frame.
    pub clicked_day: Option<CalendarDate>,
    /// Range mode: the pending start got its end this frame, so the
    /// selection is a complete range. Used by the date-picker, whose range
    /// popover stays open until both ends are chosen.
    pub completed: bool,
}

/// The shadcn calendar. See the [module docs](self) for an example.
///
/// The selection is always controlled (`&mut`), matching react-day-picker's
/// `selected` + `onSelect` props; use the constructor matching the mode:
/// [`Calendar::single`], [`Calendar::multiple`] or [`Calendar::range`].
pub struct Calendar<'a> {
    selection: SelectionRef<'a>,
    id_salt: Option<String>,
    default_month: Option<CalendarDate>,
    show_outside_days: bool,
    week_start: WeekStart,
    caption_layout: CaptionLayout,
    show_week_numbers: bool,
    min_month: Option<CalendarDate>,
    max_month: Option<CalendarDate>,
    disabled: Option<DisabledFn<'a>>,
    initial_focus: bool,
}

impl<'a> Calendar<'a> {
    /// Creates a calendar in `mode="single"`, bound to the caller's date.
    pub fn single(selected: &'a mut Option<CalendarDate>) -> Self {
        Self {
            selection: SelectionRef::Single(selected),
            id_salt: None,
            default_month: None,
            show_outside_days: true,
            week_start: WeekStart::default(),
            caption_layout: CaptionLayout::default(),
            show_week_numbers: false,
            min_month: None,
            max_month: None,
            disabled: None,
            initial_focus: false,
        }
    }

    /// Creates a calendar in `mode="multiple"`, bound to the caller's set.
    pub fn multiple(selected: &'a mut Vec<CalendarDate>) -> Self {
        Self {
            selection: SelectionRef::Multiple(selected),
            id_salt: None,
            default_month: None,
            show_outside_days: true,
            week_start: WeekStart::default(),
            caption_layout: CaptionLayout::default(),
            show_week_numbers: false,
            min_month: None,
            max_month: None,
            disabled: None,
            initial_focus: false,
        }
    }

    /// Creates a calendar in `mode="range"`, bound to the caller's range.
    pub fn range(selected: &'a mut Option<(CalendarDate, CalendarDate)>) -> Self {
        Self {
            selection: SelectionRef::Range(selected),
            id_salt: None,
            default_month: None,
            show_outside_days: true,
            week_start: WeekStart::default(),
            caption_layout: CaptionLayout::default(),
            show_week_numbers: false,
            min_month: None,
            max_month: None,
            disabled: None,
            initial_focus: false,
        }
    }

    /// Sets a stable id salt so month/focus state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Month displayed before the user navigates (defaults to today's month).
    pub fn default_month(mut self, month: CalendarDate) -> Self {
        self.default_month = Some(month.first_of_month());
        self
    }

    /// Shows days from adjacent months in the grid edges (default `true`,
    /// the source's `showOutsideDays`).
    pub fn show_outside_days(mut self, show: bool) -> Self {
        self.show_outside_days = show;
        self
    }

    /// Sets the first weekday column (default [`WeekStart::Sunday`]).
    pub fn week_starts_on(mut self, start: WeekStart) -> Self {
        self.week_start = start;
        self
    }

    /// Sets the caption style (`captionLayout`, default
    /// [`CaptionLayout::Label`]). [`CaptionLayout::Dropdown`] renders two
    /// bordered month/year buttons that open menus.
    pub fn caption_layout(mut self, layout: CaptionLayout) -> Self {
        self.caption_layout = layout;
        self
    }

    /// Shows a leading column of ISO week numbers (`showWeekNumber`): a
    /// `w-(--cell-size)` column of `text-[0.8rem] text-muted-foreground`
    /// numbers, with the grid one cell wider.
    pub fn show_week_numbers(mut self, show: bool) -> Self {
        self.show_week_numbers = show;
        self
    }

    /// Earliest month the user can navigate to (`fromDate`): the previous
    /// button renders `aria-disabled:opacity-50` and does nothing beyond it.
    pub fn min_month(mut self, month: CalendarDate) -> Self {
        self.min_month = Some(month.first_of_month());
        self
    }

    /// Latest month the user can navigate to (`toDate`): the next button
    /// renders `aria-disabled:opacity-50` and does nothing beyond it.
    pub fn max_month(mut self, month: CalendarDate) -> Self {
        self.max_month = Some(month.first_of_month());
        self
    }

    /// Disables the days for which the predicate returns `true`
    /// (`disabled` matcher): rendered `text-muted-foreground opacity-50`
    /// and not selectable.
    pub fn disabled_when(mut self, f: impl Fn(&CalendarDate) -> bool + 'a) -> Self {
        self.disabled = Some(Box::new(f));
        self
    }

    /// Requests keyboard focus for the selected day (or today) on the first
    /// frame, like the source's `initialFocus`. Used by the date-picker.
    pub fn initial_focus(mut self, on: bool) -> Self {
        self.initial_focus = on;
        self
    }

    fn is_disabled(&self, day: &CalendarDate) -> bool {
        self.disabled.as_deref().is_some_and(|f| f(day))
    }

    /// Lays out and paints the calendar, handling clicks and keyboard
    /// navigation (arrows and Home/End move the focused day, Enter/Space
    /// select, PageUp/PageDown step months — Shift steps years).
    pub fn show(mut self, ui: &mut Ui) -> CalendarResponse {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let style = theme.style;
        let p = theme.palette;
        let radius_md = theme.radius.md_u8();
        let today = CalendarDate::today(ui);

        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_calendar"),
        };
        let ctx = ui.ctx().clone();
        let mut st: CalendarState = ctx
            .data_mut(|d| d.get_temp::<CalendarState>(state_id))
            .unwrap_or_default();

        let view_first = st
            .view_month
            .and_then(|(y, m)| CalendarDate::new(y, m, 1))
            .or(self.default_month)
            .unwrap_or_else(|| today.first_of_month());
        st.view_month = Some((view_first.year, view_first.month));

        // `initialFocus`: focus the selected day (or today) once, on first show.
        if self.initial_focus && !st.initial_focus_done {
            st.initial_focus_done = true;
            let target = selection_day(&self.selection).unwrap_or(today);
            ui.memory_mut(|m| m.request_focus(state_id.with((DAY_ID_KEY, target.epoch_days()))));
        }

        // Grid geometry: leading outside days + weeks of 7 (+ week numbers).
        let week_start = self.week_start.index();
        let leading = (view_first.weekday() + 7 - week_start) % 7;
        let dim = CalendarDate::days_in_month(view_first.year, view_first.month);
        let rows = (leading + dim).div_ceil(7) as i32;
        let grid_start = view_first.epoch_days() - i64::from(leading);

        let cell = CALENDAR_CELL_SIZE;
        let week_num_w = if self.show_week_numbers { cell } else { 0.0 };
        let grid_w = CALENDAR_GRID_WIDTH + week_num_w;
        // `week: mt-2` — an 8 px gap above every week row (including the
        // first, below the weekday header).
        let content_h =
            CAPTION_HEIGHT + CAPTION_GAP + WEEKDAY_ROW_HEIGHT + rows as f32 * (cell + WEEK_GAP);
        let total = Vec2::new(
            grid_w + 2.0 * CALENDAR_PADDING,
            content_h + 2.0 * CALENDAR_PADDING,
        );
        let (_, container) = ui.allocate_space(total);

        let caption_rect = Rect::from_min_size(
            container.min + Vec2::splat(CALENDAR_PADDING),
            Vec2::new(grid_w, CAPTION_HEIGHT),
        );

        // --- Nav buttons (ghost, size = cell) + caption ----------------------
        let prev_rect = Rect::from_min_size(caption_rect.min, Vec2::splat(cell));
        let next_rect = Rect::from_min_size(
            Pos2::new(caption_rect.max.x - cell, caption_rect.min.y),
            Vec2::splat(cell),
        );
        let prev_enabled = self
            .min_month
            .is_none_or(|min| view_first.prev_month() >= min);
        let next_enabled = self
            .max_month
            .is_none_or(|max| view_first.next_month() <= max);
        let prev = ghost_icon_button(
            ui,
            prev_rect,
            state_id.with("nav_prev"),
            Icon::ChevronLeft,
            prev_enabled,
            &p,
            dark,
            style,
            radius_md,
        );
        let next = ghost_icon_button(
            ui,
            next_rect,
            state_id.with("nav_next"),
            Icon::ChevronRight,
            next_enabled,
            &p,
            dark,
            style,
            radius_md,
        );
        if prev.clicked() && prev_enabled {
            let pm = view_first.prev_month();
            st.view_month = Some((pm.year, pm.month));
        }
        if next.clicked() && next_enabled {
            let nm = view_first.next_month();
            st.view_month = Some((nm.year, nm.month));
        }

        match self.caption_layout {
            CaptionLayout::Label => {
                let caption = format!("{} {}", view_first.month_name(), view_first.year);
                ui.painter().text(
                    caption_rect.center(),
                    Align2::CENTER_CENTER,
                    caption,
                    FontId::proportional(crate::theme::tokens::type_scale::SM),
                    p.foreground,
                );
            }
            CaptionLayout::Dropdown => {
                Self::show_caption_dropdowns(
                    ui,
                    &mut st,
                    state_id,
                    caption_rect,
                    view_first,
                    &p,
                    dark,
                    radius_md,
                );
            }
        }

        // --- Weekday header -------------------------------------------------
        let weekday_y = caption_rect.bottom() + CAPTION_GAP;
        for col in 0..7i32 {
            let rect = Rect::from_min_size(
                Pos2::new(
                    container.min.x + CALENDAR_PADDING + week_num_w + col as f32 * cell,
                    weekday_y,
                ),
                Vec2::new(cell, WEEKDAY_ROW_HEIGHT),
            );
            let label = WEEKDAY_SHORT[((week_start + col as u32) % 7) as usize];
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                label,
                FontId::proportional(WEEKDAY_TEXT_SIZE),
                p.muted_foreground,
            );
        }

        // --- Day cells ------------------------------------------------------
        let grid_origin = Pos2::new(
            container.min.x + CALENDAR_PADDING,
            weekday_y + WEEKDAY_ROW_HEIGHT + WEEK_GAP,
        );
        let mut response = CalendarResponse::default();
        let mut any_day_focused = false;
        let is_range = matches!(self.selection, SelectionRef::Range(_));

        for row in 0..rows {
            let row_y = grid_origin.y + row as f32 * (cell + WEEK_GAP);
            if self.show_week_numbers {
                // `week_number_header` (`w-(--cell-size)`) + `week_number`
                // (`text-[0.8rem] text-muted-foreground`), numbered from the
                // row's first displayed day.
                let day = CalendarDate::from_epoch_days(grid_start + i64::from(row * 7));
                let week_rect = Rect::from_min_size(
                    Pos2::new(grid_origin.x, row_y),
                    Vec2::new(week_num_w, cell),
                );
                ui.painter().text(
                    week_rect.center(),
                    Align2::CENTER_CENTER,
                    iso_week_number(day).to_string(),
                    FontId::proportional(WEEKDAY_TEXT_SIZE),
                    p.muted_foreground,
                );
            }
            for col in 0..7i32 {
                let day = CalendarDate::from_epoch_days(grid_start + i64::from(row * 7 + col));
                let rect = Rect::from_min_size(
                    Pos2::new(grid_origin.x + week_num_w + col as f32 * cell, row_y),
                    Vec2::splat(cell),
                );
                if !ui.is_rect_visible(rect) {
                    continue;
                }
                let id = state_id.with((DAY_ID_KEY, day.epoch_days()));
                let cell_response = ui.interact(rect, id, Sense::click());

                let disabled = self.is_disabled(&day);
                if cell_response.clicked() && !disabled {
                    let was_pending = st.range_pending;
                    let mutated = apply_selection(&mut self.selection, day, &mut st.range_pending);
                    response.changed |= mutated;
                    response.clicked_day = Some(day);
                    response.completed |= is_range && was_pending && !st.range_pending;
                }

                let selected_role = selection_role(&self.selection, day);
                let outside = day.month != view_first.month || day.year != view_first.year;
                let role = selected_role.or_else(|| {
                    if disabled {
                        None
                    } else if cell_response.hovered() {
                        Some(CellRole::Hover)
                    } else if day == today {
                        Some(CellRole::Today)
                    } else {
                        None
                    }
                });

                paint_day_cell(
                    ui.painter(),
                    rect,
                    day,
                    role,
                    outside,
                    disabled,
                    &p,
                    radius_md,
                );
                if cell_response.has_focus() {
                    any_day_focused = true;
                    st.focused_day = Some(day);
                    if !disabled {
                        focus::paint_ring(
                            ui.painter(),
                            rect,
                            color_alpha(p.ring, 0.5),
                            3.0,
                            radius_md as f32,
                        );
                    }
                }
            }
        }

        // --- Keyboard navigation (only while a day cell holds focus) --------
        if any_day_focused {
            if let Some(focused_day) = st.focused_day {
                let (day_step, month_step, select) = ui.input(|i| {
                    let back = days_since_week_start(focused_day, self.week_start);
                    let day_step = if i.key_pressed(Key::ArrowLeft) {
                        Some(-1i64)
                    } else if i.key_pressed(Key::ArrowRight) {
                        Some(1)
                    } else if i.key_pressed(Key::ArrowUp) {
                        Some(-7)
                    } else if i.key_pressed(Key::ArrowDown) {
                        Some(7)
                    } else if i.key_pressed(Key::Home) {
                        // Home = first day of the focused week.
                        Some(-back)
                    } else if i.key_pressed(Key::End) {
                        // End = last day of the focused week.
                        Some(6 - back)
                    } else {
                        None
                    };
                    let month_step = if i.key_pressed(Key::PageUp) {
                        Some(-1i32)
                    } else if i.key_pressed(Key::PageDown) {
                        Some(1)
                    } else {
                        None
                    };
                    let shift = i.modifiers.shift;
                    let select = i.key_pressed(Key::Enter) || i.key_pressed(Key::Space);
                    (day_step, month_step.map(|s| (s, shift)), select)
                });

                let mut new_day: Option<CalendarDate> = None;
                if let Some(delta) = day_step {
                    new_day = Some(CalendarDate::from_epoch_days(
                        focused_day.epoch_days() + delta,
                    ));
                } else if let Some((step, shift)) = month_step {
                    new_day = if shift {
                        // Shift+PageUp/Down steps a whole year.
                        CalendarDate::new(focused_day.year + step, focused_day.month, 1)
                            .map(|d| d.with_day_clamped(focused_day.day))
                    } else {
                        let month = if step < 0 {
                            focused_day.prev_month()
                        } else {
                            focused_day.next_month()
                        };
                        Some(month.with_day_clamped(focused_day.day))
                    };
                } else if select && !self.is_disabled(&focused_day) {
                    let was_pending = st.range_pending;
                    if apply_selection(&mut self.selection, focused_day, &mut st.range_pending) {
                        response.changed = true;
                        response.clicked_day = Some(focused_day);
                        response.completed |= is_range && was_pending && !st.range_pending;
                    }
                }

                if let Some(nd) = new_day {
                    if nd.month != view_first.month || nd.year != view_first.year {
                        st.view_month = Some((nd.year, nd.month));
                    }
                    st.focused_day = Some(nd);
                    ui.memory_mut(|m| {
                        m.request_focus(state_id.with((DAY_ID_KEY, nd.epoch_days())))
                    });
                }
            }
        }

        ctx.data_mut(|d| d.insert_temp(state_id, st));
        response
    }

    /// `captionLayout="dropdown"`: two `dropdown_root`-styled buttons (month,
    /// year) that open [`DropdownMenu`]s of months / `year ± 12`.
    #[allow(clippy::too_many_arguments)]
    fn show_caption_dropdowns(
        ui: &mut Ui,
        st: &mut CalendarState,
        state_id: Id,
        caption_rect: Rect,
        view_first: CalendarDate,
        p: &crate::theme::Palette,
        dark: bool,
        radius_md: u8,
    ) {
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let month_label = MONTH_SHORT[(view_first.month.clamp(1, 12) - 1) as usize];
        let year_label = view_first.year.to_string();
        let text_w = |label: &str| {
            ui.painter()
                .layout_no_wrap(label.to_owned(), font.clone(), p.foreground)
                .size()
                .x
        };
        let button_w = |label: &str| {
            CAPTION_DROPDOWN_PADDING_LEFT
                + text_w(label)
                + CAPTION_DROPDOWN_GAP
                + NAV_ICON_SIZE
                + CAPTION_DROPDOWN_PADDING_RIGHT
        };
        let month_w = button_w(month_label);
        let year_w = button_w(&year_label);
        let pair_w = month_w + CAPTION_DROPDOWNS_GAP + year_w;
        let start_x = caption_rect.center().x - pair_w * 0.5;
        let month_rect = Rect::from_min_size(
            Pos2::new(start_x, caption_rect.min.y),
            Vec2::new(month_w, caption_rect.height()),
        );
        let year_rect = Rect::from_min_size(
            Pos2::new(
                start_x + month_w + CAPTION_DROPDOWNS_GAP,
                caption_rect.min.y,
            ),
            Vec2::new(year_w, caption_rect.height()),
        );

        let month_resp = dropdown_root_button(
            ui,
            month_rect,
            state_id.with("caption_month"),
            month_label,
            st.month_menu_open,
            p,
            dark,
            radius_md,
        );
        let year_resp = dropdown_root_button(
            ui,
            year_rect,
            state_id.with("caption_year"),
            &year_label,
            st.year_menu_open,
            p,
            dark,
            radius_md,
        );

        // Month menu (ids are the item order; the response's `path` carries it).
        let menu_salt = format!("{}_caption_menus", state_id.value());
        let month_menu = DropdownMenu::controlled("", &mut st.month_menu_open)
            .trigger_widget(move |_| month_resp.clone())
            .align(MenuAlign::Start)
            .width(MONTH_MENU_WIDTH)
            .id_salt(format!("{menu_salt}_month"));
        let month_result = month_menu.show(ui, |menu| {
            for name in MONTH_NAMES {
                menu.item(MenuItem::new(name));
            }
        });
        if let Some(&idx) = month_result.activated.as_ref().and_then(|a| a.path.first()) {
            st.view_month = Some((view_first.year, idx as u32 + 1));
        }

        // Year menu: `view year ± 12`.
        let years: Vec<String> = (view_first.year - YEAR_DROPDOWN_RANGE
            ..=view_first.year + YEAR_DROPDOWN_RANGE)
            .map(|y| y.to_string())
            .collect();
        let year_menu = DropdownMenu::controlled("", &mut st.year_menu_open)
            .trigger_widget(move |_| year_resp.clone())
            .align(MenuAlign::Start)
            .width(YEAR_MENU_WIDTH)
            .id_salt(format!("{menu_salt}_year"));
        let year_result = year_menu.show(ui, |menu| {
            for year in &years {
                menu.item(MenuItem::new(year.clone()).id(year.clone()));
            }
        });
        if let Some(a) = year_result.activated {
            if let Ok(y) = a.id.parse::<i32>() {
                st.view_month = Some((y, view_first.month));
            }
        }
    }
}

fn apply_selection(sel: &mut SelectionRef<'_>, day: CalendarDate, pending: &mut bool) -> bool {
    match sel {
        SelectionRef::Single(opt) => apply_single(opt, day),
        SelectionRef::Multiple(v) => apply_multiple(v, day),
        SelectionRef::Range(opt) => apply_range(opt, day, pending),
    }
}

// ---------------------------------------------------------------------------
// Painting helpers
// ---------------------------------------------------------------------------

/// Ghost hover fill, mirroring the crate's ghost button recipe
/// (`hover:bg-accent`, `dark:hover:bg-accent/50`; nova `hover:bg-muted
/// dark:hover:bg-muted/50`).
fn ghost_hover_fill(p: &crate::theme::Palette, dark: bool, style: Style) -> Color32 {
    if style == Style::Nova {
        if dark {
            color_alpha(p.muted, 0.5)
        } else {
            p.muted
        }
    } else if dark {
        color_alpha(p.accent, 0.5)
    } else {
        p.accent
    }
}

/// A 28 px ghost icon button (the source nav buttons: `buttonVariants({
/// variant: "ghost" })` + `size-(--cell-size) p-0 aria-disabled:opacity-50`).
/// Colors follow the crate's ghost button; out-of-bounds targets render
/// dimmed and inert (`aria-disabled`).
#[allow(clippy::too_many_arguments)]
fn ghost_icon_button(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    icon: Icon,
    enabled: bool,
    p: &crate::theme::Palette,
    dark: bool,
    style: Style,
    radius: u8,
) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let hovered = response.hovered() && enabled;
        let fg = if enabled {
            if hovered && style != Style::Nova {
                p.accent_foreground
            } else {
                p.foreground
            }
        } else {
            color_alpha(p.foreground, 0.5)
        };
        if hovered {
            painter.rect_filled(
                rect,
                CornerRadius::same(radius),
                ghost_hover_fill(p, dark, style),
            );
        }
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(NAV_ICON_SIZE));
        icon.paint(painter, icon_rect, fg, 2.0 * NAV_ICON_SIZE / 24.0);
    }
    response
}

/// A caption `dropdown_root` button: `rounded-md border border-input shadow-xs`,
/// `has-focus:border-ring has-focus:ring-[3px] has-focus:ring-ring/50`, with
/// the caption label (`text-sm`, `pl-1.5 pr-1`) and a muted `size-4` chevron
/// (`caption_label` in dropdown layout).
#[allow(clippy::too_many_arguments)]
fn dropdown_root_button(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    label: &str,
    open: bool,
    p: &crate::theme::Palette,
    dark: bool,
    radius: u8,
) -> Response {
    let response = ui.interact(rect, id, Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let focused = open || response.has_focus();
        let sh = crate::utils::shadow::shadow_xs(dark);
        painter.add(
            egui::epaint::RectShape::filled(
                rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                CornerRadius::same(radius),
                sh.color,
            )
            .with_blur_width(2.0),
        );
        painter.rect_filled(rect, CornerRadius::same(radius), p.background);
        let stroke = if focused { p.ring } else { p.input };
        painter.rect_stroke(
            rect,
            CornerRadius::same(radius),
            Stroke::new(1.0, stroke),
            egui::epaint::StrokeKind::Inside,
        );
        if focused {
            focus::paint_ring(painter, rect, color_alpha(p.ring, 0.5), 3.0, radius as f32);
        }
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let text_pos = Pos2::new(rect.min.x + CAPTION_DROPDOWN_PADDING_LEFT, rect.center().y);
        painter.text(text_pos, Align2::LEFT_CENTER, label, font, p.foreground);
        let icon_rect = Rect::from_center_size(
            Pos2::new(
                rect.max.x - CAPTION_DROPDOWN_PADDING_RIGHT - NAV_ICON_SIZE / 2.0,
                rect.center().y,
            ),
            Vec2::splat(NAV_ICON_SIZE),
        );
        Icon::ChevronDown.paint(
            painter,
            icon_rect,
            p.muted_foreground,
            2.0 * NAV_ICON_SIZE / 24.0,
        );
    }
    response
}

/// Paints one day cell per the source's `CalendarDayButton` styles.
#[allow(clippy::too_many_arguments)]
fn paint_day_cell(
    painter: &Painter,
    rect: Rect,
    day: CalendarDate,
    role: Option<CellRole>,
    outside: bool,
    disabled: bool,
    p: &crate::theme::Palette,
    radius_md: u8,
) {
    let mut bg: Option<(Color32, CornerRadius)> = None;
    let mut text = if outside || disabled {
        p.muted_foreground
    } else {
        p.foreground
    };
    let r = radius_md;

    match role {
        Some(CellRole::RangeStart) => {
            // The accent range bar runs behind the primary end square and
            // shows through its rounded inner corners (`range_start` td +
            // `data-[range-start=true]` button).
            painter.rect_filled(
                rect,
                CornerRadius {
                    nw: r,
                    sw: r,
                    ..CornerRadius::ZERO
                },
                p.accent,
            );
            bg = Some((p.primary, CornerRadius::same(r)));
            text = p.primary_foreground;
        }
        Some(CellRole::RangeEnd) => {
            painter.rect_filled(
                rect,
                CornerRadius {
                    ne: r,
                    se: r,
                    ..CornerRadius::ZERO
                },
                p.accent,
            );
            bg = Some((p.primary, CornerRadius::same(r)));
            text = p.primary_foreground;
        }
        Some(CellRole::RangeMiddle) => {
            // `data-[range-middle=true]:bg-accent rounded-none`
            bg = Some((p.accent, CornerRadius::same(0)));
            text = p.accent_foreground;
        }
        Some(CellRole::SelectedSingle) => {
            // `data-[selected-single=true]:bg-primary`
            bg = Some((p.primary, CornerRadius::same(r)));
            text = p.primary_foreground;
        }
        Some(CellRole::Today) => {
            // `today: rounded-md bg-accent text-accent-foreground`
            bg = Some((p.accent, CornerRadius::same(r)));
            text = p.accent_foreground;
        }
        Some(CellRole::Hover) => {
            // `hover:bg-accent hover:text-accent-foreground` (the dark-only
            // override coincides with zinc light values).
            bg = Some((p.accent, CornerRadius::same(r)));
            text = p.accent_foreground;
        }
        None => {}
    }

    if disabled {
        // `disabled: text-muted-foreground opacity-50`, no hover fill.
        text = color_alpha(p.muted_foreground, 0.5);
        bg = None;
    }

    if let Some((color, corner)) = bg {
        painter.rect_filled(rect, corner, color);
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        day.day.to_string(),
        FontId::proportional(DAY_TEXT_SIZE),
        text,
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Rect as ERect;

    // --- Date math ----------------------------------------------------------

    #[test]
    fn days_in_month_matches_calendar() {
        assert_eq!(CalendarDate::days_in_month(2024, 2), 29);
        assert_eq!(CalendarDate::days_in_month(2023, 2), 28);
        assert_eq!(CalendarDate::days_in_month(2000, 2), 29);
        assert_eq!(CalendarDate::days_in_month(1900, 2), 28);
        assert_eq!(CalendarDate::days_in_month(2026, 11), 30);
        assert_eq!(CalendarDate::days_in_month(2026, 1), 31);
        assert_eq!(CalendarDate::days_in_month(2026, 4), 30);
        assert_eq!(CalendarDate::days_in_month(2026, 13), 0);
    }

    #[test]
    fn leap_year_rule() {
        assert!(CalendarDate::is_leap_year(2000));
        assert!(CalendarDate::is_leap_year(2024));
        assert!(!CalendarDate::is_leap_year(1900));
        assert!(!CalendarDate::is_leap_year(2023));
        assert!(CalendarDate::is_leap_year(2400));
    }

    #[test]
    fn new_validates() {
        assert!(CalendarDate::new(2026, 2, 30).is_none());
        assert!(CalendarDate::new(2026, 13, 1).is_none());
        assert!(CalendarDate::new(2026, 0, 1).is_none());
        assert!(CalendarDate::new(2026, 11, 0).is_none());
        assert!(CalendarDate::new(2026, 11, 31).is_none());
        assert!(CalendarDate::new(2024, 2, 29).is_some());
        assert!(CalendarDate::new(2026, 11, 10).is_some());
    }

    #[test]
    fn epoch_round_trip_across_centuries() {
        let samples = [
            CalendarDate::new(1970, 1, 1).unwrap(),
            CalendarDate::new(1969, 12, 31).unwrap(),
            CalendarDate::new(1900, 6, 15).unwrap(),
            CalendarDate::new(2000, 2, 29).unwrap(),
            CalendarDate::new(2026, 11, 10).unwrap(),
            CalendarDate::new(2100, 3, 1).unwrap(),
        ];
        for d in samples {
            assert_eq!(CalendarDate::from_epoch_days(d.epoch_days()), d);
        }
    }

    #[test]
    fn epoch_days_known_values() {
        // 1970-01-01 is day 0.
        assert_eq!(CalendarDate::new(1970, 1, 1).unwrap().epoch_days(), 0);
        assert_eq!(CalendarDate::new(1969, 12, 31).unwrap().epoch_days(), -1);
        assert_eq!(CalendarDate::new(1970, 1, 2).unwrap().epoch_days(), 1);
        // 2026-11-10 = 56 years + leap days after epoch.
        assert_eq!(
            CalendarDate::new(2026, 11, 10).unwrap().epoch_days(),
            20_767
        );
    }

    #[test]
    fn weekday_known_dates() {
        // 0 = Sunday.
        assert_eq!(CalendarDate::new(1970, 1, 1).unwrap().weekday(), 4); // Thursday
        assert_eq!(CalendarDate::new(2026, 11, 10).unwrap().weekday(), 2); // Tuesday
        assert_eq!(CalendarDate::new(2026, 11, 15).unwrap().weekday(), 0); // Sunday
        assert_eq!(CalendarDate::new(2000, 1, 1).unwrap().weekday(), 6); // Saturday
        assert_eq!(CalendarDate::new(2024, 2, 29).unwrap().weekday(), 4); // Thursday
    }

    #[test]
    fn month_navigation_rollovers() {
        let dec = CalendarDate::new(2026, 12, 15).unwrap();
        assert_eq!(dec.next_month(), CalendarDate::new(2027, 1, 1).unwrap());
        assert_eq!(dec.prev_month(), CalendarDate::new(2026, 11, 1).unwrap());
        let jan = CalendarDate::new(2026, 1, 3).unwrap();
        assert_eq!(jan.prev_month(), CalendarDate::new(2025, 12, 1).unwrap());
        assert_eq!(jan.next_month(), CalendarDate::new(2026, 2, 1).unwrap());
        // Leap February navigation.
        let feb = CalendarDate::new(2024, 2, 29).unwrap();
        assert_eq!(feb.next_month(), CalendarDate::new(2024, 3, 1).unwrap());
        assert_eq!(feb.prev_month(), CalendarDate::new(2024, 1, 1).unwrap());
    }

    #[test]
    fn day_clamping() {
        let jan31 = CalendarDate::new(2026, 1, 31).unwrap();
        assert_eq!(
            jan31.next_month().with_day_clamped(31),
            CalendarDate::new(2026, 2, 28).unwrap()
        );
        assert_eq!(
            CalendarDate::new(2024, 1, 31)
                .unwrap()
                .next_month()
                .with_day_clamped(31),
            CalendarDate::new(2024, 2, 29).unwrap()
        );
        assert_eq!(
            jan31.prev_month().with_day_clamped(31),
            CalendarDate::new(2025, 12, 31).unwrap()
        );
    }

    #[test]
    fn unix_secs_to_date() {
        let d = CalendarDate::new(2026, 11, 10).unwrap();
        assert_eq!(
            CalendarDate::from_unix_secs(d.epoch_days() as f64 * 86_400.0),
            d
        );
        assert_eq!(
            CalendarDate::from_unix_secs(0.0),
            CalendarDate::new(1970, 1, 1).unwrap()
        );
        // Just before UTC midnight stays on the previous day.
        assert_eq!(
            CalendarDate::from_unix_secs(d.epoch_days() as f64 * 86_400.0 - 0.5),
            d.yesterday()
        );
    }

    #[test]
    fn ordering_is_chronological() {
        let a = CalendarDate::new(2026, 11, 9).unwrap();
        let b = CalendarDate::new(2026, 11, 10).unwrap();
        let c = CalendarDate::new(2026, 12, 1).unwrap();
        let d = CalendarDate::new(2027, 1, 1).unwrap();
        assert!(a < b && b < c && c < d);
    }

    #[test]
    fn adjacent_day_helpers() {
        let d = CalendarDate::new(2026, 11, 10).unwrap();
        assert_eq!(d.tomorrow(), CalendarDate::new(2026, 11, 11).unwrap());
        assert_eq!(d.yesterday(), CalendarDate::new(2026, 11, 9).unwrap());
        let eom = CalendarDate::new(2026, 11, 30).unwrap();
        assert_eq!(eom.tomorrow(), CalendarDate::new(2026, 12, 1).unwrap());
    }

    #[test]
    fn month_names_and_formatting() {
        let d = CalendarDate::new(2026, 11, 10).unwrap();
        assert_eq!(d.month_name(), "November");
        assert_eq!(d.format_ppp(), "November 10th, 2026");
        assert_eq!(
            CalendarDate::new(2026, 1, 1).unwrap().format_ppp(),
            "January 1st, 2026"
        );
        assert_eq!(
            CalendarDate::new(2026, 2, 22).unwrap().format_ppp(),
            "February 22nd, 2026"
        );
        assert_eq!(
            CalendarDate::new(2026, 3, 13).unwrap().format_ppp(),
            "March 13th, 2026"
        );
        assert_eq!(
            CalendarDate::new(2026, 7, 31).unwrap().format_ppp(),
            "July 31st, 2026"
        );
    }

    #[test]
    fn grid_geometry_starts_on_week_start() {
        // The first grid cell must fall on the configured week start, and the
        // grid must cover the whole month.
        for (y, m) in [
            (2026u32, 11u32),
            (2024, 2),
            (2027, 2),
            (2026, 1),
            (2000, 12),
        ] {
            for &ws in &[WeekStart::Sunday, WeekStart::Monday] {
                let first = CalendarDate::new(y as i32, m, 1).unwrap();
                let leading = (first.weekday() + 7 - ws.index()) % 7;
                let dim = CalendarDate::days_in_month(y as i32, m);
                let rows = (leading + dim).div_ceil(7);
                let grid_start = first.epoch_days() - i64::from(leading);
                let first_cell = CalendarDate::from_epoch_days(grid_start);
                assert_eq!(first_cell.weekday(), ws.index(), "{y}-{m} ws={ws:?}");
                let last_cell = CalendarDate::from_epoch_days(grid_start + i64::from(rows * 7) - 1);
                let last_of_month =
                    CalendarDate::from_epoch_days(first.epoch_days() + i64::from(dim) - 1);
                assert!(last_cell >= last_of_month, "{y}-{m}");
                assert!(rows <= 6, "{y}-{m} never needs more than 6 weeks");
            }
        }
    }

    // --- Selection transitions ----------------------------------------------

    #[test]
    fn single_mode_toggles() {
        let d = CalendarDate::new(2026, 11, 10).unwrap();
        let mut sel = None;
        assert!(apply_single(&mut sel, d));
        assert_eq!(sel, Some(d));
        assert!(apply_single(&mut sel, d));
        assert_eq!(sel, None);
        let e = CalendarDate::new(2026, 11, 11).unwrap();
        apply_single(&mut sel, e);
        assert_eq!(sel, Some(e));
    }

    #[test]
    fn multiple_mode_adds_removes_sorts() {
        let a = CalendarDate::new(2026, 11, 5).unwrap();
        let b = CalendarDate::new(2026, 11, 1).unwrap();
        let mut sel: Vec<CalendarDate> = Vec::new();
        apply_multiple(&mut sel, a);
        apply_multiple(&mut sel, b);
        assert_eq!(sel, vec![b, a]); // sorted
        apply_multiple(&mut sel, a);
        assert_eq!(sel, vec![b]);
    }

    #[test]
    fn range_mode_transitions() {
        let a = CalendarDate::new(2026, 11, 5).unwrap();
        let b = CalendarDate::new(2026, 11, 10).unwrap();
        let c = CalendarDate::new(2026, 11, 2).unwrap();
        let mut sel = None;
        let mut pending = false;

        // First click: pending one-day range.
        apply_range(&mut sel, a, &mut pending);
        assert_eq!(sel, Some((a, a)));
        assert!(pending);

        // Second click after the start: completes the range.
        apply_range(&mut sel, b, &mut pending);
        assert_eq!(sel, Some((a, b)));
        assert!(!pending);

        // Third click: starts a new pending range.
        apply_range(&mut sel, b, &mut pending);
        assert_eq!(sel, Some((b, b)));
        assert!(pending);

        // Click before the start restarts from that day, still pending.
        apply_range(&mut sel, c, &mut pending);
        assert_eq!(sel, Some((c, c)));
        assert!(pending);

        // Completing again.
        apply_range(&mut sel, a, &mut pending);
        assert_eq!(sel, Some((c, a)));
        assert!(!pending);
    }

    #[test]
    fn selection_role_for_ranges() {
        let a = CalendarDate::new(2026, 11, 5).unwrap();
        let b = CalendarDate::new(2026, 11, 10).unwrap();
        let mid = CalendarDate::new(2026, 11, 7).unwrap();
        let out = CalendarDate::new(2026, 11, 20).unwrap();
        let sel = Some((a, b));
        assert_eq!(range_role_of(sel, a), Some(CellRole::RangeStart));
        assert_eq!(range_role_of(sel, b), Some(CellRole::RangeEnd));
        assert_eq!(range_role_of(sel, mid), Some(CellRole::RangeMiddle));
        assert_eq!(range_role_of(sel, out), None);
        // Pending one-day range reads as a selected single day.
        assert_eq!(
            range_role_of(Some((a, a)), a),
            Some(CellRole::SelectedSingle)
        );
        assert_eq!(range_role_of(Some((a, a)), b), None);
    }

    fn range_role_of(
        sel: Option<(CalendarDate, CalendarDate)>,
        day: CalendarDate,
    ) -> Option<CellRole> {
        let mut storage = sel;
        selection_role(&SelectionRef::Range(&mut storage), day)
    }

    // --- Rendering smoke test ---------------------------------------------

    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            Self { ctx, time: 0.0 }
        }

        fn run(&mut self, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("calendar_test_ui"),
                    egui::UiBuilder::new().max_rect(ERect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(600.0, 900.0),
                    )),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn renders_all_modes_without_panic() {
        let mut h = Harness::new();
        let mut single = Some(CalendarDate::new(2026, 11, 10).unwrap());
        let mut multi: Vec<CalendarDate> = Vec::new();
        let mut range = Some((
            CalendarDate::new(2026, 11, 5).unwrap(),
            CalendarDate::new(2026, 11, 20).unwrap(),
        ));
        for _ in 0..3 {
            h.run(|ui| {
                Calendar::single(&mut single).show(ui);
                Calendar::multiple(&mut multi)
                    .week_starts_on(WeekStart::Monday)
                    .show(ui);
                Calendar::range(&mut range).show(ui);
            });
        }
    }

    #[test]
    fn renders_new_caption_and_week_number_features_without_panic() {
        let mut h = Harness::new();
        let mut single = Some(CalendarDate::new(2026, 11, 10).unwrap());
        for _ in 0..3 {
            h.run(|ui| {
                Calendar::single(&mut single)
                    .id_salt("features")
                    .caption_layout(CaptionLayout::Dropdown)
                    .show_week_numbers(true)
                    .week_starts_on(WeekStart::Monday)
                    .min_month(CalendarDate::new(2026, 10, 1).unwrap())
                    .max_month(CalendarDate::new(2027, 2, 1).unwrap())
                    .show(ui);
            });
        }
    }

    #[test]
    fn iso_week_numbers_match_known_dates() {
        let d = |y, m, dd| CalendarDate::new(y, m, dd).unwrap();
        // ISO week 1 contains January 4.
        assert_eq!(iso_week_number(d(2026, 1, 1)), 1); // Thursday
        assert_eq!(iso_week_number(d(2025, 12, 29)), 1); // Monday of week 1 2026
        assert_eq!(iso_week_number(d(2025, 12, 28)), 52); // Sunday before
        assert_eq!(iso_week_number(d(2026, 11, 9)), 46); // Monday
        assert_eq!(iso_week_number(d(2026, 11, 10)), 46); // Tuesday
        assert_eq!(iso_week_number(d(2026, 11, 15)), 46); // Sunday
                                                          // 2027-01-01 (Friday) belongs to ISO week 53 of 2026.
        assert_eq!(iso_week_number(d(2027, 1, 1)), 53);
        // Long years: 2020 has 53 ISO weeks.
        assert_eq!(iso_week_number(d(2020, 12, 31)), 53);
        assert_eq!(iso_week_number(d(2021, 1, 1)), 53); // Friday, week 53 of 2020
        assert_eq!(iso_week_number(d(2019, 12, 30)), 1); // Monday of week 1 2020
    }

    #[test]
    fn home_end_offsets_find_week_bounds() {
        let ws = |y, m, dd| CalendarDate::new(y, m, dd).unwrap();
        // Tuesday 2026-11-10 with a Sunday week start: Home = Sun 8, End = Sat 14.
        let tue = ws(2026, 11, 10);
        let back = days_since_week_start(tue, WeekStart::Sunday);
        assert_eq!(back, 2);
        assert_eq!(
            CalendarDate::from_epoch_days(tue.epoch_days() - back),
            ws(2026, 11, 8)
        );
        assert_eq!(
            CalendarDate::from_epoch_days(tue.epoch_days() + 6 - back),
            ws(2026, 11, 14)
        );
        // Monday week start: Home = Mon 9.
        assert_eq!(days_since_week_start(tue, WeekStart::Monday), 1);
        // A week-start day is already Home.
        assert_eq!(days_since_week_start(ws(2026, 11, 8), WeekStart::Sunday), 0);
        assert_eq!(days_since_week_start(ws(2026, 11, 9), WeekStart::Monday), 0);
    }

    #[test]
    fn builders_keep_fields() {
        let mut single = None;
        let mut multi = Vec::new();
        let mut range = None;
        let c = Calendar::single(&mut single)
            .id_salt("c1")
            .default_month(CalendarDate::new(2026, 11, 1).unwrap())
            .show_outside_days(false)
            .week_starts_on(WeekStart::Monday)
            .caption_layout(CaptionLayout::Dropdown)
            .show_week_numbers(true)
            .min_month(CalendarDate::new(2026, 10, 1).unwrap())
            .max_month(CalendarDate::new(2027, 2, 1).unwrap());
        assert_eq!(c.id_salt.as_deref(), Some("c1"));
        assert!(!c.show_outside_days);
        assert_eq!(c.week_start, WeekStart::Monday);
        assert_eq!(c.caption_layout, CaptionLayout::Dropdown);
        assert!(c.show_week_numbers);
        assert_eq!(c.min_month, Some(CalendarDate::new(2026, 10, 1).unwrap()));
        assert_eq!(c.max_month, Some(CalendarDate::new(2027, 2, 1).unwrap()));
        assert_eq!(
            c.default_month,
            Some(CalendarDate::new(2026, 11, 1).unwrap())
        );
        let c2 = Calendar::multiple(&mut multi);
        assert!(matches!(c2.selection, SelectionRef::Multiple(_)));
        let c3 = Calendar::range(&mut range);
        assert!(matches!(c3.selection, SelectionRef::Range(_)));
    }

    #[test]
    fn geometry_constants_match_nova() {
        // .cn-calendar: p-2 (8), --cell-size:--spacing(7) (28), 7 columns,
        // `week: mt-2` = 8 px above every week row.
        assert_eq!(CALENDAR_CELL_SIZE, 28.0);
        assert_eq!(CALENDAR_PADDING, 8.0);
        assert_eq!(CALENDAR_GRID_WIDTH, 196.0);
        assert_eq!(WEEK_GAP, 8.0);
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
