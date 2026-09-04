use super::*;
use crate::{
    catalog::PluginState,
    config::ConfigExporter,
    types::{Network, NodeStatus},
};

fn node() -> NodeConfig {
    NodeConfig {
        id: "node".into(),
        name: "node".into(),
        node_type: NodeType::NeoCli,
        network: Network::Mainnet,
        binary_path: "neo-cli".into(),
        args: Vec::new(),
        runtime_version: "3.10.1".into(),
        storage_engine: NodeType::NeoCli.default_storage_engine(),
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

#[test]
fn signclient_only_connects_to_a_credential_free_loopback_bridge() {
    for endpoint in [
        "http://127.0.0.1:9991",
        "http://[::1]:9991",
        "http://localhost:9991",
    ] {
        assert!(SignClientSettings {
            endpoint: endpoint.into(),
            ..Default::default()
        }
        .validate()
        .is_ok());
    }
    for endpoint in [
        "https://remote.example:9991",
        "http://0.0.0.0:9991",
        "http://127.0.0.1:9991/v1",
        "http://token@127.0.0.1:9991",
        "http://127.0.0.1:9991?token=secret",
    ] {
        assert!(SignClientSettings {
            endpoint: endpoint.into(),
            ..Default::default()
        }
        .validate()
        .is_err());
    }
}

#[test]
fn upstream_json_comments_do_not_corrupt_endpoint_urls() {
    let parsed: Value = serde_json::from_str(&without_comments(
        r#"{
        "PluginConfiguration": {
            "Name": "SignClient", /* operator note */
            "Endpoint": "http://127.0.0.1:9991" // official endpoint comment
        }
    }"#,
    ))
    .unwrap();
    assert_eq!(
        parsed["PluginConfiguration"]["Endpoint"],
        "http://127.0.0.1:9991"
    );
}

#[test]
fn signer_endpoint_is_injected_without_secrets_and_survives_managed_launch_render() {
    let directory = tempfile::tempdir().unwrap();
    let mut node = node();
    let path = ConfigExporter::managed_target_path(directory.path(), &node);
    let plugins = [PluginState {
        plugin_id: PluginId::SignClient,
        enabled: true,
    }];
    ConfigExporter::write_node_config_to_path(&path, &node, &plugins).unwrap();
    let plugin_path = directory.path().join("Plugins/SignClient/SignClient.json");
    let original = std::fs::read_to_string(&plugin_path).unwrap();
    assert!(original.contains("http://127.0.0.1:9991"));
    let settings = SignClientSettings {
        name: "validator1".into(),
        endpoint: "http://127.0.0.1:9992".into(),
    };
    let backup = settings
        .write_for_node(directory.path(), &node)
        .unwrap()
        .unwrap();
    assert_eq!(std::fs::read_to_string(backup).unwrap(), original);
    ConfigExporter::write_node_config_to_path(&path, &node, &plugins).unwrap();
    let configured: Value = serde_json::from_slice(&std::fs::read(&plugin_path).unwrap()).unwrap();
    assert_eq!(configured["PluginConfiguration"]["Name"], "validator1");
    assert_eq!(
        configured["PluginConfiguration"]["Endpoint"],
        "http://127.0.0.1:9992"
    );
    assert_eq!(
        configured["PluginConfiguration"].as_object().unwrap().len(),
        2
    );
    node.runtime_version = "3.11.0".into();
    assert!(ConfigExporter::write_node_config_to_path(&path, &node, &plugins).is_err());
    assert!(crate::config::config_conflict(&plugin_path)
        .unwrap()
        .is_some());
}

#[test]
fn signclient_settings_never_apply_to_a_different_node_family() {
    let directory = tempfile::tempdir().unwrap();
    let mut node = node();
    for kind in [
        NodeType::NeoGo,
        NodeType::NeoRs,
        NodeType::NeoXGeth,
        NodeType::NeoXReth,
    ] {
        node.node_type = kind;
        assert!(SignClientSettings::default()
            .write_for_node(directory.path(), &node)
            .is_err());
        assert!(!crate::catalog::PluginCatalog
            .for_node_type(kind)
            .iter()
            .any(|plugin| plugin.id == PluginId::SignClient));
    }
}
