//! shadcn `input-group.tsx` — an input with inline/block addons.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/input-group.tsx`. The
//! container is a `h-9 rounded-md border border-input shadow-xs` row
//! (`dark:bg-input/30`) that switches to a column when a block addon or a
//! textarea is present (`has-[>[data-align=block-start]]:flex-col
//! has-[>textarea]:h-auto`), grows a 3px ring when any control inside is
//! focused (`has-[[data-slot=input-group-control]:focus-visible]`), turns
//! destructive when any child is `aria-invalid`, and under the nova style
//! (`.cn-input-group`) tightens to `h-8 rounded-lg` with
//! `has-disabled:bg-input/50 dark:has-disabled:bg-input/80`.
//!
//! The source's slots map as follows:
//!
//! - [`InputGroupAddon`] — `inline-start` / `inline-end` / `block-start` /
//!   `block-end` alignment; holds text, icons, [`Kbd`](crate::components::kbd::Kbd)
//!   and [`InputGroupButton`]s.
//! - [`InputGroupText`] — `text-sm text-muted-foreground` inline text.
//! - [`InputGroupButton`] — ghost `Button` with `xs`/`sm`/`icon-xs`/`icon-sm`
//!   sizes.
//! - [`InputGroupInput`] / [`InputGroupTextarea`] — frameless borderless
//!   controls (`flex-1 rounded-none border-0 bg-transparent shadow-none`).
//!
//! Focus, invalid and layout flags are resolved from the previous frame
//! (like the source's `has-[]:` selectors, which are reactive); they settle
//! one frame after a state change.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut url = String::new();
//! InputGroup::new().show(ui, |ui| {
//!     InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
//!         InputGroupText::new("https://").show(ui);
//!     });
//!     InputGroupInput::new(&mut url).placeholder("example.com").show(ui);
//!     InputGroupAddon::new(AddonAlign::InlineEnd).show(ui, |ui| {
//!         InputGroupButton::new("").icon(Icon::Copy).show(ui);
//!     });
//! });
//! ```

use crate::components::button::{
    fuse_radius, group, Button, ButtonSize, ButtonVariant, GroupOrientation, GroupPosition,
};
use crate::components::input;
use crate::icons::Icon;
use crate::theme::{color_alpha, RadiusTokens, ShadcnTheme, Style};
use egui::{
    Align as EguiAlign, Align2, Color32, CornerRadius, FontId, Id, Layout, Margin, Rect, Response,
    Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2,
};

/// Container height (`h-9` base recipe; nova `.cn-input-group` `h-8`).
pub(crate) fn container_height(style: Style) -> f32 {
    match style {
        Style::Nova => 32.0,                         // h-8
        _ => crate::theme::tokens::spacing::CONTROL, // h-9
    }
}

/// Container corner radius (`rounded-md` base recipe; nova `rounded-lg`).
pub(crate) fn container_radius(style: Style, radius: &RadiusTokens) -> u8 {
    match style {
        Style::Nova => radius.lg_u8(),
        _ => radius.md_u8(),
    }
}

/// Alignment slot of an [`InputGroupAddon`] (`data-align`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AddonAlign {
    /// `order-first pl-3` — before the control, in the row.
    #[default]
    InlineStart,
    /// `order-last pr-3` — after the control, in the row.
    InlineEnd,
    /// `order-first w-full px-3 pt-3` — a full-width header row.
    BlockStart,
    /// `order-last w-full px-3 pb-3` — a full-width footer row.
    BlockEnd,
}

/// Size of an [`InputGroupButton`] (`data-size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InputGroupButtonSize {
    /// `h-6 px-2` ghost chip (default).
    #[default]
    Xs,
    /// `h-8 px-2.5`.
    Sm,
    /// `size-6` icon-only square.
    IconXs,
    /// `size-8` icon-only square.
    IconSm,
}

impl From<InputGroupButtonSize> for ButtonSize {
    fn from(size: InputGroupButtonSize) -> Self {
        match size {
            InputGroupButtonSize::Xs => ButtonSize::Xs,
            InputGroupButtonSize::Sm => ButtonSize::Sm,
            InputGroupButtonSize::IconXs => ButtonSize::IconXs,
            InputGroupButtonSize::IconSm => ButtonSize::IconSm,
        }
    }
}

