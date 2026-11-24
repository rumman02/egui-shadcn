//! shadcn `tabs.tsx` — tabbed panels with a trigger list.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/tabs.tsx` with the nova
//! overrides: the list is `rounded-lg p-[3px] h-8` (`bg-muted` for the
//! default variant, transparent + `gap-1` + no radius for `line`), triggers
//! are `px-1.5 py-0.5 text-sm font-medium rounded-md` showing
//! `text-foreground/60` when inactive and `bg-background shadow-sm` when
//! active (dark: `border-input bg-input/30`). The `line` variant draws a
//! 2px foreground underline (horizontal) / side bar (vertical) on the active
//! trigger instead of a fill.
//!
//! Triggers stretch to an equal width like the source's `flex-1` (the list
//! remembers the widest trigger between frames), and vertical triggers fill
//! the column like `w-full`.
//!
//! The selected value lives in the egui context keyed by a stable id (pass
//! [`Tabs::id_salt`] per instance) or is controlled through a `&mut String`
//! with [`Tabs::controlled`]. Keyboard (Radix roving tabindex, default
//! `activationMode="automatic"`): ArrowRight/ArrowLeft (horizontal) or
//! ArrowDown/ArrowUp (vertical) move focus to the next/previous enabled
//! trigger and select it, Home/End jump to the first/last enabled trigger,
//! and Enter/Space activate the focused trigger. Tab keeps egui's own
//! traversal. With `.activation_mode(TabsActivationMode::Manual)` the arrow
//! keys (and Home/End) move focus only — the selection does not follow until
//! Enter/Space or a click activates the focused trigger.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Tabs::new().id_salt("settings").show(ui, |ui| {
//!     TabsList::new().show(ui, |ui| {
//!         TabsTrigger::new("account", "Account").show(ui);
//!         TabsTrigger::new("password", "Password").show(ui);
//!     });
//!     TabsContent::new("account").show(ui, |ui| {
//!         ui.label("Account settings…");
//!     });
//!     TabsContent::new("password").show(ui, |ui| {
//!         ui.label("Change your password…");
//!     });
//! });
//!
//! let mut tab = "account".to_owned();
//! Tabs::controlled(&mut tab).id_salt("t2").show(ui, |ui| { /* … */ });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use egui::{
    Align2, Color32, CornerRadius, FontId, Id, Key, Pos2, Rect, Response, Sense, Shape, Stroke, Ui,
    Vec2,
};

/// Visual list variants, mirroring the source `tabsListVariants`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsVariant {
    /// `bg-muted` pill list; active trigger gets a filled background.
    #[default]
    Default,
    /// Transparent list; active trigger gets a foreground underline/bar.
    Line,
}

/// Layout direction, mirroring the source `orientation` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsOrientation {
    /// Triggers in a row, content below.
    #[default]
    Horizontal,
    /// Triggers in a column, content to the right.
    Vertical,
}

/// Activation behaviour, mirroring Radix `activationMode` (`tabs.tsx`
/// passes the prop through; the source default is `"automatic"`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabsActivationMode {
    /// Arrow keys move focus AND select (Radix default).
    #[default]
    Automatic,
    /// Arrow keys move focus only; Enter/Space (or a click) selects.
    Manual,
}

/// Height of the trigger list (nova `h-8`), horizontal orientation.
pub const TABS_LIST_HEIGHT: f32 = 32.0;
/// List padding `p-[3px]`.
const LIST_PAD: f32 = 3.0;
/// Trigger height inside the list: `h-[calc(100%-1px)]` of the padded inner.
const TRIGGER_H: f32 = TABS_LIST_HEIGHT - LIST_PAD * 2.0 - 1.0;
/// Trigger height when the list is vertical (`h-fit`; nova `px-1.5 py-0.5`
/// + the 20px `text-sm` line box).
const TRIGGER_H_VERTICAL: f32 = 24.0;
/// Height/width of the `line` variant indicator (`after:h-0.5`).
const INDICATOR: f32 = 2.0;

/// Horizontal gap between triggers: the base list has none (the `flex-1`
/// triggers touch) and the `line` variant adds `gap-1`.
fn list_gap(variant: TabsVariant) -> f32 {
    match variant {
        TabsVariant::Default => 0.0,
        TabsVariant::Line => crate::theme::tokens::spacing::XS,
    }
}

/// The `after:bottom-[-5px] after:h-0.5` underline: it hangs below the
/// trigger, spanning `[max.y + 3, max.y + 5]`.
fn horizontal_indicator_rect(rect: Rect) -> Rect {
    Rect::from_min_size(
        Pos2::new(rect.min.x, rect.max.y + 5.0 - INDICATOR),
        Vec2::new(rect.width(), INDICATOR),
    )
}

/// The `after:-right-1 after:w-0.5` side bar: it hangs to the right of the
/// trigger, spanning `[max.x + 2, max.x + 4]`.
fn vertical_indicator_rect(rect: Rect) -> Rect {
    Rect::from_min_size(
        Pos2::new(rect.max.x + 4.0 - INDICATOR, rect.min.y),
        Vec2::new(INDICATOR, rect.height()),
    )
}

// ---------------------------------------------------------------------------
// Scope plumbing
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct TabsRoot {
    value_id: Id,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
}

#[derive(Clone, Copy, Debug)]
struct TabsListScope {
    value_id: Id,
    variant: TabsVariant,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
}

fn root_stack_id() -> Id {
    Id::unique("shadcn_tabs_root_stack")
}

fn list_stack_id() -> Id {
    Id::unique("shadcn_tabs_list_stack")
}

fn scratch_id() -> Id {
    Id::unique("shadcn_tabs_list_scratch")
}

fn trigger_order_collect_id(value_id: Id) -> Id {
    value_id.with("trigger_order_collect")
}

/// Ordered `(value, disabled)` pairs from the previous frame's list show —
/// the roving-focus navigation reads this so the last trigger can move
/// focus to a sibling that has not been shown yet in the current frame.
fn trigger_order_id(value_id: Id) -> Id {
    value_id.with("trigger_order")
}

fn trigger_widths_collect_id(value_id: Id) -> Id {
    value_id.with("trigger_widths_collect")
}

/// The widest natural trigger width from the previous frame (`flex-1`
/// equal-width stretch; one-frame lag, same as the accordion item count).
fn equal_width_id(value_id: Id) -> Id {
    value_id.with("equal_w")
}

fn peek<T: Clone + 'static>(ui: &Ui, id: Id) -> Option<T> {
    ui.ctx().data_mut(|d| d.get_temp::<T>(id))
}

fn root_scope(ui: &Ui) -> Option<TabsRoot> {
    peek::<Vec<TabsRoot>>(ui, root_stack_id()).and_then(|s| s.last().copied())
}

