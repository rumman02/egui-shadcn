//! shadcn `textarea.tsx` — the multi-line text field.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/textarea.tsx`:
//! `min-h-16 rounded-md border border-input bg-transparent px-3 py-2 text-sm
//! shadow-xs` with the shared focus/invalid/disabled styles and the
//! `dark:bg-input/30` fill. The nova style (`.cn-textarea`) widens the
//! corners to `rounded-lg`, narrows the padding to `px-2.5` and adds the
//! disabled fills (`disabled:bg-input/50 dark:disabled:bg-input/80`); the
//! border fades to 50% like the base `disabled:opacity-50`. Body lines
//! render at Tailwind's `text-sm` 20px line height.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut bio = String::new();
//! Textarea::new(&mut bio)
//!     .placeholder("Tell us about yourself")
//!     .rows(4)
//!     .show(ui);
//! ```

use crate::theme::{color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use egui::{Color32, CornerRadius, FontId, Margin, Response, Stroke, TextEdit, Ui, Vec2};

/// `min-h-16` — the smallest textarea height (64px).
pub const TEXTAREA_MIN_HEIGHT: f32 = 64.0;

/// Horizontal padding (`px-3` base recipe; nova `.cn-textarea` `px-2.5`).
pub(crate) fn padding_x(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0, // px-2.5
        _ => 12.0,           // px-3
    }
}

/// Corner radius (`rounded-md` base recipe; nova `.cn-textarea` `rounded-lg`).
pub(crate) fn corner_radius(style: Style, radius: &RadiusTokens) -> u8 {
    match style {
        Style::Nova => radius.lg_u8(),
        _ => radius.md_u8(),
    }
}

/// Resolved frame colors for one textarea state (pure, unit-testable).
///
/// Identical recipe to [`crate::components::input`] (including the nova
/// disabled fills and the 50% disabled border fade).
pub(crate) fn resolve_textarea_colors(
    p: &Palette,
    dark: bool,
    focused: bool,
    invalid: bool,
    disabled: bool,
    style: Style,
) -> (Color32, Stroke, Color32) {
    let fill = if disabled {
        // nova `disabled:bg-input/50 dark:disabled:bg-input/80`. The base
        // recipe keeps its fill but `disabled:opacity-50` fades it.
        match style {
            Style::Nova => color_alpha(p.input, if dark { 0.8 } else { 0.5 }),
            _ => {
                let base = if dark {
                    color_alpha(p.input, 0.3)
                } else {
                    Color32::TRANSPARENT
                };
                color_alpha(base, 0.5)
            }
        }
    } else if dark {
        color_alpha(p.input, 0.3)
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
    // `disabled:opacity-50` fades the border with the rest of the element.
    let stroke_color = if disabled {
        color_alpha(stroke_color, 0.5)
    } else {
        stroke_color
    };
    let ring = if invalid {
        color_alpha(p.destructive, if dark { 0.4 } else { 0.2 })
    } else {
        color_alpha(p.ring, 0.5)
    };
    (fill, Stroke::new(1.0, stroke_color), ring)
}

/// A shadcn textarea. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Textarea<'a> {
    text: &'a mut String,
    placeholder: Option<String>,
    disabled: bool,
    invalid: bool,
    rows: usize,
    width: Option<f32>,
    id_salt: Option<String>,
}

