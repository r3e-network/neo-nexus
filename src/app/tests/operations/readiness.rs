use super::super::*;
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

fn check(severity: CheckSeverity, title: &'static str, detail: &str) -> DiagnosticCheck {
    DiagnosticCheck {
        severity,
        title,
        detail: detail.to_string(),
    }
}
