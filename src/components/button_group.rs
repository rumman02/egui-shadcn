//! shadcn `button-group.tsx` — a row (or column) of fused controls.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/button-group.tsx`. The
//! source fuses children with `[&>*:not(:first-child)]:rounded-l-none` and
//! `border-l-0`; here [`Button`], [`ButtonGroupText`] and
//! [`ButtonGroupSeparator`] read their position from the active group
//! (plumbing in `crate::components::button::group`) and adjust their own
//! corners and borders automatically.
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
//! ```

use crate::components::button::group;
use crate::components::button::{GroupOrientation, GroupPosition};
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{
    Align2, Color32, CornerRadius, FontId, InnerResponse, Response, Sense, Stroke, Ui, Vec2, Widget,
};

/// A shadcn button group. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct ButtonGroup {
    orientation: GroupOrientation,
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
            id_salt: None,
        }
    }

    /// Sets the layout orientation.
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets a stable id salt. Groups without one derive their id from the
    /// parent ui's widget counter, which is stable for static layouts.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its children with zero item spacing, so the
    /// children's borders fuse like the source.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.next_auto_id(),
        };
        group::begin(ui, id, self.orientation);
        let response = match self.orientation {
            GroupOrientation::Horizontal => ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                content(ui)
            }),
            GroupOrientation::Vertical => ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                content(ui)
            }),
        };
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

/// `flex items-center gap-2 rounded-md border bg-muted px-4 text-sm
/// font-medium shadow-xs` — the non-interactive group slot.
#[derive(Clone, Debug)]
pub struct ButtonGroupText {
    text: String,
    height: f32,
    disabled: bool,
    id_salt: Option<String>,
}

impl ButtonGroupText {
    /// Creates a group text slot sized like a default button.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            height: crate::theme::tokens::spacing::CONTROL,
            disabled: false,
            id_salt: None,
        }
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

        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout_no_wrap(self.text.clone(), font.clone(), Color32::WHITE);
        // px-4 on both sides; gap-2 slack so leading icons can be added.
        let desired = Vec2::new(
            galley.size().x + 2.0 * crate::theme::tokens::spacing::LG,
            self.height,
        );

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
                CornerRadius::same(theme.radius.md_u8()),
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
            painter.text(
                draw_rect.center(),
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
/// the group's child positions like any other member.
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
        let size = match self.orientation {
            GroupOrientation::Horizontal => Vec2::new(1.0, self.length),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect};

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
    fn builders_keep_fields() {
        let g = ButtonGroup::new()
            .orientation(GroupOrientation::Vertical)
            .id_salt("g");
        assert_eq!(g.orientation, GroupOrientation::Vertical);
        assert_eq!(g.id_salt.as_deref(), Some("g"));
        let t = ButtonGroupText::new("x")
            .height(32.0)
            .disabled(true)
            .id_salt("t");
        assert_eq!(t.height, 32.0);
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
}
