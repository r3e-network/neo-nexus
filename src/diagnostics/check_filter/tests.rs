use super::*;
use crate::diagnostics::DiagnosticResolution;

#[test]
fn diagnostic_check_filter_sorts_by_severity_then_title() {
    let checks = vec![
        check(CheckSeverity::Warning, "Plugin", "disabled"),
        check(CheckSeverity::Critical, "Binary", "missing"),
        check(CheckSeverity::Critical, "Config", "invalid"),
    ];

    let rows = filter_diagnostic_checks(&checks, &DiagnosticCheckFilter::default());

    assert_eq!(rows[0].title, "Binary");
    assert_eq!(rows[1].title, "Config");
    assert_eq!(rows[2].title, "Plugin");
}

#[test]
fn diagnostic_check_filter_applies_severity_and_query() {
    let checks = vec![
        check(CheckSeverity::Warning, "Plugin", "RPC disabled"),
        check(CheckSeverity::Critical, "Network", "RPC port blocked"),
        check(CheckSeverity::Pass, "Runtime", "neo-rs ok"),
    ];

    let rows = filter_diagnostic_checks(
        &checks,
        &DiagnosticCheckFilter::new(Some(CheckSeverity::Critical), "rpc"),
    );

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].title, "Network");
}

fn check(severity: CheckSeverity, title: &'static str, detail: &str) -> DiagnosticCheck {
    DiagnosticCheck::new(severity, title, detail, DiagnosticResolution::Operations)
}
