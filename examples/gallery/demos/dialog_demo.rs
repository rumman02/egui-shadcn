//! Dialog demo: every spec state observable and drivable for S5/S6/S7.
//!
//! Sections: probe readouts (open/closed + last action + focus, all dialogs),
//! basic form dialog, close-X variants (default X / `showCloseButton=false`),
//! footer `showCloseButton`, focus exercise (page widgets before/after the
//! trigger, focusable children inside), disabled close + disabled trigger,
//! long content (no scroll rule invented — spec §12), custom width, empty
//! footer (nova strip), and a controlled dialog with an external toggle.
//!
//! All dialogs run in controlled mode so the readouts report the exact open
//! state. The "last action" classification is inferred per frame from the
//! trigger/close responses and that frame's input, like the button probe:
//! Escape, footer/close buttons, the trigger, and the external toggle are
//! distinguished exactly; a pointer close that matches none of those is
//! reported honestly as "backdrop or X" (the X is painted inside
//! `DialogContent`, so the demo cannot tell it apart from a backdrop click
//! — S6 drives the two apart with targeted clicks).
//!
//! The header/footer responsive rules (§4) key off the WINDOW width at the
//! 640px breakpoint (deviation D4): resize the gallery window below 640px
//! to see the centered header and the stacked reversed footer. The readout
//! block shows the live window width and which side of the breakpoint the
//! gallery is on.

use egui_shadcn_ui::prelude::*;

/// One slot per demo dialog, in page order.
const BASIC: usize = 0;
const NO_X: usize = 1;
const FOOTER_CLOSE: usize = 2;
const FOCUS: usize = 3;
const DISABLED_CLOSE: usize = 4;
const DISABLED_TRIGGER: usize = 5;
const LONG: usize = 6;
const WIDE: usize = 7;
const EMPTY_FOOTER: usize = 8;
const EXTERNAL: usize = 9;
const N_DIALOGS: usize = 10;

const NAMES: [&str; N_DIALOGS] = [
    "basic",
    "no-x",
    "footer-close",
    "focus",
    "disabled-close",
    "disabled-trigger",
    "long",
    "wide",
    "empty-footer",
    "external",
];

/// Demo-local probe state, held in egui temp data (the gallery's established
/// demo-state pattern, as in the button probe).
#[derive(Clone)]
struct DialogProbeState {
    /// Controlled open flag per dialog (written back every frame).
    open: [bool; N_DIALOGS],
    /// Last classified open/close action per dialog.
    last: [String; N_DIALOGS],
    /// Widget holding keyboard focus, as named last frame (one frame stale).
    focus: String,
    name_basic: String,
    email_basic: String,
    name_focus: String,
    agree_focus: bool,
}

impl Default for DialogProbeState {
    fn default() -> Self {
        Self {
            open: [false; N_DIALOGS],
            last: std::array::from_fn(|_| "never opened".to_owned()),
            focus: "none".to_owned(),
            name_basic: String::new(),
            email_basic: String::new(),
            name_focus: String::new(),
            agree_focus: false,
        }
    }
}

/// Classifies an open-state flip from this frame's events. Returns `None`
/// when the state did not flip. `pointer_label` names the pointer-close
/// mechanism honestly per dialog (e.g. the no-X dialog can only close via
/// the backdrop; the footer-close dialog can also close via its appended
/// Close button, which the demo cannot observe separately).
#[allow(clippy::too_many_arguments)]
fn classify(
    prev_open: bool,
    open: bool,
    trigger_changed: bool,
    close_changed: bool,
    external_clicked: bool,
    escape_pressed: bool,
    pointer_clicked: bool,
    pointer_label: &str,
) -> Option<String> {
    if prev_open == open {
        return None;
    }
    if open {
        if external_clicked {
            Some("opened via external toggle".to_owned())
        } else if trigger_changed {
            Some("opened via trigger".to_owned())
        } else {
            Some("opened (other)".to_owned())
        }
    } else if external_clicked {
        Some("closed via external toggle".to_owned())
    } else if escape_pressed {
        Some("closed via Escape".to_owned())
    } else if close_changed {
        Some("closed via footer/close button".to_owned())
    } else if trigger_changed {
        Some("closed via trigger".to_owned())
    } else if pointer_clicked {
        Some(format!("closed via pointer ({pointer_label})"))
    } else {
        Some("closed (other)".to_owned())
    }
}

