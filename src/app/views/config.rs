mod context;
mod preview;
mod validation;

use eframe::egui;

use crate::app::domain::{ConfigGenerator, ConfigValidator};

use super::super::{
    theme::muted_text,
    widgets::{empty_state, panel},
    NeoNexusApp,
};

impl NeoNexusApp {
    pub(super) fn render_config(&mut self, ui: &mut egui::Ui) {
        let Some(node) = self.selected_node().cloned() else {
            empty_state(
                ui,
                "No node selected",
                "Choose a node from Inventory to preview configuration.",
            );
            return;
        };

        let plugins = self
            .repository
            .list_plugin_states(&node.id)
            .unwrap_or_default();
        let enabled_plugins = plugins.iter().filter(|plugin| plugin.enabled).count();
        let rendered_config = ConfigGenerator::render_for_node(&node, &plugins);
        let validation_report = rendered_config
            .as_ref()
            .map(|config| ConfigValidator::validate_rendered(&node, config));
        let export_path = self.config_export_path(&node);
        let managed_path = self.managed_config_path(&node);
        let launch_plan = self.launch_plan_for(&node);

        let available = ui.available_size();
        let left_width = (available.x * 0.34).clamp(300.0, 430.0);
        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(left_width, available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    panel(ui, "Context", |ui| {
                        ui.label(
                            egui::RichText::new("Generated from selected Inventory node.")
                                .color(muted_text()),
                        );
                        ui.separator();
                        let context = context::ConfigContext {
                            node: &node,
                            rendered_config: &rendered_config,
                            validation_report: &validation_report,
                            enabled_plugins,
                            export_path: &export_path,
                            managed_path: &managed_path,
                            launch_display_command: &launch_plan.display_command,
                        };
                        context::render_config_context(self, ui, &context);
                    });
                },
            );

            ui.add_space(8.0);

            ui.allocate_ui_with_layout(
                egui::vec2((available.x - left_width - 8.0).max(420.0), available.y),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    preview::render_config_preview(ui, &mut self.config_page, &rendered_config);
                },
            );
        });
    }
}
