//! shadcn `input-otp.tsx` — one-time-password input.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/input-otp.tsx`. The
//! container owns a fixed-length value and mirrors the source's
//! context-based subcomponents: [`InputOTPGroup`] (a run of slots sharing
//! borders) and [`InputOTPSeparator`] (a minus divider). Slots are
//! `h-9 w-9 border-y border-r border-input` squares (`text-sm shadow-xs`,
//! first/last of a group carry the left border and rounded-md corners,
//! `dark:bg-input/30`); the active slot gets `border-ring ring-[3px]
//! ring-ring/50` and a blinking 1px caret (`animate-caret-blink
//! bg-foreground h-4 w-px`).
//!
//! The nova style (`.cn-input-otp-slot`) shrinks the slots to `size-8`
//! (32px) with `rounded-l-lg`/`rounded-r-lg` group ends, and rings the
//! whole group with `ring-3 ring-destructive/20` (dark /40) when any slot
//! is `aria-invalid`; `has-disabled:opacity-50` fades the whole control
//! (borders, fills, shadow, chars, caret, separator and the group ring).
//!
//! # Behaviour — from the `input-otp` library source (v1.4.2)
//!
//! The snapshot pins no library version, so the behaviour below is taken
//! from `input-otp` 1.4.2 (commit `81ccdb48`, 2024-12-30 — the version
//! current during the shadcn v4 snapshot era; npm latest 1.5.0 postdates
//! the snapshot by over a year and was not used). The library renders ONE
//! invisible native `<input>` over the whole container (the slots are
//! pure render mirrors), so:
//!
//! * One Tab stop for the whole control. Slots are hover-only; the
//!   container owns focus (`control` id). A disabled control has no Tab
//!   stop and no active slot.
//! * The text selection is `(start, end)` in char positions. Focus-gain
//!   sets it to `(min(len, maxLength - 1), len)` — the first empty slot,
//!   or a 1-char range on the last slot when full. Clicking an already
//!   focused control places a collapsed caret at the clicked slot.
//! * Typing inserts at the caret (native input, truncated to `maxLength`)
//!   and advances it; typing the last char leaves the caret past the end
//!   (no active slot). A 1-char selection is replaced, which is the same
//!   result as inserting at the last slot and truncating.
//! * Paste inserts at the selection (replacing a range, if any) and lands
//!   the caret after the last inserted char. The whole new value is tested
//!   against the pattern: one bad char rejects the ENTIRE paste. An empty
//!   paste is a no-op. (The library also has an iOS / `pasteTransformer`
//!   path that lands the selection at `(min(new_len, maxLength - 1),
//!   new_len)`; egui has no iOS paste quirk, so the desktop native path
//!   above is the reference.)
//! * Backspace deletes the selection, else the char before the caret, and
//!   moves the caret back; Delete deletes the selection, else the char at
//!   the caret, and stays. Both shift the remaining chars left — the value
//!   is always a prefix, never a buffer with holes. Arrows move the
//!   collapsed caret and stop at the ends (no wrap); on a range, Left
//!   collapses to the range start and Right to the range end. Up/Down,
//!   Enter and Escape are no-ops. Repeats are honoured (native
//!   hold-to-repeat).
//! * The pattern is the only filter, tested against the whole value;
//!   `.numeric()` is the library's `REGEXP_ONLY_DIGITS` preset
//!   (`^\d+$`). `inputMode="numeric"` is a mobile-keyboard hint, not a
//!   filter. `onComplete` is reported as `OtpOutput.completed` (true on
//!   the frame the value transitions to full).
//! * A slot is active when focused and the selection covers it (a
//!   collapsed caret at the slot, or a range covering it); the caret
//!   renders only on an active EMPTY slot (`hasFakeCaret`), so a char and
//!   a caret never co-render. The caret blinks with a 1s period and a 50%
//!   square-wave duty — a documented assumption (OPEN-6: the duty is
//!   unestablished; the period is frozen at 1s by `duration-1000`).
//!
//! # Borders (N7)
//!
//! `border-y border-r border-input` + `first:border-l`: the first slot of
//! a group paints all four edges, every other slot paints top, bottom and
//! right only — each internal divider is the earlier slot's right edge,
//! painted exactly once (see `paint_otp_fused_stroke`, which mirrors the
//! crate's fused-stroke precedent). The group paints all chrome
//! post-closure, in CSS order: non-active slots, then the active slot(s)
//! (`z-10`), then the active rings, then the nova group-invalid ring.
//!
//! # What the port does not surface
//!
//! * `placeholder` — the library renders `placeholderChar` only when the
//!   value is completely empty, but the frozen `.tsx` reads only `char`,
//!   `hasFakeCaret` and `isActive` from the context, so no placeholder
//!   API is offered.
//! * A general `pattern` prop — it would need a regex engine; only the
//!   `^\d+$` numeric preset is offered.
//! * `autoComplete="one-time-code"`, `pushPasswordManagerStrategy` and
//!   the password-manager badge — browser chrome, not portable to egui.
//! * `textAlign` — not observable in the fixed-box slot render.
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
//! if out.completed {
//!     // the code is complete (the library's `onComplete`)
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
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{
    Align2, Color32, CornerRadius, Event, Id, Key, Painter, Pos2, Rect, Response, Sense, Stroke,
    Ui, Vec2,
};

/// `h-9 w-9` — the slot square side in the base recipe. The nova style is
/// 32px (`size-8`).
pub const OTP_SLOT_SIZE: f32 = 36.0;
/// `h-4` — the caret height.
pub const OTP_CARET_HEIGHT: f32 = 16.0;

/// Slot square side (`h-9 w-9` base recipe; nova `.cn-input-otp-slot`
/// `size-8`).
pub(crate) fn slot_size(style: Style) -> f32 {
    match style {
        Style::Nova => 32.0, // size-8
        _ => OTP_SLOT_SIZE,  // h-9 w-9
    }
}

/// Shared OTP context passed to groups and slots through the temp stack.
#[derive(Clone, Copy, Debug)]
struct OtpCtx {
    group_id: Id,
    disabled: bool,
    invalid: bool,
    /// The text selection `(start, end)` in char positions
    /// (`start <= end <= length`); collapsed when `start == end`.
    sel: (usize, usize),
    /// The control held keyboard focus when this frame's layout ran.
    focused: bool,
}

fn ctx_stack_id() -> Id {
    Id::unique("shadcn_otp_ctx_stack")
}

fn group_stack_id() -> Id {
    Id::unique("shadcn_otp_group_stack")
}

fn value_id(otp_id: Id) -> Id {
    otp_id.with("value")
}

fn sel_id(otp_id: Id) -> Id {
    otp_id.with("sel")
}

fn was_focused_id(otp_id: Id) -> Id {
    otp_id.with("was_focused")
}

fn container_rect_id(otp_id: Id) -> Id {
    otp_id.with("container_rect")
}

/// All `(index, rect)` pairs recorded this frame, in group order — the
/// click-to-place map for the next frame's container interact.
fn slot_rects_id(otp_id: Id) -> Id {
    otp_id.with("slot_rects")
}

fn counter_id(otp_id: Id) -> Id {
    otp_id.with("group_counter")
}

/// The container's focus id — the control's single Tab stop.
fn control_id(otp_id: Id) -> Id {
    otp_id.with("control")
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

/// Which edges of a slot carry a border — `border-y border-r
/// border-input`, plus `first:border-l`: the first slot of a group paints
/// all four, every other slot paints top, bottom and right only (N7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SlotEdges {
    left: bool,
    top: bool,
    right: bool,
    bottom: bool,
}

fn slot_edges(is_first: bool) -> SlotEdges {
    SlotEdges {
        left: is_first,
        top: true,
        right: true,
        bottom: true,
    }
}

/// The slot border colour: invalid beats active (`aria-invalid` +
/// `data-[active=true]:aria-invalid:border-destructive`).
fn slot_border_color(invalid: bool, active: bool, p: Palette) -> Color32 {
    if invalid {
        p.destructive
    } else if active {
        p.ring
    } else {
        p.input
    }
}

/// The active ring colour: `ring-ring/50`, or `ring-destructive/20`
/// (dark /40) when invalid.
fn slot_ring_color(invalid: bool, dark: bool, p: Palette) -> Color32 {
    if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    }
}

/// The slot corner radii: first carries the left corners, last the right
/// corners (`first:rounded-l-md` / `last:rounded-r-md`, nova `lg`).
fn slot_corners(is_first: bool, is_last: bool, radius: u8) -> CornerRadius {
    CornerRadius {
        nw: if is_first { radius } else { 0 },
        sw: if is_first { radius } else { 0 },
        ne: if is_last { radius } else { 0 },
        se: if is_last { radius } else { 0 },
    }
}

