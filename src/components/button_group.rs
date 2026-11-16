//! shadcn `button-group.tsx` — a row (or column) of fused controls.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/button-group.tsx` to the
//! frozen Batch A S0 spec (`batches-a-h/specs/button-group-spec.md`). The
//! container is `flex w-fit items-stretch`: a transparent row (column when
//! vertical) with no fill, border, radius, or shadow of its own. Its
//! children fuse: non-first children lose their leading radius and leading
//! border, non-last children lose their trailing radius, so adjacent
//! members share exactly ONE 1px border (border REMOVAL, not overlap —
//! spec §3.1). Members whose cross size is `auto`
//! ([`ButtonGroupText`], [`ButtonGroupSeparator`]) stretch to the
//! tallest/widest member in the SAME frame (spec §5); members with a
//! definite cross size (buttons in a horizontal group, icon buttons in a
//! vertical group) keep it, cross-start aligned.
//!
//! ## Layout engine: measure-then-real
//!
//! [`ButtonGroup::show`] runs the content closure at least TWICE: once
//! invisibly to measure the members' natural sizes (and to detect a
//! directly nested group), once for real with the true content-sized box.
//! The measuring passes are invisible (hence disabled), so no callbacks
//! can double-fire, and they run under distinct id scopes, so the
//! disabled pass cannot clear a focused widget's focus (the `create_widget`
//! / `ignore_focus` hazard). Only the real pass's return value is kept.
//! Callers must not rely on the closure running exactly once, and must not
//! put one-shot side effects in it. A third measuring pass runs only when
//! a gap applies (nested group, or an explicit [`.gap()`][ButtonGroup::gap]
//! override), so the gap is part of the measured size.
//!
//! The group presents its TRUE box to its parent: no `ui.horizontal` (which
//! seeds the row to the 36px interact size) and no zero-height seed (which
//! a centring parent centres wrongly). Positions (first/middle/last) come
//! from the shared plumbing in `crate::components::button::group`, which
//! learns the member count over frames: on the first frame the last child
//! is reported as `Middle`, and the settled state is correct from the
//! second frame (spec §14 anticipates this class of artifact; S7 judges
//! transients). Stretch, gap detection, and group size are all
//! same-frame — no frame-lagged stretch state.
//!
//! ## Gap rule (spec §8, D4)
//!
//! The 8px nested-group gap (`has-[>[data-slot=button-group]]:gap-2`) is
//! detected AUTOMATICALLY at layout time: a group that directly contains a
//! nested group gets an 8px gap between ALL its direct children, uniformly,
//! with fusing still applied. [`ButtonGroup::gap`] is an explicit override
//! for callers that need a different gap; leave it unset for the spec rule.
//!
//! ## Shared plumbing (FG-1/2/3, Foundation Loop)
//!
//! Three behaviours need channels in `button.rs` that a member cannot
//! provide from outside; they live in `crate::components::button::group`
//! and are wired here (S1b):
//!
//! * **FG-1 — stretch target.** The real pass pushes the measured cross
//!   size (`group::StretchTarget`); text buttons in vertical groups read
//!   it in `Button::show`, and Text/separator read it here. The one
//!   shared stack is the single source of truth — no group-local
//!   stretch state survives.
//! * **FG-2 — slot identity.** Children register the `data-slot` they
//!   carry: buttons self-register, the separator and nested groups
//!   register here, plain Text registers nothing. The measure pass
//!   learns the last slot-carrying child and the real pass transfers
//!   that index, so the nova last-`[data-slot]` rule (§3.2) is
//!   same-frame correct, including the trailing-non-slot edge case.
//! * **FG-3 — deferred focus rings.** Buttons queue their rings during
//!   the child pass; `group::end` flushes them into a foreground layer
//!   above following siblings (§4, `z-10`). Every group pass path —
//!   measure, gap-measure, and real — calls `begin`/`end`
//!   symmetrically, so no ring is left unflushed.
//!
//! ## Deviations and limitations (spec §15)
//!
//! * D1/D3/D6: `role="group"`/ARIA, `asChild`/Slot, and the separator's
//!   `decorative` prop have no egui meaning and no visual consequence;
//!   `decorative` is omitted from the Rust API (builder decision, D6).
//! * D5: the §13 select-trigger corner/width rules are a Batch F
//!   interface item, documented here as unimplemented — no select
//!   behaviour is invented.
//! * D8 (this implementation's interpretation): a horizontal-rule
//!   separator inside a horizontal group (§14 mismatch case) claims zero
//!   length — `w-full` against the group's indefinite `w-fit` width
//!   resolves to the empty content size. The normal compositions (vertical
//!   rule in a horizontal group, horizontal rule in a vertical group)
//!   stretch exactly per §6.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ButtonGroup::new().show(ui, |ui| {
//!     Button::new("Day").show(ui);
//!     Button::new("Month").show(ui);
//!     Button::new("Year").show(ui);
//! });
//!
//! ButtonGroup::new().show(ui, |ui| {
//!     ButtonGroupText::new("https://example.com").show(ui);
//!     Button::new("").size(ButtonSize::Icon).icon_start(Icon::Copy).show(ui);
//! });
//!
//! // A nested group triggers the 8px gap automatically (spec §8); no
//! // manual `.gap()` needed.
//! ButtonGroup::new().show(ui, |ui| {
//!     ButtonGroup::new().show(ui, |ui| {
//!         Button::new("Left").show(ui);
//!         Button::new("Right").show(ui);
//!     });
//!     Button::new("Submit").show(ui);
//! });
//! ```

use crate::components::button::group;
use crate::components::button::{fuse_radius, GroupOrientation, GroupPosition};
use crate::components::separator::SeparatorOrientation;
use crate::icons::Icon;
use crate::theme::fonts;
use crate::theme::tokens::{spacing, type_scale};
use crate::theme::{color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use crate::utils::shadow::{
    paint_shadow_layers, paint_shadow_resolved, resolve_shadow, ShadowLayer, ShadowSize, ShadowSlot,
};
use egui::{
    Align, Align2, Color32, Context, CornerRadius, FontId, Id, InnerResponse, Layout, Pos2, Rect,
    Response, Sense, Stroke, Ui, UiBuilder, Vec2, Widget,
};

/// A shadcn button group. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct ButtonGroup {
    orientation: GroupOrientation,
    /// Explicit gap override. `None` (the default) selects the automatic
    /// spec §8 rule: 0 gap, or 8px when the group directly contains a
    /// nested group.
    gap: Option<f32>,
    id_salt: Option<String>,
}

impl Default for ButtonGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonGroup {
    /// Creates a horizontal group (`orientation="horizontal"`).
    pub fn new() -> Self {
        Self {
            orientation: GroupOrientation::Horizontal,
            gap: None,
            id_salt: None,
        }
    }

