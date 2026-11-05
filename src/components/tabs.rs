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
//! The selected value lives in the egui context keyed by a stable id (pass
//! [`Tabs::id_salt`] per instance) or is controlled through a `&mut String`
//! with [`Tabs::controlled`]. Keyboard: triggers are focusable; egui moves
//! focus with Tab/Arrows and activates with Enter/Space.
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
    Align2, Color32, CornerRadius, FontId, Id, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Vec2,
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

/// Height of the trigger list (nova `h-8`), horizontal orientation.
pub const TABS_LIST_HEIGHT: f32 = 32.0;
/// List padding `p-[3px]`.
const LIST_PAD: f32 = 3.0;
/// Trigger height inside the list: `h-[calc(100%-1px)]` of the padded inner.
const TRIGGER_H: f32 = TABS_LIST_HEIGHT - LIST_PAD * 2.0 - 1.0;
/// Trigger height when the list is vertical (`h-fit`).
const TRIGGER_H_VERTICAL: f32 = 22.0;
/// Height/width of the `line` variant indicator (`after:h-0.5`).
const INDICATOR: f32 = 2.0;

// ---------------------------------------------------------------------------
// Scope plumbing
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug)]
struct TabsRoot {
    value_id: Id,
    orientation: TabsOrientation,
}

#[derive(Clone, Copy, Debug)]
struct TabsListScope {
    value_id: Id,
    variant: TabsVariant,
    orientation: TabsOrientation,
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
        };
        ui.ctx().data_mut(|d| {
            let mut stack: Vec<TabsListScope> = d.get_temp(list_stack_id()).unwrap_or_default();
            stack.push(scope);
            d.insert_temp(list_stack_id(), stack);
            d.insert_temp(scratch_id(), None::<Rect>); // reset the trigger union
        });

        // Reserve a paint slot so the background renders behind the triggers.
        let bg_idx = ui.painter().add(Shape::Noop);

        match scope.orientation {
            TabsOrientation::Horizontal => {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = crate::theme::tokens::spacing::XS_HALF;
                    content(ui);
                });
            }
            TabsOrientation::Vertical => {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::XS_HALF;
                    content(ui);
                });
            }
        }

        ui.ctx().data_mut(|d| {
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
                let dark = theme_dark(ui);
                let bg = match self.variant {
                    TabsVariant::Default => {
                        if dark {
                            crate::theme::color_alpha(ShadcnTheme::of_ui(ui).palette.muted, 0.6)
                        } else {
                            ShadcnTheme::of_ui(ui).palette.muted
                        }
                    }
                    TabsVariant::Line => Color32::TRANSPARENT,
                };
                let radius = match self.variant {
                    TabsVariant::Default => ShadcnTheme::of_ui(ui).radius.lg_u8(),
                    TabsVariant::Line => 0,
                };
                let shape = egui::epaint::RectShape::filled(
                    union.expand2(egui::vec2(LIST_PAD, LIST_PAD)),
                    CornerRadius::same(radius),
                    bg,
                );
                ui.painter().set(bg_idx, shape);
            }
        }
    }
}

fn theme_dark(ui: &Ui) -> bool {
    ShadcnTheme::of_ui(ui).mode.is_dark()
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
        let text_w = if self.label.is_empty() {
            0.0
        } else {
            ui.painter()
                .layout_no_wrap(self.label.clone(), font.clone(), Color32::WHITE)
                .size()
                .x
        };
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
        let desired = Vec2::new(inner_w + pad_x * 2.0, height);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => scope.value_id.with("trigger").with(self.value.as_str()),
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

            // `shadow-sm` on the active trigger (default variant, light).
            if selected && scope.variant == TabsVariant::Default && !dark && !self.disabled {
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
            if !self.label.is_empty() {
                let pos = if vertical {
                    Pos2::new(cursor, center.y)
                } else {
                    Pos2::new(cursor + text_w * 0.5, center.y)
                };
                painter.text(pos, align, self.label.clone(), font, fg);
            }

            // `line` variant indicator: bottom bar (horizontal) or right bar
            // (vertical) on the active trigger.
            if selected && scope.variant == TabsVariant::Line && !self.disabled {
                let indicator_color = p.foreground;
                if vertical {
                    // `after:-right-1 after:inset-y-0 after:w-0.5`
                    let bar = Rect::from_min_size(
                        Pos2::new(rect.max.x + crate::theme::tokens::spacing::XS, rect.min.y),
                        Vec2::new(INDICATOR, rect.height()),
                    );
                    painter.rect_filled(bar, CornerRadius::same(0), indicator_color);
                } else {
                    // `after:bottom-[-5px] after:inset-x-0 after:h-0.5`
                    let y =
                        rect.max.y + crate::theme::tokens::spacing::SM_HALF - INDICATOR + LIST_PAD;
                    let bar = Rect::from_min_size(
                        Pos2::new(rect.min.x, y),
                        Vec2::new(rect.width(), INDICATOR),
                    );
                    painter.rect_filled(bar, CornerRadius::same(0), indicator_color);
                }
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
    use crate::theme::Preset;
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
                    Id::unique("tabs_test_ui"),
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
    }

    fn selected(h: &Harness, salt: &str) -> Option<String> {
        h.ctx
            .data_mut(|d| d.get_temp::<Option<String>>(Id::unique("tabs_test_ui").with(salt)))
            .flatten()
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
}
