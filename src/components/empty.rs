//! shadcn `empty.tsx` — a placeholder for empty states.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/empty.tsx`: a centered
//! column (`p-6`, gap-6, `text-center`) with `border-dashed` (visible when
//! [`Empty::bordered`] is on — the source's default has no border width, the
//! docs pages add `border`), an [`EmptyHeader`] holding an [`EmptyMedia`],
//! [`EmptyTitle`] and [`EmptyDescription`], and an [`EmptyContent`] slot for
//! actions. The nova style tightens the gap to `gap-4`, the media icon box
//! to `size-8` with a `size-4` icon and the title to `text-sm`.
//!
//! Scope notes: the source pads to 48px at the `md` breakpoint
//! (`md:p-12`); the port uses the fixed nova `p-6` = 24px. [`EmptyFooter`]
//! is an invented extension (see its docs).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Empty::new().bordered(true).show(ui, |ui| {
//!     EmptyHeader::new().show(ui, |ui| {
//!         EmptyMedia::icon(Icon::Search).show(ui);
//!         EmptyTitle::new("No results found").show(ui);
//!         EmptyDescription::new("Try a different search term.").show(ui);
//!     });
//!     EmptyContent::new().show(ui, |ui| {
//!         Button::new("Clear filters").variant(ButtonVariant::Outline).show(ui);
//!     });
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{ShadcnTheme, Style};
use egui::{Align, Color32, FontId, InnerResponse, Margin, Pos2, Rect, Shape, Ui, Vec2};

/// Media variants, mirroring the source `emptyMediaVariants`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum EmptyMediaVariant {
    /// `bg-transparent` — free-form media (images etc.).
    #[default]
    Default,
    /// `size-10 rounded-lg bg-muted text-foreground` icon box (nova: size-8).
    Icon,
}

/// The empty container's vertical gap (base `gap-6`, nova `gap-4`).
pub(crate) fn gap(style: Style) -> f32 {
    match style {
        Style::Nova => 16.0,
        _ => 24.0,
    }
}

/// The icon media box side (base `size-10`, nova `size-8`).
pub(crate) fn media_icon_box(style: Style) -> f32 {
    match style {
        Style::Nova => 32.0,
        _ => 40.0,
    }
}

/// The icon media glyph size (base `size-6`, nova `size-4`).
pub(crate) fn media_icon_size(style: Style) -> f32 {
    match style {
        Style::Nova => 16.0,
        _ => 24.0,
    }
}

/// The title size (base `text-lg`, nova `text-sm`).
pub(crate) fn title_size(style: Style) -> f32 {
    match style {
        Style::Nova => crate::theme::tokens::type_scale::SM,
        _ => crate::theme::tokens::type_scale::LG,
    }
}

/// `max-w-sm` (24rem = 384px), shared by header/description/content.
pub const EMPTY_MAX_WIDTH: f32 = 384.0;

/// Lays out centered text (single- or multi-line) within the ui.
fn centered_text(
    ui: &mut Ui,
    text: &str,
    size: f32,
    color: Color32,
    max_width: f32,
    line_height: Option<f32>,
) -> egui::Response {
    let width = max_width.min(ui.available_width());
    let mut job = egui::text::LayoutJob::default();
    job.append(
        text,
        0.0,
        egui::TextFormat {
            font_id: FontId::proportional(size),
            color,
            line_height,
            ..egui::TextFormat::default()
        },
    );
    job.halign = Align::Center;
    job.wrap.max_width = width;
    let galley = ui.painter().layout_job(job);
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), galley.size().y),
        egui::Sense::hover(),
    );
    if ui.is_rect_visible(rect) {
        let x = rect.left() + (rect.width() - galley.size().x) * 0.5;
        ui.painter().galley(Pos2::new(x, rect.min.y), galley, color);
    }
    response
}

/// A shadcn empty state container. See the [module docs](self) for an example.
#[derive(Clone, Debug, Default)]
pub struct Empty {
    bordered: bool,
    fill_height: bool,
    id_salt: Option<String>,
}

impl Empty {
    /// Creates an empty state container.
    pub fn new() -> Self {
        Self {
            bordered: false,
            fill_height: false,
            id_salt: None,
        }
    }

