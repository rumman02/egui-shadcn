//! shadcn `slider.tsx` — a draggable value selector.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/slider.tsx`: a `h-1.5
//! rounded-full bg-muted` track with a `bg-primary` range fill and `size-4
//! rounded-full border border-primary bg-white shadow-sm` thumbs
//! (`hover:ring-4 focus-visible:ring-4 ring-ring/50`,
//! `data-[disabled]:opacity-50`). The nova style renders a smaller `size-3`
//! thumb with a `border-ring` border, a thinner `h-1` track and a `ring-3`
//! halo; the base keeps the source geometry.
//!
//! Both a single value and a two-value range are supported (the source
//! renders one thumb per value), in a horizontal or vertical orientation
//! (`data-[orientation=vertical]`: base reserves `min-h-44` (176pt), nova
//! `min-h-40` (160pt); the minimum value sits at the bottom). Thumbs are
//! individually focusable; Arrow keys move the focused thumb (Up/Down on
//! vertical sliders), Home/End jump to the extremes, dragging and track
//! clicks jump the nearest thumb.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut volume = 40.0;
//! Slider::new(&mut volume).min(0.0).max(100.0).show(ui);
//!
//! let mut bounds = (20.0, 80.0);
//! Slider::range(&mut bounds).min(0.0).max(100.0).show(ui);
//!
//! Slider::new(&mut volume).orientation(GroupOrientation::Vertical).show(ui);
//! ```

