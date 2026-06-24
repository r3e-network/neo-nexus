use eframe::egui;

use crate::app::domain::{NodeConfig, NodeStatus};

use super::super::super::{
    paging::page_count,
    text::truncate_middle,
    theme::{self, status_color},
    widgets::{empty_state, grid_header, pagination_bar},
    NeoNexusApp, OVERVIEW_FLEET_PAGE_SIZE,
};

pub(super) fn render_fleet_snapshot(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    if app.nodes.is_empty() {
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
    app.overview_fleet_page = app.overview_fleet_page.min(total_pages - 1);
    pagination_bar(ui, &mut app.overview_fleet_page, total_pages, nodes.len());
    ui.separator();

    let start = app.overview_fleet_page * OVERVIEW_FLEET_PAGE_SIZE;
    let rows = nodes
        .iter()
        .skip(start)
        .take(OVERVIEW_FLEET_PAGE_SIZE)
        .collect::<Vec<_>>();
    render_fleet_table(app, ui, &rows);
}

fn render_fleet_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(theme::muted_body("Status"));
        status_button(app, ui, "All", None);
        for status in NodeStatus::ALL {
            status_button(app, ui, status.label(), Some(status));
        }
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.node_query).hint_text("Search"),
    );
    if response.changed() {
        app.node_page = 0;
        app.overview_fleet_page = 0;
    }
    ui.separator();
}

fn status_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    status: Option<NodeStatus>,
) {
    if ui
        .selectable_label(app.node_status_filter == status, label)
        .clicked()
    {
        app.node_status_filter = status;
        app.node_page = 0;
        app.overview_fleet_page = 0;
    }
}

fn render_fleet_table(app: &mut NeoNexusApp, ui: &mut egui::Ui, rows: &[&NodeConfig]) {
    egui::Grid::new("fleet_snapshot")
        .striped(true)
        .min_col_width(74.0)
        .show(ui, |ui| {
            grid_header(ui, &["Name", "Type", "Network", "RPC", "Status"]);

            for node in rows {
                let selected = app.selected_node.as_deref() == Some(node.id.as_str());
                if ui
                    .selectable_label(selected, truncate_middle(&node.name, 22))
                    .clicked()
                {
                    app.selected_node = Some(node.id.clone());
                    app.selected_plugin = None;
                    app.plugin_page = 0;
                    app.config_page = 0;
                    app.log_page = 0;
                }
                ui.label(node.node_type.to_string());
                ui.label(node.network.to_string());
                ui.label(node.rpc_port.to_string());
                ui.label(
                    theme::body(node.status.label())
                        .color(status_color(node.status))
                        .strong(),
                );
                ui.end_row();
            }
        });
}
