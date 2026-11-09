//! shadcn `carousel.tsx` — a pannable slide strip (embla-carousel).
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/carousel.tsx`: a clipped
//! viewport with slides laid out in a strip (`basis-full pl-4` — one slide
//! per view, 16 px gutters), outline `size-8 rounded-full` prev/next arrows
//! perched outside the viewport (`-left-12` / `-right-12`, vertically
//! centered; above/below for the vertical orientation), drag/swipe panning
//! that snaps to the nearest slide on release, ArrowLeft/ArrowRight keyboard
//! navigation, and a dots indicator (active `bg-primary`, inactive
//! `bg-muted/50`, click to jump).
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! Carousel::new().id_salt("demo").show(ui, |ui| {
//!     for i in 0..5 {
//!         CarouselItem::new().show(ui, |ui| {
//!             ui.label(format!("slide {i}"));
//!         });
//!     }
//! });
//! ```

use crate::icons::Icon;
use crate::theme::{color_alpha, ShadcnTheme};
use egui::{CornerRadius, Id, Key, Pos2, Rect, Response, Sense, Stroke, Ui, UiBuilder, Vec2};

// ---------------------------------------------------------------------------
// Constants (carousel.tsx + nova style CSS)
// ---------------------------------------------------------------------------

/// Slide gutter: `pl-4` / `pt-4` = 16 px.
pub const CAROUSEL_ITEM_GAP: f32 = 16.0;
/// Arrow outset: `-left-12` / `-right-12` = 48 px from the viewport edge.
pub const CAROUSEL_ARROW_GUTTER: f32 = 48.0;
/// Arrow button size: `size-8` = 32 px.
pub const CAROUSEL_ARROW_SIZE: f32 = 32.0;
/// Arrow icon size (`size-4`).
const ARROW_ICON_SIZE: f32 = 16.0;
/// Dot indicator size (`h-2 w-2`).
pub const CAROUSEL_DOT_SIZE: f32 = 8.0;
/// Viewport height fallback before slide sizes have been measured.
const DEFAULT_VIEWPORT_CROSS: f32 = 120.0;
/// Snap animation duration in seconds (embla's ~duration 25 → 0.25 s).
const SNAP_DURATION: f32 = 0.25;

// ---------------------------------------------------------------------------
// Pure layout / math helpers
// ---------------------------------------------------------------------------

/// Scroll offset (px) at which each slide sits flush with the viewport start.
/// Horizontal slides have a fixed pitch (`slide_width`); vertical slides stack
/// with a 16 px gap between contents.
fn slide_offsets(sizes: &[f32], horizontal: bool, slide_width: f32) -> Vec<f32> {
    if horizontal {
        (0..sizes.len()).map(|i| i as f32 * slide_width).collect()
    } else {
        let mut offsets = Vec::with_capacity(sizes.len());
        let mut acc = 0.0;
        for size in sizes {
            offsets.push(acc);
            acc += size + CAROUSEL_ITEM_GAP;
        }
        offsets
    }
}

/// Total strip length (px) along the scroll axis.
fn strip_length(sizes: &[f32], horizontal: bool, slide_width: f32) -> f32 {
    if horizontal {
        sizes.len() as f32 * slide_width
    } else {
        sizes.iter().sum::<f32>() + sizes.len() as f32 * CAROUSEL_ITEM_GAP
    }
}

/// Largest scroll offset that keeps the strip in bounds: the last slide flush
/// with the viewport start, never scrolling past the content end.
fn max_offset(sizes: &[f32], horizontal: bool, slide_width: f32, viewport_extent: f32) -> f32 {
    let offsets = slide_offsets(sizes, horizontal, slide_width);
    let Some(last) = offsets.last().copied() else {
        return 0.0;
    };
    let total = strip_length(sizes, horizontal, slide_width);
    last.min((total - viewport_extent).max(0.0)).max(0.0)
}

