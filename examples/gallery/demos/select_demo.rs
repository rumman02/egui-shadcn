//! Select demo: every spec state observable and drivable for S5/S6/S7.
//!
//! Sections: probe readouts (open/closed + value + last action + focus, all
//! selects), basic select (label + separator + disabled item + value
//! override), sizes (default/sm — heights are recipe-resolved: nova 32/28px,
//! base 36/32px), single option (content-sized popup, no empty area), 40
//! options (scroll buttons + highlight scroll-follow), groups + labels +
//! separators (via `SelectBuilder::group`), disabled + invalid triggers,
//! empty select (OPEN-15: padding only, no invented empty state), keyboard
//! exercise (page widgets before/after the trigger; arrows/Home/End/
//! typeahead/Escape/Tab), long text (E1/OPEN-17: trigger clamp + item clip),
//! and a value-not-in-options select (OPEN-16: raw value shown, nothing
//! checked — the pre-existing behaviour, unsettled by the spec).
//!
//! Open/closed state is inferred per frame from `SelectResponse`: the
//! `closed` flag is exact, and opens are detected from trigger activation
//! (pointer click or Enter/Space on the focused trigger) and ArrowDown/Up
//! while the trigger is focused — the same transitions the component
//! implements. The "last action" classification distinguishes trigger click,
//! Enter open (first item), Space open (selected item), arrow open, item
//! click, keyboard select, Escape, Tab, trigger toggle, outside click, and
//! closed-trigger typeahead, from this frame's input.
//!
//! Highlighted index is NOT observable: `SelectResponse` exposes no
//! highlight field, and the highlight lives in component-private state. The
//! demo does not guess or re-implement highlight tracking; S6 observes the
//! highlighted row visually via the accent fill.

use egui_shadcn_ui::prelude::*;

/// One slot per demo select, in page order.
const BASIC: usize = 0;
const SIZE_SM: usize = 1;
const SIZE_DEFAULT: usize = 2;
const SINGLE: usize = 3;
const MANY: usize = 4;
const GROUPED: usize = 5;
const DISABLED: usize = 6;
const INVALID: usize = 7;
const EMPTY: usize = 8;
const KEYBOARD: usize = 9;
const LONG_TEXT: usize = 10;
const NO_VALUE: usize = 11;
const N_SELECTS: usize = 12;

const NAMES: [&str; N_SELECTS] = [
    "basic",
    "size-sm",
    "size-default",
    "single",
    "many",
    "grouped",
    "disabled",
    "invalid",
    "empty",
    "keyboard",
    "long-text",
    "no-value",
];

/// Demo-local probe state, held in egui temp data (the gallery's established
/// demo-state pattern, as in the dialog probe).
#[derive(Clone)]
struct SelectProbeState {
    /// Inferred open flag per select (written back every frame).
    open: [bool; N_SELECTS],
    /// Last classified open/close/value action per select.
    last: [String; N_SELECTS],
    /// Last reported value per select.
    values: [Option<String>; N_SELECTS],
    /// Widget holding keyboard focus, as named last frame (one frame stale).
    focus: String,
    basic: Option<String>,
    size_sm: String,
    size_default: String,
    grouped: Option<String>,
    keyboard: Option<String>,
    long_text: Option<String>,
    no_value: Option<String>,
}

impl Default for SelectProbeState {
    fn default() -> Self {
        Self {
            open: [false; N_SELECTS],
            last: std::array::from_fn(|_| "never opened".to_owned()),
            values: std::array::from_fn(|_| None),
            focus: "none".to_owned(),
            basic: None,
            size_sm: String::new(),
            size_default: String::new(),
            grouped: None,
            // Prefilled mid-list so Space-open (K1) highlights Cherry while
            // Enter-open (K2) highlights the first item.
            keyboard: Some("Cherry".to_owned()),
            long_text: None,
            // OPEN-16: a value with no matching option.
            no_value: Some("Missing value".to_owned()),
        }
    }
}

