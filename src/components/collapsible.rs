//! shadcn `collapsible.tsx` — a disclosure region with an animated height.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/collapsible.tsx`, which is a
//! thin wrapper over the Radix Collapsible primitive: a root holding an
//! arbitrary trigger and a content region that expands and collapses.
//!
//! The open state lives in the egui context keyed by a stable id (pass
//! [`Collapsible::id_salt`] so several collapsibles keep separate state), or
//! can be controlled through a `&mut bool` with [`Collapsible::controlled`].
//! Content height animates with a measured-height clip (`animated_disclosure`),
//! 150ms open / 75ms close — an embellishment with no counterpart in the
//! source, which defines no animation; the timing is the port's own choice,
//! shared with the accordion only through the helper mechanism, not through
//! its 200ms durations.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Collapsible::new().id_salt("repo_list").show(ui, |ui| {
//!     CollapsibleTrigger::new("@peduarte starred 3 repositories").show(ui);
//!     CollapsibleContent::new().show(ui, |ui| {
//!         ui.label("@radix-ui/primitives");
//!         ui.label("@radix-ui/colors");
//!     });
//! });
//!
//! // Controlled:
//! let mut open = false;
//! Collapsible::controlled(&mut open).id_salt("c2").show(ui, |ui| {
//!     CollapsibleTrigger::new("Details").show(ui);
//!     CollapsibleContent::new().show(ui, |ui| { /* … */ });
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use crate::utils::animation;
use egui::{Align2, Color32, FontId, Id, Rect, Response, Sense, Ui, Vec2};

// ---------------------------------------------------------------------------
// Shared state plumbing (scope stack, like button_group.rs)
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_collapsible_stack")
}

fn push_scope(ui: &Ui, state_id: Id, disabled: bool) {
    let ctx = ui.ctx().clone();
    let mut stack: Vec<(Id, bool)> = ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
    stack.push((state_id, disabled));
    ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
}

fn pop_scope(ui: &Ui) {
    let ctx = ui.ctx().clone();
    let mut stack: Vec<(Id, bool)> = ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
    stack.pop();
    ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
}

/// The state id of the innermost open [`Collapsible`], if any.
fn current_scope(ui: &Ui) -> Option<Id> {
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| {
        let stack: Vec<(Id, bool)> = d.get_temp(stack_id())?;
        stack.last().map(|(id, _)| *id)
    })
}

/// Whether the innermost [`Collapsible`] is disabled, if any.
///
/// A trigger toggles its nearest enclosing root (spec §8.2), so that root's
/// `disabled` flag governs it; an inner enabled root inside a disabled
/// outer root keeps its own trigger working (spec §9.8, straightforward
/// reading — S6 verifies the implemented behaviour).
fn current_scope_disabled(ui: &Ui) -> bool {
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| {
        let stack: Vec<(Id, bool)> = d.get_temp(stack_id()).unwrap_or_default();
        stack.last().map(|(_, disabled)| *disabled).unwrap_or(false)
    })
}

fn read_open(ui: &Ui, state_id: Id) -> bool {
    ui.ctx()
        .data_mut(|d| d.get_temp::<bool>(state_id).unwrap_or(false))
}

fn write_open(ui: &Ui, state_id: Id, open: bool) {
    ui.ctx().data_mut(|d| d.insert_temp(state_id, open));
}

// ---------------------------------------------------------------------------
// Collapsible (root)
// ---------------------------------------------------------------------------

/// The shadcn collapsible root. See the [module docs](self) for an example.
///
/// Uncontrolled by default: the open state is stored in the egui context and
/// persisted per `id_salt`. Use [`Collapsible::controlled`] to own the state.
#[derive(Debug)]
pub struct Collapsible<'a> {
    controlled: Option<&'a mut bool>,
    default_open: bool,
    disabled: bool,
    id_salt: Option<String>,
}

