use super::super::*;

#[test]
fn diagnostics_reward_neo_cli_plugin_alignment() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let without_plugins =
        neo_nexus::diagnostics::evaluate_node(&node, std::slice::from_ref(&node), &[]);

    repo.set_plugin_enabled(&node.id, PluginId::RpcServer, true)
        .unwrap();
    repo.set_plugin_enabled(&node.id, PluginId::RocksDbStore, true)
        .unwrap();
    let plugins = repo.list_plugin_states(&node.id).unwrap();
    let with_plugins =
        neo_nexus::diagnostics::evaluate_node(&node, std::slice::from_ref(&node), &plugins);

    assert!(with_plugins.score > without_plugins.score);
    assert!(without_plugins
        .checks
        .iter()
        .any(|check| { check.severity == CheckSeverity::Warning && check.title == "RPC" }));
}

#[test]
fn diagnostics_accept_neo_rs_builtin_rpc() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-rs", NodeType::NeoRs);
    let node = repo
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == node_id)
        .unwrap();

    let diagnostics =
        neo_nexus::diagnostics::evaluate_node(&node, std::slice::from_ref(&node), &[]);

    assert!(diagnostics.checks.iter().any(|check| {
        check.severity == CheckSeverity::Pass
            && check.title == "RPC"
            && check.detail.contains("neo-rs")
    }));
}
