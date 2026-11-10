//! shadcn `direction.tsx` — the RTL/LTR provider.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/direction.tsx`, a thin
//! wrapper around Radix's `Direction.DirectionProvider` (`dir` prop +
//! `useDirection()` hook). In the web source every Radix component reads
//! the direction and mirrors its popups, chevrons and paddings.
//!
//! egui has no global layout-direction switch, so this port is a **scoped
//! wrapper**: [`DirectionProvider`] pushes the direction onto a stack for
//! the duration of its closure, flips the closure's layout (right-aligned,
//! right-to-left flow for [`Direction::Rtl`]) and exposes
//! [`current`](current)/[`is_rtl`](is_rtl) plus helpers so components can
//! consult and honor it: [`row_layout`] for row flow, [`cross_align`] for
//! stacked content, and [`popup_align`]/[`mirrored_chevron`] for the popup
//! placement and chevron mirroring Radix performs (dropdown-menu, select,
//! combobox and popover consult it). Note egui does **no** Unicode bidi
//! reordering or contextual shaping — RTL runs render in logical order —
//! so the provider flips layout geometry only.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! DirectionProvider::rtl().show(ui, |ui| {
//!     ui.label("النص من اليمين إلى اليسار"); // right-aligned
//!     // Rows written for RTL flow:
//!     ui.with_layout(direction::row_layout(ui), |ui| {
//!         Button::new("أولاً").show(ui);
//!         Button::new("ثانياً").show(ui);
//!     });
//! });
//! ```

use crate::icons::Icon;
use egui::emath::RectAlign;
use egui::{Align, Id, InnerResponse, Layout, Ui};

/// Text/layout direction, mirroring the source `dir` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Direction {
    /// Left-to-right (the source default).
    #[default]
    Ltr,
    /// Right-to-left.
    Rtl,
}

fn direction_stack_id() -> Id {
    Id::unique("shadcn_direction_stack")
}

fn push(ui: &Ui, direction: Direction) {
    let mut stack: Vec<Direction> = ui
        .ctx()
        .data_mut(|d| d.get_temp(direction_stack_id()))
        .unwrap_or_default();
    stack.push(direction);
    ui.ctx()
        .data_mut(|d| d.insert_temp(direction_stack_id(), stack));
}

fn pop(ui: &Ui) {
    let mut stack: Vec<Direction> = ui
        .ctx()
        .data_mut(|d| d.get_temp(direction_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(direction_stack_id(), stack));
}

/// The direction in effect for `ui` (nearest enclosing
/// [`DirectionProvider`], else [`Direction::Ltr`]).
pub fn current(ui: &Ui) -> Direction {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<Direction>>(direction_stack_id()))
        .and_then(|s| s.last().copied())
        .unwrap_or(Direction::Ltr)
}

/// True when the nearest [`DirectionProvider`] is RTL.
pub fn is_rtl(ui: &Ui) -> bool {
    current(ui) == Direction::Rtl
}

/// The horizontal layout direction components should lay rows out with
/// under the current direction (`left_to_right` / `right_to_left`).
pub fn row_layout(ui: &Ui) -> Layout {
    match current(ui) {
        Direction::Ltr => Layout::left_to_right(Align::Center),
        Direction::Rtl => Layout::right_to_left(Align::Center),
    }
}

/// Cross alignment for stacked content under the current direction
/// (`Align::Min` = left for LTR, `Align::Max` = right for RTL).
pub fn cross_align(direction: Direction) -> Align {
    match direction {
        Direction::Ltr => Align::Min,
        Direction::Rtl => Align::Max,
    }
}

/// Popup alignment under the current direction: `ltr` in a left-to-right
/// scope, `rtl` when an RTL [`DirectionProvider`] encloses `ui`. Radix
/// mirrors the inline `Start`/`End` popup alignment for RTL, so popups
/// anchored to a trigger should route their [`egui::Popup`] `.align(…)`
/// through this.
///
/// ```ignore
/// use egui::emath::RectAlign;
///
/// let align = direction::popup_align(ui, RectAlign::BOTTOM_START, RectAlign::BOTTOM_END);
/// Popup::new(id, ctx.clone(), PopupAnchor::ParentRect(anchor), layer).align(align);
/// ```
pub fn popup_align(ui: &Ui, ltr: RectAlign, rtl: RectAlign) -> RectAlign {
    if is_rtl(ui) {
        rtl
    } else {
        ltr
    }
}

/// Chevron icon under the current direction: horizontal indicators point
/// into the submenu (`ltr` under LTR — a right-pointing chevron — and
/// `rtl` under RTL, which then points left). Vertical chevrons simply pass
/// the same icon for both arguments.
///
/// ```ignore
/// let chevron = direction::mirrored_chevron(ui, Icon::ChevronRight, Icon::ChevronLeft);
/// chevron.paint(ui.painter(), rect, color, 2.0);
/// ```
pub fn mirrored_chevron(ui: &Ui, ltr: Icon, rtl: Icon) -> Icon {
    if is_rtl(ui) {
        rtl
    } else {
        ltr
    }
}

