//! shadcn `date-picker` — a docs composition of popover + calendar.
//!
//! There is no dedicated `ui/date-picker.tsx` in the source; this port
//! follows `apps/v4/registry/new-york-v4/examples/date-picker-demo.tsx`:
//! an outline trigger button (`w-[240px]`, calendar icon, the formatted
//! `PPP` date or the muted "Pick a date" placeholder) opening a
//! `w-auto p-0` popover anchored `align="start"` containing a single-mode
//! [`Calendar`]. Selecting a day closes the popover (`initialFocus` puts
//! keyboard focus on the selected day when it opens).
//!
//! Two further docs compositions are covered:
//!
//! - **Range** ([`DatePicker::range`], `date-picker-with-range.tsx`): the
//!   popover stays open until both ends of the range are chosen, and the
//!   trigger shows the `LLL dd, y - LLL dd, y` span.
//! - **Presets** ([`DatePicker::with_presets`],
//!   `date-picker-with-presets.tsx`): the popover becomes a `p-2` column
//!   with a `Select` ("Today", "Tomorrow", …) above a `rounded-md border`
//!   calendar; picking a preset sets `addDays(today, n)` and keeps the
//!   popover open.
//!
//! The selection is always controlled; the popover open state is internal
//! and persisted per `id_salt`.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut date = None;
//! if DatePicker::new(&mut date).id_salt("birthday").show(ui).changed() {
//!     // date was picked or cleared
//! }
//!
//! let mut range = None;
//! DatePicker::range(&mut range).width(300.0).show(ui);
//! ```

use crate::components::calendar::{
    Calendar, CalendarDate, WeekStart, CALENDAR_GRID_WIDTH, CALENDAR_PADDING, MONTH_SHORT,
};
use crate::components::select::{Select, SelectSize};
use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme, Style};
use crate::utils::{animation::overlay_progress, focus, shadow};
use egui::{
    Align2, Color32, CornerRadius, FontId, Frame, Id, Key, LayerId, Margin, Popup, PopupAnchor,
    Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2,
};

/// Trigger width: the docs demo's `w-[240px]`.
pub const DATE_PICKER_WIDTH: f32 = 240.0;
/// Distance between trigger and popover (`sideOffset = 4`).
pub const DATE_PICKER_SIDE_OFFSET: f32 = 4.0;
/// Trigger height: the outline button's `h-9`.
const TRIGGER_HEIGHT: f32 = crate::theme::tokens::spacing::CONTROL;
/// Trigger horizontal padding: `px-4`.
const TRIGGER_PADDING_X: f32 = crate::theme::tokens::spacing::LG;
/// Gap between the calendar icon and the label: `gap-2`.
const TRIGGER_GAP: f32 = crate::theme::tokens::spacing::SM;
/// Calendar icon size (`size-4`).
const ICON_SIZE: f32 = 16.0;
/// Presets popover inner margin (`p-2`) and select-to-calendar gap
/// (`space-y-2`).
const PRESETS_MARGIN: f32 = 8.0;

/// The default placeholder from the docs demo.
pub const DATE_PICKER_PLACEHOLDER: &str = "Pick a date";

/// A quick-pick entry for [`DatePicker::with_presets`] (the
/// `date-picker-with-presets` demo's "Today / Tomorrow / …" select items).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DatePickerPreset {
    /// Label shown in the select ("Today").
    pub label: String,
    /// Days added to today when chosen (date-fns `addDays(today, n)`).
    pub offset: i64,
}

impl DatePickerPreset {
    /// Creates a preset: `label` sets the date to today + `offset` days.
    pub fn new(label: impl Into<String>, offset: i64) -> Self {
        Self {
            label: label.into(),
            offset,
        }
    }
}

/// Per-instance state persisted in the egui context.
#[derive(Clone, Copy, Debug, Default)]
struct DatePickerState {
    open: bool,
    /// Bumped on every open so the inner calendar starts fresh each time
    /// (matching react-day-picker, which shows the selected/today month on
    /// each open).
    open_count: u32,
    trigger: Option<TriggerInfo>,
}

#[derive(Clone, Copy, Debug)]
struct TriggerInfo {
    rect: Rect,
    id: Id,
    layer_id: LayerId,
}

