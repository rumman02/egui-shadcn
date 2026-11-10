//! shadcn `sidebar.tsx` — the collapsible sidebar rail and its compound
//! pieces, ported to egui's immediate layout.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/sidebar.tsx` + the nova
//! style CSS (`cn-sidebar-*`):
//!
//! - [`Sidebar`] — a full-height column with the sidebar palette, animated
//!   width (expanded 16rem ↔ icon 3rem, or 0 when `offcanvas`), variants
//!   `sidebar` / `floating` / `inset` and sides left/right. The open state
//!   is uncontrolled (persisted per `id_salt`) or controlled (`&mut bool`).
//! - Header / [`SidebarContent`] (scrollable, `overflow-auto`) /
//!   [`SidebarFooter`] / [`SidebarSeparator`], [`SidebarGroup`] /
//!   [`SidebarGroupLabel`] / [`SidebarGroupAction`] /
//!   [`SidebarGroupContent`], [`SidebarMenu`] / [`SidebarMenuItem`] /
//!   [`SidebarMenuButton`] / [`SidebarMenuAction`] /
//!   [`SidebarMenuSkeleton`], [`SidebarMenuSub`] / [`SidebarMenuSubButton`],
//!   plus [`SidebarTrigger`], [`SidebarRail`] and [`SidebarInset`].
//! - Keyboard: `Ctrl/Cmd+B` toggles (the source's global shortcut), Escape
//!   collapses while a widget inside the sidebar has focus. Menu buttons
//!   show hover tooltips when collapsed to icons, and the rail keeps
//!   working when fully collapsed (offcanvas).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ui.horizontal(|ui| {
//!     Sidebar::new()
//!         .id_salt("app")
//!         .collapsible(SidebarCollapsible::Icon)
//!         .show(ui, |ui| {
//!             SidebarHeader::new().show(ui, |ui| {
//!                 SidebarTrigger::new().show(ui);
//!             });
//!             SidebarContent::new().show(ui, |ui| {
//!                 SidebarGroup::new().show(ui, |ui| {
//!                     SidebarGroupLabel::new("Platform").show(ui);
//!                     SidebarMenu::new().show(ui, |ui| {
//!                         SidebarMenuItem::new().show(ui, |ui| {
//!                             SidebarMenuButton::new("History", Icon::Message)
//!                                 .tooltip("History")
//!                                 .show(ui);
//!                         });
//!                     });
//!                 });
//!             });
//!             SidebarFooter::new().show(ui, |_ui| {});
//!         });
//!     SidebarInset::new().show(ui, |ui| { /* main content */ });
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{
    epaint::CornerRadius, FontFamily, FontId, Id, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2,
};

/// `SIDEBAR_WIDTH` — 16rem.
pub const SIDEBAR_WIDTH: f32 = 256.0;
/// `SIDEBAR_WIDTH_ICON` — 3rem.
pub const SIDEBAR_WIDTH_ICON: f32 = 48.0;

/// Which edge the sidebar sits on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarSide {
    /// Left edge (default).
    #[default]
    Left,
    /// Right edge (border on the left side of the column).
    Right,
}

/// The sidebar decoration variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarVariant {
    /// Plain column with a border on the outer edge (default).
    #[default]
    Sidebar,
    /// Padded, floating rounded card with ring + shadow.
    Floating,
    /// Padded column; pair with [`SidebarInset`] for the rounded main area.
    Inset,
}

/// How the sidebar collapses.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarCollapsible {
    /// Slides fully out of view (default).
    #[default]
    Offcanvas,
    /// Collapses to an icon rail (`size-8` buttons, hidden labels).
    Icon,
    /// Always visible; the trigger does nothing.
    None,
}

/// A menu button size (`default` h-8, `sm` h-7, `lg` h-12).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarMenuButtonSize {
    /// `h-8 text-sm` (default).
    #[default]
    Default,
    /// `h-7 text-xs`.
    Sm,
    /// `h-12 text-sm`.
    Lg,
}

/// A menu button variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarMenuButtonVariant {
    /// Flat, hover-accent (default).
    #[default]
    Default,
    /// `bg-background` with a 1px sidebar-border ring.
    Outline,
}

/// Sub-button text size (`sm` = text-xs, `md` = text-sm).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarMenuSubSize {
    /// `text-xs`.
    Sm,
    /// `text-sm` (default).
    #[default]
    Md,
}

/// Per-frame sidebar context shared with the child slots.
#[derive(Clone, Copy, Debug)]
struct SidebarCtx {
    state_id: Id,
    open: bool,
    collapsible: SidebarCollapsible,
    collapsed_icon: bool,
    variant: SidebarVariant,
    inner_rect: Rect,
}

fn stack_id() -> Id {
    Id::unique("shadcn_sidebar_ctx_stack")
}

fn push_ctx(ui: &Ui, ctx: SidebarCtx) {
    let mut stack: Vec<SidebarCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx().data_mut(|d| d.insert_temp(stack_id(), stack));
}

fn pop_ctx(ui: &Ui) {
    let mut stack: Vec<SidebarCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx().data_mut(|d| d.insert_temp(stack_id(), stack));
}

fn current_ctx(ui: &Ui) -> Option<SidebarCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<SidebarCtx>>(stack_id()))
        .and_then(|s| s.last().copied())
}

/// The memory id base for a sidebar with the given salt.
pub(crate) fn state_id(salt: &str) -> Id {
    Id::unique(format!("shadcn_sidebar_{salt}"))
}

fn open_id(base: Id) -> Id {
    base.with("open")
}

pub(crate) fn read_open(ctx: &egui::Context, base: Id, default_open: bool) -> bool {
    ctx.data_mut(|d| d.get_temp::<bool>(open_id(base)))
        .unwrap_or(default_open)
}

/// Requests a toggle for the sidebar the caller is inside (stack-based).
fn request_toggle(ui: &Ui, base: Id) {
    ui.ctx()
        .data_mut(|d| d.insert_temp(base.with("toggle_intent"), true));
}

/// The sidebar container. See the [module docs](self) for an example.
pub struct Sidebar<'a> {
    side: SidebarSide,
    variant: SidebarVariant,
    collapsible: SidebarCollapsible,
    id_salt: Option<String>,
    controlled: Option<&'a mut bool>,
    width: f32,
    icon_width: f32,
}