/// Index of the slide whose offset is closest to `x`.
fn nearest_slide(offsets: &[f32], x: f32) -> usize {
    let mut best = 0usize;
    let mut best_dist = f32::INFINITY;
    for (i, &o) in offsets.iter().enumerate() {
        let dist = (o - x).abs();
        if dist < best_dist {
            best_dist = dist;
            best = i;
        }
    }
    best
}

/// A snap animation between two offsets.
#[derive(Clone, Copy, Debug, PartialEq)]
struct ScrollAnim {
    from: f32,
    to: f32,
    start_secs: f64,
}

impl ScrollAnim {
    /// Offset at `now`, eased with smoothstep; `Some(final)` when finished.
    fn offset_at(&self, now: f64) -> (f32, bool) {
        let t = (((now - self.start_secs) / f64::from(SNAP_DURATION)) as f32).clamp(0.0, 1.0);
        let eased = t * t * (3.0 - 2.0 * t);
        (self.from + (self.to - self.from) * eased, t >= 1.0)
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Per-instance state persisted in the egui context.
#[derive(Clone, Debug, Default)]
struct CarouselState {
    /// Current scroll offset in px (0 = first slide flush).
    offset: f32,
    /// Running snap animation, if any.
    anim: Option<ScrollAnim>,
    /// Measured slide heights from the last frame (px).
    item_sizes: Vec<f32>,
    /// Tallest slide so far — the viewport height.
    viewport_cross: f32,
    /// Selected slide index.
    selected: usize,
}

/// Scratch shared between [`Carousel`] and [`CarouselItem`] while the items
/// closure runs (rebuilt every frame).
#[derive(Clone, Debug, Default)]
struct ScopeScratch {
    counter: usize,
    sizes: Vec<f32>,
}

#[derive(Clone, Debug)]
struct CarouselScope {
    state_id: Id,
    horizontal: bool,
    slide_width: f32,
    viewport: Rect,
    strip_origin: Pos2,
    offsets: Vec<f32>,
    last_sizes: Vec<f32>,
}

// ---------------------------------------------------------------------------
// Carousel (root)
// ---------------------------------------------------------------------------

/// Carousel orientation (`orientation` prop).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CarouselOrientation {
    /// Slides pan horizontally (the source default).
    #[default]
    Horizontal,
    /// Slides pan vertically.
    Vertical,
}

/// What happened to a [`Carousel`] this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CarouselResponse {
    /// Index of the selected (snapped) slide.
    pub selected: usize,
    /// The selected slide changed this frame.
    pub changed: bool,
}

/// The shadcn carousel root. See the [module docs](self) for an example.
///
/// The viewport height equals the tallest slide (falling back to 120 px on
/// the first frame); the prev/next arrows hang outside the viewport, so the
/// root reserves a 48 px gutter on each arrow side when arrows are enabled.
pub struct Carousel {
    id_salt: Option<String>,
    orientation: CarouselOrientation,
    show_arrows: bool,
    show_dots: bool,
}

impl Default for Carousel {
    fn default() -> Self {
        Self::new()
    }
}

impl Carousel {
    /// Creates a horizontal carousel with arrows and dots.
    pub fn new() -> Self {
        Self {
            id_salt: None,
            orientation: CarouselOrientation::Horizontal,
            show_arrows: true,
            show_dots: true,
        }
    }

