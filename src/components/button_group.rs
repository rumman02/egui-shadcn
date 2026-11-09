//! shadcn `button-group.tsx` — a row (or column) of fused controls.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/button-group.tsx`. The
//! source fuses children with `[&>*:not(:first-child)]:rounded-l-none` and
//! `border-l-0`; here [`Button`], [`ButtonGroupText`] and
//! [`ButtonGroupSeparator`] read their position from the active group
//! (plumbing in `crate::components::button::group`) and adjust their own
//! corners and borders automatically. Nova (`style-nova.css`) only restyles
//! the text slot (`rounded-lg px-2.5`) and forces `rounded-lg` on the last
//! child's outer corners — both follow from the per-widget nova metrics, so
//! no extra code is needed here.
//!
//! The source row is `flex items-stretch` (all members grow to the tallest
//! one) and gains `gap-2` when it contains a nested
//! [`ButtonGroup`] (`has-[>[data-slot=button-group]]:gap-2`). Call
//! `ButtonGroup::gap(crate::theme::tokens::spacing::SM)` on the outer group
//! for the nested case; the group tracks the tallest natural member height
//! and stretches [`ButtonGroupText`] and [`ButtonGroupSeparator`] up to it.
//! (`Button` members follow their own per-size metrics and are not
//! stretched yet — that needs shared group plumbing in `button.rs`.)
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
//! // Outer group around a nested group: gap-2 between the two slots.
//! ButtonGroup::new().gap(crate::theme::tokens::spacing::SM).show(ui, |ui| {
//!     ButtonGroup::new().show(ui, |ui| { /* ... */ });
//!     Button::new("Submit").show(ui);
//! });
//! ```

use crate::components::button::group;
use crate::components::button::{GroupOrientation, GroupPosition};
use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{
    Align2, Color32, Context, CornerRadius, FontId, Id, InnerResponse, Pos2, Rect, Response, Sense,
    Stroke, Ui, Vec2, Widget,
};

/// A shadcn button group. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct ButtonGroup {
    orientation: GroupOrientation,
    gap: f32,
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
            gap: 0.0,
            id_salt: None,
        }
    }

    /// Sets the layout orientation.
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets extra spacing between the group's children. The source keeps the
    /// members fused (zero gap) and only widens to `gap-2` (8px) when the
    /// group contains a nested [`ButtonGroup`]:
    /// `has-[>[data-slot=button-group]]:gap-2`.
    pub fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    /// Sets a stable id salt. Groups without one derive their id from the
    /// parent ui's widget counter, which is stable for static layouts.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its children with zero item spacing (or
    /// [`ButtonGroup::gap`]), so the children's borders fuse like the source.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.next_auto_id(),
        };
        group::begin(ui, id, self.orientation);
        begin_stretch(ui, id);
        let gap = self.gap;
        let response = match self.orientation {
            GroupOrientation::Horizontal => ui.horizontal(move |ui| {
                ui.spacing_mut().item_spacing.x = gap;
                content(ui)
            }),
            GroupOrientation::Vertical => ui.vertical(move |ui| {
                ui.spacing_mut().item_spacing.y = gap;
                content(ui)
            }),
        };
        end_stretch(ui);
        group::end(ui, id);
        response
    }
}

/// Group-aware chrome shared by [`ButtonGroupText`] and
/// [`ButtonGroupSeparator`]: fused corners + overlapped leading border.
pub(crate) fn group_adjustments(
    orientation: GroupOrientation,
    position: GroupPosition,
    base_corner: CornerRadius,
    rect: egui::Rect,
) -> (CornerRadius, egui::Rect) {
    let corner = crate::components::button::fuse_radius(base_corner, orientation, position);
    let draw_rect = match orientation {
        GroupOrientation::Horizontal
            if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
        {
            egui::Rect::from_min_max(rect.min + Vec2::new(1.0, 0.0), rect.max)
        }
        GroupOrientation::Vertical
            if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
        {
            egui::Rect::from_min_max(rect.min + Vec2::new(0.0, 1.0), rect.max)
        }
        _ => rect,
    };
    (corner, draw_rect)
}