    /// Sets the layout orientation.
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Overrides the gap between the group's children. By default the gap
    /// is automatic (spec §8): zero, or 8px (`gap-2`) when the group
    /// directly contains a nested [`ButtonGroup`]. An explicit gap wins
    /// over the automatic rule; prefer leaving it unset.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = Some(gap);
        self
    }

    /// Sets a stable id salt. Groups without one derive their id from the
    /// parent ui's widget counter, which is stable for static layouts.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its children.
    ///
    /// The content closure runs at least twice (see the [module
    /// docs](self)): once invisibly to measure, once for real. Only the
    /// real pass's return value is kept.
    pub fn show<R>(self, ui: &mut Ui, mut content: impl FnMut(&mut Ui) -> R) -> InnerResponse<R> {
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.next_auto_id(),
        };
        // Nested-group registration (§8): if this group is shown while an
        // ancestor group is measuring, that ancestor directly contains a
        // group and gets the 8px gap. Marking is idempotent — every group
        // on the path marks the innermost active measure frame, and the
        // direct-child group on the path marks it regardless.
        mark_parent_has_nested(ui.ctx());

        let rtl = ui.layout().prefer_right_to_left();
        let flow_min = ui.cursor().min;
        let flow_w = ui.available_size_before_wrap().x;
        let measure_rect = Rect::from_min_size(flow_min, Vec2::new(flow_w, f32::INFINITY));

        // Slot registration (§3.2 / FG-2): this group is itself a direct
        // child of an enclosing group, carrying `data-slot="button-group"`.
        // Consume the parent position — the group counts as one fused unit
        // (§3.3), so siblings after it get the positions they would have
        // had around any other member — and register, while the parent is
        // still the stack top (before this group's own `begin` calls).
        // No-op outside a group.
        if group::next_child_position(ui).is_some() {
            group::register_slot(ui, group::SlotIdentity::ButtonGroup);
        }

        // Measure pass 1: zero spacing, so the measured main size is the
        // exact sum of the members' natural main sizes and the measured
        // cross size is the tallest/widest member.
        let measure_id = id.with("button_group_measure");
        let (mut measured, has_nested) = measure_content(
            ui,
            measure_id,
            self.orientation,
            rtl,
            measure_rect,
            0.0,
            &mut content,
        );
        // The FG-2 transfer reads the LAST measure pass that ran: both
        // passes see the same children, so either finalizes the same
        // last-slot-child index.
        let mut last_measure_id = measure_id;

        // Gap decision (§8 / D4): automatic unless overridden.
        let gap = self
            .gap
            .unwrap_or(if has_nested { spacing::SM } else { 0.0 });

        // Measure pass 2 (only when a gap applies): re-measure with the
        // gap as the main-axis spacing, so the measured size includes it.
        if gap > 0.0 {
            let measure_gap_id = id.with("button_group_measure_gap");
            let (with_gap, _) = measure_content(
                ui,
                measure_gap_id,
                self.orientation,
                rtl,
                measure_rect,
                gap,
                &mut content,
            );
            measured = with_gap;
            last_measure_id = measure_gap_id;
        }

        // Real pass: present the true content-sized box, with the shared
        // stretch target pushed so auto-cross-size members stretch
        // same-frame (FG-1). The push happens immediately before the
        // real pass and the pop immediately after — never during a
        // measure pass, exactly one pop per push, same frame.
        let cross = match self.orientation {
            GroupOrientation::Horizontal => measured.y,
            GroupOrientation::Vertical => measured.x,
        };
        group::push_stretch_target(
            ui.ctx(),
            group::StretchTarget {
                orientation: self.orientation,
                cross,
            },
        );
        group::begin(ui, id, self.orientation);
        // FG-2 transfer: the measure pass learned which child is the last
        // slot-carrying child. Carry it into the real pass — which uses a
        // different id, so without the transfer the flag would come from
        // last frame's real pass instead of this frame's measurement.
        // `None` when no child carries a slot.
        group::set_last_slot_child(ui, id, group::last_slot_child(ui, last_measure_id));
        let response =
            ui.allocate_ui_with_layout(measured, group_layout(self.orientation, rtl), |ui| {
                ui.spacing_mut().item_spacing = gap_spacing(self.orientation, gap);
                content(ui)
            });
        group::end(ui, id);
        group::pop_stretch_target(ui.ctx());
        response
    }
}

/// The group layout for both passes: main axis per orientation, cross axis
/// `Align::Min` (cross-start — spec §14: definite-size members are aligned
/// to the cross start). No justification: stretch is delivered per-member
/// through the stretch target, never by expanding the placer frame (which
/// would also stretch definite-size members).
fn group_layout(orientation: GroupOrientation, rtl: bool) -> Layout {
    match orientation {
        GroupOrientation::Horizontal => {
            if rtl {
                Layout::right_to_left(Align::Min)
            } else {
                Layout::left_to_right(Align::Min)
            }
        }
        GroupOrientation::Vertical => Layout::top_down(Align::Min),
    }
}

/// Item spacing for a pass: the gap on the main axis, zero on the cross
/// axis.
fn gap_spacing(orientation: GroupOrientation, gap: f32) -> Vec2 {
    match orientation {
        GroupOrientation::Horizontal => Vec2::new(gap, 0.0),
        GroupOrientation::Vertical => Vec2::new(0.0, gap),
    }
}

/// Runs the content closure once invisibly to measure it, returning the
/// sanitised content size and whether a directly nested group was shown
/// during the pass.
///
/// The measure Ui is invisible (hence disabled: no callbacks can
/// double-fire) with the layout set on the `UiBuilder` at creation — a
/// centred parent would otherwise make the region seed compute NaN — and
/// with a `Min` cross-align, so the measured cross size is the tallest /
/// widest member rather than the (infinite) available extent. The Ui gets
/// a distinct id scope derived from `id`: widgets in the disabled pass
/// must never share ids with the real pass, or `create_widget`'s
/// `ignore_focus` call would clear a focused widget's focus every frame.
fn measure_content<R>(
    ui: &mut Ui,
    id: Id,
    orientation: GroupOrientation,
    rtl: bool,
    measure_rect: Rect,
    main_spacing: f32,
    content: &mut dyn FnMut(&mut Ui) -> R,
) -> (Vec2, bool) {
    push_measure_frame(ui.ctx());
    let mut measure = ui.new_child(
        UiBuilder::new()
            .max_rect(measure_rect)
            .layout(group_layout(orientation, rtl))
            .id_salt(id)
            .invisible(),
    );
    measure.spacing_mut().item_spacing = gap_spacing(orientation, main_spacing);
    group::begin(&measure, id, orientation);
    content(&mut measure);
    group::end(&measure, id);
    let size = sanitize_size(measure.min_rect().size());
    let has_nested = pop_measure_frame(ui.ctx());
    (size, has_nested)
}

/// Clamps a measured size to a finite, non-negative value: an empty group's
/// `min_rect` is `Rect::NOTHING` (negative size) and must yield zero size
/// (spec §14), never NaN or infinity.
fn sanitize_size(size: Vec2) -> Vec2 {
    Vec2::new(sanitize_len(size.x), sanitize_len(size.y))
}

fn sanitize_len(v: f32) -> f32 {
    if v.is_finite() {
        v.max(0.0)
    } else {
        0.0
    }
}

/// One in-progress measure pass: whether a directly nested group has been
/// shown during it (spec §8 gap detection).
#[derive(Clone, Copy, Debug, Default)]
struct MeasureFrame {
    has_nested: bool,
}

fn measure_stack_id() -> Id {
    Id::unique("shadcn_button_group_measure_stack")
}

fn with_measure_stack<R>(ctx: &Context, f: impl FnOnce(&mut Vec<MeasureFrame>) -> R) -> R {
    ctx.data_mut(|d| {
        let mut stack: Vec<MeasureFrame> = d.get_temp(measure_stack_id()).unwrap_or_default();
        let result = f(&mut stack);
        d.insert_temp(measure_stack_id(), stack);
        result
    })
}

/// Marks the innermost active measure frame as containing a nested group.
/// No-op when no group is measuring (top-level show, or a real pass).
fn mark_parent_has_nested(ctx: &Context) {
    with_measure_stack(ctx, |stack| {
        if let Some(top) = stack.last_mut() {
            top.has_nested = true;
        }
    });
}

fn push_measure_frame(ctx: &Context) {
    with_measure_stack(ctx, |stack| stack.push(MeasureFrame::default()));
}

/// Pops the innermost measure frame, returning whether it saw a nested
/// group.
fn pop_measure_frame(ctx: &Context) -> bool {
    with_measure_stack(ctx, |stack| {
        stack.pop().is_some_and(|frame| frame.has_nested)
    })
}

/// Group-aware chrome shared by [`ButtonGroupText`]: fused corners on
/// the FULL border box. Mirrors [`Button`][crate::components::Button]'s
/// post-BTG-VIS-01 paint: a non-first child fills from the shared edge
/// itself, and border removal is achieved by the fused-stroke painter
/// omitting the leading edge of the ring — NOT by insetting the draw
/// rect (the old inset left the shared column painted by neither
/// member). Corners come from [`fuse_radius`], the single source of
/// truth for fusing.
pub(crate) fn group_adjustments(
    orientation: GroupOrientation,
    position: GroupPosition,
    base_corner: CornerRadius,
    rect: egui::Rect,
) -> (CornerRadius, egui::Rect) {
    (fuse_radius(base_corner, orientation, position), rect)
}

