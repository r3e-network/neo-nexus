use eframe::egui;

use crate::app::domain::NodeConfig;

use super::{
    super::super::{
        paging::{page_count, rows_that_fit},
        text::truncate_middle,
        theme::muted_text,
        widgets::{empty_state, fact, grid_header, pagination_bar, plugin_enabled},
        NeoNexusApp, PLUGIN_PAGE_SIZE,
    },
    filter::render_plugin_filter,
};

/// Measured pitch between two striped catalog rows: the row itself, which is
/// as tall as its leading selectable label, plus the grid's row spacing.
const PLUGIN_ROW_HEIGHT: f32 = 52.0;

/// Chrome between the filter and the first plugin: the pagination bar, its
/// separator, and the grid's column header.
const PLUGIN_CHROME_HEIGHT: f32 = 74.0;

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
    // Measured after the filter, where the rows begin; the bar, separator and
    // column header drawn below come off as reserved space.
    let page_size = rows_that_fit(
        ui.available_height(),
        PLUGIN_ROW_HEIGHT,
        PLUGIN_CHROME_HEIGHT,
    )
    .min(PLUGIN_PAGE_SIZE);
    let total_pages = page_count(plugins.len(), page_size);
    app.plugin_page = app.plugin_page.min(total_pages - 1);
    let start = app.plugin_page * page_size;
    let visible = plugins.iter().skip(start).take(page_size);

    pagination_bar(ui, &mut app.plugin_page, total_pages, plugins.len());
    ui.separator();

    egui::Grid::new("plugin_table")
        .striped(true)
        .min_col_width(72.0)
        .show(ui, |ui| {
            grid_header(ui, &["Plugin", "Category", "Enabled"]);

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
