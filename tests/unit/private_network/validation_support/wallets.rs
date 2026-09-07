//! Tests for launch-pack path safety and validation check collection helpers

use std::collections::BTreeMap;
use std::path::Path;

use crate::private_network::{
    collect_port, resolve_launch_pack_reference, safe_launch_pack_child,
    validate_signer_wallet_path,
};

#[test]
fn safe_launch_pack_child_accepts_relative_descendants() {
    let root = Path::new("/srv/launch-pack");
    let child = safe_launch_pack_child(root, "nodes/node-1/config.json")
        .expect("relative descendant should resolve");
    assert_eq!(child, root.join("nodes/node-1/config.json"));
}

#[test]
fn safe_launch_pack_child_rejects_parent_traversal() {
    let root = Path::new("/srv/launch-pack");
    assert!(safe_launch_pack_child(root, "../secrets.json").is_none());
    assert!(safe_launch_pack_child(root, "nodes/../../escape").is_none());
}

#[test]
fn safe_launch_pack_child_rejects_absolute_and_empty_values() {
    let root = Path::new("/srv/launch-pack");
    assert!(safe_launch_pack_child(root, "/etc/passwd").is_none());
    assert!(safe_launch_pack_child(root, "   ").is_none());
}

#[test]
fn resolve_launch_pack_reference_joins_relative_but_keeps_absolute() {
    let root = Path::new("/srv/launch-pack");
    assert_eq!(
        resolve_launch_pack_reference(root, "artifacts/genesis.json"),
        root.join("artifacts/genesis.json")
    );
    let absolute = resolve_launch_pack_reference(root, "/opt/data/genesis.json");
    assert_eq!(absolute, Path::new("/opt/data/genesis.json"));
}

#[test]
fn collect_port_groups_labels_by_port() {
    let mut ports: BTreeMap<u16, Vec<String>> = BTreeMap::new();
    collect_port(&mut ports, 20333, "node-1/p2p".to_string());
    collect_port(&mut ports, 20333, "node-2/p2p".to_string());
    collect_port(&mut ports, 20334, "node-1/rpc".to_string());

    assert_eq!(ports[&20333], vec!["node-1/p2p", "node-2/p2p"]);
    assert_eq!(ports[&20334], vec!["node-1/rpc"]);
}

#[test]
fn validate_signer_wallet_path_accepts_local_paths() {
    let path = validate_signer_wallet_path("wallets/committee-1.json")
        .expect("plain relative path should be accepted");
    assert_eq!(path, Path::new("wallets/committee-1.json"));
}

#[test]
fn validate_signer_wallet_path_rejects_empty_and_url_paths() {
    assert!(validate_signer_wallet_path("   ").is_err());
    assert!(validate_signer_wallet_path("https://example.com/wallet.json").is_err());
}
