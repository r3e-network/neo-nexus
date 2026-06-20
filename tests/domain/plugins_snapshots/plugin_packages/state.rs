use super::*;

#[test]
fn plugin_state_can_be_enabled_and_listed_per_node() {
    let repo = create_repo();
    let node_id = create_node(&repo, "neo-cli", NodeType::NeoCli);

    repo.set_plugin_enabled(&node_id, PluginId::RpcServer, true)
        .unwrap();
    repo.set_plugin_enabled(&node_id, PluginId::ApplicationLogs, false)
        .unwrap();

    let states = repo.list_plugin_states(&node_id).unwrap();
    assert_eq!(states.len(), 2);
    assert_eq!(states[0].plugin_id, PluginId::ApplicationLogs);
    assert!(!states[0].enabled);
    assert_eq!(states[1].plugin_id, PluginId::RpcServer);
    assert!(states[1].enabled);
}
