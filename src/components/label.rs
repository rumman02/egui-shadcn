//! shadcn `label.tsx` — a form label, optionally wired to a control.
//!
//! Rebuilt to the frozen Batch A S0 spec (`~/workspace/batches-a-h/specs/
//! label-spec.md`, Coordinator-signed 2026-11-15). The original is one
//! `Label` function component wrapping Radix `LabelPrimitive.Root`
//! (`label.tsx`, 23 lines), with a single class string and no variants:
//!
//! ```text
//! flex items-center gap-2 text-sm leading-none font-medium select-none
//! group-data-[disabled=true]:pointer-events-none
//! group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed
//! peer-disabled:opacity-50
//! ```
//!
//! (`label.tsx:15`.) The nova rules (`.cn-label`, `.cn-label-aria`,
//! `style-nova.css:789-795`) restate a subset of the base string and are
//! silent on the rest, so under the settled LAYER ruling nova renders
//! identically to base in every state the snapshot establishes (spec
//! §2.3). There is no colour, background, border, radius, padding,
//! shadow, icon-sizing, hover, focus, or animation rule in either
//! recipe: the label paints text (and children) only, at the inherited
//! text colour, mapped to the `foreground` token (spec §5.1 — a mapping,
//! not a class-string fact).
//!
//! Click behaviour: a label bound with [`Label::for_id`] (the `htmlFor`
//! concept, spec §8) forwards clicks to the named control: the click
//! fires on the control (activation for checkbox/radio/switch/button,
//! focus for text inputs) AND focuses it. That is the settled OPEN-1
//! rule (spec §7.2, the N6 question), closed by S6 Work B from public
//! Radix Label source (verbatim `.click()` + `.focus()` pre-2022,
//! native `<label>` platform forwarding since; see
//! `~/workspace/batches-a-h/captures/label-s6/_radix-source/`). The
//! forwarding is deferred one pass (see `record_pending_click`): egui
//! surrenders focus requested before the target is created in the
//! click pass, so the focus request and the synthetic activation are
//! issued on the next pass instead, when no click is in flight.
//!
//! The disabled flag ([`Label::disabled`]) is proposed deviation D1
//! (spec §10, Expert Reviewer approval pending): egui has no CSS
//! sibling/ancestor selectors, so the group/peer disabled conditions
//! are expressed as one explicit flag applying the group-disabled
//! outcome (50% opacity + non-interactive).
//!
//! Platform limits and notes (spec §10): D2 (`asChild`/Slot) and D3
//! (ARIA tree) are limitations — egui has no element-substitution
//! mechanism and no accessibility tree, so only the functional
//! consequence of `htmlFor` (click acts on the target) is implemented.
//! D4 (double-click text selection) is a note: egui label text has no
//! user text-selection behaviour to suppress. D5 (the peer-disabled
//! `not-allowed` cursor) is not separately expressible under D1's
//! single-flag simplification; see [`Label::disabled`].
//!
//! OPEN items (spec §11): OPEN-1 (activation + focus), OPEN-2 (Radix
//! prop forwarding: `Label` consumes no props of its own, everything
//! spreads onto the native `<label>`), and OPEN-6 (no Tab stop) are
//! settled dispositions, closed by S6 Work B from public Radix Label
//! source. OPEN-5 (composition API shape) is DECIDED — see
//! [`Label::show_with`]. OPEN-3 (`.cn-label-aria` attachment), OPEN-4
//! (font family), and OPEN-7 (`cn()` conflict resolution) stay OPEN:
//! the snapshot cannot supply the evidence, and no closure is claimed.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Label::new("Email").for_id(email_id).show(ui);
//! Label::new("Profile").icon(Icon::User).show(ui);
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, fonts, Palette, ShadcnTheme};
use egui::text::{LayoutJob, TextFormat};
use egui::{
    Align, Align2, Color32, FontId, Id, Layout, Pos2, Rect, Response, Sense, Ui, UiBuilder, Vec2,
    Widget,
};

/// `gap-2` (spec §3.2): 8px between every pair of flex children.
const GAP: f32 = crate::theme::tokens::spacing::SM;

/// Leading/trailing icon box. A builder API decision, NOT a spec
/// requirement: the label recipe contains no `[&_svg]` sizing rule
/// (spec §3.3), so icons keep their own size; the 16px slot is the size
/// this API renders its icon slots at.
const ICON_SIZE: f32 = crate::theme::tokens::spacing::LG;

/// `leading-none` text line box (spec §3.1): 1× the 14px `text-sm` size.
const LINE_H: f32 = crate::theme::tokens::type_scale::SM;

/// The label's font: `text-sm` (14px) at `font-medium` (500), spec §5.2.
/// The concrete family behind `--font-sans` is OPEN-4; the rendered face
/// is the Phase 1 embedded Inter Medium, not snapshot truth.
pub(crate) fn label_font() -> FontId {
    fonts::medium(crate::theme::tokens::type_scale::SM)
}

