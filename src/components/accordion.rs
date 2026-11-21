//! shadcn `accordion.tsx` — vertically stacked collapsible sections.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/accordion.tsx`:
//! items carry a bottom border (`border-b last:border-b-0`), the trigger is a
//! full-width row (`py-4 text-sm font-medium` — nova uses `rounded-lg
//! py-2.5`) with a `ChevronDown` icon that rotates 180° when open
//! (`[&[data-state=open]>svg]:rotate-180`), and the content animates
//! open/closed (`animate-accordion-down/up`; nova inner `pt-0 pb-2.5`).
//!
//! Radix supports `type="single"` and `type="multiple"`. Use
//! [`Accordion::new`] for an uncontrolled single accordion,
//! [`Accordion::single`] / [`Accordion::multiple`] for controlled state.
//!
//! Keyboard (Radix roving focus): ArrowDown/ArrowRight move focus to the
//! next trigger, ArrowUp/ArrowLeft to the previous one, and Home/End jump to
//! the first/last trigger; Enter/Space toggle the focused trigger (egui's
//! keyboard activation for click widgets). Like Radix, moving focus does not
//! itself expand the item — the user presses Enter/Space to toggle.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Accordion::new().id_salt("acc").default_open(0).show(ui, |ui| {
//!     AccordionItem::new().show(ui, |ui| {
//!         AccordionTrigger::new("Product Information").show(ui);
//!         AccordionContent::new().show(ui, |ui| {
//!             ui.label("Our flagship product combines cutting-edge technology.");
//!         });
//!     });
//!     AccordionItem::new().show(ui, |ui| {
//!         AccordionTrigger::new("Shipping Details").show(ui);
//!         AccordionContent::new().show(ui, |ui| { /* … */ });
//!     });
//! });
//!
//! let mut open = Some(0usize);
//! Accordion::single(&mut open).id_salt("acc2").show(ui, |ui| { /* … */ });
//! let mut many = vec![true, false];
//! Accordion::multiple(&mut many).id_salt("acc3").show(ui, |ui| { /* … */ });
//! Accordion::new_multiple().id_salt("acc4").default_open_many([0, 2]).show(ui, |ui| { /* … */ });
//! ```

use crate::theme::{color_alpha, ShadcnTheme};
use crate::utils::animation::{self, Duration};
use egui::{Align2, Color32, FontId, Id, Key, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Radix `type="single" | "multiple"`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AccordionType {
    /// One open item at a time.
    #[default]
    Single,
    /// Any number of open items.
    Multiple,
}

/// Height of one trigger row (nova `py-2.5` + the 20px `text-sm` line box).
pub const ACCORDION_TRIGGER_HEIGHT: f32 = 40.0;
/// Vertical padding of the trigger row (nova `py-2.5` = 10px top + 10px
/// bottom). N7: nova geometry is hardcoded; the base `py-4` 16px is a
/// Phase 4 item, not this line.
const TRIGGER_PAD_Y: f32 = 10.0;
/// Minimum gap between the label and the chevron (`gap-4`, spec §3.4 N6).
const TRIGGER_LABEL_GAP: f32 = 16.0;
/// Bottom padding of the content (nova `pb-2.5`).
const CONTENT_PAD_BOTTOM: f32 = 10.0;

// ---------------------------------------------------------------------------
// Scope plumbing: root stack, item stack, per-accordion item counter
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct AccordionRoot {
    state_id: Id,
    kind: AccordionType,
    collapsible: bool,
}

#[derive(Clone, Copy, Debug)]
struct AccordionItemScope {
    state_id: Id,
    kind: AccordionType,
    collapsible: bool,
    index: usize,
}

#[derive(Clone, Copy, Debug)]
struct ItemCounter {
    frame: u64,
    drawn: usize,
}

fn root_stack_id() -> Id {
    Id::unique("shadcn_accordion_root_stack")
}

fn item_stack_id() -> Id {
    Id::unique("shadcn_accordion_item_stack")
}

fn peek<T: Clone + 'static>(ui: &Ui, id: Id) -> Option<T> {
    ui.ctx().data_mut(|d| d.get_temp::<T>(id))
}

fn root_scope(ui: &Ui) -> Option<AccordionRoot> {
    peek::<Vec<AccordionRoot>>(ui, root_stack_id()).and_then(|stack| stack.last().copied())
}

fn item_scope(ui: &Ui) -> Option<AccordionItemScope> {
    peek::<Vec<AccordionItemScope>>(ui, item_stack_id()).and_then(|stack| stack.last().copied())
}

// State accessors ------------------------------------------------------------

fn read_single(ui: &Ui, state_id: Id) -> Option<usize> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Option<usize>>(state_id).flatten())
}

fn write_single(ui: &Ui, state_id: Id, value: Option<usize>) {
    ui.ctx().data_mut(|d| d.insert_temp(state_id, value));
}

fn read_multi(ui: &Ui, state_id: Id) -> Vec<bool> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<bool>>(state_id).unwrap_or_default())
}

fn write_multi(ui: &Ui, state_id: Id, value: Vec<bool>) {
    ui.ctx().data_mut(|d| d.insert_temp(state_id, value));
}

/// Pure toggle logic shared by the click handler and tests.
fn next_single(open: Option<usize>, index: usize, collapsible: bool) -> Option<usize> {
    if open == Some(index) {
        if collapsible {
            None
        } else {
            open
        }
    } else {
        Some(index)
    }
}

/// Pure toggle logic for `type="multiple"` (resize handled by the caller).
fn next_multiple(open: &[bool], index: usize) -> Vec<bool> {
    let mut out = open.to_vec();
    if out.len() <= index {
        out.resize(index + 1, false);
    }
    out[index] = !out[index];
    out
}

