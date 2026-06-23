use eframe::egui;

use crate::app::domain::NodeType;
use crate::app::widgets::chip_pill;

use super::super::super::{theme::muted_text, NeoNexusApp};

pub(super) fn render_runtime_inventory_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    render_runtime_buttons(
        ui,
        app.runtime_inventory_type_filter,
        |app, value| app.runtime_inventory_type_filter = value,
        |app| app.runtime_page = 0,
        app,
    );
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Trust").color(muted_text()));
        chip_pill(ui, |ui| {
            inventory_trust_button(app, ui, "All", None);
            inventory_trust_button(app, ui, "Signed", Some(true));
            inventory_trust_button(app, ui, "Hash", Some(false));
        });
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Platform").color(muted_text()));
        chip_pill(ui, |ui| {
            inventory_platform_button(app, ui, "All", None);
            inventory_platform_button(app, ui, "This host", Some(true));
            inventory_platform_button(app, ui, "Other", Some(false));
        });
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.runtime_inventory_query).hint_text("Search"),
    );
    if response.changed() {
        app.runtime_page = 0;
    }
    ui.separator();
}

pub(super) fn render_runtime_release_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    render_runtime_buttons(
        ui,
        app.runtime_catalog_type_filter,
        |app, value| app.runtime_catalog_type_filter = value,
        |app| app.runtime_catalog_page = 0,
        app,
    );
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Platform").color(muted_text()));
        chip_pill(ui, |ui| {
            catalog_platform_button(app, ui, "All", None);
            catalog_platform_button(app, ui, "This host", Some(true));
            catalog_platform_button(app, ui, "Other", Some(false));
        });
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.runtime_catalog_query).hint_text("Search"),
    );
    if response.changed() {
        app.runtime_catalog_page = 0;
    }
    ui.separator();
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
            if ui.selectable_label(selected.is_none(), "All").clicked() {
                set(app, None);
                reset(app);
            }
            for node_type in NodeType::ALL {
                if ui
                    .selectable_label(selected == Some(node_type), node_type.to_string())
                    .clicked()
                {
                    set(app, Some(node_type));
                    reset(app);
                }
            }
        });
    });
}

fn inventory_trust_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    signed: Option<bool>,
) {
    if ui
        .selectable_label(app.runtime_inventory_signed_filter == signed, label)
        .clicked()
    {
        app.runtime_inventory_signed_filter = signed;
        app.runtime_page = 0;
    }
}

fn inventory_platform_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    compatible: Option<bool>,
) {
    if ui
        .selectable_label(app.runtime_inventory_platform_filter == compatible, label)
        .clicked()
    {
        app.runtime_inventory_platform_filter = compatible;
        app.runtime_page = 0;
    }
}

fn catalog_platform_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    compatible: Option<bool>,
) {
    if ui
        .selectable_label(app.runtime_catalog_platform_filter == compatible, label)
        .clicked()
    {
        app.runtime_catalog_platform_filter = compatible;
        app.runtime_catalog_page = 0;
    }
}