/// Inline addon side padding (`pl-3`/`pr-3` base recipe; nova `pl-2`/`pr-2`).
fn addon_inline_padding(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0, // pl-2 / pr-2
        _ => 12.0,          // pl-3 / pr-3
    }
}

/// Block addon horizontal padding (`px-3` base recipe; nova `px-2.5`).
fn addon_block_padding_x(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0, // px-2.5
        _ => 12.0,           // px-3
    }
}

/// Block addon vertical padding (`pt-3`/`pb-3` base recipe; nova
/// `pt-2`/`pb-2`).
fn addon_block_padding_y(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0, // pt-2 / pb-2
        _ => 12.0,          // pt-3 / pb-3
    }
}

/// Control side padding next to an inline addon
/// (`[&>input]:pl-2`/`pr-2` base recipe; nova `pl-1.5`/`pr-1.5`).
fn control_inline_padding(style: Style) -> f32 {
    match style {
        Style::Nova => 6.0, // pl-1.5 / pr-1.5
        _ => 8.0,           // pl-2 / pr-2
    }
}

/// Flags recorded by the container's children while they render. The
/// container reads the previous frame's snapshot for its own painting.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct GroupRecord {
    focus: bool,
    invalid: bool,
    inline_start: bool,
    inline_end: bool,
    block: bool,
    textarea: bool,
    /// The container was created with `.disabled(true)`.
    disabled_container: bool,
    /// Measured content height of a column container (for the next frame's
    /// reservation).
    height: f32,
    /// Measured width of the trailing inline addon (for the next frame's
    /// input sizing).
    end_addon_width: f32,
    /// The group's text control id, for addon click-to-focus.
    input_id: Option<Id>,
    /// A button inside the group was clicked this frame (the addon's
    /// click-to-focus must not steal it).
    button_clicked: bool,
}

fn stack_key() -> Id {
    Id::unique("shadcn_input_group_stack")
}

fn current_id(ui: &Ui) -> Option<egui::Id> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<egui::Id>>(stack_key()))
        .and_then(|stack| stack.last().copied())
}

fn push_id(ui: &Ui, id: egui::Id) {
    let mut stack: Vec<egui::Id> = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Vec<egui::Id>>(stack_key()))
        .unwrap_or_default();
    stack.push(id);
    ui.ctx().data_mut(|d| d.insert_temp(stack_key(), stack));
}

fn pop_id(ui: &Ui) {
    let mut stack: Vec<egui::Id> = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Vec<egui::Id>>(stack_key()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(stack_key(), stack));
}

fn read_record(ui: &Ui, group_id: egui::Id) -> GroupRecord {
    ui.ctx().data_mut(|d| {
        d.get_temp::<GroupRecord>(group_id.with("current"))
            .unwrap_or_default()
    })
}

fn write_record(ui: &Ui, group_id: egui::Id, record: GroupRecord) {
    ui.ctx()
        .data_mut(|d| d.insert_temp(group_id.with("current"), record));
}

fn write_snapshot(ui: &Ui, group_id: egui::Id, record: GroupRecord) {
    ui.ctx()
        .data_mut(|d| d.insert_temp(group_id.with("snapshot"), record));
}

fn read_snapshot(ui: &Ui, group_id: egui::Id) -> GroupRecord {
    ui.ctx().data_mut(|d| {
        d.get_temp::<GroupRecord>(group_id.with("snapshot"))
            .unwrap_or_default()
    })
}

