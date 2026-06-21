use super::super::*;

#[test]
fn event_journal_report_cli_writes_text_and_json_evidence() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let report_dir = temp_dir.path().join("events");
    let repository = Repository::open(&db_path)?;
    repository.record_event_at(
        NewRuntimeEvent {
            node_id: Some("node-1".to_string()),
            node_name: Some("audit alpha".to_string()),
            kind: EventKind::NodeStarted,
            severity: EventSeverity::Info,
            message: "node started".to_string(),
        },
        1_800_000_001,
    )?;
    repository.record_event_at(
        NewRuntimeEvent {
            node_id: Some("node-1".to_string()),
            node_name: Some("audit alpha".to_string()),
            kind: EventKind::WatchdogScheduled,
            severity: EventSeverity::Warning,
            message: "restart scheduled api_key=abc123 Authorization: Bearer journal-token"
                .to_string(),
        },
        1_800_000_002,
    )?;
    drop(repository);

    let db_arg = db_path.display().to_string();
    let report_arg = report_dir.display().to_string();
    let action = action_from_args([
        "neo-nexus",
        "--export-event-journal",
        &db_arg,
        &report_arg,
        "10",
        "warning",
        "restart",
    ])?;

    assert!(
        matches!(action, CliAction::Print(text) if text.contains("event-journal-report: ok") && text.contains("filter-severity: warning") && text.contains("filter-query: restart") && text.contains("matched-events: 1") && text.contains("exported-events: 1") && text.contains("report-text:") && text.contains("report-json:"))
    );
    let report_files = std::fs::read_dir(&report_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, std::io::Error>>()?;
    assert_eq!(report_files.len(), 2);
    let text_path = report_files
        .iter()
        .find(|path| path.extension().and_then(|extension| extension.to_str()) == Some("txt"))
        .with_context(|| format!("missing text report in {}", report_dir.display()))?;
    let json_path = report_files
        .iter()
        .find(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .with_context(|| format!("missing JSON report in {}", report_dir.display()))?;

    let text = std::fs::read_to_string(text_path)?;
    assert!(text.contains("event-journal-report: ok"));
    assert!(text.contains("restart scheduled"));
    assert!(text.contains("api_key=<redacted>"));
    assert!(text.contains("Authorization:<redacted>"));
    assert!(!text.contains("node started"));
    assert!(!text.contains("abc123"));
    assert!(!text.contains("journal-token"));

    let value: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(json_path)?)?;
    assert_eq!(value["schema_version"], 1);
    assert_eq!(value["database"], db_arg);
    assert_eq!(value["requested_limit"], 10);
    assert_eq!(value["filter"]["severity"], "warning");
    assert_eq!(value["filter"]["query"], "restart");
    assert_eq!(value["matched_event_count"], 1);
    assert_eq!(value["exported_event_count"], 1);
    assert_eq!(value["events"][0]["kind"], "watchdog-scheduled");
    assert_eq!(value["events"][0]["severity"], "warning");
    assert!(value["events"][0]["message"]
        .as_str()
        .is_some_and(|message| {
            message.contains("restart scheduled")
                && message.contains("api_key=<redacted>")
                && message.contains("Authorization:<redacted>")
                && !message.contains("abc123")
                && !message.contains("journal-token")
        }));
    Ok(())
}
