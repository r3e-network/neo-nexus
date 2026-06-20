use super::super::*;
use crate::diagnostics::{
    CheckSeverity, DiagnosticCheck, FleetDiagnostics, NodeDiagnostics, ReadinessAction,
};

#[test]
fn action_queue_filters_readiness_actions_and_clamps_page() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    let diagnostics = FleetDiagnostics {
        score: 45,
        ready_nodes: 0,
        warning_count: 1,
        critical_count: 1,
        nodes: vec![
            readiness_node(
                "rpc-node",
                "RPC Node",
                80,
                vec![readiness_check(
                    CheckSeverity::Warning,
                    "Plugin",
                    "Plugin disabled",
                )],
            ),
            readiness_node(
                "validator-node",
                "Validator Node",
                20,
                vec![readiness_check(
                    CheckSeverity::Critical,
                    "Binary",
                    "Binary missing",
                )],
            ),
        ],
    };

    app.action_queue_severity_filter = Some(CheckSeverity::Warning);
    app.action_queue_query = "plugin".to_string();
    app.action_queue_page = 9;

    let visible = app.filtered_readiness_actions(&diagnostics);
    assert_eq!(
        visible,
        vec![ReadinessAction {
            node_id: "rpc-node".to_string(),
            node_name: "RPC Node".to_string(),
            node_score: 80,
            severity: CheckSeverity::Warning,
            title: "Plugin".to_string(),
            detail: "Plugin disabled".to_string(),
        }]
    );

    app.clamp_action_queue_page(&diagnostics);
    assert_eq!(app.action_queue_page, 0);

    Ok(())
}

fn readiness_node(
    id: &str,
    name: &str,
    score: usize,
    checks: Vec<DiagnosticCheck>,
) -> NodeDiagnostics {
    NodeDiagnostics {
        node_id: id.to_string(),
        node_name: name.to_string(),
        score,
        checks,
    }
}

fn readiness_check(severity: CheckSeverity, title: &'static str, detail: &str) -> DiagnosticCheck {
    DiagnosticCheck {
        severity,
        title,
        detail: detail.to_string(),
    }
}