/// Label text color: the `foreground` slot (spec §5.1 mapping), 50% alpha
/// when disabled (`group-data-[disabled=true]:opacity-50` / D1). The same
/// color paints text and icon children uniformly (element-level opacity,
/// spec §4).
pub(crate) fn text_color(p: &Palette, disabled: bool) -> Color32 {
    if disabled {
        color_alpha(p.foreground, 0.5)
    } else {
        p.foreground
    }
}

/// Flex-row content size (spec §3.2): child widths sum plus `gap-2` (8px)
/// between every pair of PRESENT children; height is the tallest child.
/// The text child exists only when the text is non-empty, so an icon-only
/// label is exactly the icon box and an empty label is zero-size
/// (spec §9). `text_h` is the text child's height: one 14px line box
/// for a single line, one 14px line box per wrapped line for wrapped
/// text (spec §9). No padding, border, or chrome is added (spec §3.4) —
/// the widget allocates exactly this size.
pub(crate) fn content_size(
    text_w: f32,
    text_h: f32,
    has_text: bool,
    icon_start: bool,
    icon_end: bool,
) -> Vec2 {
    let mut width: f32 = 0.0;
    let mut height: f32 = 0.0;
    let mut children = 0usize;
    if icon_start {
        width += ICON_SIZE;
        height = height.max(ICON_SIZE);
        children += 1;
    }
    if has_text {
        width += text_w;
        height = height.max(text_h);
        children += 1;
    }
    if icon_end {
        width += ICON_SIZE;
        height = height.max(ICON_SIZE);
        children += 1;
    }
    if children > 1 {
        width += (children - 1) as f32 * GAP;
    }
    Vec2::new(width, height)
}

/// The label's interaction sense (spec §7). The label is never focusable
/// and never takes a Tab stop (§7.3, OPEN-6): the clickable case uses
/// [`Sense::CLICK`] — click without `FOCUSABLE` — never [`Sense::click`],
/// which would register a focusable widget. Clicks are sensed only when
/// the label names a target and is enabled; otherwise the label is
/// hover-only (`select-none`, no pointer response of its own).
pub(crate) fn label_sense(has_target: bool, disabled: bool) -> Sense {
    if has_target && !disabled {
        Sense::CLICK
    } else {
        Sense::hover()
    }
}

/// Temp-data key for a bound-label click that still needs forwarding to
/// its target (see [`record_pending_click`]).
fn pending_click_id(target: Id) -> Id {
    target.with("shadcn_label_pending_click")
}

/// A pending click older than this is dropped instead of forwarded. The
/// forwarding normally lands on the very next pass; the bound only stops
/// a stale click from firing if the label vanishes before the next pass
/// and reappears much later. Generous on purpose: passes under load can
/// lag seconds behind wall time.
const PENDING_CLICK_TTL: f64 = 30.0;

/// Records that a bound-label click must be forwarded to `target` on the
/// next pass, and requests the repaint that guarantees the next pass
/// happens.
///
/// Why deferred: in the click pass the label is usually shown BEFORE its
/// target, and egui 0.37's surrender-on-click (`Context::create_widget`:
/// a click that lands elsewhere surrenders the focused widget) undoes a
/// focus request in the same pass the target is created — the S1
/// focus-only forwarding never survived a pass (LBL-INT-01). On the next
/// pass no click is in flight, so the focus request sticks, and the
/// synthetic Space press (see [`consume_pending_click`]) reaches the
/// target's `interact` while it holds focus.
fn record_pending_click(ctx: &egui::Context, target: Id) {
    let now = ctx.input(|i| i.time);
    ctx.data_mut(|data| data.insert_temp(pending_click_id(target), now));
    ctx.request_repaint();
}

/// Forwards a click recorded by [`record_pending_click`] on a previous
/// pass: requests focus on `target`, then pushes a synthetic Space
/// key-down + key-up pair into the live input events.
///
/// The pair is what makes the target ACTIVATE (LBL-INT-02): egui's
/// `interact` reports a fake primary click for the focused widget
/// whenever it sees `key_pressed(Space)` or `key_pressed(Enter)`, and
/// `key_pressed` scans the live event list, so events pushed here are
/// visible to every widget shown later in this pass — including the
/// target, which the label precedes in the usual label-then-control
/// order. Both halves of the pair matter: the key-down is what
/// `interact` reads, and the key-up is what the crate's button needs —
/// its Space-fires-on-keyup handling hides a lone key-down and only
/// re-creates it when the release arrives. A bare Space key press
/// inserts no text (text insertion reads `Event::Text`, never a lone
/// `Key` event), so a text-input target focuses with no side effect;
/// a disabled target is protected twice over — egui's `ignore_focus`
/// clears focus for non-focusable widgets before `interact` reads it,
/// and the crate's controls gate `clicked()` on `!disabled`.
///
/// The pending flag is consumed (removed) whether or not `enabled`:
/// a label disabled after the click must not fire a stale activation
/// if it is re-enabled later. Callers place this where the target runs
/// AFTER it in the same pass: at the top of [`Label::show`], and
/// between the measure and real passes of [`Label::show_with`] (the
/// measure pass runs in a disabled child `Ui`, whose `interact` would
/// `ignore_focus` the target if the request came before it).
fn consume_pending_click(ui: &mut Ui, target: Id, enabled: bool) {
    let set_at = ui
        .ctx()
        .data_mut(|data| data.remove_temp::<f64>(pending_click_id(target)));
    let Some(set_at) = set_at else {
        return;
    };
    if !enabled {
        return;
    }
    let now = ui.input(|i| i.time);
    if now - set_at > PENDING_CLICK_TTL {
        return;
    }
    // Focus first: the fake click only applies to the focused widget.
    ui.memory_mut(|mem| mem.request_focus(target));
    ui.ctx().input_mut(|input| {
        for pressed in [true, false] {
            input.events.push(egui::Event::Key {
                key: egui::Key::Space,
                physical_key: Some(egui::Key::Space),
                pressed,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            });
        }
    });
}

