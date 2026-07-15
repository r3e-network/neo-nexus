use super::super::*;
use crate::diagnostics::{
    CheckSeverity, DiagnosticCheck, DiagnosticResolution, FleetDiagnostics, NodeDiagnostics,
    ReadinessAction,
};

mod resolution;

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

    app.operations_ui.action_queue_severity_filter = Some(CheckSeverity::Warning);
    app.operations_ui.action_queue_query = "plugin".to_string();
    app.operations_ui.action_queue_page = 9;

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
            resolution: DiagnosticResolution::PluginManager,
        }]
    );

    app.clamp_action_queue_page(&diagnostics);
    assert_eq!(app.operations_ui.action_queue_page, 0);

    Ok(())
}

#[test]
fn action_queue_focuses_severity_and_selects_highest_risk_action() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    let diagnostics = FleetDiagnostics {
        score: 30,
        ready_nodes: 0,
        warning_count: 1,
        critical_count: 2,
        nodes: vec![
            readiness_node(
                "rpc-node",
                "RPC Node",
                65,
                vec![readiness_check(
                    CheckSeverity::Critical,
                    "Binary",
                    "RPC binary missing",
                )],
            ),
            readiness_node(
                "validator-node",
                "Validator Node",
                20,
                vec![
                    readiness_check(CheckSeverity::Warning, "Plugin", "Plugin disabled"),
                    readiness_check(
                        CheckSeverity::Critical,
                        "Ports",
                        "Validator RPC port unavailable",
                    ),
                ],
            ),
        ],
    };

    app.operations_ui.action_queue_severity_filter = Some(CheckSeverity::Warning);
    app.operations_ui.action_queue_query = "plugin".to_string();
    app.operations_ui.action_queue_page = 4;

    app.focus_action_queue_severity(&diagnostics, CheckSeverity::Critical);

    assert_eq!(
        app.operations_ui.action_queue_severity_filter,
        Some(CheckSeverity::Critical)
    );
    assert!(app.operations_ui.action_queue_query.is_empty());
    assert_eq!(app.operations_ui.action_queue_page, 0);
    assert_eq!(app.fleet.selected_node.as_deref(), Some("validator-node"));
    let critical_actions = app.filtered_readiness_actions(&diagnostics);
    let selected = app
        .selected_visible_readiness_action(&critical_actions)
        .expect("focused critical action should be selected");
    assert_eq!(selected.node_id, "validator-node");
    assert_eq!(selected.title, "Ports");
    assert!(app.has_active_action_queue_filter());

    app.clear_action_queue_filters(&diagnostics);

    assert!(!app.has_active_action_queue_filter());
    assert_eq!(app.filtered_readiness_actions(&diagnostics).len(), 3);
    assert_eq!(app.fleet.selected_node.as_deref(), Some("validator-node"));

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
    let resolution = match title {
        "Binary" => DiagnosticResolution::RuntimeManager,
        "Plugin" => DiagnosticResolution::PluginManager,
        "Ports" => DiagnosticResolution::NodeStudio,
        _ => DiagnosticResolution::Operations,
    };
    DiagnosticCheck::new(severity, title, detail, resolution)
}
