//! Real Lucide icons, rasterized once and cached as textures.
//!
//! Every icon's geometry is REAL Lucide data from the official Lucide
//! release (see [`data`] and `LICENSE-Lucide.txt`), converted to [`El`]
//! values by `gen_icons.py` — nothing here is hand-drawn. Icons live on
//! the Lucide 24×24 grid with 2px strokes and round caps/joins.
//!
//! Painting is cached: [`Icon::paint`] rasterizes the icon once per
//! (icon, pixel size, stroke width) into a color-independent alpha mask
//! stored as an egui texture in `Context` temp data, then draws it tinted
//! with the requested color. Cache hits allocate nothing per frame.
//! The cache is bounded (LRU, [`MAX_CACHE_ENTRIES`]); dropping an entry
//! drops its texture.
//!
//! To add an icon, add one line to `ICONS` in `gen_icons.py`, add the
//! variant to [`Icon`], and regenerate (command in `data.rs`).

use std::collections::{HashMap, VecDeque};

use egui::{
    Color32, ColorImage, Context, Painter, Pos2, Rect, Response, TextureHandle, TextureOptions, Ui,
    Vec2,
};

use super::data;

/// One drawable element of an icon, in 24×24 icon coordinates.
///
/// This is the icon geometry vocabulary: generated icon data is a static
/// slice of `El` values (see [`Icon::elements_static`]). [`Icon::elements`]
/// keeps the historical owned-`Vec` form for compatibility.
#[derive(Clone, Copy, Debug)]
pub enum El {
    /// Straight line.
    Line([f32; 2], [f32; 2]),
    /// Open polyline.
    Poly(&'static [[f32; 2]]),
    /// Closed polygon (stroked, like lucide's closed paths).
    PolyClosed(&'static [[f32; 2]]),
    /// Stroked circle.
    Circle([f32; 2], f32),
    /// Filled circle (lucide "dots").
    Dot([f32; 2], f32),
    /// Stroked arc, angles in degrees (0° = +x axis, clockwise positive
    /// because egui's y axis points down).
    Arc([f32; 2], f32, f32, f32),
    /// Open cubic bezier sampled into a polyline.
    Cubic([f32; 2], [f32; 2], [f32; 2], [f32; 2]),
    /// Rounded rectangle (x, y, w, h, radius).
    Rect([f32; 2], [f32; 2], f32),
}

/// A Lucide icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Check,
    ChevronDown,
    ChevronUp,
    ChevronLeft,
    ChevronRight,
    ChevronsLeft,
    ChevronsRight,
    ChevronsUpDown,
    X,
    Minus,
    Plus,
    CircleAlert,
    CircleCheck,
    CircleX,
    TriangleAlert,
    OctagonX,
    Info,
    LoaderCircle,
    Calendar,
    CalendarDays,
    Search,
    MoreHorizontal,
    MoreVertical,
    ArrowDown,
    ArrowUp,
    ArrowUpDown,
    ArrowLeft,
    ArrowRight,
    PanelLeft,
    User,
    Paperclip,
    Trash,
    Pencil,
    Sparkles,
    Copy,
    CornerDownLeft,
    Message,
    Send,
    Circle,
    Dot,
    GripVertical,
    GripHorizontal,
    FileText,
    RefreshCw,
    Clock,
}

impl Icon {
    /// The elements composing this icon in 24×24 coordinates.
    ///
    /// Kept for compatibility; prefer [`Icon::elements_static`], which
    /// returns the same data without allocating.
    pub fn elements(&self) -> Vec<El> {
        self.elements_static().to_vec()
    }

    /// The elements composing this icon, as static data (no allocation).
    pub fn elements_static(&self) -> &'static [El] {
        data::elements(*self)
    }