    /// Sets a stable id salt so scroll/focus state persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Sets the pan axis (`orientation` prop).
    pub fn orientation(mut self, orientation: CarouselOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Shows the prev/next arrow buttons (default `true`).
    pub fn show_arrows(mut self, show: bool) -> Self {
        self.show_arrows = show;
        self
    }

    /// Shows the dots indicator under the viewport (default `true`).
    pub fn show_dots(mut self, show: bool) -> Self {
        self.show_dots = show;
        self
    }

    /// Lays out the carousel: viewport with pannable slides, arrows and dots.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> CarouselResponse {
        let theme = ShadcnTheme::of_ui(ui);
        let dark = theme.mode.is_dark();
        let p = theme.palette;
        let horizontal = self.orientation == CarouselOrientation::Horizontal;

        let state_id = match self.id_salt.as_deref() {
            Some(salt) => ui.scope_id().with(salt),
            None => ui.auto_id_with("shadcn_carousel"),
        };
        let ctx = ui.ctx().clone();
        let mut state: CarouselState = ctx
            .data_mut(|d| d.get_temp::<CarouselState>(state_id))
            .unwrap_or_default();

        // --- Geometry ---------------------------------------------------------
        let gutter = if self.show_arrows {
            CAROUSEL_ARROW_GUTTER
        } else {
            0.0
        };
        let avail_w = ui.available_width();
        let viewport_cross = if state.viewport_cross > 0.0 {
            state.viewport_cross
        } else {
            DEFAULT_VIEWPORT_CROSS
        };
        let (root_size, viewport_size) = if horizontal {
            (
                Vec2::new((avail_w).max(2.0 * gutter + 120.0), viewport_cross),
                Vec2::new((avail_w - 2.0 * gutter).max(120.0), viewport_cross),
            )
        } else {
            (
                Vec2::new(avail_w, viewport_cross + 2.0 * gutter),
                Vec2::new(avail_w, viewport_cross),
            )
        };
        let (_, root_rect) = ui.allocate_space(root_size);
        let viewport = Rect::from_min_size(
            if horizontal {
                Pos2::new(root_rect.min.x + gutter, root_rect.min.y)
            } else {
                Pos2::new(root_rect.min.x, root_rect.min.y + gutter)
            },
            viewport_size,
        );

        // Layout from last frame's measurements.
        let sizes = state.item_sizes.clone();
        let offsets = slide_offsets(&sizes, horizontal, viewport.width());
        let max = max_offset(&sizes, horizontal, viewport.width(), viewport.height());

        // --- Scroll offset (animation or drag) --------------------------------
        let now = ui.input(|i| i.time);
        if let Some(anim) = state.anim {
            let (value, done) = anim.offset_at(now);
            state.offset = value;
            if done {
                state.anim = None;
                state.offset = anim.to;
            }
        }
        state.offset = state.offset.clamp(0.0, max);

        // Advance the selected index from the current offset.
        let selected = nearest_slide(&offsets, state.offset);
        let changed = selected != state.selected;
        state.selected = selected;

        // --- Viewport interaction (drag / keyboard / focus) --------------------
        let viewport_id = state_id.with("viewport");
        let viewport_response = ui.interact(viewport, viewport_id, Sense::click_and_drag());
        if viewport_response.dragged() {
            state.anim = None;
            let delta = viewport_response.drag_delta();
            state.offset = if horizontal {
                state.offset - delta.x
            } else {
                state.offset - delta.y
            }
            .clamp(0.0, max);
        }
        if viewport_response.drag_stopped() {
            let target = if offsets.is_empty() {
                0.0
            } else {
                offsets[nearest_slide(&offsets, state.offset)]
            };
            start_anim(&mut state, target, now);
        }

        // --- Render slides -----------------------------------------------------
        let strip_origin = if horizontal {
            Pos2::new(
                viewport.min.x - CAROUSEL_ITEM_GAP - state.offset,
                viewport.min.y,
            )
        } else {
            Pos2::new(
                viewport.min.x,
                viewport.min.y - CAROUSEL_ITEM_GAP - state.offset,
            )
        };
        let scope = CarouselScope {
            state_id,
            horizontal,
            slide_width: viewport.width(),
            viewport,
            strip_origin,
            offsets: offsets.clone(),
            last_sizes: sizes.clone(),
        };
        let scratch_id = state_id.with("scratch");
        ctx.data_mut(|d| d.insert_temp(scratch_id, ScopeScratch::default()));
        push_scope(ui, scope);
        content(ui);
        pop_scope(ui);
        let scratch: ScopeScratch = ctx
            .data_mut(|d| d.get_temp::<ScopeScratch>(scratch_id))
            .unwrap_or_default();
        state.item_sizes = scratch.sizes;
        let tallest = state.item_sizes.iter().copied().fold(0.0f32, f32::max);
        if tallest > 0.0 {
            state.viewport_cross = tallest;
        }

        // --- Arrows -------------------------------------------------------------
        let can_prev = state.offset > 0.5;
        let can_next = state.offset < max - 0.5 && !offsets.is_empty();
        if self.show_arrows {
            let (prev_rect, next_rect, prev_icon, next_icon) = arrow_rects(viewport, horizontal);
            let prev = arrow_button(
                ui,
                prev_rect,
                state_id.with("arrow_prev"),
                prev_icon,
                can_prev,
                dark,
                &p,
            );
            let next = arrow_button(
                ui,
                next_rect,
                state_id.with("arrow_next"),
                next_icon,
                can_next,
                dark,
                &p,
            );
            if prev.clicked() && state.selected > 0 {
                let target = offsets[state.selected - 1];
                start_anim(&mut state, target, now);
            }
            if next.clicked() && state.selected + 1 < offsets.len() {
                let target = offsets[state.selected + 1];
                start_anim(&mut state, target, now);
            }
        }

        // --- Keyboard (Arrow keys move a slide while the viewport is focused) --
        if viewport_response.has_focus() {
            let step = ui.input(|i| {
                if horizontal {
                    if i.key_pressed(Key::ArrowLeft) {
                        Some(-1)
                    } else if i.key_pressed(Key::ArrowRight) {
                        Some(1)
                    } else {
                        None
                    }
                } else if i.key_pressed(Key::ArrowUp) {
                    Some(-1)
                } else if i.key_pressed(Key::ArrowDown) {
                    Some(1)
                } else {
                    None
                }
            });
            if let Some(step) = step {
                let target = state.selected as isize + step;
                if target >= 0 && (target as usize) < offsets.len() {
                    start_anim(&mut state, offsets[target as usize], now);
                }
            }
        }

        // --- Dots ---------------------------------------------------------------
        if self.show_dots && offsets.len() > 1 {
            let dots_top = root_rect.bottom() + CAROUSEL_ITEM_GAP;
            let n = offsets.len() as f32;
            let row_w = n * CAROUSEL_DOT_SIZE + (n - 1.0) * CAROUSEL_ITEM_GAP;
            let row_rect = Rect::from_min_size(
                Pos2::new(viewport.center().x - row_w / 2.0, dots_top),
                Vec2::new(row_w, CAROUSEL_DOT_SIZE),
            );
            let _ = ui.allocate_space(Vec2::new(
                root_rect.width(),
                CAROUSEL_ITEM_GAP + CAROUSEL_DOT_SIZE,
            ));
            for (i, _) in offsets.iter().enumerate() {
                let x = row_rect.min.x + i as f32 * (CAROUSEL_DOT_SIZE + CAROUSEL_ITEM_GAP);
                let dot = Rect::from_min_size(
                    Pos2::new(x, row_rect.min.y),
                    Vec2::splat(CAROUSEL_DOT_SIZE),
                );
                let dot_id = state_id.with(("dot", i));
                let resp = ui.interact(dot.expand(2.0), dot_id, Sense::click());
                let active = i == state.selected;
                let fill = if active {
                    p.primary
                } else {
                    color_alpha(p.muted, 0.5)
                };
                ui.painter()
                    .circle_filled(dot.center(), CAROUSEL_DOT_SIZE / 2.0, fill);
                if resp.clicked() {
                    start_anim(&mut state, offsets[i], now);
                }
            }
        }

        ctx.data_mut(|d| d.insert_temp(state_id, state));
        CarouselResponse { selected, changed }
    }
}

