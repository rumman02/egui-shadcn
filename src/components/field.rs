//! shadcn `field.tsx` — form field layout and typography primitives.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/field.tsx`. The compound
//! pieces mirror the source's slots:
//!
//! - [`Field`] — a labeled control group (`vertical` / `horizontal` /
//!   `responsive` orientation, `gap-3`); `invalid` tints the label and
//!   title `text-destructive`, `disabled` dims them to 50%.
//! - [`FieldLabel`] / [`FieldTitle`] — `text-sm font-medium` label styles.
//! - [`FieldDescription`] — `text-sm text-muted-foreground` helper text.
//! - [`FieldError`] — `text-sm text-destructive`; renders nothing when
//!   empty, one message, or a bulleted list of messages.
//! - [`FieldContent`] — stacks description/error under each other
//!   (`gap-1.5`).
//! - [`FieldGroup`] — stacks fields (`gap-7`), [`FieldSet`] — a bordered
//!   form section (`gap-6`), [`FieldLegend`] — a section heading, and
//!   [`FieldSeparator`] — a rule with optional centered text.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut notifications = false;
//! Field::new().show(ui, |ui| {
//!     FieldLabel::new("Notifications").for_id(egui::Id::unique("notif")).show(ui);
//!     FieldDescription::new("Get notified about account activity.").show(ui);
//!     Checkbox::new(&mut notifications).id_salt("notif").show(ui);
//!     FieldError::new("Please choose at least one channel.").show(ui);
//! });
//! ```

use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{Color32, FontId, Id, Response, Sense, Ui, Vec2};

/// Shared field state passed to children through the temp stack.
#[derive(Clone, Copy, Debug)]
struct FieldCtx {
    invalid: bool,
    disabled: bool,
}

fn ctx_stack_id() -> Id {
    Id::unique("shadcn_field_ctx_stack")
}

fn push_ctx(ui: &Ui, ctx: FieldCtx) {
    let mut stack: Vec<FieldCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctx_stack_id()).unwrap_or_default());
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(ctx_stack_id(), stack));
}

fn pop_ctx(ui: &Ui) {
    let mut stack: Vec<FieldCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctx_stack_id()).unwrap_or_default());
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(ctx_stack_id(), stack));
}

fn current_ctx(ui: &Ui) -> FieldCtx {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<FieldCtx>>(ctx_stack_id()))
        .and_then(|s| s.last().copied())
        .unwrap_or(FieldCtx {
            invalid: false,
            disabled: false,
        })
}

/// The orientation preset of a [`Field`] (`fieldVariants.orientation`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldOrientation {
    /// `flex-col gap-3` — label above the control.
    #[default]
    Vertical,
    /// `flex-row items-center gap-3` — label beside the control.
    Horizontal,
    /// `@md/field-group:` — horizontal when the container is wide enough
    /// (>= 448pt, the `@md` breakpoint), vertical otherwise.
    Responsive,
}

/// A shadcn field: a labeled control group. See the [module docs](self).
#[derive(Debug)]
pub struct Field {
    orientation: FieldOrientation,
    invalid: bool,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for Field {
    fn default() -> Self {
        Self::new()
    }
}

impl Field {
    /// Creates a vertical field.
    pub fn new() -> Self {
        Self {
            orientation: FieldOrientation::Vertical,
            invalid: false,
            disabled: false,
            id_salt: None,
        }
    }

    /// Sets the orientation preset.
    pub fn orientation(mut self, orientation: FieldOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Marks the field `data-invalid` (label/title render destructive).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Marks the field disabled (label/title render at 50% opacity).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt (Field keeps no state, but nested containers
    /// derive ids from the scope).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the field around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let effective = match self.orientation {
            FieldOrientation::Vertical => FieldOrientation::Vertical,
            FieldOrientation::Horizontal => FieldOrientation::Horizontal,
            FieldOrientation::Responsive => {
                // `@md` container breakpoint = 28rem = 448px.
                if ui.available_width() >= 448.0 {
                    FieldOrientation::Horizontal
                } else {
                    FieldOrientation::Vertical
                }
            }
        };

        push_ctx(
            ui,
            FieldCtx {
                invalid: self.invalid,
                disabled: self.disabled,
            },
        );
        let gap = crate::theme::tokens::spacing::MD; // gap-3
        let inner = match effective {
            FieldOrientation::Horizontal => ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                content(ui)
            }),
            _ => ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = gap;
                content(ui)
            }),
        };
        pop_ctx(ui);
        inner
    }
}

