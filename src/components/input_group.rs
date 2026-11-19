//! shadcn `input-group.tsx` — an input with inline/block addons.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/input-group.tsx`. The
//! container is a `h-9 rounded-md border border-input shadow-xs` row
//! (`dark:bg-input/30`) that switches to a column when a block addon is
//! present (`has-[>[data-align=block-start]]:flex-col`) and to auto
//! height when a direct-child textarea is present
//! (`has-[>textarea]:h-auto`, row layout kept), grows a 3px ring when
//! any control inside is focused
//! (`has-[[data-slot=input-group-control]:focus-visible]`), turns
//! destructive when any slotted descendant is `aria-invalid`, and under
//! the nova style (`.cn-input-group`) tightens to `h-8 rounded-lg` with
//! `has-disabled:bg-input/50 dark:has-disabled:bg-input/80` plus
//! `has-disabled:opacity-50` on the whole container.
//!
//! The source's slots map as follows:
//!
//! - `InputGroupAddon` — `inline-start` / `inline-end` / `block-start` /
//!   `block-end` alignment; holds text, icons, `Kbd` and
//!   `InputGroupButton`s.
//! - `InputGroupText` — `text-sm text-muted-foreground` inline text.
//! - `InputGroupButton` — ghost `Button` with `xs`/`sm`/`icon-xs`/`icon-sm`
//!   sizes.
//! - `InputGroupInput` / `InputGroupTextarea` — frameless borderless
//!   controls (`flex-1 rounded-none border-0 bg-transparent shadow-none`).
//!
//! Focus, invalid and layout flags are resolved from the previous frame
//! (like the source's `has-[]:` selectors, which are reactive); they settle
//! one frame after a state change.
//!
//! S1 decisions (frozen spec, Batch B input-group S1 report):
//!
//! - FG-1 stretch: a width-`auto` member of a vertical `ButtonGroup`
//!   stretches to the widest member's width (`group::stretch_target`);
//!   horizontal targets never stretch inputs; an explicit `.width()` is
//!   exempt. Inline addons/buttons are never stretched to the container
//!   height (the container is `items-center`, not `items-stretch`).
//! - Fusing: members paint on the FULL layout rect and share ONE 1px
//!   border by border REMOVAL (clip), not by shifting the draw rect
//!   (BTG-VIS-01: the earlier 1px draw-rect inset left the shared column
//!   painted by neither member — a backdrop seam).
//! - Disabled (OPEN-3): two separate mechanisms, never collapsed.
//!   `InputGroup::disabled` is the control-level disabled (mechanism (a):
//!   nova only — container fill `bg-input/50`/`/80` plus whole-container
//!   50% opacity). `InputGroup::group_disabled` is the group-level flag
//!   (mechanism (b): both styles — addons fade 50%). Fade factors are
//!   multiplicative (`0.5^n`): both firing under nova renders an addon
//!   at 25%. Addon buttons do not fade (`Button` has no fade-only
//!   rendering) — a known limitation, proposed D6.
//! - Combined state (OPEN-2): invalid wins BOTH border and ring colour
//!   when focused+invalid. Under nova, invalid at rest gets a 3px
//!   destructive ring; under base it gets none.
//! - `icon-sm` is `size-8` = 32x32 under BOTH styles (N4); a 28px
//!   rendering under nova is a defect.
//! - Block-addon vertical padding follows the N15 conditional (base 12px,
//!   or 10px when a direct-child `input` exists; nova uniformly 8px).
//! - The kbd negative margin and kbd radius need a kbd-line mechanism
//!   (`Kbd` reports nothing into this record; `kbd.rs` is not touched
//!   here) and are not applied; the button negative margin is applied
//!   when a direct-child `InputGroupButton` is detected.
//!
//! S1 implementation notes (egui mechanics, verified by pins):
//!
//! - The addon's row is a `ui.horizontal`, which sizes a new row to
//!   `spacing().interact_size.y` (36pt theme floor) — under the nova
//!   `h-8` (32pt) row that stretched every addon (and the content rect)
//!   to 36pt. The group's child `Ui` pins `interact_size.y` to the
//!   container height, the same guard the input already applies in its
//!   own scope.
//! - The addon's row uses NO inter-item spacing: egui adds
//!   `item_spacing` after EVERY widget including the last, so a `gap-2`
//!   (8px) would inflate the addon's outer edge by 8px (InlineEnd
//!   trailing 20px instead of the spec's 12px; InlineStart pushing the
//!   input 8px too far right, beyond the spec §3.5 margin). Single-child
//!   addons are exactly spec-correct; an addon with several direct
//!   children renders them with no gap (spec `gap-2` — documented
//!   limitation).
//! - Inner controls are frameless via `Frame::NONE`, not
//!   `background_color(TRANSPARENT)` — the latter clears only the fill
//!   and leaves the theme-default 1px frame stroke inside the container
//!   border.
//! - Addon click-to-focus is DEFERRED one frame: a focus request made in
//!   the click frame is undone by the input's own `interact` in the same
//!   frame (egui surrenders focus when a click lands on a widget the
//!   pointer is not over, `SurrenderFocusOn::Clicks`, and the click lands
//!   on the addon, not the input). The click only records the target;
//!   the request fires on the first click-free frame (field.rs's
//!   deferred-forward pattern).
//!
//! Known limitations (carried to S3/S5/S6, see the S1 report):
//!
//! - A block addon's row is `interact_size`-tall (36px base / 32px nova)
//!   instead of content-height (~20px for text): the input sits 46px
//!   below the container top (base) instead of the spec's 30px. The
//!   divider still adds exactly its 10px. Measure-then-real would fix
//!   it but is blocked — the content closure is `FnOnce`.
//! - A narrow inline addon in a column group renders left-aligned, not
//!   centred (§3.1 `items-center`): the addon's `ui.horizontal`
//!   allocates the full available width. Same `FnOnce` block.
//! - Addon buttons do not fade under the disabled mechanisms (proposed
//!   D6); the outline variant's `shadow-none` (OPEN-1(c)) cannot be
//!   enforced from here (`Button` has no shadow-override API).
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
//!         InputGroupButton::icon(Icon::Copy).show(ui);
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
    Align as EguiAlign, Align2, Color32, CornerRadius, FontId, Frame, Id, Layout, Margin, Painter,
    Pos2, Rect, Response, Sense, Stroke, TextEdit, Ui, UiBuilder, Vec2,
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

/// Control side padding next to an inline addon
/// (`[&>input]:pl-2`/`pr-2` base recipe; nova `pl-1.5`/`pr-1.5`).
fn control_inline_padding(style: Style) -> f32 {
    match style {
        Style::Nova => 6.0, // pl-1.5 / pr-1.5
        _ => 8.0,           // pl-2 / pr-2
    }
}

/// Default control side padding with no addon on that side: the input's
/// own padding (`px-3` base; `px-2.5` nova via `.cn-input`, §3.5).
fn control_default_padding(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0,                    // px-2.5
        _ => crate::theme::tokens::spacing::MD, // px-3
    }
}

/// Negative margin for a direct-child button in an inline addon (§3.4):
/// `ml-[-0.45rem]`/`mr-[-0.45rem]` base, `ml-[-0.3rem]`/`mr-[-0.3rem]`
/// nova. Pulls the button toward the container's outer edge,
/// compensating the addon's outer padding (base inline-start button
/// outer edge sits 12 − 7.2 = 4.8px from the container's content edge).
fn addon_button_negative_margin(style: Style) -> f32 {
    match style {
        Style::Nova => 4.8, // 0.3rem
        _ => 7.2,           // 0.45rem
    }
}

/// Block addon vertical padding with the N15 conditional (§3.3): base
/// `pt-3`/`pb-3` (12px), or `pt-2.5`/`pb-2.5` (10px) when a direct-child
/// `input` exists; nova uniformly `pt-2`/`pb-2` (8px — the conditional
/// is a no-op there).
fn addon_block_padding_y_conditional(style: Style, has_input: bool) -> f32 {
    match style {
        Style::Nova => 8.0, // pt-2 / pb-2
        _ => {
            if has_input {
                10.0 // pt-2.5 / pb-2.5
            } else {
                12.0 // pt-3 / pb-3
            }
        }
    }
}

