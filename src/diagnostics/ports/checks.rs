use crate::diagnostics::{CheckSeverity, DiagnosticCheck, DiagnosticResolution};

pub(super) fn pass_check(title: &'static str, detail: &'static str) -> Vec<DiagnosticCheck> {
    vec![DiagnosticCheck::new(
        CheckSeverity::Pass,
        title,
        detail,
        DiagnosticResolution::NodeStudio,
    )]
}

pub(super) fn critical_checks(
    title: &'static str,
    details: impl IntoIterator<Item = String>,
) -> Vec<DiagnosticCheck> {
    details
        .into_iter()
        .map(|detail| {
            DiagnosticCheck::new(
                CheckSeverity::Critical,
                title,
                detail,
                DiagnosticResolution::NodeStudio,
            )
        })
        .collect()
}