/// The date-picker's controlled selection (`mode` in react-day-picker).
enum DatePickerSelection<'a> {
    /// `mode="single"`.
    Single(&'a mut Option<CalendarDate>),
    /// `mode="range"` (the `date-picker-with-range` demo).
    Range(&'a mut Option<(CalendarDate, CalendarDate)>),
}

/// The shadcn date-picker. See the [module docs](self) for an example.
pub struct DatePicker<'a> {
    selected: DatePickerSelection<'a>,
    id_salt: Option<String>,
    placeholder: String,
    width: f32,
    default_month: Option<CalendarDate>,
    show_outside_days: bool,
    week_start: WeekStart,
    initial_focus: bool,
    disabled: bool,
    disabled_days: Option<crate::components::calendar::DisabledFn<'a>>,
    presets: Vec<DatePickerPreset>,
}

impl<'a> DatePicker<'a> {
    /// Creates a single-mode date-picker bound to the caller's date.
    pub fn new(selected: &'a mut Option<CalendarDate>) -> Self {
        Self {
            selected: DatePickerSelection::Single(selected),
            id_salt: None,
            placeholder: DATE_PICKER_PLACEHOLDER.to_owned(),
            width: DATE_PICKER_WIDTH,
            default_month: None,
            show_outside_days: true,
            week_start: WeekStart::default(),
            initial_focus: true,
            disabled: false,
            disabled_days: None,
            presets: Vec::new(),
        }
    }

    /// Creates a range-mode date-picker (the `date-picker-with-range` demo):
    /// the popover stays open until both ends are chosen, then closes.
    pub fn range(selected: &'a mut Option<(CalendarDate, CalendarDate)>) -> Self {
        Self {
            selected: DatePickerSelection::Range(selected),
            id_salt: None,
            placeholder: DATE_PICKER_PLACEHOLDER.to_owned(),
            width: DATE_PICKER_WIDTH,
            default_month: None,
            show_outside_days: true,
            week_start: WeekStart::default(),
            initial_focus: true,
            disabled: false,
            disabled_days: None,
            presets: Vec::new(),
        }
    }

    /// Adds quick-pick presets rendered as a `Select` above the calendar
    /// (the `date-picker-with-presets` demo: `flex w-auto flex-col
    /// space-y-2 p-2` popover content). Picking a preset sets
    /// `today + offset` and keeps the popover open.
    pub fn with_presets(mut self, presets: impl IntoIterator<Item = DatePickerPreset>) -> Self {
        self.presets = presets.into_iter().collect();
        self
    }

    /// Sets a stable id salt so the open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Overrides the placeholder shown when no date is selected
    /// (default: [`DATE_PICKER_PLACEHOLDER`]).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Overrides the trigger width (default: [`DATE_PICKER_WIDTH`]).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Month the calendar shows on the first open (defaults to the selected
    /// month, else today's).
    pub fn default_month(mut self, month: CalendarDate) -> Self {
        self.default_month = Some(month.first_of_month());
        self
    }

    /// Passes `showOutsideDays` through to the inner calendar.
    pub fn show_outside_days(mut self, show: bool) -> Self {
        self.show_outside_days = show;
        self
    }

    /// Passes `weekStartsOn` through to the inner calendar.
    pub fn week_starts_on(mut self, start: WeekStart) -> Self {
        self.week_start = start;
        self
    }

    /// Whether the calendar grabs keyboard focus when opened
    /// (the source's `initialFocus`, default `true`).
    pub fn initial_focus(mut self, on: bool) -> Self {
        self.initial_focus = on;
        self
    }

    /// Disables the trigger.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Disables days in the inner calendar (the `disabled` matcher).
    pub fn disabled_when(mut self, f: impl Fn(&CalendarDate) -> bool + 'a) -> Self {
        self.disabled_days = Some(Box::new(f));
        self
    }

