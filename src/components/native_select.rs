//! shadcn `native-select.tsx` — a styled `<select>` replacement.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/native-select.tsx`: an
//! `h-9 rounded-md border border-input bg-transparent px-3 pr-9 text-sm
//! shadow-xs` trigger (`data-[size=sm]:h-8`) with a chevron-down icon at
//! `right-3.5` (`text-muted-foreground opacity-50`), the shared
//! focus/invalid/disabled styles and `dark:bg-input/30 dark:hover:bg-input/50`.
//!
//! The browser's native dropdown is replaced by an [`egui::Popup`] anchored
//! below the trigger; option rows follow the popover look
//! (`bg-popover`/`rounded-md shadow-md`, `bg-accent` on the hovered or
//! selected row).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut theme = Some(0);
//! NativeSelect::new(&mut theme)
//!     .option("Light")
//!     .option("Dark")
//!     .option("System")
//!     .id_salt("theme")
//!     .show(ui);
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{Align2, Color32, CornerRadius, FontId, Rect, Response, Sense, Stroke, Ui, Vec2};

/// Trigger height per size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NativeSelectSize {
    /// `h-9` (36px).
    #[default]
    Default,
    /// `h-8` (32px).
    Sm,
}

impl NativeSelectSize {
    /// Trigger height in points.
    pub fn height(self) -> f32 {
        match self {
            NativeSelectSize::Default => crate::theme::tokens::spacing::CONTROL,
            NativeSelectSize::Sm => crate::theme::tokens::spacing::CONTROL_SM,
        }
    }
}

/// One dropdown option (`NativeSelectOption`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    /// The visible label.
    pub label: String,
    /// `disabled` — the option cannot be picked.
    pub disabled: bool,
}

impl SelectOption {
    /// Creates an enabled option.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
        }
    }

    /// Creates a disabled option.
    pub fn disabled(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: true,
        }
    }
}

/// Resolved trigger colors for one state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct TriggerColors {
    fill: Color32,
    stroke: Stroke,
    /// Focus ring color (alpha already applied).
    ring: Color32,
    text: Color32,
    chevron: Color32,
}

/// Mirrors the base recipe: `border-input bg-transparent px-3 text-sm`,
/// `dark:bg-input/30 dark:hover:bg-input/50`, focus/invalid styles, chevron
/// `text-muted-foreground opacity-50`.
pub(crate) fn resolve_trigger_colors(
    p: &Palette,
    dark: bool,
    focused: bool,
    invalid: bool,
    hovered: bool,
    disabled: bool,
    has_selection: bool,
) -> TriggerColors {
    let fill = if dark {
        if hovered && !disabled {
            color_alpha(p.input, 0.5)
        } else {
            color_alpha(p.input, 0.3)
        }
    } else {
        Color32::TRANSPARENT
    };
    let stroke_color = if invalid {
        p.destructive
    } else if focused {
        p.ring
    } else {
        p.input
    };
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    let text = if disabled || !has_selection {
        color_alpha(p.muted_foreground, if disabled { 0.5 } else { 1.0 })
    } else {
        p.foreground
    };
    TriggerColors {
        fill,
        stroke: Stroke::new(1.0, stroke_color),
        ring,
        text,
        chevron: color_alpha(p.muted_foreground, 0.5),
    }
}

/// A shadcn native select. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct NativeSelect<'a> {
    selected: &'a mut Option<usize>,
    options: Vec<SelectOption>,
    placeholder: Option<String>,
    size: NativeSelectSize,
    width: Option<f32>,
    disabled: bool,
    invalid: bool,
    id_salt: Option<String>,
}

impl<'a> NativeSelect<'a> {
    /// Creates a select bound to the selected index (`None` = no selection).
    pub fn new(selected: &'a mut Option<usize>) -> Self {
        Self {
            selected,
            options: Vec::new(),
            placeholder: None,
            size: NativeSelectSize::Default,
            width: None,
            disabled: false,
            invalid: false,
            id_salt: None,
        }
    }

    /// Adds an enabled option.
    pub fn option(mut self, label: impl Into<String>) -> Self {
        self.options.push(SelectOption::new(label));
        self
    }

    /// Adds a disabled option.
    pub fn option_disabled(mut self, label: impl Into<String>) -> Self {
        self.options.push(SelectOption::disabled(label));
        self
    }

    /// Adds several options at once.
    pub fn options(mut self, labels: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.options
            .extend(labels.into_iter().map(SelectOption::new));
        self
    }

