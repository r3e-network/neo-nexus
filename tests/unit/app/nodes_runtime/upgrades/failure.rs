use super::super::super::*;

#[test]
fn fleet_catalog_upgrade_records_interrupted_batch_audit() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let first = repository.create_node(NewNode {
        name: "a-ready neo-rs".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 23032,
        p2p_port: 23033,
        ws_port: None,
    })?;
    repository.create_node(NewNode {
        name: "b-download-fails neo-go".to_string(),
        node_type: NodeType::NeoGo,
        network: Network::Testnet,
        binary_path: PathBuf::from("/bin/sh"),
        args: Vec::new(),
        runtime_version: "v1.0.0".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 23132,
        p2p_port: 23133,
        ws_port: None,
    })?;

    let platform = RuntimePlatform::current();
    let neo_rs_release = release("neo-rs-v1-2", NodeType::NeoRs, &platform, "b");
    let neo_go_release = RuntimeRelease {
        id: "neo-go-v1-2".to_string(),
        label: "neo-go v1.2.0".to_string(),
        node_type: NodeType::NeoGo,
        version: "v1.2.0".to_string(),
        platform: platform.clone(),
        url: "http://downloads.example.com/neo-go-v1-2".to_string(),
        file_name: "neo-go-v1-2.bin".to_string(),
        executable_name: "neo-go".to_string(),
        expected_sha256: "c".repeat(64),
        max_bytes: 1024,
    };
    repository.upsert_runtime_installation(&RuntimeInstallation {
        package_id: neo_rs_release.id.clone(),
        label: neo_rs_release.label.clone(),
        node_type: NodeType::NeoRs,
        version: neo_rs_release.version.clone(),
        platform: platform.clone(),
        binary_path: PathBuf::from("/bin/sh"),
        sha256: "b".repeat(64),
        signature_verified: false,
        signer_public_key: None,
        bytes: 1024,
        installed_at_unix: 1_800_000_000,
    })?;
    let mut app = NeoNexusApp::new(repository);
    app.runtime_catalog = Some(RuntimeReleaseCatalog {
        schema_version: 1,
        generated_at_unix: Some(1_800_000_000),
        releases: vec![neo_rs_release, neo_go_release],
    });

    app.upgrade_fleet_nodes_from_catalog();

    let upgraded = app
        .nodes
        .iter()
        .find(|node| node.id == first.id)
        .ok_or_else(|| anyhow::anyhow!("first node should remain present"))?;
    assert_eq!(upgraded.runtime_version, "v1.2.0");
    let notice = app
        .notice
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("interrupted fleet run should report a notice"))?;
    assert!(notice.contains("stopped after 1 node"), "{notice}");
    assert!(notice.contains("b-download-fails neo-go"), "{notice}");
    assert!(!notice.contains("stopped after 1 nodes"), "{notice}");
    assert!(notice.contains("1 stopped direct"), "{notice}");
    let events = app.repository.list_events(RuntimeEventFilter::new(
        Some(EventSeverity::Warning),
        "runtime-fleet-upgrade-run",
        10,
    ))?;
    assert!(events.iter().any(|event| {
        event.message.contains("stopped after 1 node")
            && event.message.contains("b-download-fails neo-go")
            && !event.message.contains("stopped after 1 nodes")
            && event.message.contains("1 stopped direct")
    }));

    Ok(())
}

fn release(
    id: &str,
    node_type: NodeType,
    platform: &RuntimePlatform,
    hash_seed: &str,
) -> RuntimeRelease {
    RuntimeRelease {
        id: id.to_string(),
        label: format!("{node_type} v1.2.0"),
        node_type,
        version: "v1.2.0".to_string(),
        platform: platform.clone(),
        url: format!("https://downloads.example.com/{id}"),
        file_name: format!("{id}.bin"),
        executable_name: "neo-node".to_string(),
        expected_sha256: hash_seed.repeat(64),
        max_bytes: 1024,
    }
}
