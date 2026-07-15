use super::super::super::*;
use crate::diagnostics::{CheckSeverity, DiagnosticCheck, DiagnosticResolution, NodeDiagnostics};

#[test]
fn readiness_check_resolution_opens_target_workspace_and_preserves_selection() -> anyhow::Result<()>
{
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
        let diagnostic = NodeDiagnostics {
            node_id: "node-a".to_string(),
            node_name: "Validator".to_string(),
            score: 20,
            checks: vec![DiagnosticCheck::new(
                CheckSeverity::Critical,
                "Binary",
                "neo-rs binary missing",
                resolution,
            )],
        };
        let check = diagnostic.checks[0].clone();

        app.open_readiness_check_resolution(&diagnostic, &check);

        assert_eq!(app.session.selected_view, view);
        assert_eq!(app.fleet.selected_node.as_deref(), Some("node-a"));
        assert!(app.session
            .notice
            .as_deref()
            .is_some_and(|notice| notice.contains(resolution.label())));
        assert!(app.operations_ui
            .selected_readiness_check
            .as_ref()
            .is_some_and(|key| key.matches(&check)));
    }

    Ok(())
}

#[test]
fn readiness_checks_set_resolution_filter_without_clearing_other_facets() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    let diagnostics = NodeDiagnostics {
        node_id: "node-a".to_string(),
        node_name: "Node A".to_string(),
        score: 20,
        checks: vec![
            DiagnosticCheck::new(
                CheckSeverity::Warning,
                "Plugin",
                "Plugin disabled",
                DiagnosticResolution::PluginManager,
            ),
            DiagnosticCheck::new(
                CheckSeverity::Critical,
                "Binary",
                "neo-rs missing",
                DiagnosticResolution::RuntimeManager,
            ),
        ],
    };

    app.operations_ui.readiness_check_severity_filter = Some(CheckSeverity::Warning);
    app.operations_ui.readiness_check_resolution_filter = Some(DiagnosticResolution::PluginManager);
    app.operations_ui.readiness_check_query = "plugin".to_string();
    app.operations_ui.readiness_check_page = 4;

    app.set_readiness_check_resolution_filter(
        &diagnostics,
        Some(DiagnosticResolution::RuntimeManager),
    );

    assert_eq!(
        app.operations_ui.readiness_check_resolution_filter,
        Some(DiagnosticResolution::RuntimeManager)
    );
    assert_eq!(
        app.operations_ui.readiness_check_severity_filter,
        Some(CheckSeverity::Warning)
    );
    assert_eq!(app.operations_ui.readiness_check_query, "plugin");
    assert_eq!(app.operations_ui.readiness_check_page, 0);
    let checks = app.filtered_readiness_checks(&diagnostics);
    assert!(app.selected_visible_readiness_check(&checks).is_none());

    Ok(())
}
