//! shadcn `form.tsx` — the form provider composition
//! (`Form` / `FormField` / `FormItem` / `FormLabel` / `FormControl` /
//! `FormDescription` / `FormMessage`) ported as a closure-based
//! [`Form`] builder over the crate's [`crate::components::field`] slots.
//!
//! The React version delegates to react-hook-form / TanStack Form; here the
//! form owns a `&mut T` value and per-field validation closures returning
//! `Option<String>` errors:
//!
//! - Errors are only *displayed* once the form has been submitted once
//!   (validate-on-submit, like `isTouched && !isValid`), or from the first
//!   frame with `.validate_on_change(true)`.
//! - [`FormUi::field`] renders a field through a closure receiving the form
//!   value and a [`FormFieldCtx`] (`invalid`, `error`, `id`, `disabled`) so
//!   the content can wire `aria-invalid` styling and the `FieldError` slot.
//! - [`FormUi::submit_button`] renders the submit button and records the
//!   submit intent; on a valid submit the `.on_submit` callback runs.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! #[derive(Default)]
//! struct Signup { email: String }
//!
//! let mut value = Signup::default();
//! let res = Form::new(&mut value)
//!     .id_salt("signup")
//!     .on_submit(|ui, v| { ui.label(format!("submitted {}", v.email)); })
//!     .show(ui, |ui, form| {
//!         form.field(ui, "email", |v| {
//!             if v.email.contains('@') { None } else { Some("Enter a valid email.".into()) }
//!         }, |ui, v, ctx| {
//!             Field::new().invalid(ctx.invalid).show(ui, |ui| {
//!                 FieldLabel::new("Email").for_id(ctx.id).show(ui);
//!                 Input::new(&mut v.email).invalid(ctx.invalid).show(ui);
//!                 if let Some(err) = &ctx.error {
//!                     FieldError::new(err.clone()).show(ui);
//!                 }
//!             });
//!         });
//!         form.submit_button(ui, "Submit");
//!     });
//! // res.valid / res.errors / res.submitted
//! ```

use egui::{Id, Response, Ui};

/// Context handed to each field's render closure, mirroring
/// `useFormField()` in the source (`error`, `invalid`, ids, disabled).
pub struct FormFieldCtx {
    /// Stable id for this field (`form_item`), for `FieldLabel::for_id`.
    pub id: Id,
    /// True when the field currently shows an error.
    pub invalid: bool,
    /// The validation error, when errors are being displayed.
    pub error: Option<String>,
    /// The form's disabled flag.
    pub disabled: bool,
}

/// A collected `(field name, error message)` pair.
pub type FormFieldError = (String, String);

/// The submit/validate closure type.
pub type FormValidator<'a, T> = Box<dyn Fn(&T) -> Option<String> + 'a>;

/// The validators list type (name + closure pairs).
pub type ValidatorList<'a, T> = Vec<(String, FormValidator<'a, T>)>;

/// Borrowed validator pairs for the submit re-validation pass.
pub type ValidatorPairs<'a, 'b, T> = &'b [(String, &'a dyn Fn(&T) -> Option<String>)];

/// One borrowed `(name, validator)` pair.
pub type ValidatorPair<'a, T> = (String, &'a dyn Fn(&T) -> Option<String>);

/// The on-submit callback type.
pub type OnSubmit<'a, T> = Box<dyn FnOnce(&mut Ui, &mut T) + 'a>;

/// Run all validators against the value, collecting errors, unit-testable.
pub(crate) fn run_validators<T>(
    validators: ValidatorPairs<'_, '_, T>,
    value: &T,
) -> Vec<FormFieldError> {
    validators
        .iter()
        .filter_map(|(name, validate)| validate(value).map(|error| (name.clone(), error)))
        .collect()
}

/// Whether errors should be displayed: after the first submit attempt, or
/// always with validate-on-change, unit-testable.
pub(crate) fn should_show_errors(touched: bool, validate_on_change: bool) -> bool {
    touched || validate_on_change
}

/// Response of [`Form::show`].
#[derive(Clone, Debug, Default)]
pub struct FormResponse {
    /// True when a submit was attempted this frame.
    pub submitted: bool,
    /// True when the last submit attempt (this frame) passed validation.
    pub valid: bool,
    /// Errors from the last submit attempt (or the live set with
    /// validate-on-change).
    pub errors: Vec<FormFieldError>,
}

