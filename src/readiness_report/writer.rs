use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

use crate::diagnostics::FleetDiagnostics;

use super::model::WorkspaceReadinessReport;

pub struct WorkspaceReadinessReporter;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceReadinessReportExport {
    pub output_dir: PathBuf,
    pub text_path: PathBuf,
    pub json_path: PathBuf,
    pub report: WorkspaceReadinessReport,
}

impl WorkspaceReadinessReportExport {
    pub fn to_cli_text(&self) -> String {
        format!(
            "workspace-readiness-report: {status}\nscore: {score}\ncritical: {critical}\nwarnings: {warnings}\nreport-text: {text}\nreport-json: {json}\n",
            status = self.report.status,
            score = self.report.score,
            critical = self.report.critical_count,
            warnings = self.report.warning_count,
            text = self.text_path.display(),
            json = self.json_path.display(),
        )
    }
}

impl WorkspaceReadinessReporter {
    pub fn write(
        output_dir: impl AsRef<Path>,
        database: impl AsRef<Path>,
        diagnostics: &FleetDiagnostics,
        application_version: impl Into<String>,
    ) -> Result<WorkspaceReadinessReportExport> {
        Self::write_at(
            output_dir,
            database,
            diagnostics,
            application_version,
            current_unix_time()?,
        )
    }

    pub fn write_at(
        output_dir: impl AsRef<Path>,
        database: impl AsRef<Path>,
        diagnostics: &FleetDiagnostics,
        application_version: impl Into<String>,
        generated_at_unix: u64,
    ) -> Result<WorkspaceReadinessReportExport> {
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "failed to create readiness report directory {}",
                output_dir.display()
            )
        })?;

        let report = WorkspaceReadinessReport::from_diagnostics(
            database,
            diagnostics,
            application_version,
            generated_at_unix,
        );
        let stem = format!("workspace-readiness-{generated_at_unix}");
        let text_path = output_dir.join(format!("{stem}.txt"));
        let json_path = output_dir.join(format!("{stem}.json"));

        fs::write(&text_path, report.to_text())
            .with_context(|| format!("failed to write readiness report {}", text_path.display()))?;
        fs::write(&json_path, report.to_json_text()?)
            .with_context(|| format!("failed to write readiness report {}", json_path.display()))?;

        Ok(WorkspaceReadinessReportExport {
            output_dir: output_dir.to_path_buf(),
            text_path,
            json_path,
            report,
        })
    }
}

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_secs())
}