fn is_open(ui: &Ui, scope: &AccordionItemScope) -> bool {
    match scope.kind {
        AccordionType::Single => read_single(ui, scope.state_id) == Some(scope.index),
        AccordionType::Multiple => read_multi(ui, scope.state_id)
            .get(scope.index)
            .copied()
            .unwrap_or(false),
    }
}

fn toggle(ui: &Ui, scope: &AccordionItemScope) {
    match scope.kind {
        AccordionType::Single => {
            let next = next_single(
                read_single(ui, scope.state_id),
                scope.index,
                scope.collapsible,
            );
            write_single(ui, scope.state_id, next);
        }
        AccordionType::Multiple => {
            let next = next_multiple(&read_multi(ui, scope.state_id), scope.index);
            write_multi(ui, scope.state_id, next);
        }
    }
}

// ---------------------------------------------------------------------------
// Accordion (root)
// ---------------------------------------------------------------------------

/// The shadcn accordion root. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Accordion<'a> {
    controlled_single: Option<&'a mut Option<usize>>,
    controlled_multiple: Option<&'a mut Vec<bool>>,
    kind: AccordionType,
    collapsible: bool,
    default_open: Option<usize>,
    default_open_many: Vec<usize>,
    id_salt: Option<String>,
}

impl Default for Accordion<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Accordion<'a> {
    /// Creates an uncontrolled single accordion (nothing open by default).
    pub fn new() -> Self {
        Self {
            controlled_single: None,
            controlled_multiple: None,
            kind: AccordionType::Single,
            collapsible: false,
            default_open: None,
            default_open_many: Vec::new(),
            id_salt: None,
        }
    }

    /// Creates an uncontrolled multiple accordion (nothing open by default;
    /// use [`.default_open_many`](Self::default_open_many) for a
    /// `defaultValue` array, spec §8.3 N3). There was no uncontrolled
    /// multiple entry point before S1 — `multiple` is controlled-only.
    pub fn new_multiple() -> Self {
        Self {
            kind: AccordionType::Multiple,
            ..Self::new()
        }
    }

    /// Creates a single accordion controlled through the given value.
    pub fn single(open: &'a mut Option<usize>) -> Self {
        Self {
            controlled_single: Some(open),
            ..Self::new()
        }
    }

    /// Creates a multiple accordion controlled through the given flags
    /// (one per item, in show order).
    pub fn multiple(open: &'a mut Vec<bool>) -> Self {
        Self {
            controlled_multiple: Some(open),
            kind: AccordionType::Multiple,
            ..Self::new()
        }
    }

    /// Radix `collapsible` (single mode): whether the open item can be
    /// closed by clicking its trigger. Defaults to `false`, like Radix; pass
    /// `.collapsible(true)` for the docs-demo behavior.
    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Item shown open on the first frame (uncontrolled single mode;
    /// Radix `defaultValue`).
    pub fn default_open(mut self, index: usize) -> Self {
        self.default_open = Some(index);
        self
    }

    /// Items shown open on the first frame (uncontrolled multiple mode;
    /// Radix `defaultValue` array for `type="multiple"`, spec §8.3 N3).
    /// One index per initially-open item; the set is applied on the first
    /// frame only, exactly like controlled multiple initialised with the
    /// same flags. Applies to uncontrolled multiple accordions created
    /// with [`Self::new_multiple`]; ignored in single mode (use
    /// [`Self::default_open`]) and when controlled state is supplied
    /// (controlled state wins each frame, spec §9.9).
    pub fn default_open_many(mut self, indices: impl IntoIterator<Item = usize>) -> Self {
        self.default_open_many = indices.into_iter().collect();
        self
    }

    /// Sets a stable id salt so the open state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the accordion and lays out its items.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_accordion"),
        };

        // Seed state; controlled values win each frame.
        if let Some(v) = self.controlled_single.as_deref() {
            write_single(ui, state_id, *v);
        } else if let Some(v) = self.controlled_multiple.as_deref() {
            write_multi(ui, state_id, v.clone());
        } else if !ui
            .ctx()
            .data_mut(|d| d.get_temp::<Option<usize>>(state_id).is_some())
            && self.kind == AccordionType::Single
        {
            write_single(ui, state_id, self.default_open);
        } else if self.kind == AccordionType::Multiple
            && !ui
                .ctx()
                .data_mut(|d| d.get_temp::<Vec<bool>>(state_id).is_some())
        {
            // N3 (spec §8.3): seed uncontrolled multiple from
            // `default_open_many` — the `defaultValue` array. The flags
            // are sized to the largest requested index; items past the
            // rendered count simply stay unread.
            let mut flags = vec![false; self.default_open_many.iter().max().map_or(0, |m| m + 1)];
            for i in &self.default_open_many {
                flags[*i] = true;
            }
            write_multi(ui, state_id, flags);
        }

        let root = AccordionRoot {
            state_id,
            kind: self.kind,
            collapsible: self.collapsible,
        };
        ui.ctx().data_mut(|d| {
            let mut stack: Vec<AccordionRoot> = d.get_temp(root_stack_id()).unwrap_or_default();
            stack.push(root);
            d.insert_temp(root_stack_id(), stack);
        });

        content(ui);

        ui.ctx().data_mut(|d| {
            let mut stack: Vec<AccordionRoot> = d.get_temp(root_stack_id()).unwrap_or_default();
            stack.pop();
            d.insert_temp(root_stack_id(), stack);
        });

        // Publish the item count so items know who is last (`last:border-b-0`).
        let counter_id = state_id.with("item_counter");
        let drawn = ui
            .ctx()
            .data_mut(|d| d.get_temp::<ItemCounter>(counter_id).map(|c| c.drawn))
            .unwrap_or(0);
        ui.ctx()
            .data_mut(|d| d.insert_temp(state_id.with("item_total"), drawn));

        // Controlled write-back.
        if let Some(out) = self.controlled_single {
            *out = read_single(ui, state_id);
        }
        if let Some(out) = self.controlled_multiple {
            *out = read_multi(ui, state_id);
        }
    }
}

