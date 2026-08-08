//! The neo-cli primary config carries only keys neo-cli reads.
//!
//! Two of the keys this generator used to emit were invented: a top-level
//! `Plugins` array that no release has ever read, and an
//! `ApplicationConfiguration.RPC.Port` for a section that does not exist — so
//! the port an operator chose never reached the node. Shapes are verified
//! against `src/Neo.CLI/config.json` in neo-project/neo-node.

use crate::*;

fn neo_cli_config() -> serde_json::Value {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    repo.set_plugin_enabled(&node_id, PluginId::RpcServer, true)
        .unwrap();
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let plugins = repo.list_plugin_states(&node.id).unwrap();
    ConfigGenerator::neo_cli(&node, &plugins).unwrap()
}

#[test]
fn the_plugins_key_is_a_download_source_not_an_enable_list() {
    let config = neo_cli_config();
    assert!(
        config["Plugins"].is_null(),
        "a top-level Plugins array is a NeoNexus manifest, not neo-cli config",
    );
    let url = config["ApplicationConfiguration"]["Plugins"]["DownloadUrl"].as_str();
    assert!(url.is_some_and(|url| url.starts_with("https://")));
}

/// neo-cli installs no log sink unless `Active` is true and `Path` is set, so
/// the default configuration produced a node that wrote nothing anywhere.
#[test]
fn logging_is_switched_on() {
    let config = neo_cli_config();
    let logger = &config["ApplicationConfiguration"]["Logger"];
    assert_eq!(logger["Active"], true);
    assert!(logger["Path"].as_str().is_some_and(|path| !path.is_empty()));
}

#[test]
fn there_is_no_rpc_section_in_the_primary_config() {
    let config = neo_cli_config();
    assert!(config["ApplicationConfiguration"]["RPC"].is_null());
}

/// `{0}` is substituted with the network magic, so one workspace can hold
/// several networks without their chain data colliding.
#[test]
fn the_storage_path_is_namespaced_by_network() {
    let config = neo_cli_config();
    let path = config["ApplicationConfiguration"]["Storage"]["Path"]
        .as_str()
        .expect("Storage.Path must be set");
    assert!(path.contains("{0}"), "{path} is not namespaced by network");
}

/// neo-cli opens one wallet for the whole node and every signing plugin uses
/// it, so unlike neo-go the wallet lives in the primary config.
#[test]
fn a_node_with_no_wallet_keeps_its_wallet_closed() {
    let config = neo_cli_config();
    let wallet = &config["ApplicationConfiguration"]["UnlockWallet"];
    assert_eq!(wallet["IsActive"], false);
    assert_eq!(wallet["Path"], "");
    assert_eq!(wallet["Password"], "");
}

/// The path is written so an operator can see which key the node signs with;
/// `IsActive` stays false because without a password neo-cli would prompt on a
/// console the supervisor does not have.
#[test]
fn an_assigned_wallet_is_named_but_stays_closed_without_a_password() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let context = GenerationContext::for_role(NodeRole::Oracle)
        .with_wallet(ServiceWallet::at("/opt/neo/wallets/oracle.json"));
    let config = ConfigGenerator::neo_cli_with_context(&node, &[], None, &context).unwrap();

    let wallet = &config["ApplicationConfiguration"]["UnlockWallet"];
    assert_eq!(wallet["Path"], "/opt/neo/wallets/oracle.json");
    assert_eq!(wallet["IsActive"], false);
}

#[test]
fn a_password_supplied_for_this_export_opens_the_wallet() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();
    let context = GenerationContext::for_role(NodeRole::Oracle)
        .with_wallet(ServiceWallet::at("/opt/neo/wallets/oracle.json").unlocked_with("hunter2"));
    let config = ConfigGenerator::neo_cli_with_context(&node, &[], None, &context).unwrap();

    let wallet = &config["ApplicationConfiguration"]["UnlockWallet"];
    assert_eq!(wallet["IsActive"], true);
    assert_eq!(wallet["Password"], "hunter2");
}
