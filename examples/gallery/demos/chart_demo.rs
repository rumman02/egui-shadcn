//! Chart demo: area/line/bar variants, legend toggling, tooltip indicators.

use egui_shadcn_ui::prelude::*;

const MONTHS: [&str; 6] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun"];

fn visitors_data() -> ChartData {
    ChartData::new(MONTHS).series(ChartSeries::new(
        "Desktop",
        vec![186.0, 305.0, 237.0, 73.0, 209.0, 214.0],
    ))
}

fn multi_series_data() -> ChartData {
    ChartData::new(MONTHS)
        .series(ChartSeries::new(
            "Desktop",
            vec![186.0, 305.0, 237.0, 73.0, 209.0, 214.0],
        ))
        .series(ChartSeries::new(
            "Mobile",
            vec![80.0, 200.0, 120.0, 190.0, 130.0, 140.0],
        ))
}

fn three_series_data() -> ChartData {
    ChartData::new(MONTHS)
        .series(ChartSeries::new(
            "Desktop",
            vec![186.0, 305.0, 237.0, 73.0, 209.0, 214.0],
        ))
        .series(ChartSeries::new(
            "Mobile",
            vec![80.0, 200.0, 120.0, 190.0, 130.0, 140.0],
        ))
        .series(ChartSeries::new(
            "Tablet",
            vec![50.0, 90.0, 160.0, 110.0, 95.0, 120.0],
        ))
}

pub fn run(ui: &mut egui::Ui) {
    ui.label(
        "Area chart (hover for the tooltip; legend entries toggle series — gallery extension)",
    );
    ui.add_space(4.0);
    Chart::new(visitors_data())
        .variant(ChartVariant::Area)
        .id_salt("demo_area")
        .max_width(420.0)
        .show(ui);

    ui.add_space(16.0);
    ui.label(
        "Area chart, visitors-card style (0.15 fill, no legend, line indicator nests the label)",
    );
    ui.add_space(4.0);
    Chart::new(visitors_data())
        .variant(ChartVariant::Area)
        .id_salt("demo_area_soft")
        .fill_alpha(0.15)
        .show_legend(false)
        .tooltip_indicator(ChartIndicator::Line)
        .max_width(420.0)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Line chart, dot tooltip indicator (chart.tsx default)");
    ui.add_space(4.0);
    Chart::new(multi_series_data())
        .variant(ChartVariant::Line)
        .id_salt("demo_line")
        .tooltip_indicator(ChartIndicator::Dot)
        .max_width(420.0)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Bar chart, dashed tooltip indicator");
    ui.add_space(4.0);
    Chart::new(multi_series_data())
        .variant(ChartVariant::Bar)
        .id_salt("demo_bar")
        .tooltip_indicator(ChartIndicator::Dashed)
        .max_width(420.0)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Line chart, straight segments (smooth off)");
    ui.add_space(4.0);
    Chart::new(multi_series_data())
        .variant(ChartVariant::Line)
        .id_salt("demo_line_straight")
        .smooth(false)
        .max_width(420.0)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Three series (cycles chart-1..chart-5 tokens), tooltip label hidden");
    ui.add_space(4.0);
    Chart::new(three_series_data())
        .variant(ChartVariant::Area)
        .id_salt("demo_triple")
        .tooltip_hide_label(true)
        .max_width(420.0)
        .show(ui);

    ui.add_space(16.0);
    ui.label("Empty data");
    ui.add_space(4.0);
    Chart::new(ChartData::default()).show_legend(false).show(ui);
}
