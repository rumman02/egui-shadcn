//! shadcn `alert.tsx` — a callout for user attention.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/alert.tsx`:
//! `rounded-lg border px-4 py-3 text-sm` with `default` (`bg-card
//! text-card-foreground`) and `destructive` (`text-destructive`,
//! description `text-destructive/90`) variants; the nova style tightens the
//! padding to `px-2.5 py-2` and narrows the icon gap.
//!
//! The optional icon slot sits left of the title/description column; the
//! optional [`AlertAction`] paints at the alert's top-right corner (nova
//! `absolute top-2 right-2`) and the alert reserves `pr-18` for it.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Alert::new()
//!     .icon(Icon::CircleCheck)
//!     .show(ui, |ui| {
//!         AlertTitle::new("Heads up!").show(ui);
//!         AlertDescription::new("You can add components to your app.").show(ui);
//!     });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{Color32, CornerRadius, FontId, InnerResponse, Margin, Pos2, Rect, Stroke, Ui};

/// Visual variants, mirroring the source `alertVariants.variant`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    /// `bg-card text-card-foreground`.
    #[default]
    Default,
    /// `text-destructive` with `text-destructive/90` description.
    Destructive,
}

/// Horizontal padding (`px-4` base recipe, nova `px-2.5`).
pub(crate) fn padding_x(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0,
        _ => 16.0,
    }
}

/// Vertical padding (`py-3` base recipe, nova `py-2`).
pub(crate) fn padding_y(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0,
        _ => 12.0,
    }
}

/// Gap between the icon column and the text column
/// (`gap-x-3` base recipe, nova `gap-x-2`).
pub(crate) fn icon_gap(style: Style) -> f32 {
    match style {
        Style::Nova => 8.0,
        _ => 12.0,
    }
}

/// Resolved colors for one alert state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct AlertColors {
    pub fill: Color32,
    pub stroke: Stroke,
    pub title: Color32,
    pub description: Color32,
    pub icon: Color32,
}

pub(crate) fn resolve_alert_colors(p: &Palette, variant: AlertVariant) -> AlertColors {
    match variant {
        AlertVariant::Default => AlertColors {
            fill: p.card,
            stroke: Stroke::new(1.0, p.border),
            title: p.card_foreground,
            description: p.muted_foreground,
            icon: p.card_foreground,
        },
        AlertVariant::Destructive => AlertColors {
            fill: p.card,
            stroke: Stroke::new(1.0, p.border),
            title: p.destructive,
            description: color_alpha(p.destructive, 0.9),
            icon: p.destructive,
        },
    }
}

/// Geometry of the alert currently being laid out, for [`AlertAction`].
#[derive(Clone, Copy, Debug)]
struct AlertCtx {
    id: egui::Id,
    /// Top-right anchor for the action (`top-2 right-2`).
    top: f32,
    right: f32,
    left: f32,
}

fn alert_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_alert_stack")
}

fn push_alert(ui: &Ui, ctx: AlertCtx) {
    let mut stack: Vec<AlertCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(alert_stack_id()))
        .unwrap_or_default();
    stack.push(ctx);
    ui.ctx()
        .data_mut(|d| d.insert_temp(alert_stack_id(), stack));
}

fn pop_alert(ui: &Ui) {
    let mut stack: Vec<AlertCtx> = ui
        .ctx()
        .data_mut(|d| d.get_temp(alert_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(alert_stack_id(), stack));
}

fn current_alert(ui: &Ui) -> Option<AlertCtx> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<AlertCtx>>(alert_stack_id()))
        .and_then(|s| s.last().copied())
}

/// A shadcn alert. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct Alert {
    variant: AlertVariant,
    icon: Option<Icon>,
    id_salt: Option<String>,
}

impl Default for Alert {
    fn default() -> Self {
        Self::new()
    }
}

impl Alert {
    /// Creates a default alert.
    pub fn new() -> Self {
        Self {
            variant: AlertVariant::default(),
            icon: None,
            id_salt: None,
        }
    }

    /// Sets the visual variant (default: [`AlertVariant::Default`]).
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows an icon in the left column (`has-[>svg]` grid column).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Sets a stable id salt; required for reliable [`AlertAction`]
    /// width reservation when multiple alerts render side by side.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the alert around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_alert_colors(&p, self.variant);
        let radius = theme.radius.lg_u8();

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_alert"),
        };
        // `has-data-[slot=alert-action]:pr-18` — reserve 72px on the right
        // for the action; read from the previous frame (self-correcting).
        let has_action: bool = ui
            .ctx()
            .data_mut(|d| d.get_temp(id.with("has_action")).unwrap_or(false));
        let reserved_right = if has_action { 72.0 } else { 0.0 };

        let cursor = ui.cursor();
        push_variant(ui, self.variant);
        push_alert(
            ui,
            AlertCtx {
                id,
                top: cursor.top() + 8.0,
                right: cursor.right() - 8.0,
                left: cursor.left(),
            },
        );

        let px = padding_x(theme.style);
        let py = padding_y(theme.style);
        let gap = icon_gap(theme.style);
        let frame = egui::Frame::new()
            .fill(c.fill)
            .stroke(c.stroke)
            .corner_radius(CornerRadius::same(radius))
            .inner_margin(Margin {
                left: px as i8,
                right: (px + reserved_right) as i8,
                top: py as i8,
                bottom: py as i8,
            });

        let inner = frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width()); // `w-full`
            ui.style_mut().visuals.override_text_color = Some(c.title);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = gap;
                if let Some(icon) = self.icon {
                    // `[&>svg]:size-4 [&>svg]:translate-y-0.5` — 16px icon,
                    // nudged 2px down, spanning both text rows.
                    let (rect, _) =
                        ui.allocate_exact_size(egui::Vec2::splat(16.0), egui::Sense::hover());
                    if ui.is_rect_visible(rect) {
                        let icon_rect = Rect::from_center_size(
                            rect.center() + Pos2::new(0.0, 2.0).to_vec2(),
                            egui::Vec2::splat(16.0),
                        );
                        icon.paint(ui.painter(), icon_rect, c.icon, 2.0 * 16.0 / 24.0);
                    }
                }
                ui.vertical(|ui| {
                    // `gap-y-0.5` / nova `gap-0.5`.
                    ui.spacing_mut().item_spacing.y = 2.0;
                    content(ui)
                })
                .inner
            })
            .inner
        });

        pop_alert(ui);
        pop_variant(ui);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `AlertTitle` — `min-h-4 font-medium tracking-tight`, single line.
