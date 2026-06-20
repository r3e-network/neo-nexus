use eframe::egui;

use crate::types::NodeConfig;

use super::{
    super::super::{
        paging::page_count,
        text::truncate_middle,
        theme::muted_text,
        widgets::{empty_state, fact, pagination_bar, plugin_enabled},
        NeoNexusApp, PLUGIN_PAGE_SIZE,
    },
    filter::render_plugin_filter,
};

pub(super) fn render_plugin_list(app: &mut NeoNexusApp, ui: &mut egui::Ui, node: &NodeConfig) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Target").color(muted_text()));
        ui.label(truncate_middle(&node.name, 28));
    });
    fact(ui, "Runtime", &node.node_type.to_string());
    ui.separator();

    let total_plugins = app.plugin_catalog.for_node_type(node.node_type).len();

    if total_plugins == 0 {
        empty_state(
            ui,
            "No plugins",
            "This runtime has no supported plugins yet.",
        );
        return;
    }

    render_plugin_filter(app, ui);
    app.ensure_valid_plugin_selection(node);
    let plugins = app.filtered_plugins_for_node(node);

    if plugins.is_empty() {
        empty_state(ui, "No matching plugins", "Adjust the catalog filter.");
        return;
    }

    let states = app
        .repository
        .list_plugin_states(&node.id)
        .unwrap_or_default();
    let total_pages = page_count(plugins.len(), PLUGIN_PAGE_SIZE);
    app.plugin_page = app.plugin_page.min(total_pages - 1);
    let start = app.plugin_page * PLUGIN_PAGE_SIZE;
    let visible = plugins.iter().skip(start).take(PLUGIN_PAGE_SIZE);

    pagination_bar(ui, &mut app.plugin_page, total_pages, plugins.len());
    ui.separator();

    egui::Grid::new("plugin_table")
        .striped(true)
        .min_col_width(72.0)
        .show(ui, |ui| {
            ui.strong("Plugin");
            ui.strong("Category");
            ui.strong("Enabled");
            ui.end_row();

            for plugin in visible {
                let selected = app.selected_plugin == Some(plugin.id);
                if ui.selectable_label(selected, plugin.name).clicked() {
                    app.selected_plugin = Some(plugin.id);
                }
                ui.label(plugin.category.to_string());
                ui.label(if plugin_enabled(&states, plugin.id) {
                    "Yes"
                } else {
                    "No"
                });
                ui.end_row();
            }
        });
}