fn start_anim(state: &mut CarouselState, target: f32, now: f64) {
    if (state.offset - target).abs() < 0.5 {
        state.offset = target;
        state.anim = None;
        return;
    }
    state.anim = Some(ScrollAnim {
        from: state.offset,
        to: target,
        start_secs: now,
    });
}

/// Prev/next arrow rects and icons. Horizontal: outside left/right, centered
/// vertically (`top-1/2 -left-12`). Vertical: above/below, centered
/// horizontally (`-top-12 left-1/2`), with up/down arrows (the source
/// rotates the horizontal icons by 90°).
fn arrow_rects(viewport: Rect, horizontal: bool) -> (Rect, Rect, Icon, Icon) {
    if horizontal {
        let prev = Rect::from_min_size(
            Pos2::new(
                viewport.min.x - CAROUSEL_ARROW_GUTTER,
                viewport.center().y - CAROUSEL_ARROW_SIZE / 2.0,
            ),
            Vec2::splat(CAROUSEL_ARROW_SIZE),
        );
        let next = Rect::from_min_size(
            Pos2::new(
                viewport.max.x + CAROUSEL_ARROW_GUTTER - CAROUSEL_ARROW_SIZE,
                viewport.center().y - CAROUSEL_ARROW_SIZE / 2.0,
            ),
            Vec2::splat(CAROUSEL_ARROW_SIZE),
        );
        (prev, next, Icon::ArrowLeft, Icon::ArrowRight)
    } else {
        let prev = Rect::from_min_size(
            Pos2::new(
                viewport.center().x - CAROUSEL_ARROW_SIZE / 2.0,
                viewport.min.y - CAROUSEL_ARROW_GUTTER,
            ),
            Vec2::splat(CAROUSEL_ARROW_SIZE),
        );
        let next = Rect::from_min_size(
            Pos2::new(
                viewport.center().x - CAROUSEL_ARROW_SIZE / 2.0,
                viewport.max.y + CAROUSEL_ARROW_GUTTER - CAROUSEL_ARROW_SIZE,
            ),
            Vec2::splat(CAROUSEL_ARROW_SIZE),
        );
        (prev, next, Icon::ArrowUp, Icon::ArrowDown)
    }
}

