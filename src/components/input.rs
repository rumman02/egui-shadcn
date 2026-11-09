//! shadcn `input.tsx` — the single-line text field.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/input.tsx`:
//! `h-9 rounded-md border border-input bg-transparent px-3 text-sm shadow-xs`
//! with `focus-visible:border-ring focus-visible:ring-[3px]
//! focus-visible:ring-ring/50`, `aria-invalid` destructive styling and
//! `disabled:opacity-50`. Dark mode uses the `dark:bg-input/30` fill.
//!
//! The nova style (`.cn-input`) tightens the geometry to
//! `h-8 rounded-lg px-2.5` and adds disabled fills
//! (`disabled:bg-input/50 dark:disabled:bg-input/80`); the border fades to
//! 50% like the base `disabled:opacity-50`.
//!
//! The underlying widget is a restyled [`egui::TextEdit`]; the shadcn frame
//! (border, shadow, focus ring) is drawn by this component. Hint text
//! renders in `muted-foreground` via the theme's weak text color
//! (`placeholder:text-muted-foreground`).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut email = String::new();
//! let response = Input::new(&mut email)
//!     .placeholder("m@example.com")
//!     .show(ui);
//! if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
//!     // submit
//! }
//! ```

use crate::components::button::group;
use crate::components::button::{fuse_radius, GroupOrientation, GroupPosition};
use crate::theme::{color_alpha, Palette, RadiusTokens, ShadcnTheme, Style};
use egui::{
    Align2, Color32, CornerRadius, FontId, Margin, Rect, Response, Stroke, TextEdit, Ui, Vec2,
};

/// Height of the input in the base recipe (`h-9`). The nova style is 32pt
/// (`h-8`).
pub const INPUT_HEIGHT: f32 = crate::theme::tokens::spacing::CONTROL;

/// Outer height (`h-9` base recipe; nova `.cn-input` `h-8`).
pub(crate) fn input_height(style: Style) -> f32 {
    match style {
        Style::Nova => 32.0, // h-8
        _ => INPUT_HEIGHT,   // h-9
    }
}

/// Horizontal padding (`px-3` base recipe; nova `.cn-input` `px-2.5`).
pub(crate) fn padding_x(style: Style) -> f32 {
    match style {
        Style::Nova => 10.0, // px-2.5
        _ => 12.0,           // px-3
    }
}

/// Corner radius (`rounded-md` base recipe; nova `.cn-input` `rounded-lg`).
pub(crate) fn corner_radius(style: Style, radius: &RadiusTokens) -> u8 {
    match style {
        Style::Nova => radius.lg_u8(),
        _ => radius.md_u8(),
    }
}

/// Resolved frame colors for one input state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct InputColors {
    fill: Color32,
    stroke: Stroke,
    /// Focus ring color (alpha already applied).
    ring: Color32,
}

/// Resolves the frame colors for the current palette and state.
///
/// Mirrors the base recipe (`border-input`, `dark:bg-input/30`,
/// `focus-visible:border-ring focus-visible:ring-ring/50`,
/// `aria-invalid:border-destructive aria-invalid:ring-destructive/20
/// dark:aria-invalid:ring-destructive/40`, `disabled:opacity-50`). The nova
/// style replaces the disabled fill with `disabled:bg-input/50
/// dark:disabled:bg-input/80`.
pub(crate) fn resolve_input_colors(
    p: &Palette,
    dark: bool,
    focused: bool,
    invalid: bool,
    disabled: bool,
    style: Style,
) -> InputColors {
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
    InputColors {
        fill,
        stroke: Stroke::new(1.0, stroke_color),
        ring,
    }
}

/// A shadcn text input. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Input<'a> {
    text: &'a mut String,
    placeholder: Option<String>,
    disabled: bool,
    invalid: bool,
    password: bool,
    width: Option<f32>,
    char_limit: Option<usize>,
    id_salt: Option<String>,
}

impl<'a> Input<'a> {
    /// Creates an input bound to the given string.
    pub fn new(text: &'a mut String) -> Self {
        Self {
            text,
            placeholder: None,
            disabled: false,
            invalid: false,
            password: false,
            width: None,
            char_limit: None,
            id_salt: None,
        }
    }

    /// Shows faint hint text when the field is empty
    /// (`placeholder:text-muted-foreground`).
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Disables the field (`disabled:opacity-50 disabled:pointer-events-none`).
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

    /// Masks the text (`type="password"`).
    pub fn password(mut self, password: bool) -> Self {
        self.password = password;
        self
    }

