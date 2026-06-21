use super::*;

#[test]
fn readiness_action_resolution_counts_preserve_severity_and_query_facets() {
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

    let counts = readiness_action_resolution_counts(
        &diagnostics,
        &ReadinessActionFilter::new(Some(CheckSeverity::Warning), "plugin")
            .with_resolution(Some(DiagnosticResolution::RuntimeManager)),
    );

    assert_eq!(count_for(&counts, DiagnosticResolution::PluginManager), 2);
    assert_eq!(count_for(&counts, DiagnosticResolution::RuntimeManager), 0);
}

fn count_for(counts: &[(DiagnosticResolution, usize)], resolution: DiagnosticResolution) -> usize {
    counts
        .iter()
        .find_map(|(counted, count)| (*counted == resolution).then_some(*count))
        .unwrap_or(0)
}