/// The centre of a slot's content box (the border box minus its 1px
/// borders): chars and the caret centre here, not on the border-box
/// centre — the first slot's content is 1px narrower on the left, so its
/// centre sits 0.5px right of the middle/last slots'.
fn content_center(rect: Rect, is_first: bool) -> Pos2 {
    let left = rect.left() + if is_first { 1.0 } else { 0.0 };
    let right = rect.right() - 1.0;
    Pos2::new((left + right) / 2.0, rect.center().y)
}

/// The selection the library's `_focusListener` sets on focus-gain:
/// `(min(len, maxLength - 1), len)` — a collapsed caret at the first
/// empty slot, or a 1-char range on the last slot when full.
fn focus_gain_sel(value_len: usize, length: usize) -> (usize, usize) {
    (
        value_len.min(length.saturating_sub(1)),
        value_len.min(length),
    )
}

/// A slot is active when the control is focused and the selection covers
/// it — a collapsed caret at the slot, or a range covering it (the
/// library's `isActive`).
fn is_slot_active(index: usize, sel: (usize, usize), focused: bool, disabled: bool) -> bool {
    focused
        && !disabled
        && ((sel.0 == sel.1 && index == sel.0) || (sel.0 <= index && index < sel.1))
}

/// The dark fill alpha: `bg-input/30`, halved by the
/// `has-disabled:opacity-50` container fade.
fn slot_fill_alpha(disabled: bool) -> f32 {
    if disabled {
        0.15
    } else {
        0.3
    }
}

/// The slot whose rect contains `pos`, else the slot whose centre is
/// nearest — the click-to-place target.
fn nearest_slot(slots: &[(usize, Rect)], pos: Pos2) -> Option<usize> {
    if slots.is_empty() {
        return None;
    }
    if let Some(&(index, _)) = slots.iter().find(|(_, r)| r.contains(pos)) {
        return Some(index);
    }
    slots
        .iter()
        .min_by(|(_, a), (_, b)| {
            a.center()
                .distance_sq(pos)
                .total_cmp(&b.center().distance_sq(pos))
        })
        .map(|(index, _)| *index)
}

/// Paints a slot's 1px border with the LEFT edge omitted for non-first
/// group children (source `border-y border-r border-input` +
/// `first:border-l`), mirroring the foundation fusing fix in
/// [`crate::components::button::Button`] (`paint_fused_stroke`, BTG-VIS-01)
/// and `paint_toggle_fused_stroke` — both private, so the shape is
/// repeated here: the full `Inside` ring is painted through a clip that
/// excludes the leading 1px column, and once more clipped to each of the
/// two leading corner cells. The three clips tile the ring exactly: they
/// share only zero-area boundaries, and the renderer's scissor conversion
/// rounds every clip edge, so no pixel is painted twice — exact for
/// translucent strokes, where overpainting would double-blend.
/// First/only children take the plain `rect_stroke` path, identical to a
/// standalone slot.
///
/// The divider between two slots is the earlier slot's right edge,
/// painted exactly once (N7): the later slot's corner cells repaint only
/// the top/bottom edge's first pixel, never the divider column. The
/// corner cells MUST be painted as clipped copies of the ring shape,
/// never as 1x1 fill rects: a 1x1 `rect_filled` does not rasterize in
/// egui 0.37 (BTG-VIS-01 corner-cell failure).
fn paint_otp_fused_stroke(
    painter: &Painter,
    rect: Rect,
    corner: CornerRadius,
    stroke: Stroke,
    edges: SlotEdges,
) {
    if edges.left {
        painter.rect_stroke(rect, corner, stroke, egui::epaint::StrokeKind::Inside);
        return;
    }
    // Always the left edge: the clip excludes the leading 1px column; the
    // corner cells are the two left cells.
    let clip = Rect::from_min_max(Pos2::new(rect.min.x + 1.0, rect.min.y), rect.max);
    let corners = [
        Rect::from_min_max(rect.min, Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0)),
        Rect::from_min_max(
            Pos2::new(rect.min.x, rect.max.y - 1.0),
            Pos2::new(rect.min.x + 1.0, rect.max.y),
        ),
    ];
    painter.with_clip_rect(clip).rect_stroke(
        rect,
        corner,
        stroke,
        egui::epaint::StrokeKind::Inside,
    );
    // Corner pixels: the identical ring, clipped to each leading corner
    // cell. See the doc comment — 1x1 fill rects do not rasterize.
    for corner_clip in corners {
        painter.with_clip_rect(corner_clip).rect_stroke(
            rect,
            corner,
            stroke,
            egui::epaint::StrokeKind::Inside,
        );
    }
}

/// Applies one frame's keyboard/paste events to the working value and
/// selection, per the `input-otp` 1.4.2 source (see the module docs):
/// typing inserts at the caret (replacing a range, if any), paste inserts
/// at the selection and is rejected ENTIRELY if any char fails the
/// numeric pattern, Backspace/Delete delete the selection or the
/// neighbouring char with a left-shift, arrows move the collapsed caret
/// (or collapse a range), and repeats are honoured. Returns `true` if
/// the value changed.
fn apply_otp_events(
    events: &[Event],
    value: &mut String,
    sel: &mut (usize, usize),
    length: usize,
    numeric: bool,
) -> bool {
    let original = value.clone();
    // Normalise the selection: in range, and inside the value.
    let value_len = value.chars().count();
    if sel.0 > sel.1 {
        sel.0 = sel.1;
    }
    sel.0 = sel.0.min(value_len);
    sel.1 = sel.1.min(value_len);
    for event in events {
        match event {
            Event::Text(text) => {
                for ch in text.chars().filter(|c| !c.is_control()) {
                    // The library tests the whole new value against the
                    // pattern; for a single inserted char that is exactly
                    // this per-char test.
                    if numeric && !ch.is_ascii_digit() {
                        continue;
                    }
                    let mut chars: Vec<char> = value.chars().collect();
                    let start = sel.0.min(chars.len());
                    let end = sel.1.min(chars.len());
                    chars.splice(start..end, [ch]);
                    chars.truncate(length);
                    *value = chars.into_iter().collect();
                    let at = (start + 1).min(length);
                    *sel = (at, at);
                }
            }
            Event::Paste(text) => {
                let content: Vec<char> = text.chars().filter(|c| !c.is_control()).collect();
                if content.is_empty() {
                    continue;
                }
                // One bad char rejects the ENTIRE paste (the library tests
                // the whole new value against the pattern).
                if numeric && content.iter().any(|c| !c.is_ascii_digit()) {
                    continue;
                }
                let mut chars: Vec<char> = value.chars().collect();
                let start = sel.0.min(chars.len());
                let end = sel.1.min(chars.len());
                chars.splice(start..end, content.iter().copied());
                chars.truncate(length);
                *value = chars.into_iter().collect();
                let at = (start + content.len()).min(length);
                *sel = (at, at);
            }
            Event::Key {
                key, pressed: true, ..
            } => match key {
                Key::Backspace => {
                    let mut chars: Vec<char> = value.chars().collect();
                    let start = sel.0.min(chars.len());
                    let end = sel.1.min(chars.len());
                    if start < end {
                        chars.drain(start..end);
                        *sel = (start, start);
                    } else if start > 0 {
                        chars.remove(start - 1);
                        *sel = (start - 1, start - 1);
                    } else {
                        continue;
                    }
                    *value = chars.into_iter().collect();
                }
                Key::Delete => {
                    let mut chars: Vec<char> = value.chars().collect();
                    let start = sel.0.min(chars.len());
                    let end = sel.1.min(chars.len());
                    if start < end {
                        chars.drain(start..end);
                        *sel = (start, start);
                    } else if start < chars.len() {
                        chars.remove(start);
                    } else {
                        continue;
                    }
                    *value = chars.into_iter().collect();
                }
                Key::ArrowLeft => {
                    let (start, end) = *sel;
                    let at = if start < end {
                        start
                    } else {
                        start.saturating_sub(1)
                    };
                    *sel = (at, at);
                }
                Key::ArrowRight => {
                    let (start, end) = *sel;
                    let at = if start < end {
                        end
                    } else {
                        (start + 1).min(length)
                    };
                    *sel = (at, at);
                }
                _ => {}
            },
            _ => {}
        }
    }
    *value != original
}

