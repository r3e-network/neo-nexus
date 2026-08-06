use super::sidecars_for;
use crate::{
    catalog::PluginId,
    types::{Network, NodeConfig, NodeStatus, NodeType, StorageEngine},
};
use std::path::PathBuf;

fn node() -> NodeConfig {
    NodeConfig {
        id: "n1".to_string(),
        name: "oracle-1".to_string(),
        node_type: NodeType::NeoCli,
        network: Network::Testnet,
        binary_path: PathBuf::from("/opt/neo/neo-cli"),
        args: Vec::new(),
        runtime_version: "3.10.1".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 20332,
        p2p_port: 20333,
        ws_port: Some(20334),
        status: NodeStatus::Stopped,
        pid: None,
    }
}

fn text_for(plugin: PluginId) -> String {
    sidecars_for(&node(), &[plugin])
        .into_iter()
        .next()
        .map(|sidecar| sidecar.text)
        .unwrap_or_default()
}

/// neo-cli looks for `Plugins/<Name>/<Name>.json`. A plugin whose file is
/// written as `config.json` — the pre-3.9 layout — is silently left on its
/// compiled-in defaults.
#[test]
fn each_sidecar_lands_at_the_path_its_plugin_reads() {
    let sidecars = sidecars_for(&node(), &[PluginId::RpcServer, PluginId::OracleService]);
    let paths: Vec<&str> = sidecars
        .iter()
        .map(|sidecar| sidecar.relative_path.as_str())
        .collect();
    assert_eq!(
        paths,
        [
            "Plugins/RpcServer/RpcServer.json",
            "Plugins/OracleService/OracleService.json",
        ],
    );
}

/// The RPC port an operator sets is only honoured here. It used to be written
/// to `ApplicationConfiguration.RPC.Port`, which neo-cli does not read, so the
/// node stayed on the plugin's compiled-in 10332.
#[test]
fn the_rpc_port_reaches_the_listener_that_binds_it() {
    let value: serde_json::Value = serde_json::from_str(&text_for(PluginId::RpcServer)).unwrap();
    assert_eq!(
        value["PluginConfiguration"]["Servers"][0]["Port"],
        node().rpc_port
    );
}

/// `Dependency` is a sibling of `PluginConfiguration`. Nested inside it, the
/// plugin loader never sees it and load order is undefined.
#[test]
fn dependencies_are_declared_beside_the_configuration_not_inside_it() {
    for plugin in [
        PluginId::OracleService,
        PluginId::StateService,
        PluginId::ApplicationLogs,
        PluginId::TokensTracker,
        PluginId::RestServer,
    ] {
        let value: serde_json::Value = serde_json::from_str(&text_for(plugin)).unwrap();
        assert_eq!(
            value["Dependency"][0], "RpcServer",
            "{plugin} must declare its RpcServer dependency at the top level",
        );
        assert!(value["PluginConfiguration"]["Dependency"].is_null());
    }
}

/// Current plugins inherit the network magic from the primary config. Emitting
/// a `Network` key pins the plugin to whatever value we guessed, which on a
/// testnet or private node is the wrong chain.
#[test]
fn no_sidecar_pins_its_own_network_magic() {
    for plugin in [
        PluginId::RpcServer,
        PluginId::OracleService,
        PluginId::StateService,
        PluginId::DBFTPlugin,
    ] {
        let value: serde_json::Value = serde_json::from_str(&text_for(plugin)).unwrap();
        assert!(
            value["PluginConfiguration"]["Network"].is_null(),
            "{plugin} must not carry its own Network key",
        );
    }
}

/// Both duties need a committee designation before they can do anything, so
/// neither may start itself: an undesignated node would log a failure for every
/// request or consensus round.
#[test]
fn chain_duties_do_not_start_themselves() {
    for plugin in [PluginId::OracleService, PluginId::DBFTPlugin] {
        let value: serde_json::Value = serde_json::from_str(&text_for(plugin)).unwrap();
        assert_eq!(value["PluginConfiguration"]["AutoStart"], false);
    }
    let oracle: serde_json::Value =
        serde_json::from_str(&text_for(PluginId::OracleService)).unwrap();
    assert_eq!(
        oracle["PluginConfiguration"]["AutoVerify"],
        serde_json::Value::Null
    );
    let state: serde_json::Value = serde_json::from_str(&text_for(PluginId::StateService)).unwrap();
    assert_eq!(state["PluginConfiguration"]["AutoVerify"], false);
}

/// An oracle with no acceptable content type answers nothing.
#[test]
fn the_oracle_accepts_at_least_one_content_type() {
    let value: serde_json::Value =
        serde_json::from_str(&text_for(PluginId::OracleService)).unwrap();
    let types = value["PluginConfiguration"]["AllowedContentTypes"]
        .as_array()
        .expect("AllowedContentTypes must be a list");
    assert!(!types.is_empty());
}

/// The storage engines are chosen in the primary config; they have no file.
#[test]
fn storage_engines_have_no_sidecar() {
    assert!(sidecars_for(&node(), &[PluginId::LevelDbStore, PluginId::RocksDbStore]).is_empty());
}

/// A disabled plugin must not have its configuration written: the file alone
/// does not enable it, but a stale one misleads the next operator to read it.
#[test]
fn only_the_plugins_asked_for_are_rendered() {
    assert!(sidecars_for(&node(), &[]).is_empty());
    assert_eq!(sidecars_for(&node(), &[PluginId::DBFTPlugin]).len(), 1);
}