    /// Renders the trigger and, when open, the calendar popover. The returned
    /// response is `changed()` when the popover toggled or a date was picked.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_date_picker"),
        };
        let ctx = ui.ctx().clone();
        let mut state: DatePickerState = ctx
            .data_mut(|d| d.get_temp::<DatePickerState>(state_id))
            .unwrap_or_default();

        // --- Trigger (outline button, `justify-start text-left font-normal`)
        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let id = state_id.with("trigger");
        let (_, rect) = ui.allocate_space(Vec2::new(self.width, TRIGGER_HEIGHT));
        let mut response = ui.interact(rect, id, sense);
        let radius_md = theme.radius.md_u8();
        let alpha = if self.disabled { 0.5 } else { 1.0 };
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let has_date = selection_is_some(&self.selected);
            let text_color = if has_date {
                p.foreground
            } else {
                p.muted_foreground
            };
            // The outline button recipe (`border bg-background shadow-xs
            // hover:bg-accent dark:bg-input/30 dark:hover:bg-input/50`; nova
            // `hover:bg-muted`), matching the crate's outline Button. The
            // nova outline carries no `shadow-xs`.
            let nova = theme.style == Style::Nova;
            let hovered = response.hovered() && !self.disabled;
            let fill = outline_fill(&p, dark, hovered, nova);
            let stroke_color = if dark { p.input } else { p.border };
            if !self.disabled && !nova {
                let sh = shadow::shadow_xs(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        CornerRadius::same(radius_md),
                        sh.color,
                    )
                    .with_blur_width(2.0),
                );
            }
            painter.rect_filled(
                rect,
                CornerRadius::same(radius_md),
                color_alpha(fill, alpha),
            );
            painter.rect_stroke(
                rect,
                CornerRadius::same(radius_md),
                Stroke::new(1.0, color_alpha(stroke_color, alpha)),
                StrokeKind::Inside,
            );
            // Icon + label, left aligned.
            let icon_rect = Rect::from_center_size(
                Pos2::new(
                    rect.min.x + TRIGGER_PADDING_X + ICON_SIZE / 2.0,
                    rect.center().y,
                ),
                Vec2::splat(ICON_SIZE),
            );
            Icon::Calendar.paint(
                painter,
                icon_rect,
                color_alpha(text_color, alpha),
                2.0 * ICON_SIZE / 24.0,
            );
            let label = selection_label(&self.selected, &self.placeholder);
            let text_pos = Pos2::new(
                rect.min.x + TRIGGER_PADDING_X + ICON_SIZE + TRIGGER_GAP,
                rect.center().y,
            );
            painter.text(
                text_pos,
                Align2::LEFT_CENTER,
                label,
                FontId::proportional(crate::theme::tokens::type_scale::SM),
                color_alpha(text_color, alpha),
            );
            // `focus-visible:ring-[3px] focus-visible:ring-ring/50`
            if response.has_focus() && !self.disabled {
                focus::paint_ring(
                    painter,
                    rect,
                    color_alpha(p.ring, 0.5),
                    3.0,
                    theme.radius.md(),
                );
            }
        }

        // --- Toggle -----------------------------------------------------------
        if response.clicked() && !self.disabled {
            state.open = !state.open;
            state.open_count += 1;
            response.mark_changed();
        }
        state.trigger = Some(TriggerInfo {
            rect: response.rect,
            id: response.id,
            layer_id: response.layer_id,
        });
        ctx.data_mut(|d| d.insert_temp(state_id, state));

        // --- Popup (`PopoverContent className="w-auto p-0" align="start"`) ----
        let Some(trigger) = state.trigger else {
            return response;
        };
        let popup_id = state_id.with("popup");
        let anim_id = state_id.with("anim");
        let progress = overlay_progress(&ctx, anim_id, state.open);
        if !(state.open || progress.alpha > 0.002) {
            return response;
        }

        let width = if self.presets.is_empty() {
            CALENDAR_GRID_WIDTH + 2.0 * CALENDAR_PADDING
        } else {
            // `p-2` presets popover: calendar width + 2×8 inner margin.
            CALENDAR_GRID_WIDTH + 2.0 * CALENDAR_PADDING + 2.0 * PRESETS_MARGIN
        };
        let gap = DATE_PICKER_SIDE_OFFSET + (1.0 - progress.alpha) * 8.0;
        let frame = Frame::new()
            .fill(p.popover)
            .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
            .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
            .inner_margin(if self.presets.is_empty() {
                Margin::same(0)
            } else {
                // `flex w-auto flex-col space-y-2 p-2`
                Margin::same(PRESETS_MARGIN as i8)
            })
            .shadow(shadow::shadow_md(dark));

        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        let mut close = false;
        let popup = Popup::new(
            popup_id,
            ctx.clone(),
            PopupAnchor::ParentRect(trigger.rect),
            trigger.layer_id,
        )
        .open(true)
        .align(egui::emath::RectAlign::BOTTOM_START)
        .gap(gap)
        .width(width)
        .frame(Frame::new())
        .anchor_widget(trigger.id);

        let inner = popup.show(|ui| {
            ui.set_opacity(progress.alpha);
            frame.show(ui, |ui| {
                let today = CalendarDate::today(ui);
                if self.presets.is_empty() {
                    self.show_inner_calendar(
                        ui,
                        today,
                        state.open_count,
                        &mut response,
                        &mut close,
                    );
                } else {
                    // `date-picker-with-presets`: a Select above a
                    // `rounded-md border` calendar, `space-y-2` apart.
                    let labels: Vec<&str> = self
                        .presets
                        .iter()
                        .map(|preset| preset.label.as_str())
                        .collect();
                    let select = Select::uncontrolled()
                        .options(labels)
                        .placeholder("Select")
                        .size(SelectSize::Sm)
                        .width(CALENDAR_GRID_WIDTH + 2.0 * CALENDAR_PADDING)
                        .id_salt(format!("{}_presets", state_id.value()));
                    let sel = select.show_seeded(ui);
                    if sel.changed {
                        if let Some(label) = sel.value {
                            if let Some(preset) =
                                self.presets.iter().find(|preset| preset.label == label)
                            {
                                let picked = CalendarDate::from_epoch_days(
                                    today.epoch_days() + preset.offset,
                                );
                                match &mut self.selected {
                                    DatePickerSelection::Single(sel) => **sel = Some(picked),
                                    DatePickerSelection::Range(sel) => {
                                        **sel = Some((picked, picked))
                                    }
                                }
                                response.mark_changed();
                            }
                        }
                    }
                    ui.add_space(PRESETS_MARGIN);
                    Frame::new()
                        .stroke(Stroke::new(1.0, p.border))
                        .corner_radius(CornerRadius::same(theme.radius.md_u8()))
                        .show(ui, |ui| {
                            self.show_inner_calendar(
                                ui,
                                today,
                                state.open_count,
                                &mut response,
                                &mut close,
                            );
                        });
                }
            });
        });

        if close {
            state.open = false;
            ctx.data_mut(|d| d.insert_temp(state_id, state));
            response.mark_changed();
        }

        // Dismissal: Escape or a click outside the popup (and outside the
        // trigger — clicking the trigger toggles instead). Skipped on the
        // opening frame and while closing; modals keep Escape priority.
        if let Some(ref r) = inner {
            let modal_open = ctx.memory(|m| m.top_modal_layer().is_some());
            let esc = !modal_open && ctx.input(|i| i.key_pressed(Key::Escape));
            let outside = r.response.clicked_elsewhere();
            if state.open && was_open_last_frame && (outside || esc) {
                state.open = false;
                ctx.data_mut(|d| d.insert_temp(state_id, state));
                response.mark_changed();
            }
        }
        response
    }

    /// The inner calendar of the popover (shared by the plain, range and
    /// presets variants). Marks `response` changed when a day was picked and
    /// sets `close`: single mode closes on any pick; range mode stays open
    /// until both ends are chosen ([`CalendarResponse::completed`]).
    fn show_inner_calendar(
        &mut self,
        ui: &mut Ui,
        today: CalendarDate,
        open_count: u32,
        response: &mut Response,
        close: &mut bool,
    ) {
        // Fresh calendar state per open (see `DatePickerState::open_count`).
        let salt = format!(
            "date_picker_calendar_{}_{}",
            self.id_salt.as_deref().unwrap_or("default"),
            open_count
        );
        let is_range = matches!(self.selected, DatePickerSelection::Range(_));
        let start_month = self.default_month.unwrap_or_else(|| match &self.selected {
            DatePickerSelection::Single(sel) => {
                (**sel).map(|d| d.first_of_month()).unwrap_or(today)
            }
            DatePickerSelection::Range(sel) => (**sel)
                .map(|(from, _)| from.first_of_month())
                .unwrap_or(today),
        });
        let mut cal = match &mut self.selected {
            DatePickerSelection::Single(sel) => Calendar::single(sel),
            DatePickerSelection::Range(sel) => Calendar::range(sel),
        };
        cal = cal
            .id_salt(salt)
            .default_month(start_month)
            .week_starts_on(self.week_start)
            .show_outside_days(self.show_outside_days);
        if self.initial_focus {
            cal = cal.initial_focus(true);
        }
        if let Some(pred) = self.disabled_days.take() {
            cal = cal.disabled_when(move |d| pred(d));
        }
        let cal_response = cal.show(ui);
        if cal_response.changed {
            response.mark_changed();
            if !is_range {
                // Single mode: selecting a day closes the popover.
                *close = true;
            }
        }
        if cal_response.completed {
            // Range mode: the popover stays open until both ends are chosen.
            *close = true;
        }
    }
}

