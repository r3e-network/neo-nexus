use eframe::egui;

use crate::app::domain::{NodeConfig, NodeStatus};

use super::super::super::{
    paging::page_count,
    text::truncate_middle,
    theme,
    widgets::{
        chip_pill, empty_state, filter_chip, grid_header, node_row, pagination_bar, status_badge,
    },
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
    ui.add_space(theme::SM);

    let start = app.overview_fleet_page * OVERVIEW_FLEET_PAGE_SIZE;
    let rows = nodes
        .iter()
        .skip(start)
        .take(OVERVIEW_FLEET_PAGE_SIZE)
        .collect::<Vec<_>>();

    // Card-style rows when the pane is narrow enough; fall back to a compact
    // grid when many columns still fit comfortably.
    if ui.available_width() < 420.0 {
        render_fleet_cards(app, ui, &rows);
    } else {
        render_fleet_table(app, ui, &rows);
    }
}

fn render_fleet_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(theme::muted_body("Status"));
        chip_pill(ui, |ui| {
            if filter_chip(ui, "All", app.node_status_filter.is_none()) {
                app.set_fleet_status_filter(None);
            }
            for status in NodeStatus::ALL {
                if filter_chip(
                    ui,
                    status.label(),
                    app.node_status_filter == Some(status),
                ) {
                    app.set_fleet_status_filter(Some(status));
                }
            }
        });
    });
    let response = ui.add_sized(
        [ui.available_width(), 28.0],
        egui::TextEdit::singleline(&mut app.node_query).hint_text("Search fleet"),
    );
    if response.changed() {
        app.reset_fleet_paging();
    }
    ui.add_space(theme::SM);
}

fn render_fleet_cards(app: &mut NeoNexusApp, ui: &mut egui::Ui, rows: &[&NodeConfig]) {
    for node in rows {
        let selected = app.selected_node.as_deref() == Some(node.id.as_str());
        if node_row(ui, node, selected, false) {
            select_fleet_node(app, node.id.clone());
        }
        ui.add_space(theme::XS);
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
                    select_fleet_node(app, node.id.clone());
                }
                ui.label(node.node_type.to_string());
                ui.label(node.network.to_string());
                ui.label(node.rpc_port.to_string());
                status_badge(ui, node.status);
                ui.end_row();
            }
        });
}

fn select_fleet_node(app: &mut NeoNexusApp, id: String) {
    app.select_fleet_node(Some(id));
}