// ---------------------------------------------------------------------------
// AccordionItem
// ---------------------------------------------------------------------------

/// One accordion section: `border-b last:border-b-0` wrapper.
///
/// Tracks its index inside the enclosing [`Accordion`] automatically.
#[derive(Clone, Debug, Default)]
pub struct AccordionItem {
    _private: (),
}

impl AccordionItem {
    /// Creates an accordion item.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Lays out the item's trigger and content, drawing the separator border
    /// unless this is the last item.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        let Some(root) = root_scope(ui) else {
            content(ui);
            return;
        };

        // Assign this item its per-frame index.
        let frame = ui.ctx().cumulative_frame_nr();
        let counter_id = root.state_id.with("item_counter");
        let mut counter = ui
            .ctx()
            .data_mut(|d| d.get_temp::<ItemCounter>(counter_id))
            .filter(|c| c.frame == frame)
            .unwrap_or(ItemCounter { frame, drawn: 0 });
        let index = counter.drawn;
        counter.drawn += 1;
        ui.ctx().data_mut(|d| d.insert_temp(counter_id, counter));

        // `last:border-b-0` — the total from the previous frame tells us
        // whether a later item follows.
        let total: Option<usize> = ui
            .ctx()
            .data_mut(|d| d.get_temp(root.state_id.with("item_total")));
        let is_last = total.map(|t| index + 1 >= t).unwrap_or(false);

        let scope = AccordionItemScope {
            state_id: root.state_id,
            kind: root.kind,
            collapsible: root.collapsible,
            index,
        };
        ui.ctx().data_mut(|d| {
            let mut stack: Vec<AccordionItemScope> =
                d.get_temp(item_stack_id()).unwrap_or_default();
            stack.push(scope);
            d.insert_temp(item_stack_id(), stack);
        });

        let left = ui.cursor().min.x;
        let width = ui.available_width();
        content(ui);

        ui.ctx().data_mut(|d| {
            let mut stack: Vec<AccordionItemScope> =
                d.get_temp(item_stack_id()).unwrap_or_default();
            stack.pop();
            d.insert_temp(item_stack_id(), stack);
        });

        if !is_last {
            let border_rect =
                Rect::from_min_size(Pos2::new(left, ui.cursor().min.y), Vec2::new(width, 1.0));
            if ui.is_rect_visible(border_rect) {
                let p = ShadcnTheme::of_ui(ui).palette;
                ui.painter().line_segment(
                    [
                        Pos2::new(left, ui.cursor().min.y),
                        Pos2::new(left + width, ui.cursor().min.y),
                    ],
                    Stroke::new(1.0, p.border),
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AccordionTrigger
// ---------------------------------------------------------------------------

/// The clickable header row of an [`AccordionItem`]: label left, rotating
/// chevron right.
#[derive(Debug)]
pub struct AccordionTrigger {
    label: String,
    disabled: bool,
    id_salt: Option<String>,
}

impl AccordionTrigger {
    /// Creates a trigger with the given label.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            id_salt: None,
        }
    }

    /// Disables the trigger (`disabled:pointer-events-none
    /// disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides the default stable id (derived from the accordion and item
    /// index otherwise).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the trigger row.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let Some(scope) = item_scope(ui) else {
            // Outside an accordion: render an inert, unstyled row.
            return ui.label(self.label);
        };

        let open = is_open(ui, &scope);
        let anim_id = scope.state_id.with("chevron").with(scope.index);
        // `transition-transform duration-200` — 200ms ease-out both ways.
        let progress = animation::animate_value(
            ui.ctx(),
            anim_id,
            if open { 1.0 } else { 0.0 },
            Duration::Md,
        );

        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let fg = if self.disabled {
            color_alpha(p.foreground, 0.5)
        } else {
            p.foreground
        };
        let icon = if self.disabled {
            color_alpha(p.muted_foreground, 0.5)
        } else {
            p.muted_foreground
        };

        // N6 (spec §3.4): measure the label first, wrapped to the `flex-1`
        // width — the row minus the 16px chevron and the 16px `gap-4`
        // minimum — so a long label wraps inside its own width and never
        // paints under the chevron. A wrapped label grows the row (`py-2.5`
        // around the text); a single-line label keeps the 40px nova row
        // exactly.
        let icon_size = crate::theme::tokens::spacing::LG;
        let avail_w = ui.available_width();
        let label_width = (avail_w - icon_size - TRIGGER_LABEL_GAP).max(0.0);
        let galley = ui
            .painter()
            .layout(self.label.clone(), font, fg, label_width);
        let row_h = (galley.size().y + 2.0 * TRIGGER_PAD_Y).max(ACCORDION_TRIGGER_HEIGHT);
        let desired = Vec2::new(avail_w, row_h);
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => scope.state_id.with("trigger").with(scope.index),
        };

        let (_, rect) = ui.allocate_space(desired);
        let mut response = ui.interact(
            rect,
            id,
            if self.disabled {
                Sense::hover()
            } else {
                Sense::click()
            },
        );

        let clicked = response.clicked() && !self.disabled;
        if clicked {
            toggle(ui, &scope);
            response.mark_changed();
        }

        // Radix roving focus: ArrowDown/ArrowRight move to the next trigger,
        // ArrowUp/ArrowLeft to the previous one, Home/End jump to the
        // first/last. Disabled triggers stay in the traversal like Radix.
        // While focused, the trigger claims the arrow keys via the focus lock
        // filter so egui's own spatial navigation does not also move focus
        // (Tab keeps the built-in traversal). `gained_focus()` is excluded so
        // a trigger focused earlier in the same frame does not consume the
        // same key press again.
        if response.has_focus() && !self.disabled {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    response.id,
                    egui::EventFilter {
                        tab: false,
                        horizontal_arrows: true,
                        vertical_arrows: true,
                        escape: false,
                    },
                );
            });
        }
        if response.has_focus() && !response.gained_focus() && !self.disabled {
            let total = ui
                .ctx()
                .data_mut(|d| d.get_temp::<usize>(scope.state_id.with("item_total")))
                .unwrap_or(scope.index + 1)
                .max(1);
            let last = total - 1;
            let (next, prev, home, end) = ui.input(|i| {
                (
                    i.key_pressed(Key::ArrowDown) || i.key_pressed(Key::ArrowRight),
                    i.key_pressed(Key::ArrowUp) || i.key_pressed(Key::ArrowLeft),
                    i.key_pressed(Key::Home),
                    i.key_pressed(Key::End),
                )
            });
            let target = if home {
                Some(0)
            } else if end {
                Some(last)
            } else if next {
                Some((scope.index + 1).min(last))
            } else if prev {
                Some(scope.index.saturating_sub(1))
            } else {
                None
            };
            if let Some(index) = target {
                if index != scope.index {
                    // Sibling ids follow the default trigger pattern; a
                    // custom `id_salt` trigger opts out of traversal.
                    ui.memory_mut(|m| m.request_focus(scope.state_id.with("trigger").with(index)));
                }
            }
        }

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() && !self.disabled;

            // `hover:underline` on the label.
            let underline = hovered;

            // nova `rounded-lg`; the row itself stays transparent.
            if response.has_focus() && !self.disabled {
                crate::utils::focus::paint_ring(
                    ui.painter(),
                    rect.expand2(egui::vec2(1.0, 1.0)),
                    color_alpha(p.ring, 0.5),
                    3.0,
                    theme.radius.lg(),
                );
            }

            let painter = ui.painter();
            let center_y = rect.center().y;
            // A row taller than the 40px nova row is a wrapped label: the
            // label is `items-start` (top-aligned at `py-2.5`) and the
            // chevron sits on the first 20px line box. A 40px row paints
            // exactly as before (centred label, centred chevron).
            let wrapped = galley.size().y + 2.0 * TRIGGER_PAD_Y > ACCORDION_TRIGGER_HEIGHT;
            let text_pos = if wrapped {
                Pos2::new(rect.min.x, rect.min.y + TRIGGER_PAD_Y)
            } else {
                Align2::LEFT_CENTER
                    .anchor_size(Pos2::new(rect.min.x, center_y), galley.size())
                    .min
            };
            painter.galley(text_pos, galley.clone(), fg);
            if underline {
                // `hover:underline` underlines every wrapped line, like CSS;
                // the single-line case keeps the previous segment exactly.
                let y_off = crate::theme::tokens::type_scale::SM * 0.5 + 1.5;
                if wrapped {
                    for placed in galley.rows.iter() {
                        let y = text_pos.y + placed.pos.y + placed.row.size.y * 0.5 + y_off;
                        painter.line_segment(
                            [
                                Pos2::new(text_pos.x + placed.pos.x, y),
                                Pos2::new(text_pos.x + placed.pos.x + placed.row.size.x, y),
                            ],
                            Stroke::new(1.0, fg),
                        );
                    }
                } else {
                    let y = center_y + y_off;
                    painter.line_segment(
                        [
                            Pos2::new(rect.min.x, y),
                            Pos2::new(rect.min.x + galley.size().x, y),
                        ],
                        Stroke::new(1.0, fg),
                    );
                }
            }

            // Chevron centre: the first 20px `text-sm` line box, 2px
            // `translate-y-0.5` down from its top — which is the row centre
            // for a single-line row, so the single-line paint is unchanged.
            let chevron_y = rect.min.y + TRIGGER_PAD_Y + 2.0 + icon_size * 0.5;
            let icon_rect = Rect::from_center_size(
                Pos2::new(rect.max.x - icon_size * 0.5, chevron_y),
                Vec2::splat(icon_size),
            );
            paint_chevron(painter, icon_rect, icon, progress);
        }

        response
    }
}

