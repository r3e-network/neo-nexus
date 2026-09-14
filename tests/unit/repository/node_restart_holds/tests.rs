use std::path::PathBuf;

use crate::{
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
};

fn workspace() -> (tempfile::TempDir, Repository, String) {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("n.db")).unwrap();
    let id = repository
        .create_node(NewNode {
            name: "rpc-1".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Private,
            binary_path: PathBuf::from("/opt/neo/neo-go"),
            args: Vec::new(),
            runtime_version: "0.122".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 30332,
            p2p_port: 30333,
            ws_port: None,
        })
        .unwrap()
        .id;
    (dir, repository, id)
}

/// An absent row means "follow the workspace policy", which is what every node
/// does until an operator says otherwise about one of them. A default of "held"
/// would leave a fresh fleet unsupervised.
#[test]
fn a_node_follows_the_workspace_policy_until_it_is_held() {
    let (_dir, repository, id) = workspace();
    assert_eq!(repository.node_restart_hold(&id).unwrap(), None);
    assert!(repository.held_node_ids().unwrap().is_empty());
}

/// The hold carries when it was placed and why, because an operator returning
/// to a node a week later needs to know whether the hold is still deliberate.
#[test]
fn a_hold_records_when_it_was_placed_and_why() {
    let (_dir, repository, id) = workspace();
    repository
        .hold_node_restarts(&id, "editing its config", 1_770_000_000)
        .unwrap();

    let held = repository.node_restart_hold(&id).unwrap().unwrap();
    assert_eq!(held.0, 1_770_000_000);
    assert_eq!(held.1, "editing its config");
    assert!(repository.held_node_ids().unwrap().contains(&id));
}

/// Holding twice replaces the reason rather than failing or accumulating rows.
#[test]
fn holding_again_replaces_the_reason() {
    let (_dir, repository, id) = workspace();
    repository
        .hold_node_restarts(&id, "first", 1_770_000_000)
        .unwrap();
    repository
        .hold_node_restarts(&id, "second", 1_770_000_600)
        .unwrap();

    let held = repository.node_restart_hold(&id).unwrap().unwrap();
    assert_eq!(held.1, "second");
    assert_eq!(repository.held_node_ids().unwrap().len(), 1);
}

/// Releasing returns the node to the workspace policy, and releasing one that
/// was never held is not an error — an operator pressing the button twice has
/// not done anything wrong.
#[test]
fn releasing_returns_the_node_to_the_workspace_policy() {
    let (_dir, repository, id) = workspace();
    repository
        .hold_node_restarts(&id, "", 1_770_000_000)
        .unwrap();
    repository.release_node_restarts(&id).unwrap();
    assert_eq!(repository.node_restart_hold(&id).unwrap(), None);

    repository.release_node_restarts(&id).unwrap();
    assert_eq!(repository.node_restart_hold(&id).unwrap(), None);
}

/// A hold belongs to one node. The whole point is that stopping the watchdog
/// for the node you are editing leaves the rest of the fleet supervised.
#[test]
fn a_hold_applies_to_one_node_and_leaves_the_fleet_alone() {
    let (_dir, repository, held) = workspace();
    let other = repository
        .create_node(NewNode {
            name: "rpc-2".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Private,
            binary_path: PathBuf::from("/opt/neo/neo-go"),
            args: Vec::new(),
            runtime_version: "0.122".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 30432,
            p2p_port: 30433,
            ws_port: None,
        })
        .unwrap()
        .id;

    repository
        .hold_node_restarts(&held, "", 1_770_000_000)
        .unwrap();
    assert!(repository.node_restart_hold(&other).unwrap().is_none());
    assert_eq!(
        repository
            .held_node_ids()
            .unwrap()
            .into_iter()
            .collect::<Vec<_>>(),
        vec![held]
    );
}

/// Node ids come from URLs, so the same validation every other node-scoped
/// table applies here too.
#[test]
fn a_malformed_node_id_is_refused() {
    let (_dir, repository, _id) = workspace();
    assert!(repository.hold_node_restarts("' OR 1=1 --", "", 0).is_err());
    assert!(repository.node_restart_hold("' OR 1=1 --").is_err());
}