/// Outline `size-8 rounded-full` arrow button (the source's
/// `CarouselPrevious`/`CarouselNext`).
fn arrow_button(
    ui: &mut Ui,
    rect: Rect,
    id: Id,
    icon: Icon,
    enabled: bool,
    dark: bool,
    p: &crate::theme::Palette,
) -> Response {
    let response = ui.interact(
        rect,
        id,
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
    if ui.is_rect_visible(rect) {
        let painter = ui.painter();
        let alpha = if enabled { 1.0 } else { 0.5 };
        let radius = CAROUSEL_ARROW_SIZE / 2.0;
        if !enabled {
            // Disabled outline: no shadow, dimmed.
        } else if response.hovered() {
            painter.circle_filled(rect.center(), radius - 1.0, p.accent);
        } else {
            let sh = crate::utils::shadow::shadow_xs(dark);
            painter.add(
                egui::epaint::RectShape::filled(
                    rect.translate(egui::vec2(sh.offset[0] as f32, sh.offset[1] as f32)),
                    CornerRadius::same(radius as u8),
                    sh.color,
                )
                .with_blur_width(2.0),
            );
            painter.circle_filled(rect.center(), radius - 1.0, p.background);
        }
        painter.circle_stroke(
            rect.center(),
            radius - 1.0,
            Stroke::new(1.0, color_alpha(p.input, alpha)),
        );
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(ARROW_ICON_SIZE));
        icon.paint(
            painter,
            icon_rect,
            color_alpha(p.foreground, alpha),
            2.0 * ARROW_ICON_SIZE / 24.0,
        );
    }
    response
}

// ---------------------------------------------------------------------------
// Scope plumbing (mirrors popover.rs): items find the active carousel.
// ---------------------------------------------------------------------------