/// Paints a lucide chevron-down rotated by `180° * progress`.
fn paint_chevron(painter: &egui::Painter, rect: Rect, color: Color32, progress: f32) {
    // Lucide `chevron-down`: (6,9) -> (12,15) -> (18,9) in a 24×24 box.
    let pts: [[f32; 2]; 3] = [[6.0, 9.0], [12.0, 15.0], [18.0, 9.0]];
    let angle = std::f32::consts::PI * progress.clamp(0.0, 1.0);
    let (sin, cos) = angle.sin_cos();
    let scale = rect.width().min(rect.height()) / 24.0;
    let center = rect.center();
    let stroke = Stroke::new((2.0 * scale).max(1.0), color);
    let to_screen = |p: [f32; 2]| -> Pos2 {
        let rel = Vec2::new(p[0] - 12.0, p[1] - 12.0);
        let rotated = Vec2::new(rel.x * cos - rel.y * sin, rel.x * sin + rel.y * cos);
        center + rotated * scale
    };
    let seg = [to_screen(pts[0]), to_screen(pts[1]), to_screen(pts[2])];
    painter.line_segment([seg[0], seg[1]], stroke);
    painter.line_segment([seg[1], seg[2]], stroke);
}

// ---------------------------------------------------------------------------
// AccordionContent
// ---------------------------------------------------------------------------

/// The animated content region of an [`AccordionItem`] (`text-sm`,
/// nova inner `pt-0 pb-2.5`).
#[derive(Clone, Debug, Default)]
pub struct AccordionContent {
    _private: (),
}