    /// Every icon in the crate, in enum order.
    pub fn all() -> &'static [Icon] {
        data::ALL
    }

    /// The Lucide kebab-case name this icon's data comes from
    /// (e.g. `Icon::CircleAlert` → `"circle-exclamation-point"`).
    pub fn lucide_name(&self) -> &'static str {
        data::lucide_name(*self)
    }

    /// Look up an icon by its Lucide kebab-case name.
    pub fn from_lucide_name(name: &str) -> Option<Icon> {
        data::from_lucide_name(name)
    }

    /// Paints the icon scaled into `rect`, tinted with `color`.
    ///
    /// The icon is rasterized once per (icon, pixel size, stroke width)
    /// and cached as a texture; repainting the same icon at the same size
    /// only draws the cached texture tinted with `color`. The icon is
    /// centered in `rect` using its smaller side, exactly as before.
    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32, stroke_width: f32) {
        let size = rect.width().min(rect.height());
        if !size.is_finite() || size <= 0.0 {
            return;
        }
        let ctx = painter.ctx();
        let pixels_per_point = ctx.pixels_per_point().max(0.1);
        let px = ((size * pixels_per_point).round() as u32).clamp(1, MAX_TEXTURE_PX);
        // Same minimum-stroke rule as the old vector paint (1 logical point).
        let stroke_px = stroke_width.max(1.0) * pixels_per_point;
        let key = IconCacheKey {
            icon: *self as u16,
            px,
            stroke_q: (stroke_px * 64.0).round() as u32,
        };
        let handle = cached_texture(ctx, key, || {
            rasterize(
                self.elements_static(),
                px as usize,
                stroke_px,
                0.5 * pixels_per_point,
            )
        });
        let icon_size = Vec2::splat(size);
        let origin = rect.min + 0.5 * (rect.size() - icon_size);
        let draw_rect = Rect::from_min_size(origin, icon_size);
        painter.image(
            handle.id(),
            draw_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            color,
        );
    }

    /// Paints centered in the current layout with the theme foreground color.
    pub fn show(&self, ui: &mut Ui, size: f32) -> Response {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
        let theme = crate::theme::ShadcnTheme::of_ui(ui);
        self.paint(
            ui.painter(),
            rect,
            theme.palette.foreground,
            2.0 * size / 24.0,
        );
        response
    }
}

/// Maximum cached texture side in physical pixels. Icons are small; this
/// only guards against pathological rects (a 512px mask drawn larger
/// still looks smooth under linear filtering).
const MAX_TEXTURE_PX: u32 = 512;

/// Maximum cached icon textures per context. A 16px icon mask is 1 KiB
/// (32px at 2× DPI is 4 KiB), so the realistic worst case is ~1 MiB.
const MAX_CACHE_ENTRIES: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct IconCacheKey {
    /// `Icon` discriminant (declaration order; stable within a build).
    icon: u16,
    /// Texture side in physical pixels (icons are square).
    px: u32,
    /// Stroke width in physical pixels, quantized to 1/64 px.
    stroke_q: u32,
}

/// LRU texture cache. Stored in `Context` temp data via
/// [`Context::data_mut`]; never serialized. `Clone` is required by the
/// `IdTypeMap` API (only `&mut` access is used on the hot path, so no
/// clone ever happens per frame).
#[derive(Clone)]
struct IconTextureCache {
    entries: HashMap<IconCacheKey, TextureHandle>,
    /// Least-recently-used first; every lookup/insert moves the key to the back.
    order: VecDeque<IconCacheKey>,
    max_entries: usize,
}

impl Default for IconTextureCache {
    fn default() -> Self {
        Self::new(MAX_CACHE_ENTRIES)
    }
}

impl IconTextureCache {
    fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            order: VecDeque::new(),
            max_entries: max_entries.max(1),
        }
    }

    fn lookup(&mut self, key: IconCacheKey) -> Option<TextureHandle> {
        let handle = self.entries.get(&key)?.clone();
        self.touch(key);
        Some(handle)
    }

    fn insert(&mut self, key: IconCacheKey, handle: TextureHandle) {
        if self.entries.insert(key, handle).is_some() {
            self.touch(key);
            return;
        }
        // Evict least-recently-used entries until the new one fits. The new
        // key is not in `order` yet, so it can never evict itself.
        while self.entries.len() > self.max_entries {
            if let Some(old) = self.order.pop_front() {
                // Dropping the handle frees the GPU texture.
                self.entries.remove(&old);
            } else {
                break;
            }
        }
        self.order.push_back(key);
    }

    fn touch(&mut self, key: IconCacheKey) {
        if let Some(pos) = self.order.iter().position(|k| *k == key) {
            self.order.remove(pos);
        }
        self.order.push_back(key);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.len()
    }
}

fn cache_id() -> egui::Id {
    egui::Id::unique("egui_shadcn_icon_texture_cache")
}