impl Default for Collapsible<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Collapsible<'a> {
    /// Creates an uncontrolled collapsible (starts closed).
    pub fn new() -> Self {
        Self {
            controlled: None,
            default_open: false,
            disabled: false,
            id_salt: None,
        }
    }

    /// Creates a collapsible whose open state is owned by the caller.
    /// The value is read before and written back after each frame.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            default_open: false,
            disabled: false,
            id_salt: None,
        }
    }

    /// Initial open state (uncontrolled mode only).
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    /// Disables the collapsible (spec §4.4, N4): the trigger cannot toggle
    /// and the content cannot be opened by any input path the port exposes.
    /// A disabled root does NOT force content closed — `default_open(true)`
    /// or a controlled `true` still renders the content open (straightforward
    /// reading, spec §9.8; the fine Radix semantics are OPEN-1).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt so the open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the collapsible and lays out its trigger(s) and content.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_collapsible"),
        };

        // Seed the state; controlled mode wins over anything stored.
        if let Some(open) = self.controlled.as_deref() {
            write_open(ui, state_id, *open);
        } else {
            let exists = ui
                .ctx()
                .data_mut(|d| d.get_temp::<bool>(state_id).is_some());
            if !exists {
                write_open(ui, state_id, self.default_open);
            }
        }

        push_scope(ui, state_id, self.disabled);
        content(ui);
        pop_scope(ui);

        // Controlled write-back: pick up toggles made by the trigger.
        if let Some(out) = self.controlled {
            *out = read_open(ui, state_id);
        }
    }
}

// ---------------------------------------------------------------------------
// CollapsibleTrigger
// ---------------------------------------------------------------------------

/// A button that toggles the innermost [`Collapsible`].
///
/// Styled like a ghost button (the source trigger is an unstyled primitive
/// that demos typically wrap in a Button); Enter/Space activate when focused.
#[derive(Debug)]
pub struct CollapsibleTrigger {
    label: String,
    icon_start: Option<Icon>,
    icon_end: Option<Icon>,
    disabled: bool,
    id_salt: Option<String>,
}

impl CollapsibleTrigger {
    /// Creates a trigger with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon_start: None,
            icon_end: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Shows an icon before the label.
    pub fn icon_start(mut self, icon: Icon) -> Self {
        self.icon_start = Some(icon);
        self
    }

    /// Shows an icon after the label.
    pub fn icon_end(mut self, icon: Icon) -> Self {
        self.icon_end = Some(icon);
        self
    }

    /// Disables the trigger (`disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt for focus persistence.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the trigger. Returns the response (`clicked()` toggled the
    /// collapsible; `changed()` is true whenever the open state flipped).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let state_id = current_scope(ui);
        let open = state_id.map(|id| read_open(ui, id)).unwrap_or(false);

        let height = crate::theme::tokens::spacing::CONTROL_SM;
        // C3 (spec §9.1): an icon-only trigger — empty label, exactly one
        // icon — is the `size="icon"` (`size-8`) 32px square, icon centred.
        // The label padding collapses to 0 (like Button's icon sizes, whose
        // `padding_x` is 0); the width is the 32px box, not the 16px the
        // content formula alone would give (`pad_x` 0 + 16px icon).
        let icon_only =
            self.label.is_empty() && (self.icon_start.is_some() ^ self.icon_end.is_some());
        let pad_x = if icon_only {
            0.0
        } else {
            crate::theme::tokens::spacing::MD
        };
        let gap = crate::theme::tokens::spacing::XS_HALF;
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        // Placeholder color: the paint-time fallback replaces it exactly,
        // so this one galley serves both sizing and painting.
        let galley = if self.label.is_empty() {
            None
        } else {
            Some(
                ui.painter()
                    .layout_no_wrap(self.label.clone(), font, Color32::PLACEHOLDER),
            )
        };
        let text_w = galley.as_ref().map_or(0.0, |g| g.size().x);
        let icon_size = crate::theme::tokens::spacing::LG;
        let has_icon = self.icon_start.is_some() || self.icon_end.is_some();
        let n_children = self.icon_start.is_some() as i32
            + self.icon_end.is_some() as i32
            + (!self.label.is_empty()) as i32;
        let gaps = (n_children - 1).max(0) as f32 * gap;
        let mut width = pad_x * 2.0 + icon_size * has_icon as i32 as f32 + gaps + text_w;
        if icon_only {
            width = height; // `size-8` square
        }
        let desired = Vec2::new(width, height);