impl Default for Sidebar<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Sidebar<'a> {
    /// Creates an uncontrolled sidebar (open by default, state persisted
    /// per `id_salt`).
    pub fn new() -> Self {
        Self {
            side: SidebarSide::Left,
            variant: SidebarVariant::Sidebar,
            collapsible: SidebarCollapsible::Offcanvas,
            id_salt: None,
            controlled: None,
            width: SIDEBAR_WIDTH,
            icon_width: SIDEBAR_WIDTH_ICON,
        }
    }

    /// Creates a sidebar controlled by the caller's bool.
    pub fn controlled(open: &'a mut bool) -> Self {
        Self {
            controlled: Some(open),
            ..Self::new()
        }
    }

    /// Sets the sidebar edge.
    pub fn side(mut self, side: SidebarSide) -> Self {
        self.side = side;
        self
    }

    /// Sets the decoration variant.
    pub fn variant(mut self, variant: SidebarVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the collapse mode.
    pub fn collapsible(mut self, collapsible: SidebarCollapsible) -> Self {
        self.collapsible = collapsible;
        self
    }

    /// Sets a stable id salt (the open state persists under it).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Overrides the expanded width (`--sidebar-width`, 16rem default).
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }

    /// Overrides the icon-collapsed width (`--sidebar-width-icon`, 3rem).
    pub fn icon_width(mut self, width: f32) -> Self {
        self.icon_width = width;
        self
    }

    /// Renders the sidebar column with its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> SidebarResponse {
        let Self {
            side,
            variant,
            collapsible,
            id_salt,
            controlled,
            width,
            icon_width,
        } = self;

        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let base = match id_salt.as_deref() {
            Some(salt) => state_id(salt),
            None => ui.auto_id_with("shadcn_sidebar"),
        };

        // The controlled flag wins at frame start; uncontrolled reads memory.
        let open = match controlled.as_deref() {
            Some(v) => *v,
            None => read_open(ui.ctx(), base, true),
        };

        // Keyboard: Ctrl/Cmd+B toggles (SidebarProvider's global shortcut).
        let kb_toggle =
            ui.input(|i| i.key_pressed(egui::Key::B) && (i.modifiers.ctrl || i.modifiers.command));
        if kb_toggle && collapsible != SidebarCollapsible::None {
            request_toggle(ui, base);
        }

        // Animated outer width: `transition-[width] duration-200 ease-linear`.
        let target = if open {
            width
        } else {
            match collapsible {
                SidebarCollapsible::Icon => match variant {
                    SidebarVariant::Sidebar => icon_width,
                    // `calc(var(--sidebar-width-icon) + theme(spacing.4) + 2px)`
                    _ => icon_width + 18.0, // p-2 padding + 2px ring
                },
                SidebarCollapsible::Offcanvas => 0.0,
                SidebarCollapsible::None => width,
            }
        };
        let outer_w = ui
            .ctx()
            .animate_value_with_time(base.with("width_anim"), target, 0.2)
            .min(ui.available_width().max(0.0));

        let h = ui.available_height().max(0.0);
        let (_, outer) = ui.allocate_space(Vec2::new(outer_w, h));
        let outer = match side {
            SidebarSide::Left => outer,
            SidebarSide::Right => {
                let right = ui.cursor().right();
                Rect::from_min_size(Pos2::new(right - outer.width(), outer.top()), outer.size())
            }
        };

        let painter = ui.painter().clone();
        let collapsed_icon = !open && collapsible == SidebarCollapsible::Icon;

        // Inner column: `sidebar` fills the outer; `floating`/`inset` get p-2.
        let padded = matches!(variant, SidebarVariant::Floating | SidebarVariant::Inset);
        let inner = if padded && outer.width() > 16.0 {
            outer.shrink2(Vec2::splat(8.0))
        } else {
            outer
        };

        // Reserve paint slots: shadow behind, fill above it.
        let shadow_idx = painter.add(egui::Shape::Noop);
        let fill_idx = painter.add(egui::Shape::Noop);

        if inner.width() > 4.0 && inner.height() > 4.0 {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(inner)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            child.spacing_mut().item_spacing.y = 0.0;

            push_ctx(
                ui,
                SidebarCtx {
                    state_id: base,
                    open,
                    collapsible,
                    collapsed_icon,
                    variant,
                    inner_rect: inner,
                },
            );
            let _out = content(&mut child);
            pop_ctx(ui);
        } else {
            // Fully collapsed (offcanvas): the content closure is skipped,
            // but the source's rail keeps working when collapsed
            // (`group-data-[collapsible=offcanvas]:translate-x-0
            // after:left-full`, sidebar.tsx:297-299) — render a built-in
            // rail strip at the edge so the sidebar can be re-opened
            // without an external trigger.
            push_ctx(
                ui,
                SidebarCtx {
                    state_id: base,
                    open,
                    collapsible,
                    collapsed_icon,
                    variant,
                    inner_rect: inner,
                },
            );
            pop_ctx(ui);

            let edge_x = match side {
                SidebarSide::Left => outer.right(),
                SidebarSide::Right => outer.left(),
            };
            // The rail straddles the edge (`-right-4 -translate-x-1/2`):
            // 8px on each side, clipped to the visible area.
            let strip = Rect::from_min_max(
                Pos2::new(edge_x - 8.0, outer.top()),
                Pos2::new(edge_x + 8.0, outer.bottom()),
            )
            .intersect(ui.clip_rect());
            if strip.width() > 0.5 && strip.height() > 0.5 {
                let response = ui.interact(strip, base.with("offcanvas_rail"), Sense::click());
                if response.hovered() && ui.is_rect_visible(strip) {
                    painter.line_segment(
                        [
                            Pos2::new(edge_x, strip.top()),
                            Pos2::new(edge_x, strip.bottom()),
                        ],
                        Stroke::new(2.0, p.sidebar_border),
                    );
                }
                if response.clicked() {
                    request_toggle(ui, base);
                }
            }
        }

        // Paint the column now that the rect is final.
        if inner.width() > 0.5 && inner.height() > 0.5 {
            let radius = if variant == SidebarVariant::Floating {
                theme.radius.lg_u8()
            } else {
                0
            };
            if variant == SidebarVariant::Floating {
                let sh = crate::utils::shadow::shadow_sm(theme.mode.is_dark());
                let shadow_shape = egui::epaint::RectShape::filled(
                    inner.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                    CornerRadius::same(radius),
                    sh.color,
                )
                .with_blur_width(sh.blur as f32);
                painter.set(shadow_idx, shadow_shape);
            }
            painter.set(
                fill_idx,
                egui::epaint::RectShape::filled(inner, CornerRadius::same(radius), p.sidebar),
            );
            if variant == SidebarVariant::Floating {
                painter.rect_stroke(
                    inner,
                    radius,
                    Stroke::new(1.0, p.sidebar_border),
                    egui::epaint::StrokeKind::Inside,
                );
            }
        }
        // Outer-edge border for the plain variant (`border-r` / `border-l`).
        if variant == SidebarVariant::Sidebar && outer.width() > 0.5 {
            let x = match side {
                SidebarSide::Left => outer.right() - 0.5,
                SidebarSide::Right => outer.left() + 0.5,
            };
            painter.line_segment(
                [Pos2::new(x, outer.top()), Pos2::new(x, outer.bottom())],
                Stroke::new(1.0, p.sidebar_border),
            );
        }

        // Apply toggle intents (trigger / rail / keyboard).
        let toggle_requested = ui
            .ctx()
            .data_mut(|d| d.get_temp::<bool>(base.with("toggle_intent")))
            .unwrap_or(false);
        ui.ctx()
            .data_mut(|d| d.insert_temp(base.with("toggle_intent"), false));

        // Escape collapses when the keyboard focus sits inside the sidebar.
        let esc = ui.input(|i| i.key_pressed(egui::Key::Escape));
        let mut esc_inside = false;
        if esc {
            if let Some(focused) = ui.memory(|m| m.focused()) {
                if let Some(resp) = ui.ctx().read_response(focused) {
                    esc_inside = inner.intersects(resp.rect);
                }
            }
        }
        let effective_toggle =
            (toggle_requested || esc_inside) && collapsible != SidebarCollapsible::None;
        let final_open = if effective_toggle { !open } else { open };

        // Persist state (memory is the shared source for rails/triggers).
        ui.ctx()
            .data_mut(|d| d.insert_temp(open_id(base), final_open));
        let mut controlled_updated = false;
        if effective_toggle {
            if let Some(c) = controlled {
                *c = final_open;
                controlled_updated = true;
            }
        }

        SidebarResponse {
            open: final_open,
            collapsed_icon,
            response: ui.interact(outer, base.with("hitbox"), Sense::hover()),
            controlled_updated,
        }
    }
}

/// Response of [`Sidebar::show`].
pub struct SidebarResponse {
    /// The open state after this frame (toggles included).
    pub open: bool,
    /// True when collapsed to the icon rail.
    pub collapsed_icon: bool,
    /// Hover response over the sidebar column.
    pub response: Response,
    /// True when a toggle happened and a controlled `&mut bool` was synced.
    pub controlled_updated: bool,
}

fn header_footer<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> egui::InnerResponse<R> {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::SM; // gap-2
        ui.add_space(crate::theme::tokens::spacing::SM); // p-2
        let r = content(ui);
        ui.add_space(crate::theme::tokens::spacing::SM);
        r
    })
}

/// `SidebarHeader` — `flex flex-col gap-2 p-2`.
#[derive(Default)]
pub struct SidebarHeader {
    id_salt: Option<String>,
}

impl SidebarHeader {
    /// Creates the header slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        header_footer(ui, content)
    }
}

/// `SidebarFooter` — `flex flex-col gap-2 p-2`.
#[derive(Default)]
pub struct SidebarFooter {
    id_salt: Option<String>,
}

impl SidebarFooter {
    /// Creates the footer slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the footer.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        header_footer(ui, content)
    }
}

