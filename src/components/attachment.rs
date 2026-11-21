//! shadcn `attachment.tsx` — file chips for uploads and downloads.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/attachment.tsx`: a
//! `rounded-xl border bg-card` chip holding a square [`AttachmentMedia`]
//! (`w-10 rounded-lg bg-muted`, `w-8` sm, `w-7` xs), a truncating
//! [`AttachmentTitle`] / [`AttachmentDescription`] pair inside
//! [`AttachmentContent`], and ghost icon-button [`AttachmentAction`]s inside
//! [`AttachmentActions`]. States drive the decoration: `idle` gets a dashed
//! border, `error` destructive border/media/description colors, and
//! `uploading`/`processing` shimmer the title (approximated with a color
//! pulse — CSS gradient shimmer is not portable to egui). `horizontal`
//! chips lay out as a row (`min-w-40`); `vertical` chips are a fixed
//! `w-30` (120px) column with actions overlaid on the media (`w-24`
//! media-only is not detected pre-layout, so vertical is always `w-30`).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Attachment::new()
//!     .state(AttachmentState::Error)
//!     .show(ui, |ui| {
//!         AttachmentMedia::new().icon(Icon::CircleAlert).show(ui);
//!         AttachmentContent::new().show(ui, |ui| {
//!             AttachmentTitle::new("financial-model.xlsx").show(ui);
//!             AttachmentDescription::new("Upload failed. Try again.").show(ui);
//!         });
//!         AttachmentActions::new().show(ui, |ui| {
//!             if AttachmentAction::new(Icon::RefreshCw).show(ui).clicked() {
//!                 // retry
//!             }
//!             if AttachmentAction::new(Icon::X).show(ui).clicked() {
//!                 // remove
//!             }
//!         });
//!     });
//! ```

use crate::components::avatar::{cover_uv, resolve_texture};
use crate::components::spinner::{angle_at, arc_points};
use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use egui::epaint::RectShape;
use egui::{
    Color32, Id, ImageSource, Pos2, Rect, Response, RichText, Sense, Shape, Stroke, Ui, Vec2,
    Widget,
};

/// Vertical chip width (`w-30` = 120px; the `w-24` media-only variant needs
/// post-layout slot detection and is not used).
pub(crate) const VERTICAL_CHIP_WIDTH: f32 = 120.0;

/// The upload lifecycle states, mirroring the source `state` prop
/// (`data-state`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttachmentState {
    /// Pre-upload: dashed border.
    Idle,
    /// Transfer in progress: title shimmers.
    Uploading,
    /// Post-upload processing: title shimmers.
    Processing,
    /// Failed: destructive border, media and description.
    Error,
    /// Complete (the source default).
    #[default]
    Done,
}

impl AttachmentState {
    /// True for the two in-flight states (`uploading`, `processing`) that
    /// shimmer the title.
    pub fn is_pending(self) -> bool {
        matches!(
            self,
            AttachmentState::Uploading | AttachmentState::Processing
        )
    }
}

/// Size presets, mirroring the source `size` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttachmentSize {
    /// `w-10` media, `text-sm`, `px-2.5 py-2` padding.
    #[default]
    Default,
    /// `w-8` media, `text-xs`, `px-2 py-1.5` padding.
    Sm,
    /// `w-7` media, `text-xs`, `px-1.5 py-1` padding, `rounded-lg` chip.
    Xs,
}

impl AttachmentSize {
    /// Media box side length (`w-10` / `w-8` / `w-7`).
    pub fn media_side(self) -> f32 {
        match self {
            AttachmentSize::Default => 40.0,
            AttachmentSize::Sm => 32.0,
            AttachmentSize::Xs => 28.0,
        }
    }

    /// Media icon size (`size-4`, xs `size-3.5`).
    pub fn media_icon(self) -> f32 {
        match self {
            AttachmentSize::Default | AttachmentSize::Sm => 16.0,
            AttachmentSize::Xs => 14.0,
        }
    }

    /// Chip inner padding when an `AttachmentContent` slot is present
    /// (`has-data-[slot=attachment-content]:px-2.5 py-2` etc.).
    pub fn pad(self) -> Vec2 {
        match self {
            AttachmentSize::Default => Vec2::new(10.0, 8.0),
            AttachmentSize::Sm => Vec2::new(8.0, 6.0),
            AttachmentSize::Xs => Vec2::new(6.0, 4.0),
        }
    }

    /// Chip padding when only an `AttachmentMedia` slot is present
    /// (`has-data-[slot=attachment-media]:p-2` etc.): uniform on all sides.
    pub fn media_pad(self) -> f32 {
        match self {
            AttachmentSize::Default => 8.0,
            AttachmentSize::Sm => 6.0,
            AttachmentSize::Xs => 4.0,
        }
    }

    /// Gap between slots (`gap-2` / `gap-2.5` / `gap-1.5`).
    pub fn gap(self) -> f32 {
        match self {
            AttachmentSize::Default => 8.0,
            AttachmentSize::Sm => 10.0,
            AttachmentSize::Xs => 6.0,
        }
    }

    /// Text size (`text-sm` / `text-xs`).
    pub fn text_size(self) -> f32 {
        match self {
            AttachmentSize::Default => crate::theme::tokens::type_scale::SM,
            AttachmentSize::Sm | AttachmentSize::Xs => crate::theme::tokens::type_scale::XS,
        }
    }
}

/// Vertical media side (border-box, spec §8.3): the chip is 120px wide, so
/// the media box is the chip width minus the horizontal padding on both
/// sides minus the 1px border on each side — 98px / 102px / 106px for
/// default / sm / xs. The audit's 100/104/108 omitted the border. Pure and
/// testable.
pub(crate) fn vertical_media_side(size: AttachmentSize) -> f32 {
    VERTICAL_CHIP_WIDTH - 2.0 * size.pad().x - 2.0
}

/// Media corner radius: `rounded-md` for xs in both orientations
/// (`rounded-md!` overrides the `rounded-lg` base), `rounded-lg` otherwise.
/// Pure and testable.
fn media_radius(size: AttachmentSize, md: u8, lg: u8) -> u8 {
    if size == AttachmentSize::Xs {
        md
    } else {
        lg
    }
}

/// Chip padding for a size given the slots present
/// (`has-data-[slot=attachment-content]` / `has-data-[slot=attachment-media]`
/// rules, N5): the content pad when content is present, the media pad when
/// only media is present, no padding when neither slot is present. Pure and
/// testable.
pub(crate) fn chip_pad(size: AttachmentSize, has_content: bool, has_media: bool) -> Vec2 {
    if has_content {
        size.pad()
    } else if has_media {
        Vec2::splat(size.media_pad())
    } else {
        Vec2::ZERO
    }
}