/// Resolved colors for [`ButtonGroupText`] (pure, testable).
pub(crate) fn text_colors(p: &Palette) -> (Color32, Color32, Stroke) {
    (p.muted, p.foreground, Stroke::new(1.0, p.border))
}

/// [`ButtonGroupText`] padding and corner radius per style: the base recipe
/// is `px-4 rounded-md`; nova overrides to `px-2.5 rounded-lg`
/// (`style-nova.css` `.cn-button-group-text`).
pub(crate) fn text_metrics(style: Style, radius: &crate::theme::RadiusTokens) -> (f32, u8) {
    match style {
        Style::Nova => (10.0, radius.lg_u8()), // px-2.5, rounded-lg
        _ => (crate::theme::tokens::spacing::LG, radius.md_u8()), // px-4, rounded-md
    }
}

/// `flex items-center gap-2 rounded-md border bg-muted px-4 text-sm
/// font-medium shadow-xs` — the non-interactive group slot. Nova restyles it
/// to `rounded-lg px-2.5`.
#[derive(Clone, Debug)]
pub struct ButtonGroupText {
    text: String,
    icon_start: Option<Icon>,
    height: f32,
    disabled: bool,
    id_salt: Option<String>,
}

impl ButtonGroupText {
    /// Creates a group text slot sized like a default button.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            icon_start: None,
            height: crate::theme::tokens::spacing::CONTROL,
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

    /// Overrides the slot height.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
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

    /// Renders the slot.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();

        let group = group::next_child_position(ui);
        let position = group.map(|(_, pos)| pos).unwrap_or(GroupPosition::Only);
        let orientation = group
            .map(|(o, _)| o)
            .unwrap_or(GroupOrientation::Horizontal);

        // `items-stretch`: adopt last frame's tallest member height, and
        // report this slot's natural height for the next frame.
        let group_id = current_group_id(ui);
        let stretch = group_id
            .map(|gid| stretch_height(ui.ctx(), gid))
            .unwrap_or(0.0);
        let natural = self.height;
        if let Some(gid) = group_id {
            report_member_height(ui.ctx(), gid, natural);
        }
        let height = natural.max(stretch);

        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), Color32::WHITE);

        let (pad_x, corner_radius) = text_metrics(theme.style, &theme.radius);
        let icon_size = 16.0; // [&_svg:not([class*='size-'])]:size-4
        let icon_gap = 8.0; // gap-2
        let has_icon = self.icon_start.is_some();
        let icon_w = if has_icon { icon_size } else { 0.0 };
        let icon_gap_w = if has_icon { icon_gap } else { 0.0 };
        // px-4 (base) / px-2.5 (nova) on both sides.
        let desired = Vec2::new(2.0 * pad_x + icon_w + icon_gap_w + galley.size().x, height);

        let (rect, response) = match self.id_salt.as_deref() {
            Some(salt) => {
                let id = ui.scope_id().with(salt);
                let (rect, _) = ui.allocate_exact_size(desired, Sense::hover());
                (rect, ui.interact(rect, id, Sense::hover()))
            }
            None => ui.allocate_exact_size(desired, Sense::hover()),
        };

        if ui.is_rect_visible(rect) {
            let (mut fill, mut fg, mut stroke) = text_colors(&p);
            if self.disabled {
                fill = color_alpha(fill, 0.5);
                fg = color_alpha(fg, 0.5);
                stroke.color = color_alpha(stroke.color, 0.5);
            }
            let (corner, draw_rect) = group_adjustments(
                orientation,
                position,
                CornerRadius::same(corner_radius),
                rect,
            );
            let painter = ui.painter();
            // `shadow-xs`
            if !self.disabled {
                let sh = crate::utils::shadow::shadow_xs(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        draw_rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        corner,
                        sh.color,
                    )
                    .with_blur_width(2.0),
                );
            }
            painter.rect_filled(draw_rect, corner, fill);
            painter.rect_stroke(draw_rect, corner, stroke, egui::epaint::StrokeKind::Inside);

            // Content: [icon] gap [text], centered.
            let content_w = icon_w + icon_gap_w + galley.size().x;
            let mut cursor = draw_rect.center().x - content_w * 0.5;
            let cy = draw_rect.center().y;
            if let Some(icon) = self.icon_start {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_size * 0.5, cy),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, fg, 2.0 * icon_size / 24.0);
                cursor += icon_size + icon_gap;
            }
            painter.text(
                Pos2::new(cursor + galley.size().x * 0.5, cy),
                Align2::CENTER_CENTER,
                self.text,
                font,
                fg,
            );
        }

        response
    }
}