/// The trigger label: `format(date, "PPP")` or the placeholder.
fn trigger_label(selected: Option<CalendarDate>, placeholder: &str) -> String {
    match selected {
        Some(date) => date.format_ppp(),
        None => placeholder.to_owned(),
    }
}

/// The range trigger label: the source demo's
/// `format(from, "LLL dd, y") - format(to, "LLL dd, y")`; a pending start
/// (or one-day range) shows the start alone.
fn trigger_label_range(range: Option<(CalendarDate, CalendarDate)>, placeholder: &str) -> String {
    match range {
        Some((from, to)) if from != to => {
            format!("{} - {}", format_lll_dd_y(from), format_lll_dd_y(to))
        }
        Some((from, _)) => format_lll_dd_y(from),
        None => placeholder.to_owned(),
    }
}

/// date-fns `LLL dd, y` formatting: "Jan 20, 2022".
fn format_lll_dd_y(date: CalendarDate) -> String {
    let month = MONTH_SHORT[(date.month.clamp(1, 12) - 1) as usize];
    format!("{} {:02}, {}", month, date.day, date.year)
}

/// Whether the current selection holds a value (drives the placeholder's
/// `text-muted-foreground`).
fn selection_is_some(selected: &DatePickerSelection<'_>) -> bool {
    match selected {
        DatePickerSelection::Single(sel) => sel.is_some(),
        DatePickerSelection::Range(sel) => sel.is_some(),
    }
}