/// A shadcn label. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Label {
    text: String,
    icon: Option<Icon>,
    icon_end: Option<Icon>,
    disabled: bool,
    for_id: Option<Id>,
    id_salt: Option<String>,
}

impl Label {
    /// Creates a label with the given text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon: None,
            icon_end: None,
            disabled: false,
            for_id: None,
            id_salt: None,
        }
    }

    /// Shows a 16px icon before the text (the flex row's first child,
    /// separated by `gap-2`). The 16px box is a builder API decision:
    /// the recipe defines no icon size (spec §3.3).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows a 16px icon after the text (the flex row's last child,
    /// separated by `gap-2`). Same builder-decision size as [`Label::icon`].
    /// Together the two slots cover the full §3.2 table (icon + text,
    /// text + icon, icon + text + icon).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    /// Binds the label to a control id: clicking the label forwards the
    /// click to that widget — activation for checkbox, radio, switch,
    /// and button targets, focus for text inputs — and focuses it, like
    /// Radix `htmlFor` (spec §8; the settled OPEN-1 rule, see the
    /// module docs). `htmlFor` has no default: a label without a
    /// target has no associated control.
    ///
    /// When the control is a crate widget shown after the label in the
    /// same `Ui` scope, its id is usually `ui.scope_id().with(id_salt)`
    /// for the same `id_salt` the control is constructed with. Some
    /// widgets compute their widget id inside a child `Ui` — `Input`
    /// shows its text edit in a `ui.scope`, so its widget id is NOT
    /// `ui.scope_id().with(id_salt)` — and a control nested inside
    /// [`Label::show_with`] lives one scope deeper than the label.
    /// For those, capture the id from the control's returned
    /// [`Response::id`] on one pass and bind it from the next; binding
    /// a computed id that matches no widget focuses nothing.
    pub fn for_id(mut self, id: Id) -> Self {
        self.for_id = Some(id);
        self
    }

    /// Disables the label: 50% opacity, non-interactive (proposed
    /// deviation D1, spec §10 — Expert Reviewer approval pending). The
    /// flag is the caller's expression of the group/peer disabled
    /// conditions, which egui cannot express declaratively; the visual
    /// rule (50% opacity, instant, no fade) is frozen either way. The
    /// peer-only cursor distinction is not separately expressible (D5).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt (Label keeps no state). Only affects
    /// [`Label::show`]; [`Label::show_with`] registers no interact and
    /// ignores the salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the label: leading icon, text, trailing icon as flex
    /// children (spec §§2–3). Nothing animates (spec §6); no repaint is
    /// requested.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let font = label_font();
        // Placeholder color: the paint-time fallback replaces it exactly,
        // so this one galley serves both sizing and painting.
        let no_wrap_galley =
            ui.painter()
                .layout_no_wrap(self.text.clone(), font.clone(), Color32::PLACEHOLDER);

        let has_text = !self.text.is_empty();
        // Long text wraps within the available width (spec §9: default
        // browser flex behaviour; each wrapped line is a 14px line box).
        // The width the text child can take is what remains after the
        // icon boxes and the gaps between present children. Short text
        // keeps the no-wrap galley, so single-line labels render
        // byte-identically to before.
        let mut reserved = 0.0f32;
        let mut children = 0usize;
        if self.icon.is_some() {
            reserved += ICON_SIZE;
            children += 1;
        }
        if has_text {
            children += 1;
        }
        if self.icon_end.is_some() {
            reserved += ICON_SIZE;
            children += 1;
        }
        if children > 1 {
            reserved += (children - 1) as f32 * GAP;
        }
        let text_avail = ui.available_width() - reserved;
        let (galley, text_h) =
            if has_text && text_avail > 0.0 && no_wrap_galley.size().x > text_avail {
                let mut job = LayoutJob::single_section(
                    self.text.clone(),
                    TextFormat {
                        font_id: font,
                        color: Color32::PLACEHOLDER,
                        line_height: Some(LINE_H),
                        ..Default::default()
                    },
                );
                job.wrap.max_width = text_avail;
                let galley = ui.painter().layout_job(job);
                let text_h = galley.size().y;
                (galley, text_h)
            } else {
                (no_wrap_galley, LINE_H)
            };

        let text_w = galley.size().x;
        let desired = content_size(
            text_w,
            text_h,
            has_text,
            self.icon.is_some(),
            self.icon_end.is_some(),
        );

        let sense = label_sense(self.for_id.is_some(), self.disabled);
        let interactive = self.for_id.is_some() && !self.disabled;

        // A click recorded on a previous pass is forwarded here, before
        // the target is created later in this pass (LBL-INT-01/02).
        if let Some(target) = self.for_id {
            consume_pending_click(ui, target, !self.disabled);
        }

        let (rect, response) = {
            // Reserve space without registering an auto-id focusable
            // response, so egui's spatial arrow-key navigation only sees the
            // explicit widget id (checkbox.rs pattern).
            let (_, rect) = ui.allocate_space(desired);
            let id = match self.id_salt.as_deref() {
                Some(salt) => ui.scope_id().with(salt),
                None => ui.auto_id_with("shadcn_label"),
            };
            (rect, ui.interact(rect, id, sense))
        };

        // Click forwarding (LBL-INT-01/02): the click is recorded for
        // the next pass, when no click is in flight to surrender the
        // focus request. The immediate request is kept too — it helps
        // when the label runs AFTER its target — but on its own it
        // never survives the click pass (see `record_pending_click`).
        if interactive && response.clicked() {
            if let Some(target) = self.for_id {
                ui.memory_mut(|mem| mem.request_focus(target));
                record_pending_click(ui.ctx(), target);
            }
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let color = text_color(&p, self.disabled);
            let center_y = rect.center().y;
            // Gaps exist only between present children (spec §3.2).
            let mut cursor = rect.left();
            let mut need_gap = false;
            if let Some(icon) = self.icon {
                if need_gap {
                    cursor += GAP;
                }
                let r = Rect::from_center_size(
                    Pos2::new(cursor + ICON_SIZE * 0.5, center_y),
                    Vec2::splat(ICON_SIZE),
                );
                icon.paint(painter, r, color, 2.0 * ICON_SIZE / 24.0);
                cursor += ICON_SIZE;
                need_gap = true;
            }
            if has_text {
                if need_gap {
                    cursor += GAP;
                }
                // Same paint math as `Painter::text`, reusing the galley.
                let text_rect = Align2::CENTER_CENTER
                    .anchor_size(Pos2::new(cursor + text_w * 0.5, center_y), galley.size());
                painter.galley(text_rect.min, galley, color);
                cursor += text_w;
                need_gap = true;
            }
            if let Some(icon) = self.icon_end {
                if need_gap {
                    cursor += GAP;
                }
                let r = Rect::from_center_size(
                    Pos2::new(cursor + ICON_SIZE * 0.5, center_y),
                    Vec2::splat(ICON_SIZE),
                );
                icon.paint(painter, r, color, 2.0 * ICON_SIZE / 24.0);
            }
        }

        response
    }

    /// Renders the label with arbitrary flex children: the label's own
    /// leading icon, text, and trailing icon (when present), plus the
    /// widgets `content` adds, laid out as one flex row with `gap-2`
    /// (8px) between children and vertical centering (`items-center`,
    /// spec §2). This is the Rust analogue of the original's arbitrary
    /// JSX children, including nested controls (spec §8, OPEN-5
    /// DECIDED: coordinator decision 2026-11-16 adopting the S1
    /// proposal — icon slots plus this generic composition method).
    ///
    /// The label's own text does not wrap in a `show_with` row
    /// (deliberate scope boundary, F5): the row is content-sized with
    /// no wrap-width concept, so the §9 wrapping requirement covers
    /// [`Label::show`] only.
    ///
    /// The closure runs TWICE (measure pass, then real pass — the
    /// `dialog_footer` pattern): the measure pass runs in an invisible,
    /// disabled child `Ui` so no callback can double-fire, and the real
    /// pass centers every child on the measured row height. Closure
    /// content must therefore be a pure function of its inputs, and must
    /// not depend on the available width for its height.
    ///
    /// Clicks anywhere in the row forward the click to the
    /// [`Label::for_id`] target — activation for checkbox, radio,
    /// switch, and button targets, focus for text inputs — and focus it
    /// (the settled OPEN-1 rule; see the module docs). The click is
    /// detected from pointer state rather than a click-sense interact,
    /// so no click-sense rect steals hits from nested widgets (the
    /// `FieldLabel::show_with_card` pattern). Nested interactive
    /// children keep their own click handling: a click that landed on
    /// the target itself is NOT forwarded (Radix's `isClickingControl`
    /// exclusion), so a direct click on a nested control fires exactly
    /// once. When disabled (D1), the whole row renders at 50% opacity
    /// via row opacity and no click is forwarded; nested children keep
    /// their own enabled state (the caller disables the nested control
    /// itself, as on the web).
    pub fn show_with<R>(
        self,
        ui: &mut Ui,
        mut content: impl FnMut(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let Self {
            text,
            icon,
            icon_end,
            disabled,
            for_id,
            id_salt: _,
        } = self;
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // Full-strength color: the disabled dimming comes from the row
        // opacity in the real pass, so closure children dim uniformly too.
        let color = p.foreground;
        let has_text = !text.is_empty();

        // One row rendering, shared by the measure and real passes: the
        // label's own children plus the closure's, in DOM order.
        let mut render = |ui: &mut Ui| -> R {
            ui.spacing_mut().item_spacing.x = GAP;
            if disabled {
                ui.set_opacity(0.5);
            }
            if let Some(icon) = icon {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(ICON_SIZE), Sense::hover());
                if ui.is_rect_visible(rect) {
                    icon.paint(ui.painter(), rect, color, 2.0 * ICON_SIZE / 24.0);
                }
            }
            if has_text {
                let galley =
                    ui.painter()
                        .layout_no_wrap(text.clone(), label_font(), Color32::PLACEHOLDER);
                let (rect, _) =
                    ui.allocate_exact_size(Vec2::new(galley.size().x, LINE_H), Sense::hover());
                if ui.is_rect_visible(rect) {
                    let text_rect = Align2::CENTER_CENTER.anchor_size(rect.center(), galley.size());
                    ui.painter().galley(text_rect.min, galley, color);
                }
            }
            let ret = content(ui);
            if let Some(icon) = icon_end {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(ICON_SIZE), Sense::hover());
                if ui.is_rect_visible(rect) {
                    icon.paint(ui.painter(), rect, color, 2.0 * ICON_SIZE / 24.0);
                }
            }
            ret
        };

        // Measure pass (dialog_footer pattern): cross-align Min keeps the
        // measured frame widget-sized; Center would span the full
        // (infinite) available height. The measure child gets its own
        // id salt, so nested widgets register DIFFERENT ids here than
        // in the real pass: the measure child is invisible and therefore
        // disabled, and egui's `create_widget` calls `ignore_focus` for
        // every widget in a disabled Ui — with a shared scope that
        // would clear the nested target's focus on every pass, and the
        // two passes would also trip the debug id-clash check.
        let flow_min = ui.cursor().min;
        let flow_w = ui.available_width();
        let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));
        let mut measure = ui.new_child(
            UiBuilder::new()
                .max_rect(measure_rect)
                .invisible()
                .id_salt("shadcn_label_measure"),
        );
        measure.with_layout(Layout::left_to_right(Align::Min), |ui| render(ui));
        let measured = measure.min_rect().size();

        // A click recorded on a previous pass is forwarded here — AFTER
        // the measure pass (whose disabled child Ui would `ignore_focus`
        // the target) and BEFORE the real pass (so the nested target
        // sees the focused + Space state during its own `interact`).
        if let Some(target) = for_id {
            consume_pending_click(ui, target, !disabled);
        }

        // Real pass: the row rect is the measured size, so Align::Center
        // centers every child on the row's center line (items-center).
        let row =
            ui.allocate_ui_with_layout(measured, Layout::left_to_right(Align::Center), |ui| {
                render(ui)
            });
        let row_rect = row.response.rect;

        // `<label>` semantics, read from pointer state so no click-sense
        // interact steals hits from nested widgets.
        if let Some(target) = for_id {
            if !disabled {
                // The click is recorded for the next pass, when no click
                // is in flight to surrender the focus request (see
                // `record_pending_click`). A click that landed on the
                // target itself is excluded — the target already fired
                // from the real click, and forwarding would fire it
                // twice (Radix's `isClickingControl` exclusion).
                //
                // The click position is the RELEASE position
                // (`latest_pos`): `press_origin` is cleared during
                // `begin_pass` when the release event is processed, so
                // it is always `None` on the pass where `any_click()`
                // is true and a press-origin check can never fire.
                // egui only reports a click when the pointer stayed
                // within `max_click_dist` of the press, so a release
                // inside the row means the press began inside (or
                // within a few px of) the row.
                let clicked_on_target =
                    ui.ctx().interaction_snapshot(|s| s.clicked == Some(target));
                let clicked = !clicked_on_target
                    && ui.input(|i| {
                        i.pointer.any_click()
                            && i.pointer.latest_pos().is_some_and(|p| row_rect.contains(p))
                    });
                if clicked {
                    ui.memory_mut(|mem| mem.request_focus(target));
                    record_pending_click(ui.ctx(), target);
                }
            }
        }

        row
    }
}