/// `SidebarContent` — `flex-1 flex-col overflow-auto` between header and
/// footer (`gap-0` in nova, `gap-2` in the base style). Content taller than
/// the remaining column height scrolls vertically.
#[derive(Default)]
pub struct SidebarContent {
    id_salt: Option<String>,
}

impl SidebarContent {
    /// Creates the content slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the scrollable content area.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let gap = match ShadcnTheme::of_ui(ui).style {
            Style::Nova => 0.0, // cn-sidebar-content: gap-0
            _ => crate::theme::tokens::spacing::SM,
        };
        // Fill the width, hug the content height and scroll when it exceeds
        // the remaining space (`overflow-auto` on a flex-1 column).
        let out = egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = gap;
                content(ui)
            });
        egui::InnerResponse::new(
            out.inner,
            ui.interact(out.inner_rect, out.id, egui::Sense::hover()),
        )
    }
}

/// `SidebarSeparator` — an `mx-2` rule in `bg-sidebar-border`.
#[derive(Default)]
pub struct SidebarSeparator {
    id_salt: Option<String>,
}

impl SidebarSeparator {
    /// Creates the separator.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the separator line.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let width = ui.available_width();
        let (_, rect) = ui.allocate_space(Vec2::new(width, 1.0));
        let x0 = rect.left() + 8.0; // mx-2
        let x1 = rect.right() - 8.0;
        if x1 > x0 {
            ui.painter().line_segment(
                [
                    Pos2::new(x0, rect.center().y),
                    Pos2::new(x1, rect.center().y),
                ],
                Stroke::new(1.0, p.sidebar_border),
            );
        }
        ui.interact(rect, ui.auto_id_with("sidebar_sep"), Sense::hover())
    }
}

/// `SidebarGroup` — `flex flex-col p-2`.
#[derive(Default)]
pub struct SidebarGroup {
    id_salt: Option<String>,
}

impl SidebarGroup {
    /// Creates a group.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.vertical(|ui| {
            ui.add_space(crate::theme::tokens::spacing::SM); // p-2
            ui.spacing_mut().item_spacing.y = 0.0;
            let r = content(ui);
            ui.add_space(crate::theme::tokens::spacing::SM);
            r
        })
    }
}

/// `SidebarGroupLabel` — `h-8 px-2 text-xs font-medium
/// text-sidebar-foreground/70`; hidden when collapsed to icons.
pub struct SidebarGroupLabel {
    text: String,
    id_salt: Option<String>,
}

impl SidebarGroupLabel {
    /// Creates a group label.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the label (nothing when collapsed to icons).
    pub fn show(self, ui: &mut Ui) -> Response {
        let collapsed = current_ctx(ui).is_some_and(|c| c.collapsed_icon);
        if collapsed {
            return ui.interact(
                Rect::NOTHING,
                ui.auto_id_with("sidebar_group_label_hidden"),
                Sense::hover(),
            );
        }
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let width = ui.available_width();
        let (_, rect) =
            ui.allocate_space(Vec2::new(width, crate::theme::tokens::spacing::CONTROL_SM));
        // Publish the label geometry for [`SidebarGroupAction`] (anchored to
        // the group's first label, `absolute top-3.5 right-3`).
        if let Some(info) = current_ctx(ui) {
            ui.ctx().data_mut(|d| {
                d.insert_temp(info.state_id.with("last_group_label"), rect);
            });
        }
        if ui.is_rect_visible(rect) {
            let font = FontId::new(
                crate::theme::tokens::type_scale::XS,
                FontFamily::Proportional,
            );
            let fg = color_alpha(p.sidebar_foreground, 0.7);
            let galley = ui.painter().layout_no_wrap(self.text, font, fg);
            ui.painter().galley(
                Pos2::new(rect.left() + 8.0, rect.center().y - galley.size().y * 0.5),
                galley,
                fg,
            );
        }
        ui.interact(rect, ui.auto_id_with("sidebar_group_label"), Sense::hover())
    }
}

/// `SidebarGroupContent` — `w-full text-sm` container.
#[derive(Default)]
pub struct SidebarGroupContent {
    id_salt: Option<String>,
}

impl SidebarGroupContent {
    /// Creates the group content slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group content.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            content(ui)
        })
    }
}

/// `SidebarMenu` — a vertical list of items (`gap-1`, `gap-0` in nova).
#[derive(Default)]
pub struct SidebarMenu {
    id_salt: Option<String>,
}

impl SidebarMenu {
    /// Creates a menu.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the menu list.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let gap = match ShadcnTheme::of_ui(ui).style {
            Style::Nova => 0.0, // cn-sidebar-menu: gap-0
            _ => crate::theme::tokens::spacing::XS,
        };
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = gap;
            content(ui)
        })
    }
}

/// `SidebarMenuItem` — one menu row container.
#[derive(Default)]
pub struct SidebarMenuItem {
    id_salt: Option<String>,
}

impl SidebarMenuItem {
    /// Creates a menu item container.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the item.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            content(ui)
        })
    }
}

/// The menu button label/galley helper: truncate to the available width.
fn truncate_galley(
    ui: &Ui,
    galley: std::sync::Arc<egui::Galley>,
    text: &str,
    font: FontId,
    max_w: f32,
    fg: egui::Color32,
) -> std::sync::Arc<egui::Galley> {
    if galley.size().x <= max_w || max_w <= 0.0 {
        return galley;
    }
    let mut t = text.to_owned();
    while t.chars().count() > 1 {
        t.pop();
        t.push('…');
        let g = ui.painter().layout_no_wrap(t.clone(), font.clone(), fg);
        if g.size().x <= max_w {
            return g;
        }
        t.pop(); // drop the ellipsis before removing the next char
    }
    galley
}

fn resolve_menu_button_colors(
    p: &Palette,
    variant: SidebarMenuButtonVariant,
    active: bool,
    hovered: bool,
    disabled: bool,
) -> (egui::Color32, egui::Color32) {
    let (bg, fg) = match variant {
        SidebarMenuButtonVariant::Outline => {
            if active || hovered {
                (p.sidebar_accent, p.sidebar_accent_foreground)
            } else {
                (p.background, p.sidebar_foreground)
            }
        }
        SidebarMenuButtonVariant::Default => {
            if active || hovered {
                (p.sidebar_accent, p.sidebar_accent_foreground)
            } else {
                (egui::Color32::TRANSPARENT, p.sidebar_foreground)
            }
        }
    };
    if disabled {
        (color_alpha(bg, 0.5), color_alpha(fg, 0.5))
    } else {
        (bg, fg)
    }
}

/// A small tooltip to the right of a collapsed menu button (delay 0, like
/// the source's `TooltipProvider delayDuration={0}`; the shared tooltip
/// skin — inverted `bg-primary text-primary-foreground`).
fn show_rail_tooltip(ui: &Ui, info: SidebarCtx, button_id: Id, text: &str) {
    let theme = ShadcnTheme::of_ui(ui);
    let p = theme.palette;
    let font = FontId::new(
        crate::theme::tokens::type_scale::SM,
        FontFamily::Proportional,
    );
    let galley = ui
        .painter()
        .layout_no_wrap(text.to_owned(), font, p.background);
    let galley_size = galley.size();
    let pos = Pos2::new(info.inner_rect.right() + 4.0, ui.cursor().top());
    egui::Area::new(button_id.with("rail_tooltip"))
        .order(egui::Order::Tooltip)
        .fixed_pos(pos)
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .corner_radius(theme.radius.md_u8())
                .fill(p.foreground) // bg-primary (inverted tooltip skin)
                .stroke(Stroke::NONE)
                .inner_margin(egui::Margin::symmetric(10, 5))
                .shadow(crate::utils::shadow::shadow_md(theme.mode.is_dark()))
                .show(ui, |ui| {
                    ui.set_min_width(galley_size.x);
                    ui.painter().galley(ui.cursor().min, galley, p.background);
                    ui.advance_cursor_after_rect(Rect::from_min_size(ui.cursor().min, galley_size));
                });
        });
}

