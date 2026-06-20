use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::{
    logs::{LogDiagnosisStatus, LogReader},
    redaction::redact_sensitive_text,
    supervisor::log_path_for,
    types::NodeConfig,
};

use super::{
    render::truncate_log_excerpt, SupportBundleLogDiagnosis, SupportBundleLogDiagnosisReport,
    SupportBundleLogFinding,
};

const LOG_DIAGNOSIS_MAX_BYTES: usize = 256 * 1024;

pub(super) fn default_log_dir(database: &Path) -> PathBuf {
    database
        .parent()
        .map_or_else(|| PathBuf::from("logs"), |parent| parent.join("logs"))
}

pub(super) fn log_diagnosis_report(
    nodes: &[NodeConfig],
    log_dir: &Path,
) -> Result<SupportBundleLogDiagnosisReport> {
    let entries = nodes
        .iter()
        .map(|node| support_log_diagnosis(node, log_dir))
        .collect::<Result<Vec<_>>>()?;
    let warning_count = entries
        .iter()
        .filter(|entry| entry.status == LogDiagnosisStatus::Warning.label())
        .count();
    let critical_count = entries
        .iter()
        .filter(|entry| entry.status == LogDiagnosisStatus::Critical.label())
        .count();

    Ok(SupportBundleLogDiagnosisReport {
        schema_version: 1,
        log_dir: log_dir.display().to_string(),
        node_count: nodes.len(),
        warning_count,
        critical_count,
        entries,
    })
}

fn support_log_diagnosis(node: &NodeConfig, log_dir: &Path) -> Result<SupportBundleLogDiagnosis> {
    let log_path = log_path_for(log_dir, node);
    let snapshot = LogReader::snapshot(&log_path, LOG_DIAGNOSIS_MAX_BYTES)?;
    let diagnosis = LogReader::diagnose(&snapshot);
    let findings = diagnosis
        .findings
        .iter()
        .map(|finding| SupportBundleLogFinding {
            label: finding.label.clone(),
            line_number: finding.line_number,
            excerpt: truncate_log_excerpt(&redact_sensitive_text(&finding.excerpt)),
            recommendation: finding.recommendation.clone(),
            status: finding.status.label().to_string(),
        })
        .collect::<Vec<_>>();

    Ok(SupportBundleLogDiagnosis {
        node_id: node.id.clone(),
        node_name: node.name.clone(),
        node_type: node.node_type.to_string(),
        log_path: log_path.display().to_string(),
        exists: snapshot.exists,
        bytes: snapshot.bytes,
        truncated: snapshot.truncated,
        status: diagnosis.status.label().to_string(),
        summary: diagnosis.summary,
        finding_count: findings.len(),
        findings,
        recommendations: diagnosis.recommendations,
    })
}
