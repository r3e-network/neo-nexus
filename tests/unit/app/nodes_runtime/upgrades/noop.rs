use super::super::super::*;

#[test]
fn fleet_catalog_upgrade_records_missing_catalog_attempt() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.upgrade_fleet_nodes_from_catalog();

    let notice = app
        .notice
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("missing catalog should report a notice"))?;
    assert!(notice.contains("Load a runtime catalog"), "{notice}");
    let fleet_events = app.repository.list_events(RuntimeEventFilter::new(
        Some(EventSeverity::Warning),
        "runtime-fleet-upgrade-run",
        10,
    ))?;
    assert!(fleet_events.iter().any(|event| {
        event.kind.label() == "runtime-fleet-upgrade-run"
            && event.message.contains("Load a runtime catalog")
    }));

    Ok(())
}

#[test]
fn fleet_catalog_upgrade_records_noop_skip_breakdown() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let blocked = repository.create_node(NewNode {
        name: "blocked fleet neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 22832,
        p2p_port: 22833,
        ws_port: None,
    })?;
    repository.create_node(NewNode {
        name: "current fleet neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        runtime_version: "v1.2.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 22932,
        p2p_port: 22933,
        ws_port: None,
    })?;

    let mut app = NeoNexusApp::new(repository);
    let active_blocked = app
        .nodes
        .iter_mut()
        .find(|node| node.id == blocked.id)
        .ok_or_else(|| anyhow::anyhow!("blocked node should be loaded"))?;
    active_blocked.status = NodeStatus::Starting;
    active_blocked.pid = Some(42_425);
    app.runtime_catalog = Some(RuntimeReleaseCatalog {
        schema_version: 1,
        generated_at_unix: Some(1_800_000_000),
        releases: vec![RuntimeRelease {
            id: "neo-rs-v1-2".to_string(),
            label: "neo-rs v1.2.0".to_string(),
            node_type: NodeType::NeoRs,
            version: "v1.2.0".to_string(),
            platform: RuntimePlatform::current(),
            url: "https://downloads.example.com/neo-rs-v1-2".to_string(),
            file_name: "neo-rs-v1-2.bin".to_string(),
            executable_name: "neo-node".to_string(),
            expected_sha256: "b".repeat(64),
            max_bytes: 1024,
        }],
    });

    app.upgrade_fleet_nodes_from_catalog();

    let notice = app
        .notice
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("fleet no-op should report a notice"))?;
    assert!(notice.contains("No fleet nodes have newer compatible catalog releases"));
    assert!(notice.contains("0 stopped direct"), "{notice}");
    assert!(notice.contains("0 running rollout"), "{notice}");
    assert!(notice.contains("1 blocked active"), "{notice}");
    assert!(notice.contains("1 current/unavailable"), "{notice}");
    let fleet_events = app.repository.list_events(RuntimeEventFilter::new(
        None,
        "runtime-fleet-upgrade-run",
        10,
    ))?;
    assert!(fleet_events.iter().any(|event| {
        event.kind.label() == "runtime-fleet-upgrade-run"
            && event.message.contains("No fleet nodes")
            && event.message.contains("1 blocked active")
            && event.message.contains("1 current/unavailable")
    }));

    Ok(())
}