impl AccordionContent {
    /// Creates the content region.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the content with an animated height, shown only while the
    /// item is open (or animating closed).
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let Some(scope) = item_scope(ui) else {
            content(ui);
            return;
        };
        let open = is_open(ui, &scope);
        let anim_id = scope.state_id.with("content").with(scope.index);
        // `animate-accordion-down/up`: 0.2s ease-out in both directions.
        crate::components::collapsible::animated_disclosure(
            ui,
            anim_id,
            open,
            Duration::Md,
            Duration::Md,
            |ui| {
                content(ui);
                ui.add_space(CONTENT_PAD_BOTTOM); // `pb-2.5`
            },
        );
    }
}

#[cfg(test)]
// The loop index doubles as the accordion item order, so index-based loops
// are intentional here.
#[allow(clippy::needless_range_loop)]
mod tests {
    use super::*;
    use crate::theme::Preset;
    use egui::{Key, PointerButton};

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

        fn step(&mut self, events: Vec<egui::Event>, mut f: impl FnMut(&mut Ui)) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("accordion_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
        }

        fn run(&mut self, f: impl FnMut(&mut Ui)) {
            self.step(Vec::new(), f);
        }

        /// Like [`Self::step`], but returns the frame's output so tests can
        /// inspect the painted shapes.
        fn step_out(
            &mut self,
            events: Vec<egui::Event>,
            mut f: impl FnMut(&mut Ui),
        ) -> egui::FullOutput {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("accordion_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
            out
        }

        fn run_out(&mut self, f: impl FnMut(&mut Ui)) -> egui::FullOutput {
            self.step_out(Vec::new(), f)
        }

        /// Moves the pointer, presses and releases, each on a rendered frame.
        fn click(&mut self, pos: Pos2, mut f: impl FnMut(&mut Ui)) {
            self.step(vec![egui::Event::PointerMoved(pos)], &mut f);
            for pressed in [true, false] {
                self.step(
                    vec![egui::Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed,
                        modifiers: egui::Modifiers::default(),
                    }],
                    &mut f,
                );
            }
        }