/// `DirectionProvider` — a scoped RTL/LTR wrapper. Inside the closure the
/// layout flips (children and text align right, flow right-to-left for
/// RTL); [`current`](current)/[`row_layout`] report the direction to
/// direction-aware widgets.
#[derive(Clone, Debug, Default)]
pub struct DirectionProvider {
    direction: Direction,
    id_salt: Option<String>,
}

impl DirectionProvider {
    /// Creates a provider for `direction` (the source's `dir` prop).
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            id_salt: None,
        }
    }

    /// An LTR provider.
    pub fn ltr() -> Self {
        Self::new(Direction::Ltr)
    }

    /// An RTL provider.
    pub fn rtl() -> Self {
        Self::new(Direction::Rtl)
    }

    /// Sets a stable id salt (DirectionProvider keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the scoped direction around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        push(ui, self.direction);
        // Flip the block layout: RTL right-aligns children and text.
        let layout = Layout::top_down(cross_align(self.direction));
        let inner = ui.with_layout(layout, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            content(ui)
        });
        pop(ui);
        inner
    }
}

impl Direction {
    /// Convenience scoped form: `Direction::rtl(ui, |ui| { … })`.
    pub fn rtl<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        DirectionProvider::rtl().show(ui, content)
    }

    /// Convenience scoped form: `Direction::ltr(ui, |ui| { … })`.
    pub fn ltr<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        DirectionProvider::ltr().show(ui, content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect, Vec2};

    #[test]
    fn defaults_and_nesting() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("direction_nesting_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            assert_eq!(current(ui), Direction::Ltr);
            assert!(!is_rtl(ui));
            DirectionProvider::rtl().show(ui, |ui| {
                assert_eq!(current(ui), Direction::Rtl);
                assert!(is_rtl(ui));
                DirectionProvider::ltr().show(ui, |ui| {
                    assert_eq!(current(ui), Direction::Ltr);
                });
                // Restores the RTL scope after the inner LTR scope pops.
                assert!(is_rtl(ui));
            });
            assert_eq!(current(ui), Direction::Ltr);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn cross_align_and_row_layout_flip() {
        assert_eq!(cross_align(Direction::Ltr), Align::Min);
        assert_eq!(cross_align(Direction::Rtl), Align::Max);
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("direction_layout_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            assert_eq!(row_layout(ui).main_dir, egui::Direction::LeftToRight);
            DirectionProvider::rtl().show(ui, |ui| {
                assert_eq!(row_layout(ui).main_dir, egui::Direction::RightToLeft);
            });
        });
        out.textures_delta.clear();
    }

    #[test]
    fn popup_align_and_chevron_mirror_under_rtl() {
        use egui::emath::RectAlign;
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("direction_popup_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            assert_eq!(
                popup_align(ui, RectAlign::BOTTOM_START, RectAlign::BOTTOM_END),
                RectAlign::BOTTOM_START,
                "LTR keeps the ltr alignment"
            );
            assert_eq!(
                mirrored_chevron(ui, Icon::ChevronRight, Icon::ChevronLeft),
                Icon::ChevronRight,
                "LTR keeps the ltr chevron"
            );
            DirectionProvider::rtl().show(ui, |ui| {
                assert_eq!(
                    popup_align(ui, RectAlign::BOTTOM_START, RectAlign::BOTTOM_END),
                    RectAlign::BOTTOM_END,
                    "RTL flips to the rtl alignment"
                );
                assert_eq!(
                    mirrored_chevron(ui, Icon::ChevronRight, Icon::ChevronLeft),
                    Icon::ChevronLeft,
                    "RTL flips to the rtl chevron"
                );
            });
            // Restoring the LTR scope restores the ltr choices.
            assert_eq!(
                popup_align(ui, RectAlign::BOTTOM_START, RectAlign::BOTTOM_END),
                RectAlign::BOTTOM_START
            );
        });
        out.textures_delta.clear();
    }

    #[test]
    fn rtl_right_aligns_content() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique("direction_align_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            let rtl = DirectionProvider::rtl().show(ui, |ui| {
                ui.label("مرحبا");
            });
            let inner_rect = rtl.response.rect;
            // The labeled content is pushed to the right edge of the 400px
            // wide ui instead of hugging the left edge.
            assert!(inner_rect.right() > 300.0, "right={}", inner_rect.right());
        });
        out.textures_delta.clear();
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("direction_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                DirectionProvider::rtl().show(ui, |ui| {
                    ui.label("النص من اليمين إلى اليسار");
                    ui.with_layout(row_layout(ui), |ui| {
                        ui.label("first");
                        ui.label("second");
                    });
                });
                DirectionProvider::ltr().show(ui, |ui| {
                    ui.label("Left to right");
                });
                Direction::rtl(ui, |ui| {
                    ui.label("convenience rtl");
                });
            });
            out.textures_delta.clear();
        }
    }
}