/// Look up (or rasterize + upload) the texture for one key. Lock discipline:
/// the `Context` data lock is held only for the map lookup/insert;
/// rasterization and texture upload happen outside it (`load_texture` takes
/// its own locks and must never run inside `data_mut`).
fn cached_texture(
    ctx: &Context,
    key: IconCacheKey,
    make: impl FnOnce() -> ColorImage,
) -> TextureHandle {
    if let Some(handle) = ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<IconTextureCache>(cache_id())
            .lookup(key)
    }) {
        return handle;
    }
    let image = make();
    let handle = ctx.load_texture("egui-shadcn icon", image, TextureOptions::LINEAR);
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<IconTextureCache>(cache_id())
            .insert(key, handle.clone());
    });
    handle
}

// --- Rasterizer ---
//
// Icons are rendered into a grayscale alpha mask with a distance-field
// stroke: every element flattens to line segments (in pixel space), and
// each pixel's coverage is its distance to the nearest segment, giving
// round caps/joins and a 1px antialiased edge for free. The mask is
// color-independent, so one texture serves every tint.

/// A flattened stroke segment in pixel coordinates.
type Segment = ([f32; 2], [f32; 2]);

/// A filled disc in pixel coordinates.
struct Disc {
    center: [f32; 2],
    radius: f32,
}

fn flatten(elements: &[El], scale: f32) -> (Vec<Segment>, Vec<Disc>) {
    let mut segments = Vec::new();
    let mut discs = Vec::new();
    let to_px = |p: [f32; 2]| [p[0] * scale, p[1] * scale];
    let push_polyline = |pts: &[[f32; 2]], closed: bool, segments: &mut Vec<Segment>| {
        let pts: Vec<[f32; 2]> = pts.iter().map(|&p| to_px(p)).collect();
        for w in pts.windows(2) {
            segments.push((w[0], w[1]));
        }
        if closed && pts.len() > 1 {
            let (first, last) = (pts[0], pts[pts.len() - 1]);
            // Skip the closing segment when the polyline already ends
            // where it started (avoids a zero-length segment).
            if (first[0] - last[0]).abs() > 1e-6 || (first[1] - last[1]).abs() > 1e-6 {
                segments.push((last, first));
            }
        }
    };
    for el in elements {
        match *el {
            El::Line(a, b) => segments.push((to_px(a), to_px(b))),
            El::Poly(pts) => push_polyline(pts, false, &mut segments),
            El::PolyClosed(pts) => push_polyline(pts, true, &mut segments),
            El::Circle(c, r) => {
                let pts: Vec<[f32; 2]> = (0..64)
                    .map(|i| {
                        let t = std::f32::consts::TAU * i as f32 / 64.0;
                        [c[0] + r * t.cos(), c[1] + r * t.sin()]
                    })
                    .collect();
                push_polyline(&pts, true, &mut segments);
            }
            El::Dot(c, r) => discs.push(Disc {
                center: to_px(c),
                radius: r * scale,
            }),
            El::Arc(c, r, start_deg, end_deg) => {
                let span = (end_deg - start_deg).abs().max(1.0);
                let steps = ((span / 4.0).ceil() as usize).max(2);
                let pts: Vec<[f32; 2]> = (0..=steps)
                    .map(|i| {
                        let t = start_deg + (end_deg - start_deg) * i as f32 / steps as f32;
                        let (sin, cos) = t.to_radians().sin_cos();
                        [c[0] + r * cos, c[1] + r * sin]
                    })
                    .collect();
                push_polyline(&pts, false, &mut segments);
            }
            El::Cubic(p0, p1, p2, p3) => {
                let mut pts = vec![p0];
                subdivide_cubic(p0, p1, p2, p3, 0.02, &mut pts);
                push_polyline(&pts, false, &mut segments);
            }
            El::Rect(pos, size, radius) => {
                push_polyline(
                    &rounded_rect_outline(pos, size, radius),
                    true,
                    &mut segments,
                );
            }
        }
    }
    (segments, discs)
}

fn subdivide_cubic(
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    tol: f32,
    out: &mut Vec<[f32; 2]>,
) {
    // Flatness: max distance of the control points from the chord.
    let (dx, dy) = (p3[0] - p0[0], p3[1] - p0[1]);
    let chord = dx.hypot(dy);
    let dev = if chord < 1e-9 {
        (p1[0] - p0[0])
            .hypot(p1[1] - p0[1])
            .max((p2[0] - p0[0]).hypot(p2[1] - p0[1]))
    } else {
        let d = |p: [f32; 2]| ((p[0] - p0[0]) * dy - (p[1] - p0[1]) * dx).abs() / chord;
        d(p1).max(d(p2))
    };
    if dev <= tol {
        out.push(p3);
        return;
    }
    let mid = |a: [f32; 2], b: [f32; 2]| [(a[0] + b[0]) / 2.0, (a[1] + b[1]) / 2.0];
    let m01 = mid(p0, p1);
    let m12 = mid(p1, p2);
    let m23 = mid(p2, p3);
    let m012 = mid(m01, m12);
    let m123 = mid(m12, m23);
    let m = mid(m012, m123);
    subdivide_cubic(p0, m01, m012, m, tol, out);
    subdivide_cubic(m, m123, m23, p3, tol, out);
}

