use super::*;

#[test]
fn local_edits_are_preserved_and_each_runtime_upgrade_requires_review() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    assert!(prepare(&path, b"rpc=1", "1.0").unwrap());
    publish(&path, b"rpc=1", "1.0").unwrap();
    fs::write(&path, b"rpc=9\nsecret=local").unwrap();
    assert!(!prepare(&path, b"rpc=1", "1.0").unwrap());
    assert_eq!(fs::read(&path).unwrap(), b"rpc=9\nsecret=local");
    let conflict = config_conflict(&path).unwrap().unwrap();
    resolve_config_conflict(&path, &conflict.token, true).unwrap();
    assert!(prepare(&path, b"rpc=1", "1.0").unwrap());
    publish(&path, b"rpc=1", "1.0").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"rpc=9\nsecret=local");
    // A version bump requires review even when the static renderer did not change.
    assert!(!prepare(&path, b"rpc=1", "2.0").unwrap());
    let conflict = config_conflict(&path).unwrap().unwrap();
    assert_eq!(conflict.from_version, "1.0");
    assert_eq!(conflict.to_version, "2.0");
    let backup = resolve_config_conflict(&path, &conflict.token, false)
        .unwrap()
        .unwrap();
    assert_eq!(fs::read(backup).unwrap(), b"rpc=9\nsecret=local");
    assert_eq!(fs::read(&path).unwrap(), b"rpc=1");
}

#[test]
fn accepted_local_file_requires_review_when_generated_settings_change() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.json");
    fs::write(&path, b"local").unwrap();
    assert!(!prepare(&path, b"generated", "1").unwrap());
    let conflict = config_conflict(&path).unwrap().unwrap();
    resolve_config_conflict(&path, &conflict.token, true).unwrap();
    assert!(!prepare(&path, b"changed port", "1").unwrap());
}

#[test]
fn stale_review_cannot_overwrite_new_edits_or_new_candidate() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.yml");
    fs::write(&path, b"local").unwrap();
    assert!(!prepare(&path, b"generated", "1").unwrap());
    let conflict = config_conflict(&path).unwrap().unwrap();
    fs::write(&path, b"later edit").unwrap();
    assert!(resolve_config_conflict(&path, &conflict.token, false).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"later edit");
    assert!(!prepare(&path, b"second candidate", "2").unwrap());
    assert!(resolve_config_conflict(&path, &conflict.token, false).is_err());
}

#[test]
fn unmodified_baseline_allows_generated_updates() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("config.toml");
    publish(&path, b"old", "1").unwrap();
    assert!(prepare(&path, b"new", "2").unwrap());
    publish(&path, b"new", "2").unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"new");
}

fn node(node_type: crate::types::NodeType) -> crate::types::NodeConfig {
    crate::types::NodeConfig {
        id: "node".into(),
        name: "node".into(),
        node_type,
        network: crate::types::Network::Mainnet,
        binary_path: "node".into(),
        args: Vec::new(),
        runtime_version: "1.0".into(),
        storage_engine: node_type.default_storage_engine(),
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: crate::types::NodeStatus::Stopped,
        pid: None,
    }
}

#[test]
fn exporter_protects_local_changes_for_every_runtime() {
    use crate::{config::ConfigExporter, types::NodeType};
    for kind in NodeType::ALL {
        let directory = tempfile::tempdir().unwrap();
        let node = node(kind);
        let path = ConfigExporter::managed_target_path(directory.path(), &node);
        ConfigExporter::write_node_config_to_path(&path, &node, &[]).unwrap();
        let mut edited = fs::read(&path).unwrap();
        edited.extend_from_slice(b"\nlocal setting");
        fs::write(&path, &edited).unwrap();
        assert!(
            ConfigExporter::write_node_config_to_path(&path, &node, &[]).is_err(),
            "{kind}"
        );
        assert_eq!(fs::read(&path).unwrap(), edited);
        assert!(config_conflict(&path).unwrap().is_some());
    }
}

#[test]
fn plugin_conflict_prevents_partial_primary_config_update() {
    use crate::{
        catalog::{PluginId, PluginState},
        config::ConfigExporter,
        types::NodeType,
    };
    let directory = tempfile::tempdir().unwrap();
    let mut node = node(NodeType::NeoCli);
    let path = ConfigExporter::managed_target_path(directory.path(), &node);
    let plugins = [PluginState {
        plugin_id: PluginId::RpcServer,
        enabled: true,
    }];
    let export = ConfigExporter::write_node_config_to_path(&path, &node, &plugins).unwrap();
    let primary = fs::read(&path).unwrap();
    fs::write(&export.sidecar_paths[0], b"{\"Password\":\"local secret\"}").unwrap();
    node.p2p_port += 1;
    assert!(ConfigExporter::write_node_config_to_path(&path, &node, &plugins).is_err());
    assert_eq!(fs::read(&path).unwrap(), primary);
    assert!(list_config_conflicts(directory.path()).unwrap().len() == 1);
}

#[cfg(unix)]
#[test]
fn symbolic_links_never_overwrite_other_files() {
    use std::os::unix::fs::symlink;
    let directory = tempfile::tempdir().unwrap();
    let target = directory.path().join("outside");
    fs::write(&target, b"private").unwrap();
    let path = directory.path().join("config.json");
    symlink(&target, &path).unwrap();
    assert!(prepare(&path, b"new", "1").is_err());
    assert_eq!(fs::read(target).unwrap(), b"private");
}