fn stack_id() -> Id {
    Id::unique("shadcn_carousel_stack")
}

fn push_scope(ui: &Ui, scope: CarouselScope) {
    let ctx = ui.ctx().clone();
    let mut stack: Vec<CarouselScope> =
        ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
    stack.push(scope);
    ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
}

fn pop_scope(ui: &Ui) {
    let ctx = ui.ctx().clone();
    let mut stack: Vec<CarouselScope> =
        ctx.data_mut(|d| d.get_temp(stack_id()).unwrap_or_default());
    stack.pop();
    ctx.data_mut(|d| d.insert_temp(stack_id(), stack));
}

fn current_scope(ui: &Ui) -> Option<CarouselScope> {
    let ctx = ui.ctx().clone();
    ctx.data_mut(|d| {
        let stack: Vec<CarouselScope> = d.get_temp(stack_id())?;
        stack.last().cloned()
    })
}

// ---------------------------------------------------------------------------
// CarouselItem
// ---------------------------------------------------------------------------

/// One slide of a [`Carousel`], mirroring the source's `CarouselItem`
/// (`basis-full pl-4`: one slide per view, 16 px gutter).
#[derive(Clone, Copy, Debug, Default)]
pub struct CarouselItem {
    _private: (),
}

impl CarouselItem {
    /// Creates a slide.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Renders the slide's content inside the enclosing carousel's strip.
    /// Outside a [`Carousel`] the content renders inline, so mistakes stay
    /// visible.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> Option<R> {
        let Some(scope) = current_scope(ui) else {
            return Some(content(ui));
        };
        let scratch_id = scope.state_id.with("scratch");
        let ctx = ui.ctx().clone();
        let mut scratch: ScopeScratch = ctx
            .data_mut(|d| d.get_temp::<ScopeScratch>(scratch_id))
            .unwrap_or_default();
        let index = scratch.counter;
        scratch.counter += 1;
        ctx.data_mut(|d| d.insert_temp(scratch_id, scratch));

        // Content rect: strip position + leading 16 px gutter.
        let last_size = scope.last_sizes.get(index).copied().unwrap_or(0.0);
        let content_rect = if scope.horizontal {
            let x = scope.strip_origin.x
                + scope.offsets.get(index).copied().unwrap_or(0.0)
                + CAROUSEL_ITEM_GAP;
            Rect::from_min_size(
                Pos2::new(x, scope.viewport.min.y),
                Vec2::new(
                    (scope.slide_width - CAROUSEL_ITEM_GAP).max(1.0),
                    scope.viewport.height(),
                ),
            )
        } else {
            let y = scope.strip_origin.y
                + scope.offsets.get(index).copied().unwrap_or(0.0)
                + CAROUSEL_ITEM_GAP;
            let height = if last_size > 0.0 {
                last_size
            } else {
                scope.viewport.height()
            };
            Rect::from_min_size(
                Pos2::new(scope.viewport.min.x, y),
                Vec2::new(scope.viewport.width(), height),
            )
        };

        let mut child = ui.new_child(UiBuilder::new().max_rect(content_rect));
        child.set_clip_rect(scope.viewport);
        let inner = content(&mut child);
        let measured_height = child.min_rect().height();
        let mut scratch: ScopeScratch = ctx
            .data_mut(|d| d.get_temp::<ScopeScratch>(scratch_id))
            .unwrap_or_default();
        if scratch.sizes.len() <= index {
            scratch.sizes.resize(index + 1, 0.0);
        }
        scratch.sizes[index] = measured_height.max(1.0);
        ctx.data_mut(|d| d.insert_temp(scratch_id, scratch));
        Some(inner)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    // --- Layout math ----------------------------------------------------------

    #[test]
    fn horizontal_offsets_are_fixed_pitch() {
        let sizes = vec![50.0, 80.0, 30.0];
        let offsets = slide_offsets(&sizes, true, 200.0);
        assert_eq!(offsets, vec![0.0, 200.0, 400.0]);
        assert_eq!(strip_length(&sizes, true, 200.0), 600.0);
    }