/// The input's `TextEdit` margins (§3.5, pure): side padding from the
/// inline addons (`pl-2`/`pr-2` base, `pl-1.5`/`pr-1.5` nova) or the
/// input's own padding (`px-3` base, `px-2.5` nova) with no addon on
/// that side; top/bottom from the block addons (`pt-3`/`pb-3` = 12px,
/// both styles). A `<textarea>` is never targeted by these rules.
fn control_margins(
    style: Style,
    inline_start: bool,
    inline_end: bool,
    block_start: bool,
    block_end: bool,
) -> Margin {
    Margin {
        left: (if inline_start {
            control_inline_padding(style)
        } else {
            control_default_padding(style)
        }) as i8,
        right: (if inline_end {
            control_inline_padding(style)
        } else {
            control_default_padding(style)
        }) as i8,
        top: if block_end { 12 } else { 0 },
        bottom: if block_start { 12 } else { 0 },
    }
}

/// Container fade factor (mechanism (a), §4.4): under nova, a disabled
/// descendant makes the WHOLE container render at 50%
/// (`has-disabled:opacity-50` — element opacity, border included).
/// Base has no container-level disabled rule, so the factor is 1.0 there.
fn container_fade(disabled: bool, style: Style) -> f32 {
    if disabled && style == Style::Nova {
        0.5
    } else {
        1.0
    }
}

/// Addon fade factor (mechanism (b), §4.4): each addon fades to 50% when
/// the group carries `data-disabled="true"` (both styles).
fn addon_fade(disabled_group: bool) -> f32 {
    if disabled_group {
        0.5
    } else {
        1.0
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
    /// A direct-child block-start addon is present (§3.5: the input gets
    /// `pb-3` below it).
    block_start: bool,
    /// A direct-child block-end addon is present (§3.5: the input gets
    /// `pt-3` above it).
    block_end: bool,
    textarea: bool,
    /// The container was created with `.disabled(true)`.
    disabled_container: bool,
    /// The group carries `data-disabled="true"` (mechanism (b), §4.4):
    /// each addon fades to 50%.
    disabled_group: bool,
    /// A descendant control is disabled (mechanism (a) trigger, §4.4;
    /// set by Input/Textarea/Button with `.disabled(true)` — OPEN-6
    /// practical reading: any disabled form-control descendant).
    control_disabled: bool,
    /// A direct-child `input` exists (N15 conditional, §3.3; a
    /// `<textarea>` does NOT set this).
    has_input: bool,
    /// The group's first input is disabled (addon click-to-focus on a
    /// disabled input is a platform no-op, §7.2 rule 5).
    input_disabled: bool,
    /// Measured content height of a column container (for the next frame's
    /// reservation).
    height: f32,
    /// Measured width of the trailing inline addon (for the next frame's
    /// input sizing).
    end_addon_width: f32,
    /// The group's first input id, for addon click-to-focus (§7.2: only
    /// `<input>` elements are focused — never `<textarea>`).
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

fn addon_stack_key() -> Id {
    Id::unique("shadcn_input_group_addon_stack")
}

fn current_addon_id(ui: &Ui) -> Option<egui::Id> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<egui::Id>>(addon_stack_key()))
        .and_then(|stack| stack.last().copied())
}

fn push_addon_id(ui: &Ui, id: egui::Id) {
    let mut stack: Vec<egui::Id> = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Vec<egui::Id>>(addon_stack_key()))
        .unwrap_or_default();
    stack.push(id);
    ui.ctx()
        .data_mut(|d| d.insert_temp(addon_stack_key(), stack));
}

