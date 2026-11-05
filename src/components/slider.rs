//! shadcn `slider.tsx` — a draggable value selector.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/slider.tsx`: a `h-1.5
//! rounded-full bg-muted` track with a `bg-primary` range fill and `size-4
//! rounded-full border border-primary bg-white shadow-sm` thumbs
//! (`hover:ring-4 focus-visible:ring-4 ring-ring/50`,
//! `data-[disabled]:opacity-50`).
//!
//! Both a single value and a two-value range are supported (the source
//! renders one thumb per value). Thumbs are individually focusable; Arrow
//! keys move the focused thumb, Home/End jump to the extremes, dragging and
//! track clicks jump the nearest thumb.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let mut volume = 40.0;
//! Slider::new(&mut volume).min(0.0).max(100.0).show(ui);
//!
//! let mut bounds = (20.0, 80.0);
//! Slider::range(&mut bounds).min(0.0).max(100.0).show(ui);
//! ```

use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{Color32, CornerRadius, Key, Pos2, Rect, Response, Sense, Stroke, Ui, Vec2};

/// `size-4` — the thumb diameter.
pub const SLIDER_THUMB_SIZE: f32 = 16.0;
/// `h-1.5` — the track thickness.
pub const SLIDER_TRACK_HEIGHT: f32 = 6.0;
/// Extra vertical room so the `hover:ring-4` focus ring is not clipped.
const WIDGET_HEIGHT: f32 = 24.0;

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
/// `border border-primary bg-white`.
pub(crate) fn resolve_slider_colors(p: &Palette, dark: bool) -> SliderColors {
    SliderColors {
        track: p.muted,
        range: p.primary,
        thumb_fill: if dark { p.background } else { Color32::WHITE },
        thumb_stroke: Stroke::new(1.0, p.primary),
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
        let c = resolve_slider_colors(&p, dark);

        let id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_slider"),
        };

        let width = ui.available_width().max(SLIDER_THUMB_SIZE * 3.0);
        let desired = Vec2::new(width, WIDGET_HEIGHT);
        let (_, rect) = ui.allocate_space(desired);
        let track_rect =
            Rect::from_center_size(rect.center(), Vec2::new(width, SLIDER_TRACK_HEIGHT));

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

        // Track (`h-1.5 rounded-full bg-muted`) and range (`bg-primary`).
        painter.rect_filled(
            track_rect,
            CornerRadius::same(SLIDER_TRACK_HEIGHT as u8 / 2),
            track_fill,
        );
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
            let range_rect = Rect::from_min_max(
                Pos2::new(track_rect.min.x + min_frac * width, track_rect.min.y),
                Pos2::new(track_rect.min.x + max_frac * width, track_rect.max.y),
            );
            painter.rect_filled(
                range_rect,
                CornerRadius::same(SLIDER_TRACK_HEIGHT as u8 / 2),
                range_fill,
            );
        }

        // Track interaction: click/drag jumps the nearest thumb. The thumbs
        // are interacted after the track, so they win when the pointer is
        // directly over them.
        let track_response = ui.interact(rect, id.with("track"), Sense::drag());
        let mut active = ui
            .ctx()
            .data_mut(|d| d.get_temp::<usize>(id.with("active_thumb")));

        let mut changed = false;
        let pointer_x = |ui: &Ui, rect: Rect| -> f32 {
            ui.input(|i| {
                i.pointer
                    .latest_pos()
                    .map(|pos| (pos.x - rect.min.x) / width.max(f32::EPSILON))
                    .unwrap_or(0.0)
            })
        };

        let mut thumb_responses: Vec<Response> = Vec::with_capacity(n);
        for i in 0..n {
            let v = self.read_value(i);
            let thumb_center_x = track_rect.min.x + frac(v) * width;
            let thumb_rect = Rect::from_center_size(
                Pos2::new(thumb_center_x, rect.center().y),
                Vec2::splat(SLIDER_THUMB_SIZE),
            );
            let sense = if self.disabled {
                Sense::hover()
            } else {
                Sense::drag()
            };
            let response = ui.interact(thumb_rect, id.with("thumb").with(i), sense);
            if response.dragged() {
                self.set_value(i, pointer_x(ui, track_rect), &mut changed);
                active = Some(i);
            }
            thumb_responses.push(response);
        }

        if !self.disabled && track_response.dragged() {
            let x = pointer_x(ui, track_rect);
            let nearest = match (active, values.as_slice()) {
                (Some(i), _) if i < n => i,
                _ => {
                    let mut best = 0;
                    let mut best_dist = f32::INFINITY;
                    for (i, v) in values.iter().enumerate() {
                        let d = (frac(*v) - x).abs();
                        if d < best_dist {
                            best_dist = d;
                            best = i;
                        }
                    }
                    best
                }
            };
            self.set_value(nearest, x, &mut changed);
            active = Some(nearest);
        }

        // Track the active thumb for later drags/clicks.
        ui.ctx()
            .data_mut(|d| d.insert_temp(id.with("active_thumb"), active.unwrap_or(0)));

        // Thumbs: `size-4 rounded-full border border-primary bg-white
        // shadow-sm`, `hover:ring-4 focus-visible:ring-4 ring-ring/50`.
        for (i, response) in thumb_responses.iter().enumerate() {
            if !ui.is_rect_visible(response.rect) {
                continue;
            }
            let v = self.read_value(i);
            let thumb_center_x = track_rect.min.x + frac(v) * width;
            let thumb_rect = Rect::from_center_size(
                Pos2::new(thumb_center_x, rect.center().y),
                Vec2::splat(SLIDER_THUMB_SIZE),
            );
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
                // `hover:ring-4 focus-visible:ring-4 ring-ring/50`
                crate::utils::focus::paint_ring(
                    painter,
                    thumb_rect.expand2(egui::vec2(2.0, 2.0)),
                    c.ring,
                    4.0,
                    SLIDER_THUMB_SIZE / 2.0,
                );
            }
            // `shadow-sm`
            if !self.disabled {
                let sh = crate::utils::shadow::shadow_sm(dark);
                painter.add(
                    egui::epaint::RectShape::filled(
                        thumb_rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                        CornerRadius::same(SLIDER_THUMB_SIZE as u8 / 2),
                        sh.color,
                    )
                    .with_blur_width(2.5),
                );
            }
            painter.circle_filled(thumb_rect.center(), SLIDER_THUMB_SIZE / 2.0, fill);
            painter.circle_stroke(thumb_rect.center(), SLIDER_THUMB_SIZE / 2.0, stroke);
        }

        // Keyboard on the focused thumb: arrows step, Home/End jump.
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

    fn palette(mode: crate::theme::Mode) -> Palette {
        crate::theme::Preset::default().build(mode).palette
    }

    #[test]
    fn track_and_thumb_use_source_tokens() {
        let p = palette(crate::theme::Mode::Light);
        let c = resolve_slider_colors(&p, false);
        assert_eq!(c.track, p.muted);
        assert_eq!(c.range, p.primary);
        assert_eq!(c.thumb_fill, Color32::WHITE);
        assert_eq!(c.thumb_stroke.color, p.primary);
        assert_eq!(c.ring, color_alpha(p.ring, 0.5));
        let dark = palette(crate::theme::Mode::Dark);
        let c = resolve_slider_colors(&dark, true);
        assert_eq!(c.thumb_fill, dark.background);
    }

    #[test]
    fn thumb_and_track_geometry() {
        assert_eq!(SLIDER_THUMB_SIZE, 16.0); // size-4
        assert_eq!(SLIDER_TRACK_HEIGHT, 6.0); // h-1.5
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
            .disabled(true)
            .id_salt("s");
        assert_eq!(s.min, 5.0);
        assert_eq!(s.max, 15.0);
        assert_eq!(s.step, Some(1.0));
        assert!(s.disabled);
        assert_eq!(s.id_salt.as_deref(), Some("s"));
    }
}