/// Paints a Text slot's border ring with the leading edge omitted for
/// non-first children (spec §3.1 border removal), mirroring the
/// foundation fusing fix in [`Button`][crate::components::Button]
/// (`paint_fused_stroke`, BTG-VIS-01): the full `Inside` ring is
/// painted through a clip that excludes the leading 1px column
/// (horizontal groups) or row (vertical groups), and the two leading
/// corner cells are painted back as solid stroke-color 1px squares, so
/// the top/bottom rows stay continuous across the shared edge. Leading
/// corners are square for non-first children ([`fuse_radius`]), so the
/// corner cells are exact 1px squares and the clipped ring plus the
/// cells tile with no overlap or gap at any fractional alignment (both
/// share the same boundary). A clip is exact in all states — including
/// the 50% disabled fade, where overpainting with the fill color would
/// double-blend. First/only children take the plain `rect_stroke`
/// path, identical to a standalone slot.
fn paint_text_fused_stroke(
    painter: &egui::Painter,
    rect: Rect,
    corner: CornerRadius,
    stroke: Stroke,
    orientation: GroupOrientation,
    position: GroupPosition,
) {
    if !matches!(position, GroupPosition::Middle | GroupPosition::Last) {
        painter.rect_stroke(rect, corner, stroke, egui::epaint::StrokeKind::Inside);
        return;
    }
    let (clip, cells) = match orientation {
        GroupOrientation::Horizontal => (
            Rect::from_min_max(Pos2::new(rect.min.x + 1.0, rect.min.y), rect.max),
            [
                Rect::from_min_max(rect.min, Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0)),
                Rect::from_min_max(
                    Pos2::new(rect.min.x, rect.max.y - 1.0),
                    Pos2::new(rect.min.x + 1.0, rect.max.y),
                ),
            ],
        ),
        GroupOrientation::Vertical => (
            Rect::from_min_max(Pos2::new(rect.min.x, rect.min.y + 1.0), rect.max),
            [
                Rect::from_min_max(rect.min, Pos2::new(rect.min.x + 1.0, rect.min.y + 1.0)),
                Rect::from_min_max(
                    Pos2::new(rect.max.x - 1.0, rect.min.y),
                    Pos2::new(rect.max.x, rect.min.y + 1.0),
                ),
            ],
        ),
    };
    painter.with_clip_rect(clip).rect_stroke(
        rect,
        corner,
        stroke,
        egui::epaint::StrokeKind::Inside,
    );
    for cell in cells {
        painter.rect_filled(cell, CornerRadius::ZERO, stroke.color);
    }
}

/// Resolved colors for [`ButtonGroupText`] (pure, testable).
pub(crate) fn text_colors(p: &Palette) -> (Color32, Color32, Stroke) {
    (p.muted, p.foreground, Stroke::new(1.0, p.border))
}

/// [`ButtonGroupText`] padding and corner radius per style: the base recipe
/// is `px-4 rounded-md`; nova overrides to `px-2.5 rounded-lg`
/// (`style-nova.css` `.cn-button-group-text`).
pub(crate) fn text_metrics(style: Style, radius: &RadiusTokens) -> (f32, u8) {
    match style {
        Style::Nova => (spacing::SM_HALF, radius.lg_u8()), // px-2.5, rounded-lg
        _ => (spacing::LG, radius.md_u8()),                // px-4, rounded-md
    }
}

/// The Text label font: 14px `font-medium` (`text-sm` + `font-medium`,
/// spec §2.3), i.e. Inter Medium (500) from the Phase 1 embedded fonts.
pub(crate) fn text_font() -> FontId {
    fonts::medium(type_scale::SM)
}

/// The `text-sm` line box (spec §10): the Text row height with no vertical
/// padding declared, standalone or stretched from.
const TEXT_ROW_HEIGHT: f32 = 20.0;

/// Default icon size in a Text slot: `[&_svg:not([class*='size-'])]:size-4`
/// (spec §2.3).
const TEXT_ICON_SIZE: f32 = 16.0;

/// Width of the Text flex row's content (spec §§2.3, 14): every icon counts
/// its full width, the text run counts its measured width (only when the
/// label is non-empty), and every gap between adjacent items counts 8px
/// (`gap-2`).
pub(crate) fn text_content_width(text_width: f32, has_text: bool, icon_count: usize) -> f32 {
    let mut items = icon_count;
    let mut width = icon_count as f32 * TEXT_ICON_SIZE;
    if has_text {
        items += 1;
        width += text_width;
    }
    if items > 1 {
        width += (items - 1) as f32 * spacing::SM;
    }
    width
}

/// `flex items-center gap-2 rounded-md border bg-muted px-4 text-sm
/// font-medium shadow-xs` — the non-interactive group slot. Nova restyles it
/// to `rounded-lg px-2.5`. No height or width class exists in either
/// recipe: standalone, the slot is its content size (20px tall); in a
/// group it stretches to the tallest/widest member (spec §§5, 7.1).
///
/// `disabled` is a Rust-side extension (no disabled rule exists for Text
/// in either recipe, spec §9): it renders the slot at 50% opacity.
#[derive(Clone, Debug)]
pub struct ButtonGroupText {
    text: String,
    icon_start: Option<Icon>,
    icon_end: Option<Icon>,
    disabled: bool,
    id_salt: Option<String>,
}

impl ButtonGroupText {
    /// Creates a group text slot.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon_start: None,
            icon_end: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Shows a 16px icon before the text (`[&_svg:not([class*='size-'])]:
    /// size-4` slot with `gap-2`).
    pub fn icon_start(mut self, icon: Icon) -> Self {
        self.icon_start = Some(icon);
        self
    }

    /// Shows a 16px icon after the text, separated by the same 8px gap
    /// (spec §14: multiple icons are each separated by the gap).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    /// Renders at 50% opacity (Rust-side extension; see the struct docs).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the slot.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();

        let group = group::next_child_position(ui);
        let (orientation, position) =
            group.unwrap_or((GroupOrientation::Horizontal, GroupPosition::Only));

        let (mut fill, mut fg, mut stroke) = text_colors(&p);
        if self.disabled {
            fill = color_alpha(fill, 0.5);
            fg = color_alpha(fg, 0.5);
            stroke.color = color_alpha(stroke.color, 0.5);
        }
        // Laid out once with the paint color: the same galley sizes the
        // slot and paints the label. No wrapping (the slot grows to fit
        // its content, spec §14).
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), text_font(), fg);

        let (pad_x, corner_radius) = text_metrics(theme.style, &theme.radius);
        let has_text = !self.text.is_empty();
        let icon_count =
            usize::from(self.icon_start.is_some()) + usize::from(self.icon_end.is_some());
        let content_w = text_content_width(galley.size().x, has_text, icon_count);
        // px-4 (base) / px-2.5 (nova) on both sides; no vertical padding.
        let natural = Vec2::new(2.0 * pad_x + content_w, TEXT_ROW_HEIGHT);

        // Same-frame `items-stretch` (spec §5): the enclosing group pushes
        // the measured cross size before the real pass. In a measure pass
        // (no target pushed) and standalone, the natural size stands — and
        // the stretch target never shrinks the slot below its content.
        let desired = match group::stretch_target(ui) {
            Some(target) if target.orientation == GroupOrientation::Horizontal => {
                Vec2::new(natural.x, target.cross.max(natural.y))
            }
            Some(target) => Vec2::new(target.cross.max(natural.x), natural.y),
            None => natural,
        };

        let (rect, response) = match self.id_salt.as_deref() {
            Some(salt) => {
                let id = ui.scope_id().with(salt);
                let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
                (rect, ui.interact(rect, id, Sense::hover()))
            }
            None => ui.allocate_exact_size(desired, Sense::hover()),
        };

        if ui.is_rect_visible(rect) {
            let (corner, draw_rect) = group_adjustments(
                orientation,
                position,
                CornerRadius::same(corner_radius),
                rect,
            );
            let painter = ui.painter();
            // `shadow-xs` under BOTH recipes (spec §7.2, N4): painted on
            // the full border box with the fused corner, like the
            // outline button's shadow.
            paint_text_shadow(painter, rect, corner, theme.style, dark, self.disabled);
            painter.rect_filled(draw_rect, corner, fill);
            paint_text_fused_stroke(painter, draw_rect, corner, stroke, orientation, position);

            // Content: [start icon] gap [text] gap [end icon], starting
            // at the leading padding edge of the border box,
            // vertically centred (`flex items-center`). Icons never
            // receive pointer events (the slot has hover sense only).
            let cy = rect.center().y;
            let mut cursor = rect.min.x + pad_x;
            let mut started = false;
            if let Some(icon) = self.icon_start {
                if started {
                    cursor += spacing::SM;
                }
                let r = Rect::from_center_size(
                    Pos2::new(cursor + TEXT_ICON_SIZE * 0.5, cy),
                    Vec2::splat(TEXT_ICON_SIZE),
                );
                icon.paint(painter, r, fg, 2.0 * TEXT_ICON_SIZE / 24.0);
                cursor += TEXT_ICON_SIZE;
                started = true;
            }
            if has_text {
                if started {
                    cursor += spacing::SM;
                }
                // Same paint math as `Painter::text`, reusing the galley.
                let text_size = galley.size();
                let text_rect = Align2::LEFT_CENTER.anchor_size(Pos2::new(cursor, cy), text_size);
                painter.galley(text_rect.min, galley, fg);
                cursor += text_size.x;
                started = true;
            }
            if let Some(icon) = self.icon_end {
                if started {
                    cursor += spacing::SM;
                }
                let r = Rect::from_center_size(
                    Pos2::new(cursor + TEXT_ICON_SIZE * 0.5, cy),
                    Vec2::splat(TEXT_ICON_SIZE),
                );
                icon.paint(painter, r, fg, 2.0 * TEXT_ICON_SIZE / 24.0);
            }
        }

        response
    }
}