impl Widget for ButtonGroupText {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `bg-input self-stretch w-px` — the divider inside a group. Counts toward
/// the group's child positions like any other member, and stretches to the
/// row height in horizontal groups (`self-stretch`).
#[derive(Clone, Debug)]
pub struct ButtonGroupSeparator {
    orientation: GroupOrientation,
    length: f32,
    id_salt: Option<String>,
}

impl Default for ButtonGroupSeparator {
    fn default() -> Self {
        Self::new()
    }
}

impl ButtonGroupSeparator {
    /// Creates a vertical (1px wide) divider 36pt long, matching the row of
    /// default buttons. In vertical groups this becomes a horizontal rule.
    pub fn new() -> Self {
        Self {
            orientation: GroupOrientation::Horizontal,
            length: crate::theme::tokens::spacing::CONTROL,
            id_salt: None,
        }
    }

    /// Sets the group axis the divider lives in: `Horizontal` (the default)
    /// renders a vertical 1px rule; `Vertical` renders a horizontal rule.
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Overrides the divider length.
    pub fn length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the divider.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        // Count toward the group (this is a group member like any other).
        let _ = group::next_child_position(ui);
        // The divider's own axis is perpendicular to the group axis: a
        // vertical rule in a horizontal group, a horizontal rule in a
        // vertical group.
        let group_id = current_group_id(ui);
        let size = match self.orientation {
            GroupOrientation::Horizontal => {
                // `self-stretch`: grow to the row height when the group is
                // taller, and report this divider's natural length.
                let stretch = group_id
                    .map(|gid| stretch_height(ui.ctx(), gid))
                    .unwrap_or(0.0);
                if let Some(gid) = group_id {
                    report_member_height(ui.ctx(), gid, self.length);
                }
                Vec2::new(1.0, self.length.max(stretch))
            }
            GroupOrientation::Vertical => Vec2::new(self.length, 1.0),
        };
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

/// Cross-frame `items-stretch` state for one group: the tallest natural
/// member height reported last frame ([`ButtonGroupText`],
/// [`ButtonGroupSeparator`]) plus this frame's running max.
#[derive(Clone, Copy, Debug)]
struct StretchState {
    frame: u64,
    /// Max natural member height reported last frame.
    stretch: f32,
    /// Max natural member height reported this frame.
    current: f32,
}

fn stretch_store_id() -> Id {
    Id::unique("shadcn_button_group_stretch")
}

fn group_stack_id() -> Id {
    Id::unique("shadcn_button_group_stack")
}

/// The innermost active [`ButtonGroup`], if any.
fn current_group_id(ui: &Ui) -> Option<Id> {
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| d.get_temp::<Vec<Id>>(group_stack_id()))
        .and_then(|stack| stack.last().copied())
}

/// Last frame's tallest natural member height for the group (0.0 if none).
fn stretch_height(ctx: &Context, group_id: Id) -> f32 {
    ctx.data_mut(|d| d.get_temp::<Vec<(Id, StretchState)>>(stretch_store_id()))
        .and_then(|all| {
            all.into_iter()
                .find(|(id, _)| *id == group_id)
                .map(|(_, s)| s.stretch)
        })
        .unwrap_or(0.0)
}

/// Records a member's natural (un-stretched) height into the group's
/// running max for this frame.
fn report_member_height(ctx: &Context, group_id: Id, natural: f32) {
    ctx.data_mut(|d| {
        let mut all: Vec<(Id, StretchState)> = d.get_temp(stretch_store_id()).unwrap_or_default();
        if let Some(slot) = all.iter_mut().find(|(id, _)| *id == group_id) {
            slot.1.current = slot.1.current.max(natural);
        }
        d.insert_temp(stretch_store_id(), all);
    });
}

/// Called by [`ButtonGroup::show`] before the children: rolls the per-frame
/// stretch state over and pushes the group onto the nesting stack.
fn begin_stretch(ui: &Ui, id: Id) {
    let ctx = ui.ctx().clone();
    let frame = ctx.cumulative_frame_nr();
    ctx.data_mut(|d| {
        let mut all: Vec<(Id, StretchState)> = d.get_temp(stretch_store_id()).unwrap_or_default();
        if let Some(slot) = all.iter_mut().find(|(i, _)| *i == id) {
            if slot.1.frame != frame {
                slot.1.stretch = slot.1.current;
                slot.1.current = 0.0;
                slot.1.frame = frame;
            }
        } else {
            all.push((
                id,
                StretchState {
                    frame,
                    stretch: 0.0,
                    current: 0.0,
                },
            ));
        }
        d.insert_temp(stretch_store_id(), all);
    });
    let mut stack: Vec<Id> = ctx.data_mut(|d| d.get_temp(group_stack_id()).unwrap_or_default());
    stack.push(id);
    ctx.data_mut(|d| d.insert_temp(group_stack_id(), stack));
}

/// Called by [`ButtonGroup::show`] after the children.
fn end_stretch(ui: &Ui) {
    let ctx = ui.ctx().clone();
    let mut stack: Vec<Id> = ctx.data_mut(|d| d.get_temp(group_stack_id()).unwrap_or_default());
    stack.pop();
    ctx.data_mut(|d| d.insert_temp(group_stack_id(), stack));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect, UiBuilder};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
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
    fn builders_keep_fields() {
        let g = ButtonGroup::new()
            .orientation(GroupOrientation::Vertical)
            .gap(crate::theme::tokens::spacing::SM)
            .id_salt("g");
        assert_eq!(g.orientation, GroupOrientation::Vertical);
        assert_eq!(g.gap, 8.0);
        assert_eq!(g.id_salt.as_deref(), Some("g"));
        assert_eq!(ButtonGroup::new().gap, 0.0);
        let t = ButtonGroupText::new("x")
            .height(32.0)
            .icon_start(Icon::Copy)
            .disabled(true)
            .id_salt("t");
        assert_eq!(t.height, 32.0);
        assert!(t.icon_start.is_some());
        assert!(t.disabled);
        let s = ButtonGroupSeparator::new().length(24.0).id_salt("s");
        assert_eq!(s.length, 24.0);
    }

