use std::path::Path;

use eframe::egui;

use crate::app::theme;
use crate::app::{
    domain::{ConfigValidationReport, NodeConfig, RenderedConfig},
    text::{short_path, truncate_middle},
    widgets::{fact, fact_error, primary_button, render_node_fact_sheet},
    NeoNexusApp,
};

use super::validation::render_config_validation;

pub(super) struct ConfigContext<'a> {
    pub(super) node: &'a NodeConfig,
    pub(super) rendered_config: &'a anyhow::Result<RenderedConfig>,
    pub(super) validation_report: &'a Result<ConfigValidationReport, &'a anyhow::Error>,
    pub(super) enabled_plugins: usize,
    pub(super) export_path: &'a Path,
    pub(super) managed_path: &'a Path,
    pub(super) launch_display_command: &'a str,
}

pub(super) fn render_config_context(
    app: &mut NeoNexusApp,
    ui: &mut egui::Ui,
    context: &ConfigContext<'_>,
) {
    render_node_fact_sheet(ui, context.node);
    ui.separator();
    render_config_facts(app, ui, context);
    ui.add_space(8.0);
    render_validation_summary(ui, context.validation_report);
    ui.add_space(8.0);
    render_config_actions(app, ui, context.node);
}

fn render_config_facts(app: &NeoNexusApp, ui: &mut egui::Ui, context: &ConfigContext<'_>) {
    let format_label = context
        .rendered_config
        .as_ref()
        .map_or("Unavailable", |config| config.format.label());
    fact(ui, "Format", format_label);
    match context.validation_report {
        Ok(report) => {
            fact(ui, "Validation", report.status_label());
            fact(ui, "Checks", &report.summary());
        }
        Err(error) => fact_error(ui, "Validation", &error.to_string()),
    }
    fact(ui, "Enabled plugins", &context.enabled_plugins.to_string());
    fact(ui, "Export", &short_path(context.export_path, 42));
    fact(ui, "Managed", &short_path(context.managed_path, 42));
    fact(
        ui,
        "Launch",
        &truncate_middle(context.launch_display_command, 42),
    );
    fact(ui, "Log", &short_path(&app.node_log_path(context.node), 42));
}

fn render_validation_summary(
    ui: &mut egui::Ui,
    validation_report: &Result<ConfigValidationReport, &anyhow::Error>,
) {
    match validation_report {
        Ok(report) => render_config_validation(ui, report),
        Err(error) => {
            ui.label(egui::RichText::new(error.to_string()).color(theme::danger()));
        }
    }
}

fn render_config_actions(app: &mut NeoNexusApp, ui: &mut egui::Ui, node: &NodeConfig) {
    ui.horizontal(|ui| {
        if primary_button(ui, "Apply Managed").clicked() {
            app.apply_selected_managed_config();
        }
        let restart_ready = node.status.is_active();
        if ui
            .add_enabled(restart_ready, egui::Button::new("Apply + Restart"))
            .clicked()
        {
            app.apply_selected_managed_config_and_restart();
        }
        if ui.button("Export Config").clicked() {
            app.export_selected_config();
        }
    });
}