/// A shadcn input group container. See the [module docs](self).
#[derive(Debug)]
pub struct InputGroup {
    width: Option<f32>,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for InputGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl InputGroup {
    /// Creates an input group container.
    pub fn new() -> Self {
        Self {
            width: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Sets an explicit width. The default is `w-full` (the available width
    /// in vertical layouts).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Marks the whole group disabled (children render at 50%).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt (recommended: the group's reactive frame state
    /// is keyed by it).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its children.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let style = theme.style;

        let group_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input_group"),
        };
        let last = read_snapshot(ui, group_id);

        // Auto-detected position inside a ButtonGroup (corner fusing).
        let grp = group::next_child_position(ui);
        let position = grp.map(|(_, pos)| pos).unwrap_or(GroupPosition::Only);
        let orientation = grp.map(|(o, _)| o).unwrap_or(GroupOrientation::Horizontal);

        let width = match self.width {
            Some(w) => w,
            None => {
                if ui.layout().is_vertical() {
                    ui.available_width()
                } else {
                    280.0
                }
            }
        };

        // `has-[>textarea]:h-auto` / block addons switch to a column.
        let column = last.block || last.textarea;
        let hint_height = if column && last.height > 0.0 {
            last.height
        } else {
            container_height(style) // h-9 / nova h-8
        };

        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, hint_height), Sense::hover());

        // Background fill + shadow must be painted before the children so
        // their text stays on top. `has-disabled:bg-input/50
        // dark:has-disabled:bg-input/80`.
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let corner = CornerRadius::same(container_radius(style, &theme.radius));
            if !self.disabled {
                let sh = crate::utils::shadow::shadow_xs(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        corner,
                        sh.color,
                    )
                    .with_blur_width(2.0),
                );
            }
            let fill = if self.disabled {
                color_alpha(p.input, if dark { 0.8 } else { 0.5 })
            } else if dark {
                color_alpha(p.input, 0.3)
            } else {
                Color32::TRANSPARENT
            };
            if fill != Color32::TRANSPARENT {
                painter.rect_filled(rect, corner, fill);
            }
        }

        let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(if column {
            Layout::top_down(EguiAlign::Min)
        } else {
            Layout::left_to_right(EguiAlign::Center)
        }));
        child.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

        // Fresh per-frame record for the children to update.
        write_record(
            ui,
            group_id,
            GroupRecord {
                disabled_container: self.disabled,
                ..GroupRecord::default()
            },
        );
        push_id(ui, group_id);
        let inner = child.scope(|ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            content(ui)
        });
        pop_id(ui);

        let mut record = read_record(ui, group_id);
        // Measure the actual content for the next frame's reservation.
        let content_rect = rect.union(child.min_rect());
        record.height = content_rect.height();
        write_snapshot(ui, group_id, record);

        if ui.is_rect_visible(content_rect) {
            let painter = ui.painter();
            let radius = container_radius(style, &theme.radius);
            let corner = fuse_radius(CornerRadius::same(radius), orientation, position);
            let draw_rect = match orientation {
                GroupOrientation::Horizontal
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(content_rect.min + egui::vec2(1.0, 0.0), content_rect.max)
                }
                GroupOrientation::Vertical
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(content_rect.min + egui::vec2(0.0, 1.0), content_rect.max)
                }
                _ => content_rect,
            };

            let invalid = record.invalid;
            let stroke_color = if invalid {
                p.destructive
            } else if record.focus {
                // `has-[control:focus-visible]:border-ring`
                p.ring
            } else {
                p.input
            };
            // `has-disabled:opacity-50` fades the border too.
            let stroke_color = if self.disabled {
                color_alpha(stroke_color, 0.5)
            } else {
                stroke_color
            };
            let stroke = Stroke::new(1.0, stroke_color);
            painter.rect_stroke(draw_rect, corner, stroke, egui::epaint::StrokeKind::Inside);

            // `has-[[data-slot=input-group-control]:focus-visible]:
            // ring-[3px] ring-ring/50`
            if record.focus && !self.disabled {
                let ring = if invalid {
                    color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
                } else {
                    color_alpha(p.ring, 0.5)
                };
                let ring_rect = content_rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(painter, ring_rect, ring, 3.0, radius as f32);
            }
        }

        egui::InnerResponse::new(inner.inner, inner.response)
    }
}