/// `FieldLabel` — a `text-sm font-medium` label bound to the field state.
///
/// Reads the enclosing [`Field`]: 50% opacity when the field is disabled,
/// destructive color when it is invalid.
#[derive(Debug)]
pub struct FieldLabel {
    text: String,
    for_id: Option<Id>,
    disabled: bool,
    id_salt: Option<String>,
}

impl FieldLabel {
    /// Creates a field label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            for_id: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Binds the label to a control id: clicking focuses that widget.
    pub fn for_id(mut self, id: Id) -> Self {
        self.for_id = Some(id);
        self
    }

    /// Forces the disabled look regardless of the enclosing field.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the label.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let ctx = current_ctx(ui);
        let disabled = self.disabled || ctx.disabled;

        let color = field_label_color(&p, ctx.invalid, disabled);
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), Color32::WHITE);
        let desired = Vec2::new(galley.size().x, galley.size().y.max(font.size));

        let interactive = self.for_id.is_some() && !disabled;
        let sense = if interactive {
            Sense::click()
        } else {
            Sense::hover()
        };
        let (rect, response) = {
            // Reserve space without registering an auto-id focusable
            // response, so egui's spatial arrow-key navigation only sees the
            // explicit widget id (checkbox.rs pattern).
            let (_, rect) = ui.allocate_space(desired);
            let id = match self.id_salt.as_deref() {
                Some(salt) => ui.scope_id().with(salt),
                None => ui.auto_id_with("shadcn_field_label"),
            };
            (rect, ui.interact(rect, id, sense))
        };

        if interactive && response.clicked() {
            if let Some(target) = self.for_id {
                ui.memory_mut(|mem| mem.request_focus(target));
            }
        }

        if ui.is_rect_visible(rect) {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                self.text,
                font,
                color,
            );
        }
        response
    }
}

/// `FieldTitle` — `text-sm font-medium` with an optional leading icon.
#[derive(Debug)]
pub struct FieldTitle {
    text: String,
    icon: Option<crate::icons::Icon>,
    disabled: bool,
    id_salt: Option<String>,
}

impl FieldTitle {
    /// Creates a field title.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Shows an icon before the title.
    pub fn icon(mut self, icon: crate::icons::Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Forces the disabled look regardless of the enclosing field.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let ctx = current_ctx(ui);
        let disabled = self.disabled || ctx.disabled;
        let color = field_label_color(&p, ctx.invalid, disabled);

        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let icon_size = crate::theme::tokens::spacing::LG;
        let text_w = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), Color32::WHITE)
            .size()
            .x;
        let gap = self
            .icon
            .map(|_| crate::theme::tokens::spacing::SM)
            .unwrap_or(0.0);
        let width = icon_size * self.icon.is_some() as i32 as f32 + gap + text_w;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, font.size * 1.3), Sense::hover());

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let mut cursor = rect.min.x;
            if let Some(icon) = self.icon {
                let icon_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(cursor, rect.center().y - icon_size / 2.0),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, icon_rect, color, 2.0 * icon_size / 24.0);
                cursor += icon_size + gap;
            }
            painter.text(
                egui::Pos2::new(cursor, rect.center().y),
                egui::Align2::LEFT_CENTER,
                self.text,
                font,
                color,
            );
        }
        response
    }
}

/// `FieldDescription` — `text-sm text-muted-foreground` helper text that
/// wraps to the available width.
#[derive(Debug)]
pub struct FieldDescription {
    text: String,
    id_salt: Option<String>,
}

impl FieldDescription {
    /// Creates a field description.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let ctx = current_ctx(ui);
        // `text-muted-foreground` regardless of the invalid flag (it sets an
        // explicit color, unlike the label which inherits).
        let color = if ctx.disabled {
            color_alpha(p.muted_foreground, 0.5)
        } else {
            p.muted_foreground
        };
        let max_width = ui.available_width();

        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley =
            ui.painter()
                .layout(self.text.clone(), font.clone(), Color32::WHITE, max_width);
        let (rect, response) = ui.allocate_exact_size(
            Vec2::new(galley.size().x.max(1.0), galley.size().y),
            Sense::hover(),
        );

        if ui.is_rect_visible(rect) {
            ui.painter()
                .galley(egui::Pos2::new(rect.min.x, rect.min.y), galley, color);
        }
        response
    }
}

/// `FieldError` — `text-sm text-destructive`. Renders one message, a
/// bulleted list for several, or nothing at all.
#[derive(Debug)]
pub struct FieldError {
    message: Option<String>,
    errors: Vec<String>,
    id_salt: Option<String>,
}