/// Result of an [`InputOTP`] render.
#[derive(Clone, Copy, Debug)]
pub struct OtpOutput {
    /// The bound string changed this frame.
    pub changed: bool,
    /// The control held keyboard focus this frame.
    pub focused: bool,
    /// The value transitioned to full this frame (the library's
    /// `onComplete`).
    pub completed: bool,
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

    /// Restricts input to digits — the library's `REGEXP_ONLY_DIGITS`
    /// pattern preset (`^\d+$`), tested against the whole value.
    /// `inputMode="numeric"` is a separate mobile-keyboard hint, not the
    /// filter.
    pub fn numeric(mut self, numeric: bool) -> Self {
        self.numeric = numeric;
        self
    }

    /// Disables the input (`has-disabled:opacity-50` — the whole control
    /// fades, and it has no Tab stop).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the input aria-invalid (`aria-invalid:border-destructive`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so the value, selection and focus persist
    /// per instance.
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
        let control = control_id(group_id);
        let length = self.length;

        // Sync the working value from the bound string (truncated to
        // `maxLength`).
        let synced: String = self.value.chars().take(length).collect();
        let prev_len = synced.chars().count();
        let mut value = synced.clone();
        ui.ctx()
            .data_mut(|d| d.insert_temp(value_id(group_id), value.clone()));

        // Selection + focus state from last frame.
        let mut sel: (usize, usize) = ui
            .ctx()
            .data_mut(|d| d.get_temp(sel_id(group_id)).unwrap_or((0, 0)));
        let was_focused: bool = ui
            .ctx()
            .data_mut(|d| d.get_temp(was_focused_id(group_id)).unwrap_or(false));
        let prev_slot_rects: Vec<(usize, Rect)> = ui
            .ctx()
            .data_mut(|d| d.get_temp(slot_rects_id(group_id)).unwrap_or_default());

        // The container's single Tab stop, on the PREVIOUS frame's rect so
        // a click focuses (and places the caret) before this frame's
        // layout paints — egui never grants focus on a pointer click, so
        // the clicked block requests it explicitly.
        let mut focused = ui.ctx().memory(|m| m.has_focus(control));
        if let Some(prev_rect) = ui.ctx().data_mut(|d| {
            d.get_temp::<Option<Rect>>(container_rect_id(group_id))
                .unwrap_or(None)
        }) {
            let sense = if self.disabled {
                Sense::hover()
            } else {
                Sense::click()
            };
            let response = ui.interact(prev_rect, control, sense);
            if response.clicked() {
                response.request_focus();
                focused = true;
                if was_focused {
                    // Already focused: the click places the caret at the
                    // clicked slot. Not yet focused: the focus-gain rule
                    // below places it (the library's `_focusListener`).
                    if let Some(pos) = response.interact_pointer_pos() {
                        if let Some(index) = nearest_slot(&prev_slot_rects, pos) {
                            sel = (index, index);
                        }
                    }
                }
            }
        }
        // Focus-gain: the library's `_focusListener`.
        if focused && !was_focused {
            sel = focus_gain_sel(prev_len, length);
        }

        // Keyboard handling BEFORE layout, so the echo paints same-frame.
        if focused && !self.disabled {
            let events: Vec<Event> = ui.input(|i| i.events.clone());
            if apply_otp_events(&events, &mut value, &mut sel, length, self.numeric) {
                ui.ctx()
                    .data_mut(|d| d.insert_temp(value_id(group_id), value.clone()));
            }
        }

        // Restart the group order counter and the slot-rect list for this
        // frame; the groups refill the list during layout.
        ui.ctx().data_mut(|d| {
            d.insert_temp(counter_id(group_id), 0usize);
            d.insert_temp(slot_rects_id(group_id), Vec::<(usize, Rect)>::new());
        });

        push_ctx(
            ui,
            OtpCtx {
                group_id,
                disabled: self.disabled,
                invalid: self.invalid,
                sel,
                focused,
            },
        );

        // `flex items-center gap-2` — the container row.
        let inner = ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::SM;
            content(ui);
        });

        pop_ctx(ui);

        // Read back the value and write it into the bound string.
        let after: String = ui
            .ctx()
            .data_mut(|d| d.get_temp(value_id(group_id)).unwrap_or_default());
        let text: String = after.chars().take(length).collect();
        let changed = text != *self.value;
        if changed {
            *self.value = text;
        }
        let new_len = self.value.chars().count();
        let completed = prev_len < length && new_len == length;
        let focused_now = ui.ctx().memory(|m| m.has_focus(control));

        // OTP-INT-01: plain arrow keys must move the caret, not focus.
        // egui's `Focus::begin_pass` treats an unclaimed plain arrow
        // as spatial focus navigation and clears focus entirely on an
        // unmodified Escape keydown
        // (`egui-0.37.0/src/memory/mod.rs:569-614`, gated on
        // `EventFilter::matches`), so without a filter the arrows
        // steal focus to the nearest widget (S6 caught the engine
        // moving the caret for exactly one presented frame before
        // `end_pass` moved focus away) and Escape blurs the control.
        // Claim all three non-Tab keys: the library has no behaviour
        // for any of them (module docs: "Up/Down, Enter and Escape
        // are no-ops"), so the closest match is to keep focus AND
        // the caret put — horizontal arrows move the caret (the
        // `apply_otp_events` arms), vertical arrows and Escape do
        // nothing. `tab` stays `false` so Tab still exits the
        // control's single Tab stop (S6 agreement; pinned by
        // `tab_still_exits_otp_int_01`). Enter needs no claim:
        // `matches` returns `true` for every other key, so it already
        // reaches the engine, which ignores it. The filter latches
        // only after 2 consecutive focused frames
        // (`set_focus_lock_filter` requires `had_focus_last_frame &&
        // has_focus`) and every focus grant resets it to default, so
        // it is re-installed on every focused frame; the one-pass
        // window after a grant is structural (see the
        // `arrow_in_first_pass_after_grant_moves_focus_otp_int_01`
        // documentation pin). Disabled controls never hold focus
        // (`Sense::hover`, no `request_focus`), so the gate is
        // belt-and-braces. RAD-INT-07 / SLD-INT-04 / SLD-INT-07 /
        // TOG-C2 precedent.
        if focused_now && !self.disabled {
            ui.ctx().memory_mut(|m| {
                m.set_focus_lock_filter(
                    control,
                    egui::EventFilter {
                        tab: false,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: true,
                    },
                );
            });
        }

        ui.ctx().data_mut(|d| {
            d.insert_temp(was_focused_id(group_id), focused_now);
            d.insert_temp(container_rect_id(group_id), Some(inner.response.rect));
            d.insert_temp(sel_id(group_id), sel);
        });

        OtpOutput {
            changed,
            focused: focused_now,
            completed,
        }
    }
}

/// A run of slots sharing borders (`InputOTPGroup`).
///
/// Its slots record their `(index, rect)` during layout; the group paints
/// all of their chrome post-closure (fill, shadow, fused borders, chars,
/// caret, rings — see the module docs), so first/last corners resolve in
/// the same frame, with no previous-frame registry.
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
            .data_mut(|d| d.insert_temp::<Vec<(usize, Rect)>>(seen_id, Vec::new()));

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

        // Dedupe by index (keep the first) and sort — duplicate indices
        // (E8) paint once, and the first/last corners follow the min/max
        // index, not render order.
        let seen: Vec<(usize, Rect)> = ui
            .ctx()
            .data_mut(|d| d.get_temp(seen_id).unwrap_or_default());
        let mut slots: Vec<(usize, Rect)> = Vec::new();
        for (index, rect) in seen {
            if !slots.iter().any(|(i, _)| *i == index) {
                slots.push((index, rect));
            }
        }
        slots.sort_by_key(|(index, _)| *index);

        // Append to the instance slot-rect list (the click-to-place map).
        ui.ctx().data_mut(|d| {
            let mut all: Vec<(usize, Rect)> =
                d.get_temp(slot_rects_id(ctx.group_id)).unwrap_or_default();
            all.extend_from_slice(&slots);
            d.insert_temp(slot_rects_id(ctx.group_id), all);
        });

        if !slots.is_empty() {
            paint_group_slots(ui, &ctx, &slots);
        }

        // nova `.cn-input-otp-group`: `has-aria-invalid:ring-3
        // ring-destructive/20 dark:ring-destructive/40` around the whole
        // group when any slot is invalid — faded with the container when
        // disabled (`has-disabled:opacity-50`).
        if ctx.invalid {
            let theme = ShadcnTheme::of_ui(ui);
            if theme.style == Style::Nova && ui.is_rect_visible(inner.response.rect) {
                let alpha = if theme.mode.is_dark() { 0.4 } else { 0.2 };
                let alpha = if ctx.disabled { alpha * 0.5 } else { alpha };
                let ring_color = color_alpha(theme.palette.destructive, alpha);
                crate::utils::focus::paint_ring(
                    ui.painter(),
                    inner.response.rect.expand2(egui::vec2(1.0, 1.0)),
                    ring_color,
                    3.0,
                    theme.radius.lg_u8() as f32,
                );
            }
        }

        inner
    }
}

