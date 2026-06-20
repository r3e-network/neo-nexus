use super::super::*;
use crate::readiness_report::WorkspaceReadinessReport;

impl NeoNexusApp {
    pub(in crate::app) fn export_workspace_readiness_report(
        &mut self,
        diagnostics: &FleetDiagnostics,
    ) {
        match WorkspaceReadinessReporter::write(
            self.readiness_report_dir(),
            self.repository.db_path(),
            diagnostics,
            env!("CARGO_PKG_VERSION"),
        ) {
            Ok(export) => {
                let severity = readiness_event_severity(&export.report);
                let message = format!(
                    "Readiness report exported: {} score {}, {} critical, {} warnings, {}",
                    export.report.status,
                    export.report.score,
                    export.report.critical_count,
                    export.report.warning_count,
                    short_path(&export.text_path, 48)
                );
                self.record_event(
                    None,
                    None,
                    EventKind::WorkspaceReadinessReportExported,
                    severity,
                    format!(
                        "Workspace readiness report exported to {}; JSON {}",
                        export.text_path.display(),
                        export.json_path.display()
                    ),
                );
                self.notice = Some(message);
            }
            Err(error) => self.notice = Some(error.to_string()),
        }
    }
}

fn readiness_event_severity(report: &WorkspaceReadinessReport) -> EventSeverity {
    if report.critical_count > 0 {
        EventSeverity::Critical
    } else if report.warning_count > 0 {
        EventSeverity::Warning
    } else {
        EventSeverity::Info
    }
}
