use super::*;
use crate::{
    rpc_health::{RpcHealthReport, RpcIdentityKind, RpcNetworkObservation},
    types::{NewNode, NodeType},
};

fn fixture(network: Network) -> (tempfile::TempDir, Repository, NodeConfig) {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("test.db")).unwrap();
    let mut node = repository
        .create_node(NewNode {
            name: "chain node".into(),
            node_type: NodeType::NeoRs,
            network,
            binary_path: "unused".into(),
            args: vec![],
            runtime_version: "test".into(),
            storage_engine: NodeType::NeoRs.default_storage_engine(),
            rpc_port: 10332,
            p2p_port: 10333,
            ws_port: None,
        })
        .unwrap();
    node.status = NodeStatus::Running;
    node.pid = Some(4242);
    repository
        .update_node_status(&node.id, node.status, node.pid)
        .unwrap();
    (dir, repository, node)
}

fn report(node: &NodeConfig, height: u64) -> RpcHealthReport {
    let expected = expected_public_identity(node.node_type.family(), node.network);
    RpcHealthReport {
        endpoint: format!("http://127.0.0.1:{}", node.rpc_port),
        status: RpcHealthStatus::Healthy,
        version: Some("test".into()),
        block_count: Some(height),
        syncing: None,
        network: RpcNetworkObservation {
            identity_kind: Some(RpcIdentityKind::N3NetworkMagic),
            actual_identity: Some(expected.unwrap_or(123)),
            expected_identity: expected,
            peer_count: Some(2),
            peers_expected: expected.is_some(),
        },
        methods: vec![],
    }
}

fn window(repository: &Repository, node: &NodeConfig) {
    for index in 0..=30 {
        repository
            .record_rpc_health_at(node, &report(node, 42), 1000 + index * 30)
            .unwrap();
    }
}

fn progress_events(repository: &Repository) -> Vec<crate::events::RuntimeEvent> {
    repository
        .list_recent_events(100)
        .unwrap()
        .into_iter()
        .filter(|event| {
            matches!(
                event.kind,
                EventKind::ChainProgressStalled | EventKind::ChainProgressRecovered
            )
        })
        .collect()
}

#[test]
fn a_public_stall_and_resumed_height_each_emit_once_even_after_monitor_restart() {
    let (_dir, repository, node) = fixture(Network::Mainnet);
    window(&repository, &node);
    assert_eq!(check(&repository, 1900).unwrap(), 1);
    assert_eq!(check(&repository, 1900).unwrap(), 0);
    assert_eq!(check(&repository, 1930).unwrap(), 0);
    // No in-memory state is needed to preserve deduplication across restarts.
    repository
        .record_rpc_health_at(&node, &report(&node, 42), 1930)
        .unwrap();
    assert_eq!(check(&repository, 1930).unwrap(), 0);
    repository
        .record_rpc_health_at(&node, &report(&node, 43), 1960)
        .unwrap();
    assert_eq!(check(&repository, 1960).unwrap(), 1);
    assert_eq!(check(&repository, 1960).unwrap(), 0);
    let events = progress_events(&repository);
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, EventKind::ChainProgressRecovered);
    assert_eq!(events[1].kind, EventKind::ChainProgressStalled);
    repository.delete_node(&node.id).unwrap();
    assert!(repository
        .load_chain_progress_marker(&node.id)
        .unwrap()
        .is_none());
}

#[test]
fn stale_private_stopped_and_wrong_identity_observations_do_not_raise_stall_alarms() {
    for network in [Network::Mainnet, Network::Private] {
        let (_dir, repository, mut node) = fixture(network);
        window(&repository, &node);
        if network == Network::Private {
            assert_eq!(check(&repository, 1900).unwrap(), 0);
        }
        assert_eq!(check(&repository, 2000).unwrap(), 0);
        assert_eq!(check(&repository, 1899).unwrap(), 0);
        let mut wrong = report(&node, 42);
        wrong.network.actual_identity = Some(999);
        wrong.status = RpcHealthStatus::Degraded;
        repository
            .record_rpc_health_at(&node, &wrong, 1930)
            .unwrap();
        assert_eq!(check(&repository, 1930).unwrap(), 0);
        node.status = NodeStatus::Stopped;
        repository
            .update_node_status(&node.id, node.status, None)
            .unwrap();
        assert_eq!(check(&repository, 1930).unwrap(), 0);
        assert!(progress_events(&repository).is_empty());
    }
}

#[test]
fn failed_or_gapped_observations_break_the_continuous_window() {
    for gap in [false, true] {
        let (_dir, repository, node) = fixture(Network::Mainnet);
        for index in 0..=30 {
            if gap && (10..=20).contains(&index) {
                continue;
            }
            let mut observation = report(&node, 42);
            if !gap && index == 15 {
                observation.status = RpcHealthStatus::Unreachable;
            }
            repository
                .record_rpc_health_at(&node, &observation, 1000 + index * 30)
                .unwrap();
        }
        assert_eq!(check(&repository, 1900).unwrap(), 0);
    }
}

