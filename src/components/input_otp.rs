//! shadcn `input-otp.tsx` — one-time-password input.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/input-otp.tsx`. The
//! container owns a fixed-length character buffer and mirrors the source's
//! context-based subcomponents: [`InputOTPGroup`] (a run of slots sharing
//! borders) and [`InputOTPSeparator`] (a minus divider). Slots are
//! `h-9 w-9 border-y border-r border-input` squares (`text-sm shadow-xs`,
//! first/last of a group carry the left border and rounded-md corners,
//! `dark:bg-input/30`); the active slot gets `border-ring ring-[3px]
//! ring-ring/50` and a blinking 1px caret (`animate-caret-blink
//! bg-foreground h-4 w-px`).
//!
//! Keyboard (per the source's `input-otp` behavior): characters fill the
//! focused slot and advance the caret, Backspace clears and moves back,
//! ArrowLeft/ArrowRight move the caret, and paste distributes across slots.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut code = String::new();
//! let out = InputOTP::new(&mut code)
//!     .length(6)
//!     .numeric(true)
//!     .id_salt("otp")
//!     .show(ui);
//! if out.changed {
//!     // code updated
//! }
//!
//! // Custom layout with separators, mirroring the source composition:
//! InputOTP::new(&mut code).length(6).id_salt("otp2").show_with(ui, |ui| {
//!     InputOTPGroup::new().show(ui, |ui| {
//!         InputOTPSlot::new(0).show(ui);
//!         InputOTPSlot::new(1).show(ui);
//!         InputOTPSlot::new(2).show(ui);
//!     });
//!     InputOTPSeparator::new().show(ui);
//!     InputOTPGroup::new().show(ui, |ui| {
//!         InputOTPSlot::new(3).show(ui);
//!         InputOTPSlot::new(4).show(ui);
//!         InputOTPSlot::new(5).show(ui);
//!     });
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use egui::{Align2, CornerRadius, Event, Id, Key, Rect, Response, Sense, Stroke, Ui, Vec2};

/// `h-9 w-9` — the slot square side.
pub const OTP_SLOT_SIZE: f32 = 36.0;
/// `h-4` — the caret height.
pub const OTP_CARET_HEIGHT: f32 = 16.0;

/// Shared OTP context passed to slots through the temp stack.
#[derive(Clone, Copy, Debug)]
struct OtpCtx {
    group_id: Id,
    length: usize,
    numeric: bool,
    disabled: bool,
    invalid: bool,
}

/// One group's slot range, resolved (last frame) from the group registry.
#[derive(Clone, Copy, Debug, PartialEq)]
struct GroupRange {
    first: usize,
    last: usize,
}

fn ctx_stack_id() -> Id {
    Id::unique("shadcn_otp_ctx_stack")
}

fn group_stack_id() -> Id {
    Id::unique("shadcn_otp_group_stack")
}

/// Registry of group ranges per OTP instance: `(otp id, one entry per group
/// in render order)`. Groups overwrite their entry each frame, so slots read
/// the previous frame's layout.
fn group_registry_id() -> Id {
    Id::unique("shadcn_otp_group_ranges")
}

fn chars_id(otp_id: Id) -> Id {
    otp_id.with("chars")
}

fn counter_id(otp_id: Id) -> Id {
    otp_id.with("group_counter")
}

/// Which slot consumed the keyboard events this frame. Slots activated
/// mid-frame (focus was just handed to them) must not re-consume the same
/// events.
fn handled_id(otp_id: Id) -> Id {
    otp_id.with("key_handled")
}

fn push_ctx(ui: &Ui, ctx: OtpCtx) {
    let mut stack: Vec<OtpCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctx_stack_id()).unwrap_or_default());
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(ctx_stack_id(), stack));
}

fn pop_ctx(ui: &Ui) {
    let mut stack: Vec<OtpCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(ctx_stack_id()).unwrap_or_default());
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(ctx_stack_id(), stack));
}