/// The trigger label for the active mode.
fn selection_label(selected: &DatePickerSelection<'_>, placeholder: &str) -> String {
    match selected {
        DatePickerSelection::Single(sel) => trigger_label(**sel, placeholder),
        DatePickerSelection::Range(sel) => trigger_label_range(**sel, placeholder),
    }
}

/// Outline-variant fill, mirroring the crate's outline button recipe
/// (`bg-background hover:bg-accent`, `dark:bg-input/30
/// dark:hover:bg-input/50`; nova `hover:bg-muted`).
fn outline_fill(p: &crate::theme::Palette, dark: bool, hovered: bool, nova: bool) -> Color32 {
    if hovered {
        if dark {
            color_alpha(p.input, 0.5)
        } else if nova {
            p.muted
        } else {
            p.accent
        }
    } else if dark {
        color_alpha(p.input, 0.3)
    } else {
        p.background
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::calendar::CALENDAR_CELL_SIZE;
    use crate::theme::{Mode, Preset};
    use egui::Rect as ERect;

    #[test]
    fn trigger_label_uses_ppp_or_placeholder() {
        let d = CalendarDate::new(2026, 11, 10).unwrap();
        assert_eq!(trigger_label(Some(d), "Pick a date"), "November 10th, 2026");
        assert_eq!(trigger_label(None, "Pick a date"), "Pick a date");
    }

    #[test]
    fn geometry_constants_match_source() {
        // w-[240px] trigger, sideOffset 4, calendar is 196 + 2×8 wide.
        assert_eq!(DATE_PICKER_WIDTH, 240.0);
        assert_eq!(DATE_PICKER_SIDE_OFFSET, 4.0);
        assert_eq!(CALENDAR_GRID_WIDTH + 2.0 * CALENDAR_PADDING, 212.0);
    }

    #[test]
    fn builders_keep_fields() {
        let mut selected = None;
        let dp = DatePicker::new(&mut selected)
            .id_salt("dp")
            .placeholder("When?")
            .width(300.0)
            .week_starts_on(WeekStart::Monday)
            .show_outside_days(false)
            .initial_focus(false)
            .disabled(true);
        assert_eq!(dp.id_salt.as_deref(), Some("dp"));
        assert_eq!(dp.placeholder, "When?");
        assert_eq!(dp.width, 300.0);
        assert_eq!(dp.week_start, WeekStart::Monday);
        assert!(!dp.show_outside_days);
        assert!(!dp.initial_focus);
        assert!(dp.disabled);
    }

    #[test]
    fn range_and_preset_builders_keep_fields() {
        let mut range = None;
        let dp = DatePicker::range(&mut range)
            .id_salt("rng")
            .width(300.0)
            .with_presets([
                DatePickerPreset::new("Today", 0),
                DatePickerPreset::new("In 3 days", 3),
            ]);
        assert_eq!(dp.id_salt.as_deref(), Some("rng"));
        assert_eq!(dp.width, 300.0);
        assert!(matches!(dp.selected, DatePickerSelection::Range(_)));
        assert_eq!(
            dp.presets,
            vec![
                DatePickerPreset {
                    label: "Today".to_owned(),
                    offset: 0
                },
                DatePickerPreset {
                    label: "In 3 days".to_owned(),
                    offset: 3
                }
            ]
        );
        let mut single = None;
        assert!(matches!(
            DatePicker::new(&mut single).selected,
            DatePickerSelection::Single(_)
        ));
    }

    #[test]
    fn range_trigger_label_matches_source_format() {
        let from = CalendarDate::new(2022, 1, 20).unwrap();
        let to = CalendarDate::new(2022, 2, 9).unwrap();
        // format(from, "LLL dd, y") - format(to, "LLL dd, y")
        assert_eq!(
            trigger_label_range(Some((from, to)), "Pick a date"),
            "Jan 20, 2022 - Feb 09, 2022"
        );
        // A pending start (or one-day range) shows the start alone.
        assert_eq!(
            trigger_label_range(Some((from, from)), "Pick a date"),
            "Jan 20, 2022"
        );
        assert_eq!(trigger_label_range(None, "Pick a date"), "Pick a date");
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }

    /// Advances time and renders the date-picker, like the popover tests.
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
                    Id::unique("date_picker_test_ui"),
                    egui::UiBuilder::new().max_rect(ERect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(600.0, 600.0),
                    )),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        /// Presses and releases the primary button at `pos` over two frames.
        fn click(&mut self, pos: egui::Pos2, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.time += 0.1;
                let input = egui::RawInput {
                    time: Some(self.time),
                    events: vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    ..Default::default()
                };
                let mut out = self.ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        Id::unique("date_picker_test_ui"),
                        egui::UiBuilder::new().max_rect(ERect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(600.0, 600.0),
                        )),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }
        }

        fn send_key(&mut self, key: Key, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events: vec![egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                    physical_key: None,
                }],
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("date_picker_test_ui"),
                    egui::UiBuilder::new().max_rect(ERect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(600.0, 600.0),
                    )),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn state(&self, salt: &str) -> DatePickerState {
            let id = Id::unique("date_picker_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<DatePickerState>(id))
                .unwrap_or_default()
        }

        fn popup_rect(&self, salt: &str) -> Option<ERect> {
            let popup_id = Id::unique("date_picker_test_ui").with(salt).with("popup");
            self.ctx.read_response(popup_id).map(|r| r.rect)
        }
    }

    #[test]
    fn trigger_click_toggles_open_state() {
        let mut h = Harness::new();
        let mut trigger_rect = ERect::ZERO;

        h.run(|ui| {
            let mut date = None;
            trigger_rect = DatePicker::new(&mut date).id_salt("toggle").show(ui).rect;
        });
        assert!(!h.state("toggle").open);

        h.click(trigger_rect.center(), |ui| {
            let mut date = None;
            DatePicker::new(&mut date).id_salt("toggle").show(ui);
        });
        assert!(h.state("toggle").open, "clicking the trigger opens it");

        h.click(trigger_rect.center(), |ui| {
            let mut date = None;
            DatePicker::new(&mut date).id_salt("toggle").show(ui);
        });
        assert!(
            !h.state("toggle").open,
            "clicking the trigger again closes it"
        );
    }

    #[test]
    fn escape_closes_open_picker() {
        let mut h = Harness::new();
        let mut trigger_rect = ERect::ZERO;

        h.run(|ui| {
            let mut date = None;
            trigger_rect = DatePicker::new(&mut date).id_salt("esc").show(ui).rect;
        });
        h.click(trigger_rect.center(), |ui| {
            let mut date = None;
            DatePicker::new(&mut date).id_salt("esc").show(ui);
        });
        assert!(h.state("esc").open);

        h.send_key(Key::Escape, |ui| {
            let mut date = None;
            DatePicker::new(&mut date).id_salt("esc").show(ui);
        });
        assert!(!h.state("esc").open, "Escape must close the picker");
    }

    #[test]
    fn picking_a_day_updates_date_and_closes() {
        let mut h = Harness::new();
        let mut trigger_rect = ERect::ZERO;
        // A real caller owns the date across frames; mirror that with temp
        // state keyed by a stable id.
        let picked_id = Id::unique("date_picker_picked");
        let with_date = |ui: &mut Ui| -> Response {
            let mut date: Option<CalendarDate> = ui
                .ctx()
                .data_mut(|d| d.get_temp::<Option<CalendarDate>>(picked_id))
                .flatten();
            let r = DatePicker::new(&mut date).id_salt("pick").show(ui);
            ui.ctx().data_mut(|d| d.insert_temp(picked_id, date));
            r
        };

        h.run(|ui| {
            trigger_rect = with_date(ui).rect;
        });
        h.click(trigger_rect.center(), |ui| {
            with_date(ui);
        });
        assert!(h.state("pick").open);

        // Let the open animation settle so the popup position is stable.
        for _ in 0..4 {
            h.run(|ui| {
                with_date(ui);
            });
        }

        // Click the 1st of the shown month. The harness clock starts at the
        // Unix epoch, so the calendar shows January 1970 (Jan 1 = Thursday,
        // 4 leading cells with the default Sunday week start). Layout:
        // p-2 (8) + caption (28) + gap-4 (16) + weekday row (20) + the
        // `week: mt-2` 8 px gap above the first week row.
        let Some(popup) = h.popup_rect("pick") else {
            panic!("popup should be visible");
        };
        let day_center = Pos2::new(
            popup.min.x + CALENDAR_PADDING + 4.0 * CALENDAR_CELL_SIZE + CALENDAR_CELL_SIZE / 2.0,
            popup.min.y + 8.0 + 28.0 + 16.0 + 20.0 + 8.0 + CALENDAR_CELL_SIZE / 2.0,
        );
        h.click(day_center, |ui| {
            with_date(ui);
        });

        h.run(|ui| {
            with_date(ui);
        });
        assert!(!h.state("pick").open, "picking a day closes the popover");
        let selected: Option<CalendarDate> = h
            .ctx
            .data_mut(|d| d.get_temp::<Option<CalendarDate>>(picked_id))
            .flatten();
        assert_eq!(
            selected,
            Some(CalendarDate::new(1970, 1, 1).unwrap()),
            "clicked day becomes the selected date"
        );
    }

    /// Jan 1970 day-cell center for the popup-relative column/row (0-based).
    fn day_center(popup: ERect, col: f32, row: f32) -> Pos2 {
        Pos2::new(
            popup.min.x + CALENDAR_PADDING + col * CALENDAR_CELL_SIZE + CALENDAR_CELL_SIZE / 2.0,
            popup.min.y
                + 8.0
                + 28.0
                + 16.0
                + 20.0
                + 8.0
                + row * (CALENDAR_CELL_SIZE + 8.0)
                + CALENDAR_CELL_SIZE / 2.0,
        )
    }

    #[test]
    fn range_picking_stays_open_until_both_ends() {
        let mut h = Harness::new();
        let mut trigger_rect = ERect::ZERO;
        let range_id = Id::unique("date_picker_range");
        let with_range = |ui: &mut Ui| -> Response {
            let mut range: Option<(CalendarDate, CalendarDate)> = ui
                .ctx()
                .data_mut(|d| d.get_temp::<Option<(CalendarDate, CalendarDate)>>(range_id))
                .flatten();
            let r = DatePicker::range(&mut range).id_salt("rng").show(ui);
            ui.ctx().data_mut(|d| d.insert_temp(range_id, range));
            r
        };

        h.run(|ui| {
            trigger_rect = with_range(ui).rect;
        });
        h.click(trigger_rect.center(), |ui| {
            with_range(ui);
        });
        assert!(h.state("rng").open);
        for _ in 0..4 {
            h.run(|ui| {
                with_range(ui);
            });
        }

        // Jan 1 1970 (row 0, col 4): starts a pending range.
        let Some(popup) = h.popup_rect("rng") else {
            panic!("popup should be visible");
        };
        h.click(day_center(popup, 4.0, 0.0), |ui| {
            with_range(ui);
        });
        h.run(|ui| {
            with_range(ui);
        });
        assert!(
            h.state("rng").open,
            "a pending range keeps the popover open"
        );
        let pending: Option<(CalendarDate, CalendarDate)> = h
            .ctx
            .data_mut(|d| d.get_temp::<Option<(CalendarDate, CalendarDate)>>(range_id))
            .flatten();
        assert_eq!(
            pending,
            Some((
                CalendarDate::new(1970, 1, 1).unwrap(),
                CalendarDate::new(1970, 1, 1).unwrap()
            )),
            "start day stored while pending"
        );

        // Jan 15 1970 (Jan 1 + 14 days → row 2, col 4): completes the range.
        let Some(popup) = h.popup_rect("rng") else {
            panic!("popup should still be visible");
        };
        h.click(day_center(popup, 4.0, 2.0), |ui| {
            with_range(ui);
        });
        h.run(|ui| {
            with_range(ui);
        });
        assert!(
            !h.state("rng").open,
            "completing the range closes the popover"
        );
        let range: Option<(CalendarDate, CalendarDate)> = h
            .ctx
            .data_mut(|d| d.get_temp::<Option<(CalendarDate, CalendarDate)>>(range_id))
            .flatten();
        assert_eq!(
            range,
            Some((
                CalendarDate::new(1970, 1, 1).unwrap(),
                CalendarDate::new(1970, 1, 15).unwrap()
            )),
            "both ends stored"
        );
    }

    #[test]
    fn presets_variant_renders_select_and_calendar() {
        let mut h = Harness::new();
        let mut trigger_rect = ERect::ZERO;
        let with_presets = |ui: &mut Ui| -> Response {
            let mut date = None;
            DatePicker::new(&mut date)
                .id_salt("presets")
                .with_presets([
                    DatePickerPreset::new("Today", 0),
                    DatePickerPreset::new("Tomorrow", 1),
                    DatePickerPreset::new("In 3 days", 3),
                    DatePickerPreset::new("In a week", 7),
                ])
                .show(ui)
        };

        h.run(|ui| {
            trigger_rect = with_presets(ui).rect;
        });
        h.click(trigger_rect.center(), |ui| {
            with_presets(ui);
        });
        assert!(h.state("presets").open);
        for _ in 0..4 {
            h.run(|ui| {
                with_presets(ui);
            });
        }
        let Some(popup) = h.popup_rect("presets") else {
            panic!("popup should be visible");
        };
        // The presets popover is the calendar width + the `p-2` margins.
        assert!(
            popup.width() >= CALENDAR_GRID_WIDTH + 2.0 * CALENDAR_PADDING + 16.0 - 2.0,
            "presets popover fits the select and the calendar, got {}",
            popup.width()
        );
        // Escape dismisses like the plain variant.
        h.send_key(Key::Escape, |ui| {
            with_presets(ui);
        });
        assert!(!h.state("presets").open);
    }
}