impl Widget for Label {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{fonts, Mode, Preset};

    #[test]
    fn label_font_is_medium_14() {
        let font = label_font();
        assert_eq!(font.size, 14.0);
        assert_eq!(font.family, fonts::medium_family());
        assert_eq!(font, fonts::medium(14.0));
    }

    #[test]
    fn content_size_matches_gap_arithmetic() {
        // Spec §3.2 table, text advance W = 50, icon box I = 16, gap = 8.
        let w = 50.0;
        assert_eq!(
            content_size(w, LINE_H, true, false, false),
            Vec2::new(50.0, 14.0)
        );
        assert_eq!(
            content_size(w, LINE_H, true, true, false),
            Vec2::new(74.0, 16.0)
        );
        assert_eq!(
            content_size(w, LINE_H, true, false, true),
            Vec2::new(74.0, 16.0)
        );
        assert_eq!(
            content_size(w, LINE_H, true, true, true),
            Vec2::new(98.0, 16.0)
        );
    }

    #[test]
    fn content_size_icon_only_and_empty() {
        // Icon-only: exactly the icon box, no text line box (spec §9).
        assert_eq!(
            content_size(0.0, LINE_H, false, true, false),
            Vec2::new(16.0, 16.0)
        );
        assert_eq!(
            content_size(0.0, LINE_H, false, false, true),
            Vec2::new(16.0, 16.0)
        );
        // Empty label: zero-size (spec §9).
        assert_eq!(
            content_size(0.0, LINE_H, false, false, false),
            Vec2::new(0.0, 0.0)
        );
    }

