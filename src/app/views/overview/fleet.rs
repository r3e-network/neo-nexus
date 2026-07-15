use eframe::egui;

use crate::app::domain::NodeStatus;

use super::super::super::{
    paging::page_count,
    theme,
    widgets::{chip_pill, empty_state, filter_chip, node_row, pagination_bar},
    NeoNexusApp, OVERVIEW_FLEET_PAGE_SIZE,
};

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

    let total_pages = page_count(nodes.len(), OVERVIEW_FLEET_PAGE_SIZE);
    app.fleet.overview_fleet_page = app.fleet.overview_fleet_page.min(total_pages - 1);
    pagination_bar(ui, &mut app.fleet.overview_fleet_page, total_pages, nodes.len());
    ui.add_space(theme::SM);

    let start = app.fleet.overview_fleet_page * OVERVIEW_FLEET_PAGE_SIZE;
    let rows = nodes
        .iter()
        .skip(start)
        .take(OVERVIEW_FLEET_PAGE_SIZE)
        .collect::<Vec<_>>();

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
    ui.horizontal(|ui| {
        ui.label(theme::muted_body("Status"));
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
    });
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