    /// Shows the dashed border (`border border-dashed` in the docs markup;
    /// the base component only sets the dashed style).
    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    /// Stretches to the available height (`flex-1 justify-center`).
    pub fn fill_height(mut self, fill_height: bool) -> Self {
        self.fill_height = fill_height;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the empty state around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let g = gap(theme.style);
        let radius = if theme.style == Style::Nova {
            theme.radius.xl_u8()
        } else {
            theme.radius.lg_u8()
        };

        let painter = ui.painter().clone();
        let border_idx = painter.add(Shape::Noop);

        let inner = egui::Frame::new()
            .inner_margin(Margin::same(24))
            .show(ui, |ui| {
                if self.fill_height {
                    ui.set_min_height(ui.available_height());
                }
                ui.set_min_width(ui.available_width());
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = g;
                    content(ui)
                })
                .inner
            });

        if self.bordered {
            // Dashed border approximation: four dashed segments between the
            // rounded corners.
            let rect = inner.response.rect;
            let r = radius as f32 * 0.5;
            let stroke = egui::Stroke::new(1.0, p.border);
            let (d, gpat) = (6.0, 4.0);
            let mut shapes = Vec::new();
            Shape::dashed_line(
                &[
                    Pos2::new(rect.left() + r, rect.top()),
                    Pos2::new(rect.right() - r, rect.top()),
                ],
                stroke,
                d,
                gpat,
            )
            .into_iter()
            .for_each(|s| shapes.push(s));
            Shape::dashed_line(
                &[
                    Pos2::new(rect.right(), rect.top() + r),
                    Pos2::new(rect.right(), rect.bottom() - r),
                ],
                stroke,
                d,
                gpat,
            )
            .into_iter()
            .for_each(|s| shapes.push(s));
            Shape::dashed_line(
                &[
                    Pos2::new(rect.right() - r, rect.bottom()),
                    Pos2::new(rect.left() + r, rect.bottom()),
                ],
                stroke,
                d,
                gpat,
            )
            .into_iter()
            .for_each(|s| shapes.push(s));
            Shape::dashed_line(
                &[
                    Pos2::new(rect.left(), rect.bottom() - r),
                    Pos2::new(rect.left(), rect.top() + r),
                ],
                stroke,
                d,
                gpat,
            )
            .into_iter()
            .for_each(|s| shapes.push(s));
            painter.set(border_idx, Shape::Vec(shapes));
        } else {
            painter.set(border_idx, Shape::Noop);
        }

        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `EmptyHeader` — centered column (`gap-2 max-w-sm`).
#[derive(Clone, Debug, Default)]
pub struct EmptyHeader {
    id_salt: Option<String>,
}

impl EmptyHeader {
    /// Creates an empty header.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the header.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let inner = ui.allocate_ui(
            Vec2::new(EMPTY_MAX_WIDTH.min(ui.available_width()), 0.0),
            |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = 8.0; // gap-2
                    content(ui)
                })
                .inner
            },
        );
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `EmptyMedia` — the media slot; [`EmptyMediaVariant::Icon`] draws a
/// `bg-muted` rounded box around the icon.
#[derive(Clone, Debug)]
pub struct EmptyMedia {
    variant: EmptyMediaVariant,
    icon: Option<Icon>,
    id_salt: Option<String>,
}

impl Default for EmptyMedia {
    fn default() -> Self {
        Self::new()
    }
}

impl EmptyMedia {
    /// Creates free-form media (paint your own content in [`Self::show`]).
    pub fn new() -> Self {
        Self {
            variant: EmptyMediaVariant::Default,
            icon: None,
            id_salt: None,
        }
    }

    /// Creates the icon media variant with the given lucide icon.
    pub fn icon(icon: Icon) -> Self {
        Self {
            variant: EmptyMediaVariant::Icon,
            icon: Some(icon),
            id_salt: None,
        }
    }

    /// Overrides the media variant.
    pub fn variant(mut self, variant: EmptyMediaVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the media slot (the icon box for the icon variant).
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let response = if self.variant == EmptyMediaVariant::Icon {
            let box_size = media_icon_box(theme.style);
            let (rect, response) =
                ui.allocate_exact_size(Vec2::splat(box_size), egui::Sense::hover());
            if ui.is_rect_visible(rect) {
                let painter = ui.painter();
                painter.rect_filled(rect, theme.radius.lg_u8(), p.muted);
                if let Some(icon) = self.icon {
                    let icon_size = media_icon_size(theme.style);
                    let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
                    icon.paint(painter, icon_rect, p.foreground, 2.0 * icon_size / 24.0);
                }
            }
            response
        } else {
            // Free-form media has no intrinsic size; paint it yourself with
            // [`Self::show_with`].
            ui.allocate_exact_size(Vec2::ZERO, egui::Sense::hover()).1
        };
        // `mb-2` *below* the media (the source class is on the media div).
        ui.add_space(8.0);
        response
    }

