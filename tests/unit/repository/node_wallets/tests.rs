use crate::{
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
};
use std::path::PathBuf;

fn repo_with_node() -> (tempfile::TempDir, Repository, String) {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("n.db")).unwrap();
    let id = repository
        .create_node(NewNode {
            name: "consensus-1".to_string(),
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

#[test]
fn a_node_starts_signing_with_nothing() {
    let (_dir, repository, id) = repo_with_node();
    assert_eq!(repository.load_node_wallet(&id).unwrap(), None);
}

#[test]
fn a_wallet_assignment_round_trips_and_is_replaceable() {
    let (_dir, repository, id) = repo_with_node();
    repository.set_node_wallet(&id, Some("wallet-a")).unwrap();
    assert_eq!(
        repository.load_node_wallet(&id).unwrap().as_deref(),
        Some("wallet-a")
    );
    repository.set_node_wallet(&id, Some("wallet-b")).unwrap();
    assert_eq!(
        repository.load_node_wallet(&id).unwrap().as_deref(),
        Some("wallet-b")
    );
    repository.set_node_wallet(&id, None).unwrap();
    assert_eq!(repository.load_node_wallet(&id).unwrap(), None);
}

/// Only the profile reference is stored. A password in the workspace database
/// would contradict the boundary this app enforces on imported wallets, which
/// fail validation if they carry a plaintext secret.
#[test]
fn the_table_holds_a_reference_and_nothing_else() {
    let (dir, repository, id) = repo_with_node();
    repository.set_node_wallet(&id, Some("wallet-a")).unwrap();
    let connection = rusqlite::Connection::open(dir.path().join("n.db")).unwrap();
    let columns: Vec<String> = connection
        .prepare("SELECT name FROM pragma_table_info('node_wallets')")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(columns, ["node_id", "wallet_profile_id"]);
}

fn second_node(repository: &Repository) -> String {
    repository
        .create_node(NewNode {
            name: "rpc-1".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Private,
            binary_path: PathBuf::from("/opt/neo/neo-go"),
            args: Vec::new(),
            runtime_version: "0.122".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 30342,
            p2p_port: 30343,
            ws_port: None,
        })
        .unwrap()
        .id
}

#[test]
fn assignments_are_scoped_to_their_node() {
    let (_dir, repository, first) = repo_with_node();
    let second = second_node(&repository);
    repository
        .set_node_wallet(&first, Some("wallet-a"))
        .unwrap();
    assert_eq!(repository.load_node_wallet(&second).unwrap(), None);
}

/// A wallet profile names a NEP-6 wallet. Two nodes rendering the same profile
/// into their configs are two nodes able to sign with one key — the state the
/// signer-key lane refuses, and which this lane used to allow silently.
#[test]
fn a_wallet_profile_signs_for_one_node_only() {
    let (_dir, repository, first) = repo_with_node();
    let second = second_node(&repository);
    repository
        .set_node_wallet(&first, Some("wallet-a"))
        .unwrap();

    let err = repository
        .set_node_wallet(&second, Some("wallet-a"))
        .expect_err("one wallet profile must not sign for two nodes");
    assert!(
        err.to_string().contains("already leased"),
        "the refusal should state the exclusivity rule: {err}"
    );
    assert_eq!(repository.load_node_wallet(&second).unwrap(), None);

    // Re-asserting the same lease on the node that holds it is not a conflict.
    repository
        .set_node_wallet(&first, Some("wallet-a"))
        .unwrap();

    // Releasing it makes it available again.
    repository.set_node_wallet(&first, None).unwrap();
    repository
        .set_node_wallet(&second, Some("wallet-a"))
        .unwrap();
    assert_eq!(
        repository.load_node_wallet(&second).unwrap().as_deref(),
        Some("wallet-a")
    );
}