        // Root-level `disabled` (N4) disables the trigger together with the
        // content; trigger-level `disabled` disables this trigger only.
        let disabled = self.disabled || current_scope_disabled(ui);
        let sense = if disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_collapsible_trigger"),
        };

        let (_, rect) = ui.allocate_space(desired);
        let mut response = ui.interact(rect, id, sense);

        let clicked = response.clicked() && !disabled;
        if clicked {
            if let Some(state_id) = state_id {
                write_open(ui, state_id, !open);
                response.mark_changed();
            }
        }

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() && !disabled;
            let corner = theme.radius.md_u8();
            let painter = ui.painter();

            let mut fill = if hovered {
                if dark {
                    color_alpha(p.accent, 0.5)
                } else {
                    p.accent
                }
            } else {
                Color32::TRANSPARENT
            };
            let mut fg = p.foreground;
            if disabled {
                fill = color_alpha(fill, 0.5);
                fg = color_alpha(fg, 0.5);
            }

            painter.rect_filled(rect, corner, fill);

            let total_w = icon_size * has_icon as i32 as f32 + gaps + text_w;
            let center = rect.center();
            let mut cursor = center.x - total_w * 0.5;
            if let Some(icon) = self.icon_start {
                let r = egui::Rect::from_center_size(
                    egui::Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, fg, 2.0 * icon_size / 24.0);
                cursor += icon_size + gap;
            }
            if let Some(galley) = galley {
                // Same paint math as `Painter::text`, reusing the galley.
                let text_rect = Align2::LEFT_CENTER.anchor_size(
                    egui::Pos2::new(cursor + text_w * 0.5, center.y),
                    galley.size(),
                );
                painter.galley(text_rect.min, galley, fg);
                cursor += text_w + gap;
            }
            if let Some(icon) = self.icon_end {
                let r = egui::Rect::from_center_size(
                    egui::Pos2::new(cursor + icon_size * 0.5, center.y),
                    Vec2::splat(icon_size),
                );
                icon.paint(painter, r, fg, 2.0 * icon_size / 24.0);
            }

            // `focus-visible:ring-[3px] focus-visible:ring-ring/50`
            if response.has_focus() && !disabled {
                crate::utils::focus::paint_ring(
                    painter,
                    rect.expand2(egui::vec2(1.0, 1.0)),
                    color_alpha(p.ring, 0.5),
                    3.0,
                    corner as f32,
                );
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// CollapsibleContent
// ---------------------------------------------------------------------------

/// The animated content region of a [`Collapsible`].
///
/// The content height is measured while laid out and animated from zero with
/// [`animated_disclosure`], clipped to the visible area.
#[derive(Clone, Debug, Default)]
pub struct CollapsibleContent {
    _private: (),
}

impl CollapsibleContent {
    /// Creates the content region.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the collapsible content with an animated height.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let Some(state_id) = current_scope(ui) else {
            // Outside a Collapsible: render statically so mistakes stay visible.
            content(ui);
            return;
        };
        let open = read_open(ui, state_id);
        let anim_id = state_id.with("content_anim");
        animated_disclosure(
            ui,
            anim_id,
            open,
            animation::Duration::Default,
            animation::Duration::Fast,
            content,
        );
    }
}

// ---------------------------------------------------------------------------
// Shared animated-disclosure helper (also used by accordion.rs)
// ---------------------------------------------------------------------------