/// The form. See the [module docs](self) for an example.
pub struct Form<'a, T> {
    value: &'a mut T,
    id_salt: Option<String>,
    validate_on_change: bool,
    disabled: bool,
    submitting: bool,
    on_submit: Option<OnSubmit<'a, T>>,
}

impl<'a, T> Form<'a, T> {
    /// Creates a form over the caller's value.
    pub fn new(value: &'a mut T) -> Self {
        Self {
            value,
            id_salt: None,
            validate_on_change: false,
            disabled: false,
            submitting: false,
            on_submit: None,
        }
    }

    /// Sets a stable id salt (the "touched" flag + field ids persist under
    /// it; give multiple forms distinct salts).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Validates from the first frame instead of only after a submit
    /// attempt (default false, like the source's validate-on-submit).
    pub fn validate_on_change(mut self, on: bool) -> Self {
        self.validate_on_change = on;
        self
    }

    /// Disables every field and the submit button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Shows the submitting state: the submit button spins and locks.
    pub fn submitting(mut self, submitting: bool) -> Self {
        self.submitting = submitting;
        self
    }

    /// The callback run when a submit passes validation.
    pub fn on_submit(mut self, f: impl FnOnce(&mut Ui, &mut T) + 'a) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    /// Renders the form and evaluates the submit/validation cycle.
    pub fn show(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui, &mut FormUi<'a, T>),
    ) -> FormResponse {
        let Self {
            value,
            id_salt,
            validate_on_change,
            disabled,
            submitting,
            on_submit,
        } = self;

        let form_id = match id_salt.as_deref() {
            Some(salt) => Id::unique(format!("shadcn_form_{salt}")),
            None => ui.auto_id_with("shadcn_form"),
        };
        let touched_before: bool = ui
            .ctx()
            .data_mut(|d| d.get_temp(form_id.with("touched")))
            .unwrap_or(false);
        let show_errors = should_show_errors(touched_before, validate_on_change);

        let mut form = FormUi {
            form_id,
            value,
            validators: Vec::new(),
            errors: Vec::new(),
            submit_requested: false,
            show_errors,
            disabled,
            submitting,
            field_seq: 0,
        };

        content(ui, &mut form);

        // Submit pass: re-validate everything against the post-edit value.
        let mut response = FormResponse::default();
        if form.submit_requested {
            ui.ctx()
                .data_mut(|d| d.insert_temp(form_id.with("touched"), true));
            response.submitted = true;
            // Re-run the validators registered while rendering.
            let pairs: std::vec::Vec<ValidatorPair<T>> = form
                .validators
                .iter()
                .map(|(name, v)| (name.clone(), v.as_ref()))
                .collect();
            let errors = run_validators(&pairs, form.value);
            response.valid = errors.is_empty();
            response.errors = errors;
            if response.valid {
                if let Some(on_submit) = on_submit {
                    on_submit(ui, form.value);
                }
            }
        } else {
            // Live error set for validate-on-change forms.
            response.errors = form.errors.clone();
            response.valid = form.errors.is_empty();
        }

        response
    }
}

/// The per-frame form handle passed to the [`Form::show`] closure.
pub struct FormUi<'a, T> {
    form_id: Id,
    value: &'a mut T,
    validators: ValidatorList<'a, T>,
    errors: Vec<FormFieldError>,
    submit_requested: bool,
    show_errors: bool,
    disabled: bool,
    submitting: bool,
    field_seq: usize,
}

impl<'a, T> FormUi<'a, T> {
    /// Registers and renders one field.
    ///
    /// `validate` extracts an error from the form value (`None` = valid);
    /// `render` draws the field content, receiving the form value and the
    /// [`FormFieldCtx`]. Errors show once the form has been submitted (or
    /// immediately with validate-on-change), matching the source's
    /// `isTouched && !isValid` gating.
    pub fn field<R>(
        &mut self,
        ui: &mut Ui,
        name: impl Into<String>,
        validate: impl Fn(&T) -> Option<String> + 'a,
        render: impl FnOnce(&mut Ui, &mut T, &FormFieldCtx) -> R,
    ) -> R {
        let name = name.into();
        self.validators.push((name.clone(), Box::new(validate)));

        let error = if self.show_errors {
            self.validators.last().and_then(|(_, v)| v(self.value))
        } else {
            None
        };
        // Accumulate the live error set (validate-on-change + post-submit
        // frames read this when no submit happened this frame).
        if let Some(error) = &error {
            self.errors.push((name.clone(), error.clone()));
        }

        let seq = self.field_seq;
        self.field_seq += 1;
        let ctx = FormFieldCtx {
            id: self.form_id.with(("field", seq)).with(name.as_str()),
            invalid: error.is_some(),
            error,
            disabled: self.disabled,
        };
        let value = &mut *self.value;
        render(ui, value, &ctx)
    }

