use super::*;

#[test]
fn readiness_action_severity_counts_preserve_query_and_resolution_facets() {
    let diagnostics = fleet(vec![
        node(
            "validator",
            "Validator",
            30,
            vec![
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
                    CheckSeverity::Critical,
                    "Runtime",
                    "neo-rs missing",
                    DiagnosticResolution::RuntimeManager,
                ),
            ],
        ),
        node(
            "rpc",
            "RPC",
            50,
            vec![check_with_resolution(
                CheckSeverity::Warning,
                "Plugin",
                "plugin outdated",
                DiagnosticResolution::PluginManager,
            )],
        ),
    ]);

    let counts = readiness_action_severity_counts(
        &diagnostics,
        &ReadinessActionFilter::new(Some(CheckSeverity::Critical), "plugin")
            .with_resolution(Some(DiagnosticResolution::PluginManager)),
    );

    assert_eq!(count_for(&counts, CheckSeverity::Critical), 1);
    assert_eq!(count_for(&counts, CheckSeverity::Warning), 2);
    assert_eq!(count_for(&counts, CheckSeverity::Info), 0);
    assert_eq!(count_for(&counts, CheckSeverity::Pass), 0);
}

fn count_for(counts: &[(CheckSeverity, usize)], severity: CheckSeverity) -> usize {
    counts
        .iter()
        .find_map(|(counted, count)| (*counted == severity).then_some(*count))
        .unwrap_or(0)
}