    /// Shows faint text on the trigger when nothing is selected.
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Sets the trigger size (`data-size`).
    pub fn size(mut self, size: NativeSelectSize) -> Self {
        self.size = size;
        self
    }

    /// Sets an explicit trigger width. The default fits the widest option.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Disables the select (`has-[select:disabled]:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the select aria-invalid.
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets a stable id salt so popup open state and focus persist per
    /// instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// The trigger label for the current selection.
    fn label_for(&self, index: usize) -> String {
        self.options
            .get(index)
            .map(|o| o.label.clone())
            .unwrap_or_default()
    }

    /// Renders the select trigger and popup.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let height = self.size.height();

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_native_select"),
        };

        // Trigger width: widest option + px-3 + pr-9, at least 160.
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let widest = self
            .options
            .iter()
            .map(|o| {
                ui.painter()
                    .layout_no_wrap(o.label.clone(), font.clone(), Color32::WHITE)
                    .size()
                    .x
            })
            .fold(0.0f32, f32::max)
            .max(
                self.placeholder
                    .as_deref()
                    .map(|ph| {
                        ui.painter()
                            .layout_no_wrap(ph.to_owned(), font.clone(), Color32::WHITE)
                            .size()
                            .x
                    })
                    .unwrap_or(0.0),
            );
        let width = self
            .width
            .unwrap_or_else(|| (widest + 36.0 + 12.0).max(160.0));

        let sense = if self.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let (_, rect) = ui.allocate_space(Vec2::new(width, height));
        let response = ui.interact(rect, id, sense);

        if ui.is_rect_visible(rect) {
            let selected = *self.selected;
            let c = resolve_trigger_colors(
                &p,
                dark,
                response.has_focus(),
                self.invalid,
                response.hovered(),
                self.disabled,
                selected.is_some(),
            );
            let radius = theme.radius.md_u8();
            let corner = CornerRadius::same(radius);
            let painter = ui.painter();

            // `shadow-xs`
            if !self.disabled {
                let sh = crate::utils::shadow::shadow_xs(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        corner,
                        sh.color,
                    )
                    .with_blur_width(2.0),
                );
            }

            if c.fill != Color32::TRANSPARENT {
                painter.rect_filled(rect, corner, c.fill);
            }
            painter.rect_stroke(rect, corner, c.stroke, egui::epaint::StrokeKind::Inside);

            // Label: `px-3 ... pr-9 text-sm`, left aligned, vertically centered.
            let label = selected
                .map(|i| self.label_for(i))
                .unwrap_or_else(|| self.placeholder.clone().unwrap_or_default());
            let text_pos = egui::Pos2::new(
                rect.min.x + crate::theme::tokens::spacing::MD,
                rect.center().y,
            );
            painter.text(text_pos, Align2::LEFT_CENTER, label, font, c.text);

            // Chevron: `right-3.5 size-4 text-muted-foreground opacity-50`.
            let chevron_center = egui::Pos2::new(
                rect.max.x - 14.0 - 8.0, // right-3.5 = 14px, half of size-4 = 8px
                rect.center().y,
            );
            let chevron_rect = Rect::from_center_size(
                chevron_center,
                Vec2::splat(crate::theme::tokens::spacing::LG),
            );
            let chevron_color = if self.disabled {
                color_alpha(c.chevron, 0.5)
            } else {
                c.chevron
            };
            Icon::ChevronDown.paint(
                painter,
                chevron_rect,
                chevron_color,
                2.0 * crate::theme::tokens::spacing::LG / 24.0,
            );

            // `focus-visible:ring-[3px] focus-visible:ring-ring/50`
            if response.has_focus() && !self.disabled {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(painter, ring_rect, c.ring, 3.0, radius as f32);
            }
        }

        // The anchored options popup (replaces the browser's native list).
        if !self.disabled {
            let popup = egui::Popup::from_toggle_button_response(&response)
                .close_behavior(egui::PopupCloseBehavior::CloseOnClick)
                .frame(
                    egui::Frame::default()
                        .fill(p.popover)
                        .stroke(Stroke::new(1.0, p.border))
                        .corner_radius(theme.radius.md_u8())
                        .inner_margin(egui::Margin::same(crate::theme::tokens::spacing::XS as i8))
                        .shadow(crate::utils::shadow::shadow_md(dark)),
                )
                .width(rect.width());
            let mut slf = self;
            popup.show(|ui| {
                for i in 0..slf.options.len() {
                    let option = slf.options[i].clone();
                    slf.render_option_row(ui, i, &option, &p);
                }
            });
        }

        response
    }

    fn render_option_row(&mut self, ui: &mut Ui, index: usize, option: &SelectOption, p: &Palette) {
        let selected = *self.selected == Some(index);
        let row_height = crate::theme::tokens::spacing::CONTROL;
        let (_, row_rect) = ui.allocate_space(Vec2::new(ui.available_width(), row_height));
        let sense = if option.disabled {
            Sense::hover()
        } else {
            Sense::click()
        };
        let row_id = ui.scope_id().with("row").with(index);
        let response = ui.interact(row_rect, row_id, sense);

        let hovered = response.hovered() && !option.disabled;
        let fill = if selected || hovered {
            p.accent
        } else {
            Color32::TRANSPARENT
        };
        let fg = if selected || hovered {
            p.accent_foreground
        } else {
            p.popover_foreground
        };
        let (fill, fg) = if option.disabled {
            (color_alpha(fill, 0.5), color_alpha(fg, 0.5))
        } else {
            (fill, fg)
        };

        if ui.is_rect_visible(row_rect) {
            let painter = ui.painter();
            if fill != Color32::TRANSPARENT {
                painter.rect_filled(row_rect, CornerRadius::same(theme_radius_sm(ui)), fill);
            }
            let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
            painter.text(
                egui::Pos2::new(
                    row_rect.min.x + crate::theme::tokens::spacing::SM,
                    row_rect.center().y,
                ),
                Align2::LEFT_CENTER,
                option.label.clone(),
                font,
                fg,
            );
        }

        if response.clicked() && !option.disabled {
            *self.selected = Some(index);
        }
    }
}