fn current_ctx(ui: &Ui) -> Option<OtpCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<OtpCtx>>(ctx_stack_id()))
        .and_then(|stack| stack.last().copied())
}

/// Result of an [`InputOTP`] render.
#[derive(Clone, Copy, Debug)]
pub struct OtpOutput {
    /// The bound string changed this frame.
    pub changed: bool,
    /// Any slot held keyboard focus this frame.
    pub focused: bool,
}

/// A shadcn OTP input. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct InputOTP<'a> {
    value: &'a mut String,
    length: usize,
    numeric: bool,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> InputOTP<'a> {
    /// Creates an OTP input bound to the given string.
    pub fn new(value: &'a mut String) -> Self {
        Self {
            value,
            length: 6,
            numeric: false,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Sets the number of slots (`maxLength`; default 6).
    pub fn length(mut self, length: usize) -> Self {
        self.length = length.max(1);
        self
    }

    /// Restricts input to digits (`inputMode="numeric"`).
    pub fn numeric(mut self, numeric: bool) -> Self {
        self.numeric = numeric;
        self
    }

    /// Disables the input (`has-disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the input aria-invalid (`aria-invalid:border-destructive`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so the buffer, caret and focus persist per
    /// instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the input as a single [`InputOTPGroup`] of `length` slots.
    pub fn show(self, ui: &mut Ui) -> OtpOutput {
        let length = self.length;
        self.show_with(ui, |ui| {
            InputOTPGroup::new().show(ui, |ui| {
                for i in 0..length {
                    InputOTPSlot::new(i).show(ui);
                }
            });
        })
    }

    /// Renders the input with a custom layout of groups and separators.
    pub fn show_with<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> OtpOutput {
        let group_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input_otp"),
        };

        // Sync the working buffer from the bound string.
        let mut chars: Vec<char> = self.value.chars().take(self.length).collect();
        chars.resize(self.length, '\0');
        ui.ctx()
            .data_mut(|d| d.insert_temp(chars_id(group_id), chars));

        // Restart the group order counter for this frame; the ranges from
        // the previous frame stay available for slot styling. Also reset the
        // per-frame key-handling marker.
        ui.ctx().data_mut(|d| {
            d.insert_temp(counter_id(group_id), 0usize);
            // `usize::MAX` = "no slot consumed the keyboard yet this frame".
            // NOTE: egui's temp map keys include the value type, so this
            // reset must use the same `usize` type the slots read.
            d.insert_temp::<usize>(handled_id(group_id), usize::MAX);
        });

        push_ctx(
            ui,
            OtpCtx {
                group_id,
                length: self.length,
                numeric: self.numeric,
                disabled: self.disabled,
                invalid: self.invalid,
            },
        );

        // `flex items-center gap-2` — the container row.
        let _inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::SM;
            content(ui);
        });

        pop_ctx(ui);

        // Drop stale group entries beyond this frame's group count.
        let group_count: usize = ui
            .ctx()
            .data_mut(|d| d.get_temp(counter_id(group_id)).unwrap_or(0));
        ui.ctx().data_mut(|d| {
            let mut registry: Vec<(Id, Vec<GroupRange>)> =
                d.get_temp(group_registry_id()).unwrap_or_default();
            if let Some(entry) = registry.iter_mut().find(|(id, _)| *id == group_id) {
                entry.1.truncate(group_count);
            }
            d.insert_temp(group_registry_id(), registry);
        });

        // Read back the buffer and write it into the bound string.
        let after: Vec<char> = ui.ctx().data_mut(|d| {
            d.get_temp::<Vec<char>>(chars_id(group_id))
                .unwrap_or_default()
        });
        let text: String = after.iter().filter(|c| **c != '\0').collect();
        let changed = text != *self.value;
        if changed {
            *self.value = text;
        }
        let focused = ui
            .ctx()
            .memory(|m| (0..self.length).any(|i| m.has_focus(group_id.with("slot").with(i))));

        OtpOutput { changed, focused }
    }
}

