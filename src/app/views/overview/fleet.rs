use eframe::egui;

use crate::app::domain::NodeStatus;

use super::super::super::{
    paging::{page_count, rows_that_fit},
    theme::{self, DensityMetrics},
    widgets::{chip_pill, empty_state, filter_chip, node_row, pagination_bar},
    NeoNexusApp, OVERVIEW_FLEET_PAGE_SIZE,
};

/// Chrome above the rows: the wrapped status chips, the search field, and the
/// pagination bar.
const FLEET_CHROME_HEIGHT: f32 = 150.0;

pub(super) fn render_fleet_snapshot(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    if app.fleet.nodes.is_empty() {
        empty_state(
            ui,
            "Empty fleet",
            "Use New Node to create the first local runtime.",
        );
        return;
    }

    render_fleet_filter(app, ui);
    let nodes = app.filtered_inventory_nodes();
    if nodes.is_empty() {
        empty_state(ui, "No matching nodes", "Adjust the fleet filter.");
        return;
    }

    // Page size follows the panel height so the last row always lands inside it.
    let row_height = DensityMetrics::for_density(app.session.density).list_row_expanded + theme::XS;
    let page_size = rows_that_fit(ui.available_height(), row_height, FLEET_CHROME_HEIGHT)
        .min(OVERVIEW_FLEET_PAGE_SIZE);
    let total_pages = page_count(nodes.len(), page_size);
    app.fleet.overview_fleet_page = app.fleet.overview_fleet_page.min(total_pages - 1);
    pagination_bar(
        ui,
        &mut app.fleet.overview_fleet_page,
        total_pages,
        nodes.len(),
    );
    ui.add_space(theme::SM);

    let start = app.fleet.overview_fleet_page * page_size;
    let rows = nodes.iter().skip(start).take(page_size).collect::<Vec<_>>();

    // Always use the shared node_row matrix (×0.16) — no second selection geometry.
    for node in rows {
        let selected = app.fleet.selected_node.as_deref() == Some(node.id.as_str());
        if node_row(ui, node, selected, false, app.session.density) {
            app.select_fleet_node(Some(node.id.clone()));
        }
        ui.add_space(theme::XS);
    }
}

fn render_fleet_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    // The chips name themselves; a leading "Status" label only made the row
    // wider than the panel it lives in.
    chip_pill(ui, |ui| {
        if filter_chip(ui, "All", app.fleet.node_status_filter.is_none()) {
            app.set_fleet_status_filter(None);
        }
        for status in NodeStatus::ALL {
            if filter_chip(
                ui,
                status.label(),
                app.fleet.node_status_filter == Some(status),
            ) {
                app.set_fleet_status_filter(Some(status));
            }
        }
    });
    ui.add_space(theme::XS);
    let response = ui.add_sized(
        [
            ui.available_width(),
            theme::DensityMetrics::COMFORTABLE.interact_y,
        ],
        egui::TextEdit::singleline(&mut app.fleet.node_query).hint_text("Search fleet"),
    );
    if response.changed() {
        app.reset_fleet_paging();
    }
    ui.add_space(theme::SM);
}