/// `has-[>a,>button]:hover:bg-muted/50` — the hover fill applies only when
/// the chip itself is a trigger (a direct-child `<button>`); buttons nested
/// inside `AttachmentActions` (a `<div>`) do NOT enable it. Pure and
/// testable.
pub(crate) fn hover_fill_applies(trigger: bool, hovered: bool) -> bool {
    trigger && hovered
}

/// Layout orientation, mirroring the source `orientation` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttachmentOrientation {
    /// A `min-w-40` row (the source default).
    #[default]
    Horizontal,
    /// A fixed-width column with actions overlaid on the media.
    Vertical,
}

/// Media variant, mirroring the source `variant` prop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AttachmentMediaVariant {
    /// An icon (or spinner) centered in the muted square (the default).
    #[default]
    Icon,
    /// An image preview (`object-cover`), dimmed unless done/idle.
    Image,
}

/// Layout context pushed by [`Attachment`] for its slot children.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ScopeState {
    pub chip_id: Id,
    pub orientation: AttachmentOrientation,
    pub size: AttachmentSize,
    pub state: AttachmentState,
    pub media_rect: Option<Rect>,
    /// A content slot (`AttachmentContent`) rendered inside this chip.
    pub has_content: bool,
    /// A media slot (`AttachmentMedia`) rendered inside this chip.
    pub has_media: bool,
    pub focus_within: bool,
}

fn scope_stack_id() -> Id {
    Id::unique("shadcn_attachment_scope_stack")
}

pub(crate) mod scope {
    use super::{scope_stack_id, ScopeState};
    use egui::Ui;

    pub(crate) fn push(ui: &Ui, state: ScopeState) {
        let mut stack: Vec<ScopeState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(scope_stack_id()))
            .unwrap_or_default();
        stack.push(state);
        ui.ctx()
            .data_mut(|d| d.insert_temp(scope_stack_id(), stack));
    }

    pub(crate) fn pop(ui: &Ui) {
        let mut stack: Vec<ScopeState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(scope_stack_id()))
            .unwrap_or_default();
        stack.pop();
        ui.ctx()
            .data_mut(|d| d.insert_temp(scope_stack_id(), stack));
    }

    pub(crate) fn current(ui: &Ui) -> Option<ScopeState> {
        ui.ctx()
            .data_mut(|d| d.get_temp::<Vec<ScopeState>>(scope_stack_id()))
            .and_then(|s| s.last().copied())
    }

    /// Mutates the top-of-stack scope (no-op outside an [`super::Attachment`]).
    pub(crate) fn update(ui: &Ui, f: impl FnOnce(&mut ScopeState)) {
        let mut stack: Vec<ScopeState> = ui
            .ctx()
            .data_mut(|d| d.get_temp(scope_stack_id()))
            .unwrap_or_default();
        if let Some(top) = stack.last_mut() {
            f(top);
        }
        ui.ctx()
            .data_mut(|d| d.insert_temp(scope_stack_id(), stack));
    }
}

/// The square media box (`AttachmentMedia`): an icon, an image preview, or a
/// spinner, on a `bg-muted` rounded square.
#[derive(Clone, Debug, Default)]
pub struct AttachmentMedia<'a> {
    variant: AttachmentMediaVariant,
    icon: Option<Icon>,
    image: Option<ImageSource<'a>>,
    spinner: bool,
    id_salt: Option<String>,
}

impl<'a> AttachmentMedia<'a> {
    /// Creates an icon-variant media box with no content.
    pub fn new() -> Self {
        Self {
            variant: AttachmentMediaVariant::default(),
            icon: None,
            image: None,
            spinner: false,
            id_salt: None,
        }
    }

    /// Sets the variant (`icon` by default).
    pub fn variant(mut self, variant: AttachmentMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Shows an icon (lucide `size-4`, vertical `size-6`).
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Shows an image preview (`variant="image"`, `object-cover`).
    pub fn image(mut self, image: impl Into<ImageSource<'a>>) -> Self {
        self.image = Some(image.into());
        self.variant = AttachmentMediaVariant::Image;
        self
    }

    /// Shows a spinner (the uploading state's `<Spinner />` slot).
    pub fn spinner(mut self, spinner: bool) -> Self {
        self.spinner = spinner;
        self
    }

    /// Sets a stable id salt (AttachmentMedia keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the media box.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let scope = scope::current(ui);
        let state = scope.map(|s| s.state).unwrap_or_default();
        let (side, radius, icon_size) = match scope.map(|s| (s.size, s.orientation)) {
            Some((_, AttachmentOrientation::Vertical)) => {
                // `w-full` inside the chip's inner width (per-size padding,
                // border-box: the 1px border on each side, spec §8.3).
                let size = scope.map(|s| s.size).unwrap_or_default();
                (
                    vertical_media_side(size),
                    media_radius(size, theme.radius.md_u8(), theme.radius.lg_u8()),
                    24.0, // `size-6`
                )
            }
            _ => {
                let size = scope.map(|s| s.size).unwrap_or_default();
                (
                    size.media_side(),
                    media_radius(size, theme.radius.md_u8(), theme.radius.lg_u8()),
                    size.media_icon(),
                )
            }
        };
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(side), Sense::hover());
        if !ui.is_rect_visible(rect) {
            return response;
        }
        scope::update(ui, |s| {
            s.media_rect = Some(rect);
            s.has_media = true;
        });

        let painter = ui.painter();
        // `bg-muted`; `error` → `bg-destructive/10 text-destructive`.
        let (bg, fg) = if state == AttachmentState::Error {
            (color_alpha(p.destructive, 0.10), p.destructive)
        } else {
            (p.muted, p.foreground)
        };
        painter.rect_filled(rect, radius, bg);

