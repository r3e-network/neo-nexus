//! Tests for log cursor management.

use std::path::PathBuf;

use crate::{
    logs::cursor::{CursorStore, FileIdentity, LogCursor},
    types::{Network, NodeConfig, NodeStatus, NodeType, StorageEngine},
};

fn create_test_node(id: &str, name: &str) -> NodeConfig {
    NodeConfig {
        id: id.to_string(),
        name: name.to_string(),
        node_type: NodeType::NeoCli,
        network: Network::Testnet,
        binary_path: PathBuf::from("neo-cli"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

#[test]
fn test_cursor_new() {
    let file_id = FileIdentity::new(1, 12345, 1024);
    let cursor = LogCursor::new("node-1", file_id);

    assert_eq!(cursor.node_id, "node-1");
    assert_eq!(cursor.file_identity.generation, 1);
    assert_eq!(cursor.byte_offset, 0);
}

#[test]
fn test_cursor_advance() {
    let mut cursor = LogCursor::new("node-1", FileIdentity::default());
    cursor.advance(1024);

    assert_eq!(cursor.byte_offset, 1024);
}

#[test]
fn test_cursor_reset_on_rotation() {
    let mut cursor = LogCursor::new("node-1", FileIdentity::new(1, 100, 2048));

    let new_identity = FileIdentity::new(2, 100, 0); // Generation incremented
    cursor.reset(new_identity);

    assert_eq!(cursor.file_identity.generation, 2);
    assert_eq!(cursor.byte_offset, 0);
}

#[test]
fn test_cursor_is_stale() {
    let cursor = LogCursor::new("node-1", FileIdentity::default());

    assert!(!cursor.is_stale(60)); // Should not be stale within 60 seconds

    // Simulate old timestamp
    let stale_cursor = LogCursor {
        updated_at: 0, // Very old
        ..cursor.clone()
    };

    assert!(stale_cursor.is_stale(60));
}

#[test]
fn test_cursor_store_get_or_create() {
    let mut store = CursorStore::new();
    let node = create_test_node("node-1", "Test Node");

    let cursor = store.get_or_create(&node);
    assert_eq!(cursor.node_id, "node-1");
}

#[test]
fn test_cursor_store_budget_enforcement() {
    let store = CursorStore::new();

    // Can accept within budget
    assert!(store.can_accept(1024, 10));
    assert!(store.can_accept(4 * 1024 * 1024, 8192)); // Max budget

    // Cannot exceed budget
    assert!(!store.can_accept(4 * 1024 * 1024 + 1, 8192));
    assert!(!store.can_accept(4 * 1024 * 1024, 8193));
}

#[test]
fn test_cursor_store_acquire_release_budget() {
    let mut store = CursorStore::new();

    let (allowed_bytes, allowed_lines) = store.acquire_budget(1024, 10);
    assert_eq!(allowed_bytes, 1024);
    assert_eq!(allowed_lines, 10);
    assert_eq!(store.pending_bytes, 1024);
    assert_eq!(store.pending_lines, 10);

    store.release_budget(1024, 10);
    assert_eq!(store.pending_bytes, 0);
    assert_eq!(store.pending_lines, 0);
}

#[test]
fn test_cursor_store_round_robin() {
    let mut store = CursorStore::new();

    let node1 = create_test_node("node-1", "Node 1");
    let node2 = create_test_node("node-2", "Node 2");
    let node3 = create_test_node("node-3", "Node 3");

    store.get_or_create(&node1);
    store.get_or_create(&node2);
    store.get_or_create(&node3);

    // Queue should contain all nodes in insertion order
    let ids: Vec<_> = store.round_robin_queue.iter().collect();
    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0], "node-1");
    assert_eq!(ids[1], "node-2");
    assert_eq!(ids[2], "node-3");
}

#[test]
fn test_file_identity_rotation_detection() {
    let file_id = FileIdentity::new(1, 100, 2048);
    let rotated = file_id.rotate();

    assert_eq!(rotated.generation, 2);
    assert_eq!(rotated.device_id, 100);
    assert_eq!(rotated.file_size, 0);
}