/// Lays out `content` and reveals it with an animated height clip.
///
/// The full content height is measured from the laid-out child (clipping does
/// not affect layout) and cached per `anim_id`, so the animation converges
/// after content changes. `open_dur` / `close_dur` set the reveal and collapse
/// timing — the accordion passes 200ms in both directions
/// (`animate-accordion-down/up`), while the collapsible keeps the generic
/// open/close timing.
pub(crate) fn animated_disclosure(
    ui: &mut Ui,
    anim_id: Id,
    open: bool,
    open_dur: animation::Duration,
    close_dur: animation::Duration,
    content: impl FnOnce(&mut Ui),
) {
    let ctx = ui.ctx().clone();
    let target = if open { 1.0 } else { 0.0 };
    let progress = animation::animate_value(
        &ctx,
        anim_id,
        target,
        if open { open_dur } else { close_dur },
    );
    if !open && progress <= 0.0 {
        return; // fully closed: paint and lay out nothing
    }

    let full_width = ui.available_width();
    let height_key = anim_id.with("full_height");
    let full_height: f32 = ctx.data_mut(|d| d.get_temp(height_key).unwrap_or(0.0));
    let visible = (full_height * progress).max(0.0);

    let cursor_min = ui.cursor().min;
    // Lay the content out at full height; only the top `visible` points are
    // painted thanks to the clip rect.
    let mut child = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(
        cursor_min,
        Vec2::new(full_width, f32::INFINITY),
    )));
    child.set_clip_rect(Rect::from_min_size(
        cursor_min,
        Vec2::new(full_width, visible),
    ));
    content(&mut child);
    let measured = child.min_rect().height();
    ctx.data_mut(|d| d.insert_temp(height_key, measured));

    // Reserve the animated height in the parent layout.
    if visible > 0.0 {
        ui.allocate_space(Vec2::new(full_width, visible));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    /// Advances time so `animate_value_with_time` makes progress across frames.
    struct Harness {
        ctx: egui::Context,
        time: f64,
    }

    impl Harness {
        fn new() -> Self {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            Self { ctx, time: 0.0 }
        }

        fn run(&mut self, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("collapsible_test_ui"),
                    egui::UiBuilder::new().max_rect(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        Vec2::new(400.0, 400.0),
                    )),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        /// Presses and releases the primary button at `pos` over two frames,
        /// rendering the same UI in both so the widget is registered while
        /// the pointer is down.
        fn click(&mut self, pos: egui::Pos2, mut f: impl FnMut(&mut Ui)) {
            for pressed in [true, false] {
                self.time += 0.1;
                let input = egui::RawInput {
                    time: Some(self.time),
                    events: vec![egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    ..Default::default()
                };
                let mut out = self.ctx.run_ui(input, |ctx| {
                    let ui = &mut egui::Ui::new(
                        ctx.clone(),
                        Id::unique("collapsible_test_ui"),
                        egui::UiBuilder::new().max_rect(Rect::from_min_size(
                            egui::Pos2::ZERO,
                            Vec2::new(400.0, 400.0),
                        )),
                    );
                    f(ui);
                });
                out.textures_delta.clear();
            }
        }

        fn open(&self, salt: &str) -> bool {
            let id = Id::unique("collapsible_test_ui").with(salt);
            self.ctx
                .data_mut(|d| d.get_temp::<bool>(id).unwrap_or(false))
        }

        fn set_open(&self, salt: &str, open: bool) {
            let id = Id::unique("collapsible_test_ui").with(salt);
            self.ctx.data_mut(|d| d.insert_temp(id, open));
        }
    }

    #[test]
    fn uncontrolled_state_persists() {
        let mut h = Harness::new();
        h.run(|ui| {
            Collapsible::new().id_salt("t1").show(ui, |ui| {
                CollapsibleContent::new().show(ui, |ui| {
                    ui.label("hello");
                });
            });
        });
        assert!(!h.open("t1"), "default is closed");

        h.set_open("t1", true);
        h.run(|ui| {
            Collapsible::new().id_salt("t1").show(ui, |ui| {
                CollapsibleContent::new().show(ui, |ui| {
                    ui.label("hello");
                });
            });
        });
        assert!(h.open("t1"), "state persists in ctx data per id_salt");
    }

    #[test]
    fn trigger_click_toggles_state() {
        let mut h = Harness::new();

        // Lay out to find the trigger rect.
        let mut trigger_rect = Rect::ZERO;
        h.run(|ui| {
            Collapsible::new().id_salt("clicky").show(ui, |ui| {
                trigger_rect = CollapsibleTrigger::new("Toggle").show(ui).rect;
            });
        });
        assert!(!h.open("clicky"));

        h.click(trigger_rect.center(), |ui| {
            Collapsible::new().id_salt("clicky").show(ui, |ui| {
                CollapsibleTrigger::new("Toggle").show(ui);
            });
        });
        assert!(h.open("clicky"), "clicking the trigger must open it");

        h.click(trigger_rect.center(), |ui| {
            Collapsible::new().id_salt("clicky").show(ui, |ui| {
                CollapsibleTrigger::new("Toggle").show(ui);
            });
        });
        assert!(!h.open("clicky"), "clicking again must close it");
    }

    #[test]
    fn controlled_mode_follows_caller_and_clicks() {
        let mut h = Harness::new();
        let mut open = false;

        let mut trigger_rect = Rect::ZERO;
        h.run(|ui| {
            Collapsible::controlled(&mut open)
                .id_salt("c1")
                .show(ui, |ui| {
                    trigger_rect = CollapsibleTrigger::new("Toggle").show(ui).rect;
                });
        });
        assert!(!open, "unchanged when nothing toggles");

        h.click(trigger_rect.center(), |ui| {
            Collapsible::controlled(&mut open)
                .id_salt("c1")
                .show(ui, |ui| {
                    CollapsibleTrigger::new("Toggle").show(ui);
                });
        });
        assert!(open, "click write-back reaches the controlled bool");
    }

    #[test]
    fn closed_content_lays_out_nothing_then_animates_open() {
        let mut h = Harness::new();
        h.set_open("closed", false);
        let mut bottom = 0.0;
        h.run(|ui| {
            Collapsible::new().id_salt("closed").show(ui, |ui| {
                CollapsibleContent::new().show(ui, |ui| {
                    ui.label("body");
                });
                bottom = ui.cursor().min.y;
            });
        });
        assert!(bottom.abs() < 1.0, "closed content must not reserve space");

        h.set_open("closed", true);
        for _ in 0..4 {
            h.run(|ui| {
                Collapsible::new().id_salt("closed").show(ui, |ui| {
                    CollapsibleContent::new().show(ui, |ui| {
                        ui.label("body");
                    });
                    bottom = ui.cursor().min.y;
                });
            });
        }
        assert!(bottom > 0.0, "open content must reserve space");
    }

    #[test]
    fn default_open_shows_content_on_first_frame() {
        let mut h = Harness::new();
        let mut bottom = 0.0;
        // Frame 1 measures the content height; frame 2 shows it in full.
        for _ in 0..2 {
            h.run(|ui| {
                Collapsible::new()
                    .id_salt("preopen")
                    .default_open(true)
                    .show(ui, |ui| {
                        CollapsibleContent::new().show(ui, |ui| {
                            ui.label("body");
                        });
                        bottom = ui.cursor().min.y;
                    });
            });
        }
        assert!(bottom > 0.0, "an initially open collapsible shows at once");
    }

    #[test]
    fn trigger_outside_collapsible_is_inert() {
        let mut h = Harness::new();
        let mut changed = false;
        h.run(|ui| {
            changed = CollapsibleTrigger::new("lonely").show(ui).changed();
        });
        assert!(!changed);
    }

    #[test]
    fn builders_keep_fields() {
        let mut open = false;
        let c = Collapsible::controlled(&mut open)
            .default_open(true)
            .disabled(true)
            .id_salt("x");
        assert_eq!(c.id_salt.as_deref(), Some("x"));
        assert!(c.default_open);
        assert!(c.disabled);
        assert!(!Collapsible::new().disabled, "root starts enabled");
        let t = CollapsibleTrigger::new("a")
            .icon_start(Icon::ChevronDown)
            .icon_end(Icon::ChevronUp)
            .disabled(true)
            .id_salt("t");
        assert!(t.disabled);
        assert!(t.icon_start.is_some());
        assert_eq!(t.id_salt.as_deref(), Some("t"));
        let _ = CollapsibleContent::new();
    }

    #[test]
    fn root_disabled_blocks_trigger_toggle() {
        let mut h = Harness::new();

        // Lay out to find the trigger rect.
        let mut trigger_rect = Rect::ZERO;
        let mut bottom = 0.0;
        h.run(|ui| {
            Collapsible::new()
                .id_salt("root-dis")
                .disabled(true)
                .show(ui, |ui| {
                    trigger_rect = CollapsibleTrigger::new("Toggle").show(ui).rect;
                    // The trigger itself is 32px; only the content's own
                    // height counts towards "lays out nothing".
                    let before = ui.cursor().min.y;
                    CollapsibleContent::new().show(ui, |ui| {
                        ui.label("body");
                    });
                    bottom = ui.cursor().min.y - before;
                });
        });
        assert!(!h.open("root-dis"), "disabled root starts closed");
        assert!(bottom.abs() < 1.0, "content lays out nothing while closed");

        // Clicking the trigger must not open it, and the response reports
        // no click.
        let mut clicked = true;
        h.click(trigger_rect.center(), |ui| {
            Collapsible::new()
                .id_salt("root-dis")
                .disabled(true)
                .show(ui, |ui| {
                    clicked = CollapsibleTrigger::new("Toggle").show(ui).clicked();
                    let before = ui.cursor().min.y;
                    CollapsibleContent::new().show(ui, |ui| {
                        ui.label("body");
                    });
                    bottom = ui.cursor().min.y - before;
                });
        });
        assert!(!clicked, "disabled trigger reports no click");
        assert!(!h.open("root-dis"), "root-disabled trigger must not toggle");
        assert!(bottom.abs() < 1.0, "content cannot be opened by a click");
    }

    #[test]
    fn root_disabled_does_not_force_content_closed() {
        // Straightforward reading (spec §9.8): disabled blocks interaction;
        // it does not force content closed.
        let mut h = Harness::new();
        let mut bottom = 0.0;
        for _ in 0..2 {
            h.run(|ui| {
                Collapsible::new()
                    .id_salt("preopen-dis")
                    .default_open(true)
                    .disabled(true)
                    .show(ui, |ui| {
                        CollapsibleContent::new().show(ui, |ui| {
                            ui.label("body");
                        });
                        bottom = ui.cursor().min.y;
                    });
            });
        }
        assert!(bottom > 0.0, "disabled root renders default-open content");

        // Clicking the trigger must not close it.
        let mut trigger_rect = Rect::ZERO;
        h.run(|ui| {
            Collapsible::new()
                .id_salt("preopen-dis")
                .disabled(true)
                .show(ui, |ui| {
                    trigger_rect = CollapsibleTrigger::new("Toggle").show(ui).rect;
                    CollapsibleContent::new().show(ui, |ui| {
                        ui.label("body");
                    });
                });
        });
        h.click(trigger_rect.center(), |ui| {
            Collapsible::new()
                .id_salt("preopen-dis")
                .disabled(true)
                .show(ui, |ui| {
                    CollapsibleTrigger::new("Toggle").show(ui);
                    CollapsibleContent::new().show(ui, |ui| {
                        ui.label("body");
                    });
                });
        });
        assert!(
            h.open("preopen-dis"),
            "click must not close a disabled root"
        );
    }

    #[test]
    fn root_disabled_controlled_bool_can_open() {
        let mut h = Harness::new();
        let mut open = true;
        let mut bottom = 0.0;
        let mut trigger_rect = Rect::ZERO;
        h.run(|ui| {
            Collapsible::controlled(&mut open)
                .id_salt("cc1")
                .disabled(true)
                .show(ui, |ui| {
                    trigger_rect = CollapsibleTrigger::new("Toggle").show(ui).rect;
                    CollapsibleContent::new().show(ui, |ui| {
                        ui.label("body");
                    });
                    bottom = ui.cursor().min.y;
                });
        });
        assert!(open, "controlled bool is read before the frame");
        assert!(
            bottom > 0.0,
            "controlled true renders content under a disabled root"
        );

        h.click(trigger_rect.center(), |ui| {
            Collapsible::controlled(&mut open)
                .id_salt("cc1")
                .disabled(true)
                .show(ui, |ui| {
                    CollapsibleTrigger::new("Toggle").show(ui);
                });
        });
        assert!(open, "click must not write back through a disabled root");
    }

    #[test]
    fn trigger_geometry_matches_ghost_stand_in() {
        let mut h = Harness::new();

        // Empty trigger (no label, no icon): width is pad_x * 2 = 24px.
        let mut empty_rect = Rect::ZERO;
        h.run(|ui| {
            Collapsible::new().id_salt("g0").show(ui, |ui| {
                empty_rect = CollapsibleTrigger::new("").show(ui).rect;
            });
        });
        assert!(
            (empty_rect.height() - 32.0).abs() < 0.01,
            "trigger height is 32px, got {}",
            empty_rect.height()
        );
        assert!(
            (empty_rect.width() - 24.0).abs() < 0.01,
            "pad_x is 12px each side, got width {}",
            empty_rect.width()
        );

        // Labelled trigger with one icon: 24 + 16 + 6 + text width.
        let mut rect = Rect::ZERO;
        let mut text_w = 0.0;
        h.run(|ui| {
            Collapsible::new().id_salt("g1").show(ui, |ui| {
                rect = CollapsibleTrigger::new("Toggle")
                    .icon_end(Icon::ChevronDown)
                    .show(ui)
                    .rect;
            });
            // Same layout the trigger uses, to pin the formula constants.
            let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
            text_w = ui
                .painter()
                .layout_no_wrap("Toggle".to_owned(), font, Color32::PLACEHOLDER)
                .size()
                .x;
        });
        assert!(
            (rect.height() - 32.0).abs() < 0.01,
            "trigger height is 32px, got {}",
            rect.height()
        );
        let expected_w = 24.0 + 16.0 + 6.0 + text_w;
        assert!(
            (rect.width() - expected_w).abs() < 0.01,
            "labelled trigger width is pad_x*2 + icon 16 + gap 6 + text: got {}, expected {expected_w}",
            rect.width()
        );

        // Icon-only trigger (C3): empty label + exactly one icon is the
        // `size="icon"` 32px square.
        let mut icon_rect = Rect::ZERO;
        h.run(|ui| {
            Collapsible::new().id_salt("g2").show(ui, |ui| {
                icon_rect = CollapsibleTrigger::new("")
                    .icon_start(Icon::ChevronsUpDown)
                    .show(ui)
                    .rect;
            });
        });
        assert!(
            (icon_rect.width() - 32.0).abs() < 0.01,
            "icon-only trigger is 32px wide, got {}",
            icon_rect.width()
        );
        assert!(
            (icon_rect.height() - 32.0).abs() < 0.01,
            "icon-only trigger is 32px tall, got {}",
            icon_rect.height()
        );
    }

    #[test]
    fn content_animates_150ms_open_75ms_close() {
        // The harness advances 0.1s per frame, which separates the two
        // durations: closing (75ms) finishes within two frames of the
        // flip, opening (150ms) is still part-way there.
        let mut h = Harness::new();
        let mut bottom = 0.0;
        let content = |ui: &mut Ui, bottom: &mut f32| {
            Collapsible::new().id_salt("dur").show(ui, |ui| {
                CollapsibleContent::new().show(ui, |ui| {
                    ui.label("body");
                });
                *bottom = ui.cursor().min.y;
            });
        };

        // Settle open and measure the full height.
        h.set_open("dur", true);
        for _ in 0..4 {
            h.run(|ui| content(ui, &mut bottom));
        }
        let full = bottom;
        assert!(full > 0.0, "open content has height");

        // Close: 75ms < one 0.1s step, so two frames after the flip the
        // content is gone.
        h.set_open("dur", false);
        h.run(|ui| content(ui, &mut bottom));
        h.run(|ui| content(ui, &mut bottom));
        assert!(
            bottom.abs() < 1.0,
            "75ms close finishes within two 0.1s frames, bottom = {bottom}"
        );

        // Open: 150ms > one 0.1s step, so two frames after the flip the
        // content is only part-way up.
        h.set_open("dur", true);
        h.run(|ui| content(ui, &mut bottom));
        h.run(|ui| content(ui, &mut bottom));
        assert!(
            bottom > 0.0 && bottom < full,
            "150ms open is part-way after two 0.1s frames: {bottom} of {full}"
        );

        // Two more frames settle it at full height.
        h.run(|ui| content(ui, &mut bottom));
        h.run(|ui| content(ui, &mut bottom));
        assert!(
            (bottom - full).abs() < 1.0,
            "open settles at full height: {bottom} vs {full}"
        );
    }

    #[test]
    fn nested_collapsible_inner_trigger_toggles_inner_only() {
        let mut h = Harness::new();
        // Open the outer first: the inner trigger only lays out inside the
        // outer content (spec §9.5).
        h.set_open("outer", true);
        let mut inner_rect = Rect::ZERO;
        // The inner root lives inside the content's child Ui, whose scope id
        // is the outer scope with `"child"` appended (egui `Ui::new_child`),
        // so its state key is not `h.open("inner")`'s top-level key. Observe
        // the inner state through a controlled bool instead: the write-back
        // reads the same key the trigger wrote.
        let mut inner_open = false;
        let render = |ui: &mut Ui, inner_rect: &mut Rect, inner_open: &mut bool| {
            Collapsible::new().id_salt("outer").show(ui, |ui| {
                CollapsibleTrigger::new("Outer").show(ui);
                CollapsibleContent::new().show(ui, |ui| {
                    Collapsible::controlled(inner_open)
                        .id_salt("inner")
                        .show(ui, |ui| {
                            *inner_rect = CollapsibleTrigger::new("Inner").show(ui).rect;
                        });
                });
            });
        };
        for _ in 0..2 {
            h.run(|ui| render(ui, &mut inner_rect, &mut inner_open));
        }
        assert!(!inner_open, "inner starts closed");
        assert!(h.open("outer"), "outer is open");

        h.click(inner_rect.center(), |ui| {
            render(ui, &mut inner_rect, &mut inner_open)
        });
        assert!(inner_open, "inner trigger toggles the inner root");
        assert!(h.open("outer"), "outer stays open");

        h.click(inner_rect.center(), |ui| {
            render(ui, &mut inner_rect, &mut inner_open)
        });
        assert!(!inner_open, "inner trigger toggles the inner root closed");
        assert!(h.open("outer"), "outer still open");
    }

    #[test]
    fn mode_presets_build() {
        // Light and dark palettes both resolve (used by the trigger).
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