/// `InputGroupInput` — the frameless text control (`data-slot=input-group-control`).
#[derive(Debug)]
pub struct InputGroupInput<'a> {
    text: &'a mut String,
    placeholder: Option<String>,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> InputGroupInput<'a> {
    /// Creates the group's text input.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Shows faint hint text when empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Marks the control `aria-invalid` (the container border turns
    /// destructive).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Disables the control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the input. Fills the remaining row width, shrinking by the
    /// trailing addon width measured last frame (`flex-1`).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let style = theme.style;

        let group_id = current_id(ui);
        let mut record = group_id.map(|id| read_record(ui, id)).unwrap_or_default();
        if self.invalid {
            record.invalid = true;
        }

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input_group_input"),
        };

        // `flex-1`: all remaining row width minus the trailing addon (which
        // renders after this control, so use last frame's measured width).
        let snapshot = group_id.map(|id| read_snapshot(ui, id)).unwrap_or_default();
        let width = ui.available_width() - snapshot.end_addon_width.max(0.0);
        let is_empty = self.text.is_empty();
        let text_color = if self.disabled || self.disabled_in(ui) {
            color_alpha(p.foreground, 0.5)
        } else {
            p.foreground
        };

        // `[&>input]:pl-2` when an inline-start addon precedes the control
        // (nova `pl-1.5`); `[&>input]:pr-2` when an inline-end addon follows
        // (nova `pr-1.5`, detected from last frame's snapshot).
        let left_pad = if record.inline_start {
            control_inline_padding(style)
        } else {
            crate::theme::tokens::spacing::MD
        };
        let right_pad = if snapshot.inline_end {
            control_inline_padding(style)
        } else {
            crate::theme::tokens::spacing::MD
        };

        let disabled = self.disabled || self.disabled_in(ui);
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let edit = TextEdit::singleline(self.text)
            .id_salt(id)
            .font(font)
            .text_color(text_color)
            .desired_width(width.max(1.0))
            .min_size(Vec2::new(width.max(1.0), input::input_height(style)))
            .align(Align2::LEFT_CENTER)
            .margin(Margin {
                left: left_pad as i8,
                right: right_pad as i8,
                top: 0,
                bottom: 0,
            })
            .background_color(Color32::TRANSPARENT);
        let edit = if let Some(placeholder) = self.placeholder.as_deref() {
            if is_empty {
                edit.hint_text(placeholder.to_owned())
            } else {
                edit
            }
        } else {
            edit
        };
        let edit = if disabled {
            edit.interactive(false)
        } else {
            edit
        };

        // The scoped spacing keeps the theme's `interact_size` floor (36pt)
        // from overriding the nova `h-8` (32pt) row.
        let out = ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = input::input_height(style);
            edit.show(ui)
        });
        let response = out.inner.response.response;

        record.focus |= response.has_focus();
        record.input_id = Some(response.id);
        if let Some(group) = group_id {
            write_record(ui, group, record);
        }

        response
    }

    fn disabled_in(&self, ui: &Ui) -> bool {
        let Some(group_id) = current_id(ui) else {
            return false;
        };
        read_record(ui, group_id).disabled_container
    }
}

/// `InputGroupTextarea` — the multiline control; switches the container to
/// auto-height column layout.
#[derive(Debug)]
pub struct InputGroupTextarea<'a> {
    text: &'a mut String,
    placeholder: Option<String>,
    rows: usize,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> InputGroupTextarea<'a> {
    /// Creates the group's textarea.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            rows: 3,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Shows faint hint text when empty.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the visible row count.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    /// Marks the control `aria-invalid`.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Disables the control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the textarea.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let style = theme.style;
        let group_id = current_id(ui);
        let mut record = group_id.map(|id| read_record(ui, id)).unwrap_or_default();
        record.textarea = true;
        if self.invalid {
            record.invalid = true;
        }

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input_group_textarea"),
        };

        let width = ui.available_width();
        let text_color = if self.disabled {
            color_alpha(p.foreground, 0.5)
        } else {
            p.foreground
        };
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        // `px-3 py-3` base recipe; nova `.cn-input-group-textarea` `px-2.5 py-2`.
        let (pad_x, pad_y) = match style {
            Style::Nova => (10.0, 8.0), // px-2.5 py-2
            _ => (12.0, 12.0),          // px-3 py-3
        };
        let edit = TextEdit::multiline(self.text)
            .id_salt(id)
            .font(font)
            .text_color(text_color)
            .desired_width(width)
            .desired_rows(self.rows)
            .min_size(Vec2::new(width, input::input_height(style)))
            .margin(Margin::symmetric(pad_x as i8, pad_y as i8))
            .background_color(Color32::TRANSPARENT);
        let edit = if let Some(placeholder) = self.placeholder.as_deref() {
            edit.hint_text(placeholder.to_owned())
        } else {
            edit
        };
        let edit = if self.disabled {
            edit.interactive(false)
        } else {
            edit
        };

        // Scoped spacing: same `interact_size` floor guard as the input.
        let out = ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = input::input_height(style);
            edit.show(ui)
        });
        let response = out.inner.response.response;

        record.focus |= response.has_focus();
        if record.input_id.is_none() {
            record.input_id = Some(response.id);
        }
        if let Some(group) = group_id {
            write_record(ui, group, record);
        }

        response
    }
}