#[test]
fn repeated_timestamps_or_observations_before_a_restart_do_not_prove_a_stall() {
    let (_dir, repository, mut node) = fixture(Network::Mainnet);
    for _ in 0..100 {
        repository
            .record_rpc_health_at(&node, &report(&node, 42), 1000)
            .unwrap();
    }
    assert_eq!(check(&repository, 1000).unwrap(), 0);
    window(&repository, &node);
    node.pid = Some(7777);
    repository
        .update_node_status(&node.id, node.status, node.pid)
        .unwrap();
    repository
        .record_rpc_health_at(&node, &report(&node, 42), 1930)
        .unwrap();
    assert_eq!(check(&repository, 1930).unwrap(), 0);
    assert!(progress_events(&repository).is_empty());
}

#[test]
fn event_failure_rolls_back_the_marker_and_can_be_retried() {
    let (dir, repository, node) = fixture(Network::Mainnet);
    window(&repository, &node);
    let connection = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_chain_event BEFORE INSERT ON runtime_events
        WHEN NEW.kind='chain-progress-stalled' BEGIN SELECT RAISE(ABORT,'test journal unavailable'); END;").unwrap();
    assert!(check(&repository, 1900).is_err());
    assert!(repository
        .load_chain_progress_marker(&node.id)
        .unwrap()
        .is_none());
    connection
        .execute_batch("DROP TRIGGER reject_chain_event;")
        .unwrap();
    assert_eq!(check(&repository, 1900).unwrap(), 1);
}

#[test]
fn a_new_observation_node_edit_or_stop_invalidates_a_prepared_transition() {
    let (dir, repository, node) = fixture(Network::Mainnet);
    window(&repository, &node);
    let latest = repository.latest_rpc_health(&node.id).unwrap().unwrap();
    let marker = ProgressMarker {
        observed_pid: 4242,
        identity: 860_833_102,
        last_observation_id: latest.id,
        last_checked_at_unix: latest.checked_at_unix,
        stalled_block_count: Some(42),
    };
    repository
        .record_rpc_health_at(&node, &report(&node, 43), 1930)
        .unwrap();
    assert!(!repository
        .commit_chain_progress(&node, None, &marker, None, 1930)
        .unwrap());
    let latest = repository.latest_rpc_health(&node.id).unwrap().unwrap();
    let marker = ProgressMarker {
        last_observation_id: latest.id,
        ..marker
    };
    let connection = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    connection
        .execute(
            "UPDATE nodes SET rpc_port=rpc_port+1 WHERE id=?1",
            [&node.id],
        )
        .unwrap();
    assert!(!repository
        .commit_chain_progress(&node, None, &marker, None, 1930)
        .unwrap());
    connection
        .execute(
            "UPDATE nodes SET rpc_port=rpc_port-1 WHERE id=?1",
            [&node.id],
        )
        .unwrap();
    repository
        .update_node_status(&node.id, NodeStatus::Stopped, None)
        .unwrap();
    assert!(!repository
        .commit_chain_progress(&node, None, &marker, None, 1930)
        .unwrap());
    assert!(repository
        .load_chain_progress_marker(&node.id)
        .unwrap()
        .is_none());
}

#[test]
fn retained_history_covers_a_stall_at_the_minimum_probe_interval() {
    let (_dir, repository, node) = fixture(Network::Mainnet);
    repository
        .save_rpc_health_monitor_policy(crate::rpc_health::RpcHealthMonitorPolicy {
            enabled: true,
            interval_seconds: 10,
        })
        .unwrap();
    for index in 0..=100 {
        repository
            .record_rpc_health_at(&node, &report(&node, 42), 1000 + index * 10)
            .unwrap();
        repository
            .prune_rpc_health_keep_recent_per_node(OBSERVATION_HISTORY_LIMIT)
            .unwrap();
    }
    assert_eq!(
        repository.list_rpc_health(&node.id, 100).unwrap().len(),
        100
    );
    assert_eq!(check(&repository, 2000).unwrap(), 1);
}

#[test]
fn a_matching_number_in_the_wrong_identity_namespace_is_not_comparable() {
    let (_dir, repository, node) = fixture(Network::Mainnet);
    for index in 0..=30 {
        let mut observation = report(&node, 42);
        observation.network.identity_kind = Some(RpcIdentityKind::EvmChainId);
        repository
            .record_rpc_health_at(&node, &observation, 1000 + index * 30)
            .unwrap();
    }
    assert_eq!(check(&repository, 1900).unwrap(), 0);
}