/// `SidebarMenuButton` — `flex w-full items-center gap-2 rounded-md p-2
/// text-sm` with `default`/`outline` variants, `h-8`/`h-7`/`h-12` sizes,
/// active state, optional badge, and a hover tooltip when collapsed to the
/// icon rail.
pub struct SidebarMenuButton {
    label: String,
    icon: Option<Icon>,
    badge: Option<String>,
    tooltip: Option<String>,
    variant: SidebarMenuButtonVariant,
    size: SidebarMenuButtonSize,
    is_active: bool,
    disabled: bool,
    id_salt: Option<String>,
}

impl SidebarMenuButton {
    /// Creates a menu button with a label and optional leading icon.
    pub fn new(label: impl Into<String>, icon: impl Into<Option<Icon>>) -> Self {
        Self {
            label: label.into(),
            icon: icon.into(),
            badge: None,
            tooltip: None,
            variant: SidebarMenuButtonVariant::Default,
            size: SidebarMenuButtonSize::Default,
            is_active: false,
            disabled: false,
            id_salt: None,
        }
    }

    /// Sets the button variant.
    pub fn variant(mut self, variant: SidebarMenuButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets the button size.
    pub fn size(mut self, size: SidebarMenuButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Marks the button active (`data-active=true`).
    pub fn is_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Disables the button (`disabled:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// A right-aligned count badge (`SidebarMenuBadge`); hidden when
    /// collapsed to icons.
    pub fn badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// The tooltip shown when collapsed to the icon rail (`tooltip="..."`).
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the button.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let ctx_info = current_ctx(ui);
        let collapsed = ctx_info.is_some_and(|c| c.collapsed_icon);

        let width = ui.available_width();
        let height = match self.size {
            SidebarMenuButtonSize::Default => crate::theme::tokens::spacing::CONTROL_SM, // h-8
            SidebarMenuButtonSize::Sm => 28.0,                                           // h-7
            SidebarMenuButtonSize::Lg => 48.0,                                           // h-12
        };
        let (w, h) = if collapsed {
            (
                crate::theme::tokens::spacing::CONTROL_SM,
                crate::theme::tokens::spacing::CONTROL_SM,
            )
        } else {
            (width, height)
        };
        let (_, rect) = ui.allocate_space(Vec2::new(w.max(0.0), h));

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with(format!("sidebar_menu_button_{}", self.label)),
        };
        let response = ui.interact(rect, id, Sense::click());
        let hovered = response.hovered() && !self.disabled;

        // Publish the button geometry for the absolute-positioned
        // [`SidebarMenuAction`] (`absolute top-1.5 right-1` anchors to the
        // menu row).
        if let Some(info) = ctx_info {
            ui.ctx().data_mut(|d| {
                d.insert_temp(
                    info.state_id.with("last_menu_button"),
                    (rect, self.size, self.is_active),
                );
            });
        }

        if ui.is_rect_visible(rect) && rect.width() > 0.0 {
            let radius = theme.radius.md_u8();
            let (bg, fg) = resolve_menu_button_colors(
                &p,
                self.variant,
                self.is_active,
                hovered,
                self.disabled,
            );
            if bg != egui::Color32::TRANSPARENT {
                ui.painter().rect_filled(rect, radius, bg);
            }
            if self.variant == SidebarMenuButtonVariant::Outline {
                let ring = if hovered {
                    p.sidebar_accent
                } else {
                    p.sidebar_border
                };
                ui.painter().rect_stroke(
                    rect,
                    radius,
                    Stroke::new(1.0, ring),
                    egui::epaint::StrokeKind::Inside,
                );
            }

            // Icon + truncated label + optional badge (icon-only when
            // collapsed).
            let pad = 8.0; // p-2
            let mut cx = rect.left() + pad;
            let cy = rect.center().y;
            if let Some(icon) = self.icon {
                icon.paint(
                    ui.painter(),
                    Rect::from_center_size(Pos2::new(cx + 8.0, cy), Vec2::splat(16.0)),
                    fg,
                    2.0,
                );
                cx += 24.0; // icon (16) + gap-2 (8)
            }
            if !collapsed {
                let font = FontId::new(
                    match self.size {
                        SidebarMenuButtonSize::Sm => crate::theme::tokens::type_scale::XS,
                        _ => crate::theme::tokens::type_scale::SM,
                    },
                    FontFamily::Proportional,
                );
                let badge_w = match &self.badge {
                    Some(b) => {
                        let bfont = FontId::new(
                            crate::theme::tokens::type_scale::XS,
                            FontFamily::Proportional,
                        );
                        ui.painter()
                            .layout_no_wrap(b.clone(), bfont, fg)
                            .size()
                            .x
                            .max(20.0)
                            + 4.0
                    }
                    None => 0.0,
                };
                let avail = (rect.right() - pad - cx - badge_w).max(0.0);
                let galley = ui
                    .painter()
                    .layout_no_wrap(self.label.clone(), font.clone(), fg);
                let galley = truncate_galley(ui, galley, &self.label, font, avail, fg);
                ui.painter()
                    .galley(Pos2::new(cx, cy - galley.size().y * 0.5), galley, fg);

                if let Some(badge) = &self.badge {
                    let bfont = FontId::new(
                        crate::theme::tokens::type_scale::XS,
                        FontFamily::Proportional,
                    );
                    let galley = ui.painter().layout_no_wrap(badge.clone(), bfont, fg);
                    ui.painter().galley(
                        Pos2::new(
                            rect.right() - 4.0 - galley.size().x,
                            cy - galley.size().y * 0.5,
                        ),
                        galley,
                        fg,
                    );
                }
            }
        }

        // Collapsed rail tooltip (TooltipProvider delayDuration=0 in source).
        let has_tooltip = self
            .tooltip
            .as_deref()
            .is_some_and(|t| !t.trim().is_empty());
        if collapsed && has_tooltip && response.hovered() {
            if let (Some(info), Some(tooltip)) = (ctx_info, self.tooltip.as_deref()) {
                show_rail_tooltip(ui, info, id, tooltip);
            }
        }

        response
    }
}

/// `SidebarMenuSub` — `mx-3.5 border-l border-sidebar-border px-2.5 py-0.5
/// gap-1`; hidden when collapsed to icons. The border sits 14px in
/// (`mx-3.5`); the content starts 24px in (`mx-3.5` + `px-2.5`).
#[derive(Default)]
pub struct SidebarMenuSub {
    id_salt: Option<String>,
}

impl SidebarMenuSub {
    /// Creates a sub menu.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the sub menu (nothing when collapsed to icons).
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<egui::InnerResponse<R>> {
        let collapsed = current_ctx(ui).is_some_and(|c| c.collapsed_icon);
        if collapsed {
            // `group-data-[collapsible=icon]:hidden` — content not rendered.
            return None;
        }
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let line_x = ui.cursor().left() + 14.0 + 0.5; // mx-3.5 border-l
        let inner = ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = crate::theme::tokens::spacing::XS; // gap-1
            egui::Frame::new()
                .inner_margin(egui::Margin {
                    // mx-3.5 (14) + px-2.5 (10) content inset, py-0.5.
                    left: 24,
                    top: 2,
                    bottom: 2,
                    right: 0,
                })
                .show(ui, |ui| content(ui))
                .inner
        });
        // Border-l along the sub menu.
        let top = inner.response.rect.top();
        let bottom = inner.response.rect.bottom();
        ui.painter().line_segment(
            [Pos2::new(line_x, top), Pos2::new(line_x, bottom)],
            Stroke::new(1.0, p.sidebar_border),
        );
        Some(inner)
    }
}

/// `SidebarMenuSubButton` — `h-7 px-2 rounded-md text-sm` (or `text-xs`
/// for `sm`), `data-active` accent.
pub struct SidebarMenuSubButton {
    label: String,
    icon: Option<Icon>,
    size: SidebarMenuSubSize,
    is_active: bool,
    disabled: bool,
    id_salt: Option<String>,
}

impl SidebarMenuSubButton {
    /// Creates a sub button.
    pub fn new(label: impl Into<String>, icon: impl Into<Option<Icon>>) -> Self {
        Self {
            label: label.into(),
            icon: icon.into(),
            size: SidebarMenuSubSize::Md,
            is_active: false,
            disabled: false,
            id_salt: None,
        }
    }

    /// Sets the text size.
    pub fn size(mut self, size: SidebarMenuSubSize) -> Self {
        self.size = size;
        self
    }

    /// Marks the button active.
    pub fn is_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Disables the button.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the sub button.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let width = ui.available_width();
        let (_, rect) = ui.allocate_space(Vec2::new(width.max(0.0), 28.0)); // h-7

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with(format!("sidebar_sub_button_{}", self.label)),
        };
        let response = ui.interact(rect, id, Sense::click());
        let hovered = response.hovered() && !self.disabled;
        let (bg, fg) = if self.is_active || hovered {
            (p.sidebar_accent, p.sidebar_accent_foreground)
        } else {
            (egui::Color32::TRANSPARENT, p.sidebar_foreground)
        };
        let (bg, fg) = if self.disabled {
            (color_alpha(bg, 0.5), color_alpha(fg, 0.5))
        } else {
            (bg, fg)
        };

        if ui.is_rect_visible(rect) && rect.width() > 0.0 {
            if bg != egui::Color32::TRANSPARENT {
                ui.painter().rect_filled(rect, theme.radius.md_u8(), bg);
            }
            let pad = 8.0; // px-2
            let mut cx = rect.left() + pad;
            let cy = rect.center().y;
            if let Some(icon) = self.icon {
                icon.paint(
                    ui.painter(),
                    Rect::from_center_size(Pos2::new(cx + 8.0, cy), Vec2::splat(16.0)),
                    p.sidebar_accent_foreground,
                    2.0,
                );
                cx += 24.0;
            }
            let font = FontId::new(
                match self.size {
                    SidebarMenuSubSize::Sm => crate::theme::tokens::type_scale::XS,
                    SidebarMenuSubSize::Md => crate::theme::tokens::type_scale::SM,
                },
                FontFamily::Proportional,
            );
            let avail = (rect.right() - pad - cx).max(0.0);
            let galley = ui
                .painter()
                .layout_no_wrap(self.label.clone(), font.clone(), fg);
            let galley = truncate_galley(ui, galley, &self.label, font, avail, fg);
            ui.painter()
                .galley(Pos2::new(cx, cy - galley.size().y * 0.5), galley, fg);
        }
        response
    }
}