        /// Presses a key on a rendered frame (the focused widget must exist).
        fn key(&mut self, key: Key, f: impl FnMut(&mut Ui)) {
            self.step(
                vec![egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                    physical_key: None,
                }],
                f,
            );
        }

        fn focus(&self, id: Id) {
            self.ctx.memory_mut(|m| m.request_focus(id));
        }
    }

    /// A trigger-only 3-item single accordion (layout never changes, so
    /// captured rects stay valid across clicks). Collapsible, so toggle tests
    /// can close items again.
    fn triggers_ui(ui: &mut Ui, salt: &str, rects: &mut [Rect; 3]) {
        Accordion::new()
            .id_salt(salt)
            .collapsible(true)
            .show(ui, |ui| {
                for i in 0..3 {
                    AccordionItem::new().show(ui, |ui| {
                        rects[i] = AccordionTrigger::new(format!("Item {i}")).show(ui).rect;
                    });
                }
            });
    }

    fn single_state(h: &Harness, salt: &str) -> Option<usize> {
        h.ctx
            .data_mut(|d| d.get_temp::<Option<usize>>(Id::unique("accordion_test_ui").with(salt)))
            .flatten()
    }

    fn multi_state(h: &Harness, salt: &str) -> Vec<bool> {
        // Same read as `read_multi` (which takes a `&Ui`); the harness
        // holds the `Context` directly.
        h.ctx
            .data_mut(|d| d.get_temp::<Vec<bool>>(Id::unique("accordion_test_ui").with(salt)))
            .unwrap_or_default()
    }

    #[test]
    fn single_toggle_and_switch() {
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 3];
        h.run(|ui| triggers_ui(ui, "acc", &mut rects));
        assert_eq!(single_state(&h, "acc"), None, "nothing open by default");

        h.click(rects[0].center(), |ui| {
            triggers_ui(ui, "acc", &mut [Rect::ZERO; 3])
        });
        assert_eq!(
            single_state(&h, "acc"),
            Some(0),
            "clicking a trigger opens it"
        );

        h.click(rects[1].center(), |ui| {
            triggers_ui(ui, "acc", &mut [Rect::ZERO; 3])
        });
        assert_eq!(
            single_state(&h, "acc"),
            Some(1),
            "single mode switches the open item"
        );

        h.click(rects[1].center(), |ui| {
            triggers_ui(ui, "acc", &mut [Rect::ZERO; 3])
        });
        assert_eq!(
            single_state(&h, "acc"),
            None,
            "single + collapsible closes on re-click"
        );
    }

    #[test]
    fn non_collapsible_keeps_item_open() {
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 3];
        h.run(|ui| {
            Accordion::new()
                .id_salt("fixed")
                .default_open(1)
                .collapsible(false)
                .show(ui, |ui| {
                    for i in 0..3 {
                        AccordionItem::new().show(ui, |ui| {
                            rects[i] = AccordionTrigger::new(format!("Item {i}")).show(ui).rect;
                        });
                    }
                });
        });
        assert_eq!(single_state(&h, "fixed"), Some(1));

        h.click(rects[1].center(), |ui| {
            Accordion::new()
                .id_salt("fixed")
                .collapsible(false)
                .show(ui, |ui| {
                    for i in 0..3 {
                        AccordionItem::new().show(ui, |ui| {
                            AccordionTrigger::new(format!("Item {i}")).show(ui);
                        });
                    }
                });
        });
        assert_eq!(
            single_state(&h, "fixed"),
            Some(1),
            "collapsible=false keeps the item open"
        );
    }

    #[test]
    fn multiple_toggles_independently() {
        let mut h = Harness::new();
        let mut flags = vec![false, false];
        let mut trigger_rects = [Rect::ZERO; 2];
        h.run(|ui| {
            Accordion::multiple(&mut flags)
                .id_salt("multi")
                .show(ui, |ui| {
                    for i in 0..2 {
                        AccordionItem::new().show(ui, |ui| {
                            trigger_rects[i] = AccordionTrigger::new(format!("M{i}")).show(ui).rect;
                        });
                    }
                });
        });
        assert_eq!(flags, vec![false, false]);

        let multi_ui = |ui: &mut Ui, flags: &mut Vec<bool>| {
            Accordion::multiple(flags).id_salt("multi").show(ui, |ui| {
                for i in 0..2 {
                    AccordionItem::new().show(ui, |ui| {
                        AccordionTrigger::new(format!("M{i}")).show(ui);
                    });
                }
            });
        };

        h.click(trigger_rects[0].center(), |ui| multi_ui(ui, &mut flags));
        assert_eq!(flags, vec![true, false], "first item toggled on");

        h.click(trigger_rects[1].center(), |ui| multi_ui(ui, &mut flags));
        assert_eq!(flags, vec![true, true], "both items can be open at once");
    }

    #[test]
    fn multiple_default_open_many_starts_open() {
        // N3 (spec §8.3): uncontrolled multiple with `default_open_many`
        // starts with exactly the given items open, and behaves like
        // controlled multiple initialised with the same flags.
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 3];
        h.run(|ui| {
            Accordion::new_multiple()
                .id_salt("many")
                .default_open_many([0, 2])
                .show(ui, |ui| {
                    for i in 0..3 {
                        AccordionItem::new().show(ui, |ui| {
                            rects[i] = AccordionTrigger::new(format!("M{i}")).show(ui).rect;
                        });
                    }
                });
        });
        assert_eq!(
            multi_state(&h, "many"),
            vec![true, false, true],
            "default_open_many opens exactly the given items"
        );

        // Controlled multiple with the same flags starts identically.
        let mut flags = vec![true, false, true];
        h.run(|ui| {
            Accordion::multiple(&mut flags)
                .id_salt("many-ctrl")
                .show(ui, |ui| {
                    for i in 0..3 {
                        AccordionItem::new().show(ui, |ui| {
                            AccordionTrigger::new(format!("C{i}")).show(ui);
                        });
                    }
                });
        });
        assert_eq!(flags, vec![true, false, true]);

        // Clicking item 1 toggles it on in both.
        h.click(rects[1].center(), |ui| {
            Accordion::new_multiple()
                .id_salt("many")
                .default_open_many([0, 2])
                .show(ui, |ui| {
                    for i in 0..3 {
                        AccordionItem::new().show(ui, |ui| {
                            AccordionTrigger::new(format!("M{i}")).show(ui);
                        });
                    }
                });
        });
        assert_eq!(
            multi_state(&h, "many"),
            vec![true, true, true],
            "toggled item joins the default-open set"
        );
        h.click(rects[1].center(), |ui| {
            Accordion::multiple(&mut flags)
                .id_salt("many-ctrl")
                .show(ui, |ui| {
                    for i in 0..3 {
                        AccordionItem::new().show(ui, |ui| {
                            AccordionTrigger::new(format!("C{i}")).show(ui);
                        });
                    }
                });
        });
        assert_eq!(flags, vec![true, true, true], "controlled matches");
    }

    #[test]
    fn default_open_builders_are_mode_specific() {
        // `default_open` seeds single mode only; `default_open_many`
        // seeds multiple mode only. Each is ignored in the other mode.
        let mut h = Harness::new();
        h.run(|ui| {
            Accordion::new_multiple()
                .id_salt("many-only")
                .default_open(1)
                .show(ui, |ui| {
                    for i in 0..2 {
                        AccordionItem::new().show(ui, |ui| {
                            AccordionTrigger::new(format!("M{i}")).show(ui);
                        });
                    }
                });
        });
        // Multiple mode without `default_open_many`: nothing open, even
        // though `default_open(1)` was given (single-only builder). The
        // stored flags may be shorter than the item count — no flag is
        // true, which is what "nothing open" means.
        assert!(
            !multi_state(&h, "many-only").iter().any(|open| *open),
            "default_open does not seed multiple mode"
        );

        h.run(|ui| {
            Accordion::new()
                .id_salt("single-only")
                .default_open_many([0, 1])
                .show(ui, |ui| {
                    for i in 0..2 {
                        AccordionItem::new().show(ui, |ui| {
                            AccordionTrigger::new(format!("S{i}")).show(ui);
                        });
                    }
                });
        });
        assert_eq!(
            single_state(&h, "single-only"),
            None,
            "default_open_many does not seed single mode"
        );
    }

    #[test]
    fn controlled_single_round_trips() {
        let mut h = Harness::new();
        let mut open = Some(0usize);
        let mut rects = [Rect::ZERO; 2];
        h.run(|ui| {
            Accordion::single(&mut open).id_salt("ctrl").show(ui, |ui| {
                for i in 0..2 {
                    AccordionItem::new().show(ui, |ui| {
                        rects[i] = AccordionTrigger::new(format!("C{i}")).show(ui).rect;
                    });
                }
            });
        });
        assert_eq!(open, Some(0), "controlled value written back unchanged");

        h.click(rects[1].center(), |ui| {
            Accordion::single(&mut open).id_salt("ctrl").show(ui, |ui| {
                for i in 0..2 {
                    AccordionItem::new().show(ui, |ui| {
                        AccordionTrigger::new(format!("C{i}")).show(ui);
                    });
                }
            });
        });
        assert_eq!(open, Some(1), "click write-back reaches the caller");
    }

    #[test]
    fn keyboard_enter_toggles_focused_trigger() {
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 3];
        h.run(|ui| triggers_ui(ui, "kbd", &mut rects));
        assert_eq!(single_state(&h, "kbd"), None);

        // Focus the first trigger, then press Enter (egui keyboard
        // activation of focused click widgets) while it is rendered.
        let state_id = Id::unique("accordion_test_ui").with("kbd");
        h.focus(state_id.with("trigger").with(0));
        h.key(Key::Enter, |ui| {
            triggers_ui(ui, "kbd", &mut [Rect::ZERO; 3])
        });
        assert_eq!(single_state(&h, "kbd"), Some(0), "Enter opens the item");

        // Space toggles it closed again.
        h.focus(state_id.with("trigger").with(0));
        h.key(Key::Space, |ui| {
            triggers_ui(ui, "kbd", &mut [Rect::ZERO; 3])
        });
        assert_eq!(single_state(&h, "kbd"), None, "Space closes the item");
    }

    #[test]
    fn arrow_keys_rove_trigger_focus() {
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 3];
        h.run(|ui| triggers_ui(ui, "rove", &mut rects));
        let state_id = Id::unique("accordion_test_ui").with("rove");
        let trigger = |i: usize| state_id.with("trigger").with(i);
        h.focus(trigger(0));
        // Render two plain frames so the focused trigger has installed its
        // focus lock filter (it can only be set from the frame after focus).
        h.run(|ui| triggers_ui(ui, "rove", &mut [Rect::ZERO; 3]));
        h.run(|ui| triggers_ui(ui, "rove", &mut [Rect::ZERO; 3]));

        h.key(Key::ArrowDown, |ui| {
            triggers_ui(ui, "rove", &mut [Rect::ZERO; 3])
        });
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger(1)),
            "ArrowDown moves focus to the next trigger"
        );

        h.key(Key::ArrowRight, |ui| {
            triggers_ui(ui, "rove", &mut [Rect::ZERO; 3])
        });
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger(2)),
            "ArrowRight moves to the last trigger"
        );

        h.key(Key::ArrowDown, |ui| {
            triggers_ui(ui, "rove", &mut [Rect::ZERO; 3])
        });
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger(2)),
            "ArrowDown on the last trigger keeps focus there"
        );

        h.key(Key::ArrowUp, |ui| {
            triggers_ui(ui, "rove", &mut [Rect::ZERO; 3])
        });
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger(1)),
            "ArrowUp moves focus to the previous trigger"
        );

        h.key(Key::Home, |ui| {
            triggers_ui(ui, "rove", &mut [Rect::ZERO; 3])
        });
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger(0)),
            "Home jumps to the first trigger"
        );

        h.key(Key::End, |ui| triggers_ui(ui, "rove", &mut [Rect::ZERO; 3]));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger(2)),
            "End jumps to the last trigger"
        );
    }

    #[test]
    fn content_animates_open_and_lays_out() {
        let mut h = Harness::new();
        let mut bottom = 0.0;
        for _ in 0..4 {
            h.run(|ui| {
                Accordion::new()
                    .id_salt("anim")
                    .default_open(0)
                    .show(ui, |ui| {
                        for i in 0..2 {
                            AccordionItem::new().show(ui, |ui| {
                                AccordionTrigger::new(format!("A{i}")).show(ui);
                                AccordionContent::new().show(ui, |ui| {
                                    ui.label(format!("Body {i}"));
                                });
                            });
                        }
                    });
                bottom = ui.cursor().min.y;
            });
        }
        // Two trigger rows plus the open content region (measure + animate).
        assert!(
            bottom > 3.0 * ACCORDION_TRIGGER_HEIGHT,
            "open content adds layout height (got {bottom})"
        );

        // Closing collapses the layout back down.
        let state_id = Id::unique("accordion_test_ui").with("anim");
        h.ctx.data_mut(|d| d.insert_temp(state_id, None::<usize>));
        for _ in 0..4 {
            h.run(|ui| {
                Accordion::new().id_salt("anim").show(ui, |ui| {
                    for i in 0..2 {
                        AccordionItem::new().show(ui, |ui| {
                            AccordionTrigger::new(format!("A{i}")).show(ui);
                            AccordionContent::new().show(ui, |ui| {
                                ui.label(format!("Body {i}"));
                            });
                        });
                    }
                });
                bottom = ui.cursor().min.y;
            });
        }
        assert!(
            bottom <= 3.0 * ACCORDION_TRIGGER_HEIGHT + 1.0,
            "closed content frees layout height (got {bottom})"
        );
    }

    #[test]
    fn long_label_keeps_gap_and_wraps() {
        // N6 (spec §3.4): a long label wraps inside its `flex-1` width,
        // keeps the 16px `gap-4` clearance to the chevron, and never paints
        // under it. The row grows to contain the wrapped text.
        let mut h = Harness::new();
        let label = "What is the return policy for items purchased during the holiday sale period?";
        let mut rect = Rect::ZERO;
        let out = h.run_out(|ui| {
            Accordion::new().id_salt("long").show(ui, |ui| {
                AccordionItem::new().show(ui, |ui| {
                    rect = AccordionTrigger::new(label).show(ui).rect;
                });
            });
        });
        // The painted label galley (the Text shape carrying the full label).
        let mut painted = None;
        for clipped in &out.shapes {
            if let egui::Shape::Text(ts) = &clipped.shape {
                if ts.galley.job.text == label {
                    painted = Some((ts.pos, ts.galley.clone()));
                }
            }
        }
        let (pos, galley) = painted.expect("label Text shape painted");
        // Wrapped: more than one 20px line box of text.
        assert!(
            galley.size().y > 20.0,
            "long label wraps to multiple lines (galley height {})",
            galley.size().y
        );
        // The row grew to contain the wrapped text (`py-2.5` around it).
        assert!(
            rect.height() > ACCORDION_TRIGGER_HEIGHT,
            "wrapped label grows the row (height {})",
            rect.height()
        );
        assert!(
            (rect.height() - (galley.size().y + 2.0 * TRIGGER_PAD_Y)).abs() < 0.5,
            "row height is text height + py-2.5 (got {})",
            rect.height()
        );
        // 16px chevron + 16px `gap-4` clearance: the widest label line ends
        // at least 32px left of the row's right edge — never under the
        // chevron.
        assert!(
            pos.x + galley.size().x <= rect.max.x - 32.0 + 0.5,
            "label right edge {} leaves the 16px gap + 16px chevron (row right {})",
            pos.x + galley.size().x,
            rect.max.x
        );
    }

    #[test]
    fn trigger_row_height_is_nova_40px() {
        // Spec §3.1: the nova trigger row is 40px (`py-2.5` + 20px line
        // box). The base 52px row is NOT implemented here — N7 records the
        // nova-hardcoded geometry as acceptable for Batch D; the base
        // values are a Phase 4 item. This test pins the implemented nova
        // row so a regression cannot silently change it.
        assert_eq!(ACCORDION_TRIGGER_HEIGHT, 40.0);
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 3];
        h.run(|ui| triggers_ui(ui, "h40", &mut rects));
        for (i, rect) in rects.iter().enumerate() {
            assert_eq!(
                rect.height(),
                40.0,
                "single-line trigger {i} is the 40px nova row"
            );
        }
    }

    #[test]
    fn chevron_rotates_180_when_open() {
        // Spec §3.3/§6: the chevron rotates 180° over 200ms when its item
        // opens. The painted rotation is driven by `animate_value` with
        // `Duration::Md` (0.2s); after settling, the open item's progress
        // is 1.0 and a closed item's is 0.0.
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 2];
        for _ in 0..4 {
            h.run(|ui| {
                Accordion::new()
                    .id_salt("chev")
                    .default_open(0)
                    .show(ui, |ui| {
                        for i in 0..2 {
                            AccordionItem::new().show(ui, |ui| {
                                rects[i] = AccordionTrigger::new(format!("C{i}")).show(ui).rect;
                            });
                        }
                    });
            });
        }
        assert_eq!(rects[0].height(), 40.0, "single-line rows stay 40px");
        let state_id = Id::unique("accordion_test_ui").with("chev");
        let open =
            animation::animate_value(&h.ctx, state_id.with("chevron").with(0), 1.0, Duration::Md);
        assert!(
            (open - 1.0).abs() < 0.01,
            "open item chevron settles at 180° (progress {open})"
        );
        let closed =
            animation::animate_value(&h.ctx, state_id.with("chevron").with(1), 0.0, Duration::Md);
        assert!(
            closed.abs() < 0.01,
            "closed item chevron settles at 0° (progress {closed})"
        );
    }

    #[test]
    fn focus_ring_geometry() {
        // Spec §4.4: a focused trigger paints a 3px `ring/50` ring at the
        // slot radius (10px nova) around the trigger rect expanded by 1px.
        // The base 8px radius is N7 (Phase 4); this test pins the
        // implemented nova geometry.
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 3];
        h.run(|ui| triggers_ui(ui, "ring", &mut rects));
        let state_id = Id::unique("accordion_test_ui").with("ring");
        let trigger = |i: usize| state_id.with("trigger").with(i);
        h.focus(trigger(0));
        // Settle two frames so the focused trigger has installed its
        // focus lock filter (spec §7.1 note).
        h.run(|ui| triggers_ui(ui, "ring", &mut [Rect::ZERO; 3]));
        let out = h.run_out(|ui| triggers_ui(ui, "ring", &mut [Rect::ZERO; 3]));

        let expected_rect = rects[0].expand2(Vec2::new(1.0, 1.0));
        let mut found = false;
        for clipped in &out.shapes {
            if let egui::Shape::Rect(rs) = &clipped.shape {
                if rs.stroke.width == 3.0 {
                    assert_eq!(rs.corner_radius.nw, 10, "nova ring radius is 10px");
                    assert_eq!(rs.corner_radius.ne, 10);
                    assert_eq!(rs.corner_radius.sw, 10);
                    assert_eq!(rs.corner_radius.se, 10);
                    assert!(
                        (rs.rect.min - expected_rect.min).length() < 0.5
                            && (rs.rect.max - expected_rect.max).length() < 0.5,
                        "ring rect {} is the trigger rect expanded by 1px ({})",
                        rs.rect,
                        expected_rect
                    );
                    // `color_alpha(ring, 0.5)` on an opaque colour: alpha
                    // (255 * 0.5 + 0.5) as u8 = 128.
                    assert_eq!(
                        rs.stroke.color.a(),
                        128,
                        "ring is ring/50, got alpha {}",
                        rs.stroke.color.a()
                    );
                    found = true;
                }
            }
        }
        assert!(found, "focused trigger paints a 3px focus ring");
    }

    #[test]
    fn pure_toggle_helpers() {
        assert_eq!(next_single(None, 2, true), Some(2));
        assert_eq!(next_single(Some(2), 2, true), None);
        assert_eq!(next_single(Some(2), 2, false), Some(2));
        assert_eq!(next_single(Some(1), 2, false), Some(2));
        assert_eq!(next_multiple(&[false, false], 1), vec![false, true]);
        assert_eq!(next_multiple(&[true, true], 1), vec![true, false]);
        // Resizes short vectors.
        assert_eq!(next_multiple(&[], 3), vec![false, false, false, true]);
    }

    #[test]
    fn triggers_and_items_resolve_scope() {
        let mut h = Harness::new();
        // Orphan trigger/content outside an accordion render without panic.
        h.run(|ui| {
            let _ = AccordionTrigger::new("orphan").show(ui);
            AccordionContent::new().show(ui, |ui| {
                ui.label("orphan body");
            });
            AccordionItem::new().show(ui, |ui| {
                ui.label("orphan item");
            });
        });
    }
}