    /// Renders free-form media content (images, custom glyphs) centered.
    pub fn show_with<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let inner = ui.allocate_ui(
            Vec2::new(EMPTY_MAX_WIDTH.min(ui.available_width()), 0.0),
            |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), content)
                    .inner
            },
        );
        // `mb-2` below the media.
        ui.add_space(8.0);
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `EmptyTitle` — `text-lg font-medium tracking-tight` (nova `text-sm`).
#[derive(Clone, Debug)]
pub struct EmptyTitle {
    text: String,
}

impl EmptyTitle {
    /// Creates an empty title.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the title.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        centered_text(
            ui,
            &self.text,
            title_size(theme.style),
            p.foreground,
            EMPTY_MAX_WIDTH,
            None,
        )
    }
}

/// `EmptyDescription` — `text-sm/relaxed text-muted-foreground` (14px text,
/// 20px line height).
#[derive(Clone, Debug)]
pub struct EmptyDescription {
    text: String,
}

impl EmptyDescription {
    /// Creates an empty description.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }

    /// Renders the description.
    pub fn show(self, ui: &mut Ui) -> egui::Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        centered_text(
            ui,
            &self.text,
            crate::theme::tokens::type_scale::SM,
            p.muted_foreground,
            EMPTY_MAX_WIDTH,
            Some(crate::components::typography::line_heights::SMALL),
        )
    }
}

/// `EmptyContent` — the actions slot (`gap-4 max-w-sm`; nova `gap-2.5`).
#[derive(Clone, Debug, Default)]
pub struct EmptyContent {
    id_salt: Option<String>,
}

impl EmptyContent {
    /// Creates an empty content slot.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the content slot.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let g = match theme.style {
            Style::Nova => 10.0, // gap-2.5
            _ => 16.0,           // gap-4
        };
        let inner = ui.allocate_ui(
            Vec2::new(EMPTY_MAX_WIDTH.min(ui.available_width()), 0.0),
            |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = g;
                    content(ui)
                })
                .inner
            },
        );
        InnerResponse::new(inner.inner, inner.response)
    }
}

/// `EmptyFooter` — bottom slot mirroring EmptyContent. This slot is a Rust
/// port extension: `empty.tsx` exports no footer (its docs pages wrap
/// actions in the content slot), so prefer [`EmptyContent`] for new code.
#[derive(Clone, Debug, Default)]
pub struct EmptyFooter {
    id_salt: Option<String>,
}

impl EmptyFooter {
    /// Creates an empty footer.
    pub fn new() -> Self {
        Self { id_salt: None }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the footer.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let inner = ui.allocate_ui(
            Vec2::new(EMPTY_MAX_WIDTH.min(ui.available_width()), 0.0),
            |ui| {
                ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    ui.spacing_mut().item_spacing.y = 10.0;
                    content(ui)
                })
                .inner
            },
        );
        InnerResponse::new(inner.inner, inner.response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    #[test]
    fn geometry_matches_style_recipes() {
        // base gap-6 / p-6 md:p-12; nova gap-4 / p-6.
        assert_eq!(gap(Style::Vega), 24.0);
        assert_eq!(gap(Style::Nova), 16.0);
        // media icon size-10 vs nova size-8, glyph size-6 vs size-4.
        assert_eq!(media_icon_box(Style::Vega), 40.0);
        assert_eq!(media_icon_box(Style::Nova), 32.0);
        assert_eq!(media_icon_size(Style::Vega), 24.0);
        assert_eq!(media_icon_size(Style::Nova), 16.0);
        // title text-lg vs nova text-sm.
        assert_eq!(title_size(Style::Vega), 18.0);
        assert_eq!(title_size(Style::Nova), 14.0);
        // max-w-sm
        assert_eq!(EMPTY_MAX_WIDTH, 384.0);
    }

    #[test]
    fn renders_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("empty_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 300.0))),
                );
                Empty::new().bordered(true).show(ui, |ui| {
                    EmptyHeader::new().show(ui, |ui| {
                        EmptyMedia::icon(Icon::Search).show(ui);
                        EmptyTitle::new("No results found").show(ui);
                        EmptyDescription::new("Try a different search term or clear the filters.")
                            .show(ui);
                    });
                    EmptyContent::new().show(ui, |ui| {
                        ui.label("actions");
                    });
                });
                Empty::new().show(ui, |ui| {
                    EmptyMedia::new().show(ui);
                    EmptyTitle::new("Bare empty").show(ui);
                });
            });
            out.textures_delta.clear();
        }
    }
}