/// A rendered response placeholder for orphaned absolute-positioned slots.
fn orphan_response(ui: &Ui, tag: &str) -> Response {
    ui.interact(Rect::NOTHING, ui.auto_id_with(tag), Sense::hover())
}

/// `SidebarMenuAction` — the absolute icon button at the right edge of the
/// last [`SidebarMenuButton`] row (`top-1.5 right-1 aspect-square w-5`,
/// sidebar.tsx:548-578). Ghost style: hover `bg-sidebar-accent`; with
/// `show_on_hover` it only paints while the row is hovered
/// (`md:opacity-0` → 1). Hidden when collapsed to the icon rail.
pub struct SidebarMenuAction {
    icon: Icon,
    show_on_hover: bool,
    id_salt: Option<String>,
}

impl SidebarMenuAction {
    /// Creates a menu action with an icon.
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            show_on_hover: false,
            id_salt: None,
        }
    }

    /// Only paints while the enclosing menu row is hovered
    /// (`showOnHover`); the click area stays active.
    pub fn show_on_hover(mut self, on: bool) -> Self {
        self.show_on_hover = on;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the action (does not consume flow space; anchors to the most
    /// recent [`SidebarMenuButton`] in the enclosing sidebar).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let Some(info) = current_ctx(ui) else {
            return orphan_response(ui, "sidebar_menu_action_orphan");
        };
        if info.collapsed_icon {
            // `group-data-[collapsible=icon]:hidden`
            return orphan_response(ui, "sidebar_menu_action_hidden");
        }
        let Some((btn_rect, btn_size, btn_active)) = ui.ctx().data_mut(|d| {
            d.get_temp::<(Rect, SidebarMenuButtonSize, bool)>(
                info.state_id.with("last_menu_button"),
            )
        }) else {
            return orphan_response(ui, "sidebar_menu_action_no_button");
        };

        // `peer-data-[size=sm]:top-1 peer-data-[size=default]:top-1.5
        // peer-data-[size=lg]:top-2.5`, `right-1`, `w-5`.
        let top = match btn_size {
            SidebarMenuButtonSize::Sm => 4.0,
            SidebarMenuButtonSize::Default => 6.0,
            SidebarMenuButtonSize::Lg => 10.0,
        };
        let rect = Rect::from_min_size(
            Pos2::new(btn_rect.right() - 4.0 - 20.0, btn_rect.top() + top),
            Vec2::splat(20.0),
        );
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("sidebar_menu_action"),
        };
        let response = ui.interact(rect, id, Sense::click());
        let hovered = response.hovered();
        // `showOnHover`: paint only when the row is hovered (the click area
        // stays active, like the source's opacity transition).
        let row_hovered = ui.rect_contains_pointer(btn_rect);
        let paint = !self.show_on_hover || row_hovered;
        if paint && ui.is_rect_visible(rect) {
            let (fg, bg) = if hovered {
                (p.sidebar_accent_foreground, p.sidebar_accent)
            } else if self.show_on_hover && btn_active {
                (p.sidebar_accent_foreground, egui::Color32::TRANSPARENT)
            } else {
                (p.sidebar_foreground, egui::Color32::TRANSPARENT)
            };
            if bg != egui::Color32::TRANSPARENT {
                ui.painter().rect_filled(rect, theme.radius.md_u8(), bg);
            }
            self.icon.paint(
                ui.painter(),
                Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
                fg,
                2.0,
            );
        }
        response
    }
}

/// `SidebarGroupAction` — the absolute icon button at the right edge of the
/// last [`SidebarGroupLabel`] row (`absolute top-3.5 right-3 aspect-square
/// w-5`, sidebar.tsx:417-438). Hidden when collapsed to the icon rail.
pub struct SidebarGroupAction {
    icon: Icon,
    id_salt: Option<String>,
}

impl SidebarGroupAction {
    /// Creates a group action with an icon.
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            id_salt: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the action (does not consume flow space; anchors to the most
    /// recent [`SidebarGroupLabel`] in the enclosing sidebar).
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let Some(info) = current_ctx(ui) else {
            return orphan_response(ui, "sidebar_group_action_orphan");
        };
        if info.collapsed_icon {
            return orphan_response(ui, "sidebar_group_action_hidden");
        }
        let Some(label_rect) = ui
            .ctx()
            .data_mut(|d| d.get_temp::<Rect>(info.state_id.with("last_group_label")))
        else {
            return orphan_response(ui, "sidebar_group_action_no_label");
        };

        // `top-3.5 right-3` relative to the group: the label starts at the
        // group's p-2 inset, so the offsets shift by that padding.
        let rect = Rect::from_min_size(
            Pos2::new(label_rect.right() - 4.0 - 20.0, label_rect.top() + 6.0),
            Vec2::splat(20.0),
        );
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("sidebar_group_action"),
        };
        let response = ui.interact(rect, id, Sense::click());
        if ui.is_rect_visible(rect) {
            let hovered = response.hovered();
            let (fg, bg) = if hovered {
                (p.sidebar_accent_foreground, p.sidebar_accent)
            } else {
                (p.sidebar_foreground, egui::Color32::TRANSPARENT)
            };
            if bg != egui::Color32::TRANSPARENT {
                ui.painter().rect_filled(rect, theme.radius.md_u8(), bg);
            }
            self.icon.paint(
                ui.painter(),
                Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
                fg,
                2.0,
            );
        }
        response
    }
}

/// `SidebarMenuSkeleton` — `h-8 px-2 gap-2` loading row with an optional
/// `size-4` icon skeleton and an `h-4` text skeleton whose width is a
/// stable per-id fraction between 50% and 90% (sidebar.tsx:602-638).
pub struct SidebarMenuSkeleton {
    show_icon: bool,
    id_salt: Option<String>,
}