/// A run of slots sharing borders (`InputOTPGroup`).
///
/// Registers its slot range so the first/last slots get the left border and
/// the `rounded-l-md` / `rounded-r-md` corners (resolved on the next frame;
/// before that the slots fall back to whole-input first/last styling).
#[derive(Clone, Debug, Default)]
pub struct InputOTPGroup {
    id_salt: Option<String>,
}

impl InputOTPGroup {
    /// Creates a group.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its slots (`flex items-center`).
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let Some(ctx) = current_ctx(ui) else {
            return ui.horizontal(content);
        };

        // This group's order within the instance, and the temp id its slots
        // report their indices to.
        let order: usize = ui.ctx().data_mut(|d| {
            let n: usize = d.get_temp(counter_id(ctx.group_id)).unwrap_or(0);
            d.insert_temp(counter_id(ctx.group_id), n + 1);
            n
        });
        let seen_id = ctx
            .group_id
            .with("group_seen")
            .with(self.id_salt.as_deref().unwrap_or(""))
            .with(order);
        ui.ctx()
            .data_mut(|d| d.insert_temp::<Vec<usize>>(seen_id, Vec::new()));

        let mut stack: Vec<Id> = ui
            .ctx()
            .data_mut(|d| d.get_temp(group_stack_id()).unwrap_or_default());
        stack.push(seen_id);
        ui.ctx()
            .data_mut(|d| d.insert_temp(group_stack_id(), stack));

        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            content(ui)
        });

        let mut stack: Vec<Id> = ui
            .ctx()
            .data_mut(|d| d.get_temp(group_stack_id()).unwrap_or_default());
        stack.pop();
        ui.ctx()
            .data_mut(|d| d.insert_temp(group_stack_id(), stack));

        let seen: Vec<usize> = ui
            .ctx()
            .data_mut(|d| d.get_temp(seen_id).unwrap_or_default());
        if seen.len() >= 2 {
            let first = seen.iter().min().copied().unwrap_or(0);
            let last = seen.iter().max().copied().unwrap_or(0);
            ui.ctx().data_mut(|d| {
                let mut registry: Vec<(Id, Vec<GroupRange>)> =
                    d.get_temp(group_registry_id()).unwrap_or_default();
                let entry = registry
                    .iter_mut()
                    .find(|(id, _)| *id == ctx.group_id)
                    .map(|(_, ranges)| ranges);
                match entry {
                    Some(ranges) => {
                        if let Some(slot) = ranges.get_mut(order) {
                            *slot = GroupRange { first, last };
                        } else {
                            ranges.push(GroupRange { first, last });
                        }
                    }
                    None => {
                        let mut ranges = Vec::new();
                        ranges.resize_with(order, || GroupRange { first: 0, last: 0 });
                        ranges.push(GroupRange { first, last });
                        registry.push((ctx.group_id, ranges));
                    }
                }
                d.insert_temp(group_registry_id(), registry);
            });
        }

        inner
    }
}

/// One OTP character slot (`InputOTPSlot`).
#[derive(Clone, Debug)]
pub struct InputOTPSlot {
    index: usize,
    id_salt: Option<String>,
}

