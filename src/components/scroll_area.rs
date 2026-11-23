//! shadcn `scroll-area.tsx` — a themed scrolling container.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/scroll-area.tsx`: a
//! `ScrollArea` root with a viewport and a `ScrollBar` whose thumb is
//! `rounded-full bg-border` and whose track is `w-2.5` (10px) with `p-px`
//! padding.
//!
//! The scrollbar is egui's *floating* bar, so it overlays the viewport and
//! takes no layout width (N14a — the viewport is `size-full`), and its
//! outer edge is flush with the Root edge (`bar_outer_margin = 0`, N14b).
//! egui's handle spans the bar's full cross range and cannot express the
//! source's `p-px` thumb inset, so the handle is made invisible (all handle
//! opacities 0) and the thumb is painted by this module instead: 8px wide,
//! 1px inside the 10px bar on every side, radius 4 (`rounded-full`),
//! `bg-border` in every state — the source defines no thumb hover state.
//! Drag and track-click paging still run on egui's bar, whose geometry the
//! painted thumb mirrors exactly (see [`scrollbar_thumb_rect`]).
//!
//! The viewport is one Tab stop; on focus it paints the source's
//! `focus-visible` ring (3px `ring/50`) plus the 1px solid outline in
//! `ring/50`.
//!
//! The scrollbar look is applied through a *scoped* egui style
//! (`ScrollStyle::floating` + themed handle/track colors), so the global
//! style is never permanently mutated — nested `egui::ScrollArea`s created
//! inside the closure inherit it. The track is invisible
//! (`extreme_bg_color = TRANSPARENT`, the source's `border-l-transparent`);
//! the scoped widget `bg_fill`s stay `bg-border` in all three states (the
//! audit-verified P0 fix). They no longer paint the handle itself (it is
//! opacity-0) and are retained so content widgets inside the area keep
//! their current paint — removing them would change the paint of every
//! control nested in a scroll area, which is out of scope for this line.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! ScrollArea::new().height(200.0).show(ui, |ui| {
//!     for i in 0..50 {
//!         ui.label(format!("Row {i}"));
//!     }
//! });
//! ```

use crate::theme::ShadcnTheme;
use crate::utils::focus::{is_focused, paint_focus_ring, paint_ring};
use egui::{CornerRadius, InnerResponse, Rect, Ui};

/// The scrollbar track width (`w-2.5`).
pub const SCROLLBAR_WIDTH: f32 = 10.0;

/// The thumb width: the 10px bar minus `p-px` (1px) on each side.
pub const SCROLLBAR_THUMB_WIDTH: f32 = 8.0;

/// The thumb inset inside the bar (`p-px`, 1px on every side).
const THUMB_INSET: f32 = 1.0;

/// The thumb corner radius (`rounded-full` on the 8px thumb).
const THUMB_RADIUS: u8 = 4;

/// Minimum thumb length (egui `ScrollStyle::solid()` default).
/// OPEN-3: no source basis — the `.tsx` states only `flex-1`
/// (ratio-derived); never present this as spec truth.
const THUMB_MIN_LENGTH: f32 = 12.0;

/// A shadcn scroll area. See the [module docs](self) for an example.
#[derive(Clone, Debug)]
pub struct ScrollArea {
    vertical: bool,
    horizontal: bool,
    height: Option<f32>,
    width: Option<f32>,
    id_salt: Option<String>,
}

impl Default for ScrollArea {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollArea {
    /// Creates a vertical scroll area (the source default orientation).
    pub fn new() -> Self {
        Self {
            vertical: true,
            horizontal: false,
            height: None,
            width: None,
            id_salt: None,
        }
    }

    /// Enables/disables the vertical scrollbar.
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    /// Enables the horizontal scrollbar (`orientation="horizontal"`).
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }

    /// Caps the viewport height (default: available height).
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Fixes the viewport width (default: available width).
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Sets a stable id salt so scroll position persists per instance.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Builds the scoped scrollbar style.
    ///
    /// Floating bar (`N14a` — overlays the viewport, no layout width):
    /// `floating_allocated_width = 0`, `bar_width = 10.0` (`w-2.5`),
    /// `bar_outer_margin = 0` (`N14b` — flush with the Root edge),
    /// `floating_width = 10.0` (never shrinks to the thin width),
    /// `handle_min_length = 12.0` (OPEN-3, egui default, no source basis).
    ///
    /// egui's own handle is invisible in every state (all handle opacities
    /// 0): it spans the bar's full cross range and cannot express the
    /// source's `p-px` thumb inset, so S1 paints the thumb itself (8px,
    /// 1px inset, radius 4 — see [`scrollbar_thumb_rect`]). Interaction
    /// (drag, track-click paging) still runs on egui's bar.
    pub(crate) fn themed_scroll_style() -> egui::style::ScrollStyle {
        let mut s = egui::style::ScrollStyle::floating();
        s.bar_width = SCROLLBAR_WIDTH; // w-2.5
        s.bar_inner_margin = THUMB_INSET; // p-px (documents the inset; unused by floating cross placement)
        s.bar_outer_margin = 0.0; // N14b: flush with the Root edge
        s.handle_min_length = THUMB_MIN_LENGTH; // OPEN-3: egui default, no source basis
        s.floating_width = SCROLLBAR_WIDTH; // never shrink to the thin width
        s.floating_allocated_width = 0.0; // N14a: no layout width
        s.foreground_color = false; // handle uses the widget bg_fill (= border), as before
                                    // egui's handle is invisible; the thumb is painted by this module.
        s.dormant_handle_opacity = 0.0;
        s.active_handle_opacity = 0.0;
        s.interact_handle_opacity = 0.0;
        // The track is invisible (extreme_bg_color = TRANSPARENT, below);
        // the background opacities are 0 for the same reason.
        s.dormant_background_opacity = 0.0;
        s.active_background_opacity = 0.0;
        s.interact_background_opacity = 0.0;
        s
    }

    /// Applies the shadcn scrollbar colors to the scoped visuals.
    ///
    /// The track becomes invisible (the source only paints the thumb) and
    /// the handle uses the `bg-border` token in every widget state. The
    /// handle itself is painted invisible by the opacities in
    /// [`Self::themed_scroll_style`] (egui's handle cannot express the
    /// `p-px` inset); these `bg_fill`s are retained so content widgets
    /// inside the area keep their current paint — see the module docs.
    pub(crate) fn themed_scroll_visuals(visuals: &mut egui::Visuals, border: egui::Color32) {
        visuals.extreme_bg_color = egui::Color32::TRANSPARENT;
        visuals.widgets.inactive.bg_fill = border;
        visuals.widgets.hovered.bg_fill = border;
        visuals.widgets.active.bg_fill = border;
    }

    /// Renders the scroll area around its contents.
    pub fn show<R>(self, ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;

        let outer = ui.scope(|ui| {
            // Scoped style: only scroll areas created inside inherit it.
            ui.style_mut().spacing.scroll = Self::themed_scroll_style();
            // egui 0.37 paints the solid track with `extreme_bg_color` and
            // the handle with the widget `bg_fill`s; retarget them so the
            // track disappears and the thumb is `bg-border`, rounded-full.
            Self::themed_scroll_visuals(&mut ui.style_mut().visuals, p.border);

            let mut area =
                egui::ScrollArea::new([self.horizontal, self.vertical]).auto_shrink([false, false]);
            if let Some(h) = self.height {
                area = area.max_height(h);
            }
            if let Some(salt) = self.id_salt.as_deref() {
                area = area.id_salt(ui.scope_id().with(salt));
            }
            let inner = area.show(ui, |ui| {
                if let Some(w) = self.width {
                    ui.set_min_width(w);
                    ui.set_max_width(w);
                }
                content(ui)
            });

            // Paint the thumbs. egui's own handle is invisible in every
            // state (opacities 0 in `themed_scroll_style`); the source
            // thumb is 8px wide, 1px inside the 10px bar on every side,
            // radius 4, `bg-border` in all states (no hover state, §4.2).
            // Drag and track-click paging run on egui's bar, whose geometry
            // this mirrors exactly.
            let offset = inner.state.clamped_offset();
            let clip = ui.clip_rect();
            if self.vertical {
                if let Some(thumb) =
                    scrollbar_thumb_rect(true, inner.inner_rect, inner.content_size, offset, clip)
                {
                    ui.painter()
                        .rect_filled(thumb, CornerRadius::same(THUMB_RADIUS), p.border);
                }
            }
            if self.horizontal {
                if let Some(thumb) =
                    scrollbar_thumb_rect(false, inner.inner_rect, inner.content_size, offset, clip)
                {
                    ui.painter()
                        .rect_filled(thumb, CornerRadius::same(THUMB_RADIUS), p.border);
                }
            }

            // The viewport is one Tab stop (spec §7): focusable but not
            // interactive, so it never steals clicks from the content
            // (dialog.rs precedent). egui 0.37's ScrollAreaOutput has no
            // Response; synthesize one over the visible inner rect.
            // On focus, paint the source's `focus-visible` ring (3px
            // `ring/50`) plus the 1px solid outline in `ring/50`
            // (`globals.css:193`), just outside the ring. D2: egui cannot
            // distinguish keyboard from pointer focus, so the ring shows on
            // any focus.
            let response =
                ui.allocate_rect(inner.inner_rect, egui::Sense::focusable_noninteractive());
            if is_focused(&response) {
                paint_focus_ring(ui, &response, 0.0, true);
                let ring = crate::theme::color_alpha(p.ring, 0.5);
                paint_ring(ui.painter(), response.rect.expand(4.0), ring, 1.0, 0.0);
            }
            InnerResponse::new(inner.inner, response)
        });
        outer.inner
    }
}