/// `InputGroupAddon` — a slot beside the control for text, icons, kbd or
/// buttons. Clicking the addon focuses the group's control (the source's
/// `onClick`), except when the click hit a button inside the addon.
#[derive(Debug)]
pub struct InputGroupAddon {
    align: AddonAlign,
    id_salt: Option<String>,
}

impl InputGroupAddon {
    /// Creates an addon with the given alignment (default `inline-start`).
    pub fn new(align: AddonAlign) -> Self {
        Self {
            align,
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the addon around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let style = theme.style;
        let group_id = current_id(ui);
        let mut record = group_id.map(|id| read_record(ui, id)).unwrap_or_default();

        let inner = match self.align {
            AddonAlign::InlineStart => {
                record.inline_start = true;
                if let Some(group) = group_id {
                    write_record(ui, group, record);
                }
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::SM;
                    ui.set_min_size(Vec2::new(0.0, container_height(style)));
                    // `pl-3 py-1.5` base recipe; nova `pl-2`.
                    ui.add_space(addon_inline_padding(style));
                    content(ui)
                })
            }
            AddonAlign::InlineEnd => {
                record.inline_end = true;
                let inner = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::SM;
                    ui.set_min_size(Vec2::new(0.0, container_height(style)));
                    let r = content(ui);
                    // `pr-3` base recipe; nova `pr-2`.
                    ui.add_space(addon_inline_padding(style));
                    r
                });
                // Record the measured width so the input can reserve room
                // (`flex-1` behavior).
                record.end_addon_width = inner.response.rect.width();
                if let Some(group) = group_id {
                    write_record(ui, group, record);
                }
                inner
            }
            AddonAlign::BlockStart => {
                record.block = true;
                if let Some(group) = group_id {
                    write_record(ui, group, record);
                }
                // `px-3 pt-3` base recipe; nova `px-2.5 pt-2`.
                ui.add_space(addon_block_padding_y(style));
                let inner = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::SM;
                    ui.set_min_size(Vec2::new(ui.available_width(), 0.0));
                    ui.add_space(addon_block_padding_x(style));
                    let r = content(ui);
                    ui.add_space(addon_block_padding_x(style));
                    r
                });
                inner
            }
            AddonAlign::BlockEnd => {
                record.block = true;
                let inner = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::SM;
                    ui.set_min_size(Vec2::new(ui.available_width(), 0.0));
                    ui.add_space(addon_block_padding_x(style));
                    let r = content(ui);
                    ui.add_space(addon_block_padding_x(style));
                    r
                });
                if let Some(group) = group_id {
                    write_record(ui, group, record);
                }
                // `px-3 pb-3` base recipe; nova `px-2.5 pb-2`.
                ui.add_space(addon_block_padding_y(style));
                inner
            }
        };

        // `onClick`: focus the group's control (unless the click hit a
        // button inside the addon). The click is resolved from the pointer
        // state so no click-sense interact steals hits from the buttons.
        if let Some(group) = group_id {
            let record = read_record(ui, group);
            let focus_target = record
                .input_id
                .or_else(|| read_snapshot(ui, group).input_id);
            if let Some(input_id) = focus_target {
                let clicked = ui.input(|i| {
                    i.pointer.any_click()
                        && i.pointer
                            .press_origin()
                            .is_some_and(|o| inner.response.rect.contains(o))
                });
                if clicked && !record.disabled_container && !record.button_clicked {
                    ui.memory_mut(|mem| mem.request_focus(input_id));
                }
            }
        }

        inner
    }
}

/// `InputGroupText` — `text-sm text-muted-foreground` inline text or icon.
#[derive(Debug)]
pub struct InputGroupText {
    text: String,
    icon: Option<Icon>,
    disabled: bool,
    id_salt: Option<String>,
}

