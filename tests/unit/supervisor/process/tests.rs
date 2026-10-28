//! The dedup contract of the log collector's sync progress.
//!
//! The dedup state may only move once the event batch that carries the
//! progress has actually reached the database. Marking it earlier turns one
//! failed write into a permanently suppressed sample: the retrying round
//! compares against a number the operator never saw and finds no news.

use std::sync::Arc;

use super::ProcessSupervisor;
use crate::{
    logs::observations::LogObservations,
    types::{Network, NodeStatus, NodeType, StorageEngine},
};

fn queue_node() -> crate::types::NodeConfig {
    crate::types::NodeConfig {
        id: "node-queue".to_string(),
        name: "queue node".to_string(),
        node_type: NodeType::NeoCli,
        network: Network::Testnet,
        binary_path: std::env::current_exe().expect("test executable path"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 20332,
        p2p_port: 20333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

#[test]
fn a_batch_that_never_committed_leaves_the_progress_unpublished() {
    let observations = Arc::new(LogObservations::default());
    let node = queue_node();

    let mut first_batch = Vec::new();
    let first = ProcessSupervisor::queue_sync_if_changed(
        &observations,
        &node,
        &mut first_batch,
        37,
        100,
        37.0,
        3,
    );
    assert!(first.is_some(), "a first sample always queues");
    assert_eq!(first_batch.len(), 1);

    // The batch above was never handed to the database, so the retrying round
    // sees the same file region again. It must queue the progress again —
    // the operator has not been shown anything yet.
    let mut retry_batch = Vec::new();
    let retried = ProcessSupervisor::queue_sync_if_changed(
        &observations,
        &node,
        &mut retry_batch,
        37,
        100,
        37.0,
        3,
    );
    assert!(retried.is_some(), "an uncommitted sample must re-queue");
    assert_eq!(retry_batch.len(), 1);
}

#[test]
fn a_committed_progress_stops_queueing_while_it_is_unchanged() {
    let observations = Arc::new(LogObservations::default());
    let node = queue_node();

    let mut batch = Vec::new();
    let (progress, epoch) = ProcessSupervisor::queue_sync_if_changed(
        &observations,
        &node,
        &mut batch,
        37,
        100,
        37.0,
        3,
    )
    .expect("a first sample always queues");

    // The commit path: the batch reached the database, so the progress counts
    // as shown and an unchanged sample is not news.
    observations.publish(&node, epoch, Some(Arc::new(progress)));

    let mut retry_batch = Vec::new();
    let deduplicated = ProcessSupervisor::queue_sync_if_changed(
        &observations,
        &node,
        &mut retry_batch,
        37,
        100,
        37.0,
        3,
    );
    assert!(
        deduplicated.is_none(),
        "an unchanged committed sample is not news"
    );
    assert!(retry_batch.is_empty());
}