    #[test]
    fn content_size_height_is_tallest_child() {
        // Text-only height is the 14px line box; a 16px icon beside 14px
        // text makes the row 16px tall (spec §3.2).
        assert_eq!(content_size(30.0, LINE_H, true, false, false).y, 14.0);
        assert_eq!(content_size(30.0, LINE_H, true, true, false).y, 16.0);
        // Wrapped text: one 14px line box per line; a 2-line text child
        // (28px) is taller than the 16px icon (spec §9).
        assert_eq!(content_size(30.0, 28.0, true, false, false).y, 28.0);
        assert_eq!(content_size(30.0, 28.0, true, true, false).y, 28.0);
    }

    #[test]
    fn sense_is_never_focusable() {
        // Spec §7.3: the label adds no Tab stop in any state (OPEN-6 is
        // verified at runtime by S6; the sense itself must never carry
        // FOCUSABLE).
        for (has_target, disabled) in [(false, false), (true, false), (false, true), (true, true)] {
            let sense = label_sense(has_target, disabled);
            assert!(!sense.is_focusable(), "{has_target} {disabled}");
            assert_eq!(sense.senses_click(), has_target && !disabled);
        }
    }

    #[test]
    fn disabled_label_is_half_alpha() {
        for mode in [Mode::Light, Mode::Dark] {
            let p = Preset::default().build(mode).palette;
            assert_eq!(text_color(&p, false), p.foreground);
            assert_eq!(text_color(&p, true), color_alpha(p.foreground, 0.5));
        }
    }