    /// Renders the submit button (primary; spinner + disabled while
    /// submitting, disabled with the form). Clicking it requests the submit
    /// pass at the end of [`Form::show`].
    pub fn submit_button(&mut self, ui: &mut Ui, label: impl Into<String>) -> Response {
        let mut button = crate::components::button::Button::new(label);
        if self.submitting {
            button = button.icon_start(crate::icons::Icon::LoaderCircle);
        }
        let response = button.disabled(self.disabled || self.submitting).show(ui);
        if response.clicked() {
            self.submit_requested = true;
        }
        response
    }

    /// An outline reset-style button that clears the error display state
    /// (the source's `form.reset()` for errors; the value itself stays
    /// caller-owned). Returns the button response.
    pub fn reset_errors_button(&mut self, ui: &mut Ui, label: impl Into<String>) -> Response {
        let response = crate::components::button::Button::new(label)
            .variant(crate::components::button::ButtonVariant::Outline)
            .disabled(self.disabled || self.submitting)
            .show(ui);
        if response.clicked() {
            ui.ctx()
                .data_mut(|d| d.insert_temp(self.form_id.with("touched"), false));
        }
        response
    }

    /// The form's disabled flag.
    pub fn disabled(&self) -> bool {
        self.disabled
    }

    /// The form's submitting flag.
    pub fn submitting(&self) -> bool {
        self.submitting
    }

    /// Requests submit programmatically (e.g. from a custom button).
    pub fn request_submit(&mut self) {
        self.submit_requested = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Vec2};
    use std::cell::Cell;
    use std::rc::Rc;

    #[derive(Default, Debug)]
    struct Signup {
        email: String,
    }

    fn email_validator(v: &Signup) -> Option<String> {
        if v.email.contains('@') {
            None
        } else {
            Some("Enter a valid email.".to_owned())
        }
    }