    /// Sets an explicit width. The default is `w-full` (the available width
    /// in vertical layouts; 160pt inside horizontal rows).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Limits the number of characters.
    pub fn char_limit(mut self, limit: usize) -> Self {
        self.char_limit = Some(limit);
        self
    }

    /// Sets a stable id salt so state (focus, cursor) persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Renders the input.
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
                    160.0
                }
            }
        };

        // Auto-detected position inside a ButtonGroup/InputGroup-like group.
        let group = group::next_child_position(ui);
        let position = group.map(|(_, pos)| pos).unwrap_or(GroupPosition::Only);
        let orientation = group
            .map(|(o, _)| o)
            .unwrap_or(GroupOrientation::Horizontal);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_input"),
        };

        let is_empty = self.text.is_empty();
        // `disabled:opacity-50` fades the text too.
        let text_color = if self.disabled {
            color_alpha(p.foreground, 0.5)
        } else {
            p.foreground
        };
        let font = FontId::proportional(crate::theme::tokens::type_scale::SM);
        let edit = TextEdit::singleline(self.text)
            .id_salt(id)
            .font(font)
            .text_color(text_color)
            .desired_width(width)
            .min_size(Vec2::new(width, input_height(style)))
            .align(Align2::LEFT_CENTER)
            .margin(Margin::symmetric(padding_x(style) as i8, 0))
            .password(self.password);
        let edit = if let Some(placeholder) = self.placeholder.as_deref() {
            if is_empty {
                edit.hint_text(placeholder.to_owned())
            } else {
                edit
            }
        } else {
            edit
        };
        let edit = if let Some(limit) = self.char_limit {
            edit.char_limit(limit)
        } else {
            edit
        };
        let edit = if self.disabled {
            edit.interactive(false)
        } else {
            edit
        };

        // Colors depend on focus, so resolve after the TextEdit ran. The
        // scoped spacing keeps the theme's `interact_size` floor (36pt) from
        // overriding the nova `h-8` (32pt) geometry.
        let out = ui.scope(|ui| {
            ui.spacing_mut().interact_size.y = input_height(style);
            edit.show(ui)
        });
        let response = out.inner.response.response;
        let rect = response.rect;

        if ui.is_rect_visible(rect) {
            let focused = response.has_focus() && !self.disabled;
            let c = resolve_input_colors(&p, dark, focused, self.invalid, self.disabled, style);
            let radius = corner_radius(style, &theme.radius);
            let corner = fuse_radius(CornerRadius::same(radius), orientation, position);

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

            // Non-first children of a group overlap the previous border so the
            // shared edge shows one 1px divider (`border-l-0`).
            let draw_rect = match orientation {
                GroupOrientation::Horizontal
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(rect.min + egui::vec2(1.0, 0.0), rect.max)
                }
                GroupOrientation::Vertical
                    if matches!(position, GroupPosition::Middle | GroupPosition::Last) =>
                {
                    Rect::from_min_max(rect.min + egui::vec2(0.0, 1.0), rect.max)
                }
                _ => rect,
            };

            if c.fill != Color32::TRANSPARENT {
                painter.rect_filled(draw_rect, corner, c.fill);
            }
            painter.rect_stroke(
                draw_rect,
                corner,
                c.stroke,
                egui::epaint::StrokeKind::Inside,
            );

            // `focus-visible:ring-[3px] focus-visible:ring-ring/50`
            if focused {
                let ring_rect = rect.expand2(egui::vec2(1.0, 1.0));
                crate::utils::focus::paint_ring(painter, ring_rect, c.ring, 3.0, radius as f32);
            }
        }

        response
    }
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
    fn input_height_follows_style() {
        // Base recipe `h-9`; nova `.cn-input` `h-8`.
        assert_eq!(input_height(Style::Vega), 36.0);
        assert_eq!(input_height(Style::Nova), 32.0);
        assert_eq!(INPUT_HEIGHT, 36.0);
        assert_eq!(padding_x(Style::Nova), 10.0);
        assert_eq!(padding_x(Style::Vega), 12.0);
    }

    #[test]
    fn input_is_control_height_and_text_is_vertically_centered() {
        // The default preset is nova: `h-8` (32pt).
        let (rect_h, rect_w) = {
            let ctx = egui::Context::default();
            Preset::default().install(&ctx);
            let mut text = String::from("hello");
            let mut h = 0.0;
            let mut w = 0.0;
            let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("input_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
                );
                let r = Input::new(&mut text).show(ui);
                h = r.rect.height();
                w = r.rect.width();
            });
            out.textures_delta.clear();
            (h, w)
        };
        assert!((rect_h - 32.0).abs() < 1.0, "nova h-8: {}", rect_h);
        assert!((rect_w - 400.0).abs() < 1.0, "w-full: {}", rect_w);
        // A non-nova (base recipe) theme keeps `h-9`.
        let ctx = egui::Context::default();
        Preset::default().with_style(Style::Vega).install(&ctx);
        let mut text = String::new();
        let mut base_h = 0.0;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("input_test_base"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            base_h = Input::new(&mut text).show(ui).rect.height();
        });
        out.textures_delta.clear();
        assert!((base_h - INPUT_HEIGHT).abs() < 1.0, "base h-9: {}", base_h);
    }

    #[test]
    fn frame_colors_follow_state() {
        let p = palette(Mode::Light);
        let c = resolve_input_colors(&p, false, false, false, false, Style::Vega);
        assert_eq!(c.fill, Color32::TRANSPARENT);
        assert_eq!(c.stroke.color, p.input);
        let f = resolve_input_colors(&p, false, true, false, false, Style::Vega);
        assert_eq!(f.stroke.color, p.ring);
        assert_eq!(f.ring, color_alpha(p.ring, 0.5));
        let i = resolve_input_colors(&p, false, false, true, false, Style::Vega);
        assert_eq!(i.stroke.color, p.destructive);
        assert_eq!(i.ring, color_alpha(p.destructive, 0.2));
        let dark = palette(Mode::Dark);
        let dc = resolve_input_colors(&dark, true, false, false, false, Style::Vega);
        assert_eq!(dc.fill, color_alpha(dark.input, 0.3));
        let di = resolve_input_colors(&dark, true, false, true, false, Style::Vega);
        assert_eq!(di.ring, color_alpha(dark.destructive, 0.4));
    }

    #[test]
    fn disabled_fades_border_and_fills() {
        let p = palette(Mode::Light);
        // Nova light: `disabled:bg-input/50`, border at 50%.
        let nd = resolve_input_colors(&p, false, false, false, true, Style::Nova);
        assert_eq!(nd.fill, color_alpha(p.input, 0.5));
        assert_eq!(nd.stroke.color, color_alpha(p.input, 0.5));
        // Nova dark: `dark:disabled:bg-input/80`.
        let dark = palette(Mode::Dark);
        let ndd = resolve_input_colors(&dark, true, false, false, true, Style::Nova);
        assert_eq!(ndd.fill, color_alpha(dark.input, 0.8));
        assert_eq!(ndd.stroke.color, color_alpha(dark.input, 0.5));
        // Base light keeps the transparent fill; base dark fades input/30.
        let bd = resolve_input_colors(&p, false, false, false, true, Style::Vega);
        assert_eq!(bd.fill, Color32::TRANSPARENT);
        assert_eq!(bd.stroke.color, color_alpha(p.input, 0.5));
        let bdd = resolve_input_colors(&dark, true, false, false, true, Style::Vega);
        assert_eq!(bdd.fill, color_alpha(color_alpha(dark.input, 0.3), 0.5));
    }

    #[test]
    fn builders_keep_fields() {
        let mut text = String::new();
        let input = Input::new(&mut text)
            .placeholder("Email")
            .disabled(true)
            .invalid(true)
            .password(true)
            .width(240.0)
            .char_limit(10)
            .id_salt("email");
        assert_eq!(input.placeholder.as_deref(), Some("Email"));
        assert!(input.disabled);
        assert!(input.invalid);
        assert!(input.password);
        assert_eq!(input.width, Some(240.0));
        assert_eq!(input.char_limit, Some(10));
        assert_eq!(input.id_salt.as_deref(), Some("email"));
    }

    #[test]
    fn frame_placeholder_hint_is_set() {
        // The hint renders only while the text is empty.
        let mut text = String::new();
        let input = Input::new(&mut text).placeholder("Email");
        assert!(input.placeholder.is_some());
    }

    #[test]
    fn default_senses_are_hover_only_when_disabled() {
        // Disabled inputs must not react to clicks.
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let mut text = String::from("abc");
        let mut focused = false;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("input_disabled_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(400.0, 400.0))),
            );
            focused = Input::new(&mut text).disabled(true).show(ui).has_focus();
        });
        out.textures_delta.clear();
        assert!(!focused);
    }
}