/// Clockwise rounded-rectangle outline in grid coordinates, starting at the
/// top edge. With radius 0 this is just the four corners.
fn rounded_rect_outline(pos: [f32; 2], size: [f32; 2], radius: f32) -> Vec<[f32; 2]> {
    let (x, y, w, h) = (pos[0], pos[1], size[0], size[1]);
    let r = radius.min(w / 2.0).min(h / 2.0).max(0.0);
    if r <= 0.0 {
        return vec![[x, y], [x + w, y], [x + w, y + h], [x, y + h]];
    }
    let mut pts = Vec::with_capacity(36);
    // Corner arc helper: center + start angle (degrees, 0° = +x, clockwise).
    let arc = |cx: f32, cy: f32, start_deg: f32, pts: &mut Vec<[f32; 2]>| {
        for i in 1..=8 {
            let t = (start_deg + 90.0 * i as f32 / 8.0).to_radians();
            pts.push([cx + r * t.cos(), cy + r * t.sin()]);
        }
    };
    pts.push([x + r, y]);
    pts.push([x + w - r, y]);
    arc(x + w - r, y + r, -90.0, &mut pts);
    pts.push([x + w, y + h - r]);
    arc(x + w - r, y + h - r, 0.0, &mut pts);
    pts.push([x + r, y + h]);
    arc(x + r, y + h - r, 90.0, &mut pts);
    pts.push([x, y + r]);
    arc(x + r, y + r, 180.0, &mut pts);
    pts
}

fn dist_to_segment(px: f32, py: f32, a: [f32; 2], b: [f32; 2]) -> f32 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-12 {
        // Degenerate segment: distance to the point (renders as a disc).
        return (px - a[0]).hypot(py - a[1]);
    }
    let t = (((px - a[0]) * dx + (py - a[1]) * dy) / len_sq).clamp(0.0, 1.0);
    (px - (a[0] + t * dx)).hypot(py - (a[1] + t * dy))
}

/// Rasterize icon elements to a white-with-alpha mask image.
///
/// `px` is the texture side in physical pixels (the 24×24 viewBox maps to
/// `px`×`px`); `stroke_px` is the stroke width in physical pixels;
/// `dot_floor_px` is the minimum filled-dot radius (the historical 0.5
/// logical-point floor, in physical pixels).
fn rasterize(elements: &[El], px: usize, stroke_px: f32, dot_floor_px: f32) -> ColorImage {
    let px = px.max(1);
    let scale = px as f32 / 24.0;
    let (segments, discs) = flatten(elements, scale);
    let mut coverage = vec![0.0f32; px * px];
    let half = stroke_px / 2.0;
    for (a, b) in &segments {
        stamp_segment(&mut coverage, px, *a, *b, half);
    }
    for disc in &discs {
        stamp_disc(
            &mut coverage,
            px,
            disc.center,
            disc.radius.max(dot_floor_px),
        );
    }
    let mut rgba = Vec::with_capacity(px * px * 4);
    for cov in coverage {
        rgba.extend_from_slice(&[255, 255, 255, (cov.clamp(0.0, 1.0) * 255.0).round() as u8]);
    }
    ColorImage::from_rgba_unmultiplied([px, px], &rgba)
}

fn stamp_segment(buf: &mut [f32], px: usize, a: [f32; 2], b: [f32; 2], half: f32) {
    let pad = half + 1.0;
    let min_x = ((a[0].min(b[0]) - pad).floor() as i32).max(0) as usize;
    let max_x = ((a[0].max(b[0]) + pad).ceil() as i32).clamp(0, px as i32) as usize;
    let min_y = ((a[1].min(b[1]) - pad).floor() as i32).max(0) as usize;
    let max_y = ((a[1].max(b[1]) + pad).ceil() as i32).clamp(0, px as i32) as usize;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let d = dist_to_segment(x as f32 + 0.5, y as f32 + 0.5, a, b);
            let cov = (half + 0.5 - d).clamp(0.0, 1.0);
            let idx = y * px + x;
            if cov > buf[idx] {
                buf[idx] = cov;
            }
        }
    }
}

