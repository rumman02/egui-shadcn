//! Form demo: the docs' bug-report composition — validation on submit,
//! error slots, reset + submitting states.

use egui_shadcn_ui::prelude::*;

#[derive(Clone, Default)]
struct BugReport {
    title: String,
    description: String,
}

fn validate_title(v: &BugReport) -> Option<String> {
    if v.title.chars().count() < 5 {
        Some("Bug title must be at least 5 characters.".to_owned())
    } else if v.title.chars().count() > 32 {
        Some("Bug title must be at most 32 characters.".to_owned())
    } else {
        None
    }
}

fn validate_description(v: &BugReport) -> Option<String> {
    if v.description.chars().count() < 20 {
        Some("Description must be at least 20 characters.".to_owned())
    } else if v.description.chars().count() > 100 {
        Some("Description must be at most 100 characters.".to_owned())
    } else {
        None
    }
}

pub fn run(ui: &mut egui::Ui) {
    let store = egui::Id::unique("form_demo_values");
    #[derive(Clone, Default)]
    struct DemoState {
        value: BugReport,
        submitted_toast: Option<String>,
    }
    let mut state: DemoState = ui
        .ctx()
        .data_mut(|d| d.get_temp::<DemoState>(store).unwrap_or_default());

    ui.label("Bug report (validate-on-submit; Enter in the title submits, Enter in the description inserts a newline)");
    ui.add_space(4.0);

    Card::new().show(ui, |ui| {
        CardHeader::new().show(ui, |ui| {
            CardTitle::new("Bug Report").show(ui);
            CardDescription::new("Help us improve by reporting bugs you encounter.").show(ui);
        });
        let value = &mut state.value;
        let toast = &mut state.submitted_toast;
        Form::new(value)
            .id_salt("bug_report")
            .on_submit(|_ui, v| {
                *toast = Some(format!(
                    "Submitted:\n  title = {}\n  description = {} chars",
                    v.title,
                    v.description.chars().count()
                ));
            })
            .show(ui, |ui, form| {
                CardContent::new().show(ui, |ui| {
                    FieldGroup::new().show(ui, |ui| {
                        form.field(ui, "title", validate_title, |ui, v, ctx| {
                            Field::new().invalid(ctx.invalid).show(ui, |ui| {
                                FieldLabel::new("Bug Title").for_id(ctx.id).show(ui);
                                Input::new(&mut v.title)
                                    .placeholder("Login button not working on mobile")
                                    .invalid(ctx.invalid)
                                    .disabled(ctx.disabled)
                                    .show(ui);
                                if let Some(err) = &ctx.error {
                                    FieldError::new(err.clone()).show(ui);
                                }
                            });
                        });
                        form.field_multiline(
                            ui,
                            "description",
                            validate_description,
                            |ui, v, ctx| {
                                Field::new().invalid(ctx.invalid).show(ui, |ui| {
                                    FieldLabel::new("Description").for_id(ctx.id).show(ui);
                                    Textarea::new(&mut v.description)
                                        .placeholder(
                                            "I'm having an issue with the login button on mobile.",
                                        )
                                        .invalid(ctx.invalid)
                                        .disabled(ctx.disabled)
                                        .show(ui);
                                    FieldDescription::new(format!(
                                        "{} / 100 characters",
                                        v.description.chars().count()
                                    ))
                                    .show(ui);
                                    if let Some(err) = &ctx.error {
                                        FieldError::new(err.clone()).show(ui);
                                    }
                                });
                            },
                        );
                    });
                });
                CardFooter::new().show(ui, |ui| {
                    Field::new()
                        .orientation(FieldOrientation::Horizontal)
                        .show(ui, |ui| {
                            form.reset_errors_button(ui, "Reset");
                            form.submit_button(ui, "Submit");
                        });
                });
            });
    });

    if let Some(toast) = &state.submitted_toast {
        ui.add_space(8.0);
        ui.label(egui::RichText::new(toast).weak());
    }

    ui.add_space(16.0);
    ui.label("Validate-on-change (errors appear while typing)");
    ui.add_space(4.0);
    let mut instant = BugReport {
        title: "abc".into(),
        description: String::new(),
    };
    Form::new(&mut instant)
        .id_salt("instant")
        .validate_on_change(true)
        .show(ui, |ui, form| {
            form.field(ui, "title", validate_title, |ui, v, ctx| {
                Field::new().invalid(ctx.invalid).show(ui, |ui| {
                    FieldLabel::new("Bug Title").for_id(ctx.id).show(ui);
                    Input::new(&mut v.title).invalid(ctx.invalid).show(ui);
                    if let Some(err) = &ctx.error {
                        FieldError::new(err.clone()).show(ui);
                    }
                });
            });
            form.submit_button(ui, "Submit");
        });

    ui.add_space(16.0);
    ui.label("Submitting (button locks and spins)");
    ui.add_space(4.0);
    let mut locked = BugReport {
        title: "Already submitting".into(),
        description: "A long enough description for the validator.".into(),
    };
    Form::new(&mut locked)
        .id_salt("locked")
        .submitting(true)
        .show(ui, |ui, form| {
            form.field(ui, "title", validate_title, |ui, v, ctx| {
                Field::new().show(ui, |ui| {
                    FieldLabel::new("Bug Title").for_id(ctx.id).show(ui);
                    Input::new(&mut v.title).disabled(ctx.disabled).show(ui);
                });
            });
            form.submit_button(ui, "Submit");
        });

    ui.ctx().data_mut(|d| d.insert_temp(store, state));
}
