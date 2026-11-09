//! Lucide-style icons drawn in pure Rust with [`egui::Painter`] paths.
//!
//! Every icon is defined on the lucide 24×24 grid with 2px strokes and round
//! caps/joins, then scaled to the requested size. To add an icon, append a
//! variant and define its elements in [`Icon::elements`].

use egui::epaint::StrokeKind;
use egui::{Color32, Painter, Pos2, Rect, Response, Shape, Stroke, Ui, Vec2};

/// One drawable element of an icon, in 24×24 icon coordinates.
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

/// A lucide-style icon.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Icon {
    Check,
    ChevronDown,
    ChevronUp,
    ChevronLeft,
    ChevronRight,
    ChevronsLeft,
    ChevronsRight,
    X,
    Minus,
    Plus,
    CircleAlert,
    CircleCheck,
    CircleX,
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
    pub fn elements(&self) -> Vec<El> {
        match self {
            Icon::Check => vec![El::Poly(&[[20.0, 6.0], [9.0, 17.0], [4.0, 12.0]])],
            Icon::ChevronDown => vec![El::Poly(&[[6.0, 9.0], [12.0, 15.0], [18.0, 9.0]])],
            Icon::ChevronUp => vec![El::Poly(&[[6.0, 15.0], [12.0, 9.0], [18.0, 15.0]])],
            Icon::ChevronLeft => vec![El::Poly(&[[15.0, 18.0], [9.0, 12.0], [15.0, 6.0]])],
            Icon::ChevronRight => vec![El::Poly(&[[9.0, 18.0], [15.0, 12.0], [9.0, 6.0]])],
            Icon::ChevronsLeft => vec![
                El::Poly(&[[11.0, 17.0], [6.0, 12.0], [11.0, 7.0]]),
                El::Poly(&[[18.0, 17.0], [13.0, 12.0], [18.0, 7.0]]),
            ],
            Icon::ChevronsRight => vec![
                El::Poly(&[[6.0, 17.0], [11.0, 12.0], [6.0, 7.0]]),
                El::Poly(&[[13.0, 17.0], [18.0, 12.0], [13.0, 7.0]]),
            ],
            Icon::X => vec![
                El::Line([18.0, 6.0], [6.0, 18.0]),
                El::Line([6.0, 6.0], [18.0, 18.0]),
            ],
            Icon::Minus => vec![El::Line([5.0, 12.0], [19.0, 12.0])],
            Icon::Plus => vec![
                El::Line([12.0, 5.0], [12.0, 19.0]),
                El::Line([5.0, 12.0], [19.0, 12.0]),
            ],
            Icon::CircleAlert => vec![
                El::Circle([12.0, 12.0], 10.0),
                El::Line([12.0, 8.0], [12.0, 12.0]),
                El::Dot([12.0, 16.0], 1.0),
            ],
            Icon::CircleCheck => vec![
                El::Circle([12.0, 12.0], 10.0),
                El::Poly(&[[9.0, 12.0], [11.0, 14.0], [15.0, 10.0]]),
            ],
            Icon::CircleX => vec![
                El::Circle([12.0, 12.0], 10.0),
                El::Line([15.0, 9.0], [9.0, 15.0]),
                El::Line([9.0, 9.0], [15.0, 15.0]),
            ],
            Icon::Info => vec![
                El::Circle([12.0, 12.0], 10.0),
                El::Line([12.0, 16.0], [12.0, 12.0]),
                El::Dot([12.0, 8.0], 1.0),
            ],
            // lucide loader-circle: 270° arc, radius 9, gap in the top-right quadrant.
            Icon::LoaderCircle => vec![El::Arc([12.0, 12.0], 9.0, 0.0, 270.0)],
            Icon::Calendar => vec![
                El::Rect([3.0, 4.0], [18.0, 18.0], 2.0),
                El::Line([8.0, 2.0], [8.0, 6.0]),
                El::Line([16.0, 2.0], [16.0, 6.0]),
                El::Line([3.0, 10.0], [21.0, 10.0]),
            ],
            Icon::CalendarDays => vec![
                El::Rect([3.0, 4.0], [18.0, 18.0], 2.0),
                El::Line([8.0, 2.0], [8.0, 6.0]),
                El::Line([16.0, 2.0], [16.0, 6.0]),
                El::Line([3.0, 10.0], [21.0, 10.0]),
                El::Dot([8.0, 14.0], 0.6),
                El::Dot([12.0, 14.0], 0.6),
                El::Dot([16.0, 14.0], 0.6),
                El::Dot([8.0, 18.0], 0.6),
                El::Dot([12.0, 18.0], 0.6),
                El::Dot([16.0, 18.0], 0.6),
            ],
            Icon::Search => vec![
                El::Circle([11.0, 11.0], 8.0),
                El::Line([21.0, 21.0], [16.65, 16.65]),
            ],
            Icon::MoreHorizontal => vec![
                El::Dot([5.0, 12.0], 1.0),
                El::Dot([12.0, 12.0], 1.0),
                El::Dot([19.0, 12.0], 1.0),
            ],
            Icon::MoreVertical => vec![
                El::Dot([12.0, 5.0], 1.0),
                El::Dot([12.0, 12.0], 1.0),
                El::Dot([12.0, 19.0], 1.0),
            ],
            Icon::ArrowDown => vec![
                El::Line([12.0, 5.0], [12.0, 19.0]),
                El::Poly(&[[19.0, 12.0], [12.0, 19.0], [5.0, 12.0]]),
            ],
            Icon::ArrowUp => vec![
                El::Line([12.0, 19.0], [12.0, 5.0]),
                El::Poly(&[[5.0, 12.0], [12.0, 5.0], [19.0, 12.0]]),
            ],
            Icon::ArrowUpDown => vec![
                El::Line([5.0, 8.0], [5.0, 16.0]),
                El::Poly(&[[2.0, 11.0], [5.0, 8.0], [8.0, 11.0]]),
                El::Line([19.0, 16.0], [19.0, 8.0]),
                El::Poly(&[[16.0, 13.0], [19.0, 16.0], [22.0, 13.0]]),
            ],
            Icon::ArrowLeft => vec![
                El::Line([19.0, 12.0], [5.0, 12.0]),
                El::Poly(&[[12.0, 19.0], [5.0, 12.0], [12.0, 5.0]]),
            ],
            Icon::ArrowRight => vec![
                El::Line([5.0, 12.0], [19.0, 12.0]),
                El::Poly(&[[12.0, 5.0], [19.0, 12.0], [12.0, 19.0]]),
            ],
            Icon::PanelLeft => vec![
                El::Rect([3.0, 3.0], [18.0, 18.0], 2.0),
                El::Line([9.0, 3.0], [9.0, 21.0]),
            ],
            Icon::User => vec![
                El::Circle([12.0, 7.0], 4.0),
                // Shoulders: M19 21v-2a4 4 0 0 0-4-4H9a4 4 0 0 0-4 4v2
                El::Cubic([19.0, 21.0], [19.0, 21.0], [19.0, 15.0], [15.0, 15.0]),
                El::Line([15.0, 15.0], [9.0, 15.0]),
                El::Cubic([9.0, 15.0], [5.0, 15.0], [5.0, 15.0], [5.0, 21.0]),
            ],
            Icon::Paperclip => vec![El::Poly(&[
                [21.44, 11.05],
                [12.25, 20.24],
                [11.46, 21.03],
                [10.32, 21.34],
                [9.14, 21.16],
                [8.06, 20.5],
                [7.36, 19.44],
                [7.17, 18.23],
                [7.53, 17.04],
                [15.0, 9.57],
                [15.83, 9.28],
                [16.67, 9.46],
                [17.24, 10.03],
                [17.42, 10.87],
                [17.13, 11.7],
                [10.0, 18.83],
            ])],
            Icon::Trash => vec![
                El::Line([3.0, 6.0], [21.0, 6.0]),
                El::Cubic([19.0, 6.0], [19.0, 20.0], [19.0, 20.0], [17.0, 20.0]),
                El::Line([17.0, 20.0], [7.0, 20.0]),
                El::Cubic([7.0, 20.0], [5.0, 20.0], [5.0, 20.0], [5.0, 6.0]),
                El::Cubic([8.0, 6.0], [8.0, 4.0], [8.0, 4.0], [10.0, 4.0]),
                El::Line([10.0, 4.0], [14.0, 4.0]),
                El::Cubic([14.0, 4.0], [16.0, 4.0], [16.0, 4.0], [16.0, 6.0]),
            ],
            Icon::Pencil => vec![El::Poly(&[
                [17.0, 3.0],
                [21.0, 7.0],
                [7.5, 20.5],
                [3.5, 20.5],
                [3.5, 16.5],
                [17.0, 3.0],
            ])],
            Icon::Sparkles => vec![
                // Main four-point star.
                El::PolyClosed(&[
                    [12.0, 3.0],
                    [13.9, 8.6],
                    [19.5, 10.5],
                    [13.9, 12.4],
                    [12.0, 18.0],
                    [10.1, 12.4],
                    [4.5, 10.5],
                    [10.1, 8.6],
                ]),
                El::Dot([19.0, 17.0], 0.8),
                El::Dot([5.0, 19.0], 0.8),
            ],
            Icon::Copy => vec![
                El::Rect([8.0, 8.0], [14.0, 14.0], 2.0),
                El::Cubic([4.0, 16.0], [3.0, 16.0], [2.0, 15.0], [2.0, 14.0]),
                El::Cubic([2.0, 4.0], [2.0, 3.0], [3.0, 2.0], [4.0, 2.0]),
                El::Cubic([14.0, 2.0], [15.0, 2.0], [16.0, 3.0], [16.0, 4.0]),
            ],
            Icon::CornerDownLeft => vec![
                El::Poly(&[[20.0, 4.0], [20.0, 11.0], [4.0, 11.0]]),
                El::Poly(&[[9.0, 6.0], [4.0, 11.0], [9.0, 16.0]]),
            ],
            Icon::Message => vec![El::PolyClosed(&[
                [7.9, 20.0],
                [7.0, 20.0],
                [4.0, 18.0],
                [3.0, 16.0],
                [2.6, 13.5],
                [3.2, 10.0],
                [5.0, 7.0],
                [8.0, 4.5],
                [12.0, 3.5],
                [16.0, 4.5],
                [19.0, 7.0],
                [20.8, 10.0],
                [21.4, 13.5],
                [20.5, 16.5],
                [18.5, 18.5],
                [15.5, 19.7],
                [12.0, 19.7],
            ])],
            Icon::Send => vec![El::PolyClosed(&[
                [22.0, 2.0],
                [11.0, 13.0],
                [22.0, 2.0],
                [2.0, 9.5],
                [11.0, 13.0],
                [11.0, 22.0],
                [22.0, 2.0],
            ])],
            Icon::Circle => vec![El::Circle([12.0, 12.0], 10.0)],
            Icon::Dot => vec![El::Dot([12.0, 12.0], 3.0)],
            Icon::GripVertical => vec![
                El::Dot([9.0, 5.0], 1.0),
                El::Dot([9.0, 12.0], 1.0),
                El::Dot([9.0, 19.0], 1.0),
                El::Dot([15.0, 5.0], 1.0),
                El::Dot([15.0, 12.0], 1.0),
                El::Dot([15.0, 19.0], 1.0),
            ],
            Icon::GripHorizontal => vec![
                El::Dot([5.0, 9.0], 1.0),
                El::Dot([12.0, 9.0], 1.0),
                El::Dot([19.0, 9.0], 1.0),
                El::Dot([5.0, 15.0], 1.0),
                El::Dot([12.0, 15.0], 1.0),
                El::Dot([19.0, 15.0], 1.0),
            ],
            Icon::FileText => vec![
                El::PolyClosed(&[
                    [15.0, 2.0],
                    [6.0, 2.0],
                    [4.0, 4.0],
                    [4.0, 20.0],
                    [6.0, 22.0],
                    [18.0, 22.0],
                    [20.0, 20.0],
                    [20.0, 7.0],
                ]),
                El::Poly(&[[14.0, 2.0], [14.0, 6.0], [16.0, 8.0], [20.0, 8.0]]),
                El::Line([10.0, 9.0], [8.0, 9.0]),
                El::Line([16.0, 13.0], [8.0, 13.0]),
                El::Line([16.0, 17.0], [8.0, 17.0]),
            ],
            Icon::RefreshCw => vec![
                El::Arc([12.0, 12.0], 9.0, 180.0, 340.0),
                El::Poly(&[[21.0, 3.0], [21.0, 8.0], [16.0, 8.0]]),
                El::Arc([12.0, 12.0], 9.0, 0.0, 156.0),
                El::Poly(&[[8.0, 16.0], [3.0, 16.0], [3.0, 21.0]]),
            ],
            Icon::Clock => vec![
                El::Circle([12.0, 12.0], 10.0),
                El::Poly(&[[12.0, 6.0], [12.0, 12.0], [16.0, 14.0]]),
            ],
        }
    }

    /// Paints the icon scaled into `rect`.
    pub fn paint(&self, painter: &Painter, rect: Rect, color: Color32, stroke_width: f32) {
        let scale = rect.width().min(rect.height()) / 24.0;
        let icon_size = Vec2::splat(24.0 * scale);
        let origin = rect.min + 0.5 * (rect.size() - icon_size);
        let to_screen =
            |p: [f32; 2]| -> Pos2 { Pos2::new(origin.x + p[0] * scale, origin.y + p[1] * scale) };
        let stroke = Stroke::new(stroke_width.max(1.0), color);

        for el in self.elements() {
            match el {
                El::Line(a, b) => {
                    painter.line_segment([to_screen(a), to_screen(b)], stroke);
                }
                El::Poly(points) => {
                    let pts: Vec<Pos2> = points.iter().map(|p| to_screen(*p)).collect();
                    painter.add(Shape::line(pts, stroke));
                }
                El::PolyClosed(points) => {
                    let pts: Vec<Pos2> = points.iter().map(|p| to_screen(*p)).collect();
                    painter.add(Shape::line(pts, stroke));
                }
                El::Circle(c, r) => {
                    painter.circle_stroke(to_screen(c), r * scale, stroke);
                }
                El::Dot(c, r) => {
                    painter.circle_filled(to_screen(c), (r * scale).max(0.5), color);
                }
                El::Arc(c, r, start_deg, end_deg) => {
                    let pts: Vec<Pos2> = (0..=24)
                        .map(|i| {
                            let t = start_deg + (end_deg - start_deg) * (i as f32 / 24.0);
                            let (sin, cos) = t.to_radians().sin_cos();
                            to_screen([c[0] + r * cos, c[1] + r * sin])
                        })
                        .collect();
                    painter.add(Shape::line(pts, stroke));
                }
                El::Cubic(p0, p1, p2, p3) => {
                    let pts: Vec<Pos2> = (0..=12)
                        .map(|i| {
                            let t = i as f32 / 12.0;
                            let a = lerp2(p0, p1, t);
                            let b = lerp2(p1, p2, t);
                            let c = lerp2(p2, p3, t);
                            let ab = lerp2(a, b, t);
                            let bc = lerp2(b, c, t);
                            lerp2(ab, bc, t)
                        })
                        .map(to_screen)
                        .collect();
                    painter.add(Shape::line(pts, stroke));
                }
                El::Rect(pos, size, radius) => {
                    let min = to_screen(pos);
                    let max = to_screen([pos[0] + size[0], pos[1] + size[1]]);
                    let rr = egui::CornerRadius::same((radius * scale).round() as u8);
                    painter.rect_stroke(
                        Rect::from_min_max(min, max),
                        rr,
                        egui::Stroke::new(stroke.width, color),
                        StrokeKind::Inside,
                    );
                }
            }
        }
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

fn lerp2(a: [f32; 2], b: [f32; 2], t: f32) -> [f32; 2] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t]
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Icon; 42] = [
        Icon::Check,
        Icon::ChevronDown,
        Icon::ChevronUp,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::ChevronsLeft,
        Icon::ChevronsRight,
        Icon::X,
        Icon::Minus,
        Icon::Plus,
        Icon::CircleAlert,
        Icon::CircleCheck,
        Icon::CircleX,
        Icon::Info,
        Icon::LoaderCircle,
        Icon::Calendar,
        Icon::CalendarDays,
        Icon::Search,
        Icon::MoreHorizontal,
        Icon::MoreVertical,
        Icon::ArrowDown,
        Icon::ArrowUp,
        Icon::ArrowUpDown,
        Icon::ArrowLeft,
        Icon::ArrowRight,
        Icon::PanelLeft,
        Icon::User,
        Icon::Paperclip,
        Icon::Trash,
        Icon::Pencil,
        Icon::Sparkles,
        Icon::Copy,
        Icon::CornerDownLeft,
        Icon::Message,
        Icon::Send,
        Icon::Circle,
        Icon::Dot,
        Icon::GripVertical,
        Icon::GripHorizontal,
        Icon::FileText,
        Icon::RefreshCw,
        Icon::Clock,
    ];

    #[test]
    fn every_icon_has_elements() {
        for icon in ALL {
            assert!(!icon.elements().is_empty(), "{icon:?} has no elements");
        }
    }

    #[test]
    fn elements_stay_in_viewbox() {
        for icon in ALL {
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
}