impl FieldError {
    /// Creates a field error with a single message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            errors: Vec::new(),
            id_salt: None,
        }
    }

    /// Creates a field error that renders one message or a bullet list.
    pub fn errors(errors: Vec<String>) -> Self {
        Self {
            message: None,
            errors,
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the error (or nothing when there is no message).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let color = p.destructive;
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);

        let mut messages: Vec<String> = Vec::new();
        if let Some(m) = self.message {
            if !m.is_empty() {
                messages.push(m);
            }
        } else {
            messages.extend(self.errors.into_iter().filter(|m| !m.is_empty()));
        }
        if messages.is_empty() {
            // `if (!content) return null` — no space taken.
            return ui.allocate_exact_size(Vec2::ZERO, Sense::hover()).1;
        }

        let max_width = ui.available_width();
        let (total_height, rows): (f32, Vec<_>) = if messages.len() == 1 {
            let galley =
                ui.painter()
                    .layout(messages[0].clone(), font.clone(), Color32::WHITE, max_width);
            (galley.size().y, vec![galley])
        } else {
            // `ml-4 list-disc gap-1`
            let indent = crate::theme::tokens::spacing::LG;
            let mut height = 0.0;
            let mut rows = Vec::new();
            for (i, m) in messages.iter().enumerate() {
                let galley = ui.painter().layout(
                    format!("• {m}"),
                    font.clone(),
                    Color32::WHITE,
                    (max_width - indent).max(1.0),
                );
                height += galley.size().y
                    + if i > 0 {
                        crate::theme::tokens::spacing::XS
                    } else {
                        0.0
                    };
                rows.push(galley);
            }
            (height, rows)
        };

        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(max_width, total_height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let indent = if messages.len() > 1 {
                crate::theme::tokens::spacing::LG
            } else {
                0.0
            };
            let mut y = rect.min.y;
            for galley in &rows {
                ui.painter().galley(
                    egui::Pos2::new(rect.min.x + indent, y),
                    galley.clone(),
                    color,
                );
                y += galley.size().y + crate::theme::tokens::spacing::XS;
            }
        }
        response
    }
}

/// `FieldContent` — stacks the description/error column (`gap-1.5`).
#[derive(Debug, Default)]
pub struct FieldContent {
    id_salt: Option<String>,
}

impl FieldContent {
    /// Creates a field content column.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the column.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::XS_HALF;
            content(ui)
        })
    }
}

/// `FieldGroup` — stacks fields with `gap-7`.
#[derive(Debug, Default)]
pub struct FieldGroup {
    id_salt: Option<String>,
}

impl FieldGroup {
    /// Creates a field group.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y =
                crate::theme::tokens::spacing::X2L - crate::theme::tokens::spacing::XS;
            content(ui)
        })
    }
}

/// `FieldSet` — a form section stacking fields with `gap-6`.
#[derive(Debug, Default)]
pub struct FieldSet {
    id_salt: Option<String>,
}

impl FieldSet {
    /// Creates a field set.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the set.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::XL;
            content(ui)
        })
    }
}

/// `FieldLegend` variant (`data-variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldLegendVariant {
    /// `text-base font-medium` section heading.
    #[default]
    Legend,
    /// `text-sm font-medium` sub-heading.
    Label,
}

/// `FieldLegend` — a section heading with `mb-3` bottom spacing.
#[derive(Debug)]
pub struct FieldLegend {
    text: String,
    variant: FieldLegendVariant,
    id_salt: Option<String>,
}

impl FieldLegend {
    /// Creates a legend.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: FieldLegendVariant::Legend,
            id_salt: None,
        }
    }

    /// Sets the variant.
    pub fn variant(mut self, variant: FieldLegendVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the legend.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let size = match self.variant {
            FieldLegendVariant::Legend => crate::theme::tokens::type_scale::BASE,
            FieldLegendVariant::Label => crate::theme::tokens::type_scale::SM,
        };
        let font = FontId::proportional(size);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), Color32::WHITE);
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(galley.size().x, galley.size().y), Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, p.foreground);
        }
        // `mb-3`
        ui.add_space(crate::theme::tokens::spacing::MD);
        response
    }
}

/// `FieldSeparator` — a horizontal rule with optional centered text
/// (`relative -my-2 h-5` with the content chip on top).
#[derive(Debug, Default)]
pub struct FieldSeparator {
    content: Option<String>,
    id_salt: Option<String>,
}

