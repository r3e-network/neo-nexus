use std::path::{Path, PathBuf};

use eframe::egui;

use crate::{
    app::theme::muted_text,
    backup::WorkspaceBackupImporter,
    events::EventSeverity,
    metrics::format_bytes,
    types::{NodeConfig, NodeStatus},
    workspace_integrity::WorkspaceIntegrityReport,
};

use super::super::helpers::{event_color, score_color};

pub(super) struct WorkspaceSafetySummary {
    pub(super) node_count: usize,
    pub(super) has_running_nodes: bool,
    pub(super) latest_backup: Option<PathBuf>,
    pub(super) integrity: IntegritySummary,
}

impl WorkspaceSafetySummary {
    pub(super) fn new(
        nodes: &[NodeConfig],
        backup_export_dir: &Path,
        integrity_report: Option<&WorkspaceIntegrityReport>,
    ) -> Self {
        Self {
            node_count: nodes.len(),
            has_running_nodes: nodes.iter().any(|node| node.status == NodeStatus::Running),
            latest_backup: WorkspaceBackupImporter::latest_backup_path(backup_export_dir)
                .ok()
                .flatten(),
            integrity: IntegritySummary::from_report(integrity_report),
        }
    }

    pub(super) fn can_export(&self) -> bool {
        self.node_count > 0
    }

    pub(super) fn can_import(&self) -> bool {
        self.latest_backup.is_some() && !self.has_running_nodes
    }
}

pub(super) struct IntegritySummary {
    pub(super) status_label: String,
    pub(super) schema_label: String,
    pub(super) database_label: String,
    pub(super) color: egui::Color32,
    pub(super) hint: String,
}

impl IntegritySummary {
    fn from_report(report: Option<&WorkspaceIntegrityReport>) -> Self {
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