/// Paints one group's slots post-closure, in CSS order: non-active slots,
/// then the active slot(s) (`z-10`), then the active rings — so an active
/// ring is never covered by a neighbour's fill.
fn paint_group_slots(ui: &Ui, ctx: &OtpCtx, slots: &[(usize, Rect)]) {
    let theme = ShadcnTheme::of_ui(ui);
    let dark = theme.mode.is_dark();
    let p = theme.palette;
    let radius = if theme.style == Style::Nova {
        theme.radius.lg_u8()
    } else {
        theme.radius.md_u8()
    };
    let value: String = ui
        .ctx()
        .data_mut(|d| d.get_temp(value_id(ctx.group_id)).unwrap_or_default());
    let chars: Vec<char> = value.chars().collect();
    let first = slots.first().map(|(i, _)| *i).unwrap_or(0);
    let last = slots.last().map(|(i, _)| *i).unwrap_or(0);
    let fade = if ctx.disabled { 0.5 } else { 1.0 };
    // `animate-caret-blink` — 1s period, 50% square-wave duty (a
    // documented assumption; OPEN-6 leaves the duty unestablished).
    let caret_visible = ui.input(|i| (i.time * 2.0) % 2.0 < 1.0);
    let painter = ui.painter();

    // Phase A: non-active slots.
    for &(index, rect) in slots {
        if !ui.is_rect_visible(rect) {
            continue;
        }
        if is_slot_active(index, ctx.sel, ctx.focused, ctx.disabled) {
            continue;
        }
        paint_slot_chrome(ui, rect, index, first, last, false, ctx);
        if let Some(&ch) = chars.get(index) {
            paint_slot_char(
                painter,
                rect,
                index == first,
                ch,
                color_alpha(p.foreground, fade),
            );
        }
    }
    // Phase B: active slot(s) — shadow, fill, border, char or caret.
    for &(index, rect) in slots {
        if !ui.is_rect_visible(rect) {
            continue;
        }
        if !is_slot_active(index, ctx.sel, ctx.focused, ctx.disabled) {
            continue;
        }
        paint_slot_chrome(ui, rect, index, first, last, true, ctx);
        match chars.get(index) {
            Some(&ch) => {
                paint_slot_char(
                    painter,
                    rect,
                    index == first,
                    ch,
                    color_alpha(p.foreground, fade),
                );
            }
            None => {
                // `h-4 w-px animate-caret-blink bg-foreground` — the caret
                // renders only on an active EMPTY slot (`hasFakeCaret`).
                if caret_visible {
                    let caret = Rect::from_center_size(
                        content_center(rect, index == first),
                        Vec2::new(1.0, OTP_CARET_HEIGHT),
                    );
                    painter.rect_filled(caret, 0, color_alpha(p.foreground, fade));
                }
            }
        }
    }
    // Phase C: the active rings (`ring-[3px] ring-ring/50`, invalid
    // `ring-destructive/20` dark /40).
    for &(index, rect) in slots {
        if !ui.is_rect_visible(rect) {
            continue;
        }
        if !is_slot_active(index, ctx.sel, ctx.focused, ctx.disabled) {
            continue;
        }
        crate::utils::focus::paint_ring(
            painter,
            rect.expand2(egui::vec2(1.0, 1.0)),
            slot_ring_color(ctx.invalid, dark, p),
            3.0,
            radius as f32,
        );
    }
}

/// One slot's shadow, fill and fused border.
fn paint_slot_chrome(
    ui: &Ui,
    rect: Rect,
    index: usize,
    first: usize,
    last: usize,
    active: bool,
    ctx: &OtpCtx,
) {
    let theme = ShadcnTheme::of_ui(ui);
    let dark = theme.mode.is_dark();
    let p = theme.palette;
    let radius = if theme.style == Style::Nova {
        theme.radius.lg_u8()
    } else {
        theme.radius.md_u8()
    };
    let is_first = index == first;
    let corner = slot_corners(is_first, index == last, radius);
    let painter = ui.painter();
    // `shadow-xs` — painted even when disabled, at half alpha: the
    // `has-disabled:opacity-50` container fade dims it, it does not
    // remove it.
    let sh = crate::utils::shadow::shadow_xs(dark);
    let shadow_color = if ctx.disabled {
        color_alpha(sh.color, 0.5)
    } else {
        sh.color
    };
    painter.add(
        egui::epaint::RectShape::filled(
            rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
            corner,
            shadow_color,
        )
        .with_blur_width(2.0),
    );
    // `dark:bg-input/30`; `has-disabled:opacity-50` fades the fill.
    if dark {
        painter.rect_filled(
            rect,
            corner,
            color_alpha(p.input, slot_fill_alpha(ctx.disabled)),
        );
    }
    // `border-y border-r border-input` (first also gets border-l);
    // invalid beats active; `has-disabled:opacity-50` fades the border.
    let stroke_color = color_alpha(
        slot_border_color(ctx.invalid, active, p),
        if ctx.disabled { 0.5 } else { 1.0 },
    );
    paint_otp_fused_stroke(
        painter,
        rect,
        corner,
        Stroke::new(1.0, stroke_color),
        slot_edges(is_first),
    );
}

/// One slot's character, centred in the content box.
fn paint_slot_char(painter: &Painter, rect: Rect, is_first: bool, ch: char, color: Color32) {
    let font = egui::FontId::proportional(crate::theme::tokens::type_scale::SM);
    painter.text(
        content_center(rect, is_first),
        Align2::CENTER_CENTER,
        ch.to_string(),
        font,
        color,
    );
}

