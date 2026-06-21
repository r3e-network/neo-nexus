use eframe::egui;

use crate::app::domain::{NodeConfig, NodeType};

use super::{
    super::super::{
        text::truncate_middle,
        widgets::{empty_state, fact, plugin_enabled},
        NeoNexusApp,
    },
    package::render_plugin_package_installer,
};

pub(super) fn render_plugin_details(app: &mut NeoNexusApp, ui: &mut egui::Ui, node: &NodeConfig) {
    app.ensure_valid_plugin_selection(node);
    let plugins = app.filtered_plugins_for_node(node);

    let Some(plugin) = app
        .selected_plugin
        .and_then(|id| plugins.iter().find(|plugin| plugin.id == id).cloned())
    else {
        empty_state(ui, "No plugin", "Select a plugin from the catalog.");
        return;
    };

    let states = app
        .repository
        .list_plugin_states(&node.id)
        .unwrap_or_default();
    let mut enabled = plugin_enabled(&states, plugin.id);
    let before = enabled;

    ui.heading(plugin.name);
    ui.label(plugin.description);
    ui.separator();
    fact(ui, "Internal ID", &plugin.id.to_string());
    fact(ui, "Category", &plugin.category.to_string());
    fact(
        ui,
        "Restart",
        if plugin.requires_restart {
            "Required"
        } else {
            "Not required"
        },
    );
    fact(ui, "Target node", &truncate_middle(&node.name, 34));

    ui.add_space(12.0);
    ui.checkbox(&mut enabled, "Enabled for selected node");
    if enabled != before {
        app.toggle_plugin(plugin.id, enabled);
    }

    if node.node_type == NodeType::NeoCli {
        ui.separator();
        render_plugin_package_installer(app, ui, node, &plugin);
    }
}
