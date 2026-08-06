//! The three field groups of a node definition.
//!
//! Grouped by what an operator is actually deciding, not by data type:
//! **Identity** is which chain this node is and what it is called, **Runtime**
//! is which binary runs it and how, and **Ports** is the endpoints it exposes.
//! Storage sits with Runtime rather than Identity because the engine is a
//! property of the binary (neo-cli/neo-rs default to RocksDB, neo-go to
//! LevelDB) and is validated against the selected node type.

use eframe::egui;

use crate::app::{
    domain::{Network, NodeType, StorageEngine},
    theme,
    widgets::{field_combo, field_text},
    NeoNexusApp,
};

/// Name, runtime family, and target network.
pub(super) fn identity(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    field_text(ui, "Name", &mut app.fleet.draft.name);
    ui.add_space(theme::SM);
    let previous_node_type = app.fleet.draft.node_type;
    field_combo(
        ui,
        "Type",
        "node_type",
        app.fleet.draft.node_type.to_string(),
        |ui| {
            for node_type in NodeType::ALL {
                ui.selectable_value(
                    &mut app.fleet.draft.node_type,
                    node_type,
                    node_type.to_string(),
                );
            }
        },
    );
    if previous_node_type != app.fleet.draft.node_type {
        app.fleet.draft.ensure_storage_compatible();
    }
    ui.add_space(theme::SM);
    field_combo(
        ui,
        "Network",
        "network",
        app.fleet.draft.network.to_string(),
        |ui| {
            for network in Network::ALL {
                ui.selectable_value(&mut app.fleet.draft.network, network, network.to_string());
            }
        },
    );
}

/// Executable, version, launch arguments, and storage engine.
pub(super) fn runtime(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    field_text(ui, "Binary", &mut app.fleet.draft.binary_path);
    ui.add_space(theme::SM);
    field_text(ui, "Version", &mut app.fleet.draft.runtime_version);
    ui.add_space(theme::SM);
    field_text(ui, "Args", &mut app.fleet.draft.args);
    ui.add_space(theme::SM);
    field_combo(
        ui,
        "Storage",
        "storage",
        app.fleet.draft.storage_engine.to_string(),
        |ui| {
            for engine in StorageEngine::ALL {
                if app.fleet.draft.node_type.supports_storage_engine(engine) {
                    ui.selectable_value(
                        &mut app.fleet.draft.storage_engine,
                        engine,
                        engine.to_string(),
                    );
                }
            }
        },
    );
}

/// The endpoints this node listens on.
pub(super) fn ports(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    field_text(ui, "RPC", &mut app.fleet.draft.rpc_port);
    ui.add_space(theme::SM);
    field_text(ui, "P2P", &mut app.fleet.draft.p2p_port);
    ui.add_space(theme::SM);
    field_text(ui, "WebSocket", &mut app.fleet.draft.ws_port);
}
