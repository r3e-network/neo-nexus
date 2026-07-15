use super::super::super::*;

#[test]
fn fleet_catalog_upgrade_rolls_running_nodes_with_restart() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let running = repository.create_node(NewNode {
        name: "running fleet neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: vec![
            "-c".to_string(),
            "echo fleet-runtime-upgrade-test; while true; do sleep 1; done".to_string(),
        ],
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 22332,
        p2p_port: 22333,
        ws_port: None,
    })?;
    let stopped = repository.create_node(NewNode {
        name: "stopped fleet neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 22432,
        p2p_port: 22433,
        ws_port: None,
    })?;
    let starting = repository.create_node(NewNode {
        name: "starting fleet neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 22632,
        p2p_port: 22633,
        ws_port: None,
    })?;
    let current = repository.create_node(NewNode {
        name: "current fleet neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        runtime_version: "v1.2.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 22732,
        p2p_port: 22733,
        ws_port: None,
    })?;
    let mut app = NeoNexusApp::new(repository);
    app.fleet.selected_node = Some(running.id.clone());
    app.start_selected_node();
    let active_blocked = app.fleet
        .nodes
        .iter_mut()
        .find(|node| node.id == starting.id)
        .ok_or_else(|| anyhow::anyhow!("starting node should be loaded"))?;
    active_blocked.status = NodeStatus::Starting;
    active_blocked.pid = Some(42_424);
    let first_pid = app
        .selected_node()
        .and_then(|node| node.pid)
        .ok_or_else(|| anyhow::anyhow!("start should record a PID"))?;

    let platform = RuntimePlatform::current();
    let release = RuntimeRelease {
        id: "neo-rs-v1-2".to_string(),
        label: "neo-rs v1.2.0".to_string(),
        node_type: NodeType::NeoRs,
        version: "v1.2.0".to_string(),
        platform: platform.clone(),
        url: "https://downloads.example.com/neo-rs-v1-2".to_string(),
        file_name: "neo-rs-v1-2.bin".to_string(),
        executable_name: "neo-node".to_string(),
        expected_sha256: "b".repeat(64),
        max_bytes: 1024,
    };
    app.repository
        .upsert_runtime_installation(&RuntimeInstallation {
            package_id: release.id.clone(),
            label: release.label.clone(),
            node_type: NodeType::NeoRs,
            version: release.version.clone(),
            platform,
            binary_path: PathBuf::from("/bin/sh"),
            sha256: "b".repeat(64),
            signature_verified: false,
            signer_public_key: None,
            bytes: 1024,
            installed_at_unix: 1_800_000_000,
        })?;
    app.runtime_catalog = Some(RuntimeReleaseCatalog {
        schema_version: 1,
        generated_at_unix: Some(1_800_000_000),
        releases: vec![release],
    });

    app.upgrade_fleet_nodes_from_catalog();

    let running_after = app.fleet
        .nodes
        .iter()
        .find(|node| node.id == running.id)
        .ok_or_else(|| anyhow::anyhow!("running node should remain present"))?;
    let stopped_after = app.fleet
        .nodes
        .iter()
        .find(|node| node.id == stopped.id)
        .ok_or_else(|| anyhow::anyhow!("stopped node should remain present"))?;
    let second_pid = running_after
        .pid
        .ok_or_else(|| anyhow::anyhow!("fleet running upgrade should record a replacement PID"))?;

    assert_eq!(running_after.status, NodeStatus::Running);
    assert_ne!(first_pid, second_pid);
    assert_eq!(running_after.runtime_version, "v1.2.0");
    assert_eq!(stopped_after.runtime_version, "v1.2.0");
    let notice = app.session
        .notice
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("fleet upgrade should report a notice"))?;
    assert!(
        notice.contains("Fleet catalog upgrade applied to 2 nodes"),
        "{notice}"
    );
    assert!(notice.contains("1 stopped direct"), "{notice}");
    assert!(notice.contains("1 running rollout"), "{notice}");
    assert!(notice.contains("1 blocked active"), "{notice}");
    assert!(notice.contains("1 current/unavailable"), "{notice}");
    let fleet_events = app.repository.list_events(RuntimeEventFilter::new(
        None,
        "runtime-fleet-upgrade-run",
        10,
    ))?;
    assert!(fleet_events.iter().any(|event| {
        event.kind.label() == "runtime-fleet-upgrade-run"
            && event.message.contains("2 nodes")
            && event.message.contains("1 stopped direct")
            && event.message.contains("1 running rollout")
            && event.message.contains("1 blocked active")
            && event.message.contains("1 current/unavailable")
    }));
    assert!(app.fleet
        .nodes
        .iter()
        .find(|node| node.id == current.id)
        .is_some_and(|node| node.runtime_version == "v1.2.0"));

    app.fleet.selected_node = Some(running.id);
    app.stop_selected_node();

    Ok(())
}