impl InputOTPSlot {
    /// Creates the slot at the given buffer index.
    pub fn new(index: usize) -> Self {
        Self {
            index,
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the slot.
    pub fn show(self, ui: &mut Ui) -> Response {
        let Some(ctx) = current_ctx(ui) else {
            // Rendered outside an InputOTP container: plain static box.
            let (rect, response) =
                ui.allocate_exact_size(Vec2::splat(OTP_SLOT_SIZE), Sense::hover());
            if ui.is_rect_visible(rect) {
                let theme = ShadcnTheme::of_ui(ui);
                ui.painter().rect_stroke(
                    rect,
                    theme.radius.md_u8(),
                    Stroke::new(1.0, theme.palette.input),
                    egui::epaint::StrokeKind::Inside,
                );
            }
            return response;
        };
        let OtpCtx {
            group_id,
            length,
            numeric,
            disabled,
            invalid,
        } = ctx;
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let id = match self.id_salt.as_deref() {
            Some(salt) => group_id.with("slot_salt").with(salt),
            None => group_id.with("slot").with(self.index),
        };

        // Report this index to the enclosing group (for first/last corners).
        if self.id_salt.is_none() {
            if let Some(seen_id) = ui.ctx().data_mut(|d| {
                let stack: Vec<Id> = d.get_temp(group_stack_id()).unwrap_or_default();
                stack.last().copied()
            }) {
                ui.ctx().data_mut(|d| {
                    let mut seen: Vec<usize> = d.get_temp(seen_id).unwrap_or_default();
                    seen.push(self.index);
                    d.insert_temp(seen_id, seen);
                });
            }
        }

        let sense = if disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let (_, rect) = ui.allocate_space(Vec2::splat(OTP_SLOT_SIZE));
        let response = ui.interact(rect, id, sense);

        let chars: Vec<char> = ui.ctx().data_mut(|d| {
            d.get_temp::<Vec<char>>(chars_id(group_id))
                .unwrap_or_default()
        });
        let mut chars = chars;
        if chars.len() != length {
            chars.resize(length, '\0');
        }

        let mut changed = false;
        let mut focus_target: Option<usize> = None;
        let active = response.has_focus() && !disabled;
        let already_handled: Option<usize> = ui
            .ctx()
            .data_mut(|d| d.get_temp::<usize>(handled_id(group_id)));
        let handles_keys = active && already_handled.is_none_or(|i| i == usize::MAX);
        if handles_keys {
            ui.ctx()
                .data_mut(|d| d.insert_temp(handled_id(group_id), self.index));
            // Keyboard handling for the focused slot.
            let events: Vec<Event> = ui.input(|i| i.events.clone());
            for event in events {
                match event {
                    Event::Text(text) => {
                        for ch in text.chars().filter(|c| !c.is_control()) {
                            if numeric && !ch.is_ascii_digit() {
                                continue;
                            }
                            if self.index < length {
                                chars[self.index] = ch;
                                changed = true;
                                focus_target = Some((self.index + 1).min(length - 1));
                            }
                        }
                    }
                    Event::Paste(text) => {
                        for (offset, ch) in text.chars().filter(|c| !c.is_control()).enumerate() {
                            let idx = self.index + offset;
                            if idx < length && (!numeric || ch.is_ascii_digit()) {
                                chars[idx] = ch;
                                changed = true;
                            }
                        }
                        focus_target = Some((self.index + text.chars().count()).min(length - 1));
                    }
                    Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } => match key {
                        Key::Backspace => {
                            if chars[self.index] != '\0' {
                                chars[self.index] = '\0';
                                changed = true;
                            } else if self.index > 0 {
                                let prev = self.index - 1;
                                chars[prev] = '\0';
                                changed = true;
                                focus_target = Some(prev);
                            }
                        }
                        Key::ArrowLeft if self.index > 0 => {
                            focus_target = Some(self.index - 1);
                        }
                        Key::ArrowRight if self.index + 1 < length => {
                            focus_target = Some(self.index + 1);
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }
        }

        if let Some(target) = focus_target {
            ui.memory_mut(|mem| mem.request_focus(group_id.with("slot").with(target)));
        }

        // Persist buffer mutations back to the shared temp.
        if changed {
            ui.ctx()
                .data_mut(|d| d.insert_temp(chars_id(group_id), chars.clone()));
        }

        if ui.is_rect_visible(rect) {
            // Corner resolution: last frame's group ranges, defaulting to a
            // single group covering the whole input.
            let (is_first, is_last) = ui.ctx().data_mut(|d| {
                let registry: Vec<(Id, Vec<GroupRange>)> =
                    d.get_temp(group_registry_id()).unwrap_or_default();
                let ranges = registry
                    .iter()
                    .find(|(id, _)| *id == group_id)
                    .map(|(_, r)| r.clone())
                    .unwrap_or_else(|| {
                        vec![GroupRange {
                            first: 0,
                            last: length - 1,
                        }]
                    });
                let range = ranges
                    .iter()
                    .find(|r| self.index >= r.first && self.index <= r.last)
                    .copied();
                (
                    range.map(|r| r.first == self.index).unwrap_or(false),
                    range.map(|r| r.last == self.index).unwrap_or(false),
                )
            });

            // `border-y border-r border-input` (first also gets border-l).
            let stroke = if invalid {
                Stroke::new(1.0, p.destructive)
            } else if active {
                Stroke::new(1.0, p.ring)
            } else {
                Stroke::new(1.0, p.input)
            };
            let radius = theme.radius.md_u8();
            let mut corner = CornerRadius {
                nw: 0,
                ne: 0,
                sw: 0,
                se: 0,
            };
            if is_first {
                corner.nw = radius;
                corner.sw = radius;
            }
            if is_last {
                corner.ne = radius;
                corner.se = radius;
            }

            let painter = ui.painter();
            // `shadow-xs`
            if !disabled {
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
            // `dark:bg-input/30`
            if dark {
                painter.rect_filled(rect, corner, color_alpha(p.input, 0.3));
            }
            painter.rect_stroke(rect, corner, stroke, egui::epaint::StrokeKind::Inside);

            // Character or caret.
            let ch = chars.get(self.index).copied().unwrap_or('\0');
            let alpha = if disabled { 0.5 } else { 1.0 };
            if ch != '\0' {
                let font = egui::FontId::proportional(crate::theme::tokens::type_scale::SM);
                painter.text(
                    rect.center(),
                    Align2::CENTER_CENTER,
                    ch.to_string(),
                    font,
                    color_alpha(p.foreground, alpha),
                );
            } else if active {
                // `h-4 w-px animate-caret-blink bg-foreground` — 1 Hz blink.
                let visible = ui.input(|i| (i.time * 2.0) % 2.0 < 1.0);
                if visible {
                    let caret =
                        Rect::from_center_size(rect.center(), Vec2::new(1.0, OTP_CARET_HEIGHT));
                    painter.rect_filled(caret, 0, color_alpha(p.foreground, alpha));
                }
            }

            // `data-[active=true]:ring-[3px] data-[active=true]:ring-ring/50`
            if active {
                let ring_color = if invalid {
                    color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
                } else {
                    color_alpha(p.ring, 0.5)
                };
                crate::utils::focus::paint_ring(
                    painter,
                    rect.expand2(egui::vec2(1.0, 1.0)),
                    ring_color,
                    3.0,
                    radius as f32,
                );
            }
        }

        response
    }
}

/// The divider between groups (`InputOTPSeparator`) — a lucide minus icon.
#[derive(Clone, Debug, Default)]
pub struct InputOTPSeparator {
    id_salt: Option<String>,
}

impl InputOTPSeparator {
    /// Creates a separator.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the separator.
    pub fn show(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(
            Vec2::splat(crate::theme::tokens::spacing::LG),
            Sense::hover(),
        );
        if ui.is_rect_visible(rect) {
            let theme = ShadcnTheme::of_ui(ui);
            Icon::Minus.paint(
                ui.painter(),
                rect,
                theme.palette.muted_foreground,
                2.0 * crate::theme::tokens::spacing::LG / 24.0,
            );
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Pos2;

    fn ui_rect() -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 200.0))
    }

    #[test]
    fn slot_geometry_matches_source() {
        assert_eq!(OTP_SLOT_SIZE, 36.0); // h-9 w-9
        assert_eq!(OTP_CARET_HEIGHT, 16.0); // h-4
    }

    #[test]
    fn show_truncates_long_values() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = "1234567890".to_owned(); // longer than length
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_len_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            let o = InputOTP::new(&mut value).length(4).show(ui);
            // The value is longer than `length`; it is truncated on sync.
            assert!(o.changed);
        });
        out.textures_delta.clear();
        assert_eq!(value, "1234");
    }

    #[test]
    fn container_reports_focus_and_change() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_focus_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            let o = InputOTP::new(&mut value).length(4).show(ui);
            assert!(!o.focused);
            assert!(!o.changed);
        });
        out.textures_delta.clear();
        assert!(value.is_empty());
    }

    #[test]
    fn text_event_fills_slot_and_advances_caret() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let base = Id::unique("otp_type_test").with("otp");
        let slot0 = base.with("slot").with(0usize);

        // Run one frame, then focus slot 0.
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_type_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputOTP::new(&mut value).length(4).id_salt("otp").show(ui);
        });
        out.textures_delta.clear();
        ctx.memory_mut(|mem| mem.request_focus(slot0));

        // Type "42": fills slots 0 and 1, caret advances to slot 2.
        for ch in ["4", "2"] {
            let mut input = egui::RawInput::default();
            input.events.push(Event::Text((*ch).to_owned()));
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("otp_type_test"),
                    egui::UiBuilder::new().max_rect(ui_rect()),
                );
                InputOTP::new(&mut value).length(4).id_salt("otp").show(ui);
            });
            out.textures_delta.clear();
        }
        assert_eq!(value, "42");
        assert!(ctx.memory(|m| m.has_focus(base.with("slot").with(2usize))));
    }

    #[test]
    fn numeric_filter_ignores_letters() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let base = Id::unique("otp_numeric_test").with("otp");
        let slot0 = base.with("slot").with(0usize);

        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_numeric_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputOTP::new(&mut value).length(3).id_salt("otp").show(ui);
        });
        out.textures_delta.clear();
        ctx.memory_mut(|mem| mem.request_focus(slot0));

        let mut input = egui::RawInput::default();
        input.events.push(Event::Text("a7".to_owned()));
        let mut out = ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_numeric_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputOTP::new(&mut value).length(3).id_salt("otp").show(ui);
        });
        out.textures_delta.clear();
        assert_eq!(value, "7");
    }

    #[test]
    fn backspace_clears_and_moves_back() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = "12".to_owned();
        let base = Id::unique("otp_bs_test").with("otp");
        let slot2 = base.with("slot").with(2usize);

        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_bs_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputOTP::new(&mut value).length(4).id_salt("otp").show(ui);
        });
        out.textures_delta.clear();
        ctx.memory_mut(|mem| mem.request_focus(slot2));

        let mut input = egui::RawInput::default();
        input.events.push(Event::Key {
            key: Key::Backspace,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        });
        let mut out = ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_bs_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputOTP::new(&mut value).length(4).id_salt("otp").show(ui);
        });
        out.textures_delta.clear();
        // Slot 2 was empty, so backspace cleared slot 1 and moved back.
        assert_eq!(value, "1");
        assert!(ctx.memory(|m| m.has_focus(base.with("slot").with(1usize))));
    }

    #[test]
    fn builders_keep_fields() {
        let mut value = String::new();
        let otp = InputOTP::new(&mut value)
            .length(4)
            .numeric(true)
            .disabled(true)
            .invalid(true)
            .id_salt("otp");
        assert_eq!(otp.length, 4);
        assert!(otp.numeric);
        assert!(otp.disabled);
        assert!(otp.invalid);
        assert_eq!(otp.id_salt.as_deref(), Some("otp"));
        let mut v2 = String::new();
        assert_eq!(InputOTP::new(&mut v2).length(0).length, 1); // clamped
    }

    #[test]
    fn dark_mode_uses_input_30_fill() {
        let light = Preset::default().build(Mode::Light).palette;
        let dark = Preset::default().build(Mode::Dark).palette;
        assert_eq!(color_alpha(dark.input, 0.3), color_alpha(dark.input, 0.3));
        assert_ne!(light.input, dark.input);
    }
}