impl Widget for ButtonGroupText {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Paints the Text slot's `shadow-xs` through the LAYER resolution API
/// (spec §7.2). When disabled, the shadow fades with the element's
/// opacity, mirroring the outline button's disabled shadow: the layer
/// alphas scale on the stack (no allocation).
fn paint_text_shadow(
    painter: &egui::Painter,
    rect: Rect,
    corner: CornerRadius,
    style: Style,
    dark: bool,
    disabled: bool,
) {
    if disabled {
        let layers = resolve_shadow(ShadowSize::Xs, style, ShadowSlot::ButtonGroupText).layers();
        // Tailwind shadows have at most two layers; the stack buffer covers
        // every size in the scale.
        let mut faded = [ShadowLayer {
            offset: [0.0, 0.0],
            blur: 0.0,
            spread: 0.0,
            alpha: 0.0,
        }; 2];
        let n = layers.len().min(2);
        for (i, layer) in layers.iter().take(2).enumerate() {
            faded[i] = ShadowLayer {
                alpha: layer.alpha * 0.5,
                ..*layer
            };
        }
        paint_shadow_layers(painter, rect, corner, &faded[..n], dark);
    } else {
        paint_shadow_resolved(
            painter,
            rect,
            corner,
            ShadowSize::Xs,
            style,
            ShadowSlot::ButtonGroupText,
            dark,
        );
    }
}

/// `bg-input self-stretch w-px` — the divider inside a group. Counts toward
/// the group's child positions like any other member. Its length is ALWAYS
/// stretched (spec §6): a vertical rule spans the group height, a
/// horizontal rule the group width. No fixed length exists anywhere.
///
/// The [`Self::orientation`] builder names the GROUP axis the divider lives
/// in, and the rule is perpendicular to it: `Horizontal` (the default)
/// renders the spec's default vertical rule; `Vertical` renders the
/// horizontal rule that is the normal choice inside a vertical group
/// (spec §6). A rule parallel to its group axis (the §14 mismatch
/// compositions) is not expressible through this API.
#[derive(Clone, Debug)]
pub struct ButtonGroupSeparator {
    rule: SeparatorOrientation,
    id_salt: Option<String>,
}

impl Default for ButtonGroupSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonGroupSeparator {
    /// Creates a vertical (1px wide) divider (`orientation="vertical"`,
    /// spec §6) that stretches to the group height. In a vertical group,
    /// pass [`.orientation(GroupOrientation::Vertical)`][Self::orientation]
    /// for the horizontal rule.
    pub fn new() -> Self {
        Self {
            rule: SeparatorOrientation::Vertical,
            id_salt: None,
        }
    }

    /// Sets the group axis the divider lives in: `Horizontal` (the default)
    /// renders a vertical 1px rule; `Vertical` renders a horizontal rule.
    /// The rule is always perpendicular to the named axis.
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.rule = match orientation {
            GroupOrientation::Horizontal => SeparatorOrientation::Vertical,
            GroupOrientation::Vertical => SeparatorOrientation::Horizontal,
        };
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the divider: a 1px filled rect in the `input` token, flush
    /// against its neighbours (`m-0!`), never compressed (`shrink-0`, via
    /// `allocate_exact_size`).
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        // Count toward the group (this is a group member like any other).
        // Fusing is inert for a 1px line (spec §14): no border, no radius.
        let group = group::next_child_position(ui);
        // FG-2: the separator carries `data-slot="button-group-separator"`.
        // Registered after the position call, so it lands on this child's
        // index; the flag itself is inert for a 1px line (spec §14).
        if group.is_some() {
            group::register_slot(ui, group::SlotIdentity::ButtonGroupSeparator);
        }
        let size = separator_size(
            self.rule,
            group.map(|(orientation, _)| orientation),
            group::stretch_target(ui).map(|target| target.cross),
            Vec2::new(ui.available_width(), ui.available_height()),
        );
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().rect_filled(rect, 0, theme.palette.input);
        }
        response
    }
}

impl Widget for ButtonGroupSeparator {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// Resolves the separator's allocated size (spec §6; pure, testable).
///
/// * Real pass inside a group (`Some` orientation + `Some` stretch): the
///   rule's length stretches to the group's cross size when the length
///   runs across the group (vertical rule in a horizontal group,
///   horizontal rule in a vertical group). When the length runs ALONG the
///   group (the §14 mismatch compositions), the length is the empty
///   content size, 0 — see D8 in the [module docs](self).
/// * Measure pass inside a group (`Some` orientation, no stretch target
///   yet): only the 1px thickness is claimed, so the group's cross size is
///   measured from the other members; the real pass stretches to it.
/// * Standalone (`None` orientation): the full available extent on the
///   length axis, like [`Separator::size_in`][crate::components::Separator::size_in].
pub(crate) fn separator_size(
    rule: SeparatorOrientation,
    group_orientation: Option<GroupOrientation>,
    stretch_cross: Option<f32>,
    available: Vec2,
) -> Vec2 {
    match (group_orientation, stretch_cross) {
        (Some(GroupOrientation::Horizontal), Some(cross)) => match rule {
            SeparatorOrientation::Vertical => Vec2::new(1.0, cross),
            SeparatorOrientation::Horizontal => Vec2::new(0.0, 1.0),
        },
        (Some(GroupOrientation::Vertical), Some(cross)) => match rule {
            SeparatorOrientation::Horizontal => Vec2::new(cross, 1.0),
            SeparatorOrientation::Vertical => Vec2::new(1.0, 0.0),
        },
        (Some(_), None) => match rule {
            SeparatorOrientation::Vertical => Vec2::new(1.0, 0.0),
            SeparatorOrientation::Horizontal => Vec2::new(0.0, 1.0),
        },
        (None, _) => match rule {
            SeparatorOrientation::Vertical => Vec2::new(1.0, available.y.max(0.0)),
            SeparatorOrientation::Horizontal => Vec2::new(available.x.max(0.0), 1.0),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::button::{Button, ButtonSize};
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect, UiBuilder};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    /// Runs `show` for `frames` frames in a fresh context with the given
    /// style/mode installed, collecting one value per frame. The group sets
    /// its own item spacing in every pass, so results do not depend on the
    /// ambient spacing.
    ///
    /// Geometry tests must have the content closure RETURN its
    /// measurements and read them from `InnerResponse::inner`: the closure
    /// runs in the measure passes too, so pushing into a shared `Vec`
    /// would mix measure-pass values (natural sizes) with real-pass
    /// values (stretched sizes).
    fn run_frames<T, F>(style: Style, mode: Mode, frames: usize, mut show: F) -> Vec<T>
    where
        F: FnMut(&mut Ui) -> T,
    {
        let ctx = egui::Context::default();
        Preset {
            style,
            ..Preset::default()
        }
        .build(mode)
        .install(&ctx);
        let mut collected = Vec::new();
        for _ in 0..frames {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("button_group_test"),
                    UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                collected.push(show(ui));
            });
            out.textures_delta.clear();
        }
        collected
    }