impl Default for SidebarMenuSkeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl SidebarMenuSkeleton {
    /// Creates a menu skeleton row.
    pub fn new() -> Self {
        Self {
            show_icon: false,
            id_salt: None,
        }
    }

    /// Prepends a 16×16 icon skeleton (`showIcon`).
    pub fn show_icon(mut self, on: bool) -> Self {
        self.show_icon = on;
        self
    }

    /// Sets a stable id salt (also picks the text-skeleton width).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the skeleton row.
    pub fn show(self, ui: &mut Ui) -> Response {
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("sidebar_menu_skeleton"),
        };
        // "Random width between 50 to 90%", stable per id.
        let frac = 0.5 + ((id.value() >> 13) % 100) as f32 / 100.0 * 0.4;
        let px = 8.0; // px-2
        ui.allocate_ui(Vec2::new(ui.available_width(), 32.0), |ui| {
            // h-8 row, items vertically centered (flex items-center gap-2).
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 8.0; // gap-2
                ui.add_space(px);
                if self.show_icon {
                    crate::components::skeleton::Skeleton::new()
                        .size(16.0, 16.0)
                        .show(ui);
                }
                let avail = (ui.available_width() - px).max(0.0);
                crate::components::skeleton::Skeleton::new()
                    .size(avail * frac, 16.0) // h-4 text skeleton
                    .show(ui);
            });
        })
        .response
    }
}

/// `SidebarTrigger` — the `PanelLeft` ghost icon button (`size-7`).
#[derive(Default)]
pub struct SidebarTrigger {
    /// The sidebar salt to target (empty = the enclosing sidebar).
    target_salt: Option<String>,
    id_salt: Option<String>,
}

impl SidebarTrigger {
    /// Creates a trigger bound to the enclosing sidebar.
    pub fn new() -> Self {
        Self {
            target_salt: None,
            id_salt: None,
        }
    }

    /// Creates a trigger targeting a sidebar by its id salt (works outside
    /// the sidebar's closure for uncontrolled sidebars).
    pub fn with_salt(salt: impl Into<String>) -> Self {
        Self {
            target_salt: Some(salt.into()),
            id_salt: None,
        }
    }

    /// Sets a stable id salt for the button itself.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the trigger button.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        // Target: enclosing sidebar or an explicit salt.
        let target = match &self.target_salt {
            Some(salt) => Some((state_id(salt), None)),
            None => current_ctx(ui).map(|c| (c.state_id, Some(c.collapsible))),
        };
        let Some((base, collapsible)) = target else {
            return ui.interact(
                Rect::NOTHING,
                ui.auto_id_with("sidebar_trigger_orphan"),
                Sense::hover(),
            );
        };
        let collapsible = collapsible.unwrap_or(SidebarCollapsible::Icon);

        let (_, rect) = ui.allocate_space(Vec2::splat(28.0)); // size-7
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("sidebar_trigger"),
        };
        let response = ui.interact(rect, id, Sense::click());
        if ui.is_rect_visible(rect) {
            let hovered = response.hovered();
            let dark = theme.mode.is_dark();
            if hovered {
                // The source trigger is a plain ghost Button
                // (`hover:bg-accent`; crate base recipe: accent light /
                // accent/50 dark).
                ui.painter().rect_filled(
                    rect,
                    theme.radius.md_u8(), // rounded-md
                    if dark {
                        color_alpha(p.accent, 0.5)
                    } else {
                        p.accent
                    },
                );
            }
            Icon::PanelLeft.paint(
                ui.painter(),
                Rect::from_center_size(rect.center(), Vec2::splat(16.0)),
                if hovered {
                    p.accent_foreground
                } else {
                    p.sidebar_foreground
                },
                2.0,
            );
        }
        if response.clicked() && collapsible != SidebarCollapsible::None {
            if self.target_salt.is_some() {
                // Salt-based triggers flip the stored state directly (only
                // reliable for uncontrolled sidebars).
                let current = read_open(ui.ctx(), base, true);
                ui.ctx()
                    .data_mut(|d| d.insert_temp(open_id(base), !current));
            } else {
                request_toggle(ui, base);
            }
        }
        response
    }
}

/// `SidebarRail` — a thin clickable strip on the sidebar's outer edge that
/// toggles collapse (hover shows a 2px line in `sidebar-border`).
#[derive(Default)]
pub struct SidebarRail {
    id_salt: Option<String>,
}

impl SidebarRail {
    /// Creates a rail bound to the enclosing sidebar.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the rail strip.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let Some(info) = current_ctx(ui) else {
            return ui.interact(
                Rect::NOTHING,
                ui.auto_id_with("sidebar_rail_orphan"),
                Sense::hover(),
            );
        };
        if info.collapsible == SidebarCollapsible::None || info.inner_rect.width() < 1.0 {
            return ui.interact(
                Rect::NOTHING,
                ui.auto_id_with("sidebar_rail_disabled"),
                Sense::hover(),
            );
        }

        let edge_x = info.inner_rect.right();
        // The rail straddles the sidebar edge (`-right-4 -translate-x-1/2`):
        // 8px inside + 8px outside, clipped to the visible area.
        let rect = Rect::from_min_max(
            Pos2::new(edge_x - 8.0, info.inner_rect.top()),
            Pos2::new(edge_x + 8.0, info.inner_rect.bottom()),
        )
        .intersect(ui.clip_rect());
        if rect.width() <= 0.5 || rect.height() <= 0.5 {
            return orphan_response(ui, "sidebar_rail_clipped");
        }
        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("sidebar_rail"),
        };
        let response = ui.interact(rect, id, Sense::click());
        if response.hovered() && ui.is_rect_visible(rect) {
            // The 2px hover line sits at the very edge (`after:left-full`).
            ui.painter().line_segment(
                [
                    Pos2::new(edge_x, rect.top()),
                    Pos2::new(edge_x, rect.bottom()),
                ],
                Stroke::new(2.0, p.sidebar_border),
            );
        }
        if response.clicked() {
            request_toggle(ui, info.state_id);
        }
        response
    }
}

/// `SidebarInset` — the main content wrapper next to the sidebar: plain
/// transparent by default; for the `inset` variant a rounded
/// `bg-background` card with margins (`m-2 ml-0`, `ml-2` when collapsed).
#[derive(Default)]
pub struct SidebarInset {
    id_salt: Option<String>,
}

impl SidebarInset {
    /// Creates the main content wrapper.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the main area.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let info = current_ctx(ui);
        let inset = info.is_some_and(|c| c.variant == SidebarVariant::Inset);