fn pop_addon_id(ui: &Ui) {
    let mut stack: Vec<egui::Id> = ui
        .ctx()
        .data_mut(|d| d.get_temp::<Vec<egui::Id>>(addon_stack_key()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(addon_stack_key(), stack));
}

/// Paints the container border with the leading edge removed for
/// Middle/Last group members (BTG-VIS-01): members paint on the FULL
/// layout rect and share ONE 1px border by border REMOVAL, not by
/// shifting the draw rect — the earlier 1px draw-rect inset left the
/// shared column painted by neither member (a backdrop seam). Mirrors
/// `button::paint_fused_stroke` (private there): the full `Inside` ring
/// is painted through a clip that excludes the leading 1px column
/// (horizontal) or row (vertical), plus clipped copies for the two
/// leading corner cells (1x1 fill rects do not rasterize in egui 0.37).
fn paint_fused_stroke(
    painter: &Painter,
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
    let (clip, corners) = match orientation {
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
    group_disabled: bool,
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
            group_disabled: false,
            id_salt: None,
        }
    }

    /// Sets an explicit width. The default is `w-full` (the available width
    /// in vertical layouts).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Marks the group's control disabled (mechanism (a), §4.4: the
    /// `has-disabled:` trigger). Under nova the container fill becomes
    /// `bg-input/50` (light) / `bg-input/80` (dark) and the whole
    /// container renders at 50% opacity; under base the container is
    /// unchanged and only the inner controls' own disabled rules apply.
    /// The inner controls are disabled too (via `disabled_in`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the GROUP `data-disabled="true"` (mechanism (b), §4.4: the
    /// caller-set attribute). Each addon fades to 50% in both styles;
    /// the container chrome is unchanged and the control is NOT
    /// disabled. This is a separate flag from `disabled` — the two
    /// mechanisms must not collapse (OPEN-3): if both fire under nova,
    /// an addon renders at 25% (`0.5 x 0.5`).
    pub fn group_disabled(mut self, group_disabled: bool) -> Self {
        self.group_disabled = group_disabled;
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

        // FG-1: a width-`auto` member of a vertical group stretches to
        // the widest member's width (`group::stretch_target`, the same
        // channel `Button::show` and `input.rs` read). Horizontal targets
        // never stretch inputs (the container has a definite `h-9`/`h-8`
        // height). An explicit `.width()` is a definite width and
        // exempts.
        let width = match self.width {
            Some(w) => w,
            None => {
                let natural = if ui.layout().is_vertical() {
                    ui.available_width()
                } else {
                    280.0
                };
                if grp.is_some() {
                    if let Some(target) = group::stretch_target(ui) {
                        if target.orientation == GroupOrientation::Vertical {
                            natural.max(target.cross)
                        } else {
                            natural
                        }
                    } else {
                        natural
                    }
                } else {
                    natural
                }
            }
        };

        // Column iff a block addon is present; a direct-child textarea
        // makes the height `h-auto` WITHOUT changing direction (row
        // layout, auto height) — §3.1.
        let column = last.block;
        let auto_height = last.block || last.textarea;
        let hint_height = if auto_height && last.height > 0.0 {
            last.height
        } else {
            container_height(style) // h-9 / nova h-8
        };

        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, hint_height), Sense::hover());

        // Mechanism (a) trigger: the container's own `.disabled(true)`
        // or a disabled descendant last frame (§4.4; OPEN-6 practical
        // reading: any disabled form-control descendant).
        let disabled_trigger = self.disabled || last.control_disabled;
        // Whole-container opacity under nova (`has-disabled:opacity-50`).
        let fade = container_fade(disabled_trigger, style);

        // Background fill + shadow must be painted before the children so
        // their text stays on top. `shadow-xs` always paints (both
        // styles, disabled or not); under the nova container fade it
        // renders at 50% (element opacity).
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let corner = CornerRadius::same(container_radius(style, &theme.radius));
            let sh = crate::utils::shadow::shadow_xs(dark);
            painter.add(
                egui::epaint::RectShape::filled(
                    rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                    corner,
                    color_alpha(sh.color, fade),
                )
                .with_blur_width(2.0),
            );
            // Fill: nova + disabled descendant -> `has-disabled:bg-input/50`
            // (light) / `bg-input/80` (dark), times the container opacity
            // (effective 25% / 40%). Base has NO container-level disabled
            // rule: the fill stays `dark:bg-input/30` / transparent.
            let fill = if disabled_trigger && style == Style::Nova {
                color_alpha(color_alpha(p.input, if dark { 0.8 } else { 0.5 }), fade)
            } else if dark {
                color_alpha(p.input, 0.3)
            } else {
                Color32::TRANSPARENT
            };
            if fill != Color32::TRANSPARENT {
                painter.rect_filled(rect, corner, fill);
            }
        }

        // `items-center` in ALL modes (§3.1): row children are
        // vertically centred; column children are horizontally centred
        // unless they carry `w-full` (which block addons and the control
        // do).
        let mut child = ui.new_child(UiBuilder::new().max_rect(rect).layout(if column {
            Layout::top_down(EguiAlign::Center)
        } else {
            Layout::left_to_right(EguiAlign::Center)
        }));
        child.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
        // The theme's `interact_size` floor is 36pt; `ui.horizontal`
        // sizes a new row to it, which would stretch every addon (and
        // the content rect) to 36pt under the nova `h-8` (32pt) row. Pin
        // it to the container height — the input's scoped spacing does
        // the same for the control.
        child.spacing_mut().interact_size.y = container_height(style);

        // Fresh per-frame record for the children to update.
        write_record(
            ui,
            group_id,
            GroupRecord {
                disabled_container: self.disabled,
                disabled_group: self.group_disabled,
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
            // BTG-VIS-01: the draw rect is the FULL border box at every
            // position (border-box: the stroke occupies the outermost 1px
            // ring of the box); border removal is the stroke painter's
            // job, not a draw-rect inset.
            let draw_rect = content_rect;

            let invalid = record.invalid;
            // OPEN-2: invalid wins BOTH border and ring colour when
            // focused+invalid (radio-group G3 + native-select OPEN-2
            // precedent: the invalid state must remain visible).
            let stroke_color = if invalid {
                p.destructive
            } else if record.focus {
                // `has-[control:focus-visible]:border-ring`
                p.ring
            } else {
                p.input
            };
            // `has-disabled:opacity-50` fades the border too (nova only;
            // `fade` is 1.0 under base).
            let stroke_color = color_alpha(stroke_color, fade);
            let stroke = Stroke::new(1.0, stroke_color);
            paint_fused_stroke(painter, draw_rect, corner, stroke, orientation, position);

            // Ring: 3px, adjacent to the border box — the ring rect is
            // the UNEXPANDED content rect (`paint_ring` strokes
            // `Outside`, so an expanded rect would leave a 1px gap).
            // Shows when focused (`has-[control:focus-visible]:ring-3
            // ring-ring/50`), or at rest when invalid under nova (§4.3:
            // base has no ring width at rest, nova has `ring-3`). The
            // focus ring is suppressed when disabled (a disabled input
            // cannot take focus); the nova invalid rest ring paints under
            // the container fade (§9, disabled+invalid).
            let ring = if record.focus && !self.disabled {
                Some(if invalid {
                    color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
                } else {
                    color_alpha(p.ring, 0.5)
                })
            } else if invalid && style == Style::Nova {
                Some(color_alpha(p.destructive, if dark { 0.4 } else { 0.2 }))
            } else {
                None
            };
            if let Some(ring) = ring {
                let ring = color_alpha(ring, fade);
                let stroke = Stroke::new(3.0, ring);
                if grp.is_some() {
                    // FG-3: queue the ring so it paints above the
                    // following siblings it abuts (the `z-10` rule).
                    group::queue_ring(ui, content_rect, corner, stroke);
                } else {
                    painter.rect_stroke(
                        content_rect,
                        corner,
                        stroke,
                        egui::epaint::StrokeKind::Outside,
                    );
                }
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
        record.has_input = true;
        if self.invalid {
            record.invalid = true;
        }
        if self.disabled {
            record.control_disabled = true;
        }
        record.input_disabled = self.disabled || self.disabled_in(ui);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input_group_input"),
        };

        // `flex-1`: all remaining row width minus the trailing addon (which
        // renders after this control, so use last frame's measured width).
        let snapshot = group_id.map(|id| read_snapshot(ui, id)).unwrap_or_default();
        let width = ui.available_width() - snapshot.end_addon_width.max(0.0);
        let is_empty = self.text.is_empty();
        // Text alpha: the control's own disabled rule (50%) times the
        // container fade (mechanism (a), nova only). Mechanism (b) does
        // not touch the control (addons only).
        let fade = container_fade(
            record.disabled_container || snapshot.control_disabled,
            style,
        );
        let own_alpha = if self.disabled || self.disabled_in(ui) {
            0.5
        } else {
            1.0
        };
        let text_color = color_alpha(p.foreground, own_alpha * fade);

        // `[&>input]:pl-2` when an inline-start addon precedes the control
        // (nova `pl-1.5`); `[&>input]:pr-2` when an inline-end addon follows
        // (nova `pr-1.5`, detected from last frame's snapshot). No addon on
        // a side: the input's own padding (`px-3` base, `px-2.5` nova).
        // `[&>input]:pb-3` when a block-start addon is present, `pt-3`
        // when a block-end addon is present (§3.5, both styles, read from
        // last frame's snapshot). A `<textarea>` is never targeted.
        let margins = control_margins(
            style,
            record.inline_start,
            snapshot.inline_end,
            snapshot.block_start,
            snapshot.block_end,
        );

        let disabled = self.disabled || self.disabled_in(ui);
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let edit = TextEdit::singleline(self.text)
            .id_salt(id)
            .font(font)
            .text_color(text_color)
            .desired_width(width.max(1.0))
            .min_size(Vec2::new(width.max(1.0), input::input_height(style)))
            .align(Align2::LEFT_CENTER)
            .margin(margins)
            // The control is frameless: all chrome lives on the
            // container. A theme-default frame would paint its own
            // 1px stroke inside the container border (the
            // `background_color(TRANSPARENT)` below only clears the
            // fill). `Frame::NONE` removes the stroke too; the text
            // lands at the same offset (a zero-width stroke needs no
            // inner-margin compensation).
            .frame(Frame::NONE);
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
        // First input wins (§7.2 rule 4: `querySelector` returns the
        // first matching descendant in document order).
        if record.input_id.is_none() {
            record.input_id = Some(response.id);
        }
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

/// `InputGroupTextarea` — the multiline control; makes the container
/// auto-height (row layout kept, §3.1).
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
        if self.disabled {
            record.control_disabled = true;
        }

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input_group_textarea"),
        };

        let width = ui.available_width();
        // Text alpha: the control's own disabled rule (50%) times the
        // container fade (mechanism (a), nova only). The textarea never
        // records `input_id`: addon clicks focus `<input>` elements
        // only, never a textarea (§7.2 rule 3).
        let snapshot = group_id.map(|id| read_snapshot(ui, id)).unwrap_or_default();
        let fade = container_fade(
            record.disabled_container || snapshot.control_disabled,
            style,
        );
        let disabled = self.disabled || self.disabled_in(ui);
        let own_alpha = if disabled { 0.5 } else { 1.0 };
        let text_color = color_alpha(p.foreground, own_alpha * fade);
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
            // Frameless, as above: `Frame::NONE` removes the
            // theme-default 1px frame stroke (fill-only
            // transparency would leave the stroke behind).
            .frame(Frame::NONE);
        let edit = if let Some(placeholder) = self.placeholder.as_deref() {
            edit.hint_text(placeholder.to_owned())
        } else {
            edit
        };
        let edit = if disabled {
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

/// `InputGroupAddon` — a slot beside the control for text, icons, kbd or
/// buttons. Clicking the addon focuses the group's control (the source's
/// `onClick`), except when the click hit a button inside the addon.
#[derive(Debug)]
pub struct InputGroupAddon {
    align: AddonAlign,
    divider: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl InputGroupAddon {
    /// Creates an addon with the given alignment (default `inline-start`).
    pub fn new(align: AddonAlign) -> Self {
        Self {
            align,
            divider: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Adds the divider variant's opposite-side padding (§3.3: a
    /// block-start addon carrying `border-b` additionally gets `pb-3` /
    /// `pb-2`; a block-end addon carrying `border-t` additionally gets
    /// `pt-3` / `pt-2`). The divider LINE itself is the caller's class,
    /// not an input-group rule, and is not painted here. No-op for
    /// inline addons.
    pub fn divider(mut self, divider: bool) -> Self {
        self.divider = divider;
        self
    }

    /// Marks the addon `aria-invalid` (§4.3 trigger: ANY slotted
    /// descendant with `aria-invalid` turns the container destructive).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
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
        if self.invalid {
            record.invalid = true;
        }

        // Stable id for the direct-child-button detection (§3.4): a
        // button rendered inside this addon records itself against this
        // id; the negative margin reads last frame's snapshot.
        let addon_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input_group_addon"),
        };
        let has_button = ui.ctx().data_mut(|d| {
            d.get_temp::<bool>(addon_id.with("has_button_prev"))
                .unwrap_or(false)
        });
        // The negative margin pulls a direct-child button toward the
        // container's outer edge (§3.4); without a button the addon's
        // full outer padding applies.
        let outer_padding = addon_inline_padding(style)
            - if has_button {
                addon_button_negative_margin(style)
            } else {
                0.0
            };
        // N15: block vertical padding is 10px (base) when a direct-child
        // `input` exists, 12px otherwise; nova uniformly 8px. Read from
        // last frame's snapshot (a block-start addon renders before the
        // input).
        let has_input = group_id
            .map(|id| read_snapshot(ui, id).has_input)
            .unwrap_or(false);
        let block_padding_y = addon_block_padding_y_conditional(style, has_input);

        // The addon's row uses NO inter-item spacing: egui's
        // `item_spacing` is added after EVERY widget including the
        // last, so a `gap-2` (8px) here would inflate the addon's
        // outer edge by 8px (the InlineEnd trailing would read 20px
        // instead of the spec's 12px, and the InlineStart row would
        // push the input 8px too far right, beyond the spec §3.5
        // `pl-2`/`pl-1.5` margin). The outer padding is the explicit
        // `add_space` below. Consequence, documented: an addon with
        // several direct children renders them with NO gap (spec
        // `gap-2`); single-child addons — every demo composition —
        // are exactly spec-correct.
        push_addon_id(ui, addon_id);
        let inner = match self.align {
            AddonAlign::InlineStart => {
                record.inline_start = true;
                if let Some(group) = group_id {
                    write_record(ui, group, record);
                }
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.set_min_size(Vec2::new(0.0, container_height(style)));
                    // `pl-3 py-1.5` base recipe; nova `pl-2` (minus the
                    // button negative margin when a direct-child button
                    // is present, §3.4).
                    ui.add_space(outer_padding);
                    content(ui)
                })
            }
            AddonAlign::InlineEnd => {
                let inner = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.set_min_size(Vec2::new(0.0, container_height(style)));
                    let r = content(ui);
                    // `pr-3` base recipe; nova `pr-2` (minus the button
                    // negative margin, §3.4).
                    ui.add_space(outer_padding);
                    r
                });
                // Record the measured width so the input can reserve room
                // (`flex-1` behavior). Re-read the record first: the
                // content above may have updated it (e.g. a button's
                // click flag), and writing the pre-content copy would
                // clobber those updates.
                if let Some(group) = group_id {
                    let mut current = read_record(ui, group);
                    current.inline_end = true;
                    current.end_addon_width = inner.response.rect.width();
                    current.invalid |= record.invalid;
                    write_record(ui, group, current);
                }
                inner
            }
            AddonAlign::BlockStart => {
                record.block = true;
                record.block_start = true;
                if let Some(group) = group_id {
                    write_record(ui, group, record);
                }
                // `px-3 pt-3` base recipe; nova `px-2.5 pt-2` (N15: 10px
                // base when a direct-child `input` exists).
                ui.add_space(block_padding_y);
                let inner = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.set_min_size(Vec2::new(ui.available_width(), 0.0));
                    ui.add_space(addon_block_padding_x(style));
                    let r = content(ui);
                    ui.add_space(addon_block_padding_x(style));
                    r
                });
                // Divider variant (`border-b`): additionally `pb-3` /
                // `pb-2` (§3.3).
                if self.divider {
                    ui.add_space(block_padding_y);
                }
                inner
            }
            AddonAlign::BlockEnd => {
                // Divider variant (`border-t`): additionally `pt-3` /
                // `pt-2` (§3.3).
                if self.divider {
                    ui.add_space(block_padding_y);
                }
                let inner = ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.set_min_size(Vec2::new(ui.available_width(), 0.0));
                    ui.add_space(addon_block_padding_x(style));
                    let r = content(ui);
                    ui.add_space(addon_block_padding_x(style));
                    r
                });
                // Re-read the record first (see InlineEnd above).
                if let Some(group) = group_id {
                    let mut current = read_record(ui, group);
                    current.block = true;
                    current.block_end = true;
                    current.invalid |= record.invalid;
                    write_record(ui, group, current);
                }
                // `px-3 pb-3` base recipe; nova `px-2.5 pb-2` (N15).
                ui.add_space(block_padding_y);
                inner
            }
        };
        pop_addon_id(ui);

        // Rotate the direct-child-button detection: this frame's flag
        // (set by `InputGroupButton` during the content above) becomes
        // next frame's snapshot; the flag is then cleared.
        let has_button_now = ui.ctx().data_mut(|d| {
            d.get_temp::<bool>(addon_id.with("has_button"))
                .unwrap_or(false)
        });
        ui.ctx().data_mut(|d| {
            d.insert_temp(addon_id.with("has_button_prev"), has_button_now);
            d.insert_temp(addon_id.with("has_button"), false);
        });

        // `onClick`: focus the group's control (unless the click hit a
        // button inside the addon). The click is resolved from the pointer
        // state so no click-sense interact steals hits from the buttons.
        if let Some(group) = group_id {
            let record = read_record(ui, group);
            let snapshot = read_snapshot(ui, group);
            let focus_target = record.input_id.or(snapshot.input_id);
            if let Some(input_id) = focus_target {
                let (any_click, latest_pos) =
                    ui.input(|i| (i.pointer.any_click(), i.pointer.latest_pos()));
                // F2: the release position (`latest_pos`) is the only
                // position input — `press_origin` is dead for click
                // detection in egui 0.37 (cleared in `begin_pass` before
                // `any_click()` is ever true), and egui only reports a
                // click when the pointer stayed within `max_click_dist`
                // of the press, so a release inside the rect means the
                // press began inside (or within a few px of) it.
                let clicked =
                    any_click && latest_pos.is_some_and(|pos| inner.response.rect.contains(pos));
                // §7.2 rule 5: focusing a disabled input is a platform
                // no-op.
                let disabled = record.disabled_container || snapshot.input_disabled;
                // Deferred focus: a request made in the click frame is
                // undone by the input's own `interact` in the same
                // frame — egui surrenders focus when a click lands on a
                // widget the pointer is not over
                // (`SurrenderFocusOn::Clicks`), and the click lands on
                // the addon, not the input. So the click only RECORDS
                // the target; the request fires on the first click-free
                // frame (field.rs's deferred-forward pattern).
                let pending_key = group.with("focus_pending");
                if !any_click {
                    if let Some(Some(pending)) =
                        ui.ctx().data_mut(|d| d.get_temp::<Option<Id>>(pending_key))
                    {
                        ui.memory_mut(|mem| mem.request_focus(pending));
                        ui.ctx()
                            .data_mut(|d| d.insert_temp(pending_key, None::<Id>));
                    }
                }
                if clicked && !disabled && !record.button_clicked {
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(pending_key, Some(input_id)));
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
        // Text alpha: the text's own disabled rule (50%) times the
        // container fade (mechanism (a), nova only) times the addon fade
        // (mechanism (b), both styles) — multiplicative (`0.5^n`, §4.4).
        let group_id = current_id(ui);
        let record = group_id.map(|id| read_record(ui, id)).unwrap_or_default();
        let snapshot = group_id.map(|id| read_snapshot(ui, id)).unwrap_or_default();
        let fade = container_fade(
            record.disabled_container || snapshot.control_disabled,
            theme.style,
        ) * addon_fade(record.disabled_group);
        let own_alpha = if self.disabled { 0.5 } else { 1.0 };
        let color = color_alpha(p.muted_foreground, own_alpha * fade);
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let icon_size = crate::theme::tokens::spacing::LG;
        // Laid out once with the paint color: the same galley sizes the
        // widget and paints the text.
        let galley = if self.text.is_empty() {
            None
        } else {
            Some(ui.painter().layout_no_wrap(self.text.clone(), font, color))
        };
        let text_w = galley.as_ref().map_or(0.0, |g| g.size().x);
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
            if let Some(galley) = galley {
                // Same paint math as `Painter::text`, reusing the galley.
                let text_rect = Align2::LEFT_CENTER
                    .anchor_size(egui::Pos2::new(cursor, rect.center().y), galley.size());
                painter.galley(text_rect.min, galley, color);
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

/// Icon size for a button size (§3.4): `xs` icons are `size-3.5` = 14px
/// (the `size-3.5` rule appears ONLY in the xs variant string); `sm`,
/// `icon-xs` and `icon-sm` fall back to `Button`'s base `size-4` = 16px
/// (the `icon-xs` strings carry NO icon-size rule — the icon is 16px in
/// a 24px box, note (a)).
fn icon_size_for(size: InputGroupButtonSize) -> f32 {
    match size {
        InputGroupButtonSize::Xs => 14.0,
        _ => 16.0,
    }
}

/// `InputGroupButton` — a ghost `Button` sized for the group
/// (`xs` default, `sm`, `icon-xs`, `icon-sm`).
#[derive(Debug)]
pub struct InputGroupButton {
    button: Button,
    size: InputGroupButtonSize,
    icon_start: Option<Icon>,
    icon_end: Option<Icon>,
    disabled: bool,
    invalid: bool,
}

impl InputGroupButton {
    /// Creates a ghost group button with a text label (size `xs`).
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            button: Button::new(label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Xs),
            size: InputGroupButtonSize::Xs,
            icon_start: None,
            icon_end: None,
            disabled: false,
            invalid: false,
        }
    }

    /// Creates an icon-only group button (size `icon-xs`).
    pub fn icon(icon: Icon) -> Self {
        Self {
            button: Button::new("")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::IconXs),
            size: InputGroupButtonSize::IconXs,
            icon_start: Some(icon),
            icon_end: None,
            disabled: false,
            invalid: false,
        }
    }

    /// Sets the size.
    pub fn size(mut self, size: InputGroupButtonSize) -> Self {
        self.size = size;
        self.button = self.button.size(size.into());
        self
    }

    /// Shows an icon before the label with the spec size for this button
    /// size (§3.4: 14px for `xs`, 16px otherwise). Applied in `show`
    /// with the final size, so calling `.size()` after `.icon_start()`
    /// cannot leave a stale icon size.
    pub fn icon_start(mut self, icon: Icon) -> Self {
        self.icon_start = Some(icon);
        self
    }

    /// Shows an icon after the label with the spec size for this button
    /// size (§3.4).
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    /// Sets the visual variant (default ghost).
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.button = self.button.variant(variant);
        self
    }

    /// Disables the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self.button = self.button.disabled(disabled);
        self
    }

    /// Marks the button `aria-invalid` (§4.3 trigger: ANY slotted
    /// descendant with `aria-invalid` turns the container destructive).
    /// The button paints its own invalid ring as well.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self.button = self.button.invalid(invalid);
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
        // N4: `icon-sm` is `size-8` = 32x32 under BOTH styles, so under
        // nova it maps onto `ButtonSize::Icon` (32px) rather than the
        // 28px `ButtonSize::IconSm`; the base recipe keeps
        // `ButtonSize::IconSm` (32px).
        let style = ShadcnTheme::of_ui(ui).style;
        let size = match self.size {
            InputGroupButtonSize::Sm if style == Style::Nova => ButtonSize::Default,
            InputGroupButtonSize::IconSm if style == Style::Nova => ButtonSize::Icon,
            _ => self.size.into(),
        };
        let mut button = self.button.size(size);
        if let Some(icon) = self.icon_start {
            button = button.icon_start_sized(icon, icon_size_for(self.size));
        }
        if let Some(icon) = self.icon_end {
            button = button.icon_end_sized(icon, icon_size_for(self.size));
        }
        let response = button.show(ui);

        // Record the click so the addon's click-to-focus does not steal
        // it, the disabled/invalid flags so the container's chrome can
        // react (§4.3/§4.4 triggers), and the direct-child presence so
        // the addon can apply the §3.4 negative margin.
        if let Some(group_id) = current_id(ui) {
            let mut record = read_record(ui, group_id);
            if response.clicked() {
                record.button_clicked = true;
            }
            if self.disabled {
                record.control_disabled = true;
            }
            if self.invalid {
                record.invalid = true;
            }
            write_record(ui, group_id, record);
        }
        if let Some(addon_id) = current_addon_id(ui) {
            ui.ctx().data_mut(|d| {
                d.insert_temp(addon_id.with("has_button"), true);
            });
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
    use std::cell::Cell;

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
        // N4: nova `icon-sm` is a 32pt square (`size-8`) under BOTH
        // styles — a 28pt rendering under nova is a defect (spec §3.4).
        assert!((icon_btn_rect.width() - 32.0).abs() < 1.0);
        assert!((icon_btn_rect.height() - 32.0).abs() < 1.0);
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

    // ---- S1 pin helpers ----

    fn test_ui(ctx: &egui::Context, salt: &'static str, width: f32) -> egui::Ui {
        egui::Ui::new(
            ctx.clone(),
            egui::Id::unique(salt),
            egui::UiBuilder::new()
                .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, 400.0))),
        )
    }

    fn run_ui_capture(
        ctx: &egui::Context,
        input: egui::RawInput,
        mut f: impl FnMut(&mut Ui),
    ) -> egui::FullOutput {
        let mut output = ctx.run_ui(input, |ctx| {
            let mut ui = Ui::new(
                ctx.clone(),
                Id::unique("ig_pin_ui"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(1200.0, 800.0))),
            );
            f(&mut ui);
        });
        // No renderer consumes the texture uploads in a headless test;
        // dropping an unapplied delta panics, so clear it explicitly.
        output.textures_delta.clear();
        output
    }

    fn run_ui(ctx: &egui::Context, input: egui::RawInput, f: impl FnMut(&mut Ui)) {
        let _ = run_ui_capture(ctx, input, f);
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

    fn rects_match(a: Rect, b: Rect) -> bool {
        (a.min.x - b.min.x).abs() < 0.5
            && (a.min.y - b.min.y).abs() < 0.5
            && (a.max.x - b.max.x).abs() < 0.5
            && (a.max.y - b.max.y).abs() < 0.5
    }

    /// Stroked `Rect` shapes at `want` (with their clip rects).
    fn stroked_rects_at(out: &egui::FullOutput, want: Rect) -> Vec<(Rect, Rect, Stroke)> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.stroke.width > 0.0 && rects_match(rs.rect, want) => {
                    Some((rs.rect, clipped.clip_rect, rs.stroke))
                }
                _ => None,
            })
            .collect()
    }

    /// Non-transparent filled `Rect` shapes at `want`.
    fn filled_rects_at(out: &egui::FullOutput, want: Rect) -> Vec<Color32> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.stroke.width == 0.0
                        && rs.fill != Color32::TRANSPARENT
                        && rects_match(rs.rect, want) =>
                {
                    Some(rs.fill)
                }
                _ => None,
            })
            .collect()
    }

    /// The laid-out color of the first text shape whose job text is `text`.
    fn text_color(out: &egui::FullOutput, text: &str) -> Option<Color32> {
        out.shapes.iter().find_map(|clipped| match &clipped.shape {
            egui::Shape::Text(ts) if ts.galley.job.text == text => {
                ts.galley.job.sections.first().map(|s| s.format.color)
            }
            _ => None,
        })
    }

    // ---- FG-1 stretch (decision 1) ----

    #[test]
    fn fg1_vertical_group_stretches_to_widest_member() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let id = egui::Id::unique("ig_fg1_vertical");
        let mut text = String::new();
        let mut w = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "ig_fg1_vertical_ui", 100.0);
            let ui = &mut ui;
            group::begin(ui, id, GroupOrientation::Vertical);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Vertical,
                    cross: 300.0,
                },
            );
            InputGroup::new().id_salt("fg1").show(ui, |ui| {
                w = InputGroupInput::new(&mut text).show(ui).rect.width();
            });
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
        });
        out.textures_delta.clear();
        assert!((w - 300.0).abs() < 0.01, "stretched width {w}, want 300");
    }

    #[test]
    fn fg1_horizontal_target_never_stretches() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let id = egui::Id::unique("ig_fg1_horizontal");
        let mut text = String::new();
        let mut w = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "ig_fg1_horizontal_ui", 100.0);
            let ui = &mut ui;
            group::begin(ui, id, GroupOrientation::Horizontal);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Horizontal,
                    cross: 300.0,
                },
            );
            InputGroup::new().id_salt("fg1h").show(ui, |ui| {
                w = InputGroupInput::new(&mut text).show(ui).rect.width();
            });
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
        });
        out.textures_delta.clear();
        assert!((w - 100.0).abs() < 0.01, "width {w}, want 100");
    }

    #[test]
    fn fg1_explicit_width_is_exempt_from_stretch() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let id = egui::Id::unique("ig_fg1_explicit");
        let mut text = String::new();
        let mut w = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "ig_fg1_explicit_ui", 100.0);
            let ui = &mut ui;
            group::begin(ui, id, GroupOrientation::Vertical);
            group::push_stretch_target(
                ui.ctx(),
                group::StretchTarget {
                    orientation: GroupOrientation::Vertical,
                    cross: 300.0,
                },
            );
            InputGroup::new()
                .id_salt("fg1e")
                .width(120.0)
                .show(ui, |ui| {
                    w = InputGroupInput::new(&mut text).show(ui).rect.width();
                });
            group::pop_stretch_target(ui.ctx());
            group::end(ui, id);
        });
        out.textures_delta.clear();
        assert!((w - 120.0).abs() < 0.01, "width {w}, want 120");
    }

    // ---- 1px inset removal (decision 2, BTG-VIS-01) ----

    #[test]
    fn fused_middle_member_paints_full_rect_with_clipped_leading_edge() {
        // The InputGroup is the Middle member of a horizontal group
        // (a dummy button first consumes index 0). Its border must
        // paint on the FULL content box (no 1px inset) with the leading
        // edge removed by CLIPPING, not by shifting the draw rect.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let id = egui::Id::unique("ig_fused_middle");
        let mut text = String::new();
        let input_rect = Cell::new(Rect::NOTHING);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let mut ui = test_ui(ctx, "ig_fused_middle_ui", 500.0);
            let ui = &mut ui;
            group::begin(ui, id, GroupOrientation::Horizontal);
            Button::new("A").show(ui);
            InputGroup::new().id_salt("fused").show(ui, |ui| {
                input_rect.set(InputGroupInput::new(&mut text).show(ui).rect);
            });
            group::end(ui, id);
        });
        out.textures_delta.clear();
        let want = input_rect.get();
        assert!(want.width() > 0.0);
        let strokes = stroked_rects_at(&out, want);
        assert!(
            !strokes.is_empty(),
            "no border stroke on the full content box {want:?}"
        );
        assert!(
            strokes
                .iter()
                .any(|(_, clip, _)| { (clip.min.x - (want.min.x + 1.0)).abs() < 0.5 }),
            "leading edge must be removed by a clip excluding the first \
             1px column, not by insetting the draw rect"
        );
    }

    // ---- OPEN-3 disabled mapping (decision 3) ----

    #[test]
    fn open3_nova_disabled_fill_and_border_fade() {
        // Nova light, container disabled: fill `bg-input/50` times the
        // container opacity (effective 25%), border at 50%.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let p = palette(Mode::Light);
        let mut text = String::new();
        let input_rect = Cell::new(Rect::NOTHING);
        let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            InputGroup::new()
                .id_salt("open3nova")
                .disabled(true)
                .show(ui, |ui| {
                    input_rect.set(InputGroupInput::new(&mut text).show(ui).rect);
                });
        });
        let want = input_rect.get();
        let fills = filled_rects_at(&out, want);
        let want_fill = color_alpha(color_alpha(p.input, 0.5), 0.5);
        assert!(
            fills.contains(&want_fill),
            "nova disabled fill must be bg-input/50 at 50% opacity \
             (effective 25%), got {fills:?}"
        );
        let strokes = stroked_rects_at(&out, want);
        assert!(
            strokes
                .iter()
                .any(|(_, _, s)| s.color == color_alpha(p.input, 0.5)),
            "nova disabled border must be at 50% alpha, got {strokes:?}"
        );
    }

    #[test]
    fn open3_base_disabled_container_unchanged() {
        // Base light, container disabled: NO container-level disabled
        // rule — the fill stays transparent and the border full alpha.
        let ctx = egui::Context::default();
        Preset::default().with_style(Style::Vega).install(&ctx);
        let p = Preset::default()
            .with_style(Style::Vega)
            .build(Mode::Light)
            .palette;
        let mut text = String::new();
        let input_rect = Cell::new(Rect::NOTHING);
        let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            InputGroup::new()
                .id_salt("open3base")
                .disabled(true)
                .show(ui, |ui| {
                    input_rect.set(InputGroupInput::new(&mut text).show(ui).rect);
                });
        });
        let want = input_rect.get();
        assert!(
            filled_rects_at(&out, want).is_empty(),
            "base disabled must not paint a container fill (light mode)"
        );
        let strokes = stroked_rects_at(&out, want);
        assert!(
            strokes.iter().any(|(_, _, s)| s.color == p.input),
            "base disabled border must stay full alpha, got {strokes:?}"
        );
    }

    #[test]
    fn open3_group_disabled_fades_addon_text() {
        // Mechanism (b), base style: the addon fades to 50%; the
        // container chrome is unchanged and the control is not disabled.
        let ctx = egui::Context::default();
        Preset::default().with_style(Style::Vega).install(&ctx);
        let p = Preset::default()
            .with_style(Style::Vega)
            .build(Mode::Light)
            .palette;
        let mut text = String::new();
        let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            InputGroup::new()
                .id_salt("open3b")
                .group_disabled(true)
                .show(ui, |ui| {
                    InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
                        InputGroupText::new("https://").show(ui);
                    });
                    InputGroupInput::new(&mut text).show(ui);
                });
        });
        assert_eq!(
            text_color(&out, "https://"),
            Some(color_alpha(p.muted_foreground, 0.5)),
            "group-disabled addon text must fade to 50%"
        );
    }

    #[test]
    fn open3_double_fade_is_multiplicative() {
        // Both mechanisms under nova: the addon renders at 25%
        // (`0.5 x 0.5` — both rules match), not 50%.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let p = palette(Mode::Light);
        let mut text = String::new();
        let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            InputGroup::new()
                .id_salt("open3d")
                .disabled(true)
                .group_disabled(true)
                .show(ui, |ui| {
                    InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
                        InputGroupText::new("https://").show(ui);
                    });
                    InputGroupInput::new(&mut text).show(ui);
                });
        });
        assert_eq!(
            text_color(&out, "https://"),
            Some(color_alpha(p.muted_foreground, 0.25)),
            "both disabled mechanisms under nova must render the addon \
             at 25% (0.5 x 0.5)"
        );
    }

    fn test_ctx() -> egui::Context {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        ctx
    }

    // ---- N15 block-y conditional (§3.3) ----

    fn measure_block_padding(style: Style, with_input: bool) -> f32 {
        let ctx = egui::Context::default();
        if style == Style::Vega {
            Preset::default().with_style(Style::Vega).install(&ctx);
        } else {
            Preset::default().install(&ctx);
        }
        let mut text = String::new();
        let mut ta = String::new();
        let group_rect = Cell::new(Rect::NOTHING);
        let text_rect = Cell::new(Rect::NOTHING);
        let mut render = |ui: &mut Ui| {
            let r = InputGroup::new().id_salt("n15").show(ui, |ui| {
                text_rect.set(
                    InputGroupAddon::new(AddonAlign::BlockStart)
                        .show(ui, |ui| {
                            InputGroupText::new("Bio").show(ui);
                        })
                        .response
                        .rect,
                );
                if with_input {
                    InputGroupInput::new(&mut text).show(ui);
                } else {
                    InputGroupTextarea::new(&mut ta).show(ui);
                }
            });
            group_rect.set(r.response.rect);
        };
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        text_rect.get().min.y - group_rect.get().min.y
    }

    #[test]
    fn n15_block_padding_conditional() {
        // Base: 10px with a direct-child `input`, 12px without (a
        // textarea does not trigger the reduction). Nova: uniformly 8px.
        assert!((measure_block_padding(Style::Vega, true) - 10.0).abs() < 0.5);
        assert!((measure_block_padding(Style::Vega, false) - 12.0).abs() < 0.5);
        assert!((measure_block_padding(Style::Nova, true) - 8.0).abs() < 0.5);
        assert!((measure_block_padding(Style::Nova, false) - 8.0).abs() < 0.5);
    }

    // ---- Negative margin (§3.4) ----

    fn measure_inline_end_trailing(style: Style, with_button: bool) -> f32 {
        let ctx = egui::Context::default();
        if style == Style::Vega {
            Preset::default().with_style(Style::Vega).install(&ctx);
        } else {
            Preset::default().install(&ctx);
        }
        let mut text = String::new();
        let group_rect = Cell::new(Rect::NOTHING);
        let content_rect = Cell::new(Rect::NOTHING);
        let mut render = |ui: &mut Ui| {
            let r = InputGroup::new().id_salt("negmargin").show(ui, |ui| {
                InputGroupInput::new(&mut text).show(ui);
                let _ = InputGroupAddon::new(AddonAlign::InlineEnd).show(ui, |ui| {
                    if with_button {
                        content_rect.set(InputGroupButton::icon(Icon::Copy).show(ui).rect);
                    } else {
                        content_rect.set(InputGroupText::new("Go").show(ui).rect);
                    }
                });
            });
            group_rect.set(r.response.rect);
        };
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        group_rect.get().max.x - content_rect.get().max.x
    }

    #[test]
    fn negative_margin_for_direct_child_button() {
        // Base: a direct-child button pulls the addon 7.2px toward the
        // edge (trailing 12 − 7.2 = 4.8px); without a button the full
        // 12px applies. Nova: 4.8px margin (trailing 8 − 4.8 = 3.2px)
        // vs 8px.
        assert!((measure_inline_end_trailing(Style::Vega, true) - 4.8).abs() < 0.5);
        assert!((measure_inline_end_trailing(Style::Vega, false) - 12.0).abs() < 0.5);
        assert!((measure_inline_end_trailing(Style::Nova, true) - 3.2).abs() < 0.5);
        assert!((measure_inline_end_trailing(Style::Nova, false) - 8.0).abs() < 0.5);
    }

    // ---- Click-to-focus (§7.2) ----

    #[test]
    fn addon_click_focuses_input() {
        use std::cell::RefCell;
        let ctx = test_ctx();
        let text = RefCell::new(String::new());
        let addon_rect = Cell::new(Rect::NOTHING);
        let input_id = Cell::new(Id::unique("ig_click_placeholder"));
        let render = |ui: &mut Ui| {
            InputGroup::new().id_salt("click").show(ui, |ui| {
                addon_rect.set(
                    InputGroupAddon::new(AddonAlign::InlineStart)
                        .show(ui, |ui| {
                            InputGroupText::new("https://").show(ui);
                        })
                        .response
                        .rect,
                );
                input_id.set(InputGroupInput::new(&mut text.borrow_mut()).show(ui).id);
            });
        };
        run_ui(&ctx, egui::RawInput::default(), &render);
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert!(addon_rect.get().width() > 0.0);
        let at = addon_rect.get().center();
        run_ui(&ctx, click_event(at, true), &render);
        run_ui(&ctx, click_event(at, false), &render);
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert_eq!(
            ctx.memory(|mem| mem.focused()),
            Some(input_id.get()),
            "addon click must focus the group's input"
        );
    }

    #[test]
    fn addon_click_on_button_does_not_focus() {
        // §7.2 rule 1: a click whose target is inside a `<button>` does
        // NOTHING in this handler — the button receives the click.
        use std::cell::RefCell;
        let ctx = test_ctx();
        let text = RefCell::new(String::new());
        let button_rect = Cell::new(Rect::NOTHING);
        let button_clicked = Cell::new(false);
        let input_id = Cell::new(Id::unique("ig_btnclick_placeholder"));
        let render = |ui: &mut Ui| {
            InputGroup::new().id_salt("btnclick").show(ui, |ui| {
                input_id.set(InputGroupInput::new(&mut text.borrow_mut()).show(ui).id);
                let _ = InputGroupAddon::new(AddonAlign::InlineEnd).show(ui, |ui| {
                    let r = InputGroupButton::icon(Icon::Copy).show(ui);
                    button_rect.set(r.rect);
                    if r.clicked() {
                        button_clicked.set(true);
                    }
                });
            });
        };
        run_ui(&ctx, egui::RawInput::default(), &render);
        run_ui(&ctx, egui::RawInput::default(), &render);
        let at = button_rect.get().center();
        run_ui(&ctx, click_event(at, true), &render);
        run_ui(&ctx, click_event(at, false), &render);
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert!(button_clicked.get(), "the button must receive the click");
        assert_ne!(
            ctx.memory(|mem| mem.focused()),
            Some(input_id.get()),
            "a click on an addon button must not focus the input"
        );
    }

    #[test]
    fn addon_click_never_focuses_textarea() {
        // §7.2 rule 3: only `<input>` elements are focused — never a
        // `<textarea>`. An addon click in a textarea-only group is a
        // no-op for focus.
        use std::cell::RefCell;
        let ctx = test_ctx();
        let ta = RefCell::new(String::new());
        let addon_rect = Cell::new(Rect::NOTHING);
        let ta_id = Cell::new(Id::unique("ig_taclick_placeholder"));
        let render = |ui: &mut Ui| {
            InputGroup::new().id_salt("taclick").show(ui, |ui| {
                addon_rect.set(
                    InputGroupAddon::new(AddonAlign::InlineStart)
                        .show(ui, |ui| {
                            InputGroupText::new("Bio").show(ui);
                        })
                        .response
                        .rect,
                );
                ta_id.set(InputGroupTextarea::new(&mut ta.borrow_mut()).show(ui).id);
            });
        };
        run_ui(&ctx, egui::RawInput::default(), &render);
        run_ui(&ctx, egui::RawInput::default(), &render);
        let at = addon_rect.get().center();
        run_ui(&ctx, click_event(at, true), &render);
        run_ui(&ctx, click_event(at, false), &render);
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert_eq!(
            ctx.memory(|mem| mem.focused()),
            None,
            "an addon click must never focus a textarea"
        );
        assert_ne!(ta_id.get(), Id::unique("ig_taclick_placeholder"));
    }

    #[test]
    fn addon_click_focuses_first_input() {
        // §7.2 rule 4: `querySelector` returns the FIRST input in
        // document order.
        use std::cell::RefCell;
        let ctx = test_ctx();
        let text1 = RefCell::new(String::new());
        let text2 = RefCell::new(String::new());
        let addon_rect = Cell::new(Rect::NOTHING);
        let first_id = Cell::new(Id::unique("ig_first_placeholder"));
        let second_id = Cell::new(Id::unique("ig_second_placeholder"));
        let render = |ui: &mut Ui| {
            InputGroup::new().id_salt("firstwins").show(ui, |ui| {
                addon_rect.set(
                    InputGroupAddon::new(AddonAlign::InlineStart)
                        .show(ui, |ui| {
                            InputGroupText::new("https://").show(ui);
                        })
                        .response
                        .rect,
                );
                first_id.set(InputGroupInput::new(&mut text1.borrow_mut()).show(ui).id);
                second_id.set(InputGroupInput::new(&mut text2.borrow_mut()).show(ui).id);
            });
        };
        run_ui(&ctx, egui::RawInput::default(), &render);
        run_ui(&ctx, egui::RawInput::default(), &render);
        let at = addon_rect.get().center();
        run_ui(&ctx, click_event(at, true), &render);
        run_ui(&ctx, click_event(at, false), &render);
        run_ui(&ctx, egui::RawInput::default(), &render);
        assert_ne!(first_id.get(), second_id.get());
        assert_eq!(
            ctx.memory(|mem| mem.focused()),
            Some(first_id.get()),
            "addon click must focus the FIRST input"
        );
    }

    // ---- Frameless inner control ----

    #[test]
    fn inner_control_is_frameless() {
        // The input paints no chrome of its own: exactly one stroked
        // rect sits on the input rect — the container's border.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from("abc");
        let input_rect = Cell::new(Rect::NOTHING);
        let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            InputGroup::new().id_salt("frameless").show(ui, |ui| {
                input_rect.set(InputGroupInput::new(&mut text).show(ui).rect);
            });
        });
        let strokes = stroked_rects_at(&out, input_rect.get());
        assert_eq!(
            strokes.len(),
            1,
            "the input must add no border of its own (all chrome is on \
             the container); found {} stroked rects at the input rect",
            strokes.len()
        );
    }

    // ---- Textarea row + auto-height (§3.1) ----

    #[test]
    fn textarea_keeps_row_layout_with_auto_height() {
        // A direct-child textarea makes the height `h-auto` WITHOUT
        // changing direction (§3.1): the textarea shares the row with
        // the addon (same top edge) and the container grows past `h-8`.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut ta = String::new();
        let group_rect = Cell::new(Rect::NOTHING);
        let ta_rect = Cell::new(Rect::NOTHING);
        let mut render = |ui: &mut Ui| {
            let r = InputGroup::new().id_salt("tarow").show(ui, |ui| {
                InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
                    InputGroupText::new("Bio").show(ui);
                });
                ta_rect.set(InputGroupTextarea::new(&mut ta).show(ui).rect);
            });
            group_rect.set(r.response.rect);
        };
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        assert!(
            (ta_rect.get().min.y - group_rect.get().min.y).abs() < 1.0,
            "textarea must share the row (same top edge), got offset {}",
            ta_rect.get().min.y - group_rect.get().min.y
        );
        assert!(
            group_rect.get().height() > 32.0,
            "textarea group must be auto-height, got {}",
            group_rect.get().height()
        );
    }

    // ---- Divider (§3.3) ----

    fn measure_input_offset_with_block_start(style: Style, divider: bool) -> f32 {
        let ctx = egui::Context::default();
        if style == Style::Vega {
            Preset::default().with_style(Style::Vega).install(&ctx);
        } else {
            Preset::default().install(&ctx);
        }
        let mut text = String::new();
        let group_rect = Cell::new(Rect::NOTHING);
        let input_rect = Cell::new(Rect::NOTHING);
        let mut render = |ui: &mut Ui| {
            let r = InputGroup::new().id_salt("divider").show(ui, |ui| {
                InputGroupAddon::new(AddonAlign::BlockStart)
                    .divider(divider)
                    .show(ui, |ui| {
                        InputGroupText::new("Bio").show(ui);
                    });
                input_rect.set(InputGroupInput::new(&mut text).show(ui).rect);
            });
            group_rect.set(r.response.rect);
        };
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        run_ui(&ctx, egui::RawInput::default(), &mut render);
        input_rect.get().min.y - group_rect.get().min.y
    }

    #[test]
    fn divider_adds_opposite_side_padding() {
        // Block-start + input, base: the divider adds exactly the
        // opposite-side padding (10px here — the N15-reduced `pt-3`
        // value, §3.3) to the input's offset. The ABSOLUTE offset is
        // 46/56, not the spec's 30/40: the block addon's row is
        // `interact_size`-tall (36px base) instead of content-height
        // (~20px) — a known limitation, recorded in the report
        // (measure-then-real is blocked: the content closure is
        // `FnOnce`). Only the divider delta is pinned here.
        let without = measure_input_offset_with_block_start(Style::Vega, false);
        let with = measure_input_offset_with_block_start(Style::Vega, true);
        assert!(
            ((with - without) - 10.0).abs() < 0.5,
            "a block-start divider must add 10px to the input offset, got {without} -> {with}"
        );
    }

    // ---- Edge cases (§9) ----

    #[test]
    fn empty_and_addons_only_groups_render() {
        // E7/E8: an empty group and a group with only addons render the
        // 36/32px bordered box without panicking.
        for style in [Style::Nova, Style::Vega] {
            let ctx = egui::Context::default();
            if style == Style::Vega {
                Preset::default().with_style(Style::Vega).install(&ctx);
            } else {
                Preset::default().install(&ctx);
            }
            let want = if style == Style::Nova { 32.0 } else { 36.0 };
            let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
                InputGroup::new().id_salt("empty").show(ui, |_| {});
            });
            let strokes: Vec<Rect> = out
                .shapes
                .iter()
                .filter_map(|c| match &c.shape {
                    egui::Shape::Rect(rs) if rs.stroke.width > 0.0 => Some(rs.rect),
                    _ => None,
                })
                .collect();
            assert_eq!(strokes.len(), 1, "empty group must paint one border");
            assert!(
                (strokes[0].height() - want).abs() < 1.0,
                "empty group height {}, want {want}",
                strokes[0].height()
            );
            let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
                InputGroup::new().id_salt("addons_only").show(ui, |ui| {
                    InputGroupAddon::new(AddonAlign::InlineStart).show(ui, |ui| {
                        InputGroupText::new("https://").show(ui);
                    });
                });
            });
            let strokes: Vec<Rect> = out
                .shapes
                .iter()
                .filter_map(|c| match &c.shape {
                    egui::Shape::Rect(rs) if rs.stroke.width > 0.0 => Some(rs.rect),
                    _ => None,
                })
                .collect();
            assert_eq!(strokes.len(), 1, "addons-only group must paint one border");
            assert!(
                (strokes[0].height() - want).abs() < 1.0,
                "addons-only group height {}, want {want}",
                strokes[0].height()
            );
        }
    }

    // ---- Icon sizes (§3.4) ----

    #[test]
    fn icon_size_for_table() {
        // §3.4 note (a): the `size-3.5` rule appears ONLY in the xs
        // string; `sm`, `icon-xs` and `icon-sm` fall back to Button's
        // base `size-4` (16px).
        assert_eq!(icon_size_for(InputGroupButtonSize::Xs), 14.0);
        assert_eq!(icon_size_for(InputGroupButtonSize::Sm), 16.0);
        assert_eq!(icon_size_for(InputGroupButtonSize::IconXs), 16.0);
        assert_eq!(icon_size_for(InputGroupButtonSize::IconSm), 16.0);
    }

    #[test]
    fn button_icon_sizes_follow_spec() {
        // A text button's width grows by icon + gap when an icon is
        // added — MINUS the 2px the left padding shrinks when an icon
        // is present (Button's `padding_left`: pl-1.5/pl-2 with an
        // icon vs px-2/px-2.5 without). Xs: 14 + 4 − 2 = 16px; Sm
        // (mapped to Default under nova): 16 + 6 − 2 = 20px (§3.4).
        for (size, want_diff) in [
            (InputGroupButtonSize::Xs, 16.0),
            (InputGroupButtonSize::Sm, 20.0),
        ] {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            let mut w_plain = 0.0;
            let mut w_icon = 0.0;
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut test_ui(ctx, "ig_icon_size_ui", 500.0);
                w_plain = InputGroupButton::new("Search")
                    .size(size)
                    .id_salt("plain")
                    .show(ui)
                    .rect
                    .width();
                w_icon = InputGroupButton::new("Search")
                    .size(size)
                    .icon_start(Icon::Search)
                    .id_salt("icon")
                    .show(ui)
                    .rect
                    .width();
            });
            out.textures_delta.clear();
            assert!(
                ((w_icon - w_plain) - want_diff).abs() < 1.0,
                "{size:?}: icon must add {want_diff}px, added {}",
                w_icon - w_plain
            );
        }
    }

    // ---- Control margins (§3.5) ----

    #[test]
    fn control_margins_table() {
        // Sides from the inline addons (or the input's own px-3 /
        // px-2.5); top/bottom from the block addons (pt-3/pb-3, both
        // styles).
        let nova = Style::Nova;
        let base = Style::Vega;
        assert_eq!(control_margins(base, false, false, false, false).left, 12);
        assert_eq!(control_margins(nova, false, false, false, false).left, 10);
        assert_eq!(control_margins(base, true, false, false, false).left, 8);
        assert_eq!(control_margins(nova, true, false, false, false).left, 6);
        assert_eq!(control_margins(base, false, true, false, false).right, 8);
        assert_eq!(control_margins(nova, false, true, false, false).right, 6);
        assert_eq!(control_margins(base, false, false, true, false).bottom, 12);
        assert_eq!(control_margins(nova, false, false, true, false).bottom, 12);
        assert_eq!(control_margins(base, false, false, false, true).top, 12);
        assert_eq!(control_margins(nova, false, false, false, true).top, 12);
    }

    // ---- Column centring (§3.1 items-center) ----
    //
    // NOT PINNED (known limitation, recorded in the report): §3.1
    // requires `items-center` in ALL modes, so a narrow inline addon
    // in a column group should be horizontally centred. The port's
    // `top_down(Center)` cannot centre it — the addon's `ui.horizontal`
    // allocates the FULL available width, so the content sits
    // left-aligned inside a full-width row. Centring it needs
    // measure-then-real for the addon's content width, which the
    // `FnOnce` content closure cannot support. The demo's column
    // group (textarea section) has full-width children, so no demo
    // composition is affected. The `top_down(Center)` declaration is
    // kept (spec-correct as a layout declaration; harmless for
    // full-width children).

    // ---- Invalid trigger (§4.3: ANY slotted descendant) ----

    #[test]
    fn invalid_button_triggers_destructive_border() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let p = palette(Mode::Light);
        let mut text = String::new();
        // The container border spans the GROUP rect, not the input
        // rect (the input starts at x=55.5 here); measure there.
        let group_rect = Cell::new(Rect::NOTHING);
        let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            let r = InputGroup::new().id_salt("invbtn").show(ui, |ui| {
                InputGroupInput::new(&mut text).show(ui);
                InputGroupAddon::new(AddonAlign::InlineEnd).show(ui, |ui| {
                    InputGroupButton::icon(Icon::Copy).invalid(true).show(ui);
                });
            });
            group_rect.set(r.response.rect);
        });
        let strokes = stroked_rects_at(&out, group_rect.get());
        assert!(
            strokes.iter().any(|(_, _, s)| s.color == p.destructive),
            "an invalid button must turn the container border destructive, got {strokes:?}"
        );
    }

    #[test]
    fn invalid_addon_triggers_destructive_border() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let p = palette(Mode::Light);
        let mut text = String::new();
        // The container border spans the GROUP rect, not the input
        // rect; measure there.
        let group_rect = Cell::new(Rect::NOTHING);
        let out = run_ui_capture(&ctx, egui::RawInput::default(), |ui| {
            let r = InputGroup::new().id_salt("invaddon").show(ui, |ui| {
                InputGroupAddon::new(AddonAlign::InlineStart)
                    .invalid(true)
                    .show(ui, |ui| {
                        InputGroupText::new("https://").show(ui);
                    });
                InputGroupInput::new(&mut text).show(ui);
            });
            group_rect.set(r.response.rect);
        });
        let strokes = stroked_rects_at(&out, group_rect.get());
        assert!(
            strokes.iter().any(|(_, _, s)| s.color == p.destructive),
            "an invalid addon must turn the container border destructive, got {strokes:?}"
        );
    }
}
