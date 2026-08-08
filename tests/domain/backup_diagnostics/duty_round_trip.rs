//! A backup has to bring back the duties it was taken with.
//!
//! Node duties and wallet bindings live in their own tables, and the backup
//! schema did not know about them. So an operator who assigned duties, exported
//! a workspace and restored it after a disk loss got their fleet back with every
//! duty erased — and the import summary reported the nodes and plugin states as
//! fully restored, because no counter existed that could come back zero.
//!
//! The loss was not cosmetic: the generators branch on the duty to decide which
//! service sections a config carries, so the restored nodes would regenerate as
//! plain relays and boot with consensus, state and indexer services off.

use super::*;

/// `node_wallets` stores a profile id and has no foreign key to the profile
/// table, so the binding can be asserted without building a real NEP-6 wallet.
const WALLET_PROFILE_ID: &str = "wallet-profile-1";

/// The shared `create_node` helper puts every node on 10332, and the backup
/// exporter rejects a port collision, so this fleet needs its own ports.
fn node_on(repo: &Repository, name: &str, node_type: NodeType, offset: u16) -> String {
    repo.create_node(NewNode {
        name: name.to_string(),
        node_type,
        network: Network::Testnet,
        binary_path: PathBuf::from("/usr/local/bin/node"),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: node_type.default_storage_engine(),
        rpc_port: 10332 + offset * 10,
        p2p_port: 10333 + offset * 10,
        ws_port: Some(10334 + offset * 10),
    })
    .unwrap()
    .id
}

/// Export with duties, import into an empty workspace, and read the duties back
/// out of the target — not out of the backup struct, which would only prove
/// serialisation.
#[test]
fn duties_and_wallet_bindings_survive_a_backup_round_trip() {
    let temp = tempfile::tempdir().unwrap();
    let source = Repository::open(temp.path().join("source.db")).unwrap();

    let assignments = [
        ("validator-01", NodeType::NeoGo, NodeRole::Consensus),
        ("oracle-01", NodeType::NeoCli, NodeRole::Oracle),
        ("rpc-01", NodeType::NeoRs, NodeRole::RpcApi),
    ];
    let mut ids = Vec::new();
    for (offset, (name, node_type, role)) in assignments.into_iter().enumerate() {
        let id = node_on(&source, name, node_type, offset as u16);
        source.set_node_role(&id, Some(role)).unwrap();
        source
            .set_node_wallet(&id, Some(WALLET_PROFILE_ID))
            .unwrap();
        ids.push((id, role));
    }
    // One node with no duty, so the restore cannot pass by assigning a duty to
    // everything it sees.
    let plain = node_on(&source, "relay-01", NodeType::NeoGo, 9);

    let backup = WorkspaceBackupExporter::snapshot(&source, "3.2.0", 1_770_000_000).unwrap();
    let target = Repository::open(temp.path().join("target.db")).unwrap();
    let imported = WorkspaceBackupImporter::import(&target, &backup).unwrap();

    assert_eq!(imported.role_count, 3, "three duties were exported");
    assert_eq!(imported.wallet_binding_count, 3);

    for (id, role) in ids {
        assert_eq!(
            target.load_node_role(&id).unwrap(),
            Some(role),
            "node {id} lost its duty across the round trip",
        );
        assert_eq!(
            target.load_node_wallet(&id).unwrap().as_deref(),
            Some(WALLET_PROFILE_ID),
            "node {id} lost its wallet binding",
        );
    }
    assert_eq!(
        target.load_node_role(&plain).unwrap(),
        None,
        "a node with no duty was given one",
    );
}

/// Re-importing must not accumulate: a second restore of the same backup leaves
/// the same duties, not duplicates or a cleared table.
#[test]
fn a_second_import_leaves_the_same_duties() {
    let temp = tempfile::tempdir().unwrap();
    let source = Repository::open(temp.path().join("source.db")).unwrap();
    let id = create_node(&source, "validator-01", NodeType::NeoGo);
    source
        .set_node_role(&id, Some(NodeRole::Consensus))
        .unwrap();

    let backup = WorkspaceBackupExporter::snapshot(&source, "3.2.0", 1_770_000_000).unwrap();
    let target = Repository::open(temp.path().join("target.db")).unwrap();
    WorkspaceBackupImporter::import(&target, &backup).unwrap();
    let again = WorkspaceBackupImporter::import(&target, &backup).unwrap();

    assert_eq!(again.role_count, 1);
    assert_eq!(
        target.load_node_role(&id).unwrap(),
        Some(NodeRole::Consensus)
    );
}

/// A backup taken before duties existed carries neither field. It must still
/// restore, with no duty rather than an error.
#[test]
fn a_backup_without_duty_fields_still_restores() {
    let temp = tempfile::tempdir().unwrap();
    let source = Repository::open(temp.path().join("source.db")).unwrap();
    let id = create_node(&source, "legacy-01", NodeType::NeoGo);
    source
        .set_node_role(&id, Some(NodeRole::Consensus))
        .unwrap();

    let backup = WorkspaceBackupExporter::snapshot(&source, "3.2.0", 1_770_000_000).unwrap();
    let mut json = serde_json::to_value(&backup).unwrap();
    for node in json["nodes"].as_array_mut().unwrap() {
        node.as_object_mut().unwrap().remove("role");
        node.as_object_mut().unwrap().remove("wallet_profile_id");
    }
    let legacy: WorkspaceBackup = serde_json::from_value(json).unwrap();

    let target = Repository::open(temp.path().join("target.db")).unwrap();
    let imported = WorkspaceBackupImporter::import(&target, &legacy).unwrap();
    assert_eq!(imported.role_count, 0);
    assert_eq!(target.load_node_role(&id).unwrap(), None);
}
