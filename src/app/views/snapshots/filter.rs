use eframe::egui;

use crate::app::domain::{Network, NodeType};
use crate::app::widgets::chip_pill;

use super::super::super::{theme::muted_text, NeoNexusApp};

pub(super) fn render_snapshot_registry_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    render_network_buttons(
        ui,
        app.snapshot_network_filter,
        |app, value| app.snapshot_network_filter = value,
        |app| app.snapshot_page = 0,
        app,
    );
    render_runtime_buttons(
        ui,
        app.snapshot_type_filter,
        |app, value| app.snapshot_type_filter = value,
        |app| app.snapshot_page = 0,
        app,
    );
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Integrity").color(muted_text()));
        chip_pill(ui, |ui| {
            registry_bool_button(app, ui, "All", None, RegistryBoolField::Verified);
            registry_bool_button(app, ui, "Verified", Some(true), RegistryBoolField::Verified);
            registry_bool_button(
                app,
                ui,
                "Unverified",
                Some(false),
                RegistryBoolField::Verified,
            );
        });
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Cache").color(muted_text()));
        chip_pill(ui, |ui| {
            registry_bool_button(app, ui, "All", None, RegistryBoolField::Cached);
            registry_bool_button(app, ui, "Cached", Some(true), RegistryBoolField::Cached);
            registry_bool_button(app, ui, "Missing", Some(false), RegistryBoolField::Cached);
        });
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.snapshot_query).hint_text("Search"),
    );
    if response.changed() {
        app.snapshot_page = 0;
    }
    ui.separator();
}

pub(super) fn render_snapshot_catalog_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    render_network_buttons(
        ui,
        app.snapshot_catalog_network_filter,
        |app, value| app.snapshot_catalog_network_filter = value,
        |app| app.snapshot_catalog_page = 0,
        app,
    );
    render_runtime_buttons(
        ui,
        app.snapshot_catalog_type_filter,
        |app, value| app.snapshot_catalog_type_filter = value,
        |app| app.snapshot_catalog_page = 0,
        app,
    );
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.snapshot_catalog_query).hint_text("Search"),
    );
    if response.changed() {
        app.snapshot_catalog_page = 0;
    }
    ui.separator();
}

fn render_network_buttons(
    ui: &mut egui::Ui,
    selected: Option<Network>,
    mut set: impl FnMut(&mut NeoNexusApp, Option<Network>),
    mut reset: impl FnMut(&mut NeoNexusApp),
    app: &mut NeoNexusApp,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Network").color(muted_text()));
        chip_pill(ui, |ui| {
            option_button(
                ui,
                selected.is_none(),
                "All",
                app,
                &mut set,
                &mut reset,
                None,
            );
            for network in Network::ALL {
                option_button(
                    ui,
                    selected == Some(network),
                    &network.to_string(),
                    app,
                    &mut set,
                    &mut reset,
                    Some(network),
                );
            }
        });
    });
}

fn render_runtime_buttons(
    ui: &mut egui::Ui,
    selected: Option<NodeType>,
    mut set: impl FnMut(&mut NeoNexusApp, Option<NodeType>),
    mut reset: impl FnMut(&mut NeoNexusApp),
    app: &mut NeoNexusApp,
) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Runtime").color(muted_text()));
        chip_pill(ui, |ui| {
            option_button(
                ui,
                selected.is_none(),
                "All",
                app,
                &mut set,
                &mut reset,
                None,
            );
            for node_type in NodeType::ALL {
                option_button(
                    ui,
                    selected == Some(node_type),
                    &node_type.to_string(),
                    app,
                    &mut set,
                    &mut reset,
                    Some(node_type),
                );
            }
        });
    });
}

fn option_button<T: Copy>(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    app: &mut NeoNexusApp,
    set: &mut impl FnMut(&mut NeoNexusApp, Option<T>),
    reset: &mut impl FnMut(&mut NeoNexusApp),
    value: Option<T>,
) {
    if ui.selectable_label(selected, label).clicked() {
        set(app, value);
        reset(app);
    }
}

enum RegistryBoolField {
    Verified,
    Cached,
}

fn registry_bool_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    value: Option<bool>,
    field: RegistryBoolField,
) {
    let selected = match field {
        RegistryBoolField::Verified => app.snapshot_verified_filter == value,
        RegistryBoolField::Cached => app.snapshot_cached_filter == value,
    };
    if ui.selectable_label(selected, label).clicked() {
        match field {
            RegistryBoolField::Verified => app.snapshot_verified_filter = value,
            RegistryBoolField::Cached => app.snapshot_cached_filter = value,
        }
        app.snapshot_page = 0;
    }
}