    #[test]
    fn builder_fields() {
        let label = Label::new("Email")
            .icon(Icon::User)
            .icon_end(Icon::Check)
            .for_id(Id::unique("email"))
            .disabled(true)
            .id_salt("email_label");
        assert_eq!(label.text, "Email");
        assert!(label.icon.is_some());
        assert!(label.icon_end.is_some());
        assert!(label.disabled);
        assert_eq!(label.for_id, Some(Id::unique("email")));
        assert_eq!(label.id_salt.as_deref(), Some("email_label"));
    }

    // --- S9 harness tests: real egui passes (chart.rs / button.rs pattern).
    //
    // Each test drives a fresh `Context` through explicit passes so the
    // click + deferred-forward sequence (LBL-INT-01/02) and the wrap
    // layout (LBL-VIS-01) are exercised exactly as the gallery runs them.
    // The Ui id and rect are fixed within a test so widget ids are
    // stable across passes.

    fn test_ctx() -> egui::Context {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        ctx
    }

    fn run_ui(ctx: &egui::Context, input: egui::RawInput, mut f: impl FnMut(&mut Ui)) {
        let mut output = ctx.run_ui(input, |ctx| {
            let mut ui = Ui::new(
                ctx.clone(),
                Id::unique("label_s9_test"),
                UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 800.0))),
            );
            f(&mut ui);
        });
        // No renderer consumes the texture uploads in a headless test;
        // dropping an unapplied delta panics, so clear it explicitly.
        output.textures_delta.clear();
    }

    fn click_event(pos: Pos2, pressed: bool) -> egui::RawInput {
        egui::RawInput {
            events: vec![egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::default(),
            }],
            ..Default::default()
        }
    }

    #[test]
    fn long_text_wraps_within_available_width() {
        // LBL-VIS-01: the demo §10 sentence wraps in a 320px container
        // (width <= 320, height a multiple of 14px, >= 2 lines) and
        // stays single-line where it fits; short labels are unchanged.
        let ctx = test_ctx();
        let sentence =
            "This is a very long label that exceeds the width of its container by a wide margin";

        let mut wide = Rect::NOTHING;
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            wide = Label::new(sentence).show(ui).rect;
        });
        assert_eq!(wide.height(), 14.0, "wide container: single line");
        assert!(wide.width() > 320.0, "sentence is wider than 320px");

        let mut narrow = Rect::NOTHING;
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            narrow = ui
                .allocate_ui(Vec2::new(320.0, 60.0), |ui| {
                    Label::new(sentence).show(ui).rect
                })
                .inner;
        });
        assert!(narrow.width() <= 320.0, "wrapped width: {narrow:?}");
        assert!(narrow.height() >= 28.0, "at least two lines: {narrow:?}");
        assert_eq!(narrow.height() % LINE_H, 0.0, "one 14px box per line");

        // A short label is byte-identical in both widths.
        let mut short_wide = Rect::NOTHING;
        let mut short_narrow = Rect::NOTHING;
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            short_wide = Label::new("Email address").show(ui).rect;
        });
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            short_narrow = ui
                .allocate_ui(Vec2::new(320.0, 60.0), |ui| {
                    Label::new("Email address").show(ui).rect
                })
                .inner;
        });
        assert_eq!(short_wide.height(), 14.0);
        assert_eq!(short_wide, short_narrow);
    }

    #[test]
    fn bound_label_click_focuses_and_toggles_checkbox() {
        // LBL-INT-01 + LBL-INT-02 (show path): a click on a bound label
        // focuses the checkbox AND toggles it on the deferred pass; a
        // second click toggles back. On unfixed code the focus is
        // surrendered in the click pass and nothing toggles.
        use crate::components::checkbox::Checkbox;

        let ctx = test_ctx();
        // Interior mutability: the render closure is reused across
        // passes, so state lives in cells the test can read between
        // passes. The checkbox borrow ends within each pass.
        let checked = std::cell::RefCell::new(false);
        let target = std::cell::Cell::new(Id::NULL);
        let label_rect = std::cell::Cell::new(Rect::NOTHING);

        let render = |ui: &mut Ui| {
            ui.horizontal(|ui| {
                let t = ui.scope_id().with("label_s9_cb");
                target.set(t);
                let lr = Label::new("Accept terms").for_id(t).show(ui);
                let mut guard = checked.borrow_mut();
                let cr = Checkbox::new(&mut guard).id_salt("label_s9_cb").show(ui);
                (lr, cr)
            })
            .inner
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| {
            label_rect.set(render(ui).0.rect);
        });
        assert!(!ctx.memory(|m| m.has_focus(target.get())));

        for expected in [true, false] {
            let center = label_rect.get().center();
            run_ui(&ctx, click_event(center, true), |ui| {
                render(ui);
            });
            let mut clicked = false;
            run_ui(&ctx, click_event(center, false), |ui| {
                clicked = render(ui).0.clicked();
            });
            assert!(clicked, "label click registers");
            // Deferred pass: no click in flight, the forwarded
            // focus + synthetic Space reach the checkbox.
            run_ui(&ctx, egui::RawInput::default(), |ui| {
                render(ui);
            });
            assert!(ctx.memory(|m| m.has_focus(target.get())), "INT-01: focused");
            assert_eq!(*checked.borrow(), expected, "INT-02: toggled");
        }
    }

    #[test]
    fn bound_label_click_activates_button_on_deferred_pass() {
        // LBL-INT-02 for a button target (show path): the forwarded
        // synthetic Space press fires the button's `clicked()` on the
        // deferred pass only — not on the press or release pass, and
        // not again on the pass after. The button path is distinct
        // from checkbox/radio/switch: its Space-fires-on-keyup handling
        // hides a lone key-down, so the key-down + key-up pair the
        // forwarding pushes is what produces the click.
        use crate::components::button::Button;

        let ctx = test_ctx();
        let target = std::cell::Cell::new(Id::NULL);
        let label_rect = std::cell::Cell::new(Rect::NOTHING);

        let render = |ui: &mut Ui| {
            ui.horizontal(|ui| {
                let t = ui.scope_id().with("label_s9_btn");
                target.set(t);
                let lr = Label::new("Submit").for_id(t).show(ui);
                let br = Button::new("Target button")
                    .id_salt("label_s9_btn")
                    .show(ui);
                (lr, br)
            })
            .inner
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| {
            label_rect.set(render(ui).0.rect);
        });
        let center = label_rect.get().center();
        run_ui(&ctx, click_event(center, true), |ui| {
            assert!(!render(ui).1.clicked(), "no activation on press pass");
        });
        run_ui(&ctx, click_event(center, false), |ui| {
            assert!(!render(ui).1.clicked(), "no activation on release pass");
        });
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            assert!(render(ui).1.clicked(), "INT-02: button activates");
        });
        assert!(ctx.memory(|m| m.has_focus(target.get())), "INT-01: focused");
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            assert!(!render(ui).1.clicked(), "no repeat activation");
        });
    }

    #[test]
    fn bound_label_click_focuses_input_without_inserting_text() {
        // LBL-INT-01 for a text target (show path): the click focuses
        // the input, and the synthetic Space press inserts nothing.
        //
        // The binding id is CAPTURED from the input's returned response:
        // `Input` shows its text edit inside a `ui.scope`, so its widget
        // id is not `ui.scope_id().with(id_salt)` (see `Label::for_id`).
        // Binding the computed id would target a widget that does not
        // exist; the demo §8a wiring has exactly this mismatch (S9
        // finding — demo-side fix, capture the response id).
        use crate::components::input::Input;

        let ctx = test_ctx();
        let text = std::cell::RefCell::new("m@example.com".to_string());
        let target = std::cell::Cell::new(Id::NULL);
        let label_rect = std::cell::Cell::new(Rect::NOTHING);

        // Pass 1: render unbound, capture the input's widget id.
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            ui.horizontal(|ui| {
                Label::new("Email").show(ui);
                let mut guard = text.borrow_mut();
                let ir = Input::new(&mut guard).id_salt("label_s9_input").show(ui);
                target.set(ir.id);
            });
        });
        assert_ne!(target.get(), Id::NULL);

        let render = |ui: &mut Ui| {
            ui.horizontal(|ui| {
                let lr = Label::new("Email").for_id(target.get()).show(ui);
                let mut guard = text.borrow_mut();
                let ir = Input::new(&mut guard).id_salt("label_s9_input").show(ui);
                (lr, ir)
            })
            .inner
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| {
            label_rect.set(render(ui).0.rect);
        });
        let center = label_rect.get().center();
        run_ui(&ctx, click_event(center, true), |ui| {
            render(ui);
        });
        run_ui(&ctx, click_event(center, false), |ui| {
            render(ui);
        });
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            render(ui);
        });
        assert!(ctx.memory(|m| m.has_focus(target.get())), "INT-01: focused");
        assert_eq!(*text.borrow(), "m@example.com", "no Space inserted");
    }

    #[test]
    fn bound_label_click_never_focuses_disabled_target() {
        // Regression guard (passes on unfixed code too): a click on a
        // label bound to a DISABLED checkbox focuses nothing and
        // activates nothing — the disabled pair stays inert.
        use crate::components::checkbox::Checkbox;

        let ctx = test_ctx();
        let checked = std::cell::RefCell::new(false);
        let target = std::cell::Cell::new(Id::NULL);
        let label_rect = std::cell::Cell::new(Rect::NOTHING);

        let render = |ui: &mut Ui| {
            ui.horizontal(|ui| {
                let t = ui.scope_id().with("label_s9_cb_dis");
                target.set(t);
                let lr = Label::new("Disabled terms").for_id(t).show(ui);
                let mut guard = checked.borrow_mut();
                let cr = Checkbox::new(&mut guard)
                    .id_salt("label_s9_cb_dis")
                    .disabled(true)
                    .show(ui);
                (lr, cr)
            })
            .inner
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| {
            label_rect.set(render(ui).0.rect);
        });
        let center = label_rect.get().center();
        run_ui(&ctx, click_event(center, true), |ui| {
            render(ui);
        });
        run_ui(&ctx, click_event(center, false), |ui| {
            render(ui);
        });
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            render(ui);
        });
        assert!(!ctx.memory(|m| m.has_focus(target.get())));
        assert!(!*checked.borrow());
    }

    #[test]
    fn show_with_click_forwards_to_nested_target_once() {
        // LBL-INT-01 + LBL-INT-02 (show_with path): a click on the label
        // text forwards to the NESTED checkbox exactly once (focus +
        // toggle); a direct click on the nested checkbox also toggles
        // exactly once (the target-itself exclusion). The nested
        // control lives one Ui scope deeper than the label, so its id
        // is captured from the first pass rather than guessed.
        use crate::components::checkbox::Checkbox;

        let ctx = test_ctx();
        let checked = std::cell::RefCell::new(false);
        let mut nested_id = Id::NULL;

        // Pass 1: capture the nested checkbox's id.
        run_ui(&ctx, egui::RawInput::default(), |ui| {
            let row = Label::new("Accept").show_with(ui, |ui| {
                let mut guard = checked.borrow_mut();
                Checkbox::new(&mut guard)
                    .id_salt("label_s9_nested")
                    .show(ui)
            });
            nested_id = row.inner.id;
        });
        assert_ne!(nested_id, Id::NULL);

        let row_rect = std::cell::Cell::new(Rect::NOTHING);
        let cb_rect = std::cell::Cell::new(Rect::NOTHING);
        let render = |ui: &mut Ui| {
            let row = Label::new("Accept").for_id(nested_id).show_with(ui, |ui| {
                let mut guard = checked.borrow_mut();
                Checkbox::new(&mut guard)
                    .id_salt("label_s9_nested")
                    .show(ui)
            });
            row_rect.set(row.response.rect);
            cb_rect.set(row.inner.rect);
        };

        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        let text_pt = Pos2::new(row_rect.get().left() + 8.0, row_rect.get().center().y);
        assert!(
            !cb_rect.get().contains(text_pt),
            "text point is on the label"
        );
        assert!(row_rect.get().contains(text_pt));

        // Click the label text: forwards exactly once.
        for expected in [true, false] {
            run_ui(&ctx, click_event(text_pt, true), |ui| render(ui));
            run_ui(&ctx, click_event(text_pt, false), |ui| render(ui));
            run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
            assert!(ctx.memory(|m| m.has_focus(nested_id)), "INT-01");
            assert_eq!(*checked.borrow(), expected, "INT-02: exactly one toggle");
            // Focus persists across later passes: the measure pass runs
            // in a disabled child Ui, and must not clear it (distinct
            // measure scope; with a shared scope this fails).
            run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
            run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
            assert!(
                ctx.memory(|m| m.has_focus(nested_id)),
                "INT-01: focus persists past the measure pass"
            );
            assert_eq!(*checked.borrow(), expected, "no extra toggle");
        }

        // Direct click on the nested checkbox: its own click toggles
        // exactly once; the forwarding must not fire it a second time.
        let cb_center = cb_rect.get().center();
        run_ui(&ctx, click_event(cb_center, true), |ui| render(ui));
        run_ui(&ctx, click_event(cb_center, false), |ui| render(ui));
        assert!(*checked.borrow(), "direct click toggles");
        run_ui(&ctx, egui::RawInput::default(), |ui| render(ui));
        assert!(*checked.borrow(), "no second toggle from forwarding");
    }
}