    #[test]
    fn vertical_offsets_stack_with_gap() {
        let sizes = vec![50.0, 80.0, 30.0];
        let offsets = slide_offsets(&sizes, false, 0.0);
        assert_eq!(offsets, vec![0.0, 66.0, 162.0]);
        // total = sizes + 3 gaps
        assert_eq!(strip_length(&sizes, false, 0.0), 50.0 + 80.0 + 30.0 + 48.0);
    }

    #[test]
    fn max_offset_clamps_to_content() {
        // Horizontal: 3 slides of 200 in a 200 viewport → 2 pitches.
        let sizes = vec![50.0, 80.0, 30.0];
        assert_eq!(max_offset(&sizes, true, 200.0, 200.0), 400.0);
        // Vertical: last slide flush = 162, but the viewport (200) is taller
        // than the content tail allows — clamped to total(208) - viewport(200).
        assert_eq!(max_offset(&sizes, false, 0.0, 200.0), 8.0);
        // Vertical: viewport shorter than the tail → last slide flush.
        assert_eq!(max_offset(&sizes, false, 0.0, 100.0), 108.0); // total 208 - 100 = 108 < 162
    }

    #[test]
    fn max_offset_never_negative() {
        let sizes = vec![50.0];
        assert_eq!(max_offset(&sizes, true, 200.0, 200.0), 0.0);
    }

    #[test]
    fn nearest_slide_picks_closest() {
        let offsets = vec![0.0, 200.0, 400.0];
        assert_eq!(nearest_slide(&offsets, 0.0), 0);
        assert_eq!(nearest_slide(&offsets, 99.0), 0);
        assert_eq!(nearest_slide(&offsets, 101.0), 1);
        assert_eq!(nearest_slide(&offsets, 320.0), 2);
        assert_eq!(nearest_slide(&offsets, 1000.0), 2);
        assert_eq!(nearest_slide(&[], 12.0), 0);
    }

    #[test]
    fn snap_animation_eases_and_finishes() {
        let anim = ScrollAnim {
            from: 0.0,
            to: 200.0,
            start_secs: 0.0,
        };
        let (mid, _) = anim.offset_at(0.125);
        assert!((mid - 100.0).abs() < 60.0, "eased mid-value, got {mid}");
        let (start, done) = anim.offset_at(0.0);
        assert_eq!(start, 0.0);
        assert!(!done);
        let (end, done) = anim.offset_at(0.25);
        assert_eq!(end, 200.0);
        assert!(done);
        let (past, done) = anim.offset_at(2.0);
        assert_eq!(past, 200.0);
        assert!(done);
    }

    // --- Rendering -------------------------------------------------------------

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

