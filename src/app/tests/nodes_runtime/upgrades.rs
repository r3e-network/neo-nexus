use super::super::*;

#[cfg(unix)]
#[test]
fn running_catalog_upgrade_applies_runtime_and_restarts_selected_node() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let node = repository.create_node(NewNode {
        name: "running neo-rs upgrade".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: vec![
            "-c".to_string(),
            "echo app-runtime-upgrade-test; while true; do sleep 1; done".to_string(),
        ],
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 21332,
        p2p_port: 21333,
        ws_port: None,
    })?;
    let mut app = NeoNexusApp::new(repository);
    app.selected_node = Some(node.id.clone());

    app.start_selected_node();
    let first_pid = app
        .selected_node()
        .and_then(|node| node.pid)
        .ok_or_else(|| anyhow::anyhow!("start should record a PID"))?;

    let platform = RuntimePlatform::current();
    let release = RuntimeRelease {
        id: "neo-rs-v1-1".to_string(),
        label: "neo-rs v1.1.0".to_string(),
        node_type: NodeType::NeoRs,
        version: "v1.1.0".to_string(),
        platform: platform.clone(),
        url: "https://downloads.example.com/neo-rs-v1-1".to_string(),
        file_name: "neo-rs-v1-1.bin".to_string(),
        executable_name: "neo-node".to_string(),
        expected_sha256: "a".repeat(64),
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
            sha256: "a".repeat(64),
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

    app.upgrade_selected_node_from_catalog();

    let upgraded = app
        .selected_node()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("node should remain selected"))?;
    let second_pid = upgraded
        .pid
        .ok_or_else(|| anyhow::anyhow!("upgrade restart should record a PID"))?;

    assert_eq!(upgraded.status, NodeStatus::Running);
    assert_ne!(first_pid, second_pid);
    assert_eq!(upgraded.runtime_version, "v1.1.0");
    assert_eq!(upgraded.binary_path, PathBuf::from("/bin/sh"));
    assert!(app.notice.as_deref().is_some_and(|notice| {
        notice.contains("upgraded from v1.0.0 to v1.1.0") && notice.contains("restarted")
    }));
    let runtime_events =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "runtime-applied", 10))?;
    assert!(runtime_events.iter().any(|event| {
        event.kind == EventKind::RuntimeApplied
            && event.node_id.as_deref() == Some(node.id.as_str())
    }));
    let restart_events =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "node-restarted", 10))?;
    assert!(restart_events.iter().any(|event| {
        event.kind == EventKind::NodeRestarted && event.node_id.as_deref() == Some(node.id.as_str())
    }));

    app.stop_selected_node();

    Ok(())
}