    /// Like [`run_frames`], but returns the LAST frame's [`egui::FullOutput`]
    /// alongside the value the closure returned on that frame, so tests can
    /// inspect paint order and shape geometry. Measure passes paint nothing
    /// (invisible Uis emit `Shape::Noop`), so the output's shapes are the
    /// real pass's alone.
    fn run_frames_output<T, F>(
        style: Style,
        mode: Mode,
        frames: usize,
        mut show: F,
    ) -> (egui::FullOutput, T)
    where
        F: FnMut(&mut Ui) -> T,
    {
        let ctx = egui::Context::default();
        Preset {
            style,
            ..Preset::default()
        }
        .build(mode)
        .install(&ctx);
        let mut values = Vec::new();
        let mut last_out = None;
        for _ in 0..frames {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("button_group_test"),
                    UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                values.push(show(ui));
            });
            out.textures_delta.clear();
            last_out = Some(out);
        }
        (
            last_out.expect("at least one frame"),
            values.pop().expect("at least one value"),
        )
    }

    /// A 500x500 test Ui with a stable id, for tests that drive the clock
    /// frame by frame (FG-3).
    fn fg_test_ui(ctx: &egui::Context, name: &str) -> egui::Ui {
        egui::Ui::new(
            ctx.clone(),
            egui::Id::unique(name),
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 500.0))),
        )
    }

    /// Input carrying only a controlled clock time.
    fn input_at(time: f64) -> egui::RawInput {
        egui::RawInput {
            time: Some(time),
            ..Default::default()
        }
    }

    /// Border-stroke shapes (`Shape::Rect` with a 1px stroke and a
    /// transparent fill) in paint order, with their corner radii.
    fn border_strokes(out: &egui::FullOutput) -> Vec<(Rect, CornerRadius)> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.stroke.width == 1.0 && rs.fill == Color32::TRANSPARENT =>
                {
                    Some((rs.rect, rs.corner_radius))
                }
                _ => None,
            })
            .collect()
    }

    /// Fill shapes (non-transparent fill, no stroke) in paint order, with
    /// their corner radii.
    fn fill_shapes(out: &egui::FullOutput) -> Vec<(Rect, CornerRadius)> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.fill != Color32::TRANSPARENT && rs.stroke == Stroke::NONE =>
                {
                    Some((rs.rect, rs.corner_radius))
                }
                _ => None,
            })
            .collect()
    }

    /// Whether two rects coincide (sub-pixel tolerance for layout math).
    fn same_rect(a: Rect, b: Rect) -> bool {
        (a.min - b.min).length() < 0.01 && (a.max - b.max).length() < 0.01
    }

    /// The rect a group member paints on: the full border box at every
    /// position (BTG-VIS-01: no shared-edge inset; border removal is the
    /// stroke painter's job).
    fn draw_rect_for(rect: Rect, position: GroupPosition) -> Rect {
        group_adjustments(
            GroupOrientation::Horizontal,
            position,
            CornerRadius::same(0),
            rect,
        )
        .1
    }

    #[test]
    fn text_slot_uses_muted_tokens() {
        let p = palette(Mode::Light);
        let (fill, fg, stroke) = text_colors(&p);
        assert_eq!(fill, p.muted);
        assert_eq!(fg, p.foreground);
        assert_eq!(stroke.color, p.border);
    }

    #[test]
    fn nova_text_slot_uses_lg_radius_and_px_2_5() {
        let theme = Preset::default().build(Mode::Light);
        let (pad, corner) = text_metrics(Style::Nova, &theme.radius);
        assert_eq!(pad, 10.0); // px-2.5
        assert_eq!(corner, theme.radius.lg_u8()); // rounded-lg
        let (pad, corner) = text_metrics(Style::Vega, &theme.radius);
        assert_eq!(pad, 16.0); // px-4
        assert_eq!(corner, theme.radius.md_u8()); // rounded-md
    }

    #[test]
    fn text_font_is_medium_14() {
        let font = text_font();
        assert_eq!(font.size, 14.0);
        assert_eq!(font.family, fonts::medium_family());
    }

    #[test]
    fn text_shadow_resolves_to_xs_under_both_recipes() {
        // N4 / LAYER: no style rule overrides the base `shadow-xs`.
        for style in [Style::Vega, Style::Nova] {
            assert_eq!(
                resolve_shadow(ShadowSize::Xs, style, ShadowSlot::ButtonGroupText),
                ShadowSize::Xs,
                "shadow-xs survives under {style:?}"
            );
        }
    }

    #[test]
    fn builders_keep_fields() {
        let g = ButtonGroup::new()
            .orientation(GroupOrientation::Vertical)
            .gap(spacing::SM)
            .id_salt("g");
        assert_eq!(g.orientation, GroupOrientation::Vertical);
        assert_eq!(g.gap, Some(8.0));
        assert_eq!(g.id_salt.as_deref(), Some("g"));
        assert_eq!(ButtonGroup::new().gap, None);
        let t = ButtonGroupText::new("x")
            .icon_start(Icon::Copy)
            .icon_end(Icon::Check)
            .disabled(true)
            .id_salt("t");
        assert!(t.icon_start.is_some());
        assert!(t.icon_end.is_some());
        assert!(t.disabled);
        let s = ButtonGroupSeparator::new().id_salt("s");
        assert_eq!(s.rule, SeparatorOrientation::Vertical);
    }

    #[test]
    fn default_separator_is_a_vertical_rule() {
        let s = ButtonGroupSeparator::new();
        assert_eq!(s.rule, SeparatorOrientation::Vertical);
        let s = ButtonGroupSeparator::new().orientation(GroupOrientation::Vertical);
        assert_eq!(s.rule, SeparatorOrientation::Horizontal);
        let s = ButtonGroupSeparator::new().orientation(GroupOrientation::Horizontal);
        assert_eq!(s.rule, SeparatorOrientation::Vertical);
    }

    #[test]
    fn group_adjustments_fuse_every_position() {
        let base = CornerRadius::same(8);
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(40.0, 36.0));
        // (orientation, position, (nw, ne, sw, se)); the draw rect is the
        // FULL border box at every position (BTG-VIS-01: border removal
        // is the stroke painter's job, not a draw-rect inset).
        let cases = [
            (
                GroupOrientation::Horizontal,
                GroupPosition::Only,
                (8, 8, 8, 8),
            ),
            (
                GroupOrientation::Horizontal,
                GroupPosition::First,
                (8, 0, 8, 0),
            ),
            (
                GroupOrientation::Horizontal,
                GroupPosition::Middle,
                (0, 0, 0, 0),
            ),
            (
                GroupOrientation::Horizontal,
                GroupPosition::Last,
                (0, 8, 0, 8),
            ),
            (
                GroupOrientation::Vertical,
                GroupPosition::Only,
                (8, 8, 8, 8),
            ),
            (
                GroupOrientation::Vertical,
                GroupPosition::First,
                (8, 8, 0, 0),
            ),
            (
                GroupOrientation::Vertical,
                GroupPosition::Middle,
                (0, 0, 0, 0),
            ),
            (
                GroupOrientation::Vertical,
                GroupPosition::Last,
                (0, 0, 8, 8),
            ),
        ];
        for (orientation, position, corners) in cases {
            let (corner, draw) = group_adjustments(orientation, position, base, rect);
            assert_eq!(
                (corner.nw, corner.ne, corner.sw, corner.se),
                corners,
                "{orientation:?} {position:?}"
            );
            assert_eq!(draw, rect, "{orientation:?} {position:?}");
        }
    }

    #[test]
    fn text_mirror_paints_full_rect_and_omits_leading_edge_horizontal() {
        // BTG-VIS-01 mirror (horizontal): a non-first Text fills the
        // full border box and its ring omits the leading edge — the
        // stroke shape sits on the full rect with a clip that starts
        // exactly 1px in, plus two 1px corner cells. On unfixed code
        // the fill and stroke sit on the inset rect and there are no
        // cells, so every assertion here fails.
        let (out, text_rect) = run_frames_output(Style::Vega, Mode::Light, 2, |ui| {
            ButtonGroup::new()
                .show(ui, |ui| {
                    Button::new("A").show(ui);
                    ButtonGroupText::new("text").show(ui).rect
                })
                .inner
        });
        let p = palette(Mode::Light);
        let (fill, _, stroke) = text_colors(&p);
        let (fills, stroke_clip, cells) = text_paint_shapes(&out, text_rect, fill, stroke);
        assert_eq!(fills, 1, "one muted fill on the full Text rect");
        let clip = stroke_clip.expect("a border stroke on the full Text rect");
        assert!(
            (clip.min.x - (text_rect.min.x + 1.0)).abs() < 0.01
                && (clip.min.y - text_rect.min.y).abs() < 0.01,
            "stroke clip starts exactly 1px in: {clip:?} vs {text_rect:?}"
        );
        assert!(
            (clip.max - text_rect.max).length() < 0.01,
            "stroke clip ends at the rect max: {clip:?} vs {text_rect:?}"
        );
        assert_eq!(cells.len(), 2, "two leading corner cells");
        let expected = [
            Rect::from_min_max(
                text_rect.min,
                Pos2::new(text_rect.min.x + 1.0, text_rect.min.y + 1.0),
            ),
            Rect::from_min_max(
                Pos2::new(text_rect.min.x, text_rect.max.y - 1.0),
                Pos2::new(text_rect.min.x + 1.0, text_rect.max.y),
            ),
        ];
        for cell in expected {
            assert!(
                cells.iter().any(|c| same_rect(*c, cell)),
                "corner cell at {cell:?} among {cells:?}"
            );
        }
    }

    #[test]
    fn text_mirror_paints_full_rect_and_omits_leading_edge_vertical() {
        // BTG-VIS-01 mirror (vertical): the clip excludes the top row
        // and the corner cells sit at the top-left/top-right.
        let (out, text_rect) = run_frames_output(Style::Vega, Mode::Light, 2, |ui| {
            ButtonGroup::new()
                .orientation(GroupOrientation::Vertical)
                .show(ui, |ui| {
                    Button::new("A").show(ui);
                    ButtonGroupText::new("text").show(ui).rect
                })
                .inner
        });
        let p = palette(Mode::Light);
        let (fill, _, stroke) = text_colors(&p);
        let (fills, stroke_clip, cells) = text_paint_shapes(&out, text_rect, fill, stroke);
        assert_eq!(fills, 1, "one muted fill on the full Text rect");
        let clip = stroke_clip.expect("a border stroke on the full Text rect");
        assert!(
            (clip.min.x - text_rect.min.x).abs() < 0.01
                && (clip.min.y - (text_rect.min.y + 1.0)).abs() < 0.01,
            "stroke clip starts exactly 1px in: {clip:?} vs {text_rect:?}"
        );
        assert!(
            (clip.max - text_rect.max).length() < 0.01,
            "stroke clip ends at the rect max: {clip:?} vs {text_rect:?}"
        );
        assert_eq!(cells.len(), 2, "two leading corner cells");
        let expected = [
            Rect::from_min_max(
                text_rect.min,
                Pos2::new(text_rect.min.x + 1.0, text_rect.min.y + 1.0),
            ),
            Rect::from_min_max(
                Pos2::new(text_rect.max.x - 1.0, text_rect.min.y),
                Pos2::new(text_rect.max.x, text_rect.min.y + 1.0),
            ),
        ];
        for cell in expected {
            assert!(
                cells.iter().any(|c| same_rect(*c, cell)),
                "corner cell at {cell:?} among {cells:?}"
            );
        }
    }

    #[test]
    fn standalone_text_slot_paint_is_unchanged() {
        // A standalone Text (Only) takes the plain `rect_stroke` path:
        // one fill + one unclipped ring on the full rect, zero corner
        // cells. Pins the plain path the mirror must not disturb.
        let (out, text_rect) = run_frames_output(Style::Vega, Mode::Light, 1, |ui| {
            ButtonGroupText::new("text").show(ui).rect
        });
        let p = palette(Mode::Light);
        let (fill, _, stroke) = text_colors(&p);
        let (fills, stroke_clip, cells) = text_paint_shapes(&out, text_rect, fill, stroke);
        assert_eq!(fills, 1, "one muted fill on the full Text rect");
        let clip = stroke_clip.expect("a border stroke on the full Text rect");
        assert!(
            clip.min.x <= text_rect.min.x + 0.01 && clip.min.y <= text_rect.min.y + 0.01,
            "standalone ring unclipped: {clip:?} vs {text_rect:?}"
        );
        assert!(cells.is_empty(), "no corner cells standalone");
    }

    /// Text paint shapes on `target`: (muted fills, stroke clip rect,
    /// 1px corner cells). Corner cells are 1x1 stroke-color fills with
    /// zero corner radius.
    fn text_paint_shapes(
        out: &egui::FullOutput,
        target: Rect,
        fill: Color32,
        stroke: Stroke,
    ) -> (usize, Option<Rect>, Vec<Rect>) {
        let mut fills = 0;
        let mut stroke_clip = None;
        let mut cells = Vec::new();
        for clipped in &out.shapes {
            if let egui::Shape::Rect(rs) = &clipped.shape {
                if same_rect(rs.rect, target) && rs.fill == fill && rs.stroke == Stroke::NONE {
                    fills += 1;
                }
                if same_rect(rs.rect, target)
                    && rs.fill == Color32::TRANSPARENT
                    && rs.stroke.width == 1.0
                {
                    stroke_clip = Some(clipped.clip_rect);
                }
                if rs.fill == stroke.color
                    && rs.stroke == Stroke::NONE
                    && rs.corner_radius == CornerRadius::ZERO
                    && (rs.rect.width() - 1.0).abs() < 0.01
                    && (rs.rect.height() - 1.0).abs() < 0.01
                {
                    cells.push(rs.rect);
                }
            }
        }
        (fills, stroke_clip, cells)
    }

    #[test]
    fn separator_size_resolves_every_case() {
        // Real pass, normal compositions: stretched to the cross size.
        assert_eq!(
            separator_size(
                SeparatorOrientation::Vertical,
                Some(GroupOrientation::Horizontal),
                Some(36.0),
                Vec2::ZERO
            ),
            Vec2::new(1.0, 36.0)
        );
        assert_eq!(
            separator_size(
                SeparatorOrientation::Horizontal,
                Some(GroupOrientation::Vertical),
                Some(80.0),
                Vec2::ZERO
            ),
            Vec2::new(80.0, 1.0)
        );
        // Real pass, mismatch compositions (D8): zero length.
        assert_eq!(
            separator_size(
                SeparatorOrientation::Horizontal,
                Some(GroupOrientation::Horizontal),
                Some(36.0),
                Vec2::ZERO
            ),
            Vec2::new(0.0, 1.0)
        );
        assert_eq!(
            separator_size(
                SeparatorOrientation::Vertical,
                Some(GroupOrientation::Vertical),
                Some(80.0),
                Vec2::ZERO
            ),
            Vec2::new(1.0, 0.0)
        );
        // Measure pass: thickness only, so the cross size is measured from
        // the other members.
        assert_eq!(
            separator_size(
                SeparatorOrientation::Vertical,
                Some(GroupOrientation::Horizontal),
                None,
                Vec2::ZERO
            ),
            Vec2::new(1.0, 0.0)
        );
        assert_eq!(
            separator_size(
                SeparatorOrientation::Horizontal,
                Some(GroupOrientation::Vertical),
                None,
                Vec2::ZERO
            ),
            Vec2::new(0.0, 1.0)
        );
        // Standalone: full available extent on the length axis.
        assert_eq!(
            separator_size(
                SeparatorOrientation::Vertical,
                None,
                None,
                Vec2::new(400.0, 300.0)
            ),
            Vec2::new(1.0, 300.0)
        );
        assert_eq!(
            separator_size(
                SeparatorOrientation::Horizontal,
                None,
                None,
                Vec2::new(400.0, 300.0)
            ),
            Vec2::new(400.0, 1.0)
        );
    }

    #[test]
    fn text_content_width_counts_items_and_gaps() {
        assert_eq!(text_content_width(50.0, true, 0), 50.0);
        assert_eq!(text_content_width(50.0, true, 1), 50.0 + 16.0 + 8.0);
        assert_eq!(text_content_width(50.0, true, 2), 50.0 + 32.0 + 16.0);
        assert_eq!(text_content_width(0.0, false, 1), 16.0);
        assert_eq!(text_content_width(0.0, false, 0), 0.0);
    }

    #[test]
    fn text_stretches_to_tallest_member_same_frame() {
        // Spec §§5, 7.1 (N2): Text height = tallest member, on frame 1 —
        // no frame-lagged stretch state.
        let frames = run_frames(Style::Vega, Mode::Light, 2, |ui| {
            let group = ButtonGroup::new().show(ui, |ui| {
                let button_h = Button::new("Tall").show(ui).rect.height();
                let text_h = ButtonGroupText::new("https://example.com")
                    .show(ui)
                    .rect
                    .height();
                vec![button_h, text_h]
            });
            (group.inner, group.response.rect.height())
        });
        for (heights, group_h) in &frames {
            assert_eq!(
                heights,
                &vec![36.0, 36.0],
                "default button 36px, Text stretched to 36px"
            );
            assert_eq!(*group_h, 36.0);
        }
    }

    #[test]
    fn nova_text_stretches_to_nova_button_height() {
        // No style-conditional code: a group of nova default buttons
        // (32px) yields a 32px-tall Text (spec §7.1).
        let frames = run_frames(Style::Nova, Mode::Light, 1, |ui| {
            ButtonGroup::new()
                .show(ui, |ui| {
                    vec![
                        Button::new("Nova").show(ui).rect.height(),
                        ButtonGroupText::new("text").show(ui).rect.height(),
                    ]
                })
                .inner
        });
        assert_eq!(frames[0], vec![32.0, 32.0]);
    }

    #[test]
    fn mixed_heights_text_matches_tallest() {
        // The N2 "ragged" case: xs (24px) + Text + default (36px).
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            ButtonGroup::new()
                .show(ui, |ui| {
                    vec![
                        Button::new("xs")
                            .size(ButtonSize::Xs)
                            .show(ui)
                            .rect
                            .height(),
                        ButtonGroupText::new("mid").show(ui).rect.height(),
                        Button::new("default").show(ui).rect.height(),
                    ]
                })
                .inner
        });
        assert_eq!(frames[0], vec![24.0, 36.0, 36.0]);
    }

    #[test]
    fn separator_stretches_to_group_height_same_frame() {
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            ButtonGroup::new()
                .show(ui, |ui| {
                    vec![
                        Button::new("A").show(ui).rect.size(),
                        ButtonGroupSeparator::new().show(ui).rect.size(),
                        Button::new("B").show(ui).rect.size(),
                    ]
                })
                .inner
        });
        assert_eq!(frames[0][1], Vec2::new(1.0, 36.0));
    }

    #[test]
    fn vertical_group_stretches_text_width_same_frame() {
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            let group = ButtonGroup::new()
                .orientation(GroupOrientation::Vertical)
                .show(ui, |ui| {
                    vec![
                        Button::new("A fairly long label").show(ui).rect.size(),
                        ButtonGroupText::new("x").show(ui).rect.size(),
                    ]
                });
            (group.inner, group.response.rect.size())
        });
        let (sizes, group_size) = &frames[0];
        assert_eq!(sizes[1].x, sizes[0].x, "Text width == button width");
        assert_eq!(sizes[1].x, group_size.x, "Text width == group width");
        assert_eq!(sizes[1].y, 20.0, "Text height stays the content height");
        assert_eq!(group_size.y, 36.0 + 20.0, "zero gap between members");
    }

    #[test]
    fn vertical_group_horizontal_separator_spans_group_width() {
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            ButtonGroup::new()
                .orientation(GroupOrientation::Vertical)
                .show(ui, |ui| {
                    vec![
                        Button::new("Wide label here").show(ui).rect.size(),
                        ButtonGroupSeparator::new()
                            .orientation(GroupOrientation::Vertical)
                            .show(ui)
                            .rect
                            .size(),
                        Button::new("B").show(ui).rect.size(),
                    ]
                })
                .inner
        });
        let sizes = &frames[0];
        assert_eq!(sizes[1], Vec2::new(sizes[0].x, 1.0));
    }

    #[test]
    fn icon_button_keeps_fixed_width_in_vertical_group() {
        // Spec §5: a definite cross size wins over stretch.
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            let group = ButtonGroup::new()
                .orientation(GroupOrientation::Vertical)
                .show(ui, |ui| {
                    vec![
                        ButtonGroupText::new("a wide text slot")
                            .show(ui)
                            .rect
                            .size(),
                        Button::new("")
                            .size(ButtonSize::Icon)
                            .icon_start(Icon::Copy)
                            .show(ui)
                            .rect
                            .size(),
                    ]
                });
            (group.inner, group.response.rect.size())
        });
        let (sizes, group_size) = &frames[0];
        assert_eq!(sizes[1], Vec2::new(36.0, 36.0));
        assert!(group_size.x > 36.0, "group is wider than the icon button");
        assert_eq!(sizes[0].x, group_size.x, "Text stretches to group width");
    }

    #[test]
    fn nested_group_gets_eight_px_gap_automatically() {
        // Spec §8: no manual `.gap()` — detection is automatic, same-frame.
        let frames = run_frames(Style::Vega, Mode::Light, 2, |ui| {
            let group = ButtonGroup::new().show(ui, |ui| {
                let nested = ButtonGroup::new().show(ui, |ui| {
                    vec![
                        Button::new("Left").show(ui).rect.width(),
                        Button::new("Right").show(ui).rect.width(),
                    ]
                });
                let mut widths = nested.inner;
                widths.push(nested.response.rect.width());
                widths.push(Button::new("Submit").show(ui).rect.width());
                widths
            });
            (group.inner, group.response.rect.width())
        });
        for (widths, group_w) in &frames {
            assert_eq!(widths[2], widths[0] + widths[1], "nested group: zero gap");
            assert_eq!(
                *group_w,
                widths[2] + 8.0 + widths[3],
                "outer group: 8px gap"
            );
        }
    }

    #[test]
    fn nested_vertical_group_gets_eight_px_gap() {
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            let group = ButtonGroup::new()
                .orientation(GroupOrientation::Vertical)
                .show(ui, |ui| {
                    let nested = ButtonGroup::new()
                        .orientation(GroupOrientation::Vertical)
                        .show(ui, |ui| {
                            vec![
                                Button::new("Top").show(ui).rect.height(),
                                Button::new("Bottom").show(ui).rect.height(),
                            ]
                        });
                    let mut heights = nested.inner;
                    heights.push(nested.response.rect.height());
                    heights.push(Button::new("Submit").show(ui).rect.height());
                    heights
                });
            (group.inner, group.response.rect.height())
        });
        let (heights, group_h) = &frames[0];
        assert_eq!(heights[2], heights[0] + heights[1]);
        assert_eq!(*group_h, heights[2] + 8.0 + heights[3]);
    }

    #[test]
    fn plain_group_has_zero_gap() {
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            let group = ButtonGroup::new().show(ui, |ui| {
                vec![
                    Button::new("Day").show(ui).rect.width(),
                    Button::new("Month").show(ui).rect.width(),
                    Button::new("Year").show(ui).rect.width(),
                ]
            });
            (group.inner, group.response.rect.width())
        });
        let (widths, group_w) = &frames[0];
        assert_eq!(*group_w, widths.iter().sum::<f32>());
    }

    #[test]
    fn gap_override_wins_over_automatic_rule() {
        // Explicit `.gap(0)` suppresses the nested gap.
        let no_gap = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            let group = ButtonGroup::new().gap(0.0).show(ui, |ui| {
                let nested = ButtonGroup::new().show(ui, |ui| {
                    Button::new("L").show(ui);
                });
                vec![
                    nested.response.rect.width(),
                    Button::new("S").show(ui).rect.width(),
                ]
            });
            (group.inner, group.response.rect.width())
        });
        let (widths, group_w) = &no_gap[0];
        assert_eq!(*group_w, widths[0] + widths[1]);

        // Explicit `.gap(8)` adds a gap without nesting.
        let forced_gap = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            let group = ButtonGroup::new().gap(8.0).show(ui, |ui| {
                vec![
                    Button::new("A").show(ui).rect.width(),
                    Button::new("B").show(ui).rect.width(),
                ]
            });
            (group.inner, group.response.rect.width())
        });
        let (widths, group_w) = &forced_gap[0];
        assert_eq!(*group_w, widths[0] + 8.0 + widths[1]);
    }

    #[test]
    fn empty_group_is_zero_size() {
        let frames = run_frames(Style::Vega, Mode::Light, 2, |ui| {
            ButtonGroup::new().show(ui, |_| {}).response.rect.size()
        });
        for size in &frames {
            assert_eq!(*size, Vec2::ZERO);
        }
    }

    #[test]
    fn single_member_group_fits_member() {
        let frames = run_frames(Style::Vega, Mode::Light, 2, |ui| {
            let group = ButtonGroup::new().show(ui, |ui| Button::new("Only").show(ui).rect.size());
            (group.inner, group.response.rect.size())
        });
        for (member, group) in &frames {
            assert_eq!(*member, *group);
        }
    }

    #[test]
    fn standalone_text_is_content_sized() {
        // Spec §7.1: standalone height = 20px content height.
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            ButtonGroupText::new("https://example.com")
                .show(ui)
                .rect
                .size()
        });
        assert_eq!(frames[0].y, 20.0);
        assert!(frames[0].x > 2.0 * spacing::LG);
    }

    #[test]
    fn text_padding_differs_by_recipe() {
        // Same text, same font: nova px-2.5 vs base px-4 → exactly 12px
        // narrower (2 × (16 − 10)).
        let base = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            ButtonGroupText::new("padding probe").show(ui).rect.size()
        })[0];
        let nova = run_frames(Style::Nova, Mode::Light, 1, |ui| {
            ButtonGroupText::new("padding probe").show(ui).rect.size()
        })[0];
        assert_eq!(base.y, 20.0);
        assert_eq!(nova.y, 20.0);
        assert!((base.x - nova.x - 12.0).abs() < 1e-4);
    }

    #[test]
    fn standalone_separator_spans_available_extent() {
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            ButtonGroupSeparator::new().show(ui).rect.size()
        });
        assert_eq!(frames[0], Vec2::new(1.0, 400.0));
    }

    #[test]
    fn content_closure_runs_more_than_once() {
        // The measure-then-real contract: the closure runs at least twice
        // (measure + real), so stretch and gap detection are same-frame.
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            let mut runs = 0;
            ButtonGroup::new().show(ui, |ui| {
                runs += 1;
                Button::new("x").show(ui);
            });
            runs
        });
        assert!(frames[0] >= 2);
    }

    #[test]
    fn vertical_group_text_button_stretches_to_widest_member() {
        // FG-1 end to end (spec §5): in a vertical group a text button
        // stretches to the widest member's width through the shared
        // stretch channel, while an icon button keeps its fixed size —
        // both through the real `ButtonGroup::show`, not the raw plumbing.
        let frames = run_frames(Style::Vega, Mode::Light, 1, |ui| {
            let group = ButtonGroup::new()
                .orientation(GroupOrientation::Vertical)
                .show(ui, |ui| {
                    vec![
                        ButtonGroupText::new("a wide text slot")
                            .show(ui)
                            .rect
                            .size(),
                        Button::new("B").show(ui).rect.size(),
                        Button::new("")
                            .size(ButtonSize::Icon)
                            .icon_start(Icon::Copy)
                            .show(ui)
                            .rect
                            .size(),
                    ]
                });
            (group.inner, group.response.rect.size())
        });
        let (sizes, group_size) = &frames[0];
        assert_eq!(sizes[0].x, group_size.x, "Text stretches to group width");
        assert_eq!(
            sizes[1].x, group_size.x,
            "text button stretches to the widest member"
        );
        assert_eq!(sizes[1].y, 36.0, "text button keeps its fixed height");
        assert_eq!(
            sizes[2],
            Vec2::new(36.0, 36.0),
            "icon button keeps its fixed size"
        );
        assert!(group_size.x > 36.0, "group is wider than the icon button");
    }

    #[test]
    fn nova_trailing_non_slot_child_button_keeps_lg_trailing_corners() {
        // FG-2 end to end (spec §3.2): [Button, Text] under nova. The
        // button is the last SLOT child (Text carries no `data-slot`), so
        // its trailing corners are forced to `rounded-lg!` even though it
        // is not positionally last. Without the transfer the button would
        // render fused trailing corners.
        let (out, button_rect) = run_frames_output(Style::Nova, Mode::Light, 2, |ui| {
            ButtonGroup::new()
                .show(ui, |ui| {
                    let r = Button::new("A").show(ui);
                    ButtonGroupText::new("text").show(ui);
                    r.rect
                })
                .inner
        });
        let strokes: Vec<(Rect, CornerRadius)> = border_strokes(&out)
            .into_iter()
            .filter(|(rect, _)| same_rect(*rect, button_rect))
            .collect();
        assert_eq!(strokes.len(), 1, "one border stroke on the button rect");
        let (_, rounding) = strokes[0];
        assert_eq!(
            (rounding.nw, rounding.ne, rounding.sw, rounding.se),
            (10, 10, 10, 10),
            "trailing corners forced to rounded-lg by the last-slot rule"
        );
    }

    #[test]
    fn nova_plain_last_child_button_corners_unchanged() {
        // FG-2: the common case — [Button, Button] under nova. The last
        // button IS the last slot child, so the flag changes nothing: its
        // trailing corners are its own `rounded-lg` either way, and the
        // first button gets no flag.
        let (out, rects) = run_frames_output(Style::Nova, Mode::Light, 2, |ui| {
            ButtonGroup::new()
                .show(ui, |ui| {
                    vec![
                        Button::new("A").show(ui).rect,
                        Button::new("B").show(ui).rect,
                    ]
                })
                .inner
        });
        let strokes = border_strokes(&out);
        let rounding_at = |target: Rect| {
            strokes
                .iter()
                .find(|(rect, _)| same_rect(*rect, target))
                .map(|(_, c)| *c)
                .expect("a border stroke on the button draw rect")
        };
        let first = rounding_at(draw_rect_for(rects[0], GroupPosition::First));
        assert_eq!(
            (first.nw, first.ne, first.sw, first.se),
            (10, 0, 10, 0),
            "first button: fused trailing corners, no flag"
        );
        let last = rounding_at(draw_rect_for(rects[1], GroupPosition::Last));
        assert_eq!(
            (last.nw, last.ne, last.sw, last.se),
            (0, 10, 0, 10),
            "last button: fused leading corners, own trailing radius"
        );
    }

    #[test]
    fn base_recipe_ignores_last_slot_flag() {
        // FG-2: the same [Button, Text] composition under the base recipe.
        // The flag is computed but the base recipe ignores it, so the
        // button's trailing corners stay fused (radius 8 own, 0 fused).
        let (out, button_rect) = run_frames_output(Style::Vega, Mode::Light, 2, |ui| {
            ButtonGroup::new()
                .show(ui, |ui| {
                    let r = Button::new("A").show(ui);
                    ButtonGroupText::new("text").show(ui);
                    r.rect
                })
                .inner
        });
        let fills: Vec<(Rect, CornerRadius)> = fill_shapes(&out)
            .into_iter()
            .filter(|(rect, _)| same_rect(*rect, button_rect))
            .collect();
        assert_eq!(fills.len(), 1, "one fill shape on the button rect");
        let (_, rounding) = fills[0];
        assert_eq!(
            (rounding.nw, rounding.ne, rounding.sw, rounding.se),
            (8, 0, 8, 0),
            "base recipe ignores the last-slot flag: fusing stands"
        );
    }

    #[test]
    fn nested_group_registers_slot_identity() {
        // FG-2: [Button, NestedGroup, Text] under nova. The nested group
        // is the last slot-carrying child, so the first button must NOT
        // receive the rule — its trailing corners stay fused. Without the
        // nested registration the button would be flagged.
        let (out, first_rect) = run_frames_output(Style::Nova, Mode::Light, 2, |ui| {
            ButtonGroup::new()
                .show(ui, |ui| {
                    let r = Button::new("A").show(ui);
                    ButtonGroup::new().show(ui, |ui| {
                        Button::new("In").show(ui);
                    });
                    ButtonGroupText::new("text").show(ui);
                    r.rect
                })
                .inner
        });
        let strokes: Vec<(Rect, CornerRadius)> = border_strokes(&out)
            .into_iter()
            .filter(|(rect, _)| same_rect(*rect, first_rect))
            .collect();
        assert_eq!(strokes.len(), 1, "one border stroke on the button rect");
        let (_, rounding) = strokes[0];
        assert_eq!(
            (rounding.nw, rounding.ne, rounding.sw, rounding.se),
            (10, 0, 10, 0),
            "first button is not the last slot child: fusing stands"
        );
    }

    #[test]
    fn group_focus_ring_paints_above_following_sibling() {
        // FG-3 end to end (spec §4): through the real `ButtonGroup::show`,
        // a focused middle member's ring paints above the following
        // sibling — the deferred flush in `group::end`, not inline paint.
        let ctx = egui::Context::default();
        Preset {
            style: Style::Nova,
            ..Preset::default()
        }
        .build(Mode::Light)
        .install(&ctx);
        let mut middle_id = egui::Id::NULL;
        let mut third_rect = Rect::NOTHING;

        let show_group = |ui: &mut Ui| {
            ButtonGroup::new()
                .id_salt("s1b_fg3")
                .show(ui, |ui| {
                    let _ = Button::new("One").id_salt("s1b_fg3_b1").show(ui);
                    let mid = Button::new("Two").id_salt("s1b_fg3_b2").show(ui);
                    let third = Button::new("Three").id_salt("s1b_fg3_b3").show(ui);
                    (mid.id, third.rect)
                })
                .inner
        };

        // Frame 1: render, record the middle button's id and the third
        // button's rect.
        let mut out = ctx.run_ui(input_at(0.0), |ctx| {
            let ui = &mut fg_test_ui(ctx, "s1b_fg3_ui");
            let (id, rect) = show_group(ui);
            middle_id = id;
            third_rect = rect;
        });
        out.textures_delta.clear();

        // Focus the middle button, then run two more frames with a
        // controlled clock so the 150ms ring fade completes.
        ctx.memory_mut(|m| m.request_focus(middle_id));
        for time in [0.05, 0.30] {
            out = ctx.run_ui(input_at(time), |ctx| {
                let ui = &mut fg_test_ui(ctx, "s1b_fg3_ui");
                show_group(ui);
            });
            out.textures_delta.clear();
        }

        // The ring: a 3px outside stroke. It must exist (the queue
        // flushed) and come after every shape the third button painted.
        let mut ring_idx = None;
        let mut sibling_max_idx = 0usize;
        let mut sibling_count = 0usize;
        for (i, clipped) in out.shapes.iter().enumerate() {
            match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.stroke.width == 3.0
                        && rs.stroke_kind == egui::epaint::StrokeKind::Outside =>
                {
                    ring_idx = Some(i);
                }
                egui::Shape::Rect(rs) if rs.rect.intersects(third_rect) => {
                    sibling_max_idx = sibling_max_idx.max(i);
                    sibling_count += 1;
                }
                egui::Shape::Text(ts) if third_rect.contains(ts.pos) => {
                    sibling_max_idx = sibling_max_idx.max(i);
                    sibling_count += 1;
                }
                _ => {}
            }
        }
        let ring_idx = ring_idx.expect("a deferred focus ring must be painted");
        assert!(
            sibling_count > 0,
            "the following sibling must paint shapes to compare against"
        );
        assert!(
            ring_idx > sibling_max_idx,
            "ring at shape {ring_idx} must paint after the following sibling's \
             last shape at {sibling_max_idx}"
        );
    }
}
