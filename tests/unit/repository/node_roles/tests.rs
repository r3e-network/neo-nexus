use crate::{
    catalog::PluginId,
    repository::Repository,
    roles::{NodeRole, RolePlanner},
    types::{Network, NewNode, NodeStatus, NodeType, StorageEngine},
};
use std::path::PathBuf;

fn repo_with_node() -> (tempfile::TempDir, Repository, String) {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("n.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "oracle-1".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Testnet,
            binary_path: PathBuf::from("/opt/neo/neo-cli"),
            args: Vec::new(),
            runtime_version: "3.10.1".to_string(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();
    let id = node.id.clone();
    (dir, repository, id)
}

#[test]
fn a_node_starts_with_no_assigned_duty() {
    let (_dir, repository, id) = repo_with_node();
    assert_eq!(repository.load_node_role(&id).unwrap(), None);
}

#[test]
fn every_role_round_trips() {
    let (_dir, repository, id) = repo_with_node();
    for role in NodeRole::ALL {
        repository.set_node_role(&id, Some(role)).unwrap();
        assert_eq!(repository.load_node_role(&id).unwrap(), Some(role));
    }
}

#[test]
fn assigning_a_role_replaces_the_previous_one() {
    let (_dir, repository, id) = repo_with_node();
    repository
        .set_node_role(&id, Some(NodeRole::Oracle))
        .unwrap();
    repository
        .set_node_role(&id, Some(NodeRole::Consensus))
        .unwrap();
    assert_eq!(
        repository.load_node_role(&id).unwrap(),
        Some(NodeRole::Consensus)
    );
}

#[test]
fn a_duty_can_be_cleared() {
    let (_dir, repository, id) = repo_with_node();
    repository
        .set_node_role(&id, Some(NodeRole::Oracle))
        .unwrap();
    repository.set_node_role(&id, None).unwrap();
    assert_eq!(repository.load_node_role(&id).unwrap(), None);
}

/// A role retired in a later version must not make an existing node
/// unloadable; it reads as unassigned instead.
#[test]
fn an_unrecognised_stored_role_reads_as_unassigned() {
    let (dir, repository, id) = repo_with_node();
    let connection = rusqlite::Connection::open(dir.path().join("n.db")).unwrap();
    connection
        .execute(
            "INSERT INTO node_roles (node_id, role) VALUES (?1, 'retired-duty')",
            rusqlite::params![id],
        )
        .unwrap();
    assert_eq!(repository.load_node_role(&id).unwrap(), None);
}

/// Roles are per node, so assigning one must not touch its neighbours.
#[test]
fn roles_are_scoped_to_their_node() {
    let (_dir, repository, first) = repo_with_node();
    let second = repository
        .create_node(NewNode {
            name: "rpc-1".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/opt/neo/neo-go"),
            args: Vec::new(),
            runtime_version: "0.122".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20342,
            p2p_port: 20343,
            ws_port: None,
        })
        .unwrap()
        .id;
    repository
        .set_node_role(&first, Some(NodeRole::Oracle))
        .unwrap();
    assert_eq!(repository.load_node_role(&second).unwrap(), None);
}

#[test]
fn applying_a_role_persists_role_and_plugins_together() {
    let (_dir, repository, id) = repo_with_node();
    let node = repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|node| node.id == id)
        .unwrap();
    let plan = RolePlanner::plan(&node, NodeRole::Oracle);
    repository.apply_node_role_plan(&id, &plan).unwrap();

    assert_eq!(
        repository.load_node_role(&id).unwrap(),
        Some(NodeRole::Oracle)
    );
    let plugins = repository.list_plugin_states(&id).unwrap();
    for change in plan.plugin_changes {
        assert_eq!(
            plugins
                .iter()
                .find(|plugin| plugin.plugin_id == change.plugin_id)
                .map(|plugin| plugin.enabled),
            Some(change.enabled)
        );
    }
    assert!(plugins
        .iter()
        .any(|plugin| plugin.plugin_id == PluginId::OracleService));
}

#[test]
fn active_node_cannot_change_duty() {
    let (_dir, repository, id) = repo_with_node();
    repository
        .update_node_status(&id, NodeStatus::Running, Some(42))
        .unwrap();
    let error = repository
        .set_node_role(&id, Some(NodeRole::Oracle))
        .expect_err("active node identity must remain stable");
    assert!(error.to_string().contains("stop node"));
}