        if self.spinner {
            // Animated loader arc (lucide `LoaderCircle`, `animate-spin`).
            // OPEN-6: the source sizes the spinner only for the vertical
            // orientation (`size-6!` = 24px); 24px is used by analogy in the
            // horizontal orientation too, independent of the icon size.
            let spinner_size = 24.0;
            ui.ctx().request_repaint();
            let elapsed = ui.input(|i| i.time);
            let angle = angle_at(elapsed);
            let scale = spinner_size / 24.0;
            let stroke = Stroke::new(2.0 * scale, fg);
            let points: Vec<Pos2> = arc_points(angle, 24)
                .iter()
                .map(|pt| {
                    Pos2::new(
                        rect.center().x + (pt[0] - 12.0) * scale,
                        rect.center().y + (pt[1] - 12.0) * scale,
                    )
                })
                .collect();
            painter.add(Shape::line(points, stroke));
        } else if self.variant == AttachmentMediaVariant::Image {
            // `opacity-60` until the transfer completes (`done`/`idle`).
            let full_opacity = matches!(state, AttachmentState::Done | AttachmentState::Idle);
            let tint = if full_opacity {
                Color32::WHITE
            } else {
                Color32::from_white_alpha(153) // 0.6 × 255
            };
            if let Some(source) = self.image.as_ref() {
                if let Some(tex) = resolve_texture(ui, source, side) {
                    let uv = cover_uv(tex.size, Vec2::splat(side));
                    painter.add(RectShape::filled(rect, radius, tint).with_texture(tex.id, uv));
                }
            }
        } else if let Some(icon) = self.icon {
            let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
            icon.paint(painter, icon_rect, fg, 2.0 * icon_size / 24.0);
        }

        response
    }
}

impl Widget for AttachmentMedia<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// `AttachmentContent` — the flexible text column (`flex-1 min-w-0
/// leading-tight`); title and description stack vertically and truncate at
/// the container edge unless capped with [`AttachmentContent::max_width`].
#[derive(Clone, Debug)]
pub struct AttachmentContent {
    max_width: f32,
    id_salt: Option<String>,
}

impl Default for AttachmentContent {
    fn default() -> Self {
        Self::new()
    }
}

impl AttachmentContent {
    /// Creates an empty content column. The column is uncapped by default
    /// (N9): the source `w-fit` chip grows with its content up to the
    /// container width; the old 240px cap is gone.
    pub fn new() -> Self {
        Self {
            max_width: f32::INFINITY,
            id_salt: None,
        }
    }

    /// Caps the column width (titles/descriptions truncate at it).
    pub fn max_width(mut self, max_width: f32) -> Self {
        self.max_width = max_width;
        self
    }

    /// Sets a stable id salt (AttachmentContent keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the content column around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let max_width = self.max_width;
        scope::update(ui, |s| s.has_content = true);
        let vertical =
            scope::current(ui).map(|s| s.orientation) == Some(AttachmentOrientation::Vertical);
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            // N9: only cap the column when the caller set an explicit finite
            // width; the default is unbounded (container edge).
            if max_width.is_finite() {
                ui.set_max_width(max_width);
            }
            ui.spacing_mut().item_spacing.y = 2.0; // `mt-0.5` between slots
            if vertical {
                // Vertical chips pin the column to the chip's inner width.
                let pad_x = scope::current(ui)
                    .map(|s| s.size.pad().x)
                    .unwrap_or_else(|| AttachmentSize::default().pad().x);
                ui.set_min_width(VERTICAL_CHIP_WIDTH - 2.0 * pad_x);
                // `px-1` — a horizontal 4px inset on each side of the text
                // column (not vertical space).
                egui::Frame::new()
                    .inner_margin(egui::Margin::symmetric(4, 0))
                    .show(ui, content)
                    .inner
            } else {
                content(ui)
            }
        })
    }
}

/// `AttachmentTitle` — the `font-medium` primary text (`truncate` + shimmer
/// while uploading/processing).
#[derive(Clone, Debug)]
pub struct AttachmentTitle {
    text: String,
    id_salt: Option<String>,
}

impl AttachmentTitle {
    /// Creates a title.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            id_salt: None,
        }
    }

    /// Sets a stable id salt (AttachmentTitle keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the (truncating) title.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let state = scope::current(ui).map(|s| s.state).unwrap_or_default();
        let size = scope::current(ui).map(|s| s.size).unwrap_or_default();
        let mut color = p.card_foreground;
        // `group-data-[state=uploading|processing]:shimmer` — approximated
        // with a color pulse between muted-foreground and card-foreground.
        if state.is_pending() {
            ui.ctx().request_repaint();
            let phase = shimmer_phase(ui.input(|i| i.time));
            color = mix(p.muted_foreground, p.card_foreground, phase);
        }
        let text = RichText::new(self.text)
            .size(size.text_size())
            .strong() // `font-medium` (no medium font is configured)
            .color(color);
        ui.add(egui::Label::new(text).truncate())
    }
}

/// `AttachmentDescription` — the `text-xs text-muted-foreground` secondary
/// text (destructive-tinted in the error state).
#[derive(Clone, Debug)]
pub struct AttachmentDescription {
    text: String,
    id_salt: Option<String>,
}

impl AttachmentDescription {
    /// Creates a description.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            id_salt: None,
        }
    }

    /// Sets a stable id salt (AttachmentDescription keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the (truncating) description.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let state = scope::current(ui).map(|s| s.state).unwrap_or_default();
        let color = if state == AttachmentState::Error {
            color_alpha(p.destructive, 0.80)
        } else {
            p.muted_foreground
        };
        let text = RichText::new(self.text)
            .size(crate::theme::tokens::type_scale::XS)
            .color(color);
        ui.add(egui::Label::new(text).truncate())
    }
}

/// `AttachmentActions` — the row of [`AttachmentAction`] buttons. In the
/// vertical orientation it floats over the media's top-right corner
/// (`absolute top-3 right-3`) instead of taking an inline slot.
#[derive(Clone, Debug, Default)]
pub struct AttachmentActions {
    id_salt: Option<String>,
}

