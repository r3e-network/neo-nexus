use super::super::*;

#[test]
fn backup_validate_latest_action_records_audit_event() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.create_node(NewNode {
        name: "validator".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Private,
        binary_path: PathBuf::from("/usr/local/bin/neo-node"),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 30332,
        p2p_port: 30333,
        ws_port: Some(30334),
    })?;
    let mut app = NeoNexusApp::new(repository);

    app.export_workspace_backup();
    app.validate_latest_workspace_backup();

    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Backup validated: 1 nodes")));
    let Some(validation) = app.last_backup_validation.as_ref() else {
        anyhow::bail!("backup validation evidence should remain visible in app state");
    };
    assert_eq!(validation.node_count, 1);
    assert!(validation.source_path.is_some());
    let events =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "backup-validated", 10))?;
    assert!(events.iter().any(|event| {
        event.kind == EventKind::BackupValidated && event.severity == EventSeverity::Info
    }));

    Ok(())
}

#[test]
fn backup_import_requires_current_latest_validation() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.create_node(NewNode {
        name: "rpc".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Private,
        binary_path: PathBuf::from("/usr/local/bin/neo-node"),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 31332,
        p2p_port: 31333,
        ws_port: Some(31334),
    })?;
    let mut app = NeoNexusApp::new(repository);

    app.export_workspace_backup();
    app.import_latest_workspace_backup();

    assert_eq!(
        app.notice.as_deref(),
        Some("Validate latest workspace backup before importing")
    );
    let imports_before =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "backup-imported", 10))?;
    assert!(imports_before.is_empty());

    app.validate_latest_workspace_backup();
    app.import_latest_workspace_backup();

    assert!(app
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("Backup imported")));
    assert!(app.last_backup_validation.is_none());
    let imports_after =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "backup-imported", 10))?;
    assert!(imports_after
        .iter()
        .any(|event| event.kind == EventKind::BackupImported));

    Ok(())
}