#[derive(Clone, Debug)]
pub struct AlertTitle {
    text: String,
}

impl AlertTitle {
    /// Creates an alert title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_alert_colors(&p, current_variant(ui).unwrap_or_default());
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui.painter().layout_no_wrap(self.text, font, c.title);
        let (rect, response) = ui.allocate_exact_size(
            galley.size().max(egui::Vec2::new(0.0, 16.0)),
            egui::Sense::hover(),
        );
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, c.title);
        }
        response
    }
}

/// `AlertDescription` — `text-sm text-muted-foreground` (destructive:
/// `text-destructive/90`), wraps to the width.
#[derive(Clone, Debug)]
pub struct AlertDescription {
    text: String,
}

impl AlertDescription {
    /// Creates an alert description.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let c = resolve_alert_colors(&p, current_variant(ui).unwrap_or_default());
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let galley = ui
            .painter()
            .layout(self.text, font, c.description, ui.available_width());
        let (rect, response) = ui.allocate_exact_size(galley.size(), egui::Sense::hover());
        if ui.is_rect_visible(rect) {
            ui.painter().galley(rect.min, galley, c.description);
        }
        response
    }
}

/// The variant of the enclosing [`Alert`], from the ui temp stack.
fn variant_stack_id() -> egui::Id {
    egui::Id::unique("shadcn_alert_variant_stack")
}

fn current_variant(ui: &Ui) -> Option<AlertVariant> {
    ui.ctx()
        .data_mut(|d| d.get_temp::<Vec<AlertVariant>>(variant_stack_id()))
        .and_then(|s| s.last().copied())
}

fn push_variant(ui: &Ui, v: AlertVariant) {
    let mut stack: Vec<AlertVariant> = ui
        .ctx()
        .data_mut(|d| d.get_temp(variant_stack_id()))
        .unwrap_or_default();
    stack.push(v);
    ui.ctx()
        .data_mut(|d| d.insert_temp(variant_stack_id(), stack));
}

fn pop_variant(ui: &Ui) {
    let mut stack: Vec<AlertVariant> = ui
        .ctx()
        .data_mut(|d| d.get_temp(variant_stack_id()))
        .unwrap_or_default();
    stack.pop();
    ui.ctx()
        .data_mut(|d| d.insert_temp(variant_stack_id(), stack));
}

/// `AlertAction` — nova `absolute top-2 right-2` action slot. Shown inside
/// an [`Alert`], it paints at the alert's top-right corner without
/// consuming flow height (the alert reserves `pr-18` for it).
#[derive(Clone, Debug, Default)]
pub struct AlertAction {
    id_salt: Option<String>,
}

impl AlertAction {
    /// Creates an alert action slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the action contents at the alert's top-right.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        match current_alert(ui) {
            Some(ctx) => {
                // Mark the alert so the next frame reserves `pr-18`.
                ui.ctx()
                    .data_mut(|d| d.insert_temp(ctx.id.with("has_action"), true));
                let region = Rect::from_min_max(
                    Pos2::new(ctx.left, ctx.top),
                    Pos2::new(ctx.right, ctx.top + 100.0),
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(region)
                        .layout(egui::Layout::right_to_left(egui::Align::Min)),
                );
                let ret = content(&mut child);
                InnerResponse::new(ret, child.response())
            }
            None => {
                let inner = ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), content);
                InnerResponse::new(inner.inner, inner.response)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Vec2;

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn paddings_match_style_recipes() {
        // base `px-4 py-3`, nova `px-2.5 py-2`.
        assert_eq!(padding_x(Style::Vega), 16.0);
        assert_eq!(padding_y(Style::Vega), 12.0);
        assert_eq!(padding_x(Style::Nova), 10.0);
        assert_eq!(padding_y(Style::Nova), 8.0);
        assert_eq!(icon_gap(Style::Nova), 8.0);
        assert_eq!(icon_gap(Style::Vega), 12.0);
    }

    #[test]
    fn destructive_uses_destructive_text() {
        let p = palette(Mode::Light);
        let d = resolve_alert_colors(&p, AlertVariant::Destructive);
        assert_eq!(d.fill, p.card);
        assert_eq!(d.title, p.destructive);
        assert_eq!(d.description, color_alpha(p.destructive, 0.9));
        assert_eq!(d.icon, p.destructive);
        assert_eq!(d.stroke.color, p.border);

        let def = resolve_alert_colors(&p, AlertVariant::Default);
        assert_eq!(def.title, p.card_foreground);
        assert_eq!(def.description, p.muted_foreground);
        assert_eq!(def.icon, p.card_foreground);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("alert_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                Alert::new()
                    .id_salt("render_alert")
                    .icon(Icon::CircleCheck)
                    .show(ui, |ui| {
                        AlertTitle::new("Heads up!").show(ui);
                        AlertDescription::new("Description line.").show(ui);
                        AlertAction::new().show(ui, |ui| {
                            ui.label("Retry");
                        });
                    });
            });
            out.textures_delta.clear();
        }
    }
}
