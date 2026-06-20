use super::super::super::model::{LogDiagnosisStatus, LogFinding};

pub(super) fn finding(
    label: &str,
    line_number: usize,
    line: &str,
    recommendation: &str,
    status: LogDiagnosisStatus,
) -> LogFinding {
    LogFinding {
        label: label.to_string(),
        line_number,
        excerpt: line.to_string(),
        recommendation: recommendation.to_string(),
        status,
    }
}
