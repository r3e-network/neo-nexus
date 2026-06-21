use super::*;

#[test]
fn diagnostic_check_severity_counts_preserve_query_and_resolution_facets() {
    let checks = vec![
        check_with_resolution(
            CheckSeverity::Warning,
            "Plugin",
            "plugin disabled",
            DiagnosticResolution::PluginManager,
        ),
        check_with_resolution(
            CheckSeverity::Critical,
            "Plugin",
            "plugin missing",
            DiagnosticResolution::PluginManager,
        ),
        check_with_resolution(
            CheckSeverity::Pass,
            "Plugin",
            "plugin enabled",
            DiagnosticResolution::PluginManager,
        ),
        check_with_resolution(
            CheckSeverity::Info,
            "Runtime",
            "plugin note",
            DiagnosticResolution::RuntimeManager,
        ),
    ];

    let counts = diagnostic_check_severity_counts(
        &checks,
        &DiagnosticCheckFilter::new(Some(CheckSeverity::Critical), "plugin")
            .with_resolution(Some(DiagnosticResolution::PluginManager)),
    );

    assert_eq!(count_for(&counts, CheckSeverity::Critical), 1);
    assert_eq!(count_for(&counts, CheckSeverity::Warning), 1);
    assert_eq!(count_for(&counts, CheckSeverity::Pass), 1);
    assert_eq!(count_for(&counts, CheckSeverity::Info), 0);
}

fn count_for(counts: &[(CheckSeverity, usize)], severity: CheckSeverity) -> usize {
    counts
        .iter()
        .find_map(|(counted, count)| (*counted == severity).then_some(*count))
        .unwrap_or(0)
}
