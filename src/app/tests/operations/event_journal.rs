use super::super::*;

#[test]
fn event_journal_export_action_writes_reports_and_audit_event() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.record_event_at(
        NewRuntimeEvent {
            node_id: Some("node-a".to_string()),
            node_name: Some("validator-a".to_string()),
            kind: EventKind::WatchdogRestarted,
            severity: EventSeverity::Warning,
            message: "restart after abnormal exit".to_string(),
        },
        42,
    )?;
    let mut app = NeoNexusApp::new(repository);
    app.event_severity_filter = Some(EventSeverity::Warning);
    app.event_query = "restart".to_string();

    app.export_event_journal_report();

    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Event journal exported")));

    let event_dir = app.event_journal_export_dir();
    let exported_paths = std::fs::read_dir(event_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    assert!(exported_paths
        .iter()
        .any(|path| path.extension().is_some_and(|extension| extension == "txt")));
    assert!(exported_paths.iter().any(|path| path
        .extension()
        .is_some_and(|extension| extension == "json")));

    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "event-journal", 10))?;
    assert!(events.iter().any(|event| {
        event.kind == EventKind::EventJournalExported && event.severity == EventSeverity::Info
    }));

    Ok(())
}

#[test]
fn event_journal_selection_tracks_visible_events() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let info = repository.record_event_at(
        NewRuntimeEvent {
            node_id: None,
            node_name: None,
            kind: EventKind::WorkspaceIntegrityChecked,
            severity: EventSeverity::Info,
            message: "workspace integrity ok".to_string(),
        },
        10,
    )?;
    let warning = repository.record_event_at(
        NewRuntimeEvent {
            node_id: Some("node-a".to_string()),
            node_name: Some("validator-a".to_string()),
            kind: EventKind::WatchdogRestarted,
            severity: EventSeverity::Warning,
            message: "restart after abnormal exit".to_string(),
        },
        20,
    )?;
    let mut app = NeoNexusApp::new(repository);
    app.selected_event = Some(info.id);
    let filtered = app.repository.list_events(RuntimeEventFilter::new(
        Some(EventSeverity::Warning),
        "restart",
        10,
    ))?;

    app.ensure_valid_event_selection(&filtered);

    assert_eq!(app.selected_event, Some(warning.id));
    let Some(selected) = app.selected_event_from(&filtered) else {
        anyhow::bail!("filtered event should be selected");
    };
    assert_eq!(selected.message, "restart after abnormal exit");

    app.ensure_valid_event_selection(&[]);
    assert_eq!(app.selected_event, None);

    Ok(())
}
