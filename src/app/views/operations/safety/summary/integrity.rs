use eframe::egui;

use crate::app::{
    domain::{format_bytes, EventSeverity, WorkspaceIntegrityReport},
    theme::muted_text,
};

use super::super::super::helpers::{event_color, score_color};

pub(in crate::app) struct IntegritySummary {
    pub(in crate::app) status_label: String,
    pub(in crate::app) schema_label: String,
    pub(in crate::app) database_label: String,
    pub(in crate::app) color: egui::Color32,
    pub(in crate::app) hint: String,
}

impl IntegritySummary {
    pub(super) fn from_report(report: Option<&WorkspaceIntegrityReport>) -> Self {
        match report {
            Some(report) => Self::checked(report),
            None => Self::not_checked(),
        }
    }

    fn checked(report: &WorkspaceIntegrityReport) -> Self {
        let present_tables = report
            .required_tables
            .iter()
            .filter(|table| table.present && table.missing_columns.is_empty())
            .count();
        let present_indexes = report
            .required_indexes
            .iter()
            .filter(|index| index.present)
            .count();

        Self {
            status_label: report.status_label().to_string(),
            schema_label: format!(
                "{present_tables}/{}, {present_indexes}/{} idx",
                report.required_tables.len(),
                report.required_indexes.len()
            ),
            database_label: format!(
                "{}, {} pages",
                format_bytes(report.database_bytes),
                report.sqlite_page_count
            ),
            color: integrity_color(report),
            hint: integrity_hint(report),
        }
    }

    fn not_checked() -> Self {
        Self {
            status_label: "not checked".to_string(),
            schema_label: "run check before restore".to_string(),
            database_label: "-".to_string(),
            color: muted_text(),
            hint: "Run integrity before import or release packaging.".to_string(),
        }
    }
}

fn integrity_color(report: &WorkspaceIntegrityReport) -> egui::Color32 {
    if report.is_success() {
        score_color(100)
    } else {
        event_color(EventSeverity::Critical)
    }
}

fn integrity_hint(report: &WorkspaceIntegrityReport) -> String {
    if report.is_success() {
        "Last integrity check passed.".to_string()
    } else {
        format!(
            "Integrity failed: {} foreign-key violations.",
            report.foreign_key_violations.len()
        )
    }
}
