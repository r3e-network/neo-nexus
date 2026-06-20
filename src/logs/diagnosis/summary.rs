use std::collections::BTreeSet;

use super::{
    super::model::{LogDiagnosis, LogDiagnosisStatus, LogFinding, LogSnapshot},
    classifier::classify_log_line,
};

pub(in crate::logs) fn diagnose_snapshot(snapshot: &LogSnapshot) -> LogDiagnosis {
    if !snapshot.exists {
        return LogDiagnosis {
            status: LogDiagnosisStatus::NoLog,
            summary: "No runtime log has been captured for this node.".to_string(),
            findings: Vec::new(),
            recommendations: vec![
                "Start the node or run Smoke Runtime to create first evidence.".to_string(),
            ],
        };
    }

    if snapshot.lines.is_empty() {
        return LogDiagnosis {
            status: LogDiagnosisStatus::Quiet,
            summary: "The log exists but has no retained runtime output.".to_string(),
            findings: Vec::new(),
            recommendations: vec![
                "If startup failed, run Probe Binary and Smoke Runtime from Node Studio."
                    .to_string(),
            ],
        };
    }

    let findings = collect_findings(snapshot);
    if findings.is_empty() {
        return LogDiagnosis {
            status: LogDiagnosisStatus::Informational,
            summary: "No known failure pattern was found in the retained log window.".to_string(),
            findings,
            recommendations: vec![
                "Use RPC Health or search for runtime-specific errors if the node is unhealthy."
                    .to_string(),
            ],
        };
    }

    let status = findings
        .iter()
        .map(|finding| finding.status)
        .max_by_key(|status| status.rank())
        .unwrap_or(LogDiagnosisStatus::Informational);
    let recommendations = recommendations_for(&findings);
    let primary = findings.first().map_or_else(
        || "runtime output".to_string(),
        |finding| finding.label.clone(),
    );

    LogDiagnosis {
        status,
        summary: format!(
            "Found {} actionable log pattern(s); primary: {primary}.",
            findings.len()
        ),
        findings,
        recommendations,
    }
}

fn collect_findings(snapshot: &LogSnapshot) -> Vec<LogFinding> {
    let mut findings = Vec::new();
    let mut seen_labels = BTreeSet::new();

    for (index, line) in snapshot.lines.iter().enumerate() {
        let Some(finding) = classify_log_line(index + 1, line) else {
            continue;
        };
        if !seen_labels.insert(finding.label.clone()) {
            continue;
        }
        findings.push(finding);
        if findings.len() >= 5 {
            break;
        }
    }

    findings
}

fn recommendations_for(findings: &[LogFinding]) -> Vec<String> {
    findings
        .iter()
        .map(|finding| finding.recommendation.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