pub fn dialog_demo(ui: &mut egui::Ui) {
    let store = egui::Id::unique("dialog_demo_probe");
    let mut probe: DialogProbeState = ui.ctx().data_mut(|d| d.get_temp(store).unwrap_or_default());
    // Snapshot this frame's input BEFORE showing any dialog, so the
    // classification sees the raw input for the frame a flip lands on.
    let (escape_pressed, pointer_clicked) = ui.input(|i| {
        (
            i.key_pressed(egui::Key::Escape),
            i.pointer.primary_clicked(),
        )
    });
    // First widget found holding focus this frame wins the readout; only
    // one widget can hold focus at a time, so check order is irrelevant.
    let mut focus_name: Option<String> = None;

    // -- Probe readouts (previous frame's state; settles immediately). --
    ui.heading("Probe readouts");
    ui.add_space(4.0);
    let win_w = ui.ctx().content_rect().width();
    let rules = if win_w >= 640.0 {
        "wide (sm: rules: left header, footer row)"
    } else {
        "narrow (below sm: centered header, footer stacks reversed)"
    };
    ui.label(format!("window width: {win_w:.0}px — {rules}"));
    for (i, name) in NAMES.iter().enumerate() {
        let state = if probe.open[i] { "open" } else { "closed" };
        ui.label(format!("{name}: {state} — last: {}", probe.last[i]));
    }
    ui.label(format!("Focus: {}", probe.focus));
    ui.label(
        "Pointer closes report \"backdrop or X\": the demo cannot tell the \
         corner X apart from a backdrop click (both are pointer closes \
         inside DialogContent). S6 drives them apart with targeted clicks.",
    );

    // -- 1. Basic form dialog (spec §§1-5). --
    ui.add_space(16.0);
    ui.heading("Basic form dialog");
    ui.add_space(4.0);
    let mut open = probe.open[BASIC];
    let prev = open;
    let mut trigger_changed = false;
    let mut close_changed = false;
    let mut name = std::mem::take(&mut probe.name_basic);
    let mut email = std::mem::take(&mut probe.email_basic);
    Dialog::controlled(&mut open)
        .id_salt("demo_basic")
        .show(ui, |ui| {
            let trig = DialogTrigger::widget(|ui| Button::new("Edit profile").show(ui)).show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("basic trigger (Edit profile)".to_owned());
            }
            DialogContent::new().show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new("Edit profile").show(ui);
                    DialogDescription::new(
                        "Make changes to your profile here. Click save when you're done.",
                    )
                    .show(ui);
                });
                ui.label("Name");
                ui.add(egui::TextEdit::singleline(&mut name).desired_width(f32::INFINITY));
                ui.label("Email");
                ui.add(
                    egui::TextEdit::singleline(&mut email)
                        .hint_text("you@example.com")
                        .desired_width(f32::INFINITY),
                );
                DialogFooter::new().show(ui, |ui| {
                    let cancel = DialogClose::widget(|ui| {
                        Button::new("Cancel")
                            .variant(ButtonVariant::Outline)
                            .show(ui)
                    })
                    .show(ui);
                    close_changed |= cancel.changed();
                    let save =
                        DialogClose::widget(|ui| Button::new("Save changes").show(ui)).show(ui);
                    close_changed |= save.changed();
                });
            });
        });
    probe.name_basic = name;
    probe.email_basic = email;
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        close_changed,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop or X",
    ) {
        probe.last[BASIC] = action;
    }
    probe.open[BASIC] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[BASIC]
    ));

    // -- 2. Close X variants (spec §7, §12). --
    ui.add_space(16.0);
    ui.heading("Without close button (showCloseButton = false)");
    ui.add_space(4.0);
    ui.label("No corner X renders; Escape, backdrop click, and the footer button remain.");
    let mut open = probe.open[NO_X];
    let prev = open;
    let mut trigger_changed = false;
    let mut close_changed = false;
    Dialog::controlled(&mut open)
        .id_salt("demo_noclose")
        .show(ui, |ui| {
            let trig = DialogTrigger::widget(|ui| {
                Button::new("No X button")
                    .variant(ButtonVariant::Outline)
                    .show(ui)
            })
            .show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("no-x trigger".to_owned());
            }
            DialogContent::new()
                .show_close_button(false)
                .show(ui, |ui| {
                    DialogHeader::new().show(ui, |ui| {
                        DialogTitle::new("Dismiss with Escape or backdrop").show(ui);
                        DialogDescription::new("showCloseButton = false hides the X.").show(ui);
                    });
                    DialogFooter::new().show(ui, |ui| {
                        let got_it =
                            DialogClose::widget(|ui| Button::new("Got it").show(ui)).show(ui);
                        close_changed |= got_it.changed();
                    });
                });
        });
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        close_changed,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop (no X)",
    ) {
        probe.last[NO_X] = action;
    }
    probe.open[NO_X] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[NO_X]
    ));

    // -- 3. Footer showCloseButton (spec §4). --
    ui.add_space(16.0);
    ui.heading("Footer close button");
    ui.add_space(4.0);
    ui.label(
        "The footer appends an outline Close button after its children: \
         rightmost in the row (wide), topmost in the stack (narrow).",
    );
    let mut open = probe.open[FOOTER_CLOSE];
    let prev = open;
    let mut trigger_changed = false;
    let mut close_changed = false;
    Dialog::controlled(&mut open)
        .id_salt("demo_footer_close")
        .show(ui, |ui| {
            let trig =
                DialogTrigger::widget(|ui| Button::new("Footer showCloseButton").show(ui)).show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("footer-close trigger".to_owned());
            }
            DialogContent::new().show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new("Confirm action").show(ui);
                    DialogDescription::new(
                        "The footer appends an outline Close button \
                         (showCloseButton = true).",
                    )
                    .show(ui);
                });
                DialogFooter::new().show_close_button(true).show(ui, |ui| {
                    let cont = DialogClose::widget(|ui| Button::new("Continue").show(ui)).show(ui);
                    close_changed |= cont.changed();
                });
            });
        });
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        close_changed,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop, X, or appended footer Close",
    ) {
        probe.last[FOOTER_CLOSE] = action;
    }
    probe.open[FOOTER_CLOSE] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[FOOTER_CLOSE]
    ));

    // -- 4. Focus exercise (spec §10). --
    ui.add_space(16.0);
    ui.heading("Focus exercise");
    ui.add_space(4.0);
    ui.label(
        "Tab order on the page: Before button, trigger, After button. \
         Inside: name field, agree checkbox, Cancel, Confirm. Open the \
         dialog, Tab through it (focus must stay inside), close it (focus \
         must return to the trigger).",
    );
    let mut open = probe.open[FOCUS];
    let prev = open;
    let mut trigger_changed = false;
    let mut close_changed = false;
    let mut name = std::mem::take(&mut probe.name_focus);
    let mut agree = probe.agree_focus;
    ui.horizontal(|ui| {
        let before = Button::new("Before trigger")
            .variant(ButtonVariant::Outline)
            .show(ui);
        if before.has_focus() {
            focus_name = Some("page: Before button".to_owned());
        }
        Dialog::controlled(&mut open)
            .id_salt("demo_focus")
            .show(ui, |ui| {
                let trig =
                    DialogTrigger::widget(|ui| Button::new("Focus exercise").show(ui)).show(ui);
                trigger_changed = trig.changed();
                if trig.has_focus() {
                    focus_name = Some("focus trigger".to_owned());
                }
                DialogContent::new().show(ui, |ui| {
                    DialogHeader::new().show(ui, |ui| {
                        DialogTitle::new("Focus trap").show(ui);
                        DialogDescription::new(
                            "Tab cycles through the field, checkbox, and \
                             footer buttons; focus cannot leave while open.",
                        )
                        .show(ui);
                    });
                    ui.label("Name");
                    let name_resp =
                        ui.add(egui::TextEdit::singleline(&mut name).desired_width(f32::INFINITY));
                    if name_resp.has_focus() {
                        focus_name = Some("inside: name field".to_owned());
                    }
                    let agree_resp = Checkbox::new(&mut agree).show(ui);
                    if agree_resp.has_focus() {
                        focus_name = Some("inside: agree checkbox".to_owned());
                    }
                    ui.label("I agree to the terms");
                    DialogFooter::new().show(ui, |ui| {
                        let cancel = DialogClose::widget(|ui| {
                            Button::new("Cancel")
                                .variant(ButtonVariant::Outline)
                                .show(ui)
                        })
                        .show(ui);
                        if cancel.has_focus() {
                            focus_name = Some("inside: Cancel".to_owned());
                        }
                        close_changed |= cancel.changed();
                        let confirm =
                            DialogClose::widget(|ui| Button::new("Confirm").show(ui)).show(ui);
                        if confirm.has_focus() {
                            focus_name = Some("inside: Confirm".to_owned());
                        }
                        close_changed |= confirm.changed();
                    });
                });
            });
        let after = Button::new("After trigger")
            .variant(ButtonVariant::Outline)
            .show(ui);
        if after.has_focus() {
            focus_name = Some("page: After button".to_owned());
        }
    });
    probe.name_focus = name;
    probe.agree_focus = agree;
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        close_changed,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop or X",
    ) {
        probe.last[FOCUS] = action;
    }
    probe.open[FOCUS] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[FOCUS]
    ));

    // -- 5. Disabled close + disabled trigger (spec §7, §8, §11). --
    ui.add_space(16.0);
    ui.heading("Disabled close");
    ui.add_space(4.0);
    ui.label(
        "The footer Delete button is a disabled DialogClose: clicking it \
         must not close the dialog. Cancel still closes.",
    );
    let mut open = probe.open[DISABLED_CLOSE];
    let prev = open;
    let mut trigger_changed = false;
    let mut close_changed = false;
    Dialog::controlled(&mut open)
        .id_salt("demo_disabled_close")
        .show(ui, |ui| {
            let trig = DialogTrigger::widget(|ui| Button::new("Disabled close").show(ui)).show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("disabled-close trigger".to_owned());
            }
            DialogContent::new().show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new("Delete item?").show(ui);
                    DialogDescription::new("The Delete close control is disabled.").show(ui);
                });
                DialogFooter::new().show(ui, |ui| {
                    let delete = DialogClose::widget(|ui| {
                        Button::new("Delete")
                            .variant(ButtonVariant::Destructive)
                            .disabled(true)
                            .show(ui)
                    })
                    .disabled(true)
                    .show(ui);
                    close_changed |= delete.changed();
                    let cancel = DialogClose::widget(|ui| {
                        Button::new("Cancel")
                            .variant(ButtonVariant::Outline)
                            .show(ui)
                    })
                    .show(ui);
                    close_changed |= cancel.changed();
                });
            });
        });
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        close_changed,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop or X",
    ) {
        probe.last[DISABLED_CLOSE] = action;
    }
    probe.open[DISABLED_CLOSE] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[DISABLED_CLOSE]
    ));

    ui.add_space(8.0);
    ui.label("Disabled trigger (must never open):");
    let mut open = probe.open[DISABLED_TRIGGER];
    let prev = open;
    let mut trigger_changed = false;
    Dialog::controlled(&mut open)
        .id_salt("demo_disabled_trigger")
        .show(ui, |ui| {
            let trig =
                DialogTrigger::widget(|ui| Button::new("Disabled trigger").disabled(true).show(ui))
                    .disabled(true)
                    .show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("disabled trigger".to_owned());
            }
            DialogContent::new().show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new("Must never render").show(ui);
                    DialogDescription::new("A disabled trigger does not open the dialog.").show(ui);
                });
            });
        });
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        false,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop or X",
    ) {
        probe.last[DISABLED_TRIGGER] = action;
    }
    probe.open[DISABLED_TRIGGER] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[DISABLED_TRIGGER]
    ));

    // -- 6. Long content + long title (spec §3, §12). --
    ui.add_space(16.0);
    ui.heading("Long content");
    ui.add_space(4.0);
    ui.label(
        "No max-height or scroll rule is specified: the panel is \
         content-determined and the title wraps. Forty body lines overflow \
         an 800px viewport by design.",
    );
    let mut open = probe.open[LONG];
    let prev = open;
    let mut trigger_changed = false;
    let mut close_changed = false;
    Dialog::controlled(&mut open)
        .id_salt("demo_long")
        .show(ui, |ui| {
            let trig = DialogTrigger::widget(|ui| Button::new("Long content").show(ui)).show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("long trigger".to_owned());
            }
            DialogContent::new().show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new(
                        "A deliberately long dialog title that wraps \
                         across multiple lines within the header stack",
                    )
                    .show(ui);
                    DialogDescription::new("Forty body lines follow.").show(ui);
                });
                for i in 1..=40 {
                    ui.label(format!("Body line {i} of 40."));
                }
                DialogFooter::new().show(ui, |ui| {
                    let close = DialogClose::widget(|ui| Button::new("Close").show(ui)).show(ui);
                    close_changed |= close.changed();
                });
            });
        });
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        close_changed,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop or X",
    ) {
        probe.last[LONG] = action;
    }
    probe.open[LONG] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[LONG]
    ));

    // -- 7. Custom width (spec §11: the className max-width equivalent). --
    ui.add_space(16.0);
    ui.heading("Custom width");
    ui.add_space(4.0);
    let mut open = probe.open[WIDE];
    let prev = open;
    let mut trigger_changed = false;
    Dialog::controlled(&mut open)
        .id_salt("demo_wide")
        .show(ui, |ui| {
            let trig = DialogTrigger::widget(|ui| Button::new("Wide (560px)").show(ui)).show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("wide trigger".to_owned());
            }
            DialogContent::new().width(560.0).show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new("Wide dialog").show(ui);
                    DialogDescription::new(
                        "Content wider than the sm:max-w-sm default; the panel \
                         is clamped to the window minus 2rem.",
                    )
                    .show(ui);
                });
            });
        });
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        false,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop or X",
    ) {
        probe.last[WIDE] = action;
    }
    probe.open[WIDE] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[WIDE]
    ));

    // -- 8. Empty footer (spec §12: the nova strip renders regardless). --
    ui.add_space(16.0);
    ui.heading("Empty footer");
    ui.add_space(4.0);
    ui.label("The footer has no children; under nova its strip still renders.");
    let mut open = probe.open[EMPTY_FOOTER];
    let prev = open;
    let mut trigger_changed = false;
    Dialog::controlled(&mut open)
        .id_salt("demo_empty_footer")
        .show(ui, |ui| {
            let trig = DialogTrigger::widget(|ui| Button::new("Empty footer").show(ui)).show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("empty-footer trigger".to_owned());
            }
            DialogContent::new().show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new("No footer content").show(ui);
                    DialogDescription::new("Only the header and an empty footer.").show(ui);
                });
                DialogFooter::new().show(ui, |_| {});
            });
        });
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        false,
        false,
        escape_pressed,
        pointer_clicked,
        "backdrop or X",
    ) {
        probe.last[EMPTY_FOOTER] = action;
    }
    probe.open[EMPTY_FOOTER] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[EMPTY_FOOTER]
    ));

    // -- 9. Controlled + external toggle (spec §11: caller-owned state). --
    ui.add_space(16.0);
    ui.heading("Controlled");
    ui.add_space(4.0);
    ui.label("The open state is owned by the caller; the toggle flips it directly.");
    let mut open = probe.open[EXTERNAL];
    let prev = open;
    let mut trigger_changed = false;
    Dialog::controlled(&mut open)
        .id_salt("demo_controlled")
        .show(ui, |ui| {
            let trig = DialogTrigger::widget(|ui| Button::new("Open controlled").show(ui)).show(ui);
            trigger_changed = trig.changed();
            if trig.has_focus() {
                focus_name = Some("controlled trigger".to_owned());
            }
            DialogContent::new().show(ui, |ui| {
                DialogHeader::new().show(ui, |ui| {
                    DialogTitle::new("Controlled dialog").show(ui);
                    DialogDescription::new(
                        "The open state is owned by the caller. Escape, \
                         backdrop click and the X button all write back.",
                    )
                    .show(ui);
                });
            });
        });
    let mut external_clicked = false;
    ui.horizontal(|ui| {
        if Button::new(if open {
            "toggle: open"
        } else {
            "toggle: closed"
        })
        .variant(ButtonVariant::Ghost)
        .show(ui)
        .clicked()
        {
            open = !open;
            external_clicked = true;
        }
        ui.label("External toggle (flips the caller's open flag directly).");
    });
    if let Some(action) = classify(
        prev,
        open,
        trigger_changed,
        false,
        external_clicked,
        escape_pressed,
        pointer_clicked,
        "backdrop or X",
    ) {
        probe.last[EXTERNAL] = action;
    }
    probe.open[EXTERNAL] = open;
    ui.label(format!(
        "state: {} — last: {}",
        if open { "open" } else { "closed" },
        probe.last[EXTERNAL]
    ));

    // -- Focus readout for next frame. --
    probe.focus = match focus_name {
        Some(name) => name,
        None if probe.open.contains(&true) => {
            "dialog internals (panel / close X / backdrop)".to_owned()
        }
        None => "none".to_owned(),
    };
    ui.ctx().data_mut(|d| d.insert_temp(store, probe));
}
