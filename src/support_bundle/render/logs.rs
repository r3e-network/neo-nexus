use anyhow::Result;

use crate::support_bundle::{SupportBundleLogDiagnosisReport, LOG_DIAGNOSIS_MAX_EXCERPT_CHARS};

pub(in crate::support_bundle) fn support_log_diagnosis_json(
    report: &SupportBundleLogDiagnosisReport,
) -> Result<String> {
    Ok(format!("{}\n", serde_json::to_string_pretty(report)?))
}

pub(in crate::support_bundle) fn support_log_diagnosis_text(
    report: &SupportBundleLogDiagnosisReport,
) -> String {
    let mut lines = vec![
        format!("log-diagnosis: {} nodes", report.node_count),
        format!("log-dir: {}", report.log_dir),
        format!("critical: {}", report.critical_count),
        format!("warnings: {}", report.warning_count),
    ];
    if report.entries.is_empty() {
        lines.push("node: none".to_string());
    } else {
        for entry in &report.entries {
            lines.push(format!(
                "node: {} | {} | status={} | findings={} | bytes={} | truncated={}",
                entry.node_name,
                entry.node_type,
                entry.status,
                entry.finding_count,
                entry.bytes,
                entry.truncated,
            ));
            lines.push(format!("summary: {}", entry.summary));
            for finding in &entry.findings {
                lines.push(format!(
                    "finding: line {} | {} | {}",
                    finding.line_number, finding.label, finding.recommendation
                ));
            }
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

pub(in crate::support_bundle) fn truncate_log_excerpt(value: &str) -> String {
    if value.chars().count() <= LOG_DIAGNOSIS_MAX_EXCERPT_CHARS {
        return value.to_string();
    }

    let kept = LOG_DIAGNOSIS_MAX_EXCERPT_CHARS.saturating_sub(3);
    let prefix = value.chars().take(kept).collect::<String>();
    format!("{prefix}...")
}
