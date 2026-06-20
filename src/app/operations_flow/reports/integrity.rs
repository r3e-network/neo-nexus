use super::super::*;

impl NeoNexusApp {
    pub(in crate::app) fn run_workspace_integrity_check(&mut self) {
        match WorkspaceIntegrityChecker::check(self.repository.db_path(), env!("CARGO_PKG_VERSION"))
        {
            Ok(report) => {
                let message = workspace_integrity_notice(&report);
                let severity = if report.is_success() {
                    EventSeverity::Info
                } else {
                    EventSeverity::Critical
                };
                self.record_event(
                    None,
                    None,
                    EventKind::WorkspaceIntegrityChecked,
                    severity,
                    message.clone(),
                );
                self.workspace_integrity_report = Some(report);
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}

fn workspace_integrity_notice(report: &WorkspaceIntegrityReport) -> String {
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
    let status = report.status_label();
    format!(
        "Workspace integrity {status}: {present_tables}/{} tables, {present_indexes}/{} indexes, {} foreign-key violations, {}",
        report.required_tables.len(),
        report.required_indexes.len(),
        report.foreign_key_violations.len(),
        format_bytes(report.database_bytes)
    )
}
