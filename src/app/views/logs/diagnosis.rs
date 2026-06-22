use eframe::egui;

use crate::app::domain::{LogDiagnosis, LogDiagnosisStatus, LogSnapshot};
use crate::app::theme;

use super::super::super::{text::truncate_middle, theme::muted_text};

pub(super) fn render_log_diagnosis(ui: &mut egui::Ui, diagnosis: &LogDiagnosis) {
    ui.horizontal(|ui| {
        ui.strong("Diagnosis");
        ui.label(
            egui::RichText::new(diagnosis.status.label())
                .strong()
                .color(diagnosis_color(diagnosis.status)),
        );
        ui.label(truncate_middle(&diagnosis.summary, 74));
    });

    let mut rendered = 0usize;
    for finding in diagnosis.findings.iter().take(3) {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!("L{}", finding.line_number))
                    .strong()
                    .color(diagnosis_color(finding.status)),
            );
            ui.label(truncate_middle(&finding.label, 24));
            ui.label(truncate_middle(&finding.recommendation, 68));
        });
        rendered += 1;
    }

    if rendered == 0 {
        if let Some(action) = diagnosis.recommendations.first() {
            ui.label(egui::RichText::new(truncate_middle(action, 96)).color(muted_text()));
        }
    }
    ui.separator();
}

pub(super) fn diagnosis_color(status: LogDiagnosisStatus) -> egui::Color32 {
    match status {
        LogDiagnosisStatus::NoLog | LogDiagnosisStatus::Quiet => muted_text(),
        LogDiagnosisStatus::Informational => theme::info(),
        LogDiagnosisStatus::Warning => theme::warning(),
        LogDiagnosisStatus::Critical => theme::danger(),
    }
}

pub(super) fn retention_label(snapshot: &LogSnapshot) -> &'static str {
    if snapshot.truncated {
        "tail"
    } else {
        "complete"
    }
}