/// One OTP character slot (`InputOTPSlot`).
///
/// A slot only allocates space, records its `(index, rect)` for the
/// enclosing group, and answers hover — it paints nothing itself. The
/// group paints all chrome post-closure (see the module docs), and the
/// container owns the single Tab stop, so slots are never focusable.
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
            let theme = ShadcnTheme::of_ui(ui);
            let (rect, response) =
                ui.allocate_exact_size(Vec2::splat(slot_size(theme.style)), Sense::hover());
            if ui.is_rect_visible(rect) {
                ui.painter().rect_stroke(
                    rect,
                    theme.radius.md_u8(),
                    Stroke::new(1.0, theme.palette.input),
                    egui::epaint::StrokeKind::Inside,
                );
            }
            return response;
        };
        let theme = ShadcnTheme::of_ui(ui);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ctx.group_id.with("slot_salt").with(salt),
            None => ctx.group_id.with("slot").with(self.index),
        };

        let (_, rect) = ui.allocate_space(Vec2::splat(slot_size(theme.style)));

        // Report this index to the enclosing group (for first/last corners
        // and the click-to-place map).
        if let Some(seen_id) = ui.ctx().data_mut(|d| {
            let stack: Vec<Id> = d.get_temp(group_stack_id()).unwrap_or_default();
            stack.last().copied()
        }) {
            ui.ctx().data_mut(|d| {
                let mut seen: Vec<(usize, Rect)> = d.get_temp(seen_id).unwrap_or_default();
                seen.push((self.index, rect));
                d.insert_temp(seen_id, seen);
            });
        }

        // Hover-only: the control has ONE Tab stop on the container.
        ui.interact(rect, id, Sense::hover())
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
            // The source separator has no color class, so the icon inherits
            // the foreground color — faded with the container when disabled
            // (`has-disabled:opacity-50`).
            let theme = ShadcnTheme::of_ui(ui);
            let alpha = match current_ctx(ui) {
                Some(ctx) if ctx.disabled => 0.5,
                _ => 1.0,
            };
            Icon::Minus.paint(
                ui.painter(),
                rect,
                color_alpha(theme.palette.foreground, alpha),
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

    /// Renders one frame of the OTP control and returns the frame output
    /// plus the control's output.
    fn run_frame(
        ctx: &egui::Context,
        value: &mut String,
        input: egui::RawInput,
        name: &'static str,
        salt: &'static str,
        length: usize,
        numeric: bool,
    ) -> (egui::FullOutput, OtpOutput) {
        let mut output = OtpOutput {
            changed: false,
            focused: false,
            completed: false,
        };
        let out = ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique(name),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            output = InputOTP::new(value)
                .length(length)
                .numeric(numeric)
                .id_salt(salt)
                .show(ui);
        });
        (out, output)
    }

    /// The control id for a test whose `Ui` id is `Id::unique(name)` and
    /// whose salt is `salt`.
    fn test_control_id(name: &'static str, salt: &'static str) -> Id {
        Id::unique(name).with(salt).with("control")
    }

    /// The selection stored in temp data for a test control.
    fn test_sel(ctx: &egui::Context, name: &'static str, salt: &'static str) -> (usize, usize) {
        let group_id = Id::unique(name).with(salt);
        ctx.data_mut(|d| d.get_temp(sel_id(group_id)).unwrap_or((0, 0)))
    }

    /// The slot rects recorded this frame for a test control, sorted by
    /// index — the ground truth for geometry assertions (the default
    /// preset is nova, 32px slots, and the harness lays the row out at
    /// y=2, so tests must not hardcode a size or origin).
    fn test_slot_rects(
        ctx: &egui::Context,
        name: &'static str,
        salt: &'static str,
    ) -> Vec<(usize, Rect)> {
        let group_id = Id::unique(name).with(salt);
        let mut rects: Vec<(usize, Rect)> =
            ctx.data_mut(|d| d.get_temp(slot_rects_id(group_id)).unwrap_or_default());
        rects.sort_by_key(|(i, _)| *i);
        rects
    }

    /// The 1px border strokes in a frame's shapes: `(clip_rect, rect, stroke)`.
    /// A point is covered by a stroke only if it lies inside BOTH the clip
    /// rect AND the shape's 1px inner border band — the clip alone is not
    /// enough (a plain `rect_stroke` carries the whole ui clip rect).
    fn border_strokes(out: &egui::FullOutput) -> Vec<(Rect, Rect, Stroke)> {
        let mut strokes = Vec::new();
        for clipped in &out.shapes {
            if let egui::Shape::Rect(rect_shape) = &clipped.shape {
                if rect_shape.stroke.width == 1.0 && !rect_shape.stroke.is_empty() {
                    strokes.push((clipped.clip_rect, rect_shape.rect, rect_shape.stroke));
                }
            }
        }
        strokes
    }

    /// Whether a 1px `Inside` border stroke covers `point`.
    fn border_covers(strokes: &[(Rect, Rect, Stroke)], point: Pos2) -> bool {
        strokes.iter().any(|(clip, rect, _)| {
            if !clip.contains(point) {
                return false;
            }
            // The 1px inner border band of the shape's rect.
            (point.x >= rect.left() && point.x < rect.left() + 1.0)
                || (point.x >= rect.right() - 1.0 && point.x < rect.right())
                || (point.y >= rect.top() && point.y < rect.top() + 1.0)
                || (point.y >= rect.bottom() - 1.0 && point.y < rect.bottom())
        })
    }

    #[test]
    fn slot_geometry_matches_source() {
        assert_eq!(OTP_SLOT_SIZE, 36.0); // h-9 w-9 (base recipe)
        assert_eq!(slot_size(Style::Vega), 36.0);
        assert_eq!(slot_size(Style::Nova), 32.0); // nova size-8
        assert_eq!(OTP_CARET_HEIGHT, 16.0); // h-4
    }

    #[test]
    fn show_truncates_long_values() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = "1234567890".to_owned(); // longer than length
        let (mut out, o) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_len_test",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        assert!(o.changed);
        assert_eq!(value, "1234");
    }

    #[test]
    fn container_reports_focus_and_change() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let (mut out, o) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_focus_test",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        assert!(!o.focused);
        assert!(!o.changed);
        assert!(!o.completed);
        assert!(value.is_empty());
    }

    #[test]
    fn slot_edges_assign_borders_once() {
        // N7: first paints all four edges, every other slot omits the left
        // edge (its divider is the earlier slot's right edge).
        assert_eq!(
            slot_edges(true),
            SlotEdges {
                left: true,
                top: true,
                right: true,
                bottom: true,
            }
        );
        assert_eq!(
            slot_edges(false),
            SlotEdges {
                left: false,
                top: true,
                right: true,
                bottom: true,
            }
        );
    }

    #[test]
    fn slot_corners_follow_first_last() {
        assert_eq!(
            slot_corners(true, true, 6),
            CornerRadius {
                nw: 6,
                ne: 6,
                sw: 6,
                se: 6,
            }
        ); // only slot: all four corners
        assert_eq!(
            slot_corners(true, false, 6),
            CornerRadius {
                nw: 6,
                ne: 0,
                sw: 6,
                se: 0,
            }
        ); // first
        assert_eq!(
            slot_corners(false, true, 6),
            CornerRadius {
                nw: 0,
                ne: 6,
                sw: 0,
                se: 6,
            }
        ); // last
        assert_eq!(slot_corners(false, false, 6), CornerRadius::ZERO); // middle
    }

    #[test]
    fn invalid_beats_active_border() {
        let p = Preset::default().build(Mode::Light).palette;
        assert_eq!(slot_border_color(true, true, p), p.destructive);
        assert_eq!(slot_border_color(true, false, p), p.destructive);
        assert_eq!(slot_border_color(false, true, p), p.ring);
        assert_eq!(slot_border_color(false, false, p), p.input);
        assert_eq!(
            slot_ring_color(true, false, p),
            color_alpha(p.destructive, 0.2)
        );
        assert_eq!(
            slot_ring_color(true, true, p),
            color_alpha(p.destructive, 0.4)
        );
        assert_eq!(slot_ring_color(false, false, p), color_alpha(p.ring, 0.5));
    }

    #[test]
    fn focus_gain_sel_places_caret_at_first_empty() {
        assert_eq!(focus_gain_sel(0, 6), (0, 0));
        assert_eq!(focus_gain_sel(2, 4), (2, 2));
        assert_eq!(focus_gain_sel(4, 4), (3, 4)); // full: 1-char range on the last slot
        assert_eq!(focus_gain_sel(0, 1), (0, 0));
    }

    #[test]
    fn content_center_uses_content_box() {
        let rect = Rect::from_min_size(Pos2::new(36.0, 0.0), Vec2::splat(36.0));
        // Middle/last slot: content x in [36, 71], centre 53.5.
        assert_eq!(content_center(rect, false), Pos2::new(53.5, 18.0));
        // First slot: content x in [37, 71], centre 54.0 — 0.5px right.
        let first = Rect::from_min_size(Pos2::ZERO, Vec2::splat(36.0));
        assert_eq!(content_center(first, true), Pos2::new(18.0, 18.0));
    }

    #[test]
    fn typing_inserts_and_advances() {
        // Mid-value typing inserts (native input), it does not replace.
        let mut value = "13".to_owned();
        let mut sel = (1, 1);
        assert!(apply_otp_events(
            &[Event::Text("2".to_owned())],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "123");
        assert_eq!(sel, (2, 2));
        // Typing at the end inserts and advances past the end when full.
        let mut value = "123".to_owned();
        let mut sel = (3, 3);
        assert!(apply_otp_events(
            &[Event::Text("4".to_owned())],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "1234");
        assert_eq!(sel, (4, 4)); // past the end: no active slot
                                 // Typing on the focus-gain range (full) replaces the last char.
        let mut value = "1234".to_owned();
        let mut sel = (3, 4);
        assert!(apply_otp_events(
            &[Event::Text("9".to_owned())],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "1239");
        assert_eq!(sel, (4, 4));
        // Typing past the end when full is a no-op.
        let mut value = "1234".to_owned();
        let mut sel = (4, 4);
        assert!(!apply_otp_events(
            &[Event::Text("9".to_owned())],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "1234");
    }

    #[test]
    fn paste_is_atomic_and_lands_after_last_inserted() {
        // Plain paste inserts at the caret and lands after it.
        let mut value = String::new();
        let mut sel = (0, 0);
        assert!(apply_otp_events(
            &[Event::Paste("12".to_owned())],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "12");
        assert_eq!(sel, (2, 2));
        // One bad char rejects the ENTIRE paste (whole-value pattern test).
        let mut value = "1".to_owned();
        let mut sel = (1, 1);
        assert!(!apply_otp_events(
            &[Event::Paste("2a".to_owned())],
            &mut value,
            &mut sel,
            4,
            true
        ));
        assert_eq!(value, "1");
        assert_eq!(sel, (1, 1));
        // Overflow truncates to maxLength; the caret lands after the last
        // accepted char.
        let mut value = "12".to_owned();
        let mut sel = (2, 2);
        assert!(apply_otp_events(
            &[Event::Paste("3456".to_owned())],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "1234");
        assert_eq!(sel, (4, 4));
        // Paste on a range replaces the range.
        let mut value = "1234".to_owned();
        let mut sel = (3, 4);
        assert!(apply_otp_events(
            &[Event::Paste("9".to_owned())],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "1239");
        assert_eq!(sel, (4, 4));
        // Empty paste is a no-op.
        let mut value = "12".to_owned();
        let mut sel = (1, 1);
        assert!(!apply_otp_events(
            &[Event::Paste(String::new())],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "12");
        assert_eq!(sel, (1, 1));
    }

    #[test]
    fn delete_clears_at_caret_without_moving() {
        // Forward delete at the caret, shifting left, caret stays.
        let mut value = "123".to_owned();
        let mut sel = (1, 1);
        assert!(apply_otp_events(
            &[Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "13");
        assert_eq!(sel, (1, 1));
        // Delete at the end is a no-op.
        let mut value = "123".to_owned();
        let mut sel = (3, 3);
        assert!(!apply_otp_events(
            &[Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "123");
        // Delete on a range deletes the range.
        let mut value = "1234".to_owned();
        let mut sel = (1, 3);
        assert!(apply_otp_events(
            &[Event::Key {
                key: Key::Delete,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "14");
        assert_eq!(sel, (1, 1));
    }

    #[test]
    fn backspace_shifts_left_without_holes() {
        // Backspace on a mid-value caret deletes the previous char and
        // shifts left — no hole is left behind.
        let mut value = "123".to_owned();
        let mut sel = (2, 2);
        assert!(apply_otp_events(
            &[Event::Key {
                key: Key::Backspace,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "13");
        assert_eq!(sel, (1, 1));
        // Backspace on a range deletes the range.
        let mut value = "1234".to_owned();
        let mut sel = (1, 3);
        assert!(apply_otp_events(
            &[Event::Key {
                key: Key::Backspace,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "14");
        assert_eq!(sel, (1, 1));
        // Backspace at the start is a no-op.
        let mut value = "123".to_owned();
        let mut sel = (0, 0);
        assert!(!apply_otp_events(
            &[Event::Key {
                key: Key::Backspace,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            }],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(value, "123");
    }

    #[test]
    fn arrows_collapse_ranges_and_stop_at_ends() {
        let key = |k: Key| Event::Key {
            key: k,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        };
        // Left on a range collapses to the range start.
        let mut value = "1234".to_owned();
        let mut sel = (1, 3);
        assert!(!apply_otp_events(
            &[key(Key::ArrowLeft)],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(sel, (1, 1));
        // Right on a range collapses to the range end.
        let mut sel = (1, 3);
        assert!(!apply_otp_events(
            &[key(Key::ArrowRight)],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(sel, (3, 3));
        // Arrows stop at the ends (no wrap).
        let mut sel = (0, 0);
        assert!(!apply_otp_events(
            &[key(Key::ArrowLeft)],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(sel, (0, 0));
        let mut sel = (4, 4);
        assert!(!apply_otp_events(
            &[key(Key::ArrowRight)],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(sel, (4, 4));
        // Repeats are honoured (native hold-to-repeat).
        let mut sel = (3, 3);
        assert!(!apply_otp_events(
            &[Event::Key {
                key: Key::ArrowLeft,
                physical_key: None,
                pressed: true,
                repeat: true,
                modifiers: egui::Modifiers::default(),
            }],
            &mut value,
            &mut sel,
            4,
            false
        ));
        assert_eq!(sel, (2, 2));
    }

    #[test]
    fn text_event_fills_slot_and_advances_caret() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let control = test_control_id("otp_type_test", "otp");

        // Run one frame, then focus the control (its single Tab stop).
        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_type_test",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        ctx.memory_mut(|mem| mem.request_focus(control));

        // Type "42": fills slots 0 and 1, caret advances to slot 2.
        for ch in ["4", "2"] {
            let mut input = egui::RawInput::default();
            input.events.push(Event::Text((*ch).to_owned()));
            let (mut out, _) = run_frame(&ctx, &mut value, input, "otp_type_test", "otp", 4, false);
            out.textures_delta.clear();
        }
        assert_eq!(value, "42");
        assert_eq!(test_sel(&ctx, "otp_type_test", "otp"), (2, 2));
        assert!(ctx.memory(|m| m.has_focus(control)));
    }

    #[test]
    fn numeric_filter_ignores_letters() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let control = test_control_id("otp_numeric_test", "otp");

        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_numeric_test",
            "otp",
            3,
            true,
        );
        out.textures_delta.clear();
        ctx.memory_mut(|mem| mem.request_focus(control));

        let mut input = egui::RawInput::default();
        input.events.push(Event::Text("a7".to_owned()));
        let (mut out, _) = run_frame(&ctx, &mut value, input, "otp_numeric_test", "otp", 3, true);
        out.textures_delta.clear();
        assert_eq!(value, "7");
        assert_eq!(test_sel(&ctx, "otp_numeric_test", "otp"), (1, 1));
    }

    #[test]
    fn backspace_clears_and_moves_back() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = "12".to_owned();
        let control = test_control_id("otp_bs_test", "otp");

        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_bs_test",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        // Focus-gain puts the caret at the first empty slot (2).
        ctx.memory_mut(|mem| mem.request_focus(control));
        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_bs_test",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        assert_eq!(test_sel(&ctx, "otp_bs_test", "otp"), (2, 2));

        let mut input = egui::RawInput::default();
        input.events.push(Event::Key {
            key: Key::Backspace,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        });
        let (mut out, _) = run_frame(&ctx, &mut value, input, "otp_bs_test", "otp", 4, false);
        out.textures_delta.clear();
        // Backspace deleted the char before the caret ("2") and moved back.
        assert_eq!(value, "1");
        assert_eq!(test_sel(&ctx, "otp_bs_test", "otp"), (1, 1));
    }

    #[test]
    fn completing_value_reports_completed() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = "123".to_owned();
        let control = test_control_id("otp_complete_test", "otp");

        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_complete_test",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        ctx.memory_mut(|mem| mem.request_focus(control));

        let mut completed = false;
        let mut input = egui::RawInput::default();
        input.events.push(Event::Text("4".to_owned()));
        let mut out = ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_complete_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            let o = InputOTP::new(&mut value).length(4).id_salt("otp").show(ui);
            completed = o.completed;
            assert!(o.changed);
        });
        out.textures_delta.clear();
        assert_eq!(value, "1234");
        assert!(completed);
        // Typing the last char leaves the caret past the end: no active slot.
        assert_eq!(test_sel(&ctx, "otp_complete_test", "otp"), (4, 4));
    }

    #[test]
    fn n7_divider_painted_once() {
        // RED on the old code: the old full-perimeter `rect_stroke` painted
        // slot 1's left edge, so the omitted point was covered by a border
        // stroke. The fused stroke omits that edge — only slot 0's right
        // edge paints the divider. The probe points are derived from the
        // actual slot rects, so the test does not hardcode a size or origin.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_n7_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputOTP::new(&mut value).length(2).id_salt("otp").show(ui);
        });
        let strokes = border_strokes(&out);
        out.textures_delta.clear();
        let rects = test_slot_rects(&ctx, "otp_n7_test", "otp");
        assert_eq!(rects.len(), 2);
        let (slot0, slot1) = (rects[0].1, rects[1].1);
        // The divider: slot 0's right edge (its last pixel column).
        let divider = Pos2::new(slot0.right() - 0.5, slot0.center().y);
        // Slot 1's omitted left edge (its first pixel column) — covered
        // only at the two corner pixels, never mid-edge.
        let omitted = Pos2::new(slot1.left() + 0.5, slot1.center().y);
        assert!(
            border_covers(&strokes, divider),
            "the divider (slot 0's right edge) must be painted"
        );
        assert!(
            !border_covers(&strokes, omitted),
            "slot 1's left edge must not be painted mid-edge (N7)"
        );
        // The corner cells DO repaint the top/bottom edge's first pixel.
        let top_corner = Pos2::new(slot1.left() + 0.5, slot1.top() + 0.5);
        let bottom_corner = Pos2::new(slot1.left() + 0.5, slot1.bottom() - 0.5);
        assert!(
            border_covers(&strokes, top_corner),
            "the top edge's first pixel must still be painted"
        );
        assert!(
            border_covers(&strokes, bottom_corner),
            "the bottom edge's first pixel must still be painted"
        );
    }

    #[test]
    fn caret_geometry_is_1x16_at_content_center() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let control = test_control_id("otp_caret_test", "otp");

        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_caret_test",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        ctx.memory_mut(|mem| mem.request_focus(control));

        // Time 0.0: the 1s-period blink is in its visible half.
        let mut input = egui::RawInput::default();
        input.time = Some(0.0);
        let (mut out, _) = run_frame(&ctx, &mut value, input, "otp_caret_test", "otp", 4, false);
        // The caret is the only 1x16 filled rect: centred in the first
        // slot's content box (derived from the actual slot rect, so the
        // test does not hardcode a size or origin).
        let caret = out.shapes.iter().find_map(|clipped| {
            if let egui::Shape::Rect(rect_shape) = &clipped.shape {
                if rect_shape.fill != egui::Color32::TRANSPARENT
                    && rect_shape.stroke.is_empty()
                    && rect_shape.rect.size() == Vec2::new(1.0, OTP_CARET_HEIGHT)
                {
                    return Some(rect_shape.rect);
                }
            }
            None
        });
        out.textures_delta.clear();
        let rects = test_slot_rects(&ctx, "otp_caret_test", "otp");
        assert_eq!(
            caret.map(|r| r.center()),
            Some(content_center(rects[0].1, true)),
            "the caret must be 1x16 at the first slot's content-box centre"
        );
        // Type "1": the caret advances to slot 1 (a middle slot), and the
        // filled slot 0 shows its char, never a caret.
        let mut input = egui::RawInput::default();
        input.time = Some(0.0);
        input.events.push(Event::Text("1".to_owned()));
        let (mut out, _) = run_frame(&ctx, &mut value, input, "otp_caret_test", "otp", 4, false);
        out.textures_delta.clear();
        assert_eq!(value, "1");
        let mut input = egui::RawInput::default();
        input.time = Some(0.0);
        let (mut out, _) = run_frame(&ctx, &mut value, input, "otp_caret_test", "otp", 4, false);
        let carets: Vec<Rect> = out
            .shapes
            .iter()
            .filter_map(|clipped| {
                if let egui::Shape::Rect(rect_shape) = &clipped.shape {
                    if rect_shape.fill != egui::Color32::TRANSPARENT
                        && rect_shape.stroke.is_empty()
                        && rect_shape.rect.size() == Vec2::new(1.0, OTP_CARET_HEIGHT)
                    {
                        return Some(rect_shape.rect);
                    }
                }
                None
            })
            .collect();
        out.textures_delta.clear();
        let rects = test_slot_rects(&ctx, "otp_caret_test", "otp");
        assert_eq!(
            carets.len(),
            1,
            "exactly one caret: in slot 1 (the first empty slot)"
        );
        assert_eq!(carets[0].center(), content_center(rects[1].1, false));
    }

    #[test]
    fn disabled_fades_borders_and_shadow() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let palette = Preset::default().build(Mode::Light).palette;
        let mut value = String::new();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_disabled_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            InputOTP::new(&mut value)
                .length(2)
                .disabled(true)
                .id_salt("otp")
                .show(ui);
        });
        // Every 1px border stroke is the input colour at 50% alpha.
        let expected = color_alpha(palette.input, 0.5);
        let strokes = border_strokes(&out);
        out.textures_delta.clear();
        assert!(!strokes.is_empty());
        for (_, _, stroke) in &strokes {
            assert_eq!(stroke.color, expected, "disabled borders must fade to 50%");
        }
        // The shadow is painted (not skipped) at half alpha.
        let shadow = out.shapes.iter().find_map(|clipped| {
            if let egui::Shape::Rect(rect_shape) = &clipped.shape {
                if rect_shape.blur_width > 0.0 {
                    return Some(rect_shape.fill);
                }
            }
            None
        });
        assert_eq!(
            shadow,
            Some(color_alpha(
                crate::utils::shadow::shadow_xs(false).color,
                0.5
            )),
            "the disabled shadow must paint at half alpha, not disappear"
        );
    }

    #[test]
    fn single_tab_stop() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let control = test_control_id("otp_tab_test", "otp");
        let tab = || {
            let mut input = egui::RawInput::default();
            input.events.push(Event::Key {
                key: Key::Tab,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
            input
        };
        let render = |ctx: &egui::Context, value: &mut String| {
            let mut before = None;
            let mut after = None;
            let out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("otp_tab_test"),
                    egui::UiBuilder::new().max_rect(ui_rect()),
                );
                before = Some(ui.button("before").id);
                InputOTP::new(value).length(4).id_salt("otp").show(ui);
                after = Some(ui.button("after").id);
            });
            (out, before.unwrap(), after.unwrap())
        };
        // Frame 1: no focus yet. Tab 1 -> before, Tab 2 -> the control
        // (its ONE stop), Tab 3 -> after. If the slots were focusable,
        // Tab 2 would land on slot 0 and Tab 3 on slot 1.
        let (mut out, before, after) = render(&ctx, &mut value);
        out.textures_delta.clear();
        assert_eq!(ctx.memory(|m| m.focused()), None);
        let mut out = ctx.run_ui(tab(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_tab_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            ui.button("before");
            InputOTP::new(&mut value).length(4).id_salt("otp").show(ui);
            let _ = ui.button("after");
        });
        out.textures_delta.clear();
        assert_eq!(ctx.memory(|m| m.focused()), Some(before));
        let mut out = ctx.run_ui(tab(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_tab_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            ui.button("before");
            InputOTP::new(&mut value).length(4).id_salt("otp").show(ui);
            let _ = ui.button("after");
        });
        out.textures_delta.clear();
        assert_eq!(ctx.memory(|m| m.focused()), Some(control));
        let mut out = ctx.run_ui(tab(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_tab_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            ui.button("before");
            InputOTP::new(&mut value).length(4).id_salt("otp").show(ui);
            let _ = ui.button("after");
        });
        out.textures_delta.clear();
        assert_eq!(ctx.memory(|m| m.focused()), Some(after));
    }

    #[test]
    fn zero_slots_and_out_of_range_index_do_not_panic() {
        // E7: a group with no slots renders nothing and does not panic.
        // E8: a slot with an out-of-range (or duplicate) index renders as
        // an empty box and does not panic.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = String::new();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("otp_nopanic_test"),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            let o = InputOTP::new(&mut value)
                .length(4)
                .id_salt("otp")
                .show_with(ui, |ui| {
                    InputOTPGroup::new().show(ui, |_| {});
                    InputOTPGroup::new().show(ui, |ui| {
                        InputOTPSlot::new(99).show(ui);
                        InputOTPSlot::new(1).show(ui);
                        InputOTPSlot::new(1).show(ui);
                    });
                });
            assert!(!o.changed);
            assert!(!o.focused);
            assert!(!o.completed);
        });
        out.textures_delta.clear();
        assert!(value.is_empty());
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
        assert_eq!(slot_fill_alpha(false), 0.3);
        assert_eq!(slot_fill_alpha(true), 0.15); // disabled: halved by the container fade
        assert_ne!(light.input, dark.input);
    }

    /// Renders one frame with TWO OTP controls (for focus-navigation
    /// pins), returning the frame output and the scope id the controls
    /// were shown in. `vertical` stacks them top-to-bottom; otherwise
    /// they sit side by side. The scope id is returned (not assumed)
    /// because `ui.horizontal`/`ui.vertical` wrap the controls in a
    /// child Ui whose scope differs from the top Ui's id — focusing
    /// an id built from the top id alone targets nothing, and the
    /// dead-man's switch clears it on the next frame.
    fn run_two_frame(
        ctx: &egui::Context,
        value_a: &mut String,
        value_b: &mut String,
        input: egui::RawInput,
        name: &'static str,
        vertical: bool,
    ) -> (egui::FullOutput, Id) {
        let scope = std::cell::Cell::new(Id::NULL);
        let mut out = ctx.run_ui(input, |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique(name),
                egui::UiBuilder::new().max_rect(ui_rect()),
            );
            let mut render = |ui: &mut Ui| {
                scope.set(ui.scope_id());
                InputOTP::new(value_a).length(4).id_salt("otp-a").show(ui);
                InputOTP::new(value_b).length(4).id_salt("otp-b").show(ui);
            };
            if vertical {
                ui.vertical(|ui| render(ui));
            } else {
                ui.horizontal(|ui| render(ui));
            }
        });
        out.textures_delta.clear();
        (out, scope.get())
    }

    /// The control id for a two-control pin, from the scope id the
    /// controls were actually shown in.
    fn two_control_id(scope: Id, salt: &'static str) -> Id {
        scope.with(salt).with("control")
    }

    /// The selection stored in temp data for a two-control pin.
    fn two_test_sel(ctx: &egui::Context, scope: Id, salt: &'static str) -> (usize, usize) {
        let group_id = scope.with(salt);
        ctx.data_mut(|d| d.get_temp(sel_id(group_id)).unwrap_or((0, 0)))
    }

    /// A single key press/release event for the focus pins.
    fn key_event(key: Key, pressed: bool) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: egui::Modifiers::default(),
        }
    }

    #[test]
    fn arrow_right_moves_caret_and_retains_focus_otp_int_01() {
        // OTP-INT-01: ArrowRight on a focused control must move the
        // caret AND keep focus (the library moves the caret; egui's
        // default filter treats an unclaimed plain arrow as spatial
        // focus navigation). RED before the fix: focus moves to the
        // right control, so the focus assertion fails. The caret still
        // moves in that frame (the engine reads the event before
        // `end_pass` moves focus) — the defect is the focus theft,
        // not the engine.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value_a = "12".to_owned();
        let mut value_b = String::new();

        let (_, scope) = run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_arrows",
            false,
        );
        let control_a = two_control_id(scope, "otp-a");
        ctx.memory_mut(|mem| mem.request_focus(control_a));
        // Two settle frames: the focus-lock filter latches only after
        // two consecutive focused frames (`set_focus_lock_filter` gate).
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_arrows",
            false,
        );
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_arrows",
            false,
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(control_a));
        // Focus-gain put the caret at the first empty slot (2, 2).
        assert_eq!(two_test_sel(&ctx, scope, "otp-a"), (2, 2));

        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::ArrowRight, true));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            input,
            "otp_int01_arrows",
            false,
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(control_a),
            "ArrowRight must not move focus off the control"
        );
        assert_eq!(two_test_sel(&ctx, scope, "otp-a"), (3, 3));
        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::ArrowRight, false));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            input,
            "otp_int01_arrows",
            false,
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(control_a));
    }

    #[test]
    fn arrow_left_moves_caret_and_retains_focus_otp_int_01() {
        // OTP-INT-01, mirror of the Right pin: ArrowLeft on the right
        // control must move the caret left AND keep focus. RED before
        // the fix: focus moves to the left control.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value_a = String::new();
        let mut value_b = "12".to_owned();

        let (_, scope) = run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_arrows",
            false,
        );
        let control_b = two_control_id(scope, "otp-b");
        ctx.memory_mut(|mem| mem.request_focus(control_b));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_arrows",
            false,
        );
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_arrows",
            false,
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(control_b));
        assert_eq!(two_test_sel(&ctx, scope, "otp-b"), (2, 2));

        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::ArrowLeft, true));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            input,
            "otp_int01_arrows",
            false,
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(control_b),
            "ArrowLeft must not move focus off the control"
        );
        assert_eq!(two_test_sel(&ctx, scope, "otp-b"), (1, 1));
        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::ArrowLeft, false));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            input,
            "otp_int01_arrows",
            false,
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(control_b));
    }

    #[test]
    fn escape_retains_focus_otp_int_01() {
        // OTP-INT-01: Escape on a focused control must be a no-op
        // (module docs: "Up/Down, Enter and Escape are no-ops"; the
        // library has no Escape handler). Mechanism (egui 0.37
        // `Focus::begin_pass`): an unmodified Escape keydown clears
        // `focused_widget` ENTIRELY unless the focused widget's
        // `EventFilter.escape` is true. RED before the fix: the first
        // Escape keydown clears focus.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value = "12".to_owned();
        let control = test_control_id("otp_int01_escape", "otp");

        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_int01_escape",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        ctx.memory_mut(|mem| mem.request_focus(control));
        // Two settle frames: the focus-lock filter latches only after
        // two consecutive focused frames.
        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_int01_escape",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        let (mut out, _) = run_frame(
            &ctx,
            &mut value,
            egui::RawInput::default(),
            "otp_int01_escape",
            "otp",
            4,
            false,
        );
        out.textures_delta.clear();
        assert_eq!(ctx.memory(|m| m.focused()), Some(control));
        assert_eq!(test_sel(&ctx, "otp_int01_escape", "otp"), (2, 2));

        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::Escape, true));
        let (mut out, _) = run_frame(&ctx, &mut value, input, "otp_int01_escape", "otp", 4, false);
        out.textures_delta.clear();
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(control),
            "Escape keydown must not clear focus"
        );
        assert_eq!(value, "12", "Escape must not change the value");
        assert_eq!(test_sel(&ctx, "otp_int01_escape", "otp"), (2, 2));
        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::Escape, false));
        let (mut out, _) = run_frame(&ctx, &mut value, input, "otp_int01_escape", "otp", 4, false);
        out.textures_delta.clear();
        assert_eq!(ctx.memory(|m| m.focused()), Some(control));
    }

    #[test]
    fn vertical_arrows_retain_focus_otp_int_01() {
        // OTP-INT-01: Up/Down on a focused control must be no-ops
        // (module docs: "Up/Down, Enter and Escape are no-ops"). An
        // unclaimed plain Down is egui spatial navigation, so before
        // the fix focus moves to the control below. RED before the
        // fix: focus moves to the bottom control.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value_a = "12".to_owned();
        let mut value_b = String::new();

        let (_, scope) = run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_vertical",
            true,
        );
        let control_a = two_control_id(scope, "otp-a");
        ctx.memory_mut(|mem| mem.request_focus(control_a));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_vertical",
            true,
        );
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_vertical",
            true,
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(control_a));
        assert_eq!(two_test_sel(&ctx, scope, "otp-a"), (2, 2));

        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::ArrowDown, true));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            input,
            "otp_int01_vertical",
            true,
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(control_a),
            "ArrowDown must not move focus off the control"
        );
        assert_eq!(
            two_test_sel(&ctx, scope, "otp-a"),
            (2, 2),
            "ArrowDown must not move the caret"
        );
        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::ArrowDown, false));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            input,
            "otp_int01_vertical",
            true,
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(control_a));
    }

    #[test]
    fn tab_still_exits_otp_int_01() {
        // Guard (passes before AND after the fix): the filter must not
        // claim Tab — Tab still moves from one control's stop to the
        // next (single Tab stop per control, S6 agreement).
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value_a = "12".to_owned();
        let mut value_b = String::new();

        let (_, scope) = run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_tab",
            false,
        );
        let control_a = two_control_id(scope, "otp-a");
        let control_b = two_control_id(scope, "otp-b");
        ctx.memory_mut(|mem| mem.request_focus(control_a));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_tab",
            false,
        );
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_tab",
            false,
        );
        assert_eq!(ctx.memory(|m| m.focused()), Some(control_a));

        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::Tab, true));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            input,
            "otp_int01_tab",
            false,
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(control_b),
            "Tab must exit control A to control B"
        );
    }

    #[test]
    fn arrow_in_first_pass_after_grant_moves_focus_otp_int_01() {
        // OTP-INT-01 documentation pin (passes BEFORE and AFTER the
        // fix, by design): the one-pass latch window is structural to
        // egui's API. `request_focus` resets the filter to default and
        // `set_focus_lock_filter` installs only after two consecutive
        // focused frames, so an ArrowRight arriving in the very first
        // pass after a grant still moves focus (here: to the right
        // control). This pin asserts that limitation honestly — it
        // must never be "fixed" by weakening the filter gate. The
        // engine still moves the caret in that frame (the key reaches
        // the engine before `end_pass` moves focus), so the caret
        // lands at (3, 3) either way.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut value_a = "12".to_owned();
        let mut value_b = String::new();

        let (_, scope) = run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            egui::RawInput::default(),
            "otp_int01_latch",
            false,
        );
        let control_a = two_control_id(scope, "otp-a");
        let control_b = two_control_id(scope, "otp-b");
        ctx.memory_mut(|mem| mem.request_focus(control_a));
        // NO settle frames: the ArrowRight arrives in the first pass
        // after the grant, before the filter could latch.
        let mut input = egui::RawInput::default();
        input.events.push(key_event(Key::ArrowRight, true));
        run_two_frame(
            &ctx,
            &mut value_a,
            &mut value_b,
            input,
            "otp_int01_latch",
            false,
        );
        assert_eq!(
            ctx.memory(|m| m.focused()),
            Some(control_b),
            "ArrowRight in the first pass after a grant still moves focus (latch window)"
        );
        assert_eq!(two_test_sel(&ctx, scope, "otp-a"), (3, 3));
    }
}
