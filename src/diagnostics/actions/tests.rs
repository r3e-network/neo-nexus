use super::*;
use crate::diagnostics::{DiagnosticCheck, DiagnosticResolution, NodeDiagnostics};

mod resolution_counts;
mod severity_counts;

#[test]
fn readiness_actions_prioritize_critical_low_score_nodes() {
    let diagnostics = fleet(vec![
        node(
            "node-b",
            "Beta",
            70,
            vec![check(CheckSeverity::Critical, "binary", "missing")],
        ),
        node(
            "node-a",
            "Alpha",
            25,
            vec![check(CheckSeverity::Critical, "ports", "blocked")],
        ),
        node(
            "node-c",
            "Gamma",
            10,
            vec![check(CheckSeverity::Warning, "plugins", "disabled")],
        ),
    ]);

    let actions = filter_readiness_actions(&diagnostics, &ReadinessActionFilter::default());

    assert_eq!(actions[0].node_id, "node-a");
    assert_eq!(actions[1].node_id, "node-b");
    assert_eq!(actions[2].node_id, "node-c");
}

#[test]
fn readiness_actions_filter_by_severity_and_query() {
    let diagnostics = fleet(vec![
        node(
            "validator",
            "Validator",
            50,
            vec![
                check(CheckSeverity::Warning, "config", "slow state sync"),
                check(CheckSeverity::Pass, "runtime", "ok"),
            ],
        ),
        node(
            "rpc",
            "RPC",
            15,
            vec![check(CheckSeverity::Critical, "binary", "missing")],
        ),
    ]);

    let warnings = filter_readiness_actions(
        &diagnostics,
        &ReadinessActionFilter::new(Some(CheckSeverity::Warning), "sync"),
    );

    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].node_id, "validator");
    assert_eq!(warnings[0].title, "config");
}

#[test]
fn readiness_actions_filter_by_resolution_handoff_metadata() {
    let diagnostics = fleet(vec![node(
        "validator",
        "Validator",
        50,
        vec![
            check_with_resolution(
                CheckSeverity::Critical,
                "binary",
                "missing",
                DiagnosticResolution::RuntimeManager,
            ),
            check_with_resolution(
                CheckSeverity::Warning,
                "plugin",
                "disabled",
                DiagnosticResolution::PluginManager,
            ),
        ],
    )]);

    for query in ["runtime-manager", "Open Runtimes", "apply node runtime"] {
        let actions =
            filter_readiness_actions(&diagnostics, &ReadinessActionFilter::new(None, query));

        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].title, "binary");
    }
}

#[test]
fn readiness_actions_filter_by_resolution_facet() {
    let diagnostics = fleet(vec![node(
        "validator",
        "Validator",
        50,
        vec![
            check_with_resolution(
                CheckSeverity::Critical,
                "binary",
                "missing",
                DiagnosticResolution::RuntimeManager,
            ),
            check_with_resolution(
                CheckSeverity::Warning,
                "plugin",
                "disabled",
                DiagnosticResolution::PluginManager,
            ),
        ],
    )]);

    let actions = filter_readiness_actions(
        &diagnostics,
        &ReadinessActionFilter::new(None, "")
            .with_resolution(Some(DiagnosticResolution::PluginManager)),
    );

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0].title, "plugin");
}

fn fleet(nodes: Vec<NodeDiagnostics>) -> FleetDiagnostics {
    FleetDiagnostics {
        score: 0,
        ready_nodes: 0,
        warning_count: 0,
        critical_count: 0,
        nodes,
    }
}

fn node(id: &str, name: &str, score: usize, checks: Vec<DiagnosticCheck>) -> NodeDiagnostics {
    NodeDiagnostics {
        node_id: id.to_string(),
        node_name: name.to_string(),
        score,
        checks,
    }
}

fn check(severity: CheckSeverity, title: &'static str, detail: &str) -> DiagnosticCheck {
    DiagnosticCheck::new(severity, title, detail, DiagnosticResolution::Operations)
}

fn check_with_resolution(
    severity: CheckSeverity,
    title: &'static str,
    detail: &str,
    resolution: DiagnosticResolution,
) -> DiagnosticCheck {
    DiagnosticCheck::new(severity, title, detail, resolution)
}