/// This frame's raw input, snapshotted before any select runs, for the
/// open/close classification.
#[derive(Clone, Copy, Default)]
struct FrameInput {
    escape: bool,
    tab: bool,
    enter: bool,
    /// Space pressed or released: egui surfaces Space activation on key
    /// release, Enter on key press (the button probe's finding).
    space: bool,
    arrow: bool,
    pointer_clicked: bool,
    any_click: bool,
}

/// Classifies a close transition from this frame's response + input.
fn classify_close(res: &SelectResponse, input: &FrameInput) -> String {
    if res.changed {
        let value = res.value.as_deref().unwrap_or("(none)");
        if input.pointer_clicked {
            format!("closed via item click (selected {value})")
        } else if input.enter || input.space {
            format!("closed via keyboard select (selected {value})")
        } else {
            format!("closed via selection ({value})")
        }
    } else if input.escape {
        "closed via Escape".to_owned()
    } else if input.tab {
        "closed via Tab".to_owned()
    } else if res.trigger.clicked() {
        "closed via trigger toggle".to_owned()
    } else if input.any_click {
        "closed via outside click".to_owned()
    } else {
        "closed (other)".to_owned()
    }
}

/// Classifies an open transition from this frame's response + input.
fn classify_open(res: &SelectResponse, input: &FrameInput) -> String {
    if res.trigger.clicked() && input.pointer_clicked {
        "opened via trigger click".to_owned()
    } else if res.trigger.clicked() && input.enter {
        "opened via Enter (first item)".to_owned()
    } else if res.trigger.clicked() && input.space {
        "opened via Space (selected item)".to_owned()
    } else if res.trigger.clicked() {
        "opened via trigger (keyboard)".to_owned()
    } else if input.arrow {
        "opened via ArrowDown/ArrowUp".to_owned()
    } else {
        "opened (other)".to_owned()
    }
}

/// Folds one frame's response into the probe slot: exact close detection
/// via `closed`, open detection via trigger activation / arrow keys while
/// focused, and closed-trigger typeahead value changes.
fn update_slot(
    probe: &mut SelectProbeState,
    slot: usize,
    res: &SelectResponse,
    input: &FrameInput,
) {
    probe.values[slot] = res.value.clone();
    if res.closed {
        probe.open[slot] = false;
        probe.last[slot] = classify_close(res, input);
    } else if !probe.open[slot]
        && (res.trigger.clicked() || (input.arrow && res.trigger.has_focus()))
    {
        probe.open[slot] = true;
        probe.last[slot] = classify_open(res, input);
    } else if res.changed {
        // Changed while closed: closed-trigger typeahead select (K11).
        probe.last[slot] = format!(
            "value changed via closed-trigger typeahead: {}",
            res.value.as_deref().unwrap_or("(none)")
        );
    }
}

/// Per-section state line, rendered under its trigger.
fn state_line(probe: &SelectProbeState, slot: usize) -> String {
    format!(
        "state: {} — value: {} — last: {}",
        if probe.open[slot] { "open" } else { "closed" },
        probe.values[slot].as_deref().unwrap_or("(none)"),
        probe.last[slot]
    )
}