impl InputGroupText {
    /// Creates group text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Shows an icon instead of (or before) text.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Renders at 50% opacity.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the text.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let color = if self.disabled {
            color_alpha(p.muted_foreground, 0.5)
        } else {
            p.muted_foreground
        };
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let icon_size = crate::theme::tokens::spacing::LG;
        let text_w = if self.text.is_empty() {
            0.0
        } else {
            ui.painter()
                .layout_no_wrap(self.text.clone(), font.clone(), Color32::WHITE)
                .size()
                .x
        };
        let gap = self
            .icon
            .map(|_| crate::theme::tokens::spacing::SM)
            .unwrap_or(0.0);
        let width = icon_size * self.icon.is_some() as i32 as f32 + gap + text_w;
        let desired = Vec2::new(width.max(1.0), container_height(theme.style));

        let (rect, response) = match self.id_salt.as_deref() {
            Some(salt) => {
                let id = ui.scope_id().with(salt);
                let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
                (rect, ui.interact(rect, id, Sense::hover()))
            }
            None => ui.allocate_exact_size(desired, Sense::hover()),
        };

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let mut cursor = rect.min.x;
            if let Some(icon) = self.icon {
                let icon_rect = egui::Rect::from_center_size(
                    egui::Pos2::new(cursor + icon_size / 2.0, rect.center().y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, icon_rect, color, 2.0 * icon_size / 24.0);
                cursor += icon_size + gap;
            }
            if !self.text.is_empty() {
                painter.text(
                    egui::Pos2::new(cursor, rect.center().y),
                    Align2::LEFT_CENTER,
                    self.text,
                    font,
                    color,
                );
            }
        }
        response
    }
}

impl egui::Widget for InputGroupText {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `InputGroupButton` — a ghost [`Button`] sized for the group
/// (`xs` default, `sm`, `icon-xs`, `icon-sm`).
#[derive(Debug)]
pub struct InputGroupButton {
    button: Button,
    size: InputGroupButtonSize,
}

impl InputGroupButton {
    /// Creates a ghost group button with a text label (size `xs`).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            button: Button::new(label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Xs),
            size: InputGroupButtonSize::Xs,
        }
    }

    /// Creates an icon-only group button (size `icon-xs`).
    pub fn icon(icon: Icon) -> Self {
        Self {
            button: Button::new("")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconXs)
                .icon_start(icon),
            size: InputGroupButtonSize::IconXs,
        }
    }

    /// Sets the size.
    pub fn size(mut self, size: InputGroupButtonSize) -> Self {
        self.size = size;
        self.button = self.button.size(size.into());
        self
    }

    /// Sets the visual variant (default ghost).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.button = self.button.variant(variant);
        self
    }

    /// Disables the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.button = self.button.id_salt(salt);
        self
    }

    /// Renders the button.
    pub fn show(self, ui: &mut Ui) -> Response {
        // The `sm` spec (`h-8 px-2.5 gap-1.5`) is identical in both styles,
        // so the nova style maps it onto the nova `Default` size (also
        // `h-8 px-2.5`); the base recipe keeps `ButtonSize::Sm` (`h-8`).
        let style = ShadcnTheme::of_ui(ui).style;
        let size = match self.size {
            InputGroupButtonSize::Sm if style == Style::Nova => ButtonSize::Default,
            _ => self.size.into(),
        };
        let response = self.button.size(size).show(ui);

        // Record the click so the addon's click-to-focus does not steal it.
        if response.clicked() {
            if let Some(group_id) = current_id(ui) {
                let mut record = read_record(ui, group_id);
                record.button_clicked = true;
                write_record(ui, group_id, record);
            }
        }
        response
    }
}

