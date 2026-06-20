use anyhow::Result;

use crate::diagnostics::{
    CheckSeverity, DiagnosticCheck, DiagnosticResolution, FleetDiagnostics, NodeDiagnostics,
};

use super::WorkspaceReadinessReporter;

#[test]
fn readiness_report_writes_text_and_json_evidence() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let diagnostics = FleetDiagnostics {
        score: 65,
        ready_nodes: 0,
        warning_count: 1,
        critical_count: 1,
        nodes: vec![NodeDiagnostics {
            node_id: "node-1".to_string(),
            node_name: "blocked validator".to_string(),
            score: 65,
            checks: vec![
                DiagnosticCheck::new(
                    CheckSeverity::Critical,
                    "Binary path",
                    "neo-node was not found.",
                    DiagnosticResolution::RuntimeManager,
                ),
                DiagnosticCheck::new(
                    CheckSeverity::Warning,
                    "Version",
                    "Runtime follows latest.",
                    DiagnosticResolution::RuntimeManager,
                ),
            ],
        }],
    };

    let export = WorkspaceReadinessReporter::write_at(
        temp_dir.path(),
        temp_dir.path().join("neonexus.db"),
        &diagnostics,
        "test-version",
        1_800_000_000,
    )?;

    assert_eq!(export.report.status, "blocked");
    assert_eq!(export.report.exit_code(), 1);
    assert!(export.text_path.is_file());
    assert!(export.json_path.is_file());

    let text = std::fs::read_to_string(export.text_path)?;
    assert!(text.contains("workspace-readiness-report: blocked"));
    assert!(text.contains("finding: critical | blocked validator | Binary path"));
    assert!(text.contains("resolve: Open Runtimes"));
    assert!(text.contains("next: Install, verify, or apply node runtime binaries."));

    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(export.json_path)?)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["application_version"], "test-version");
    assert_eq!(value["status"], "blocked");
    assert_eq!(value["findings"][0]["severity"], "critical");
    assert_eq!(value["findings"][0]["resolution_key"], "runtime-manager");
    assert_eq!(value["findings"][0]["resolution"], "Runtimes");
    assert_eq!(value["findings"][0]["resolution_action"], "Open Runtimes");
    assert_eq!(
        value["findings"][0]["resolution_hint"],
        "Install, verify, or apply node runtime binaries."
    );
    assert_eq!(value["nodes"][0]["checks"][1]["title"], "Version");
    assert_eq!(
        value["nodes"][0]["checks"][1]["resolution_key"],
        "runtime-manager"
    );
    assert_eq!(value["nodes"][0]["checks"][1]["resolution"], "Runtimes");
    assert_eq!(
        value["nodes"][0]["checks"][1]["resolution_action"],
        "Open Runtimes"
    );
    assert_eq!(
        value["nodes"][0]["checks"][1]["resolution_hint"],
        "Install, verify, or apply node runtime binaries."
    );
    Ok(())
}
