use eframe::egui;

use crate::catalog::PluginCategory;

use super::super::super::{theme::muted_text, NeoNexusApp};

pub(super) fn render_plugin_filter(app: &mut NeoNexusApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("State").color(muted_text()));
        state_button(app, ui, "All", None);
        state_button(app, ui, "Enabled", Some(true));
        state_button(app, ui, "Disabled", Some(false));
    });
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Category").color(muted_text()));
        category_button(app, ui, "All", None);
        for category in PluginCategory::ALL {
            category_button(app, ui, category_label(category), Some(category));
        }
    });
    let response = ui.add_sized(
        [ui.available_width(), 24.0],
        egui::TextEdit::singleline(&mut app.plugin_query).hint_text("Search"),
    );
    if response.changed() {
        app.plugin_page = 0;
    }
    ui.separator();
}

fn state_button(app: &mut NeoNexusApp, ui: &mut egui::Ui, label: &str, enabled: Option<bool>) {
    if ui
        .selectable_label(app.plugin_enabled_filter == enabled, label)
        .clicked()
    {
        app.plugin_enabled_filter = enabled;
        app.plugin_page = 0;
    }
}

fn category_button(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    label: &str,
    category: Option<PluginCategory>,
) {
    if ui
        .selectable_label(app.plugin_category_filter == category, label)
        .clicked()
    {
        app.plugin_category_filter = category;
        app.plugin_page = 0;
    }
}

fn category_label(category: PluginCategory) -> &'static str {
    match category {
        PluginCategory::Api => "API",
        PluginCategory::Core => "Core",
        PluginCategory::Indexing => "Indexing",
        PluginCategory::Storage => "Storage",
    }
}
