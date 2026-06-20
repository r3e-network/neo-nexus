use super::*;
use crate::diagnostics::{DiagnosticCheck, NodeDiagnostics};

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
    DiagnosticCheck {
        severity,
        title,
        detail: detail.to_string(),
    }
}
