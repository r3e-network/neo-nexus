use super::super::super::*;
use super::{readiness_check, readiness_node};
use crate::diagnostics::{CheckSeverity, DiagnosticResolution, FleetDiagnostics, ReadinessAction};

#[test]
fn action_queue_resolution_opens_target_workspace_and_preserves_node() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    for (resolution, view) in [
        (DiagnosticResolution::ConfigWorkspace, View::Config),
        (DiagnosticResolution::Logs, View::Logs),
        (DiagnosticResolution::Monitor, View::Monitor),
        (DiagnosticResolution::NodeStudio, View::Nodes),
        (DiagnosticResolution::Operations, View::Operations),
        (DiagnosticResolution::PluginManager, View::Plugins),
        (DiagnosticResolution::RolePlanner, View::Roles),
        (DiagnosticResolution::RuntimeManager, View::Runtimes),
        (DiagnosticResolution::WalletProfiles, View::Wallets),
    ] {
        let action = readiness_action("node-a", "Validator", "Binary", resolution);

        app.open_readiness_action_resolution(&action);

        assert_eq!(app.selected_view, view);
        assert_eq!(app.selected_node.as_deref(), Some("node-a"));
        assert!(app
            .notice
            .as_deref()
            .is_some_and(|notice| notice.contains(resolution.label())));
        assert!(app
            .selected_readiness_action
            .as_ref()
            .is_some_and(|key| key.matches(&action)));
    }

    Ok(())
}

#[test]
fn action_queue_sets_resolution_filter_without_clearing_other_facets() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    let diagnostics = FleetDiagnostics {
        score: 30,
        ready_nodes: 0,
        warning_count: 1,
        critical_count: 1,
        nodes: vec![readiness_node(
            "validator-node",
            "Validator Node",
            20,
            vec![
                readiness_check(CheckSeverity::Warning, "Plugin", "Plugin disabled"),
                readiness_check(CheckSeverity::Critical, "Binary", "Binary missing"),
            ],
        )],
    };

    app.action_queue_severity_filter = Some(CheckSeverity::Warning);
    app.action_queue_resolution_filter = Some(DiagnosticResolution::PluginManager);
    app.action_queue_query = "plugin".to_string();
    app.action_queue_page = 4;

    app.set_action_queue_resolution_filter(
        &diagnostics,
        Some(DiagnosticResolution::RuntimeManager),
    );

    assert_eq!(
        app.action_queue_resolution_filter,
        Some(DiagnosticResolution::RuntimeManager)
    );
    assert_eq!(
        app.action_queue_severity_filter,
        Some(CheckSeverity::Warning)
    );
    assert_eq!(app.action_queue_query, "plugin");
    assert_eq!(app.action_queue_page, 0);
    let actions = app.filtered_readiness_actions(&diagnostics);
    assert!(app.selected_visible_readiness_action(&actions).is_none());

    Ok(())
}

fn readiness_action(
    node_id: &str,
    node_name: &str,
    title: &str,
    resolution: DiagnosticResolution,
) -> ReadinessAction {
    ReadinessAction {
        node_id: node_id.to_string(),
        node_name: node_name.to_string(),
        node_score: 10,
        severity: CheckSeverity::Critical,
        title: title.to_string(),
        detail: "needs operator action".to_string(),
        resolution,
    }
}