fn stamp_disc(buf: &mut [f32], px: usize, center: [f32; 2], radius: f32) {
    let pad = radius + 1.0;
    let min_x = ((center[0] - pad).floor() as i32).max(0) as usize;
    let max_x = ((center[0] + pad).ceil() as i32).clamp(0, px as i32) as usize;
    let min_y = ((center[1] - pad).floor() as i32).max(0) as usize;
    let max_y = ((center[1] + pad).ceil() as i32).clamp(0, px as i32) as usize;
    for y in min_y..max_y {
        for x in min_x..max_x {
            let d = (x as f32 + 0.5 - center[0]).hypot(y as f32 + 0.5 - center[1]);
            let cov = (radius + 0.5 - d).clamp(0.0, 1.0);
            let idx = y * px + x;
            if cov > buf[idx] {
                buf[idx] = cov;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::LayerId;

    fn run_pass(ctx: &Context, f: impl FnOnce()) {
        ctx.begin_pass(egui::RawInput {
            time: Some(0.0),
            ..Default::default()
        });
        f();
        let mut full = ctx.end_pass();
        full.textures_delta.clear();
    }

    fn cache_len(ctx: &Context) -> usize {
        ctx.data_mut(|data| {
            data.get_temp_mut_or_default::<IconTextureCache>(cache_id())
                .len()
        })
    }

    #[test]
    fn icon_count_is_pinned() {
        assert_eq!(Icon::all().len(), 45);
    }

    #[test]
    fn every_icon_has_elements() {
        for icon in Icon::all() {
            assert!(!icon.elements().is_empty(), "{icon:?} has no elements");
            assert!(
                !icon.elements_static().is_empty(),
                "{icon:?} has no static elements"
            );
        }
    }

    #[test]
    fn elements_stay_in_viewbox() {
        for icon in Icon::all() {
            for el in icon.elements() {
                let pts: Vec<[f32; 2]> = match el {
                    El::Line(a, b) => vec![a, b],
                    El::Poly(p) | El::PolyClosed(p) => p.to_vec(),
                    El::Circle(c, r) | El::Dot(c, r) => {
                        vec![[c[0] - r, c[1] - r], [c[0] + r, c[1] + r]]
                    }
                    El::Arc(c, r, _, _) => vec![[c[0] - r, c[1] - r], [c[0] + r, c[1] + r]],
                    El::Cubic(a, b, c, d) => vec![a, b, c, d],
                    El::Rect(p, s, _) => vec![p, [p[0] + s[0], p[1] + s[1]]],
                };
                for p in pts {
                    assert!(
                        (-2.0..=26.0).contains(&p[0]) && (-2.0..=26.0).contains(&p[1]),
                        "{icon:?} element out of bounds: {p:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn lucide_names_roundtrip() {
        let mut names = Vec::new();
        for icon in Icon::all() {
            let name = icon.lucide_name();
            assert!(!name.is_empty());
            assert_eq!(Icon::from_lucide_name(name), Some(*icon), "{name}");
            names.push(name);
        }
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), 45, "lucide names must be distinct");
        assert_eq!(Icon::from_lucide_name("no-such-icon"), None);
    }

    #[test]
    fn generated_data_matches_known_lucide_geometry() {
        // Spot-checks pinning the generated data against the Lucide source:
        // `check` is the polyline M20 6 L9 17 L4 12; `circle` is r=10;
        // `dot` is a stroked r=1 circle (renders as a dot at stroke width 2).
        assert!(matches!(
            Icon::Check.elements_static(),
            [El::Poly(&[[20.0, 6.0], [9.0, 17.0], [4.0, 12.0]])]
        ));
        assert!(matches!(
            Icon::Circle.elements_static(),
            [El::Circle([12.0, 12.0], 10.0)]
        ));
        assert!(matches!(
            Icon::Dot.elements_static(),
            [El::Circle([12.0, 12.0], 1.0)]
        ));
    }

    #[test]
    fn rasterize_is_nonempty_and_deterministic() {
        for icon in Icon::all() {
            let a = rasterize(icon.elements_static(), 24, 2.0, 0.5);
            let b = rasterize(icon.elements_static(), 24, 2.0, 0.5);
            assert_eq!(a.size, [24, 24]);
            assert_eq!(
                a.pixels, b.pixels,
                "{icon:?} rasterization not deterministic"
            );
            let alpha: u32 = a.pixels.iter().map(|c| u32::from(c.a())).sum();
            assert!(alpha > 0, "{icon:?} rasterized empty");
            // No fully-opaque flood: icons are strokes, not blobs.
            let opaque = a.pixels.iter().filter(|c| c.a() == 255).count();
            assert!(
                opaque < 24 * 24 / 2,
                "{icon:?} rasterized suspiciously solid"
            );
        }
    }

    #[test]
    fn rasterize_scales_with_size() {
        // The same icon at 2x pixels has ~4x the ink (area scales).
        let small = rasterize(Icon::Check.elements_static(), 24, 2.0, 0.5);
        let big = rasterize(Icon::Check.elements_static(), 48, 4.0, 1.0);
        let sum = |img: &ColorImage| img.pixels.iter().map(|c| u32::from(c.a())).sum::<u32>();
        let (s, b) = (sum(&small), sum(&big));
        assert!(b > 3 * s && b < 5 * s, "small={s} big={b}");
    }

    #[test]
    fn paint_all_icons_without_panicking() {
        let ctx = Context::default();
        run_pass(&ctx, || {
            let painter = ctx.layer_painter(LayerId::background());
            let rect = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(16.0));
            for icon in Icon::all() {
                icon.paint(&painter, rect, Color32::BLACK, 2.0 * 16.0 / 24.0);
                icon.paint(&painter, rect, Color32::WHITE, 2.0);
            }
        });
    }

    #[test]
    fn flatten_handles_all_element_kinds() {
        // Generated data uses only Line/Poly/PolyClosed/Circle/Rect, but
        // the other `El` variants stay constructible; pin that they all
        // rasterize (a synthetic icon exercising every arm).
        let elements = [
            El::Line([2.0, 2.0], [6.0, 6.0]),
            El::Poly(&[[8.0, 2.0], [12.0, 6.0]]),
            El::PolyClosed(&[[14.0, 2.0], [18.0, 2.0], [16.0, 6.0]]),
            El::Circle([5.0, 14.0], 3.0),
            El::Dot([11.0, 14.0], 2.0),
            El::Arc([18.0, 14.0], 3.0, 0.0, 270.0),
            El::Cubic([2.0, 20.0], [6.0, 18.0], [10.0, 22.0], [14.0, 20.0]),
            El::Rect([16.0, 18.0], [6.0, 4.0], 1.0),
        ];
        let a = rasterize(&elements, 24, 2.0, 0.5);
        let b = rasterize(&elements, 24, 2.0, 0.5);
        assert_eq!(a.pixels, b.pixels);
        let alpha: u32 = a.pixels.iter().map(|c| u32::from(c.a())).sum();
        assert!(alpha > 0, "synthetic icon rasterized empty");
    }

    #[test]
    fn cache_reuses_texture_across_tints() {
        let ctx = Context::default();
        run_pass(&ctx, || {
            let painter = ctx.layer_painter(LayerId::background());
            let rect = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(16.0));
            Icon::Check.paint(&painter, rect, Color32::BLACK, 2.0 * 16.0 / 24.0);
            Icon::Check.paint(&painter, rect, Color32::RED, 2.0 * 16.0 / 24.0);
            Icon::Check.paint(&painter, rect, Color32::BLUE, 2.0 * 16.0 / 24.0);
            // Same icon/size/stroke in three colors: one mask texture.
            assert_eq!(cache_len(&ctx), 1);
            Icon::X.paint(&painter, rect, Color32::BLACK, 2.0 * 16.0 / 24.0);
            assert_eq!(cache_len(&ctx), 2);
        });
    }

    #[test]
    fn cache_bound_is_enforced() {
        let ctx = Context::default();
        run_pass(&ctx, || {
            let painter = ctx.layer_painter(LayerId::background());
            // 300 distinct sizes of one icon force eviction past the bound.
            for size in 8..308 {
                let rect = Rect::from_min_size(Pos2::new(10.0, 10.0), Vec2::splat(size as f32));
                Icon::Check.paint(&painter, rect, Color32::BLACK, 2.0 * size as f32 / 24.0);
            }
            assert_eq!(cache_len(&ctx), MAX_CACHE_ENTRIES);
        });
    }
}