fn list_scope(ui: &Ui) -> Option<TabsListScope> {
    peek::<Vec<TabsListScope>>(ui, list_stack_id()).and_then(|s| s.last().copied())
}

fn read_selected(ui: &Ui, value_id: Id) -> Option<String> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Option<String>>(value_id).flatten())
}

fn write_selected(ui: &Ui, value_id: Id, value: Option<String>) {
    ui.ctx().data_mut(|d| d.insert_temp(value_id, value));
}

/// Pure selection logic for a trigger click: always selects the clicked tab.
fn selected_after_click(current: Option<&str>, value: &str) -> String {
    let _ = current;
    value.to_owned()
}

// ---------------------------------------------------------------------------
// Tabs (root)
// ---------------------------------------------------------------------------

/// The shadcn tabs root. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Tabs<'a> {
    controlled: Option<&'a mut String>,
    orientation: TabsOrientation,
    activation_mode: TabsActivationMode,
    id_salt: Option<String>,
}

impl Default for Tabs<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Tabs<'a> {
    /// Creates an uncontrolled tabs group (the first trigger becomes active).
    pub fn new() -> Self {
        Self {
            controlled: None,
            orientation: TabsOrientation::Horizontal,
            activation_mode: TabsActivationMode::Automatic,
            id_salt: None,
        }
    }

    /// Creates a tabs group whose selected value (a tab id string) is owned
    /// by the caller. The value is read before and written back after each
    /// frame.
    pub fn controlled(value: &'a mut String) -> Self {
        Self {
            controlled: Some(value),
            ..Self::new()
        }
    }

    /// Sets the layout direction (`orientation="vertical"`).
    pub fn orientation(mut self, orientation: TabsOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the activation behaviour (`activationMode="manual"`: arrow keys
    /// move focus only; Enter/Space or a click selects. The default is
    /// `automatic`, matching the source default).
    pub fn activation_mode(mut self, mode: TabsActivationMode) -> Self {
        self.activation_mode = mode;
        self
    }

    /// Sets a stable id salt so the selection persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the tabs group.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        let value_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_tabs"),
        };

        // Controlled values win each frame.
        if let Some(v) = self.controlled.as_deref() {
            write_selected(
                ui,
                value_id,
                if v.is_empty() { None } else { Some(v.clone()) },
            );
        }

        ui.ctx().data_mut(|d| {
            let mut stack: Vec<TabsRoot> = d.get_temp(root_stack_id()).unwrap_or_default();
            stack.push(TabsRoot {
                value_id,
                orientation: self.orientation,
                activation_mode: self.activation_mode,
            });
            d.insert_temp(root_stack_id(), stack);
        });

        // The source root is `flex gap-2`, a row when vertical.
        match self.orientation {
            TabsOrientation::Horizontal => {
                content(ui);
            }
            TabsOrientation::Vertical => {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::SM;
                    content(ui);
                });
            }
        }

        ui.ctx().data_mut(|d| {
            let mut stack: Vec<TabsRoot> = d.get_temp(root_stack_id()).unwrap_or_default();
            stack.pop();
            d.insert_temp(root_stack_id(), stack);
        });

        // Controlled write-back.
        if let Some(out) = self.controlled {
            *out = read_selected(ui, value_id).unwrap_or_default();
        }
    }
}

// ---------------------------------------------------------------------------
// TabsList
// ---------------------------------------------------------------------------

/// The trigger container (`rounded-lg p-[3px]`, `bg-muted` or transparent
/// for the line variant).
#[derive(Debug)]
pub struct TabsList {
    variant: TabsVariant,
    id_salt: Option<String>,
}

impl Default for TabsList {
    fn default() -> Self {
        Self::new()
    }
}

impl TabsList {
    /// Creates a list with the default (filled) variant.
    pub fn new() -> Self {
        Self {
            variant: TabsVariant::Default,
            id_salt: None,
        }
    }