impl AttachmentActions {
    /// Creates an actions container.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt (AttachmentActions keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the actions row around its contents.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let scope = scope::current(ui);
        let (orientation, media_rect, chip_id, size) = match scope {
            Some(s) => (s.orientation, s.media_rect, s.chip_id, s.size),
            None => (
                AttachmentOrientation::Horizontal,
                None,
                Id::unique("attachment_orphan_actions"),
                AttachmentSize::default(),
            ),
        };
        // Vertical: overlay the row on the media's top-right corner.
        if orientation == AttachmentOrientation::Vertical {
            if let Some(media) = media_rect {
                // `absolute top-3 right-3`: the row's RIGHT edge sits 12px
                // from the chip's right edge and the row grows leftward
                // regardless of how many actions it holds (N2). The chip's
                // right edge in screen coordinates is
                // `media.left - pad.x + VERTICAL_CHIP_WIDTH`; pin the
                // overlay there with a RIGHT_TOP anchor against the content
                // rect (the rect `Area` resolves anchors against — an
                // `Area::fixed_pos` left edge cannot express a right-edge pin
                // for an opaque closure of unknown width).
                let overlay_right = media.left() + VERTICAL_CHIP_WIDTH - size.pad().x - 12.0;
                let content_rect = ui.ctx().content_rect();
                let area_id = chip_id.with("attachment_actions_overlay");
                let inner = egui::Area::new(area_id)
                    .anchor(
                        egui::Align2::RIGHT_TOP,
                        egui::Vec2::new(
                            overlay_right - content_rect.right(),
                            media.top() + 12.0 - content_rect.top(),
                        ),
                    )
                    .order(egui::Order::Middle)
                    .show(ui.ctx(), |ui| {
                        ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                            ui.spacing_mut().item_spacing.x = 4.0; // overlay `gap-1`
                            content(ui)
                        })
                        .inner
                    })
                    .inner;
                // Reserve no inline space; the row floats.
                let spacer = ui.interact(
                    Rect::NOTHING,
                    chip_id.with("attachment_actions_spacer"),
                    Sense::hover(),
                );
                return egui::InnerResponse::new(inner, spacer);
            }
        }
        ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
            // No inter-action gap in the horizontal orientation: the source
            // row is a bare flex row (`gap-1` applies to the vertical
            // overlay only, N3).
            ui.spacing_mut().item_spacing.x = 0.0;
            content(ui)
        })
    }
}

/// `AttachmentAction` — a ghost `icon-xs`
/// [`Button`](crate::components::button::Button) inside [`AttachmentActions`]
/// (remove, retry, cancel...). The source accepts any `Button`
/// `variant`/`size`; override the ghost/`icon-xs` defaults with
/// [`AttachmentAction::variant`] / [`AttachmentAction::size`].
#[derive(Clone, Debug)]
pub struct AttachmentAction {
    icon: Icon,
    disabled: bool,
    variant: crate::components::button::ButtonVariant,
    size: crate::components::button::ButtonSize,
    id_salt: Option<String>,
}

impl AttachmentAction {
    /// Creates an icon-only action.
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            disabled: false,
            variant: crate::components::button::ButtonVariant::Ghost,
            size: crate::components::button::ButtonSize::IconXs,
            id_salt: None,
        }
    }

    /// Disables the action.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides the button variant (default `ghost`).
    pub fn variant(mut self, variant: crate::components::button::ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Overrides the button size (default `icon-xs`).
    pub fn size(mut self, size: crate::components::button::ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Sets a stable id salt so focus/behavior persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the action button.
    pub fn show(self, ui: &mut Ui) -> Response {
        let mut button = crate::components::button::Button::new("")
            .variant(self.variant)
            .size(self.size)
            .icon_start(self.icon);
        if let Some(salt) = self.id_salt.as_deref() {
            button = button.id_salt(salt);
        }
        if self.disabled {
            button = button.disabled(true);
        }
        let response = button.show(ui);
        // `focus-within:ring-1 focus-within:ring-ring/50` on the chip.
        if response.has_focus() {
            scope::update(ui, |s| s.focus_within = true);
        }
        response
    }
}

impl Widget for AttachmentAction {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}

/// The attachment chip (`Attachment`) — `rounded-xl border bg-card` with
/// state-driven decoration. Compose it from the slot components; the
/// returned response is the chip surface (hover), and `.trigger(true)`
/// mirrors `AttachmentTrigger` by making the whole chip clickable.
#[derive(Clone, Debug, Default)]
pub struct Attachment {
    state: AttachmentState,
    size: AttachmentSize,
    orientation: AttachmentOrientation,
    trigger: bool,
    id_salt: Option<String>,
}

impl Attachment {
    /// Creates a `done`, default-size, horizontal chip.
    pub fn new() -> Self {
        Self {
            state: AttachmentState::default(),
            size: AttachmentSize::default(),
            orientation: AttachmentOrientation::default(),
            trigger: false,
            id_salt: None,
        }
    }

    /// Sets the lifecycle state (`data-state`).
    pub fn state(mut self, state: AttachmentState) -> Self {
        self.state = state;
        self
    }

    /// Sets the size preset.
    pub fn size(mut self, size: AttachmentSize) -> Self {
        self.size = size;
        self
    }

    /// Sets the layout orientation.
    pub fn orientation(mut self, orientation: AttachmentOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Makes the whole chip a click target (`AttachmentTrigger`). The
    /// interact uses `Sense::click()`, which is focusable in egui — matching
    /// the source's focusable `<button>` (and `:focus-within` matches the
    /// focused element itself too).
    pub fn trigger(mut self, trigger: bool) -> Self {
        self.trigger = trigger;
        self
    }

    /// Sets a stable id salt (hover state and vertical overlays key off it).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the chip around its slot children.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        // Layout pad: the vertical `set_min_width` below is computed before
        // layout (slot presence is unknowable there — D5), so it stays
        // content-pad-based; the painted chip rect uses the slot-conditional
        // pad (N5) instead.
        let layout_pad = self.size.pad();
        let radius = if self.size == AttachmentSize::Xs {
            theme.radius.lg_u8()
        } else {
            theme.radius.xl_u8()
        };

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_attachment"),
        };

        scope::push(
            ui,
            ScopeState {
                chip_id: id,
                orientation: self.orientation,
                size: self.size,
                state: self.state,
                media_rect: None,
                has_content: false,
                has_media: false,
                focus_within: false,
            },
        );

        // The chip surface: fill + border are painted behind the children
        // via a reserved shape slot (hover is read for the current pass from
        // the post-layout interact below).
        let bg_idx = ui.painter().add(Shape::Noop);

        let laid_out = match self.orientation {
            AttachmentOrientation::Horizontal => {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.x = self.size.gap();
                    content(ui)
                })
            }
            AttachmentOrientation::Vertical => {
                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                    ui.set_min_width(VERTICAL_CHIP_WIDTH - 2.0 * layout_pad.x);
                    ui.spacing_mut().item_spacing.y = self.size.gap();
                    content(ui)
                })
            }
        };
        let _inner = laid_out.inner;

        // Slot-conditional padding (N5): read the scope after layout, when
        // the slots have recorded their presence.
        let scope_now = scope::current(ui);
        let (has_content, has_media) = scope_now
            .map(|s| (s.has_content, s.has_media))
            .unwrap_or((false, false));
        let rect_pad = chip_pad(self.size, has_content, has_media);

        let chip_rect = match self.orientation {
            AttachmentOrientation::Horizontal => {
                // Row + padding; `min-w-40` keeps tiny chips usable.
                let mut rect = laid_out.response.rect.expand2(rect_pad);
                if rect.width() < 160.0 {
                    rect.set_width(160.0);
                }
                rect
            }
            AttachmentOrientation::Vertical => {
                let mut rect = laid_out.response.rect.expand2(rect_pad);
                rect.set_width(VERTICAL_CHIP_WIDTH);
                rect
            }
        };

        let sense = if self.trigger {
            Sense::click()
        } else {
            Sense::hover()
        };
        let response = ui.interact(chip_rect, id, sense);
        // `has-[>a,>button]:hover:bg-muted/50` — the hover fill applies only
        // when the chip itself is a trigger (a direct-child `<button>`);
        // buttons nested inside `AttachmentActions` do NOT enable it.
        let hovered = hover_fill_applies(self.trigger, response.hovered());

        // Border: destructive/30 in the error state, dashed while idle.
        let border_color = if self.state == AttachmentState::Error {
            color_alpha(p.destructive, 0.30)
        } else {
            p.border
        };
        let bg = if hovered {
            color_alpha(p.muted, 0.50)
        } else {
            p.card
        };

        if ui.is_rect_visible(chip_rect) {
            let painter = ui.painter();
            let shapes = if self.state == AttachmentState::Idle {
                // `border-dashed` (epaint has no dash support).
                let dashes = dashed_rect_segments(chip_rect, radius as f32, 4.0, 4.0);
                vec![
                    Shape::Rect(RectShape::filled(chip_rect, radius, bg)),
                    Shape::Vec(
                        dashes
                            .into_iter()
                            .map(|(a, b)| {
                                Shape::line_segment([a, b], Stroke::new(1.0, border_color))
                            })
                            .collect(),
                    ),
                ]
            } else {
                vec![
                    Shape::Rect(RectShape::filled(chip_rect, radius, bg)),
                    Shape::rect_stroke(
                        chip_rect,
                        radius,
                        Stroke::new(1.0, border_color),
                        egui::epaint::StrokeKind::Inside,
                    ),
                ]
            };
            painter.set(bg_idx, Shape::Vec(shapes));
            // `focus-within:ring-1 focus-within:ring-ring/50` — matches when
            // a descendant (action) or the trigger chip itself holds focus.
            let focused = scope_now.map(|s| s.focus_within).unwrap_or(false)
                || (self.trigger && response.has_focus());
            if focused {
                crate::utils::focus::paint_ring(
                    painter,
                    chip_rect.expand(1.0),
                    color_alpha(p.ring, 0.5),
                    1.0,
                    radius as f32 + 1.0,
                );
            }
        }

        scope::pop(ui);
        response
    }
}