pub fn select_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("select_demo_probe");
    let mut probe: SelectProbeState = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or_default());
    // Snapshot this frame's input BEFORE showing any select, so the
    // classification sees the raw input for the frame a flip lands on.
    let input = ui.input(|i| FrameInput {
        escape: i.key_pressed(egui::Key::Escape),
        tab: i.key_pressed(egui::Key::Tab),
        enter: i.key_pressed(egui::Key::Enter),
        space: i.key_pressed(egui::Key::Space) || i.key_released(egui::Key::Space),
        arrow: i.key_pressed(egui::Key::ArrowDown) || i.key_pressed(egui::Key::ArrowUp),
        pointer_clicked: i.pointer.primary_clicked(),
        any_click: i.pointer.any_click(),
    });
    // First widget found holding focus this frame wins the readout; only
    // one widget can hold focus at a time, so check order is irrelevant.
    let mut focus_name: Option<String> = None;

    // -- Probe readouts (previous frame's state; settles immediately). --
    ui.heading("Probe readouts");
    ui.add_space(4.0);
    for (i, name) in NAMES.iter().enumerate() {
        ui.label(format!(
            "{name}: {} — value: {} — last: {}",
            if probe.open[i] { "open" } else { "closed" },
            probe.values[i].as_deref().unwrap_or("(none)"),
            probe.last[i]
        ));
    }
    ui.label(format!("Focus: {}", probe.focus));
    ui.label(
        "Highlighted row is not observable from the public API \
         (SelectResponse has no highlight field); S6 reads it visually \
         from the accent fill. Open/closed is inferred from the exact \
         `closed` flag + trigger/arrow transitions.",
    );

    // -- 1. Basic select (spec §§2-3, §7). --
    ui.add_space(16.0);
    ui.heading("Basic");
    ui.add_space(4.0);
    ui.label("Label + items + disabled item + separator + a value override (Other → \"other\").");
    let mut framework = probe.basic.take();
    let res = Select::new(&mut framework)
        .placeholder("Select a framework")
        .id_salt("select_demo_framework")
        .width(200.0)
        .show(ui, |s| {
            s.label("Frameworks");
            s.item(SelectItem::new("Next.js"));
            s.item(SelectItem::new("SvelteKit"));
            s.item(SelectItem::new("Astro").disabled(true));
            s.separator();
            s.item(SelectItem::new("Other").value("other"));
        });
    if res.trigger.has_focus() {
        focus_name = Some("basic trigger".to_owned());
    }
    update_slot(&mut probe, BASIC, &res, &input);
    probe.basic = framework;
    ui.label(state_line(&probe, BASIC));

    // -- 2. Sizes (spec §3.3). --
    ui.add_space(16.0);
    ui.heading("Sizes");
    ui.add_space(4.0);
    ui.label("Heights are recipe-resolved: nova default 32px / sm 28px; base default 36px / sm 32px. Nova sm radius is 8px, default 10px (§3.4).");
    ui.horizontal(|ui| {
        let mut small = std::mem::take(&mut probe.size_sm);
        let res = Select::bind(&mut small)
            .placeholder("Small")
            .size(SelectSize::Sm)
            .width(140.0)
            .id_salt("select_demo_sm")
            .options(["One", "Two", "Three"])
            .show_seeded(ui);
        if res.trigger.has_focus() {
            focus_name = Some("size-sm trigger".to_owned());
        }
        update_slot(&mut probe, SIZE_SM, &res, &input);
        probe.size_sm = small;

        let mut default = std::mem::take(&mut probe.size_default);
        let res = Select::bind(&mut default)
            .placeholder("Default")
            .width(140.0)
            .id_salt("select_demo_default")
            .options(["One", "Two", "Three"])
            .show_seeded(ui);
        if res.trigger.has_focus() {
            focus_name = Some("size-default trigger".to_owned());
        }
        update_slot(&mut probe, SIZE_DEFAULT, &res, &input);
        probe.size_default = default;
    });
    ui.label(state_line(&probe, SIZE_SM));
    ui.label(state_line(&probe, SIZE_DEFAULT));

    // -- 3. Single option (spec §5: content-sized, no empty area). --
    ui.add_space(16.0);
    ui.heading("Single option");
    ui.add_space(4.0);
    ui.label(
        "Exactly one option: the popup must be content-sized with no empty area below the item.",
    );
    let res = Select::uncontrolled()
        .placeholder("One option")
        .id_salt("select_demo_single")
        .width(200.0)
        .options(["Only option"])
        .show_seeded(ui);
    if res.trigger.has_focus() {
        focus_name = Some("single trigger".to_owned());
    }
    update_slot(&mut probe, SINGLE, &res, &input);
    ui.label(state_line(&probe, SINGLE));

    // -- 4. Many options (spec §§5-6, §8: scroll buttons + scroll-follow). --
    ui.add_space(16.0);
    ui.heading("Scrolling (40 options)");
    ui.add_space(4.0);
    ui.label("Tall popup: scroll buttons appear when the list scrolls; arrow keys move the highlight and the highlighted row scrolls into view (OPEN-7).");
    let res = Select::uncontrolled()
        .placeholder("Pick a number")
        .id_salt("select_demo_scroll")
        .width(200.0)
        .options((1..=40).map(|i| format!("Option {i}")))
        .show_seeded(ui);
    if res.trigger.has_focus() {
        focus_name = Some("many trigger".to_owned());
    }
    update_slot(&mut probe, MANY, &res, &input);
    ui.label(state_line(&probe, MANY));

    // -- 5. Groups + labels + separators (spec §2.2, §2.12, §2.13). --
    ui.add_space(16.0);
    ui.heading("Groups");
    ui.add_space(4.0);
    ui.label("Two groups with labels, separated by a separator. Labels are never focusable; under nova each group has 4px padding.");
    let mut grouped = probe.grouped.take();
    let res = Select::new(&mut grouped)
        .placeholder("Select food")
        .id_salt("select_demo_grouped")
        .width(200.0)
        .show(ui, |s| {
            s.group(|g| {
                g.label("Fruits");
                g.item(SelectItem::new("Apple"));
                g.item(SelectItem::new("Banana"));
            });
            s.separator();
            s.group(|g| {
                g.label("Vegetables");
                g.item(SelectItem::new("Carrot"));
                g.item(SelectItem::new("Broccoli"));
            });
        });
    if res.trigger.has_focus() {
        focus_name = Some("grouped trigger".to_owned());
    }
    update_slot(&mut probe, GROUPED, &res, &input);
    probe.grouped = grouped;
    ui.label(state_line(&probe, GROUPED));

    // -- 6. Disabled + invalid (spec §9.4, §9.5). --
    ui.add_space(16.0);
    ui.heading("States");
    ui.add_space(4.0);
    ui.label(
        "Disabled trigger must never open; invalid trigger shows the destructive border/ring.",
    );
    ui.horizontal(|ui| {
        let res = Select::uncontrolled()
            .placeholder("Disabled")
            .disabled(true)
            .id_salt("select_demo_disabled")
            .width(150.0)
            .options(["A", "B"])
            .show_seeded(ui);
        if res.trigger.has_focus() {
            focus_name = Some("disabled trigger".to_owned());
        }
        update_slot(&mut probe, DISABLED, &res, &input);

        let res = Select::uncontrolled()
            .placeholder("Invalid")
            .invalid(true)
            .id_salt("select_demo_invalid")
            .width(150.0)
            .options(["A", "B"])
            .show_seeded(ui);
        if res.trigger.has_focus() {
            focus_name = Some("invalid trigger".to_owned());
        }
        update_slot(&mut probe, INVALID, &res, &input);
    });
    ui.label(state_line(&probe, DISABLED));
    ui.label(state_line(&probe, INVALID));

    // -- 7. Empty select (spec §9.15, OPEN-15). --
    ui.add_space(16.0);
    ui.heading("Empty");
    ui.add_space(4.0);
    ui.label("Zero options: no empty-state element exists in the spec (OPEN-15); the popup renders padding only.");
    let res = Select::uncontrolled()
        .placeholder("No options")
        .id_salt("select_demo_empty")
        .width(200.0)
        .show(ui, |_| {});
    if res.trigger.has_focus() {
        focus_name = Some("empty trigger".to_owned());
    }
    update_slot(&mut probe, EMPTY, &res, &input);
    ui.label(state_line(&probe, EMPTY));

    // -- 8. Keyboard exercise (spec §11). --
    ui.add_space(16.0);
    ui.heading("Keyboard exercise");
    ui.add_space(4.0);
    ui.label(
        "Tab order on the page: Before button, trigger, After button. \
         Prefilled with Cherry (mid-list): Space opens on the selection \
         (K1), Enter opens on the first item (K2). Arrows move the \
         highlight (Durian is disabled — skip behaviour is OPEN-8), \
         Home/End jump, typing jumps by letter, Escape closes keeping \
         focus on the trigger, Tab closes.",
    );
    let mut keyboard = probe.keyboard.take();
    ui.horizontal(|ui| {
        let before = Button::new("Before trigger")
            .variant(ButtonVariant::Outline)
            .show(ui);
        if before.has_focus() {
            focus_name = Some("page: Before button".to_owned());
        }
        let res = Select::new(&mut keyboard)
            .placeholder("Select a fruit")
            .id_salt("select_demo_keyboard")
            .width(200.0)
            .show(ui, |s| {
                s.item(SelectItem::new("Apple"));
                s.item(SelectItem::new("Banana"));
                s.item(SelectItem::new("Cherry"));
                s.item(SelectItem::new("Date"));
                s.item(SelectItem::new("Durian").disabled(true));
                s.item(SelectItem::new("Elderberry"));
                s.item(SelectItem::new("Fig"));
                s.item(SelectItem::new("Grape"));
            });
        if res.trigger.has_focus() {
            focus_name = Some("keyboard trigger".to_owned());
        }
        update_slot(&mut probe, KEYBOARD, &res, &input);
        let after = Button::new("After trigger")
            .variant(ButtonVariant::Outline)
            .show(ui);
        if after.has_focus() {
            focus_name = Some("page: After button".to_owned());
        }
    });
    probe.keyboard = keyboard;
    ui.label(state_line(&probe, KEYBOARD));

    // -- 9. Long text (spec §13-E1, OPEN-17). --
    ui.add_space(16.0);
    ui.heading("Long text");
    ui.add_space(4.0);
    ui.label("Explicit narrow width: the trigger value clamps to one line with ellipsis; item labels clip to the label zone (defined behaviour, OPEN-17).");
    let mut long_text = probe.long_text.take();
    let res = Select::new(&mut long_text)
        .placeholder("A very long placeholder that exceeds the trigger width")
        .id_salt("select_demo_long_text")
        .width(160.0)
        .show(ui, |s| {
            s.item(SelectItem::new(
                "A very long option label that exceeds the trigger width",
            ));
            s.item(SelectItem::new("Short"));
            s.item(SelectItem::new(
                "Another extremely long option label for clipping behaviour",
            ));
        });
    if res.trigger.has_focus() {
        focus_name = Some("long-text trigger".to_owned());
    }
    update_slot(&mut probe, LONG_TEXT, &res, &input);
    probe.long_text = long_text;
    ui.label(state_line(&probe, LONG_TEXT));

    // -- 10. Value not in options (spec §13-E7, OPEN-16). --
    ui.add_space(16.0);
    ui.heading("Value not in options");
    ui.add_space(4.0);
    ui.label("Bound to \"Missing value\", which matches no option: the trigger shows the raw value and no item shows checked (pre-existing behaviour, OPEN-16 unsettled).");
    let mut no_value = probe.no_value.take();
    let res = Select::new(&mut no_value)
        .id_salt("select_demo_no_value")
        .width(200.0)
        .options(["Alpha", "Beta"])
        .show_seeded(ui);
    if res.trigger.has_focus() {
        focus_name = Some("no-value trigger".to_owned());
    }
    update_slot(&mut probe, NO_VALUE, &res, &input);
    probe.no_value = no_value;
    ui.label(state_line(&probe, NO_VALUE));

    // -- Focus readout for next frame. --
    probe.focus = match focus_name {
        Some(name) => name,
        None if probe.open.contains(&true) => "none (a popup is open)".to_owned(),
        None => "none".to_owned(),
    };
    ui.ctx().data_mut(|d| d.insert_temp(store, probe));
}