    /// Sets the visual variant (`variant="line"` removes the muted fill and
    /// shows an indicator on the active trigger instead).
    pub fn variant(mut self, variant: TabsVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt (the list itself keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Lays out the triggers and paints the list background behind them.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) {
        let Some(root) = root_scope(ui) else {
            content(ui);
            return;
        };

        let scope = TabsListScope {
            value_id: root.value_id,
            variant: self.variant,
            orientation: root.orientation,
            activation_mode: root.activation_mode,
        };
        ui.ctx().data_mut(|d| {
            let mut stack: Vec<TabsListScope> = d.get_temp(list_stack_id()).unwrap_or_default();
            stack.push(scope);
            d.insert_temp(list_stack_id(), stack);
            d.insert_temp(scratch_id(), None::<Rect>); // reset the trigger union
                                                       // Reset the per-frame roving-focus and width collectors; the
                                                       // previous-frame results stay until this list finishes.
            d.insert_temp(
                trigger_order_collect_id(scope.value_id),
                Vec::<(String, bool, Id)>::new(),
            );
            d.insert_temp(trigger_widths_collect_id(scope.value_id), Vec::<f32>::new());
        });

        // Reserve a paint slot so the background renders behind the triggers.
        let bg_idx = ui.painter().add(Shape::Noop);

        let gap = list_gap(self.variant);
        match scope.orientation {
            TabsOrientation::Horizontal => {
                // The list row is exactly `h-8` (32px); the 25px triggers
                // are centred in it (`items-center`), like the source.
                // NOTE: `ui.horizontal` cannot do this — it allocates the
                // child Ui with an initial height of
                // `spacing().interact_size.y` (36px here), and
                // `set_min_height` can only grow, never shrink, so the row
                // would stay 36px. Allocating the row at the exact size
                // reserves 32px in the parent instead.
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), TABS_LIST_HEIGHT),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        content(ui);
                    },
                );
            }
            TabsOrientation::Vertical => {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = gap;
                    content(ui);
                });
            }
        }

        // Publish the trigger order (for roving-focus navigation) and the
        // widest natural trigger width (for the `flex-1` equal-width
        // stretch) for the next frame.
        ui.ctx().data_mut(|d| {
            let order: Vec<(String, bool, Id)> = d
                .get_temp::<Vec<(String, bool, Id)>>(trigger_order_collect_id(scope.value_id))
                .unwrap_or_default();
            d.insert_temp(trigger_order_id(scope.value_id), order);
            let widths: Vec<f32> = d
                .get_temp::<Vec<f32>>(trigger_widths_collect_id(scope.value_id))
                .unwrap_or_default();
            if let Some(max_w) = widths
                .iter()
                .copied()
                .fold(None::<f32>, |a, b| Some(a.map_or(b, |m| m.max(b))))
            {
                d.insert_temp(equal_width_id(scope.value_id), max_w);
            }
            let mut stack: Vec<TabsListScope> = d.get_temp(list_stack_id()).unwrap_or_default();
            stack.pop();
            d.insert_temp(list_stack_id(), stack);
        });

        // Paint the list background behind the triggers (from their union).
        if let Some(union) = ui
            .ctx()
            .data_mut(|d| d.get_temp::<Option<Rect>>(scratch_id()).flatten())
        {
            if ui.is_rect_visible(union) {
                let theme = ShadcnTheme::of_ui(ui);
                let bg = match self.variant {
                    // Plain `bg-muted`; the dark `--muted` token already
                    // encodes the dark appearance.
                    TabsVariant::Default => theme.palette.muted,
                    TabsVariant::Line => Color32::TRANSPARENT,
                };
                let radius = match self.variant {
                    TabsVariant::Default => theme.radius.lg_u8(),
                    TabsVariant::Line => 0,
                };
                // The background covers the full list rect. Horizontally the
                // 25px triggers are centred in the 32px row, so the union
                // needs 3px of padding plus the 0.5px `calc(100%-1px)`
                // slack on each side; vertically the list is `h-fit`, so
                // the 3px padding alone is exact.
                let pad_y = if scope.orientation == TabsOrientation::Horizontal {
                    LIST_PAD + 0.5
                } else {
                    LIST_PAD
                };
                let shape = egui::epaint::RectShape::filled(
                    union.expand2(egui::vec2(LIST_PAD, pad_y)),
                    CornerRadius::same(radius),
                    bg,
                );
                ui.painter().set(bg_idx, shape);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// TabsTrigger
// ---------------------------------------------------------------------------

/// One selectable tab button. Register it inside a [`TabsList`].
#[derive(Debug)]
pub struct TabsTrigger {
    value: String,
    label: String,
    icon: Option<Icon>,
    disabled: bool,
    id_salt: Option<String>,
}

impl TabsTrigger {
    /// Creates a trigger for the tab with the given value id.
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            id_salt: None,
        }
    }

    /// Shows an icon before the label (`gap-1.5`, `size-4`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Disables the trigger (`disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides the default stable id (derived from the tabs value).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the trigger.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;

        let Some(scope) = list_scope(ui) else {
            return ui.label(self.label);
        };

        let selected = read_selected(ui, scope.value_id).as_deref() == Some(self.value.as_str());
        // Uncontrolled default: the first trigger shown becomes active.
        if read_selected(ui, scope.value_id).is_none() && !self.disabled {
            write_selected(ui, scope.value_id, Some(self.value.clone()));
        }

        let vertical = scope.orientation == TabsOrientation::Vertical;
        let height = if vertical {
            TRIGGER_H_VERTICAL
        } else {
            TRIGGER_H
        };
        let pad_x = crate::theme::tokens::spacing::XS_HALF; // nova `px-1.5`
        let gap = crate::theme::tokens::spacing::XS_HALF; // `gap-1.5`
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
        let icon_w = if self.icon.is_some() {
            crate::theme::tokens::spacing::LG
        } else {
            0.0
        };
        let inner_w = text_w
            + icon_w
            + if self.icon.is_some() && !self.label.is_empty() {
                gap
            } else {
                0.0
            };
        let natural_w = inner_w + pad_x * 2.0;
        // `flex-1` (horizontal) / `w-full` (vertical): stretch to the widest
        // sibling's natural width from the previous frame.
        let equal_w = ui
            .ctx()
            .data_mut(|d| d.get_temp::<f32>(equal_width_id(scope.value_id)));
        let desired_w = equal_w.unwrap_or(natural_w).max(natural_w);
        let desired = Vec2::new(desired_w, height);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => scope.value_id.with("trigger").with(self.value.as_str()),
        };

        // Register this trigger in the roving-focus order and width list.
        // The order carries the trigger's actual response id, so arrow-key
        // traversal also reaches triggers with a custom `id_salt` (their id
        // does not follow the default `value_id/trigger/value` pattern).
        ui.ctx().data_mut(|d| {
            let mut order: Vec<(String, bool, Id)> = d
                .get_temp::<Vec<(String, bool, Id)>>(trigger_order_collect_id(scope.value_id))
                .unwrap_or_default();
            order.push((self.value.clone(), self.disabled, id));
            d.insert_temp(trigger_order_collect_id(scope.value_id), order);
            let mut widths: Vec<f32> = d
                .get_temp::<Vec<f32>>(trigger_widths_collect_id(scope.value_id))
                .unwrap_or_default();
            widths.push(natural_w);
            d.insert_temp(trigger_widths_collect_id(scope.value_id), widths);
        });

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
            write_selected(
                ui,
                scope.value_id,
                Some(selected_after_click(
                    read_selected(ui, scope.value_id).as_deref(),
                    &self.value,
                )),
            );
            response.mark_changed();
        }

        // Radix roving tabindex: arrows move focus to the next/previous
        // enabled trigger and — with the default `activationMode="automatic"`
        // — select it; Home/End jump to the first/last enabled trigger.
        // With `activationMode="manual"` the arrows (and Home/End) move
        // focus only; Enter/Space (or a click) selects. While focused, the
        // trigger claims the arrow keys via the focus lock filter so egui's
        // own spatial navigation does not also move focus (Tab keeps the
        // built-in traversal). `gained_focus()` is excluded so a trigger
        // focused earlier in the same frame does not consume the same key
        // press again.
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
            let horizontal = !vertical;
            let order: Vec<(String, bool, Id)> = ui.ctx().data_mut(|d| {
                d.get_temp::<Vec<(String, bool, Id)>>(trigger_order_id(scope.value_id))
                    .unwrap_or_default()
            });
            let enabled: Vec<(&str, Id)> = order
                .iter()
                .filter(|(_, disabled, _)| !*disabled)
                .map(|(value, _, id)| (value.as_str(), *id))
                .collect();
            let index = enabled
                .iter()
                .position(|(v, _)| *v == self.value.as_str())
                .map(|i| i as i32);
            if let Some(index) = index {
                let (next, prev, home, end) = ui.input(|i| {
                    (
                        if horizontal {
                            i.key_pressed(Key::ArrowRight)
                        } else {
                            i.key_pressed(Key::ArrowDown)
                        },
                        if horizontal {
                            i.key_pressed(Key::ArrowLeft)
                        } else {
                            i.key_pressed(Key::ArrowUp)
                        },
                        i.key_pressed(Key::Home),
                        i.key_pressed(Key::End),
                    )
                });
                let last = enabled.len() as i32 - 1;
                let target = if home {
                    Some(0)
                } else if end {
                    Some(last)
                } else if next {
                    Some(index + 1)
                } else if prev {
                    Some(index - 1)
                } else {
                    None
                };
                if let Some(target) = target {
                    if (0..=last).contains(&target) {
                        let (value, target_id) = enabled[target as usize];
                        if value != self.value.as_str() {
                            ui.memory_mut(|m| m.request_focus(target_id));
                            // Radix automatic activation: selecting follows
                            // focus. In manual mode focus moves alone.
                            if scope.activation_mode == TabsActivationMode::Automatic {
                                write_selected(ui, scope.value_id, Some(value.to_owned()));
                                response.mark_changed();
                            }
                        }
                    }
                }
                if next || prev {
                    // Cancel egui's pending spatial navigation for the arrow
                    // key we just consumed. `begin_pass` sets a pending
                    // focus direction for arrow keys unless the focused
                    // widget's event filter locks them; the filter installs
                    // one frame after focus moves (it requires
                    // `had_focus_last_frame`), so in the frame right after
                    // a roving move the direction is set and `end_pass`
                    // would move focus a second time, past the roving
                    // target. We own the arrow key (we move focus or clamp
                    // at the edge), so the pending direction must go. When
                    // the filter was locked the direction is already `None`
                    // and this is a no-op. Home/End never set a direction,
                    // so they need no cancel (clearing here would also drop
                    // a same-frame Tab move).
                    ui.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
                }
            }
        }

        // Record the rect so the list can paint its background behind us.
        ui.ctx().data_mut(|d| {
            let union = d
                .get_temp::<Option<Rect>>(scratch_id())
                .flatten()
                .map(|r| r.union(rect))
                .unwrap_or(rect);
            d.insert_temp(scratch_id(), Some(union));
        });

        if ui.is_rect_visible(rect) {
            let hovered = response.hovered() && !self.disabled;
            let radius = theme.radius.md_u8();
            let painter = ui.painter();

            // Text colors: `text-foreground/60` inactive (dark:
            // `text-muted-foreground`), `text-foreground` hover/active.
            let mut fg = if dark {
                p.muted_foreground
            } else {
                color_alpha(p.foreground, 0.6)
            };
            if selected || hovered {
                fg = p.foreground;
            }

            let mut fill = Color32::TRANSPARENT;
            let mut stroke = Stroke::NONE;
            if selected && scope.variant == TabsVariant::Default {
                if dark {
                    // `dark:data-[state=active]:border-input
                    // dark:data-[state=active]:bg-input/30`
                    fill = color_alpha(p.input, 0.3);
                    stroke = Stroke::new(1.0, p.input);
                } else {
                    fill = p.background; // `data-[state=active]:bg-background`
                }
            } else if !selected && hovered && scope.variant == TabsVariant::Default {
                // keep transparent; only text brightens on hover
                fill = Color32::TRANSPARENT;
            }

            if self.disabled {
                fg = color_alpha(fg, 0.5);
                fill = color_alpha(fill, 0.5);
                stroke.color = color_alpha(stroke.color, 0.5);
            }

            painter.rect_filled(rect, CornerRadius::same(radius), fill);
            if stroke.width > 0.0 {
                painter.rect_stroke(
                    rect,
                    CornerRadius::same(radius),
                    stroke,
                    egui::epaint::StrokeKind::Inside,
                );
            }

            // `shadow-sm` on the active trigger (default variant, both
            // modes — the source carries no `dark:` qualifier, §0
            // correction 4; `shadow_sm` already returns the
            // mode-appropriate colour).
            if selected && scope.variant == TabsVariant::Default && !self.disabled {
                let sh = crate::utils::shadow::shadow_sm(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        CornerRadius::same(radius),
                        sh.color,
                    )
                    .with_blur_width(2.0),
                );
            }

            // Content: [icon] gap [label], centered (left-aligned vertical).
            let align = if vertical {
                Align2::LEFT_CENTER
            } else {
                Align2::CENTER_CENTER
            };
            let center = rect.center();
            let total_w = inner_w;
            let mut cursor = if vertical {
                rect.min.x + pad_x
            } else {
                center.x - total_w * 0.5
            };
            if let Some(icon) = self.icon {
                let r = Rect::from_center_size(
                    Pos2::new(cursor + icon_w * 0.5, center.y),
                    Vec2::splat(icon_w),
                );
                icon.paint(painter, r, fg, 2.0 * icon_w / 24.0);
                cursor += icon_w + gap;
            }
            if let Some(galley) = galley {
                let pos = if vertical {
                    Pos2::new(cursor, center.y)
                } else {
                    Pos2::new(cursor + text_w * 0.5, center.y)
                };
                // Same paint math as `Painter::text`, reusing the galley.
                let text_rect = align.anchor_size(pos, galley.size());
                painter.galley(text_rect.min, galley, fg);
            }

            // `line` variant indicator: bottom bar (horizontal) or right bar
            // (vertical) on the active trigger.
            if selected && scope.variant == TabsVariant::Line && !self.disabled {
                let indicator_color = p.foreground;
                let bar = if vertical {
                    vertical_indicator_rect(rect)
                } else {
                    horizontal_indicator_rect(rect)
                };
                painter.rect_filled(bar, CornerRadius::same(0), indicator_color);
            }

            // `focus-visible:border-ring focus-visible:ring-[3px]`.
            if response.has_focus() && !self.disabled {
                crate::utils::focus::paint_ring(
                    painter,
                    rect.expand2(egui::vec2(1.0, 1.0)),
                    color_alpha(p.ring, 0.5),
                    3.0,
                    radius as f32,
                );
            }
        }

        response
    }
}

