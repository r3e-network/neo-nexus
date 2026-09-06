use super::*;

#[test]
fn rejects_node_definition_changes_while_runtime_is_active() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "managed".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();

    repository
        .update_node_status(&node.id, NodeStatus::Running, Some(1234))
        .unwrap();
    let error = repository
        .update_node(
            &node.id,
            NewNode {
                name: "renamed".to_string(),
                node_type: NodeType::NeoGo,
                network: Network::Testnet,
                binary_path: "/opt/neo-go".into(),
                args: vec!["node".into()],
                runtime_version: "v0.110.1".to_string(),
                storage_engine: StorageEngine::LevelDb,
                rpc_port: 20332,
                p2p_port: 20333,
                ws_port: Some(20334),
            },
        )
        .expect_err("an active process must keep the definition it was launched with");

    assert!(error.to_string().contains("stop node"));
    let unchanged = repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|stored| stored.id == node.id)
        .unwrap();
    assert_eq!(unchanged.name, "managed");
    assert_eq!(unchanged.status, NodeStatus::Running);
    assert_eq!(unchanged.pid, Some(1234));
    assert_eq!(unchanged.rpc_port, 10332);
    assert_eq!(unchanged.runtime_version, "latest");
}

#[test]
fn refuses_to_delete_a_node_until_its_runtime_is_settled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "active".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 11332,
            p2p_port: 11333,
            ws_port: None,
        })
        .unwrap();
    repository
        .update_node_status(&node.id, NodeStatus::Running, Some(1234))
        .unwrap();

    let error = repository
        .delete_node(&node.id)
        .expect_err("deleting an active runtime would orphan its process");
    assert!(error.to_string().contains("confirm its process exited"));
    assert!(repository
        .list_nodes()
        .unwrap()
        .iter()
        .any(|stored| stored.id == node.id));
}

#[test]
fn refuses_to_restore_over_an_active_node() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "active restore target".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 15332,
            p2p_port: 15333,
            ws_port: None,
        })
        .unwrap();
    repository
        .update_node_status(&node.id, NodeStatus::Running, Some(1234))
        .unwrap();

    let mut restored = node.clone();
    restored.name = "backup replacement".to_string();
    restored.status = NodeStatus::Stopped;
    restored.pid = None;
    let error = repository
        .restore_node_with_plugins(&restored, &[])
        .expect_err("restore must not orphan an active process");
    assert!(error.to_string().contains("before restoring over it"));

    let current = repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|candidate| candidate.id == node.id)
        .unwrap();
    assert_eq!(current.name, node.name);
    assert_eq!(current.status, NodeStatus::Running);
    assert_eq!(current.pid, Some(1234));
}

#[test]
fn runtime_status_transition_is_compare_and_set_on_status_and_pid() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "cas".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 12332,
            p2p_port: 12333,
            ws_port: None,
        })
        .unwrap();
    repository
        .update_node_status(&node.id, NodeStatus::Running, Some(42))
        .unwrap();

    assert!(!repository
        .transition_node_status(
            &node.id,
            NodeStatus::Running,
            Some(41),
            NodeStatus::Stopped,
            None,
        )
        .unwrap());
    assert!(repository
        .transition_node_status(
            &node.id,
            NodeStatus::Running,
            Some(42),
            NodeStatus::Stopped,
            None,
        )
        .unwrap());
}

#[test]
fn a_stale_node_definition_cannot_claim_a_launch_after_an_edit() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let stale = repository
        .create_node(NewNode {
            name: "before edit".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 13332,
            p2p_port: 13333,
            ws_port: None,
        })
        .unwrap();
    repository
        .update_node(
            &stale.id,
            NewNode {
                name: "after edit".to_string(),
                node_type: NodeType::NeoCli,
                network: Network::Private,
                binary_path: "/opt/neo/neo-cli".into(),
                args: vec!["--trace".to_string()],
                runtime_version: "v3.8.2".to_string(),
                storage_engine: StorageEngine::LevelDb,
                rpc_port: 14332,
                p2p_port: 14333,
                ws_port: None,
            },
        )
        .unwrap();

    assert!(!repository.claim_node_launch(&stale).unwrap());
    let current = repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|stored| stored.id == stale.id)
        .unwrap();
    assert_eq!(current.name, "after edit");
    assert_eq!(current.status, NodeStatus::Stopped);
    assert_eq!(current.pid, None);
}

#[test]
fn deletes_node_and_plugin_state() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "managed".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();

    repository
        .set_plugin_enabled(&node.id, neo_nexus::catalog::PluginId::RpcServer, true)
        .unwrap();
    repository.delete_node(&node.id).unwrap();

    assert!(repository.list_nodes().unwrap().is_empty());
    assert!(repository.list_plugin_states(&node.id).unwrap().is_empty());
}

#[test]
fn refuses_plugin_configuration_changes_until_runtime_is_settled() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "active plugins".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Private,
            binary_path: "/usr/local/bin/neo-cli".into(),
            args: Vec::new(),
            runtime_version: "latest".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 16332,
            p2p_port: 16333,
            ws_port: None,
        })
        .unwrap();
    repository
        .update_node_status(&node.id, NodeStatus::Running, Some(1234))
        .unwrap();

    let error = repository
        .set_plugin_enabled(&node.id, neo_nexus::catalog::PluginId::RpcServer, true)
        .expect_err("an active node cannot adopt a config the process has not loaded");
    assert!(error
        .to_string()
        .contains("before changing its plugin configuration"));
    assert!(repository.list_plugin_states(&node.id).unwrap().is_empty());

    repository
        .update_node_status(&node.id, NodeStatus::Stopped, Some(1234))
        .unwrap();
    repository
        .set_plugin_enabled(&node.id, neo_nexus::catalog::PluginId::RpcServer, true)
        .expect_err("a recorded pid must be settled even after stop intent is durable");

    repository
        .update_node_status(&node.id, NodeStatus::Stopped, None)
        .unwrap();
    repository
        .set_plugin_enabled(&node.id, neo_nexus::catalog::PluginId::RpcServer, true)
        .unwrap();
    assert_eq!(repository.list_plugin_states(&node.id).unwrap().len(), 1);
}