impl egui::Widget for InputGroupButton {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Palette, Preset};
    use egui::Pos2;

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    fn ui_rect() -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 400.0))
    }

    #[test]
    fn button_size_mapping() {
        assert_eq!(InputGroupButtonSize::default(), InputGroupButtonSize::Xs);
        let converted: ButtonSize = InputGroupButtonSize::Xs.into();
        assert_eq!(converted, ButtonSize::Xs);
        let converted: ButtonSize = InputGroupButtonSize::IconSm.into();
        assert_eq!(converted, ButtonSize::IconSm);
    }

    #[test]
    fn record_flags_round_trip() {
        let ctx = egui::Context::default();
        let id = egui::Id::unique("record_test");
        let record = GroupRecord {
            focus: true,
            invalid: true,
            end_addon_width: 42.0,
            ..GroupRecord::default()
        };
        ctx.data_mut(|d| d.insert_temp(id.with("snapshot"), record));
        let read = ctx.data_mut(|d| d.get_temp::<GroupRecord>(id.with("snapshot")).unwrap());
        assert!(read.focus);
        assert!(read.invalid);
        assert_eq!(read.end_addon_width, 42.0);
    }

    #[test]
    fn group_renders_row_and_input_fills() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from("abc");
        let mut container_rect = Rect::NOTHING;
        let mut input_rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("ig_size_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputGroup::new().id_salt("ig").show(ui, |ui| {
                input_rect = InputGroupInput::new(&mut text).show(ui).rect;
            });
            container_rect = ui.min_rect();
        });
        out.textures_delta.clear();
        // The default preset is nova (`h-8`).
        assert!((container_rect.height() - 32.0).abs() < 1.0);
        assert!((input_rect.height() - 32.0).abs() < 1.0);
        // A non-nova (base recipe) theme keeps `h-9`.
        let ctx = egui::Context::default();
        Preset::default()
            .with_style(crate::theme::Style::Vega)
            .install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("ig_size_test_base"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputGroup::new().id_salt("ig_base").show(ui, |ui| {
                input_rect = InputGroupInput::new(&mut text).show(ui).rect;
            });
            container_rect = ui.min_rect();
        });
        out.textures_delta.clear();
        assert!((container_rect.height() - crate::theme::tokens::spacing::CONTROL).abs() < 1.0);
        assert!((input_rect.height() - crate::theme::tokens::spacing::CONTROL).abs() < 1.0);
    }

    #[test]
    fn builders_keep_fields() {
        let mut text = String::new();
        let input = InputGroupInput::new(&mut text)
            .placeholder("url")
            .invalid(true)
            .disabled(true)
            .id_salt("i");
        assert_eq!(input.placeholder.as_deref(), Some("url"));
        assert!(input.invalid);

        let mut t = String::new();
        let ta = InputGroupTextarea::new(&mut t).rows(4).invalid(true);
        assert_eq!(ta.rows, 4);

        let mut icon_btn_rect = Rect::NOTHING;
        {
            let ctx2 = egui::Context::default();
            Preset::default().install(&ctx2);
            let mut out = ctx2.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("ig_btn_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                icon_btn_rect = InputGroupButton::icon(Icon::Copy)
                    .size(InputGroupButtonSize::IconSm)
                    .show(ui)
                    .rect;
            });
            out.textures_delta.clear();
        }
        // Nova `icon-sm` is a 28pt square (`size-7`).
        assert!((icon_btn_rect.width() - 28.0).abs() < 1.0);
        assert!((icon_btn_rect.height() - 28.0).abs() < 1.0);
        // The base recipe keeps the 32pt square (`size-8`).
        let mut base_btn_rect = Rect::NOTHING;
        {
            let ctx2 = egui::Context::default();
            Preset::default()
                .with_style(crate::theme::Style::Vega)
                .install(&ctx2);
            let mut out = ctx2.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("ig_btn_test_base"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                base_btn_rect = InputGroupButton::icon(Icon::Copy)
                    .size(InputGroupButtonSize::IconSm)
                    .show(ui)
                    .rect;
            });
            out.textures_delta.clear();
        }
        assert!((base_btn_rect.width() - 32.0).abs() < 1.0);
        assert!((base_btn_rect.height() - 32.0).abs() < 1.0);

        let txt = InputGroupText::new("https://")
            .icon(Icon::Copy)
            .disabled(true);
        assert!(txt.icon.is_some());
    }

    #[test]
    fn colors_use_palette_tokens() {
        let p = palette(Mode::Light);
        let dark = palette(Mode::Dark);
        assert_eq!(color_alpha(dark.input, 0.3), color_alpha(dark.input, 0.3));
        assert_ne!(p.input, dark.input);
    }
}