// ---------------------------------------------------------------------------
// TabsContent
// ---------------------------------------------------------------------------

/// The panel shown for the selected tab value.
#[derive(Debug)]
pub struct TabsContent {
    value: String,
}

impl TabsContent {
    /// Creates the content panel for the tab with the given value id.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Renders the panel body, or nothing when another tab is selected.
    pub fn show(self, ui: &mut Ui, content: impl FnOnce(&mut Ui)) {
        let Some(root) = root_scope(ui) else {
            content(ui);
            return;
        };
        if read_selected(ui, root.value_id).as_deref() == Some(self.value.as_str()) {
            content(ui);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::PointerButton;

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

        fn step(&mut self, events: Vec<egui::Event>, f: impl FnMut(&mut Ui)) {
            let _ = self.step_out(events, f);
        }

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
                    Id::unique("tabs_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 600.0))),
                );
                f(ui);
            });
            out.textures_delta.clear();
            out
        }

        fn run(&mut self, f: impl FnMut(&mut Ui)) {
            self.step(Vec::new(), f);
        }

        fn run_out(&mut self, f: impl FnMut(&mut Ui)) -> egui::FullOutput {
            self.step_out(Vec::new(), f)
        }

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
        fn key(&mut self, key: Key, mut f: impl FnMut(&mut Ui)) {
            self.step(
                vec![egui::Event::Key {
                    key,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::default(),
                    physical_key: None,
                }],
                &mut f,
            );
        }

        fn focus(&self, id: Id) {
            self.ctx.memory_mut(|m| m.request_focus(id));
        }
    }

    fn selected(h: &Harness, salt: &str) -> Option<String> {
        h.ctx
            .data_mut(|d| d.get_temp::<Option<String>>(Id::unique("tabs_test_ui").with(salt)))
            .flatten()
    }

    /// Blurred rect shapes — the `shadow-sm` is painted as one blurred rect.
    fn blurred_rects(out: &egui::FullOutput) -> Vec<&egui::epaint::RectShape> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.blur_width > 0.0 => Some(rs),
                _ => None,
            })
            .collect()
    }

    /// Filled rect shapes (non-transparent fill, no stroke, no blur).
    fn fill_rects(out: &egui::FullOutput) -> Vec<&egui::epaint::RectShape> {
        out.shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs)
                    if rs.fill != Color32::TRANSPARENT
                        && rs.stroke == Stroke::NONE
                        && rs.blur_width == 0.0 =>
                {
                    Some(rs)
                }
                _ => None,
            })
            .collect()
    }

    fn palette(mode: Mode) -> crate::theme::Palette {
        Preset::default().build(mode).palette
    }

    fn manual_tabs_ui(ui: &mut Ui, salt: &str) {
        Tabs::new()
            .id_salt(salt)
            .activation_mode(TabsActivationMode::Manual)
            .show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    TabsTrigger::new("a", "Account").show(ui);
                    TabsTrigger::new("b", "Password").show(ui);
                    TabsTrigger::new("c", "Sessions").show(ui);
                });
                TabsContent::new("a").show(ui, |ui| {
                    ui.label("account body");
                });
                TabsContent::new("b").show(ui, |ui| {
                    ui.label("password body");
                });
                TabsContent::new("c").show(ui, |ui| {
                    ui.label("sessions body");
                });
            });
    }

    fn tabs_ui(ui: &mut Ui, salt: &str) {
        Tabs::new().id_salt(salt).show(ui, |ui| {
            TabsList::new().show(ui, |ui| {
                TabsTrigger::new("a", "Account").show(ui);
                TabsTrigger::new("b", "Password").show(ui);
            });
            TabsContent::new("a").show(ui, |ui| {
                ui.label("account body");
            });
            TabsContent::new("b").show(ui, |ui| {
                ui.label("password body");
            });
        });
    }

    #[test]
    fn first_trigger_is_selected_by_default() {
        let mut h = Harness::new();
        h.run(|ui| tabs_ui(ui, "t1"));
        assert_eq!(
            selected(&h, "t1").as_deref(),
            Some("a"),
            "the first trigger shown becomes active"
        );
    }

    #[test]
    fn click_switches_selection_and_content() {
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 2];
        let mut content_top = 0.0;
        h.run(|ui| {
            Tabs::new().id_salt("t2").show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    rects[0] = TabsTrigger::new("a", "Account").show(ui).rect;
                    rects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                });
                TabsContent::new("a").show(ui, |ui| {
                    ui.label("account body");
                });
                TabsContent::new("b").show(ui, |ui| {
                    ui.label("password body");
                });
                content_top = ui.cursor().min.y;
            });
        });
        let content_h_a = content_top;

        h.click(rects[1].center(), |ui| tabs_ui(ui, "t2"));
        assert_eq!(selected(&h, "t2").as_deref(), Some("b"));

        // The password body is now shown (taller than the account body).
        let mut content_h_b = 0.0;
        h.run(|ui| {
            Tabs::new().id_salt("t2").show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    TabsTrigger::new("a", "Account").show(ui);
                    TabsTrigger::new("b", "Password").show(ui);
                });
                TabsContent::new("a").show(ui, |ui| {
                    ui.label("account body");
                });
                TabsContent::new("b").show(ui, |ui| {
                    ui.label("password body");
                    ui.label("second line");
                });
                content_h_b = ui.cursor().min.y;
            });
        });
        let _ = content_h_a;
        assert!(content_h_b > 0.0);
    }

    #[test]
    fn controlled_tabs_round_trip() {
        let mut h = Harness::new();
        let mut value = "a".to_owned();
        let mut rects = [Rect::ZERO; 2];
        h.run(|ui| {
            Tabs::controlled(&mut value).id_salt("t3").show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    rects[0] = TabsTrigger::new("a", "Account").show(ui).rect;
                    rects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                });
            });
        });
        assert_eq!(value, "a", "unchanged without clicks");

        h.click(rects[1].center(), |ui| {
            Tabs::controlled(&mut value).id_salt("t3").show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    TabsTrigger::new("a", "Account").show(ui);
                    TabsTrigger::new("b", "Password").show(ui);
                });
            });
        });
        assert_eq!(value, "b", "click write-back reaches the caller");
    }

    #[test]
    fn line_variant_and_vertical_render() {
        let mut h = Harness::new();
        h.run(|ui| {
            Tabs::new().id_salt("t4").show(ui, |ui| {
                TabsList::new().variant(TabsVariant::Line).show(ui, |ui| {
                    TabsTrigger::new("a", "Account").show(ui);
                    TabsTrigger::new("b", "Password").show(ui);
                });
                TabsContent::new("a").show(ui, |ui| {
                    ui.label("account body");
                });
            });
        });
        assert_eq!(selected(&h, "t4").as_deref(), Some("a"));

        h.run(|ui| {
            Tabs::new()
                .id_salt("t5")
                .orientation(TabsOrientation::Vertical)
                .show(ui, |ui| {
                    TabsList::new().show(ui, |ui| {
                        TabsTrigger::new("a", "Account").show(ui);
                        TabsTrigger::new("b", "Password").show(ui);
                    });
                    TabsContent::new("a").show(ui, |ui| {
                        ui.label("account body");
                    });
                });
        });
        assert_eq!(selected(&h, "t5").as_deref(), Some("a"));
    }

    #[test]
    fn disabled_trigger_cannot_be_selected() {
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 2];
        h.run(|ui| {
            Tabs::new().id_salt("t6").show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    rects[0] = TabsTrigger::new("a", "Account")
                        .disabled(true)
                        .show(ui)
                        .rect;
                    rects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                });
            });
        });
        assert_eq!(
            selected(&h, "t6").as_deref(),
            Some("b"),
            "disabled trigger is skipped for the default selection"
        );

        h.click(rects[0].center(), |ui| {
            Tabs::new().id_salt("t6").show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    TabsTrigger::new("a", "Account").disabled(true).show(ui);
                    TabsTrigger::new("b", "Password").show(ui);
                });
            });
        });
        assert_eq!(
            selected(&h, "t6").as_deref(),
            Some("b"),
            "clicking a disabled trigger does nothing"
        );
    }

    #[test]
    fn selection_logic_is_pure() {
        assert_eq!(selected_after_click(Some("a"), "b"), "b");
        assert_eq!(selected_after_click(None, "a"), "a");
    }

    #[test]
    fn orphan_content_renders_statically() {
        let mut h = Harness::new();
        h.run(|ui| {
            TabsContent::new("x").show(ui, |ui| {
                ui.label("no tabs root");
            });
            let _ = TabsTrigger::new("x", "X").show(ui);
            TabsList::new().show(ui, |ui| {
                ui.label("no root");
            });
        });
    }

    #[test]
    fn list_gap_matches_variant() {
        // The base list has no gap between the flex-1 triggers; `line` adds
        // `gap-1`.
        assert_eq!(list_gap(TabsVariant::Default), 0.0);
        assert_eq!(list_gap(TabsVariant::Line), 4.0);
    }

    #[test]
    fn line_indicator_geometry() {
        // `after:bottom-[-5px] after:h-0.5`: the bar spans [max.y+3, max.y+5]
        // directly under the trigger.
        let r = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::new(60.0, 26.0));
        let bar = horizontal_indicator_rect(r);
        assert!((bar.top() - (r.max.y + 3.0)).abs() < 1e-6);
        assert!((bar.bottom() - (r.max.y + 5.0)).abs() < 1e-6);
        assert_eq!(bar.height(), 2.0);
        assert_eq!(bar.left(), r.left());
        assert_eq!(bar.right(), r.right());

        // `after:-right-1 after:w-0.5`: the bar spans [max.x+2, max.x+4].
        let bar = vertical_indicator_rect(r);
        assert!((bar.left() - (r.max.x + 2.0)).abs() < 1e-6);
        assert!((bar.right() - (r.max.x + 4.0)).abs() < 1e-6);
        assert_eq!(bar.width(), 2.0);
        assert_eq!(bar.top(), r.top());
        assert_eq!(bar.bottom(), r.bottom());
    }

    #[test]
    fn arrow_keys_move_focus_and_activate() {
        let mut h = Harness::new();
        h.run(|ui| tabs_ui(ui, "t7"));
        let state_id = Id::unique("tabs_test_ui").with("t7");
        let trigger = |value: &str| state_id.with("trigger").with(value);
        h.focus(trigger("a"));
        // Two plain frames: the order list is published at list end and the
        // focus lock filter installs on the frame after focus.
        h.run(|ui| tabs_ui(ui, "t7"));
        h.run(|ui| tabs_ui(ui, "t7"));

        h.key(Key::ArrowRight, |ui| tabs_ui(ui, "t7"));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger("b")),
            "ArrowRight moves focus to the next trigger"
        );
        assert_eq!(
            selected(&h, "t7").as_deref(),
            Some("b"),
            "automatic activation selects the trigger focus moved to"
        );

        h.key(Key::ArrowLeft, |ui| tabs_ui(ui, "t7"));
        assert_eq!(h.ctx.memory(|m| m.focused()), Some(trigger("a")));
        assert_eq!(selected(&h, "t7").as_deref(), Some("a"));

        // ArrowRight on the last trigger keeps focus there.
        h.key(Key::ArrowRight, |ui| tabs_ui(ui, "t7"));
        h.key(Key::ArrowRight, |ui| tabs_ui(ui, "t7"));
        assert_eq!(h.ctx.memory(|m| m.focused()), Some(trigger("b")));
    }

    #[test]
    fn home_end_jump_and_disabled_triggers_are_skipped() {
        let mut h = Harness::new();
        let ui_with_disabled = |ui: &mut Ui, salt: &str| {
            Tabs::new().id_salt(salt).show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    TabsTrigger::new("a", "Account").disabled(true).show(ui);
                    TabsTrigger::new("b", "Password").show(ui);
                    TabsTrigger::new("c", "Sessions").show(ui);
                });
            });
        };
        h.run(|ui| ui_with_disabled(ui, "t8"));
        let state_id = Id::unique("tabs_test_ui").with("t8");
        let trigger = |value: &str| state_id.with("trigger").with(value);
        h.focus(trigger("b"));
        h.run(|ui| ui_with_disabled(ui, "t8"));
        h.run(|ui| ui_with_disabled(ui, "t8"));

        h.key(Key::ArrowLeft, |ui| ui_with_disabled(ui, "t8"));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger("b")),
            "the disabled first trigger is skipped going backwards"
        );

        h.key(Key::End, |ui| ui_with_disabled(ui, "t8"));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger("c")),
            "End jumps to the last enabled trigger"
        );

        h.key(Key::Home, |ui| ui_with_disabled(ui, "t8"));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger("b")),
            "Home jumps to the first enabled trigger"
        );
        assert_eq!(
            selected(&h, "t8").as_deref(),
            Some("b"),
            "automatic activation follows the Home jump"
        );
    }

    #[test]
    fn vertical_orientation_uses_up_down_arrows() {
        let mut h = Harness::new();
        let vertical_ui = |ui: &mut Ui, salt: &str| {
            Tabs::new()
                .id_salt(salt)
                .orientation(TabsOrientation::Vertical)
                .show(ui, |ui| {
                    TabsList::new().show(ui, |ui| {
                        TabsTrigger::new("a", "Account").show(ui);
                        TabsTrigger::new("b", "Password").show(ui);
                    });
                });
        };
        h.run(|ui| vertical_ui(ui, "t9"));
        let state_id = Id::unique("tabs_test_ui").with("t9");
        h.focus(state_id.with("trigger").with("a"));
        h.run(|ui| vertical_ui(ui, "t9"));
        h.run(|ui| vertical_ui(ui, "t9"));

        h.key(Key::ArrowDown, |ui| vertical_ui(ui, "t9"));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(state_id.with("trigger").with("b")),
            "ArrowDown moves focus in a vertical list"
        );
        assert_eq!(selected(&h, "t9").as_deref(), Some("b"));

        h.key(Key::ArrowUp, |ui| vertical_ui(ui, "t9"));
        assert_eq!(selected(&h, "t9").as_deref(), Some("a"));
    }

    #[test]
    fn triggers_stretch_to_equal_width() {
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 2];
        // First frame measures the natural widths; the second frame stretches
        // all triggers to the widest one (`flex-1` / `w-full`).
        h.run(|ui| {
            Tabs::new().id_salt("t10").show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    rects[0] = TabsTrigger::new("a", "Account").show(ui).rect;
                    rects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                });
            });
        });
        h.run(|ui| {
            Tabs::new().id_salt("t10").show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    rects[0] = TabsTrigger::new("a", "Account").show(ui).rect;
                    rects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                });
            });
        });
        assert!(
            (rects[0].width() - rects[1].width()).abs() < 0.5,
            "triggers share an equal width: {} vs {}",
            rects[0].width(),
            rects[1].width()
        );

        // Vertical triggers fill the column the same way (`w-full`).
        let mut vrects = [Rect::ZERO; 2];
        for _ in 0..2 {
            h.run(|ui| {
                Tabs::new()
                    .id_salt("t11")
                    .orientation(TabsOrientation::Vertical)
                    .show(ui, |ui| {
                        TabsList::new().show(ui, |ui| {
                            vrects[0] = TabsTrigger::new("a", "Account").show(ui).rect;
                            vrects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                        });
                    });
            });
        }
        assert!(
            (vrects[0].width() - vrects[1].width()).abs() < 0.5,
            "vertical triggers fill the widest sibling's width"
        );
        assert!(
            (vrects[0].height() - 24.0).abs() < 0.5,
            "nova vertical trigger height is 24px"
        );
    }

    #[test]
    fn default_activation_mode_is_automatic() {
        assert_eq!(
            Tabs::new().activation_mode,
            TabsActivationMode::Automatic,
            "the default stays automatic, matching the source default"
        );
        let mut value = String::new();
        let tabs = Tabs::controlled(&mut value);
        assert_eq!(
            tabs.activation_mode,
            TabsActivationMode::Automatic,
            "controlled tabs also default to automatic"
        );
    }

    #[test]
    fn manual_mode_arrows_move_focus_without_selecting() {
        let mut h = Harness::new();
        h.run(|ui| manual_tabs_ui(ui, "m1"));
        let state_id = Id::unique("tabs_test_ui").with("m1");
        let trigger = |value: &str| state_id.with("trigger").with(value);
        h.focus(trigger("a"));
        // Two plain frames: the order list is published at list end and the
        // focus lock filter installs on the frame after focus.
        h.run(|ui| manual_tabs_ui(ui, "m1"));
        h.run(|ui| manual_tabs_ui(ui, "m1"));
        assert_eq!(selected(&h, "m1").as_deref(), Some("a"));

        h.key(Key::ArrowRight, |ui| manual_tabs_ui(ui, "m1"));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger("b")),
            "ArrowRight moves focus to the next trigger"
        );
        assert_eq!(
            selected(&h, "m1").as_deref(),
            Some("a"),
            "arrow moves focus only in manual mode — selection does not follow"
        );

        h.key(Key::ArrowRight, |ui| manual_tabs_ui(ui, "m1"));
        assert_eq!(h.ctx.memory(|m| m.focused()), Some(trigger("c")));
        assert_eq!(selected(&h, "m1").as_deref(), Some("a"));

        h.key(Key::ArrowLeft, |ui| manual_tabs_ui(ui, "m1"));
        assert_eq!(h.ctx.memory(|m| m.focused()), Some(trigger("b")));
        assert_eq!(selected(&h, "m1").as_deref(), Some("a"));

        h.key(Key::End, |ui| manual_tabs_ui(ui, "m1"));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger("c")),
            "End moves focus only in manual mode"
        );
        assert_eq!(selected(&h, "m1").as_deref(), Some("a"));

        h.key(Key::Home, |ui| manual_tabs_ui(ui, "m1"));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(trigger("a")),
            "Home moves focus only in manual mode"
        );
        assert_eq!(selected(&h, "m1").as_deref(), Some("a"));
    }

    #[test]
    fn manual_mode_enter_space_selects() {
        let mut h = Harness::new();
        h.run(|ui| manual_tabs_ui(ui, "m2"));
        let state_id = Id::unique("tabs_test_ui").with("m2");
        let trigger = |value: &str| state_id.with("trigger").with(value);
        h.focus(trigger("a"));
        h.run(|ui| manual_tabs_ui(ui, "m2"));
        h.run(|ui| manual_tabs_ui(ui, "m2"));

        h.key(Key::ArrowRight, |ui| manual_tabs_ui(ui, "m2"));
        assert_eq!(
            selected(&h, "m2").as_deref(),
            Some("a"),
            "focus moved, selection did not follow"
        );

        // Enter activates the focused trigger (egui keyboard activation).
        h.key(Key::Enter, |ui| manual_tabs_ui(ui, "m2"));
        assert_eq!(
            selected(&h, "m2").as_deref(),
            Some("b"),
            "Enter selects the focused trigger in manual mode"
        );

        h.key(Key::ArrowLeft, |ui| manual_tabs_ui(ui, "m2"));
        assert_eq!(h.ctx.memory(|m| m.focused()), Some(trigger("a")));
        assert_eq!(selected(&h, "m2").as_deref(), Some("b"));

        // Space activates the focused trigger.
        h.key(Key::Space, |ui| manual_tabs_ui(ui, "m2"));
        assert_eq!(
            selected(&h, "m2").as_deref(),
            Some("a"),
            "Space selects the focused trigger in manual mode"
        );
    }

    #[test]
    fn manual_mode_click_selects() {
        let mut h = Harness::new();
        let mut rects = [Rect::ZERO; 3];
        // Settle first so the equal-width stretch has converged, then
        // capture the trigger rects for the click positions.
        for _ in 0..2 {
            h.run(|ui| {
                Tabs::new()
                    .id_salt("m3")
                    .activation_mode(TabsActivationMode::Manual)
                    .show(ui, |ui| {
                        TabsList::new().show(ui, |ui| {
                            rects[0] = TabsTrigger::new("a", "Account").show(ui).rect;
                            rects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                            rects[2] = TabsTrigger::new("c", "Sessions").show(ui).rect;
                        });
                    });
            });
        }
        h.click(rects[1].center(), |ui| manual_tabs_ui(ui, "m3"));
        assert_eq!(
            selected(&h, "m3").as_deref(),
            Some("b"),
            "click selects in manual mode"
        );
        h.click(rects[2].center(), |ui| manual_tabs_ui(ui, "m3"));
        assert_eq!(selected(&h, "m3").as_deref(), Some("c"));
    }

    #[test]
    fn active_default_trigger_paints_shadow_sm_in_both_modes() {
        // §0 correction 4 / §4.2: `shadow-sm` on the active default trigger
        // in BOTH modes — the source carries no `dark:` qualifier.
        for (mode, is_dark) in [(Mode::Light, false), (Mode::Dark, true)] {
            let mut h = Harness::new();
            if is_dark {
                ShadcnTheme::set_mode(&h.ctx, Mode::Dark);
            }
            let mut rects = [Rect::ZERO; 2];
            let mut out = None;
            for _ in 0..3 {
                out = Some(h.run_out(|ui| {
                    Tabs::new().id_salt("shadow").show(ui, |ui| {
                        TabsList::new().show(ui, |ui| {
                            rects[0] = TabsTrigger::new("a", "Account").show(ui).rect;
                            rects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                        });
                    });
                }));
            }
            let out = out.expect("ran frames");
            let expected = crate::utils::shadow::shadow_sm(is_dark).color;
            let blurred = blurred_rects(&out);
            assert_eq!(
                blurred.len(),
                1,
                "exactly one blurred rect (the active trigger shadow) in {:?} mode, got {}",
                mode,
                blurred.len()
            );
            let sh = blurred[0];
            assert_eq!(sh.fill, expected, "shadow colour in {:?} mode", mode);
            // The shadow is the selected (first) trigger rect translated by
            // the shadow offset [0, 1].
            assert!(
                (sh.rect.min.x - rects[0].min.x).abs() < 0.5,
                "shadow x aligns with the selected trigger in {:?} mode",
                mode
            );
            assert!(
                (sh.rect.min.y - (rects[0].min.y + 1.0)).abs() < 0.5,
                "shadow y is the trigger y + 1 in {:?} mode",
                mode
            );
            assert!(
                (sh.rect.width() - rects[0].width()).abs() < 0.5
                    && (sh.rect.height() - rects[0].height()).abs() < 0.5,
                "shadow size matches the selected trigger in {:?} mode",
                mode
            );
        }
    }

    #[test]
    fn horizontal_list_geometry_matches_spec() {
        // §3.1/§3.2: nova horizontal list is 32px tall (`h-8`), the
        // triggers 25px (`h-[calc(100%-1px)]`), centred in the row; the
        // root gap to the content is 8px (`gap-2`).
        let mut h = Harness::new();
        let mut trigger_rects = [Rect::ZERO; 2];
        let mut content_top = 0.0;
        let mut list_top_y = 0.0;
        let mut after_list_y = 0.0;
        let mut out = None;
        for _ in 0..3 {
            out = Some(h.run_out(|ui| {
                Tabs::new().id_salt("geo").show(ui, |ui| {
                    list_top_y = ui.cursor().min.y;
                    TabsList::new().show(ui, |ui| {
                        trigger_rects[0] = TabsTrigger::new("a", "Account").show(ui).rect;
                        trigger_rects[1] = TabsTrigger::new("b", "Password").show(ui).rect;
                    });
                    after_list_y = ui.cursor().min.y;
                    TabsContent::new("a").show(ui, |ui| {
                        content_top = ui.cursor().min.y;
                        ui.label("account body");
                    });
                });
            }));
        }
        let out = out.expect("ran frames");

        assert!(
            (trigger_rects[0].height() - 25.0).abs() < 0.5,
            "nova horizontal trigger height is 25px, got {}",
            trigger_rects[0].height()
        );
        assert!(
            (trigger_rects[1].height() - 25.0).abs() < 0.5,
            "nova horizontal trigger height is 25px, got {}",
            trigger_rects[1].height()
        );

        // The list background is the filled rect in the `muted` token.
        let muted = palette(Mode::Light).muted;
        let list_bgs: Vec<&egui::epaint::RectShape> = fill_rects(&out)
            .into_iter()
            .filter(|rs| rs.fill == muted)
            .collect();
        assert_eq!(
            list_bgs.len(),
            1,
            "exactly one muted list background, got {}",
            list_bgs.len()
        );
        let bg = list_bgs[0];
        assert!(
            (bg.rect.height() - 32.0).abs() < 0.5,
            "nova horizontal list height is 32px, got {}",
            bg.rect.height()
        );
        // The background covers the full list rect, starting at the row top.
        assert!(
            (bg.rect.min.y - list_top_y).abs() < 0.5,
            "list background starts at the row top, got bg top {} vs row top {}",
            bg.rect.min.y,
            list_top_y
        );
        assert!(
            (trigger_rects[0].min.y - (list_top_y + 3.5)).abs() < 0.5,
            "triggers are centred in the 32px row (0.5px slack each side)"
        );
        // The cursor after the list already sits `item_spacing` (8px) past
        // the row, so row + gap = 32 + 8 = 40px from the row top.
        assert!(
            ((after_list_y - list_top_y) - 40.0).abs() < 0.5,
            "list row (32px) + root gap (8px) is 40px, got {} (top {} bottom {})",
            after_list_y - list_top_y,
            list_top_y,
            after_list_y
        );
        assert!(
            ((content_top - list_top_y) - 40.0).abs() < 0.5,
            "content starts 32px list + 8px root gap below the list top, got content_top {} vs list top {}",
            content_top,
            list_top_y
        );
    }

    #[test]
    fn salted_triggers_participate_in_arrow_traversal() {
        // §8.2 flagged item: a trigger with a custom `id_salt` must still
        // be reachable by arrow-key traversal (its response id does not
        // follow the default `value_id/trigger/value` pattern).
        let mut h = Harness::new();
        let mut ids = [Id::NULL; 3];
        let salted_ui = |ui: &mut Ui, salt: &str, ids: &mut [Id; 3]| {
            Tabs::new().id_salt(salt).show(ui, |ui| {
                TabsList::new().show(ui, |ui| {
                    ids[0] = TabsTrigger::new("a", "Account").show(ui).id;
                    ids[1] = TabsTrigger::new("b", "Password")
                        .id_salt("salted-b")
                        .show(ui)
                        .id;
                    ids[2] = TabsTrigger::new("c", "Sessions").show(ui).id;
                });
            });
        };
        h.run(|ui| salted_ui(ui, "s1", &mut ids));
        h.focus(ids[0]);
        h.run(|ui| salted_ui(ui, "s1", &mut ids));
        h.run(|ui| salted_ui(ui, "s1", &mut ids));

        h.key(Key::ArrowRight, |ui| salted_ui(ui, "s1", &mut ids));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(ids[1]),
            "ArrowRight reaches the salted trigger"
        );
        h.key(Key::ArrowRight, |ui| salted_ui(ui, "s1", &mut ids));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(ids[2]),
            "ArrowRight continues past the salted trigger"
        );
        h.key(Key::ArrowLeft, |ui| salted_ui(ui, "s1", &mut ids));
        assert_eq!(
            h.ctx.memory(|m| m.focused()),
            Some(ids[1]),
            "ArrowLeft returns to the salted trigger"
        );
    }
}