impl Widget for Attachment {
    fn ui(self, ui: &mut Ui) -> Response {
        // A bare `ui.add(Attachment)` has no slots; it still renders the
        // empty chip surface.
        self.show(ui, |_ui| {})
    }
}

/// `AttachmentGroup` — the `gap-3` horizontal row of chips
/// (`overflow-x-auto snap-x`): scrolls horizontally when the chips overflow.
#[derive(Clone, Debug, Default)]
pub struct AttachmentGroup {
    id_salt: Option<String>,
}

impl AttachmentGroup {
    /// Creates an attachment group.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt (AttachmentGroup keeps no state).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the group around its chips.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        // `py-1` breathing room inside the scroll strip; `scrollbar-none` —
        // the source hides the scrollbar (the fade still shows overflow).
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(0, 4))
            .show(ui, |ui| {
                egui::ScrollArea::horizontal()
                    .auto_shrink([false, true])
                    .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 12.0; // gap-3
                            content(ui)
                        })
                        .inner
                    })
                    .inner
            })
    }
}

/// Shimmer phase at `elapsed_secs` for the pending-state title pulse
/// (one cycle per 1.6s, in `0..=1`). Pure and testable.
pub(crate) fn shimmer_phase(elapsed_secs: f64) -> f32 {
    (0.5 + 0.5 * (elapsed_secs * std::f64::consts::TAU / 1.6).sin() as f32).clamp(0.0, 1.0)
}

/// Linear color mix (`t = 0` → `a`, `t = 1` → `b`). Pure and testable.
pub(crate) fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let lerp = |x: u8, y: u8| -> u8 { (x as f32 + (y as f32 - x as f32) * t).round() as u8 };
    Color32::from_rgba_premultiplied(
        lerp(a.r(), b.r()),
        lerp(a.g(), b.g()),
        lerp(a.b(), b.b()),
        lerp(a.a(), b.a()),
    )
}

/// One segment of the rounded-rect perimeter walk: a straight run or a
/// quarter-circle arc (center, start angle, sweep).
enum PerimeterSeg {
    Line(Pos2, Pos2),
    Arc(Pos2, f32, f32),
}

impl PerimeterSeg {
    fn length(&self) -> f32 {
        match self {
            PerimeterSeg::Line(a, b) => a.distance(*b),
            PerimeterSeg::Arc(_, _, sweep) => sweep.abs(),
        }
    }

    fn point_at(&self, r: f32, t: f32) -> Pos2 {
        match self {
            PerimeterSeg::Line(a, b) => Pos2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t),
            PerimeterSeg::Arc(c, start, sweep) => {
                let ang = start + sweep * t;
                let (sin, cos) = ang.sin_cos();
                Pos2::new(c.x + r * cos, c.y + r * sin)
            }
        }
    }
}

/// Points along a rounded-rect perimeter at arc-length `s` from the top-left
/// straight-edge start, walking clockwise. Pure and testable.
fn rounded_rect_point(rect: Rect, radius: f32, s: f32) -> Pos2 {
    let r = radius
        .min(rect.width() * 0.5)
        .min(rect.height() * 0.5)
        .max(0.0);
    let top_len = (rect.width() - 2.0 * r).max(0.0);
    let side_len = (rect.height() - 2.0 * r).max(0.0);
    let corner_len = std::f32::consts::FRAC_PI_2 * r;
    let mut s = s.clamp(0.0, 2.0 * (top_len + side_len) + 4.0 * corner_len);

    let quarter = std::f32::consts::FRAC_PI_2;
    let tr = Pos2::new(rect.right() - r, rect.top() + r);
    let br = Pos2::new(rect.right() - r, rect.bottom() - r);
    let bl = Pos2::new(rect.left() + r, rect.bottom() - r);
    let tl = Pos2::new(rect.left() + r, rect.top() + r);
    let segs = [
        PerimeterSeg::Line(
            Pos2::new(rect.left() + r, rect.top()),
            Pos2::new(rect.right() - r, rect.top()),
        ),
        PerimeterSeg::Arc(tr, -90.0f32.to_radians(), quarter),
        PerimeterSeg::Line(
            Pos2::new(rect.right(), rect.top() + r),
            Pos2::new(rect.right(), rect.bottom() - r),
        ),
        PerimeterSeg::Arc(br, 0.0, quarter),
        PerimeterSeg::Line(
            Pos2::new(rect.right() - r, rect.bottom()),
            Pos2::new(rect.left() + r, rect.bottom()),
        ),
        PerimeterSeg::Arc(bl, 90.0f32.to_radians(), quarter),
        PerimeterSeg::Line(
            Pos2::new(rect.left(), rect.bottom() - r),
            Pos2::new(rect.left(), rect.top() + r),
        ),
        PerimeterSeg::Arc(tl, 180.0f32.to_radians(), quarter),
    ];

    for seg in &segs {
        let len = match seg {
            PerimeterSeg::Line(..) => seg.length(),
            PerimeterSeg::Arc(..) => corner_len,
        };
        if s <= len {
            let t = if len > 0.0 { s / len } else { 0.0 };
            return seg.point_at(r, t);
        }
        s -= len;
    }
    // Numerical fallback: back to the start.
    Pos2::new(rect.left() + r, rect.top())
}

