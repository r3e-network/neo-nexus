use super::super::super::*;

#[test]
fn runtime_upgrade_policy_rolls_running_fleet_nodes() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let node = repository.create_node(NewNode {
        name: "policy fleet neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: vec![
            "-c".to_string(),
            "echo policy-runtime-upgrade-test; while true; do sleep 1; done".to_string(),
        ],
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 22532,
        p2p_port: 22533,
        ws_port: None,
    })?;

    let platform = RuntimePlatform::current();
    let release = RuntimeRelease {
        id: "neo-rs-v1-3".to_string(),
        label: "neo-rs v1.3.0".to_string(),
        node_type: NodeType::NeoRs,
        version: "v1.3.0".to_string(),
        platform: platform.clone(),
        url: "https://downloads.example.com/neo-rs-v1-3".to_string(),
        file_name: "neo-rs-v1-3.bin".to_string(),
        executable_name: "neo-node".to_string(),
        expected_sha256: "c".repeat(64),
        max_bytes: 1024,
    };
    let catalog_path = temp_dir.path().join("runtime-catalog.json");
    std::fs::write(
        &catalog_path,
        serde_json::json!({
            "schema_version": 1,
            "generated_at_unix": 1_800_000_000u64,
            "releases": [{
                "id": release.id,
                "label": release.label,
                "node_type": "neo-rs",
                "version": release.version,
                "platform": {
                    "os": release.platform.os,
                    "arch": release.platform.arch
                },
                "url": release.url,
                "file_name": release.file_name,
                "executable_name": release.executable_name,
                "expected_sha256": release.expected_sha256,
                "max_bytes": release.max_bytes
            }]
        })
        .to_string(),
    )?;
    repository.upsert_runtime_catalog_profile(&RuntimeCatalogProfile {
        id: "policy-catalog".to_string(),
        label: "Policy catalog".to_string(),
        source: catalog_path.display().to_string(),
        signature_source: None,
        ed25519_public_key: None,
        max_bytes: 1024 * 1024,
        enabled: true,
        last_loaded_at_unix: None,
        last_signature_verified: None,
        last_bytes: None,
    })?;
    repository.upsert_runtime_installation(&RuntimeInstallation {
        package_id: "neo-rs-v1-3".to_string(),
        label: "neo-rs v1.3.0".to_string(),
        node_type: NodeType::NeoRs,
        version: "v1.3.0".to_string(),
        platform,
        binary_path: PathBuf::from("/bin/sh"),
        sha256: "c".repeat(64),
        signature_verified: false,
        signer_public_key: None,
        bytes: 1024,
        installed_at_unix: 1_800_000_000,
    })?;

    let mut app = NeoNexusApp::new(repository);
    app.selected_node = Some(node.id.clone());
    app.start_selected_node();
    let first_pid = app
        .selected_node()
        .and_then(|node| node.pid)
        .ok_or_else(|| anyhow::anyhow!("start should record a PID"))?;
    app.runtime_upgrade_policy = RuntimeUpgradePolicy {
        enabled: true,
        catalog_profile_id: Some("policy-catalog".to_string()),
        interval_minutes: RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES,
        require_signed_catalog: false,
        max_nodes_per_run: 1,
        maintenance_window_enabled: false,
        maintenance_window_start_minute_utc: 0,
        maintenance_window_end_minute_utc: 6 * 60,
        wave_delay_minutes: 0,
        last_checked_at_unix: None,
        last_applied_at_unix: None,
    };

    app.run_runtime_upgrade_policy_now();

    let upgraded = app
        .selected_node()
        .ok_or_else(|| anyhow::anyhow!("node should remain selected"))?;
    let second_pid = upgraded
        .pid
        .ok_or_else(|| anyhow::anyhow!("policy upgrade should record a replacement PID"))?;
    assert_eq!(upgraded.status, NodeStatus::Running);
    assert_ne!(first_pid, second_pid);
    assert_eq!(upgraded.runtime_version, "v1.3.0");
    assert!(app.notice.as_deref().is_some_and(|notice| {
        notice.contains("Runtime upgrade policy manual run")
            && notice.contains("1 upgraded")
            && notice.contains("1 ready")
            && notice.contains("planned 1 (0 stopped, 1 running)")
    }));
    assert!(app
        .repository
        .load_runtime_upgrade_policy()?
        .last_applied_at_unix
        .is_some());

    app.stop_selected_node();

    Ok(())
}
