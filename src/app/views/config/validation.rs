use eframe::egui;

use crate::app::domain::{ConfigValidationReport, ConfigValidationSeverity};
use crate::app::theme;

pub(super) fn render_config_validation(ui: &mut egui::Ui, report: &ConfigValidationReport) {
    ui.separator();
    ui.label(egui::RichText::new("Config validation").strong());
    for check in report
        .checks
        .iter()
        .filter(|check| check.severity != ConfigValidationSeverity::Pass)
        .take(4)
    {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(check.severity.label()).color(validation_color(check.severity)),
            );
            ui.label(format!("{}: {}", check.title, check.detail));
        });
    }

    if report.is_success() && !report.has_warnings() {
        ui.label(
            egui::RichText::new("Ready for managed write")
                .color(validation_color(ConfigValidationSeverity::Pass)),
        );
    } else if report.is_success() {
        ui.label(
            egui::RichText::new(report.operator_summary())
                .color(validation_color(ConfigValidationSeverity::Warning)),
        );
    } else {
        ui.label(
            egui::RichText::new(report.operator_summary())
                .color(validation_color(ConfigValidationSeverity::Critical)),
        );
    }
}

fn validation_color(severity: ConfigValidationSeverity) -> egui::Color32 {
    match severity {
        ConfigValidationSeverity::Pass => theme::success(),
        ConfigValidationSeverity::Warning => theme::warning(),
        ConfigValidationSeverity::Critical => theme::danger(),
    }
}