impl FieldSeparator {
    /// Creates a plain rule.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a rule with centered text.
    pub fn content(content: impl Into<String>) -> Self {
        Self {
            content: Some(content.into()),
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the separator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let height = crate::theme::tokens::spacing::LG; // h-4 visual row
        let width = ui.available_width();
        let (rect, response) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let y = rect.center().y;
            painter.line_segment(
                [
                    egui::Pos2::new(rect.min.x, y),
                    egui::Pos2::new(rect.max.x, y),
                ],
                egui::Stroke::new(1.0, p.border),
            );
            if let Some(text) = self.content {
                // The chip paints `bg-background` over the rule, then the text.
                let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
                let galley = painter.layout_no_wrap(text.clone(), font.clone(), Color32::WHITE);
                let pad = crate::theme::tokens::spacing::SM; // px-2
                let chip_w = galley.size().x + pad * 2.0;
                let chip = egui::Rect::from_center_size(
                    rect.center(),
                    Vec2::new(chip_w, galley.size().y.max(height)),
                );
                painter.rect_filled(chip, 0, p.background);
                painter.text(
                    egui::Pos2::new(chip.center().x, chip.center().y),
                    egui::Align2::CENTER_CENTER,
                    text,
                    font,
                    p.muted_foreground,
                );
            }
        }
        response
    }
}

/// Resolves the label/title color for the field state (pure, testable).
pub(crate) fn field_label_color(p: &Palette, invalid: bool, disabled: bool) -> Color32 {
    let base = if invalid { p.destructive } else { p.foreground };
    if disabled {
        color_alpha(base, 0.5)
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn label_color_follows_field_state() {
        let p = palette(Mode::Light);
        assert_eq!(field_label_color(&p, false, false), p.foreground);
        assert_eq!(field_label_color(&p, true, false), p.destructive);
        assert_eq!(
            field_label_color(&p, false, true),
            color_alpha(p.foreground, 0.5)
        );
        assert_eq!(
            field_label_color(&p, true, true),
            color_alpha(p.destructive, 0.5)
        );
    }

    #[test]
    fn responsive_orientation_matches_container_width() {
        // The @md breakpoint is 28rem = 448px.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut horizontal = false;
        let mut vertical = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let wide = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_wide"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(600.0, 400.0),
                )),
            );
            Field::new()
                .orientation(FieldOrientation::Responsive)
                .show(wide, |_ui| horizontal = true);
            let narrow = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_narrow"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(300.0, 400.0),
                )),
            );
            Field::new()
                .orientation(FieldOrientation::Responsive)
                .show(narrow, |ui| {
                    vertical = ui.layout().is_vertical();
                });
        });
        out.textures_delta.clear();
        assert!(horizontal); // wide container laid out horizontally
        assert!(vertical); // narrow container laid out vertically
    }

    #[test]
    fn field_error_renders_nothing_when_empty() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut rect = egui::Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("field_error_test"),
                egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    Vec2::new(400.0, 400.0),
                )),
            );
            rect = FieldError::errors(vec![]).show(ui).rect;
        });
        out.textures_delta.clear();
        assert!(rect.size() == Vec2::ZERO);
    }

    #[test]
    fn builders_keep_fields() {
        let f = Field::new()
            .orientation(FieldOrientation::Horizontal)
            .invalid(true)
            .disabled(true)
            .id_salt("f");
        assert_eq!(f.orientation, FieldOrientation::Horizontal);
        assert!(f.invalid);
        assert!(f.disabled);
        assert_eq!(f.id_salt.as_deref(), Some("f"));

        let l = FieldLabel::new("Email")
            .for_id(Id::unique("email"))
            .disabled(true)
            .id_salt("l");
        assert_eq!(l.for_id, Some(Id::unique("email")));
        assert!(l.disabled);

        let t = FieldTitle::new("Name")
            .icon(crate::icons::Icon::User)
            .id_salt("t");
        assert!(t.icon.is_some());

        let d = FieldDescription::new("Help").id_salt("d");
        assert_eq!(d.id_salt.as_deref(), Some("d"));

        let e = FieldError::new("bad").id_salt("e");
        assert_eq!(e.id_salt.as_deref(), Some("e"));

        let legend = FieldLegend::new("Billing")
            .variant(FieldLegendVariant::Label)
            .id_salt("legend");
        assert_eq!(legend.variant, FieldLegendVariant::Label);

        let sep = FieldSeparator::content("or").id_salt("sep");
        assert_eq!(sep.content.as_deref(), Some("or"));
    }
}