/// The perimeter length of a rounded rect (sizes the dash loop). Pure.
fn rounded_rect_perimeter(rect: Rect, radius: f32) -> f32 {
    let r = radius
        .min(rect.width() * 0.5)
        .min(rect.height() * 0.5)
        .max(0.0);
    2.0 * (rect.width() - 2.0 * r).max(0.0)
        + 2.0 * (rect.height() - 2.0 * r).max(0.0)
        + std::f32::consts::TAU * r
}

/// Dash segments (`dash` drawn, `gap` skipped) along a rounded-rect border.
/// Pure; rendered as individual line segments.
fn dashed_rect_segments(rect: Rect, radius: f32, dash: f32, gap: f32) -> Vec<(Pos2, Pos2)> {
    let period = (dash + gap).max(0.001);
    let total = rounded_rect_perimeter(rect, radius);
    let mut out = Vec::new();
    let mut s = 0.0;
    while s < total {
        let end = (s + dash).min(total);
        out.push((
            rounded_rect_point(rect, radius, s),
            rounded_rect_point(rect, radius, end),
        ));
        s += period;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    #[test]
    fn sizes_match_source() {
        // w-10 / w-8 / w-7; icon size-4 / size-3.5.
        assert_eq!(AttachmentSize::Default.media_side(), 40.0);
        assert_eq!(AttachmentSize::Sm.media_side(), 32.0);
        assert_eq!(AttachmentSize::Xs.media_side(), 28.0);
        assert_eq!(AttachmentSize::Default.media_icon(), 16.0);
        assert_eq!(AttachmentSize::Xs.media_icon(), 14.0);
        // gap-2 / gap-2.5 / gap-1.5.
        assert_eq!(AttachmentSize::Default.gap(), 8.0);
        assert_eq!(AttachmentSize::Sm.gap(), 10.0);
        assert_eq!(AttachmentSize::Xs.gap(), 6.0);
        // px-2.5 py-2 / px-2 py-1.5 / px-1.5 py-1.
        assert_eq!(AttachmentSize::Default.pad(), Vec2::new(10.0, 8.0));
        assert_eq!(AttachmentSize::Sm.pad(), Vec2::new(8.0, 6.0));
        assert_eq!(AttachmentSize::Xs.pad(), Vec2::new(6.0, 4.0));
        // Media-only chip pad (p-2 / p-1.5 / p-1): uniform, equals pad().y.
        assert_eq!(AttachmentSize::Default.media_pad(), 8.0);
        assert_eq!(AttachmentSize::Sm.media_pad(), 6.0);
        assert_eq!(AttachmentSize::Xs.media_pad(), 4.0);
        // text-sm / text-xs.
        assert_eq!(AttachmentSize::Default.text_size(), 14.0);
        assert_eq!(AttachmentSize::Xs.text_size(), 12.0);
    }

    #[test]
    fn state_helpers() {
        assert!(AttachmentState::Uploading.is_pending());
        assert!(AttachmentState::Processing.is_pending());
        assert!(!AttachmentState::Done.is_pending());
        assert!(!AttachmentState::Idle.is_pending());
        assert!(!AttachmentState::Error.is_pending());
        assert_eq!(AttachmentState::default(), AttachmentState::Done);
    }

    #[test]
    fn shimmer_phase_osculates() {
        assert!((shimmer_phase(0.0) - 0.5).abs() < 1e-4);
        let p = shimmer_phase(0.4); // quarter cycle
        assert!(p > 0.9, "phase {p} should be near 1");
        assert!((shimmer_phase(1.6) - 0.5).abs() < 1e-3); // one cycle
        assert!(shimmer_phase(100.0) >= 0.0 && shimmer_phase(100.0) <= 1.0);
    }

    #[test]
    fn mix_lerps_channels() {
        let a = Color32::from_rgba_premultiplied(0, 0, 0, 255);
        let b = Color32::from_rgba_premultiplied(200, 100, 50, 255);
        assert_eq!(mix(a, b, 0.0), a);
        assert_eq!(mix(a, b, 1.0), b);
        let mid = mix(a, b, 0.5);
        assert_eq!(mid.r(), 100);
        assert_eq!(mid.g(), 50);
        // Clamped out-of-range t.
        assert_eq!(mix(a, b, -1.0), a);
        assert_eq!(mix(a, b, 2.0), b);
    }

    #[test]
    fn rounded_rect_perimeter_math() {
        // Plain rect: 2(w+h).
        let r = Rect::from_min_size(Pos2::ZERO, Vec2::new(30.0, 20.0));
        assert!((rounded_rect_perimeter(r, 0.0) - 100.0).abs() < 1e-4);
        // Radius 5: corners replace 4×5px straight runs with 2πr.
        let expected = 2.0 * 20.0 + 2.0 * 10.0 + std::f32::consts::TAU * 5.0;
        assert!((rounded_rect_perimeter(r, 5.0) - expected).abs() < 1e-4);
        // Radius clamped to half the shorter side (10): straight runs of
        // 2×(30−20) remain plus the two semicircles.
        let expected = 2.0 * 10.0 + std::f32::consts::TAU * 10.0;
        assert!((rounded_rect_perimeter(r, 99.0) - expected).abs() < 1e-3);
    }

    #[test]
    fn rounded_rect_point_walks_perimeter() {
        let r = Rect::from_min_size(Pos2::ZERO, Vec2::new(30.0, 20.0));
        // s=0: top-left straight-edge start.
        let p = rounded_rect_point(r, 0.0, 0.0);
        assert!((p.x - 0.0).abs() < 1e-4 && (p.y - 0.0).abs() < 1e-4);
        // Half the perimeter of a plain rect: bottom edge midpoint.
        let p = rounded_rect_point(r, 0.0, 50.0);
        assert!((p.y - 20.0).abs() < 1e-4);
        // Full perimeter returns near the start.
        let perim = rounded_rect_perimeter(r, 5.0);
        let p = rounded_rect_point(r, 5.0, perim);
        let start = rounded_rect_point(r, 5.0, 0.0);
        assert!(p.distance(start) < 1e-2, "{p:?} vs {start:?}");
        // Corner point at radius distance stays on the circle.
        let q = rounded_rect_point(r, 5.0, 20.0 + 1.0); // into the TR corner
        let center = Pos2::new(25.0, 5.0);
        assert!((q.distance(center) - 5.0).abs() < 1e-3);
    }

    #[test]
    fn dashed_segments_cover_perimeter() {
        let r = Rect::from_min_size(Pos2::ZERO, Vec2::new(60.0, 40.0));
        let segs = dashed_rect_segments(r, 8.0, 4.0, 4.0);
        assert!(!segs.is_empty());
        // Total drawn length ≈ half the perimeter (dash 4 of period 8).
        let drawn: f32 = segs.iter().map(|(a, b)| a.distance(*b)).sum();
        let perim = rounded_rect_perimeter(r, 8.0);
        assert!((drawn - perim * 0.5).abs() < 4.0, "{drawn} vs {perim}");
        // Segments stay inside the rect.
        for (a, b) in &segs {
            for p in [a, b] {
                assert!(r.expand(1.0).contains(*p), "{p:?} outside {r:?}");
            }
        }
    }

    #[test]
    fn vertical_chip_width_matches_source() {
        // w-30 (120px).
        assert_eq!(VERTICAL_CHIP_WIDTH, 120.0);
    }

    #[test]
    fn vertical_media_side_follows_chip_size() {
        // Border-box: 120 − 2·px − 2·1px border = 98 / 102 / 106.
        assert_eq!(vertical_media_side(AttachmentSize::Default), 98.0);
        assert_eq!(vertical_media_side(AttachmentSize::Sm), 102.0);
        assert_eq!(vertical_media_side(AttachmentSize::Xs), 106.0);
    }

    #[test]
    fn media_radius_selects_md_for_xs() {
        // `rounded-md!` overrides `rounded-lg` for xs in both orientations.
        assert_eq!(media_radius(AttachmentSize::Xs, 8, 10), 8);
        assert_eq!(media_radius(AttachmentSize::Default, 8, 10), 10);
        assert_eq!(media_radius(AttachmentSize::Sm, 8, 10), 10);
    }

    #[test]
    fn chip_pad_is_slot_conditional() {
        // N5: content pad / media-only pad / no pad.
        assert_eq!(
            chip_pad(AttachmentSize::Default, true, true),
            Vec2::new(10.0, 8.0)
        );
        assert_eq!(
            chip_pad(AttachmentSize::Default, false, true),
            Vec2::splat(8.0)
        );
        assert_eq!(chip_pad(AttachmentSize::Default, false, false), Vec2::ZERO);
        assert_eq!(chip_pad(AttachmentSize::Xs, false, true), Vec2::splat(4.0));
    }

    #[test]
    fn hover_fill_needs_direct_child_trigger() {
        // The hover fill applies only when the chip itself is a trigger.
        assert!(hover_fill_applies(true, true));
        assert!(!hover_fill_applies(true, false));
        assert!(!hover_fill_applies(false, true));
        assert!(!hover_fill_applies(false, false));
    }

    /// Renders into a fresh light-mode context; the `show` closure captures
    /// the response rects the test asserts on. Runs two frames: an `Area`
    /// (the vertical actions overlay) needs a sizing pass on frame 1 and
    /// only lands on its anchored position from frame 2, so the captured
    /// rects come from the second frame.
    fn render_once(width: f32, height: f32, mut show: impl FnMut(&mut Ui)) {
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        for _ in 0..2 {
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("attachment_s1_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(width, height))),
                );
                show(ui);
            });
            out.textures_delta.clear();
        }
    }

    #[test]
    fn vertical_actions_overlay_pins_right_edge() {
        // N2: two actions on a vertical default chip — the row's right edge
        // sits 12px from the chip's right edge (108px from the chip's left),
        // the actions keep their 4px gap, and the media is 98px wide.
        let mut captured: Option<(Rect, Rect, Rect, Rect)> = None;
        render_once(500.0, 700.0, |ui| {
            let mut a1 = Rect::NOTHING;
            let mut a2 = Rect::NOTHING;
            let mut media = Rect::NOTHING;
            let chip = Attachment::new()
                .orientation(AttachmentOrientation::Vertical)
                .show(ui, |ui| {
                    media = AttachmentMedia::new().icon(Icon::FileText).show(ui).rect;
                    AttachmentContent::new().show(ui, |ui| {
                        AttachmentTitle::new("clip.mp4").show(ui);
                    });
                    AttachmentActions::new().show(ui, |ui| {
                        a1 = AttachmentAction::new(Icon::RefreshCw).show(ui).rect;
                        a2 = AttachmentAction::new(Icon::X).show(ui).rect;
                    });
                })
                .rect;
            captured = Some((a1, a2, media, chip));
        });
        let (a1, a2, media, chip) = captured.expect("chip rendered");
        assert_eq!(media.width(), 98.0, "vertical media side {media:?}");
        assert!(
            (a2.right() - chip.left() - 108.0).abs() < 1.0,
            "overlay right edge: a2 {a2:?} chip {chip:?}"
        );
        assert!(
            (a2.left() - a1.right() - 4.0).abs() < 1.0,
            "overlay gap: a1 {a1:?} a2 {a2:?}"
        );
        // The row floats over the media, not below it.
        assert!(a1.top() >= media.top() && a1.bottom() <= media.bottom());
    }

    #[test]
    fn horizontal_actions_have_no_gap() {
        // N3: `gap-1` is vertical-overlay-only; the horizontal row has no
        // inter-action gap.
        let mut captured: Option<(Rect, Rect)> = None;
        render_once(500.0, 700.0, |ui| {
            let mut a1 = Rect::NOTHING;
            let mut a2 = Rect::NOTHING;
            Attachment::new().show(ui, |ui| {
                AttachmentMedia::new().icon(Icon::FileText).show(ui);
                AttachmentContent::new().show(ui, |ui| {
                    AttachmentTitle::new("file.pdf").show(ui);
                });
                AttachmentActions::new().show(ui, |ui| {
                    a1 = AttachmentAction::new(Icon::RefreshCw).show(ui).rect;
                    a2 = AttachmentAction::new(Icon::X).show(ui).rect;
                });
            });
            captured = Some((a1, a2));
        });
        let (a1, a2) = captured.expect("chip rendered");
        assert!(
            (a2.left() - a1.right()).abs() < 1.0,
            "horizontal gap: a1 {a1:?} a2 {a2:?}"
        );
    }

    #[test]
    fn chip_padding_is_slot_conditional() {
        // N5: media-only chips use the media pad (8px default), chips with
        // content use the content pad (10px default) — in both orientations.
        // Each chip renders in its own Ui: a horizontal chip's laid-out rect
        // spans the full available height (pre-existing), so stacking chips
        // in one Ui would push the later ones below the visible rect and
        // their media would early-return before recording `has_media`.
        fn pad_for(horizontal: bool, with_content: bool) -> f32 {
            let mut captured: Option<f32> = None;
            render_once(500.0, 700.0, |ui| {
                let mut m_left = 0.0;
                let chip = if horizontal {
                    Attachment::new().show(ui, |ui| {
                        m_left = AttachmentMedia::new()
                            .icon(Icon::FileText)
                            .show(ui)
                            .rect
                            .left();
                        if with_content {
                            AttachmentContent::new().show(ui, |ui| {
                                AttachmentTitle::new("file.pdf").show(ui);
                            });
                        }
                    })
                } else {
                    Attachment::new()
                        .orientation(AttachmentOrientation::Vertical)
                        .show(ui, |ui| {
                            m_left = AttachmentMedia::new()
                                .icon(Icon::FileText)
                                .show(ui)
                                .rect
                                .left();
                            if with_content {
                                AttachmentContent::new().show(ui, |ui| {
                                    AttachmentTitle::new("clip.mp4").show(ui);
                                });
                            }
                        })
                };
                captured = Some(m_left - chip.rect.left());
            });
            captured.expect("chip rendered")
        }
        assert!(
            (pad_for(true, false) - 8.0).abs() < 1.0,
            "horizontal media-only pad"
        );
        assert!(
            (pad_for(true, true) - 10.0).abs() < 1.0,
            "horizontal content pad"
        );
        assert!(
            (pad_for(false, false) - 8.0).abs() < 1.0,
            "vertical media-only pad"
        );
        assert!(
            (pad_for(false, true) - 10.0).abs() < 1.0,
            "vertical content pad"
        );
    }

    #[test]
    fn content_grows_past_240() {
        // N9: no 240px cap — a long title in a 500px-wide Ui grows the
        // content column past 240px.
        let mut captured: Option<f32> = None;
        render_once(500.0, 700.0, |ui| {
            let mut content_w = 0.0;
            Attachment::new().show(ui, |ui| {
                AttachmentMedia::new().icon(Icon::FileText).show(ui);
                let content = AttachmentContent::new().show(ui, |ui| {
                    AttachmentTitle::new("a".repeat(120)).show(ui);
                });
                content_w = content.response.rect.width();
            });
            captured = Some(content_w);
        });
        let w = captured.expect("chip rendered");
        assert!(
            w > 240.0,
            "content width {w} should exceed the removed 240px cap"
        );
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("attachment_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(500.0, 700.0))),
                );
                for state in [
                    AttachmentState::Idle,
                    AttachmentState::Uploading,
                    AttachmentState::Processing,
                    AttachmentState::Error,
                    AttachmentState::Done,
                ] {
                    Attachment::new().state(state).show(ui, |ui| {
                        AttachmentMedia::new().icon(Icon::FileText).show(ui);
                        AttachmentContent::new().show(ui, |ui| {
                            AttachmentTitle::new("file.pdf").show(ui);
                            AttachmentDescription::new("PDF · 2.4 MB").show(ui);
                        });
                        AttachmentActions::new().show(ui, |ui| {
                            AttachmentAction::new(Icon::X).show(ui);
                        });
                    });
                }
                // Vertical + spinner media + group.
                Attachment::new()
                    .orientation(AttachmentOrientation::Vertical)
                    .show(ui, |ui| {
                        AttachmentMedia::new().spinner(true).show(ui);
                        AttachmentContent::new().show(ui, |ui| {
                            AttachmentTitle::new("clip.mp4").show(ui);
                        });
                        AttachmentActions::new().show(ui, |ui| {
                            AttachmentAction::new(Icon::X).show(ui);
                        });
                    });
                // Image media (`opacity-60` while uploading) + size-aware
                // vertical chips (Sm/Xs).
                let tex = ctx.load_texture(
                    "attachment_test_img",
                    {
                        let mut pixels = Vec::new();
                        for y in 0..8u8 {
                            for x in 0..8u8 {
                                pixels.push(Color32::from_rgb(40 + x * 20, 60 + y * 20, 120));
                            }
                        }
                        egui::ColorImage::new([8, 8], pixels)
                    },
                    egui::TextureOptions::default(),
                );
                let img_src =
                    ImageSource::Texture(egui::load::SizedTexture::new(tex.id(), tex.size_vec2()));
                Attachment::new()
                    .state(AttachmentState::Uploading)
                    .show(ui, |ui| {
                        AttachmentMedia::new().image(img_src).show(ui);
                        AttachmentContent::new().show(ui, |ui| {
                            AttachmentTitle::new("photo.png").show(ui);
                        });
                    });
                for size in [AttachmentSize::Sm, AttachmentSize::Xs] {
                    Attachment::new()
                        .size(size)
                        .orientation(AttachmentOrientation::Vertical)
                        .show(ui, |ui| {
                            AttachmentMedia::new().icon(Icon::FileText).show(ui);
                            AttachmentContent::new().show(ui, |ui| {
                                AttachmentTitle::new("note.md").show(ui);
                            });
                            AttachmentActions::new().show(ui, |ui| {
                                AttachmentAction::new(Icon::X).show(ui);
                            });
                        });
                }
                AttachmentGroup::new().show(ui, |ui| {
                    Attachment::new().trigger(true).show(ui, |ui| {
                        AttachmentMedia::new().icon(Icon::Paperclip).show(ui);
                        AttachmentContent::new().show(ui, |ui| {
                            AttachmentTitle::new("a.txt").show(ui);
                        });
                    });
                });
            });
            out.textures_delta.clear();
        }
    }
}