        fn frame(&mut self, events: Vec<egui::Event>) {
            self.time += 0.1;
            let input = egui::RawInput {
                time: Some(self.time),
                events,
                ..Default::default()
            };
            let mut out = self.ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    Id::unique("carousel_test_ui"),
                    UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(600.0, 800.0))),
                );
                Carousel::new().id_salt("c").show(ui, |ui| {
                    for i in 0..3 {
                        CarouselItem::new().show(ui, |ui| {
                            ui.allocate_space(Vec2::new(200.0, 100.0 + 10.0 * i as f32));
                        });
                    }
                });
            });
            out.textures_delta.clear();
        }

        fn rect(&self, key: &str) -> Option<Rect> {
            let id = Id::unique("carousel_test_ui").with("c").with(key);
            self.ctx.read_response(id).map(|r| r.rect)
        }

        fn state(&self) -> CarouselState {
            self.ctx
                .data_mut(|d| d.get_temp::<CarouselState>(Id::unique("carousel_test_ui").with("c")))
                .unwrap_or_default()
        }

        fn click(&mut self, pos: Pos2) {
            for pressed in [true, false] {
                self.frame(vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: egui::Modifiers::default(),
                }]);
            }
        }
    }

    #[test]
    fn renders_and_records_slide_sizes() {
        let mut h = Harness::new();
        h.frame(vec![]);
        h.frame(vec![]);
        let st = h.state();
        assert_eq!(st.item_sizes, vec![100.0, 110.0, 120.0], "heights measured");
        assert!(
            (st.viewport_cross - 120.0).abs() < f32::EPSILON,
            "tallest slide"
        );
        assert_eq!(st.selected, 0);
    }

    #[test]
    fn next_arrow_snaps_to_slide_one() {
        let mut h = Harness::new();
        h.frame(vec![]);
        h.frame(vec![]);
        let next = h.rect("arrow_next").expect("next arrow rendered");
        h.click(next.center());
        // Let the snap animation finish.
        for _ in 0..4 {
            h.frame(vec![]);
        }
        let st = h.state();
        assert_eq!(st.selected, 1);
        // Pitch = viewport width = 600 - 2×48 = 504.
        assert!(
            (st.offset - 504.0).abs() < 1.0,
            "offset snapped to pitch, got {}",
            st.offset
        );
    }

    #[test]
    fn drag_then_release_snaps_to_nearest() {
        let mut h = Harness::new();
        h.frame(vec![]);
        h.frame(vec![]);
        let viewport = h.rect("viewport").expect("viewport rendered");
        let start = Pos2::new(viewport.center().x, viewport.center().y);
        // Press and drag left by 300px total — more than half the 504px
        // pitch, so the nearest slide is 1.
        h.frame(vec![egui::Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        }]);
        for step in 1..=5 {
            h.frame(vec![egui::Event::PointerMoved(Pos2::new(
                start.x - 60.0 * step as f32,
                start.y,
            ))]);
        }
        h.frame(vec![egui::Event::PointerButton {
            pos: Pos2::new(start.x - 300.0, start.y),
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        }]);
        for _ in 0..4 {
            h.frame(vec![]);
        }
        let st = h.state();
        assert_eq!(st.selected, 1, "dragged 300px of a 504px pitch → slide 1");
        assert!(
            (st.offset - 504.0).abs() < 1.0,
            "snapped, got {}",
            st.offset
        );
    }

    #[test]
    fn dots_click_jumps_to_slide() {
        let mut h = Harness::new();
        h.frame(vec![]);
        h.frame(vec![]);
        let dot_id = Id::unique("carousel_test_ui").with("c").with(("dot", 2));
        let Some(dot) = h.ctx.read_response(dot_id).map(|r| r.rect) else {
            panic!("dot 2 rendered");
        };
        h.click(dot.center());
        for _ in 0..4 {
            h.frame(vec![]);
        }
        let st = h.state();
        assert_eq!(st.selected, 2);
    }

    #[test]
    fn builders_keep_fields() {
        let c = Carousel::new()
            .id_salt("c1")
            .orientation(CarouselOrientation::Vertical)
            .show_arrows(false)
            .show_dots(false);
        assert_eq!(c.id_salt.as_deref(), Some("c1"));
        assert_eq!(c.orientation, CarouselOrientation::Vertical);
        assert!(!c.show_arrows);
        assert!(!c.show_dots);
    }

    #[test]
    fn constants_match_source() {
        // pl-4 = 16, -left-12 = 48, size-8 = 32, h-2 = 8.
        assert_eq!(CAROUSEL_ITEM_GAP, 16.0);
        assert_eq!(CAROUSEL_ARROW_GUTTER, 48.0);
        assert_eq!(CAROUSEL_ARROW_SIZE, 32.0);
        assert_eq!(CAROUSEL_DOT_SIZE, 8.0);
    }

    #[test]
    fn palettes_resolve_for_both_modes() {
        let _ = Preset::default().build(Mode::Light).palette;
        let _ = Preset::default().build(Mode::Dark).palette;
    }
}