    fn run_validators_collects_named_errors_check() {
        let value = Signup::default();
        let validators: std::vec::Vec<ValidatorPair<Signup>> =
            vec![("email".into(), &email_validator)];
        let errors = run_validators(&validators, &value);
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].0, "email");
        assert_eq!(errors[0].1, "Enter a valid email.");

        let value = Signup { email: "a@b.c".into() };
        assert!(run_validators(&validators, &value).is_empty());
    }

    #[test]
    fn run_validators_collects_named_errors() {
        run_validators_collects_named_errors_check();
    }

    #[test]
    fn errors_are_gated_until_touched() {
        assert!(!should_show_errors(false, false));
        assert!(should_show_errors(true, false));
        assert!(should_show_errors(false, true));
    }

    /// Test harness: fonts need a real pass, clicks need press+release frames.
    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    #[derive(Clone, Copy, Default)]
    struct Cfg {
        submit_button: bool,
        reset_button: bool,
        validate_on_change: bool,
        disabled: bool,
        submitting: bool,
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            Self { ctx, time: 0.0 }
        }

        fn run(
            &mut self,
            value: &mut Signup,
            cfg: Cfg,
            press: Option<(Pos2, bool)>,
            on_submit: Option<Rc<Cell<bool>>>,
        ) -> FormResponse {
            self.time += 0.1;
            let mut input = egui::RawInput {
                time: Some(self.time),
                ..Default::default()
            };
            if let Some((pos, is_pressed)) = press {
                input.events.push(egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: is_pressed,
                    modifiers: Default::default(),
                });
            }
            let mut res = FormResponse::default();
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("form_test_ui"),
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(420.0, 420.0),
                    )),
                );
                let mut form = Form::new(value).id_salt("harness");
                if cfg.validate_on_change {
                    form = form.validate_on_change(true);
                }
                if cfg.disabled {
                    form = form.disabled(true);
                }
                if cfg.submitting {
                    form = form.submitting(true);
                }
                if let Some(flag) = on_submit.clone() {
                    form = form.on_submit(move |_ui, _v| flag.set(true));
                }
                res = form.show(ui, |ui, form| {
                    form.field(ui, "email", email_validator, |_ui, _v, _ctx| {});
                    if cfg.submit_button {
                        let r = form.submit_button(ui, "Submit");
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(Id::unique("h_submit_rect"), r.rect);
                        });
                    }
                    if cfg.reset_button {
                        let r = form.reset_errors_button(ui, "Reset");
                        ui.ctx().data_mut(|d| {
                            d.insert_temp(Id::unique("h_reset_rect"), r.rect);
                        });
                    }
                });
            });
            out.textures_delta.clear();
            res
        }

        /// A full press + release at `pos` across two passes.
        fn click(
            &mut self,
            value: &mut Signup,
            pos: Pos2,
            cfg: Cfg,
            on_submit: Option<Rc<Cell<bool>>>,
        ) -> FormResponse {
            let _ = self.run(value, cfg, Some((pos, true)), on_submit.clone());
            self.run(value, cfg, Some((pos, false)), on_submit)
        }
    }

    #[test]
    fn no_errors_before_submit_without_validate_on_change() {
        let h = Harness::new();
        let mut value = Signup::default();
        let mut seen_error = None;
        let mut out = h.ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("form_gate_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(420.0, 420.0),
                )),
            );
            let res = Form::new(&mut value).id_salt("gate").show(ui, |ui, form| {
                form.field(ui, "email", email_validator, |_ui, _v, ctx| {
                    seen_error = ctx.error.clone();
                });
            });
            assert!(!res.submitted);
            assert!(res.errors.is_empty());
        });
        out.textures_delta.clear();
        assert!(seen_error.is_none());
    }

    #[test]
    fn validate_on_change_shows_errors_immediately() {
        let mut h = Harness::new();
        let mut value = Signup::default();
        let res = h.run(
            &mut value,
            Cfg {
                validate_on_change: true,
                ..Cfg::default()
            },
            None,
            None,
        );
        assert!(!res.submitted);
        assert_eq!(res.errors.len(), 1);
        assert!(!res.valid);
    }

    #[test]
    fn invalid_submit_reports_errors_and_runs_no_callback() {
        let h = Harness::new();
        let mut value = Signup::default();
        let called = Rc::new(Cell::new(false));

        // Programmatic submit via request_submit (single frame).
        let mut res = FormResponse::default();
        let res = {
            let mut out = h.ctx.run_ui(egui::RawInput::default(), |ctx| {
                let called2 = called.clone();
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("form_invalid_ui"),
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(420.0, 420.0),
                    )),
                );
                res = Form::new(&mut value)
                    .id_salt("invalid")
                    .on_submit(move |_ui, _v| called2.set(true))
                    .show(ui, |ui, form| {
                        form.field(ui, "email", email_validator, |_ui, _v, _ctx| {});
                        form.request_submit();
                    });
            });
            out.textures_delta.clear();
            res
        };
        assert!(res.submitted);
        assert!(!res.valid);
        assert_eq!(res.errors.len(), 1);
        assert!(!called.get(), "invalid submit must not run on_submit");
    }

    #[test]
    fn button_click_requests_submit_and_persists_errors() {
        let mut h = Harness::new();
        let mut value = Signup::default();
        let cfg = Cfg {
            submit_button: true,
            ..Cfg::default()
        };

        // Frame 1: lay out the button (the harness records its rect).
        let _ = h.run(&mut value, cfg, None, None);
        let rect = h
            .ctx
            .data_mut(|d| d.get_temp::<egui::Rect>(Id::unique("h_submit_rect")))
            .expect("submit button rect must be captured");

        // Click the button: the form becomes touched and invalid.
        let res = h.click(&mut value, rect.center(), cfg, None);
        assert!(res.submitted, "clicking the submit button must submit");
        assert!(!res.valid);
        assert_eq!(res.errors.len(), 1);

        // Touched persists: errors keep rendering on later frames.
        let res = h.run(&mut value, cfg, None, None);
        assert!(!res.submitted);
        assert_eq!(res.errors.len(), 1);
    }

    #[test]
    fn valid_submit_runs_callback_once() {
        let h = Harness::new();
        let mut value = Signup { email: "a@b.c".into() };
        let called = Rc::new(Cell::new(false));
        {
            let mut out = h.ctx.run_ui(egui::RawInput::default(), |ctx| {
                let called2 = called.clone();
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("form_valid_ui"),
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(420.0, 420.0),
                    )),
                );
                Form::new(&mut value)
                    .id_salt("valid")
                    .on_submit(move |_ui, v| {
                        called2.set(true);
                        assert_eq!(v.email, "a@b.c");
                    })
                    .show(ui, |ui, form| {
                        form.field(ui, "email", email_validator, |_ui, _v, ctx| {
                            assert!(ctx.error.is_none());
                        });
                        form.request_submit();
                    });
            });
            out.textures_delta.clear();
        }
        assert!(called.get(), "on_submit must run on a valid submit");
    }

    #[test]
    fn valid_submit_response_reports_valid() {
        let h = Harness::new();
        let mut value = Signup { email: "a@b.c".into() };
        let mut res = FormResponse::default();
        let mut out = h.ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("form_valid2_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(420.0, 420.0),
                )),
            );
            res = Form::new(&mut value)
                .id_salt("valid2")
                .show(ui, |ui, form| {
                    form.field(ui, "email", email_validator, |_ui, _v, _ctx| {});
                    form.request_submit();
                });
        });
        out.textures_delta.clear();
        assert!(res.submitted);
        assert!(res.valid);
        assert!(res.errors.is_empty());
    }

    #[test]
    fn submit_revalidates_the_post_edit_value() {
        let h = Harness::new();
        let mut value = Signup::default();
        let mut res = FormResponse::default();
        let mut out = h.ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("form_revalidate_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(420.0, 420.0),
                )),
            );
            res = Form::new(&mut value)
                .id_salt("revalidate")
                .show(ui, |ui, form| {
                    // The edit happens before the submit pass re-runs the
                    // validators against the post-edit value.
                    form.field(ui, "email", email_validator, |_ui, v, _ctx| {
                        v.email = "fixed@x.io".into();
                    });
                    form.request_submit();
                });
        });
        out.textures_delta.clear();
        assert!(res.valid, "submit must validate the edited value");
        assert!(res.errors.is_empty());
    }

    #[test]
    fn disabled_and_submitting_flags_reach_fields_and_button() {
        let h = Harness::new();
        let mut value = Signup::default();
        let mut out = h.ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("form_disabled_ui"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    Pos2::ZERO,
                    Vec2::new(420.0, 420.0),
                )),
            );
            Form::new(&mut value)
                .id_salt("disabled")
                .disabled(true)
                .submitting(true)
                .show(ui, |ui, form| {
                    assert!(form.disabled());
                    assert!(form.submitting());
                    form.field(ui, "email", email_validator, |_ui, _v, ctx| {
                        assert!(ctx.disabled);
                    });
                    form.submit_button(ui, "Submit");
                });
        });
        out.textures_delta.clear();
    }

    #[test]
    fn reset_errors_button_clears_touched_state() {
        let mut h = Harness::new();
        let mut value = Signup::default();
        let cfg = Cfg {
            submit_button: true,
            reset_button: true,
            ..Cfg::default()
        };

        // Lay out (the harness records both button rects).
        let _ = h.run(&mut value, cfg, None, None);
        let submit_rect = h
            .ctx
            .data_mut(|d| d.get_temp::<egui::Rect>(Id::unique("h_submit_rect")))
            .expect("submit rect");
        let reset_rect = h
            .ctx
            .data_mut(|d| d.get_temp::<egui::Rect>(Id::unique("h_reset_rect")))
            .expect("reset rect");

        // Submit (invalid) marks touched.
        let res = h.click(&mut value, submit_rect.center(), cfg, None);
        assert_eq!(res.errors.len(), 1);

        // Click Reset: the touched flag clears.
        let _ = h.click(&mut value, reset_rect.center(), cfg, None);

        // Errors are hidden again.
        let res = h.run(&mut value, cfg, None, None);
        assert!(res.errors.is_empty(), "reset must hide the errors");
    }

    #[test]
    fn renders_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut value = Signup::default();
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("form_render_ui"),
                    egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(420.0, 420.0),
                    )),
                );
                use crate::prelude::*;
                Form::new(&mut value)
                    .id_salt("render")
                    .validate_on_change(true)
                    .show(ui, |ui, form| {
                        form.field(ui, "email", email_validator, |ui, v, ctx| {
                            Field::new().invalid(ctx.invalid).show(ui, |ui| {
                                FieldLabel::new("Email").for_id(ctx.id).show(ui);
                                Input::new(&mut v.email)
                                    .placeholder("you@example.com")
                                    .invalid(ctx.invalid)
                                    .show(ui);
                                if let Some(err) = &ctx.error {
                                    FieldError::new(err.clone()).show(ui);
                                }
                            });
                        });
                        form.submit_button(ui, "Submit");
                    });
            });
            out.textures_delta.clear();
        }
    }
}