impl<'a> Textarea<'a> {
    /// Creates a textarea bound to the given string.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            disabled: false,
            invalid: false,
            rows: 3,
            width: None,
            id_salt: None,
        }
    }

    /// Shows faint hint text when the field is empty
    /// (`placeholder:text-muted-foreground`).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Disables the field (`disabled:opacity-50 disabled:cursor-not-allowed`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Marks the field aria-invalid (`aria-invalid:border-destructive
    /// aria-invalid:ring-destructive/20`).
    pub fn invalid(mut self, invalid: bool) -> Self {
        self.invalid = invalid;
        self
    }

    /// Sets the visible row count (`field-sizing-content` grows beyond it).
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(1);
        self
    }

    /// Sets an explicit width. The default is `w-full` (the available width
    /// in vertical layouts; 200pt inside horizontal rows).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a stable id salt so state (focus, cursor) persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the textarea.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let style = theme.style;

        let width = match self.width {
            Some(w) => w,
            None => {
                if ui.layout().is_vertical() {
                    ui.available_width()
                } else {
                    200.0
                }
            }
        };

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_textarea"),
        };

        let is_empty = self.text.is_empty();
        // `disabled:opacity-50` fades the text too.
        let text_color = if self.disabled {
            color_alpha(p.foreground, 0.5)
        } else {
            p.foreground
        };
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        // `px-3 py-2` base recipe; nova `.cn-textarea` `px-2.5 py-2`.
        let margin = Margin::symmetric(padding_x(style) as i8, 8);
        let edit = TextEdit::multiline(self.text)
            .id_salt(id)
            .font(font.clone())
            .text_color(text_color)
            .desired_width(width)
            .desired_rows(self.rows)
            .min_size(Vec2::new(width, TEXTAREA_MIN_HEIGHT))
            .margin(margin);
        let edit = if let Some(placeholder) = self.placeholder.as_deref() {
            if is_empty {
                edit.hint_text(placeholder.to_owned())
            } else {
                edit
            }
        } else {
            edit
        };
        let edit = if self.disabled {
            edit.interactive(false)
        } else {
            edit
        };

        // Body lines at Tailwind `text-sm` 20px line height: widen the
        // layouter's row height inside a scoped Ui so no sibling is affected.
        let row_height = ui.fonts_mut(|f| f.row_height(&font));
        let extra = (20.0 - row_height).max(0.0);
        let out = ui.scope(|ui| {
            ui.spacing_mut().extra_text_line_spacing = extra;
            edit.show(ui)
        });
        let response = out.inner.response.response;
        let rect = response.rect;

        if ui.is_rect_visible(rect) {
            let focused = response.has_focus() && !self.disabled;
            let (fill, stroke, ring) =
                resolve_textarea_colors(&p, dark, focused, self.invalid, self.disabled, style);
            let corner = CornerRadius::same(corner_radius(style, &theme.radius));
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
            if fill != Color32::TRANSPARENT {
                painter.rect_filled(rect, corner, fill);
            }
            painter.rect_stroke(rect, corner, stroke, egui::epaint::StrokeKind::Inside);

            // `focus-visible:ring-[3px] focus-visible:ring-ring/50`
            if focused {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(painter, ring_rect, ring, 3.0, corner.nw as f32);
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    #[test]
    fn textarea_metrics_follow_style() {
        assert_eq!(padding_x(Style::Nova), 10.0); // px-2.5
        assert_eq!(padding_x(Style::Vega), 12.0); // px-3
    }

    #[test]
    fn textarea_is_at_least_64pt_tall() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from("line");
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("textarea_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            rect = Textarea::new(&mut text).show(ui).rect;
        });
        out.textures_delta.clear();
        assert!(
            rect.height() >= TEXTAREA_MIN_HEIGHT,
            "min-h-16: {}",
            rect.height()
        );
    }

    #[test]
    fn frame_colors_follow_state() {
        let p = palette(Mode::Light);
        let (fill, stroke, _) =
            resolve_textarea_colors(&p, false, false, false, false, Style::Vega);
        assert_eq!(fill, Color32::TRANSPARENT);
        assert_eq!(stroke.color, p.input);
        let (_, stroke, ring) = resolve_textarea_colors(&p, false, true, false, false, Style::Vega);
        assert_eq!(stroke.color, p.ring);
        assert_eq!(ring, color_alpha(p.ring, 0.5));
        let (_, stroke, ring) = resolve_textarea_colors(&p, false, false, true, false, Style::Vega);
        assert_eq!(stroke.color, p.destructive);
        assert_eq!(ring, color_alpha(p.destructive, 0.2));
        let (fill, _, _) =
            resolve_textarea_colors(&palette(Mode::Dark), true, false, false, false, Style::Vega);
        assert_eq!(fill, color_alpha(palette(Mode::Dark).input, 0.3));
    }

    #[test]
    fn disabled_fades_border_and_fills() {
        let p = palette(Mode::Light);
        // Nova light: `disabled:bg-input/50`, border at 50%.
        let nd = resolve_textarea_colors(&p, false, false, false, true, Style::Nova);
        assert_eq!(nd.0, color_alpha(p.input, 0.5));
        assert_eq!(nd.1.color, color_alpha(p.input, 0.5));
        // Nova dark: `dark:disabled:bg-input/80`.
        let dark = palette(Mode::Dark);
        let ndd = resolve_textarea_colors(&dark, true, false, false, true, Style::Nova);
        assert_eq!(ndd.0, color_alpha(dark.input, 0.8));
        // Base keeps its fill (transparent light) with the border fade.
        let bd = resolve_textarea_colors(&p, false, false, false, true, Style::Vega);
        assert_eq!(bd.0, Color32::TRANSPARENT);
        assert_eq!(bd.1.color, color_alpha(p.input, 0.5));
        let bdd = resolve_textarea_colors(&dark, true, false, false, true, Style::Vega);
        assert_eq!(bdd.0, color_alpha(color_alpha(dark.input, 0.3), 0.5));
    }

    #[test]
    fn builders_keep_fields() {
        let mut text = String::new();
        let ta = Textarea::new(&mut text)
            .placeholder("Bio")
            .disabled(true)
            .invalid(true)
            .rows(6)
            .width(320.0)
            .id_salt("bio");
        assert_eq!(ta.placeholder.as_deref(), Some("Bio"));
        assert!(ta.disabled);
        assert!(ta.invalid);
        assert_eq!(ta.rows, 6);
        assert_eq!(ta.width, Some(320.0));
        assert_eq!(ta.id_salt.as_deref(), Some("bio"));
        // rows() clamps to at least one row.
        let mut t2 = String::new();
        assert_eq!(Textarea::new(&mut t2).rows(0).rows, 1);
    }
}
