//! shadcn `chart.tsx` — `ChartContainer`, series config, legend and tooltip,
//! with the recharts area/line/bar recipes from the docs examples painted in
//! pure egui.
//!
//! Ported from `apps/v4/registry/new-york-v4/ui/chart.tsx` plus the chart
//! examples (`chart-example.tsx`, `chart-bar-demo.tsx`, the `visitors` card).
//! The React component is a thin wrapper around recharts; here the plots are
//! drawn directly with the egui `Painter`:
//!
//! - [`ChartContainer`] — the responsive `min-h-[200px] w-full text-xs`
//!   frame with a rounded `Card`-style border and padding.
//! - [`Chart`] — data + variant (`Area` / `Line` / `Bar`) rendered inside a
//!   [`ChartContainer`]: horizontal grid lines (`stroke-border/50`), muted
//!   `text-xs` axis ticks, a hover cursor (vertical line + active dots, or a
//!   muted band for bars), and the `chart.tsx` tooltip (`border-border/50
//!   bg-background rounded-lg shadow-xl`, dot/line/dashed indicators).
//! - The legend rows mirror `ChartLegendContent` (8px rounded-square dot +
//!   label, `gap-4`) and are clickable to toggle a series' visibility
//!   (persisted per `id_salt`).
//!
//! Series colors come from the palette `chart-1..chart-5` tokens, cycled by
//! the series position unless a series pins an explicit color index.
//!
//! ```ignore
//! use egui_shadcn_ui::prelude::*;
//!
//! let data = ChartData::new(vec!["Jan", "Feb", "Mar"])
//!     .series(ChartSeries::new("Desktop", vec![186.0, 305.0, 237.0]))
//!     .series(ChartSeries::new("Mobile", vec![80.0, 200.0, 120.0]).color_index(1));
//!
//! Chart::new(data)
//!     .variant(ChartVariant::Bar)
//!     .id_salt("visitors")
//!     .show(ui);
//! ```

use crate::theme::{color_alpha, Palette, ShadcnTheme};
use egui::{
    epaint::CornerRadius, Color32, FontFamily, FontId, Id, Order, Pos2, Rect, Response, Sense,
    Stroke, Ui, Vec2,
};

/// The chart type, mirroring the recharts components used by the docs
/// examples.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartVariant {
    /// `AreaChart` — translucent fill + 2px stroke.
    #[default]
    Area,
    /// `LineChart` — 2px stroke.
    Line,
    /// `BarChart` — grouped rounded-top bars.
    Bar,
}

/// The tooltip indicator shape from `ChartTooltipContent`
/// (`dot` / `line` / `dashed`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChartIndicator {
    /// A 10×10 rounded square in the series color.
    #[default]
    Dot,
    /// A 4px wide, row-height bar in the series color.
    Line,
    /// A dashed 1.5px outlined square.
    Dashed,
}

/// One named series of y-values. X positions come from the chart's labels.
#[derive(Clone, Debug)]
pub struct ChartSeries {
    /// Legend / tooltip label (`config[key].label`).
    pub label: String,
    /// One value per x label (shorter series just stop early).
    pub points: Vec<f64>,
    /// Index into the palette `chart-1..chart-5` tokens. `usize::MAX`
    /// resolves to the series' position in [`ChartData::series`] (mod 5).
    pub color_index: usize,
}

impl ChartSeries {
    /// Creates a series with an auto-resolved color index.
    pub fn new(label: impl Into<String>, points: Vec<f64>) -> Self {
        Self {
            label: label.into(),
            points,
            color_index: usize::MAX,
        }
    }

    /// Pins the palette chart color index (0-based, wraps mod 5).
    pub fn color_index(mut self, index: usize) -> Self {
        self.color_index = index;
        self
    }

    /// The resolved color index given the series' position.
    pub fn resolved_color_index(&self, position: usize) -> usize {
        if self.color_index == usize::MAX {
            position % 5
        } else {
            self.color_index % 5
        }
    }
}

/// The chart data: x labels plus one or more series.
#[derive(Clone, Debug, Default)]
pub struct ChartData {
    /// X axis labels (e.g. months).
    pub labels: Vec<String>,
    /// Series drawn on top of each other / side by side.
    pub series: Vec<ChartSeries>,
}

impl ChartData {
    /// Creates chart data from x labels.
    pub fn new<S: Into<String>>(labels: impl IntoIterator<Item = S>) -> Self {
        Self {
            labels: labels.into_iter().map(Into::into).collect(),
            series: Vec::new(),
        }
    }

    /// Adds a series (builder style).
    pub fn series(mut self, series: ChartSeries) -> Self {
        self.series.push(series);
        self
    }

    /// The number of x positions.
    pub fn len(&self) -> usize {
        self.labels.len()
    }

