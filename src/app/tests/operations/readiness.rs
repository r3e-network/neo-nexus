use super::super::*;
use crate::diagnostics::DiagnosticResolution;
use crate::diagnostics::{CheckSeverity, DiagnosticCheck, NodeDiagnostics};

#[test]
fn readiness_check_filters_selected_node_checks_and_clamps_page() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    let diagnostics = NodeDiagnostics {
        node_id: "node-a".to_string(),
        node_name: "Node A".to_string(),
        score: 50,
        checks: vec![
            check(CheckSeverity::Warning, "Plugin", "RPC disabled"),
            check(CheckSeverity::Critical, "Network", "RPC port blocked"),
            check(CheckSeverity::Pass, "Runtime", "neo-rs ok"),
        ],
    };

    app.readiness_check_severity_filter = Some(CheckSeverity::Critical);
    app.readiness_check_query = "rpc".to_string();
    app.readiness_check_page = 12;

    let visible = app.filtered_readiness_checks(&diagnostics);
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].title, "Network");
    assert_eq!(visible[0].severity, CheckSeverity::Critical);

    app.clamp_readiness_check_page(&diagnostics);
    assert_eq!(app.readiness_check_page, 0);

    Ok(())
}

#[test]
fn readiness_checks_focus_severity_and_select_first_visible_check() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    let diagnostics = NodeDiagnostics {
        node_id: "node-a".to_string(),
        node_name: "Node A".to_string(),
        score: 20,
        checks: vec![
            check(CheckSeverity::Warning, "Plugin", "RPC disabled"),
            check(CheckSeverity::Critical, "Binary", "neo-rs binary missing"),
            check(CheckSeverity::Critical, "Network", "RPC port blocked"),
            check(CheckSeverity::Pass, "Runtime", "neo-rs ok"),
        ],
    };

    app.readiness_check_severity_filter = Some(CheckSeverity::Warning);
    app.readiness_check_query = "plugin".to_string();
    app.readiness_check_page = 5;

    app.focus_readiness_check_severity(&diagnostics, CheckSeverity::Critical);

    assert_eq!(
        app.readiness_check_severity_filter,
        Some(CheckSeverity::Critical)
    );
    assert!(app.readiness_check_query.is_empty());
    assert_eq!(app.readiness_check_page, 0);
    let critical_checks = app.filtered_readiness_checks(&diagnostics);
    let selected = app
        .selected_visible_readiness_check(&critical_checks)
        .expect("focused critical check should be selected");
    assert_eq!(selected.title, "Binary");
    assert_eq!(selected.detail, "neo-rs binary missing");
    assert!(app.has_active_readiness_check_filter());

    app.clear_readiness_check_filters(&diagnostics);

    assert!(!app.has_active_readiness_check_filter());
    assert_eq!(app.filtered_readiness_checks(&diagnostics).len(), 4);
    assert!(app
        .selected_visible_readiness_check(&app.filtered_readiness_checks(&diagnostics))
        .is_some());

    Ok(())
}

fn check(severity: CheckSeverity, title: &'static str, detail: &str) -> DiagnosticCheck {
    DiagnosticCheck::new(severity, title, detail, DiagnosticResolution::Operations)
}