fn theme_radius_sm(ui: &Ui) -> u8 {
    (ShadcnTheme::of_ui(ui).radius.sm().round() as u8).max(2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::Pos2;

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn trigger_colors_follow_state() {
        let p = palette(Mode::Light);
        let c = resolve_trigger_colors(&p, false, false, false, false, false, false);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.stroke.color, p.input);
        assert_eq!(c.text, p.muted_foreground); // placeholder look
        let sel = resolve_trigger_colors(&p, false, false, false, false, false, true);
        assert_eq!(sel.text, p.foreground);
        let focused = resolve_trigger_colors(&p, false, true, false, false, false, true);
        assert_eq!(focused.stroke.color, p.ring);
        assert_eq!(focused.ring, color_alpha(p.ring, 0.5));
        let invalid = resolve_trigger_colors(&p, false, false, true, false, false, true);
        assert_eq!(invalid.stroke.color, p.destructive);
        assert_eq!(invalid.ring, color_alpha(p.destructive, 0.2));
        let disabled = resolve_trigger_colors(&p, false, false, false, false, true, true);
        assert_eq!(disabled.text, color_alpha(p.muted_foreground, 0.5));
    }

    #[test]
    fn dark_mode_hover_uses_input_50() {
        let dark = palette(Mode::Dark);
        let c = resolve_trigger_colors(&dark, true, false, false, true, false, true);
        assert_eq!(c.fill, color_alpha(dark.input, 0.5));
        let resting = resolve_trigger_colors(&dark, true, false, false, false, false, true);
        assert_eq!(resting.fill, color_alpha(dark.input, 0.3));
    }

    #[test]
    fn size_heights() {
        assert_eq!(NativeSelectSize::Default.height(), 36.0);
        assert_eq!(NativeSelectSize::Sm.height(), 32.0);
    }

    #[test]
    fn builders_keep_fields() {
        let mut selected = None;
        let s = NativeSelect::new(&mut selected)
            .option("Light")
            .option_disabled("Dark")
            .options(["System"])
            .placeholder("Pick a theme")
            .size(NativeSelectSize::Sm)
            .width(240.0)
            .disabled(true)
            .invalid(true)
            .id_salt("theme");
        assert_eq!(s.options.len(), 3);
        assert!(s.options[1].disabled);
        assert_eq!(s.placeholder.as_deref(), Some("Pick a theme"));
        assert_eq!(s.size, NativeSelectSize::Sm);
        assert_eq!(s.width, Some(240.0));
        assert!(s.disabled);
        assert!(s.invalid);
        assert_eq!(s.id_salt.as_deref(), Some("theme"));
    }

    #[test]
    fn trigger_is_36pt_tall() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut selected = Some(0);
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("native_select_size_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            rect = NativeSelect::new(&mut selected)
                .options(["A", "B"])
                .show(ui)
                .rect;
        });
        out.textures_delta.clear();
        assert_eq!(rect.height(), NativeSelectSize::Default.height());
    }
}