use crate::components::button::GroupOrientation;
use crate::theme::{color_alpha, Palette, ShadcnTheme, Style};
use egui::{Color32, CornerRadius, Key, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// `size-4` — the (base) thumb diameter; nova renders `size-3` (12pt).
pub const SLIDER_THUMB_SIZE: f32 = 16.0;
/// `h-1.5` — the (base) track thickness; nova renders `h-1` (4pt).
pub const SLIDER_TRACK_HEIGHT: f32 = 6.0;
/// Extra room so the focus/hover ring is not clipped.
const WIDGET_WIDTH: f32 = 24.0;

/// Thumb diameter per style (`size-4` base, nova `size-3`).
pub(crate) fn thumb_size(style: Style) -> f32 {
    match style {
        Style::Nova => 12.0,
        _ => SLIDER_THUMB_SIZE,
    }
}

/// Track thickness per style (`h-1.5` base, nova `h-1`).
pub(crate) fn track_height(style: Style) -> f32 {
    match style {
        Style::Nova => 4.0,
        _ => SLIDER_TRACK_HEIGHT,
    }
}

/// Thumb halo width per style (`ring-4` base, nova `ring-3`).
pub(crate) fn ring_width(style: Style) -> f32 {
    match style {
        Style::Nova => 3.0,
        _ => 4.0,
    }
}

/// Reserved column height for `data-[orientation=vertical]`
/// (base `min-h-44` = 176, nova `min-h-40` = 160).
pub(crate) fn vertical_min_height(style: Style) -> f32 {
    match style {
        Style::Nova => 160.0,
        _ => 176.0,
    }
}

/// What values the slider controls.
#[derive(Debug)]
pub enum SliderValue<'a> {
    /// One thumb.
    Single(&'a mut f32),
    /// Two thumbs (lo must stay <= hi).
    Range(&'a mut (f32, f32)),
}

impl<'a> SliderValue<'a> {
    fn count(&self) -> usize {
        match self {
            SliderValue::Single(_) => 1,
            SliderValue::Range(_) => 2,
        }
    }
}

/// Resolved colors for one slider state (pure, unit-testable).
#[derive(Clone, Copy, Debug)]
pub(crate) struct SliderColors {
    track: Color32,
    range: Color32,
    thumb_fill: Color32,
    thumb_stroke: Stroke,
    /// Focus/hover ring color (alpha already applied).
    ring: Color32,
}

/// Mirrors the base recipe: track `bg-muted`, range `bg-primary`, thumb
/// `border border-primary bg-white`; nova replaces the thumb border with
/// `border-ring`.
pub(crate) fn resolve_slider_colors(p: &Palette, dark: bool, style: Style) -> SliderColors {
    let thumb_stroke_color = match style {
        Style::Nova => p.ring,
        _ => p.primary,
    };
    SliderColors {
        track: p.muted,
        range: p.primary,
        thumb_fill: if dark { p.background } else { Color32::WHITE },
        thumb_stroke: Stroke::new(1.0, thumb_stroke_color),
        ring: color_alpha(p.ring, 0.5),
    }
}

/// A shadcn slider. See the [module docs](self) for an example.
#[derive(Debug)]
pub struct Slider<'a> {
    value: SliderValue<'a>,
    min: f32,
    max: f32,
    step: Option<f32>,
    orientation: GroupOrientation,
    disabled: bool,
    id_salt: Option<String>,
}

impl<'a> Slider<'a> {
    /// Creates a single-value slider bound to the given float.
    pub fn new(value: &'a mut f32) -> Self {
        Self {
            value: SliderValue::Single(value),
            min: 0.0,
            max: 100.0,
            step: None,
            orientation: GroupOrientation::Horizontal,
            disabled: false,
            id_salt: None,
        }
    }

    /// Creates a two-value (range) slider bound to the given pair
    /// (the source renders one thumb per value).
    pub fn range(value: &'a mut (f32, f32)) -> Self {
        Self {
            value: SliderValue::Range(value),
            min: 0.0,
            max: 100.0,
            step: None,
            orientation: GroupOrientation::Horizontal,
            disabled: false,
            id_salt: None,
        }
    }

    /// Sets the minimum value (source default `0`).
    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    /// Sets the maximum value (source default `100`).
    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    /// Snaps values to this increment while dragging and on Arrow keys.
    pub fn step(mut self, step: f32) -> Self {
        self.step = Some(step.max(f32::EPSILON));
        self
    }

    /// Sets the layout orientation (`data-[orientation=vertical]` renders a
    /// tall column with the minimum at the bottom; default horizontal).
    pub fn orientation(mut self, orientation: GroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Disables the slider (`data-[disabled]:opacity-50`).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Sets a stable id salt so focus and drag state persist per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    fn read_value(&self, index: usize) -> f32 {
        match &self.value {
            SliderValue::Single(v) => **v,
            SliderValue::Range(pair) => {
                if index == 0 {
                    pair.0
                } else {
                    pair.1
                }
            }
        }
    }

    /// Renders the slider.
    pub fn show(mut self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let dark = theme.mode.is_dark();
        let style = theme.style;
        let c = resolve_slider_colors(&p, dark, style);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_slider"),
        };

        let vertical = self.orientation == GroupOrientation::Vertical;
        let thumb = thumb_size(style);
        let track = track_height(style);
        let (_, rect) = if vertical {
            // `data-[orientation=vertical]:min-h-44` (nova `min-h-40`).
            let height = vertical_min_height(style).max(thumb * 3.0);
            ui.allocate_space(Vec2::new(WIDGET_WIDTH, height))
        } else {
            let width = ui.available_width().max(thumb * 3.0);
            ui.allocate_space(Vec2::new(width, WIDGET_WIDTH))
        };
        let track_rect = if vertical {
            Rect::from_center_size(rect.center(), Vec2::new(track, rect.height()))
        } else {
            Rect::from_center_size(rect.center(), Vec2::new(rect.width(), track))
        };

        if !ui.is_rect_visible(rect) {
            return ui.interact(rect, id, Sense::hover());
        }

        let painter = ui.painter();
        let mut alpha = 1.0;
        if self.disabled {
            alpha = 0.5;
        }
        let track_fill = color_alpha(c.track, alpha);
        let range_fill = color_alpha(c.range, alpha);

        // Axis helpers: `frac` maps a value onto 0..1; `pos_at` places that
        // fraction along the axis (vertical sliders grow bottom-up, like the
        // Radix `data-[orientation=vertical]` layout).
        let axis_len = if vertical {
            track_rect.height()
        } else {
            track_rect.width()
        };
        let pos_at = |frac: f32| -> f32 {
            if vertical {
                track_rect.max.y - frac * axis_len
            } else {
                track_rect.min.x + frac * axis_len
            }
        };
        let thumb_center = |frac: f32| -> Pos2 {
            if vertical {
                Pos2::new(track_rect.center().x, pos_at(frac))
            } else {
                Pos2::new(pos_at(frac), rect.center().y)
            }
        };

        // Track (`h-1.5 rounded-full bg-muted`) and range (`bg-primary`).
        painter.rect_filled(track_rect, CornerRadius::same(track as u8 / 2), track_fill);
        let n = self.value.count();
        let (min, max) = (self.min, self.max);
        let frac = |v: f32| ((v - min) / (max - min).max(f32::EPSILON)).clamp(0.0, 1.0);
        let values: Vec<f32> = (0..n).map(|i| self.read_value(i)).collect();
        let min_frac = values
            .iter()
            .map(|v| frac(*v))
            .fold(f32::INFINITY, f32::min);
        let max_frac = values
            .iter()
            .map(|v| frac(*v))
            .fold(f32::NEG_INFINITY, f32::max);
        if max_frac > min_frac {
            let range_rect = if vertical {
                Rect::from_min_max(
                    Pos2::new(track_rect.min.x, pos_at(max_frac)),
                    Pos2::new(track_rect.max.x, pos_at(min_frac)),
                )
            } else {
                Rect::from_min_max(
                    Pos2::new(pos_at(min_frac), track_rect.min.y),
                    Pos2::new(pos_at(max_frac), track_rect.max.y),
                )
            };
            painter.rect_filled(range_rect, CornerRadius::same(track as u8 / 2), range_fill);
        }

        // Track interaction: click/drag jumps the nearest thumb. The thumbs
        // are interacted after the track, so they win when the pointer is
        // directly over them.
        let track_response = ui.interact(rect, id.with("track"), Sense::drag());
        let mut active = ui
            .ctx()
            .data_mut(|d| d.get_temp::<usize>(id.with("active_thumb")));

        let mut changed = false;
        let pointer_frac = |ui: &Ui| -> f32 {
            ui.input(|i| {
                i.pointer
                    .latest_pos()
                    .map(|pos| {
                        let along = if vertical {
                            track_rect.max.y - pos.y
                        } else {
                            pos.x - track_rect.min.x
                        };
                        along / axis_len.max(f32::EPSILON)
                    })
                    .unwrap_or(0.0)
            })
        };

        let mut thumb_responses: Vec<Response> = Vec::with_capacity(n);
        for i in 0..n {
            let v = self.read_value(i);
            let thumb_rect = Rect::from_center_size(thumb_center(frac(v)), Vec2::splat(thumb));
            let sense = if self.disabled {
                Sense::hover()
            } else {
                Sense::drag()
            };
            let response = ui.interact(thumb_rect, id.with("thumb").with(i), sense);
            if response.dragged() {
                self.set_value(i, pointer_frac(ui), &mut changed);
                active = Some(i);
            }
            thumb_responses.push(response);
        }

        if !self.disabled && track_response.dragged() {
            let f = pointer_frac(ui);
            let nearest = match (active, values.as_slice()) {
                (Some(i), _) if i < n => i,
                _ => {
                    let mut best = 0;
                    let mut best_dist = f32::INFINITY;
                    for (i, v) in values.iter().enumerate() {
                        let d = (frac(*v) - f).abs();
                        if d < best_dist {
                            best_dist = d;
                            best = i;
                        }
                    }
                    best
                }
            };
            self.set_value(nearest, f, &mut changed);
            active = Some(nearest);
        }

        // Track the active thumb for later drags/clicks.
        ui.ctx()
            .data_mut(|d| d.insert_temp(id.with("active_thumb"), active.unwrap_or(0)));

        // Thumbs: `size-4 rounded-full border border-primary bg-white
        // shadow-sm`, `hover:ring-4 focus-visible:ring-4 ring-ring/50`
        // (nova: `size-3 border-ring`, `ring-3`).
        for (i, response) in thumb_responses.iter().enumerate() {
            if !ui.is_rect_visible(response.rect) {
                continue;
            }
            let v = self.read_value(i);
            let thumb_rect = Rect::from_center_size(thumb_center(frac(v)), Vec2::splat(thumb));
            let mut fill = color_alpha(c.thumb_fill, alpha);
            let mut stroke = Stroke::new(
                c.thumb_stroke.width,
                color_alpha(c.thumb_stroke.color, alpha),
            );
            let ring = (response.hovered() || response.dragged() || response.has_focus())
                && !self.disabled;
            if self.disabled {
                fill = color_alpha(fill, 0.5);
                stroke.color = color_alpha(stroke.color, 0.5);
            }

            if ring {
                crate::utils::focus::paint_ring(
                    painter,
                    thumb_rect.expand2(egui::vec2(2.0, 2.0)),
                    c.ring,
                    ring_width(style),
                    thumb / 2.0,
                );
            }
            // `shadow-sm`
            if !self.disabled {
                let sh = crate::utils::shadow::shadow_sm(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        thumb_rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        CornerRadius::same(thumb as u8 / 2),
                        sh.color,
                    )
                    .with_blur_width(2.5),
                );
            }
            painter.circle_filled(thumb_rect.center(), thumb / 2.0, fill);
            painter.circle_stroke(thumb_rect.center(), thumb / 2.0, stroke);
        }

        // Keyboard on the focused thumb: arrows step, Home/End jump. On
        // vertical sliders Up/Down are the primary axes (Radix vertical).
        if !self.disabled {
            for (i, response) in thumb_responses.iter().enumerate() {
                if !response.has_focus() {
                    continue;
                }
                let step_size = self
                    .step
                    .unwrap_or((self.max - self.min).max(f32::EPSILON) * 0.01);
                ui.input(|inp| {
                    let mut target: Option<f32> = None;
                    if inp.key_pressed(Key::ArrowRight) || inp.key_pressed(Key::ArrowUp) {
                        target = Some(self.read_value(i) + step_size);
                    } else if inp.key_pressed(Key::ArrowLeft) || inp.key_pressed(Key::ArrowDown) {
                        target = Some(self.read_value(i) - step_size);
                    } else if inp.key_pressed(Key::Home) {
                        target = Some(self.min);
                    } else if inp.key_pressed(Key::End) {
                        target = Some(self.max);
                    }
                    if let Some(t) = target {
                        let mut c = false;
                        self.set_value(i, t, &mut c);
                        changed |= c;
                    }
                });
            }
        }

        let mut response = track_response;
        if changed {
            response.mark_changed();
        }
        response
    }

    fn set_value(&mut self, index: usize, fraction: f32, changed: &mut bool) {
        let raw = self.min + fraction.clamp(0.0, 1.0) * (self.max - self.min);
        let mut snapped = match self.step {
            Some(step) => (raw / step).round() * step,
            None => raw,
        };
        snapped = snapped.clamp(self.min, self.max);
        match &mut self.value {
            SliderValue::Single(v) => {
                if (**v - snapped).abs() > f32::EPSILON {
                    **v = snapped;
                    *changed = true;
                }
            }
            SliderValue::Range(pair) => {
                let target = if index == 0 {
                    // lo may not pass hi
                    snapped.min(pair.1)
                } else {
                    snapped.max(pair.0)
                };
                let current = if index == 0 { pair.0 } else { pair.1 };
                if (current - target).abs() > f32::EPSILON {
                    if index == 0 {
                        pair.0 = target;
                    } else {
                        pair.1 = target;
                    }
                    *changed = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Style;

    fn palette(mode: crate::theme::Mode) -> Palette {
        crate::theme::Preset::default().build(mode).palette
    }

    #[test]
    fn track_and_thumb_use_source_tokens() {
        let p = palette(crate::theme::Mode::Light);
        let c = resolve_slider_colors(&p, false, Style::Vega);
        assert_eq!(c.track, p.muted);
        assert_eq!(c.range, p.primary);
        assert_eq!(c.thumb_fill, Color32::WHITE);
        assert_eq!(c.thumb_stroke.color, p.primary);
        assert_eq!(c.ring, color_alpha(p.ring, 0.5));
        let dark = palette(crate::theme::Mode::Dark);
        let c = resolve_slider_colors(&dark, true, Style::Vega);
        assert_eq!(c.thumb_fill, dark.background);
    }

    #[test]
    fn nova_uses_smaller_thumb_ring_border_and_thinner_track() {
        let p = palette(crate::theme::Mode::Light);
        // nova thumb `size-3 border border-ring bg-white`
        let c = resolve_slider_colors(&p, false, Style::Nova);
        assert_eq!(c.thumb_stroke.color, p.ring);
        // base keeps `border-primary`
        let base = resolve_slider_colors(&p, false, Style::Vega);
        assert_eq!(base.thumb_stroke.color, p.primary);
        // geometry: base size-4/h-1.5, nova size-3/h-1
        assert_eq!(thumb_size(Style::Vega), 16.0);
        assert_eq!(thumb_size(Style::Nova), 12.0);
        assert_eq!(track_height(Style::Vega), 6.0);
        assert_eq!(track_height(Style::Nova), 4.0);
        // halo: ring-4 base, ring-3 nova
        assert_eq!(ring_width(Style::Vega), 4.0);
        assert_eq!(ring_width(Style::Nova), 3.0);
        // vertical column: min-h-44 (176) base, min-h-40 (160) nova
        assert_eq!(vertical_min_height(Style::Vega), 176.0);
        assert_eq!(vertical_min_height(Style::Nova), 160.0);
    }

    #[test]
    fn set_value_snaps_to_step_and_clamps() {
        let mut v = 50.0;
        let mut slider = Slider::new(&mut v).min(0.0).max(100.0).step(10.0);
        let mut changed = false;
        slider.set_value(0, 0.54, &mut changed); // 54 -> snaps to 50
        assert!(!changed);
        assert_eq!(slider.read_value(0), 50.0);
        slider.set_value(0, 0.55, &mut changed); // 55 rounds to 60
        assert!(changed);
        assert_eq!(slider.read_value(0), 60.0);
        slider.set_value(0, 1.5, &mut changed); // clamps to max
        assert_eq!(slider.read_value(0), 100.0);
        slider.set_value(0, -0.5, &mut changed); // clamps to min
        assert_eq!(slider.read_value(0), 0.0);
    }

    #[test]
    fn range_thumbs_cannot_cross() {
        let mut pair = (20.0, 80.0);
        let mut slider = Slider::range(&mut pair).min(0.0).max(100.0);
        let mut changed = false;
        slider.set_value(0, 0.95, &mut changed); // lo tries to pass hi
        assert!(changed);
        assert_eq!((slider.read_value(0), slider.read_value(1)), (80.0, 80.0));
        slider.set_value(1, 0.05, &mut changed); // hi clamps to lo (never crosses)
        assert_eq!((slider.read_value(0), slider.read_value(1)), (80.0, 80.0));
    }

    #[test]
    fn builders_keep_fields() {
        let mut v = 0.0;
        let s = Slider::new(&mut v)
            .min(5.0)
            .max(15.0)
            .step(1.0)
            .orientation(GroupOrientation::Vertical)
            .disabled(true)
            .id_salt("s");
        assert_eq!(s.min, 5.0);
        assert_eq!(s.max, 15.0);
        assert_eq!(s.step, Some(1.0));
        assert_eq!(s.orientation, GroupOrientation::Vertical);
        assert!(s.disabled);
        assert_eq!(s.id_salt.as_deref(), Some("s"));
    }

    #[test]
    fn vertical_slider_renders_a_tall_column() {
        let ctx = egui::Context::default();
        crate::theme::Preset::default().install(&ctx);
        let mut v = 40.0;
        let mut rect = Rect::NOTHING;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("slider_vertical_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(200.0, 400.0))),
            );
            rect = Slider::new(&mut v)
                .min(0.0)
                .max(100.0)
                .orientation(GroupOrientation::Vertical)
                .id_salt("vertical")
                .show(ui)
                .rect;
        });
        out.textures_delta.clear();
        // The default preset is nova: `min-h-40` column (160 tall), 24pt wide.
        assert!((rect.height() - vertical_min_height(Style::Nova)).abs() < 0.5);
        assert!((rect.width() - 24.0).abs() < 0.5);
        assert_eq!(v, 40.0);
    }
}