    /// True when there is nothing to draw.
    pub fn is_empty(&self) -> bool {
        self.labels.is_empty()
    }
}

/// `ChartContainer` — the responsive chart frame: `min-h-[200px] w-full
/// text-xs`, drawn with the docs' `Card`-style rounded border and padding.
///
/// Lower-level than [`Chart`]; use it to compose custom plots.
pub struct ChartContainer {
    id_salt: Option<String>,
    min_height: f32,
    max_width: Option<f32>,
}

impl Default for ChartContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl ChartContainer {
    /// Creates the container (`min-h-[200px] w-full`).
    pub fn new() -> Self {
        Self {
            id_salt: None,
            min_height: 200.0,
            max_width: None,
        }
    }

    /// Sets a stable id salt.
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Overrides the minimum height (`min-h-[...]`).
    pub fn min_height(mut self, height: f32) -> Self {
        self.min_height = height;
        self
    }

    /// Caps the container width (centered in the leftover space, like
    /// `max-w-md mx-auto`).
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Renders the bordered frame around the chart content.
    pub fn show<R>(
        self,
        ui: &mut Ui,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> egui::InnerResponse<R> {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let pad = crate::theme::tokens::spacing::SM; // p-2

        let width = self
            .max_width
            .unwrap_or(ui.available_width())
            .min(ui.available_width());

        egui::Frame::new()
            .corner_radius(theme.radius.lg_u8()) // rounded-lg
            .stroke(Stroke::new(1.0, p.border))
            .inner_margin(egui::Margin::same(pad as i8))
            .show(ui, |ui| {
                let inner_w = (width - pad * 2.0).max(0.0);
                ui.set_width(inner_w);
                ui.set_min_size(Vec2::new(inner_w, (self.min_height - pad * 2.0).max(0.0)));
                content(ui)
            })
    }
}

/// Nice round bounds + step for the y axis, unit-testable.
pub(crate) fn nice_bounds(min: f64, max: f64, tick_count: usize) -> (f64, f64, f64) {
    let tick_count = tick_count.max(2) as f64;
    if max <= min {
        // Degenerate (flat or single point) — give a ±1 unit window.
        return (min - 1.0, min + 1.0, 1.0);
    }
    let raw_step = (max - min) / (tick_count - 1.0);
    // Nice steps: 1, 2, 5 × 10^k.
    let exponent = raw_step.log10().floor();
    let magnitude = 10f64.powf(exponent);
    let frac = raw_step / magnitude;
    let step = if frac < 1.5 {
        1.0
    } else if frac < 3.0 {
        2.0
    } else if frac < 7.0 {
        5.0
    } else {
        10.0
    } * magnitude;
    let lo = (min / step).floor() * step;
    let hi = (max / step).ceil() * step;
    (lo, hi, step)
}

/// The y tick values from `lo` to `hi` stepping by `step` (inclusive),
/// unit-testable.
pub(crate) fn y_ticks(lo: f64, hi: f64, step: f64) -> Vec<f64> {
    let step = if step > 0.0 { step } else { 1.0 };
    let count = (((hi - lo) / step).round() as usize + 1).max(2);
    (0..count)
        .map(|i| {
            let v = lo + step * i as f64;
            // Kill float drift (0.30000000000000004 style).
            (v * 1e9).round() / 1e9
        })
        .collect()
}

/// `Number.toLocaleString()`-style formatting: thousands separators, no
/// trailing `.0` for whole numbers.
pub(crate) fn format_value(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded == rounded.trunc() {
        let sign = if rounded < 0.0 { "-" } else { "" };
        let digits = (rounded.abs().trunc() as i64).to_string();
        let mut grouped = String::new();
        for (i, ch) in digits.chars().enumerate() {
            if i > 0 && (digits.len() - i) % 3 == 0 {
                grouped.push(',');
            }
            grouped.push(ch);
        }
        format!("{sign}{grouped}")
    } else {
        format!("{rounded:.2}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

/// The palette chart color for a series index (`chart-1..chart-5`, wraps).
pub(crate) fn series_color(p: &Palette, color_index: usize) -> Color32 {
    match color_index % 5 {
        0 => p.chart_1,
        1 => p.chart_2,
        2 => p.chart_3,
        3 => p.chart_4,
        _ => p.chart_5,
    }
}

/// Hidden-series state persisted per chart id (`legend` toggles).
fn hidden_id(id: Id) -> Id {
    id.with("hidden_series")
}

fn read_hidden(ctx: &egui::Context, id: Id, len: usize) -> Vec<bool> {
    let stored: Vec<bool> = ctx
        .data_mut(|d| d.get_temp(hidden_id(id)))
        .unwrap_or_default();
    let mut v = vec![false; len];
    for (i, hidden) in stored.into_iter().enumerate().take(len) {
        v[i] = hidden;
    }
    v
}

fn write_hidden(ctx: &egui::Context, id: Id, hidden: Vec<bool>) {
    ctx.data_mut(|d| d.insert_temp(hidden_id(id), hidden));
}

/// The chart: [`ChartData`] + [`ChartVariant`] inside a
/// [`ChartContainer`], with legend, hover cursor and tooltip.
pub struct Chart {
    data: ChartData,
    variant: ChartVariant,
    id_salt: Option<String>,
    height: f32,
    max_width: Option<f32>,
    legend: bool,
    fill_alpha: f32,
    tooltip_indicator: ChartIndicator,
    tooltip_hide_label: bool,
    tick_count: usize,
}

impl Chart {
    /// Creates an area chart from the data.
    pub fn new(data: ChartData) -> Self {
        Self {
            data,
            variant: ChartVariant::Area,
            id_salt: None,
            height: 200.0,
            max_width: None,
            legend: true,
            fill_alpha: 0.4,
            tooltip_indicator: ChartIndicator::Line,
            tooltip_hide_label: false,
            tick_count: 5,
        }
    }

    /// Sets the chart type.
    pub fn variant(mut self, variant: ChartVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Sets a stable id salt (legend visibility + hover ids persist under
    /// it; give multiple charts distinct salts).
    pub fn id_salt(mut self, salt: impl Into<String>) -> Self {
        self.id_salt = Some(salt.into());
        self
    }

    /// Sets the plot height (`min-h-[200px]` default).
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    /// Caps the container width.
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Shows or hides the legend (default shown).
    pub fn show_legend(mut self, legend: bool) -> Self {
        self.legend = legend;
        self
    }

    /// Area fill opacity (`fillOpacity`, default 0.4 like the docs example;
    /// the `visitors` card uses 0.15).
    pub fn fill_alpha(mut self, alpha: f32) -> Self {
        self.fill_alpha = alpha.clamp(0.0, 1.0);
        self
    }

    /// The tooltip indicator shape (default `Line`, per the docs examples).
    pub fn tooltip_indicator(mut self, indicator: ChartIndicator) -> Self {
        self.tooltip_indicator = indicator;
        self
    }

    /// Hides the tooltip's label row (`hideLabel`).
    pub fn tooltip_hide_label(mut self, hide: bool) -> Self {
        self.tooltip_hide_label = hide;
        self
    }

    /// The number of horizontal grid lines / y ticks (default 5).
    pub fn tick_count(mut self, count: usize) -> Self {
        self.tick_count = count.max(2);
        self
    }

    /// Renders the chart.
    pub fn show(self, ui: &mut Ui) -> Response {
        let theme = ShadcnTheme::of_ui(ui);
        let p = theme.palette;
        let id = match self.id_salt.as_deref() {
            Some(salt) => Id::unique(format!("shadcn_chart_{salt}")),
            None => ui.auto_id_with("shadcn_chart"),
        };

        let mut hidden = read_hidden(ui.ctx(), id, self.data.series.len());

        let legend_space = if self.legend { 28.0 } else { 0.0 };
        let height = self.height;
        let max_width = self.max_width;

        let container_response = ChartContainer::new()
            .min_height(height + legend_space)
            .max_width(max_width.unwrap_or(f32::INFINITY))
            .show(ui, |ui| {
                if self.data.is_empty() || self.data.series.is_empty() {
                    // Empty state: an invisible placeholder keeps layout stable.
                    ui.allocate_space(Vec2::new(ui.available_width(), height));
                    return;
                }
                self.draw_plot(ui, id, &mut hidden, &p);
                if self.legend {
                    legend_row(ui, id, &self.data, &mut hidden, &p);
                }
            });

        // Persist the legend toggles for the next frame.
        write_hidden(ui.ctx(), id, hidden);
        container_response.response
    }

    fn draw_plot(&self, ui: &mut Ui, id: Id, hidden: &mut [bool], p: &Palette) {
        let n = self.data.len();
        let height = self.height;
        let visible: Vec<usize> = self
            .data
            .series
            .iter()
            .enumerate()
            .filter(|(i, _)| !hidden[*i])
            .map(|(i, _)| i)
            .collect();

        // --- geometry -----------------------------------------------------
        let plot_w = ui.available_width();
        let top_margin = 8.0;
        let bottom_margin = 20.0; // x tick labels
        let plot_top = ui.cursor().top() + top_margin;
        let plot_h = (height - top_margin - bottom_margin).max(40.0);

        // y bounds over the visible series.
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &si in &visible {
            for &v in &self.data.series[si].points {
                min = min.min(v);
                max = max.max(v);
            }
        }
        if visible.is_empty() {
            min = 0.0;
            max = 1.0;
        }
        let (lo, hi, step) = nice_bounds(min.min(0.0), max, self.tick_count);
        let ticks = y_ticks(lo, hi, step);

        // Left margin fits the widest y tick label.
        let x_font = FontId::new(
            crate::theme::tokens::type_scale::XS,
            FontFamily::Proportional,
        );
        let y_label_w = ticks
            .iter()
            .map(|t| {
                ui.painter()
                    .layout_no_wrap(format_value(*t), x_font.clone(), p.muted_foreground)
                    .size()
                    .x
            })
            .fold(0.0_f32, f32::max);
        let left_margin = y_label_w + 8.0;

        let plot = Rect::from_min_size(
            Pos2::new(ui.cursor().left() + left_margin, plot_top),
            Vec2::new((plot_w - left_margin).max(40.0), plot_h),
        );

        let band_w = plot.width() / n as f32;
        let x_at = |i: usize| plot.left() + band_w * (i as f32 + 0.5);
        let y_at = |v: f64| {
            let frac = ((v - lo) / (hi - lo)).clamp(0.0, 1.0);
            plot.bottom() - (frac * plot.height() as f64) as f32
        };

        // --- grid + axes ---------------------------------------------------
        let painter = ui.painter().clone();
        let grid_idx = painter.add(egui::Shape::Noop);
        let mut grid: Vec<egui::Shape> = Vec::new();
        for t in &ticks {
            let y = y_at(*t);
            grid.push(egui::Shape::line_segment(
                [Pos2::new(plot.left(), y), Pos2::new(plot.right(), y)],
                Stroke::new(1.0, color_alpha(p.border, 0.5)),
            ));
        }
        for t in &ticks {
            let y = y_at(*t);
            let galley =
                painter.layout_no_wrap(format_value(*t), x_font.clone(), p.muted_foreground);
            let pos = Pos2::new(
                plot.left() - 8.0 - galley.size().x,
                y - galley.size().y * 0.5,
            );
            grid.push(egui::Shape::galley(pos, galley, p.muted_foreground));
        }
        // X tick labels, centered under each band; skipped when cramped.
        let max_label_w = self
            .data
            .labels
            .iter()
            .map(|l| {
                painter
                    .layout_no_wrap(l.clone(), x_font.clone(), p.muted_foreground)
                    .size()
                    .x
            })
            .fold(0.0_f32, f32::max);
        let every = (((max_label_w + 8.0) / band_w.max(1.0)).ceil().max(1.0)) as usize;
        for (i, label) in self.data.labels.iter().enumerate() {
            if i % every != 0 {
                continue;
            }
            let galley = painter.layout_no_wrap(label.clone(), x_font.clone(), p.muted_foreground);
            let cx = x_at(i);
            let pos = Pos2::new(cx - galley.size().x * 0.5, plot.bottom() + 6.0);
            grid.push(egui::Shape::galley(pos, galley, p.muted_foreground));
        }
        painter.set(grid_idx, egui::Shape::Vec(grid));

        // --- plot interaction zone ------------------------------------------
        ui.allocate_space(Vec2::new(plot_w, height));
        let plot_interact = ui.interact(plot, id.with("plot"), Sense::hover());

        // --- series painting -------------------------------------------------
        let series_idx = painter.add(egui::Shape::Noop);
        let mut shapes: Vec<egui::Shape> = Vec::new();

        match self.variant {
            ChartVariant::Bar => {
                let m = visible.len().max(1);
                let usable = band_w * 0.8; // barCategoryGap 10% each side
                let bar_w = ((usable - 4.0 * (m - 1) as f32) / m as f32).max(2.0);
                for (k, &si) in visible.iter().enumerate() {
                    let color = series_color(p, self.data.series[si].resolved_color_index(si));
                    for (i, v) in self.data.series[si].points.iter().enumerate().take(n) {
                        let group_left = x_at(i) - usable * 0.5;
                        let x = group_left + k as f32 * (bar_w + 4.0);
                        let y = y_at(*v);
                        let rect = Rect::from_min_max(
                            Pos2::new(x, y.min(plot.bottom() - 0.5)),
                            Pos2::new((x + bar_w).min(plot.right()), plot.bottom()),
                        );
                        if rect.width() <= 0.0 {
                            continue;
                        }
                        shapes.push(egui::Shape::rect_filled(
                            rect,
                            CornerRadius {
                                nw: 4,
                                ne: 4,
                                sw: 0,
                                se: 0,
                            },
                            color,
                        ));
                    }
                }
            }
            ChartVariant::Area | ChartVariant::Line => {
                for &si in &visible {
                    let color = series_color(p, self.data.series[si].resolved_color_index(si));
                    let pts: Vec<Pos2> = self.data.series[si]
                        .points
                        .iter()
                        .take(n)
                        .enumerate()
                        .map(|(i, v)| Pos2::new(x_at(i), y_at(*v)))
                        .collect();
                    if pts.is_empty() {
                        continue;
                    }
                    if self.variant == ChartVariant::Area {
                        // Fill down to the baseline.
                        let mut fill = pts.clone();
                        fill.push(Pos2::new(pts[pts.len() - 1].x, plot.bottom()));
                        fill.push(Pos2::new(pts[0].x, plot.bottom()));
                        shapes.push(egui::Shape::convex_polygon(
                            fill,
                            color_alpha(color, self.fill_alpha),
                            Stroke::NONE,
                        ));
                    }
                    shapes.push(egui::Shape::line(pts, Stroke::new(2.0, color)));
                }
            }
        }
        painter.set(series_idx, egui::Shape::Vec(shapes));

        // --- hover cursor + tooltip -------------------------------------------
        if let Some(pointer) = plot_interact.hover_pos() {
            {
                let band = (((pointer.x - plot.left()) / band_w)
                    .floor()
                    .clamp(0.0, (n - 1) as f32)) as usize;

                let cursor_idx = painter.add(egui::Shape::Noop);
                let mut cursor: Vec<egui::Shape> = Vec::new();
                match self.variant {
                    ChartVariant::Bar => {
                        // `recharts-rectangle.recharts-tooltip-cursor` = `fill-muted`.
                        let band_rect = Rect::from_min_max(
                            Pos2::new(plot.left() + band_w * band as f32, plot.top()),
                            Pos2::new(
                                (plot.left() + band_w * (band as f32 + 1.0)).min(plot.right()),
                                plot.bottom(),
                            ),
                        );
                        cursor.push(egui::Shape::rect_filled(band_rect, 0, p.muted));
                    }
                    _ => {
                        let x = x_at(band);
                        cursor.push(egui::Shape::line_segment(
                            [Pos2::new(x, plot.top()), Pos2::new(x, plot.bottom())],
                            Stroke::new(1.0, p.border),
                        ));
                        for &si in &visible {
                            if let Some(v) = self.data.series[si].points.get(band) {
                                let color =
                                    series_color(p, self.data.series[si].resolved_color_index(si));
                                cursor.push(egui::Shape::circle_filled(
                                    Pos2::new(x, y_at(*v)),
                                    3.5,
                                    color,
                                ));
                            }
                        }
                    }
                }
                painter.set(cursor_idx, egui::Shape::Vec(cursor));

                let visible_values: Vec<(usize, f64)> = visible
                    .iter()
                    .filter_map(|&si| {
                        self.data
                            .series
                            .get(si)
                            .and_then(|s| s.points.get(band))
                            .map(|v| (si, *v))
                    })
                    .collect();

                let label = self.data.labels.get(band).cloned();
                draw_tooltip(
                    ui,
                    TooltipCtx {
                        id: id.with("tooltip"),
                        visible: &visible_values,
                        data: &self.data,
                        label,
                        indicator: self.tooltip_indicator,
                        hide_label: self.tooltip_hide_label,
                        palette: p,
                        anchor: Pos2::new(x_at(band) + 12.0, plot.top()),
                    },
                );
            }
        }
    }
}

/// `ChartLegendContent` — centered dot + label entries, clickable to toggle
/// a series' visibility.
fn legend_row(ui: &mut Ui, chart_id: Id, data: &ChartData, hidden: &mut [bool], p: &Palette) {
    ui.add_space(12.0); // pt-3
    let font = FontId::new(
        crate::theme::tokens::type_scale::XS,
        FontFamily::Proportional,
    );

    // Measure the total width first so the row can be centered.
    let mut widths: Vec<f32> = Vec::with_capacity(data.series.len());
    for s in &data.series {
        let label_w = ui
            .painter()
            .layout_no_wrap(s.label.clone(), font.clone(), Color32::WHITE)
            .size()
            .x;
        widths.push(8.0 + 6.0 + label_w); // dot + gap-1.5 + label
    }
    let total: f32 = widths.iter().sum::<f32>() + 16.0 * (widths.len().saturating_sub(1)) as f32;
    let available = ui.available_width();
    let mut cx = ui.cursor().left() + ((available - total) * 0.5).max(0.0);
    let cy = ui.cursor().top();

    let row_h = 16.0; // text-xs line height
    for (i, series) in data.series.iter().enumerate() {
        let rect = Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(widths[i], row_h));
        let response = ui.interact(rect, chart_id.with(("legend", i)), Sense::click());
        if response.clicked() {
            hidden[i] = !hidden[i];
        }
        let _ = response;
        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let is_hidden = hidden[i];
            let dot_color = if is_hidden {
                color_alpha(series_color(p, series.resolved_color_index(i)), 0.4)
            } else {
                series_color(p, series.resolved_color_index(i))
            };
            painter.rect_filled(
                Rect::from_min_size(
                    Pos2::new(rect.left(), rect.center().y - 4.0),
                    Vec2::splat(8.0),
                ),
                CornerRadius::same(2), // rounded-[2px] — clamped up for visibility
                dot_color,
            );
            let galley = painter.layout_no_wrap(
                series.label.clone(),
                font.clone(),
                if is_hidden {
                    p.muted_foreground
                } else {
                    p.foreground
                },
            );
            painter.galley(
                Pos2::new(
                    rect.left() + 14.0, // dot (8) + gap-1.5 (6)
                    rect.center().y - galley.size().y * 0.5,
                ),
                galley,
                if is_hidden {
                    p.muted_foreground
                } else {
                    p.foreground
                },
            );
        }
        cx += widths[i] + 16.0; // gap-4
    }
    ui.advance_cursor_after_rect(Rect::from_min_size(
        Pos2::new(ui.cursor().left(), cy),
        Vec2::new(available, row_h),
    ));
}

/// Everything the tooltip painter needs (keeps the fn signature small).
struct TooltipCtx<'a> {
    id: Id,
    visible: &'a [(usize, f64)],
    data: &'a ChartData,
    label: Option<String>,
    indicator: ChartIndicator,
    hide_label: bool,
    palette: &'a Palette,
    anchor: Pos2,
}

/// `ChartTooltipContent` — `min-w-32 rounded-lg border border-border/50
/// bg-background px-2.5 py-1.5 text-xs shadow-xl` popup.
fn draw_tooltip(ui: &Ui, t: TooltipCtx<'_>) {
    let TooltipCtx {
        id,
        visible,
        data,
        label,
        indicator,
        hide_label,
        palette: p,
        anchor,
    } = t;
    if visible.is_empty() {
        return;
    }
    let x_font = FontId::new(
        crate::theme::tokens::type_scale::XS,
        FontFamily::Proportional,
    );
    let mono_font = FontId::new(crate::theme::tokens::type_scale::XS, FontFamily::Monospace);

    let show_label = !hide_label && label.is_some();

    // Measure: label row + one row per visible series.
    let mut w: f32 = 128.0; // min-w-[8rem]
    let mut rows_h: f32 = 0.0;
    for (si, v) in visible {
        let s = &data.series[*si];
        let name_w = ui
            .painter()
            .layout_no_wrap(s.label.clone(), x_font.clone(), Color32::WHITE)
            .size()
            .x;
        let value_w = ui
            .painter()
            .layout_no_wrap(format_value(*v), mono_font.clone(), Color32::WHITE)
            .size()
            .x;
        w = w.max(14.0 + 8.0 + name_w + 12.0 + value_w + 10.0);
        rows_h += 10.0_f32.max(x_font.size) + 6.0; // indicator row + gap-1.5
    }
    let label_h = if show_label { 16.0 + 6.0 } else { 0.0 };
    let size = Vec2::new(w + 20.0, rows_h + label_h + 12.0); // + px-2.5/py-1.5

    // Keep the popup on screen: flip left of the crosshair when it would
    // overflow, clamp to the bottom.
    let screen = ui.input(|i| i.raw.screen_rect.unwrap_or(Rect::EVERYTHING));
    let mut pos = anchor;
    if pos.x + size.x > screen.right() - 4.0 {
        pos.x = anchor.x - 24.0 - size.x;
    }
    pos.y = pos.y.min(screen.bottom() - size.y - 4.0);
    pos.y = pos.y.max(screen.top() + 4.0);

    let theme = ShadcnTheme::of_ui(ui);
    egui::Area::new(id)
        .order(Order::Tooltip)
        .fixed_pos(pos)
        .interactable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::new()
                .corner_radius(theme.radius.lg_u8()) // rounded-lg
                .fill(p.background)
                .stroke(Stroke::new(1.0, color_alpha(p.border, 0.5)))
                .inner_margin(egui::Margin::symmetric(10, 6)) // px-2.5 py-1.5
                .shadow(crate::utils::shadow::shadow_lg(theme.mode.is_dark())) // shadow-xl
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 6.0; // gap-1.5
                    ui.set_min_width(w);
                    if let Some(label) = label.filter(|_| show_label) {
                        let galley =
                            ui.painter()
                                .layout_no_wrap(label, x_font.clone(), p.foreground);
                        ui.painter().galley(ui.cursor().min, galley, p.foreground);
                        ui.advance_cursor_after_rect(Rect::from_min_size(
                            ui.cursor().min,
                            Vec2::new(w, 16.0),
                        ));
                    }
                    for (si, value) in visible {
                        let s = &data.series[*si];
                        let color = series_color(p, s.resolved_color_index(*si));
                        let name_g = ui.painter().layout_no_wrap(
                            s.label.clone(),
                            x_font.clone(),
                            p.muted_foreground,
                        );
                        let value_g = ui.painter().layout_no_wrap(
                            format_value(*value),
                            mono_font.clone(),
                            p.foreground,
                        );
                        let row_h = 10.0_f32.max(name_g.size().y).max(value_g.size().y);
                        let row_rect = Rect::from_min_size(ui.cursor().min, Vec2::new(w, row_h));
                        let painter = ui.painter();
                        let cy = row_rect.center().y;
                        match indicator {
                            ChartIndicator::Dot => {
                                painter.rect_filled(
                                    Rect::from_min_size(
                                        Pos2::new(row_rect.left(), cy - 5.0),
                                        Vec2::splat(10.0),
                                    ),
                                    CornerRadius::same(2),
                                    color,
                                );
                            }
                            ChartIndicator::Line => {
                                painter.rect_filled(
                                    Rect::from_min_size(
                                        Pos2::new(row_rect.left(), row_rect.top()),
                                        Vec2::new(4.0, row_h),
                                    ),
                                    0,
                                    color,
                                );
                            }
                            ChartIndicator::Dashed => {
                                // Dashed 1.5px outlined square.
                                let r = Rect::from_min_size(
                                    Pos2::new(row_rect.left(), cy - 5.0),
                                    Vec2::splat(10.0),
                                );
                                for k in 0..3 {
                                    let seg_w = r.width() / 3.0;
                                    let x0 = r.left() + seg_w * k as f32;
                                    let x1 = r.left() + seg_w * (k as f32 + 0.6);
                                    painter.line_segment(
                                        [Pos2::new(x0, r.top()), Pos2::new(x1, r.top())],
                                        Stroke::new(1.5, color),
                                    );
                                    painter.line_segment(
                                        [Pos2::new(x0, r.bottom()), Pos2::new(x1, r.bottom())],
                                        Stroke::new(1.5, color),
                                    );
                                }
                                painter.line_segment(
                                    [
                                        Pos2::new(r.left(), r.top()),
                                        Pos2::new(r.left(), r.bottom()),
                                    ],
                                    Stroke::new(1.5, color),
                                );
                                painter.line_segment(
                                    [
                                        Pos2::new(r.right(), r.top()),
                                        Pos2::new(r.right(), r.bottom()),
                                    ],
                                    Stroke::new(1.5, color),
                                );
                            }
                        }
                        painter.galley(
                            Pos2::new(row_rect.left() + 14.0, cy - name_g.size().y * 0.5),
                            name_g,
                            p.muted_foreground,
                        );
                        painter.galley(
                            Pos2::new(
                                row_rect.right() - value_g.size().x,
                                cy - value_g.size().y * 0.5,
                            ),
                            value_g,
                            p.foreground,
                        );
                        ui.advance_cursor_after_rect(row_rect);
                    }
                });
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{Mode, Preset};

    fn palette(mode: Mode) -> Palette {
        Preset::default().build(mode).palette
    }

    fn test_data() -> ChartData {
        ChartData::new(vec!["Jan", "Feb", "Mar", "Apr"])
            .series(ChartSeries::new("Desktop", vec![186.0, 305.0, 237.0, 73.0]))
            .series(ChartSeries::new("Mobile", vec![80.0, 200.0, 120.0, 190.0]).color_index(1))
    }

    fn run_ui(ctx: &egui::Context, ui_id: &str, mut f: impl FnMut(&mut Ui)) {
        let mut out = ctx.run_ui(egui::RawInput::default(), |ctx| {
            let ui = &mut egui::Ui::new(
                ctx.clone(),
                Id::unique(ui_id),
                egui::UiBuilder::new()
                    .max_rect(Rect::from_min_size(Pos2::ZERO, Vec2::new(480.0, 400.0))),
            );
            f(ui);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn nice_bounds_rounds_to_friendly_steps() {
        let (lo, hi, step) = nice_bounds(0.0, 232.0, 5);
        assert_eq!(step, 50.0);
        assert_eq!(lo, 0.0);
        assert_eq!(hi, 250.0);
        assert_eq!(nice_bounds(0.0, 100.0, 5), (0.0, 100.0, 20.0));
        let (lo, hi, _step) = nice_bounds(73.0, 874.0, 5);
        assert!(lo <= 73.0 && hi >= 874.0);
        // Flat data still gets a ±1 window.
        let (lo, hi, _step) = nice_bounds(5.0, 5.0, 5);
        assert!(lo < 5.0 && hi > 5.0);
    }

    #[test]
    fn y_ticks_spans_bounds() {
        assert_eq!(
            y_ticks(0.0, 100.0, 20.0),
            vec![0.0, 20.0, 40.0, 60.0, 80.0, 100.0]
        );
        assert_eq!(
            y_ticks(0.0, 250.0, 50.0),
            vec![0.0, 50.0, 100.0, 150.0, 200.0, 250.0]
        );
        // Non-integral steps survive without float drift.
        assert_eq!(y_ticks(0.0, 0.3, 0.1).last(), Some(&0.3));
        // Degenerate step falls back to 1.
        assert_eq!(y_ticks(0.0, 2.0, 0.0), vec![0.0, 1.0, 2.0]);
    }

    #[test]
    fn format_value_groups_thousands() {
        assert_eq!(format_value(186.0), "186");
        assert_eq!(format_value(12486.0), "12,486");
        assert_eq!(format_value(0.0), "0");
        assert_eq!(format_value(1234567.0), "1,234,567");
        assert_eq!(format_value(-2500.0), "-2,500");
        assert_eq!(format_value(1.5), "1.5");
    }

    #[test]
    fn series_color_cycles_palette_tokens() {
        let p = palette(Mode::Light);
        assert_eq!(series_color(&p, 0), p.chart_1);
        assert_eq!(series_color(&p, 4), p.chart_5);
        assert_eq!(series_color(&p, 5), p.chart_1);
    }

    #[test]
    fn auto_color_index_follows_position() {
        let s0 = ChartSeries::new("A", vec![1.0]);
        let s1 = ChartSeries::new("B", vec![2.0]).color_index(3);
        assert_eq!(s0.resolved_color_index(0), 0);
        assert_eq!(s0.resolved_color_index(4), 4);
        assert_eq!(s1.resolved_color_index(0), 3);
    }

    #[test]
    fn builders_keep_fields() {
        let chart = Chart::new(test_data())
            .variant(ChartVariant::Bar)
            .id_salt("test")
            .height(180.0)
            .max_width(320.0)
            .show_legend(false)
            .fill_alpha(0.15)
            .tooltip_indicator(ChartIndicator::Dot)
            .tooltip_hide_label(true)
            .tick_count(3);
        assert_eq!(chart.variant, ChartVariant::Bar);
        assert_eq!(chart.id_salt.as_deref(), Some("test"));
        assert_eq!(chart.height, 180.0);
        assert_eq!(chart.max_width, Some(320.0));
        assert!(!chart.legend);
        assert_eq!(chart.fill_alpha, 0.15);
        assert_eq!(chart.tooltip_indicator, ChartIndicator::Dot);
        assert!(chart.tooltip_hide_label);
        assert_eq!(chart.tick_count, 3);
    }

    #[test]
    fn renders_area_line_bar_in_light_and_dark() {
        for mode in [Mode::Light, Mode::Dark] {
            let ctx = egui::Context::default();
            Preset::default().build(mode).install(&ctx);
            for variant in [ChartVariant::Area, ChartVariant::Line, ChartVariant::Bar] {
                run_ui(&ctx, "chart_render_test", |ui| {
                    Chart::new(test_data())
                        .variant(variant)
                        .id_salt("render")
                        .show(ui);
                });
            }
        }
    }

    #[test]
    fn legend_hidden_state_persists_per_id_salt() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        let id = Id::unique("shadcn_chart_persist");

        assert_eq!(read_hidden(&ctx, id, 2), vec![false, false]);

        // Toggle the second series, write, and read back on a fresh lookup.
        let mut hidden = read_hidden(&ctx, id, 2);
        hidden[1] = true;
        write_hidden(&ctx, id, hidden);
        assert_eq!(read_hidden(&ctx, id, 2), vec![false, true]);

        // A shorter request clips to the stored prefix; a longer one pads.
        assert_eq!(read_hidden(&ctx, id, 1), vec![false]);
        assert_eq!(read_hidden(&ctx, id, 3), vec![false, true, false]);
    }

    #[test]
    fn hidden_series_are_excluded_from_hover_values() {
        // Visible indices respect the hidden vector (the same filter used in
        // draw_plot).
        let hidden = [true, false, false, false, false];
        let series: Vec<usize> = (0..3).filter(|i| !hidden[*i]).collect();
        assert_eq!(series, vec![1, 2]);
    }

    #[test]
    fn empty_data_renders_placeholder() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        run_ui(&ctx, "chart_empty_test", |ui| {
            Chart::new(ChartData::default()).show(ui);
        });
    }

    #[test]
    fn hover_shows_cursor_and_tooltip() {
        let ctx = egui::Context::default();
        Preset::default().install(&ctx);
        // Warm up one frame so the plot rect exists, then hover inside it.
        run_ui(&ctx, "chart_hover_test", |ui| {
            Chart::new(test_data()).id_salt("hover").show(ui);
        });
        run_ui(&ctx, "chart_hover_test", |ui| {
            let r = Chart::new(test_data()).id_salt("hover").show(ui);
            let center = Pos2::new(r.rect.center().x, r.rect.top() + 60.0);
            ctx.input_mut(|i| i.events.push(egui::Event::PointerMoved(center)));
        });
        run_ui(&ctx, "chart_hover_test", |ui| {
            Chart::new(test_data()).id_salt("hover").show(ui);
        });
    }
}