/// Computes the thumb rect for one axis, mirroring egui 0.37's
/// `calculate_handle_rect` (`containers/scroll_area.rs`) with the source's
/// `p-px` inset applied: the thumb is 8px wide inside the 10px bar
/// (`rounded-full` → radius 4), 1px inside the bar on every side.
///
/// `viewport` is the scroll area's inner rect (== the Root rect for the
/// floating bar); `content_size` / `offset` are the along-axis content size
/// and clamped scroll offset from the `ScrollAreaOutput`; `clip_rect` is
/// the outer ui's clip rect — egui clamps the bar so it stays visible when
/// the area is scrolled out of view in a parent (spec §9 item 3).
///
/// Returns `None` when the content fits (no bar), matching egui's
/// `content_is_too_large` check.
pub(crate) fn scrollbar_thumb_rect(
    vertical: bool,
    viewport: Rect,
    content_size: egui::Vec2,
    offset: egui::Vec2,
    clip_rect: Rect,
) -> Option<Rect> {
    let d = usize::from(vertical); // 1 = y (vertical), 0 = x (horizontal)
    let along = viewport.size()[d];
    let content = content_size[d];
    // egui: `limit_rect.size()[d].ceil() < content_size[d]`
    if along.ceil() >= content {
        return None;
    }
    let max_offset = content - along;
    // egui: `max_cross = min(outer_rect.max, clip_rect.max) - outer_margin`
    // (outer_margin is 0 — N14b flush; outer_rect == viewport for floating).
    let cross_max = viewport.max[1 - d].min(clip_rect.max[1 - d]);
    let bar_min_cross = cross_max - SCROLLBAR_WIDTH;
    // Along-axis bar range (egui's `scroll_bar_rect` defaults to inner_rect).
    let bar_min = viewport.min[d];
    let bar_max = viewport.max[d];
    let from_content = |c: f32| egui::emath::remap_clamp(c, 0.0..=content, bar_min..=bar_max);
    let handle_size =
        (from_content(offset[d] + along) - from_content(offset[d])).max(THUMB_MIN_LENGTH);
    let handle_start = egui::emath::remap_clamp(
        offset[d],
        0.0..=max_offset,
        bar_min..=(bar_max - handle_size),
    );
    let mut min = viewport.min;
    let mut max = viewport.max;
    min[1 - d] = bar_min_cross + THUMB_INSET;
    max[1 - d] = cross_max - THUMB_INSET;
    min[d] = handle_start;
    max[d] = handle_start + handle_size;
    Some(Rect::from_min_max(min, max))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};
    use egui::{Pos2, Rect, Vec2};

    #[test]
    fn scrollbar_style_matches_source() {
        let s = ScrollArea::themed_scroll_style();
        assert!(s.floating); // N14a: overlay — no layout width
        assert_eq!(s.bar_width, 10.0); // w-2.5
        assert_eq!(s.bar_inner_margin, 1.0); // p-px
        assert_eq!(s.bar_outer_margin, 0.0); // N14b: flush with the Root edge
        assert_eq!(s.floating_width, 10.0); // never shrinks to the thin width
        assert_eq!(s.floating_allocated_width, 0.0); // N14a: no layout width
        assert_eq!(s.handle_min_length, 12.0); // OPEN-3: egui default, no source basis
                                               // egui's own handle is invisible in every state; S1 paints the
                                               // thumb itself (see `scrollbar_thumb_rect`).
        assert_eq!(s.dormant_handle_opacity, 0.0);
        assert_eq!(s.active_handle_opacity, 0.0);
        assert_eq!(s.interact_handle_opacity, 0.0);
        assert_eq!(s.dormant_background_opacity, 0.0);
        assert_eq!(s.active_background_opacity, 0.0);
        assert_eq!(s.interact_background_opacity, 0.0);
    }

    #[test]
    fn scrollbar_thumb_rect_matches_source() {
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));
        let clip = viewport; // unclipped
                             // Vertical, content twice the viewport, offset 0: thumb is the top
                             // half of the bar, 8px wide, 1px inside the bar on every side.
        let thumb = scrollbar_thumb_rect(true, viewport, Vec2::new(300.0, 600.0), Vec2::ZERO, clip)
            .expect("bar when content overflows");
        assert_eq!(thumb.width(), SCROLLBAR_THUMB_WIDTH);
        assert_eq!(thumb.min.x, 300.0 - SCROLLBAR_WIDTH + THUMB_INSET);
        assert_eq!(thumb.max.x, 300.0 - THUMB_INSET);
        assert_eq!(thumb.min.y, 0.0);
        assert_eq!(thumb.height(), 150.0); // half the content visible -> half the bar
                                           // Scrolled to the bottom: the thumb sits at the bar's bottom end.
        let thumb = scrollbar_thumb_rect(
            true,
            viewport,
            Vec2::new(300.0, 600.0),
            Vec2::new(0.0, 300.0),
            clip,
        )
        .expect("bar when content overflows");
        assert_eq!(thumb.max.y, 300.0);
        assert_eq!(thumb.height(), 150.0);
        // Content fits: no thumb.
        assert!(
            scrollbar_thumb_rect(true, viewport, Vec2::new(300.0, 300.0), Vec2::ZERO, clip)
                .is_none()
        );
        // Horizontal mirror: 8px tall, 1px inside the bar top/bottom.
        let thumb =
            scrollbar_thumb_rect(false, viewport, Vec2::new(600.0, 300.0), Vec2::ZERO, clip)
                .expect("bar when content overflows");
        assert_eq!(thumb.height(), SCROLLBAR_THUMB_WIDTH);
        assert_eq!(thumb.min.y, 300.0 - SCROLLBAR_WIDTH + THUMB_INSET);
        assert_eq!(thumb.max.y, 300.0 - THUMB_INSET);
        assert_eq!(thumb.width(), 150.0);
        // Clipped viewport: the bar moves inward so it stays visible
        // (egui's "move the scrollbar so it is visible" clamp).
        let clip = Rect::from_min_size(Pos2::ZERO, Vec2::new(280.0, 300.0));
        let thumb = scrollbar_thumb_rect(true, viewport, Vec2::new(300.0, 600.0), Vec2::ZERO, clip)
            .expect("bar when content overflows");
        assert_eq!(thumb.max.x, 280.0 - THUMB_INSET);
    }

    #[test]
    fn scrollbar_visuals_match_source() {
        // The source has no visible track (`border-l-transparent`) and a
        // `rounded-full bg-border` thumb; egui paints the track with
        // `extreme_bg_color` and the handle with the widget `bg_fill`s.
        let ctx = egui::Context::default();
        let preset = Preset::default();
        preset.build(Mode::Light).install(&ctx);
        let border = preset.build(Mode::Light).palette.border;
        let mut captured: Option<egui::Visuals> = None;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("scroll_area_visuals_test"),
                egui::UiBuilder::new(),
            );
            ScrollArea::new().show(ui, |ui| {
                captured = Some(ui.style().visuals.clone());
            });
        });
        out.textures_delta.clear();
        let v = captured.expect("scroll area content ran");
        assert_eq!(v.extreme_bg_color, egui::Color32::TRANSPARENT);
        assert_eq!(v.widgets.inactive.bg_fill, border);
        assert_eq!(v.widgets.hovered.bg_fill, border);
        assert_eq!(v.widgets.active.bg_fill, border);
        // The default (unscoped) style must not be mutated.
        let global = ctx.style_of(egui::Theme::Light).visuals.clone();
        assert_ne!(global.extreme_bg_color, egui::Color32::TRANSPARENT);
    }

    #[test]
    fn scrollbar_overlays_content() {
        // N14a: a vertical scroll area with overflowing content in a
        // 300px-wide Ui — the viewport keeps the full 300px because the bar
        // overlays it and takes no layout width. With the old solid bar the
        // content width was 288px (300 - 10 bar - 2 margins).
        let ctx = egui::Context::default();
        Preset::default().build(Mode::Light).install(&ctx);
        let mut content_width = None;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("scroll_area_overlay_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
            );
            ScrollArea::new().height(120.0).show(ui, |ui| {
                content_width = Some(ui.available_width());
                for i in 0..20 {
                    ui.label(format!("Row {i}"));
                }
            });
        });
        out.textures_delta.clear();
        assert_eq!(content_width, Some(300.0));
    }

    #[test]
    fn thumb_painted_8px_wide() {
        // Painted-shape closing evidence for the 8px thumb (D4): exactly
        // one border-filled rect, 8px wide, 1px inside the bar's right edge,
        // radius 4; no 10px-wide border-filled rect (the old egui handle).
        let ctx = egui::Context::default();
        let preset = Preset::default();
        preset.build(Mode::Light).install(&ctx);
        let border = preset.build(Mode::Light).palette.border;
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                egui::Id::unique("scroll_area_thumb_test"),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
            );
            ScrollArea::new()
                .height(120.0)
                .id_salt("thumb")
                .show(ui, |ui| {
                    for i in 0..20 {
                        ui.label(format!("Row {i}"));
                    }
                });
        });
        out.textures_delta.clear();
        let fills: Vec<(Rect, CornerRadius)> = out
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.fill == border => Some((rs.rect, rs.corner_radius)),
                _ => None,
            })
            .collect();
        assert_eq!(fills.len(), 1, "thumb fills: {fills:?}");
        let (thumb, radius) = fills[0];
        assert_eq!(thumb.width(), SCROLLBAR_THUMB_WIDTH);
        assert_eq!(thumb.min.x, 300.0 - SCROLLBAR_WIDTH + THUMB_INSET);
        assert_eq!(thumb.max.x, 300.0 - THUMB_INSET);
        assert_eq!(thumb.min.y, 0.0);
        assert_eq!(radius, CornerRadius::same(THUMB_RADIUS));
    }

    #[test]
    fn viewport_focus_ring_paints_on_focus() {
        // The viewport is one Tab stop (spec §7); on focus it paints the
        // 3px `ring/50` ring plus the 1px solid outline in `ring/50`.
        let ctx = egui::Context::default();
        let preset = Preset::default();
        preset.build(Mode::Light).install(&ctx);
        let ring50 = crate::theme::color_alpha(preset.build(Mode::Light).palette.ring, 0.5);

        let frame = |ctx: &egui::Context, input: egui::RawInput| {
            let mut id = egui::Id::NULL;
            let mut out = ctx.run_ui(input, |ctx| {
                let ui = &mut egui::Ui::new(
                    ctx.clone(),
                    egui::Id::unique("scroll_area_focus_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                id = ScrollArea::new()
                    .height(120.0)
                    .id_salt("focus_ring")
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                    })
                    .response
                    .id;
            });
            out.textures_delta.clear();
            (id, out.shapes)
        };

        // Frame 1: render, record the viewport response id; then focus it.
        let (id, _) = frame(&ctx, egui::RawInput::default());
        ctx.memory_mut(|m| m.request_focus(id));
        assert!(ctx.memory(|m| m.has_focus(id)));

        // Frame 2: the focused viewport paints the ring + the outline.
        let (_, shapes) = frame(&ctx, egui::RawInput::default());
        let ring_strokes: Vec<(Rect, egui::Stroke)> = shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                egui::Shape::Rect(rs) if rs.stroke.color == ring50 => Some((rs.rect, rs.stroke)),
                _ => None,
            })
            .collect();
        assert!(
            ring_strokes.iter().any(|(_, s)| s.width == 3.0),
            "3px focus ring in ring/50 painted; ring strokes: {ring_strokes:?}"
        );
        assert!(
            ring_strokes.iter().any(|(_, s)| s.width == 1.0),
            "1px focus outline in ring/50 painted; ring strokes: {ring_strokes:?}"
        );
        // The ring sits just outside the viewport rect (expanded by 1px,
        // 3px stroke outside); the outline sits just outside the ring.
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 120.0));
        assert!(
            ring_strokes
                .iter()
                .any(|(r, s)| s.width == 3.0 && *r == viewport.expand(1.0)),
            "ring rect is the viewport expanded by 1px; ring strokes: {ring_strokes:?}"
        );
        assert!(
            ring_strokes
                .iter()
                .any(|(r, s)| s.width == 1.0 && *r == viewport.expand(4.0)),
            "outline rect is the viewport expanded by 4px; ring strokes: {ring_strokes:?}"
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
                    egui::Id::unique("scroll_area_render_test"),
                    egui::UiBuilder::new()
                        .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0))),
                );
                ScrollArea::new()
                    .height(120.0)
                    .id_salt("render_scroll")
                    .show(ui, |ui| {
                        for i in 0..20 {
                            ui.label(format!("Row {i}"));
                        }
                    });
                ScrollArea::new()
                    .horizontal(true)
                    .vertical(false)
                    .height(60.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            for i in 0..30 {
                                ui.label(format!("Item {i}"));
                            }
                        });
                    });
            });
            out.textures_delta.clear();
        }
    }
}
