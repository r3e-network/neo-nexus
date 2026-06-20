use crate::diagnostics::{CheckSeverity, DiagnosticCheck};

pub(super) fn pass_check(title: &'static str, detail: &'static str) -> Vec<DiagnosticCheck> {
    vec![DiagnosticCheck {
        severity: CheckSeverity::Pass,
        title,
        detail: detail.to_string(),
    }]
}

pub(super) fn critical_checks(
    title: &'static str,
    details: impl IntoIterator<Item = String>,
) -> Vec<DiagnosticCheck> {
    details
        .into_iter()
        .map(|detail| DiagnosticCheck {
            severity: CheckSeverity::Critical,
            title,
            detail,
        })
        .collect()
}
