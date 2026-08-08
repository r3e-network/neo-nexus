//! The three field groups of a node definition.
//!
//! Grouped by what an operator is actually deciding, not by data type:
//! **Identity** is which chain this node is and what it is called, **Runtime**
//! is which binary runs it and how, and **Ports** is the endpoints it exposes.
//! Storage sits with Runtime rather than Identity because the engine is a
//! property of the binary, and is validated against the selected node type.
//! Only neo-cli offers a choice at all: it ships LevelDBStore and accepts
//! RocksDBStore when that plugin assembly is present. neo-go is LevelDB,
//! neo-rs is RocksDB, and neither Neo X client is selectable.

use eframe::egui;

use crate::app::{
    domain::{ChainFamily, Network, NodeType, StorageEngine},
    theme,
    widgets::{field_combo, field_static, field_text},
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
            // Grouped by chain, because the choice is which chain first and
            // which client second — and because `neo-rs` and `neox-rs` differ
            // by one character in a flat list.
            for family in ChainFamily::ALL {
                ui.label(egui::RichText::new(family.label()).color(theme::muted_text()));
                for node_type in NodeType::ALL
                    .into_iter()
                    .filter(|node_type| node_type.family() == family)
                {
                    ui.selectable_value(
                        &mut app.fleet.draft.node_type,
                        node_type,
                        node_type.to_string(),
                    );
                }
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
    storage(app, ui);
}

/// The storage engine, when it is an operator's choice at all.
///
/// Neither Neo X client lets one pick: Geth keeps its own Pebble store and
/// neox-rs keeps Reth's MDBX. A picker there would offer a decision that
/// changes nothing about what the node actually runs.
fn storage(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    let node_type = app.fleet.draft.node_type;
    let choices: Vec<StorageEngine> = StorageEngine::ALL
        .into_iter()
        .filter(|engine| node_type.supports_storage_engine(*engine))
        .collect();

    if node_type.family() == ChainFamily::NeoX || choices.len() < 2 {
        field_static(
            ui,
            "Storage",
            &node_type.storage_label(app.fleet.draft.storage_engine),
        );
        return;
    }

    field_combo(
        ui,
        "Storage",
        "storage",
        app.fleet.draft.storage_engine.to_string(),
        |ui| {
            for engine in choices {
                ui.selectable_value(
                    &mut app.fleet.draft.storage_engine,
                    engine,
                    engine.to_string(),
                );
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