    #[test]
    fn default_separator_is_one_by_control() {
        let s = ButtonGroupSeparator::new();
        assert_eq!(s.orientation, GroupOrientation::Horizontal);
        assert_eq!(s.length, 36.0);
    }

    #[test]
    fn group_adjustments_fuse_and_overlap() {
        let base = CornerRadius::same(8);
        let rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(40.0, 36.0));
        let (corner, draw) = group_adjustments(
            GroupOrientation::Horizontal,
            GroupPosition::Middle,
            base,
            rect,
        );
        assert_eq!((corner.nw, corner.ne, corner.sw, corner.se), (0, 0, 0, 0));
        assert!((draw.min.x - 1.0).abs() < 1e-4);
        // Only child: untouched.
        let (corner, draw) =
            group_adjustments(GroupOrientation::Vertical, GroupPosition::Only, base, rect);
        assert_eq!(corner, base);
        assert_eq!(draw, rect);
    }

    #[test]
    fn members_stretch_to_the_tallest() {
        // `items-stretch`: a 36pt slot next to a 48pt slot adopts 48pt from
        // the second frame on (the stretch state converges over frames).
        let ctx = egui::Context::default();
        let mut frames: Vec<Vec<f32>> = Vec::new();
        for _ in 0..3 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("stretch_test"),
                    UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                let mut heights = Vec::new();
                ButtonGroup::new().show(ui, |ui| {
                    let a = ButtonGroupText::new("a").height(36.0).show(ui);
                    heights.push(a.rect.height());
                    let b = ButtonGroupText::new("b").height(48.0).show(ui);
                    heights.push(b.rect.height());
                });
                frames.push(heights);
            });
            out.textures_delta.clear();
        }
        assert_eq!(
            frames[0],
            vec![36.0, 48.0],
            "first frame has no stretch data"
        );
        assert_eq!(frames[1], vec![48.0, 48.0], "both members stretch");
        assert_eq!(frames[2], vec![48.0, 48.0], "stable across frames");
    }
}
