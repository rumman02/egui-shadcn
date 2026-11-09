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
//! The picked date is always controlled (`&mut Option<CalendarDate>`); the
//! popover open state is internal and persisted per `id_salt`.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut date = None;
//! if DatePicker::new(&mut date).id_salt("birthday").show(ui).changed() {
//!     // date was picked or cleared
//! }
//! ```

use crate::components::calendar::{
    Calendar, CalendarDate, WeekStart, CALENDAR_GRID_WIDTH, CALENDAR_PADDING,
};
use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use crate::utils::{animation::overlay_progress, focus, shadow};
use egui::{
    Align2, CornerRadius, FontId, Frame, Id, Key, LayerId, Margin, Popup, PopupAnchor, Pos2, Rect,
    Response, Sense, Stroke, StrokeKind, Ui, Vec2,
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

/// The default placeholder from the docs demo.
pub const DATE_PICKER_PLACEHOLDER: &str = "Pick a date";

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

/// The shadcn date-picker. See the [module docs](self) for an example.
pub struct DatePicker<'a> {
    selected: &'a mut Option<CalendarDate>,
    id_salt: Option<String>,
    placeholder: String,
    width: f32,
    default_month: Option<CalendarDate>,
    show_outside_days: bool,
    week_start: WeekStart,
    initial_focus: bool,
    disabled: bool,
    disabled_days: Option<crate::components::calendar::DisabledFn<'a>>,
}

impl<'a> DatePicker<'a> {
    /// Creates a date-picker bound to the caller's date.
    pub fn new(selected: &'a mut Option<CalendarDate>) -> Self {
        Self {
            selected,
            id_salt: None,
            placeholder: DATE_PICKER_PLACEHOLDER.to_owned(),
            width: DATE_PICKER_WIDTH,
            default_month: None,
            show_outside_days: true,
            week_start: WeekStart::default(),
            initial_focus: true,
            disabled: false,
            disabled_days: None,
        }
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
            let has_date = self.selected.is_some();
            let text_color = if has_date {
                p.foreground
            } else {
                p.muted_foreground
            };
            // `border bg-background shadow-xs hover:bg-accent`
            if !self.disabled {
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
            let fill = if response.hovered() && !self.disabled {
                p.accent
            } else {
                p.background
            };
            painter.rect_filled(
                rect,
                CornerRadius::same(radius_md),
                color_alpha(fill, alpha),
            );
            painter.rect_stroke(
                rect,
                CornerRadius::same(radius_md),
                Stroke::new(1.0, color_alpha(p.input, alpha)),
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
            let label = trigger_label(*self.selected, &self.placeholder);
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

        let width = CALENDAR_GRID_WIDTH + 2.0 * CALENDAR_PADDING;
        let gap = DATE_PICKER_SIDE_OFFSET + (1.0 - progress.alpha) * 8.0;
        let frame = Frame::new()
            .fill(p.popover)
            .stroke(Stroke::new(1.0, color_alpha(p.foreground, 0.10)))
            .corner_radius(CornerRadius::same(theme.radius.lg_u8()))
            .inner_margin(Margin::same(0))
            .shadow(shadow::shadow_md(dark));

        let was_open_last_frame = ctx.read_response(popup_id).is_some();
        let mut date_changed = false;
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
                // Fresh calendar state per open (see `DatePickerState::open_count`).
                let salt = format!(
                    "date_picker_calendar_{}_{}",
                    self.id_salt.as_deref().unwrap_or("default"),
                    state.open_count
                );
                let start_month = self
                    .default_month
                    .unwrap_or_else(|| self.selected.map(|d| d.first_of_month()).unwrap_or(today));
                let mut cal = Calendar::single(&mut *self.selected)
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
                    // Single mode: selecting a day closes the popover.
                    date_changed = true;
                }
            });
        });

        if date_changed {
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
}

/// The trigger label: `format(date, "PPP")` or the placeholder.
fn trigger_label(selected: Option<CalendarDate>, placeholder: &str) -> String {
    match selected {
        Some(date) => date.format_ppp(),
        None => placeholder.to_owned(),
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
        // 4 leading cells with the default Sunday week start).
        let Some(popup) = h.popup_rect("pick") else {
            panic!("popup should be visible");
        };
        let day_center = Pos2::new(
            popup.min.x + CALENDAR_PADDING + 4.0 * CALENDAR_CELL_SIZE + CALENDAR_CELL_SIZE / 2.0,
            popup.min.y + 8.0 + 28.0 + 16.0 + 20.0 + CALENDAR_CELL_SIZE / 2.0,
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
}
