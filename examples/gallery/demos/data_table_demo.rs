//! Data table demo: sorting, filtering, pagination, selection and row
//! actions over the docs' payment rows.

use egui_shadcn_ui::prelude::*;

#[derive(Clone, Debug, Default)]
struct Payment {
    id: String,
    status: String,
    email: String,
    amount: f64,
}

fn payments() -> Vec<Payment> {
    vec![
        Payment {
            id: "m5gr84i9".into(),
            status: "success".into(),
            email: "ken99@example.com".into(),
            amount: 316.0,
        },
        Payment {
            id: "3u1reuv4".into(),
            status: "success".into(),
            email: "Abe45@example.com".into(),
            amount: 242.0,
        },
        Payment {
            id: "derv1ws0".into(),
            status: "processing".into(),
            email: "monserrat44@example.com".into(),
            amount: 837.0,
        },
        Payment {
            id: "5kma53ae".into(),
            status: "success".into(),
            email: "silas22@example.com".into(),
            amount: 874.0,
        },
        Payment {
            id: "bhqecj4p".into(),
            status: "failed".into(),
            email: "carmella@example.com".into(),
            amount: 721.0,
        },
        Payment {
            id: "0m8rxs2".into(),
            status: "pending".into(),
            email: "jane42@example.com".into(),
            amount: 512.0,
        },
        Payment {
            id: "k9v3nq1".into(),
            status: "processing".into(),
            email: "leo77@example.com".into(),
            amount: 128.0,
        },
    ]
}

fn status_badge(status: &str) -> Badge {
    match status {
        "success" => Badge::new("success").variant(BadgeVariant::Default),
        "processing" => Badge::new("processing").variant(BadgeVariant::Secondary),
        "failed" => Badge::new("failed").variant(BadgeVariant::Destructive),
        _ => Badge::new(status).variant(BadgeVariant::Outline),
    }
}

pub fn run(ui: &mut egui::Ui) {
    let rows = payments();

    // Controlled filter + selection, like the source's table state.
    let store = egui::Id::unique("data_table_demo_values");
    #[derive(Clone, Default)]
    struct DemoState {
        filter: String,
        selected: Vec<usize>,
    }
    let mut state: DemoState = ui
        .ctx()
        .data_mut(|d| d.get_temp::<DemoState>(store).unwrap_or_default());

    let res = DataTable::new(&rows)
        .id_salt("demo")
        .filter(&mut state.filter, |p, needle| {
            p.email.to_lowercase().contains(&needle.to_lowercase())
        })
        .filter_placeholder("Filter emails...")
        .page_size(5)
        .selection(&mut state.selected)
        .column(
            DataTableColumn::new("status", "Status")
                .width(120.0)
                .sort_key(|p: &Payment| SortValue::Text(p.status.clone()))
                .cell(|ui, p| {
                    status_badge(&p.status).show(ui);
                }),
        )
        .column(
            DataTableColumn::new("email", "Email")
                .sort_key(|p: &Payment| SortValue::Text(p.email.clone()))
                .cell(|ui, p| {
                    ui.label(p.email.clone());
                }),
        )
        .column(
            DataTableColumn::new("amount", "Amount")
                .width(100.0)
                .align(DataTableCellAlign::End)
                .sort_key(|p: &Payment| SortValue::Number(p.amount))
                .cell(|ui, p| {
                    ui.label(egui::RichText::new(format!("${:.2}", p.amount)).strong());
                }),
        )
        .row_actions(|ui, p| {
            DropdownMenu::new("")
                .id_salt(format!("actions_{}", p.id))
                .show(ui, |m| {
                    m.label("Actions");
                    m.item(MenuItem::new("Copy payment ID").icon(Icon::Copy));
                    m.separator();
                    m.item(MenuItem::new("View customer"));
                    m.item(MenuItem::new("View payment details"));
                });
        })
        .show(ui);

    ui.add_space(8.0);
    ui.label(format!(
        "{} of {} row(s) selected (filtered from {}), page {}.",
        res.selected_count, res.filtered_count, res.total_count, res.page
    ));

    ui.add_space(16.0);
    ui.label("Without pagination or selection (sort + filter only)");
    ui.add_space(4.0);
    let mut filter2 = String::new();
    DataTable::new(&rows)
        .id_salt("demo_plain")
        .filter(&mut filter2, |p, needle| {
            p.status.to_lowercase().contains(&needle.to_lowercase())
        })
        .filter_placeholder("Filter status...")
        .column(
            DataTableColumn::new("status", "Status")
                .width(140.0)
                .sort_key(|p: &Payment| SortValue::Text(p.status.clone()))
                .cell(|ui, p| {
                    ui.label(p.status.clone());
                }),
        )
        .column(
            DataTableColumn::new("email", "Email").cell(|ui, p: &Payment| {
                ui.label(p.email.clone());
            }),
        )
        .show(ui);

    ui.ctx().data_mut(|d| d.insert_temp(store, state));
}
