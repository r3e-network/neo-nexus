use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::super::model::{NodeConfigExportReport, WorkspaceConfigReport};

pub(super) fn build_workspace_config_report(
    database: &Path,
    output_dir: &Path,
    node_count: usize,
    node_reports: Vec<NodeConfigExportReport>,
    application_version: String,
    generated_at_unix: u64,
) -> WorkspaceConfigReport {
    let total_bytes_written = node_reports
        .iter()
        .map(|node| node.bytes_written)
        .sum::<usize>();

    WorkspaceConfigReport {
        schema_version: 1,
        status: "ok",
        application: "NeoNexus",
        application_version,
        generated_at_unix,
        database: database.display().to_string(),
        output_dir: output_dir.display().to_string(),
        node_count,
        exported_file_count: node_reports.len(),
        total_bytes_written,
        nodes: node_reports,
    }
}

pub(super) fn write_report_files(
    output_dir: &Path,
    generated_at_unix: u64,
    report: &WorkspaceConfigReport,
) -> Result<(PathBuf, PathBuf)> {
    let stem = format!("node-config-export-{generated_at_unix}");
    let text_path = output_dir.join(format!("{stem}.txt"));
    let json_path = output_dir.join(format!("{stem}.json"));
    fs::write(&text_path, report.to_text())
        .with_context(|| format!("failed to write node config export {}", text_path.display()))?;
    fs::write(&json_path, report.to_json_text()?)
        .with_context(|| format!("failed to write node config export {}", json_path.display()))?;
    Ok((text_path, json_path))
}