        if !inset {
            return ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                content(ui)
            });
        }

        // m-2 ml-0; ml-2 when the sidebar is collapsed.
        let m = 8.0; // m-2
        let left = if info.is_some_and(|c| !c.open) {
            m
        } else {
            0.0
        };
        let avail = ui.available_rect_before_wrap();
        let card_rect = Rect::from_min_max(
            Pos2::new(avail.left() + left, avail.top() + m),
            Pos2::new(avail.right() - m, avail.bottom() - m),
        );

        let painter = ui.painter().clone();
        // `md:peer-data-[variant=inset]:shadow-sm` behind the rounded card
        // (added first so the card fill paints above it).
        let sh = crate::utils::shadow::shadow_sm(theme.mode.is_dark());
        painter.add(
            egui::epaint::RectShape::filled(
                card_rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                theme.radius.xl_u8(),
                sh.color,
            )
            .with_blur_width(sh.blur as f32),
        );
        let bg_idx = painter.add(egui::Shape::Noop);
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(card_rect)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        let out = content(&mut child);
        painter.set(
            bg_idx,
            egui::Shape::Rect(egui::epaint::RectShape::filled(
                card_rect,
                theme.radius.xl_u8(), // rounded-xl
                p.background,
            )),
        );
        egui::InnerResponse::new(out, child.response())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn harness() -> egui::Context {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        ctx
    }

    fn run<R>(ctx: &egui::Context, mut f: impl FnMut(&mut Ui) -> R) {
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut Ui::new(
                ctx.clone(),
                Id::unique("sidebar_test_ui"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(900.0, 600.0))),
            );
            f(ui);
        });
        out.textures_delta.clear();
    }

    fn minimal_sidebar(ui: &mut Ui, salt: &str, collapsible: SidebarCollapsible) -> bool {
        Sidebar::new()
            .id_salt(salt)
            .collapsible(collapsible)
            .show(ui, |ui| {
                SidebarContent::new().show(ui, |ui| {
                    SidebarMenu::new().show(ui, |ui| {
                        SidebarMenuItem::new().show(ui, |ui| {
                            SidebarMenuButton::new("History", Icon::Message)
                                .tooltip("History")
                                .show(ui);
                        });
                    });
                });
            })
            .open
    }

    #[test]
    fn widths_follow_source_constants() {
        assert_eq!(SIDEBAR_WIDTH, 256.0); // 16rem
        assert_eq!(SIDEBAR_WIDTH_ICON, 48.0); // 3rem
    }

    #[test]
    fn open_state_persists_per_id_salt() {
        let ctx = harness();
        run(&ctx, |ui| {
            assert!(minimal_sidebar(ui, "persist", SidebarCollapsible::Icon));
        });
        // The toggle intent flips it on the next frame.
        run(&ctx, |ui| {
            request_toggle(ui, state_id("persist"));
        });
        run(&ctx, |ui| {
            assert!(!minimal_sidebar(ui, "persist", SidebarCollapsible::Icon));
        });
        // A second sidebar keeps its own state.
        run(&ctx, |ui| {
            assert!(minimal_sidebar(ui, "other", SidebarCollapsible::Icon));
        });
    }

    #[test]
    fn collapsible_none_ignores_toggle() {
        let ctx = harness();
        run(&ctx, |ui| {
            request_toggle(ui, state_id("fixed"));
        });
        run(&ctx, |ui| {
            assert!(minimal_sidebar(ui, "fixed", SidebarCollapsible::None));
        });
    }

    #[test]
    fn controlled_sidebar_round_trips() {
        let ctx = harness();
        let mut open = true;
        run(&ctx, |ui| {
            let r = Sidebar::controlled(&mut open)
                .id_salt("ctrl")
                .show(ui, |_ui| {});
            assert!(r.open);
        });
        assert!(open);
        open = false;
        run(&ctx, |ui| {
            let r = Sidebar::controlled(&mut open)
                .id_salt("ctrl2")
                .show(ui, |_ui| {});
            assert!(!r.open);
        });
        assert!(!open);
    }

    #[test]
    fn controlled_sidebar_toggles_back() {
        let ctx = harness();
        let mut open = true;
        run(&ctx, |ui| {
            request_toggle(ui, state_id("ctrl3"));
        });
        run(&ctx, |ui| {
            let r = Sidebar::controlled(&mut open)
                .id_salt("ctrl3")
                .show(ui, |_ui| {});
            assert!(r.controlled_updated);
            assert!(!r.open);
        });
        assert!(!open, "the controlled bool must be synced");
    }

    #[test]
    fn builders_keep_fields() {
        let s = Sidebar::new()
            .side(SidebarSide::Right)
            .variant(SidebarVariant::Floating)
            .collapsible(SidebarCollapsible::Icon)
            .width(280.0)
            .icon_width(56.0)
            .id_salt("x");
        assert_eq!(s.side, SidebarSide::Right);
        assert_eq!(s.variant, SidebarVariant::Floating);
        assert_eq!(s.collapsible, SidebarCollapsible::Icon);
        assert_eq!(s.width, 280.0);
        assert_eq!(s.icon_width, 56.0);
        assert_eq!(s.id_salt.as_deref(), Some("x"));

        let b = SidebarMenuButton::new("A", Icon::Check)
            .variant(SidebarMenuButtonVariant::Outline)
            .size(SidebarMenuButtonSize::Lg)
            .is_active(true)
            .disabled(true)
            .badge("12")
            .tooltip("A");
        assert_eq!(b.variant, SidebarMenuButtonVariant::Outline);
        assert_eq!(b.size, SidebarMenuButtonSize::Lg);
        assert!(b.is_active);
        assert!(b.disabled);
        assert_eq!(b.badge.as_deref(), Some("12"));
        assert_eq!(b.tooltip.as_deref(), Some("A"));
    }

    #[test]
    fn menu_button_colors_match_source_states() {
        let p = Preset::default().build(Mode::Light).palette;
        // Default variant, idle: transparent bg.
        let (bg, _) =
            resolve_menu_button_colors(&p, SidebarMenuButtonVariant::Default, false, false, false);
        assert_eq!(bg, egui::Color32::TRANSPARENT);
        // Active and hover share the accent fill.
        let (a, _) =
            resolve_menu_button_colors(&p, SidebarMenuButtonVariant::Default, true, false, false);
        let (h, _) =
            resolve_menu_button_colors(&p, SidebarMenuButtonVariant::Default, false, true, false);
        assert_eq!(a, p.sidebar_accent);
        assert_eq!(h, p.sidebar_accent);
        // Disabled dims to 50%.
        let (d, df) =
            resolve_menu_button_colors(&p, SidebarMenuButtonVariant::Default, true, false, true);
        assert_eq!(d, color_alpha(p.sidebar_accent, 0.5));
        assert_eq!(df, color_alpha(p.sidebar_accent_foreground, 0.5));
        // Outline idle sits on background.
        let (o, of) =
            resolve_menu_button_colors(&p, SidebarMenuButtonVariant::Outline, false, false, false);
        assert_eq!(o, p.background);
        assert_eq!(of, p.sidebar_foreground);
    }

    #[test]
    fn renders_all_slots_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = harness();
            let theme = Preset::default().build(mode);
            theme.install(&ctx);
            run(&ctx, |ui| {
                ui.horizontal(|ui| {
                    Sidebar::new()
                        .id_salt("render")
                        .variant(SidebarVariant::Floating)
                        .collapsible(SidebarCollapsible::Icon)
                        .show(ui, |ui| {
                            SidebarHeader::new().show(ui, |ui| {
                                SidebarTrigger::new().show(ui);
                            });
                            SidebarContent::new().show(ui, |ui| {
                                SidebarGroup::new().show(ui, |ui| {
                                    SidebarGroupLabel::new("Platform").show(ui);
                                    SidebarGroupContent::new().show(ui, |ui| {
                                        SidebarMenu::new().show(ui, |ui| {
                                            SidebarMenuItem::new().show(ui, |ui| {
                                                SidebarMenuButton::new("History", Icon::Message)
                                                    .badge("3")
                                                    .tooltip("History")
                                                    .show(ui);
                                            });
                                            SidebarMenuItem::new().show(ui, |ui| {
                                                SidebarMenuButton::new("Starred", Icon::Sparkles)
                                                    .variant(SidebarMenuButtonVariant::Outline)
                                                    .size(SidebarMenuButtonSize::Sm)
                                                    .show(ui);
                                            });
                                            SidebarMenuSub::new().show(ui, |ui| {
                                                SidebarMenuSubButton::new("Drafts", None::<Icon>)
                                                    .is_active(true)
                                                    .show(ui);
                                                SidebarMenuSubButton::new("Sent", None::<Icon>)
                                                    .size(SidebarMenuSubSize::Sm)
                                                    .show(ui);
                                            });
                                        });
                                    });
                                });
                                SidebarSeparator::new().show(ui);
                            });
                            SidebarFooter::new().show(ui, |ui| {
                                SidebarMenuButton::new("Account", Icon::User).show(ui);
                            });
                            SidebarRail::new().show(ui);
                        });
                    SidebarInset::new().show(ui, |ui| {
                        ui.label("Main content");
                    });
                });
            });
        }
    }

    #[test]
    fn collapsed_icon_hides_labels_and_subs() {
        let ctx = harness();
        // Collapse via intent, then verify the open state flips and the
        // hidden slots render nothing.
        run(&ctx, |ui| {
            request_toggle(ui, state_id("collapse"));
        });
        run(&ctx, |ui| {
            let r = Sidebar::new()
                .id_salt("collapse")
                .collapsible(SidebarCollapsible::Icon)
                .show(ui, |ui| {
                    SidebarGroupLabel::new("Platform").show(ui);
                    SidebarMenuSub::new().show(ui, |ui| {
                        SidebarMenuSubButton::new("Drafts", None::<Icon>).show(ui);
                    });
                });
            assert!(!r.open, "the toggle applies on this frame");
        });
        // Collapsed on the following frame: labels and subs render nothing.
        run(&ctx, |ui| {
            let r = Sidebar::new()
                .id_salt("collapse")
                .collapsible(SidebarCollapsible::Icon)
                .show(ui, |ui| {
                    SidebarGroupLabel::new("Platform").show(ui);
                    let sub = SidebarMenuSub::new().show(ui, |ui| {
                        SidebarMenuSubButton::new("Drafts", None::<Icon>).show(ui);
                    });
                    assert!(sub.is_none(), "menu subs are hidden when collapsed");
                });
            assert!(!r.open);
            assert!(r.collapsed_icon);
        });
    }

    #[test]
    fn offcanvas_collapse_hides_the_column() {
        let ctx = harness();
        run(&ctx, |ui| {
            request_toggle(ui, state_id("off"));
        });
        run(&ctx, |ui| {
            let r = Sidebar::new().id_salt("off").show(ui, |_ui| {});
            assert!(!r.open);
        });
    }

    #[test]
    fn escape_inside_sidebar_collapses() {
        let ctx = harness();
        run(&ctx, |ui| {
            minimal_sidebar(ui, "esc", SidebarCollapsible::Icon);
        });
        // Focus a widget inside the sidebar, then send Escape.
        run(&ctx, |ui| {
            Sidebar::new()
                .id_salt("esc")
                .collapsible(SidebarCollapsible::Icon)
                .show(ui, |ui| {
                    let b = SidebarMenuButton::new("History", Icon::Message).show(ui);
                    b.request_focus();
                });
        });
        // The Escape key press is delivered through run_ui's input.
        let mut out = ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Escape,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: Default::default(),
                }],
                ..Default::default()
            },
            |ctx| {
                let ui = &mut Ui::new(
                    ctx.clone(),
                    Id::unique("sidebar_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(900.0, 600.0))),
                );
                let r = Sidebar::new()
                    .id_salt("esc")
                    .collapsible(SidebarCollapsible::Icon)
                    .show(ui, |ui| {
                        let b = SidebarMenuButton::new("History", Icon::Message).show(ui);
                        b.request_focus();
                    });
                assert!(!r.open, "Escape with focus inside must collapse");
            },
        );
        out.textures_delta.clear();
        run(&ctx, |ui| {
            assert!(!minimal_sidebar(ui, "esc", SidebarCollapsible::Icon));
        });
    }

    #[test]
    fn menu_sub_content_is_inset_24px() {
        let ctx = harness();
        let mut inner_left = None;
        let mut sub_left = None;
        run(&ctx, |ui| {
            Sidebar::new().id_salt("subinset").show(ui, |ui| {
                inner_left = Some(ui.cursor().left());
                SidebarContent::new().show(ui, |ui| {
                    SidebarMenu::new().show(ui, |ui| {
                        SidebarMenuItem::new().show(ui, |ui| {
                            SidebarMenuButton::new("History", Icon::Message).show(ui);
                            SidebarMenuSub::new().show(ui, |ui| {
                                let b = SidebarMenuSubButton::new("Clone", Icon::Copy).show(ui);
                                sub_left = Some(b.rect.left());
                            });
                        });
                    });
                });
            });
        });
        let inner = inner_left.expect("column left");
        let sub = sub_left.expect("sub button left");
        // `mx-3.5` (14) + `px-2.5` (10): content starts 24px from the edge,
        // while the border line stays at 14.5px.
        assert!(
            (sub - inner - 24.0).abs() < 0.5,
            "sub content must start 24px in (got {} at inner {})",
            sub,
            inner
        );
    }

    /// Press + release a click at `pos` across two passes (with advancing
    /// time so the width animation completes).
    fn click(ctx: &egui::Context, pos: Pos2, salt: &str, start_time: f64) {
        for (i, pressed) in [true, false].iter().enumerate() {
            let mut input = egui::RawInput {
                time: Some(start_time + i as f64 * 0.1),
                ..Default::default()
            };
            input.events.push(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed: *pressed,
                modifiers: Default::default(),
            });
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut Ui::new(
                    ctx.clone(),
                    Id::unique("sidebar_click_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(900.0, 600.0))),
                );
                Sidebar::new().id_salt(salt).show(ui, |_ui| {});
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn rail_strip_reopens_collapsed_offcanvas_sidebar() {
        let ctx = harness();
        // Advance time so the collapse width animation (200ms) completes
        // before the click.
        let mut t = 0.0;
        run(&ctx, |ui| {
            request_toggle(ui, state_id("offrail"));
        });
        for step in 1..=4 {
            let input = egui::RawInput {
                time: Some(step as f64 * 0.2),
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut Ui::new(
                    ctx.clone(),
                    Id::unique("sidebar_test_ui"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(900.0, 600.0))),
                );
                let r = Sidebar::new().id_salt("offrail").show(ui, |_ui| {});
                assert!(!r.open);
            });
            out.textures_delta.clear();
        }
        t += 1.0;
        // The built-in strip straddles the collapsed edge (8px visible).
        click(&ctx, Pos2::new(4.0, 300.0), "offrail", t);
        let input = egui::RawInput {
            time: Some(t + 0.5),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ctx| {
            let ui = &mut Ui::new(
                ctx.clone(),
                Id::unique("sidebar_test_ui"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(900.0, 600.0))),
            );
            let r = Sidebar::new().id_salt("offrail").show(ui, |_ui| {});
            assert!(r.open, "the offcanvas rail strip must re-open the sidebar");
        });
        out.textures_delta.clear();
    }

    #[test]
    fn menu_action_anchors_to_menu_button() {
        let ctx = harness();
        run(&ctx, |ui| {
            Sidebar::new().id_salt("mact").show(ui, |ui| {
                SidebarContent::new().show(ui, |ui| {
                    SidebarMenu::new().show(ui, |ui| {
                        SidebarMenuItem::new().show(ui, |ui| {
                            let btn = SidebarMenuButton::new("History", Icon::Message).show(ui);
                            let act = SidebarMenuAction::new(Icon::MoreVertical).show(ui);
                            assert!((act.rect.width() - 20.0).abs() < 0.5, "w-5");
                            assert!((act.rect.left() - (btn.rect.right() - 24.0)).abs() < 0.5);
                            assert!((act.rect.top() - (btn.rect.top() + 6.0)).abs() < 0.5);
                        });
                    });
                });
            });
        });
    }

    #[test]
    fn group_action_anchors_to_label() {
        let ctx = harness();
        run(&ctx, |ui| {
            Sidebar::new().id_salt("gact").show(ui, |ui| {
                SidebarContent::new().show(ui, |ui| {
                    SidebarGroup::new().show(ui, |ui| {
                        let label = SidebarGroupLabel::new("Platform").show(ui);
                        let act = SidebarGroupAction::new(Icon::Plus).show(ui);
                        assert!((act.rect.width() - 20.0).abs() < 0.5, "w-5");
                        // `top-3.5 right-3` shifted by the group's p-2.
                        assert!((act.rect.left() - (label.rect.right() - 24.0)).abs() < 0.5);
                        assert!((act.rect.top() - (label.rect.top() + 6.0)).abs() < 0.5);
                    });
                });
            });
        });
    }

    #[test]
    fn menu_skeleton_renders_with_stable_width() {
        let ctx = harness();
        run(&ctx, |ui| {
            Sidebar::new().id_salt("skel").show(ui, |ui| {
                SidebarContent::new().show(ui, |ui| {
                    SidebarMenu::new().show(ui, |ui| {
                        let a = SidebarMenuSkeleton::new().show_icon(true).show(ui);
                        assert!((a.rect.height() - 32.0).abs() < 0.5, "h-8 row");
                        let b = SidebarMenuSkeleton::new().show_icon(true).show(ui);
                        // Each row keeps its own (stable) text width.
                        assert!(b.rect.height() > 0.0);
                    });
                });
            });
        });
    }
}
