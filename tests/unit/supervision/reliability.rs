use super::*;
use crate::types::{Network, NewNode, NodeType, StorageEngine};

pub(super) fn node(
    state: &EngineState,
    name: &str,
    status: NodeStatus,
    pid: Option<u32>,
) -> NodeConfig {
    let rpc = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let p2p = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let node = state
        .repository
        .create_node(NewNode {
            name: name.into(),
            node_type: NodeType::NeoRs,
            network: Network::Testnet,
            binary_path: state.data_dir.join("missing-node-executable"),
            args: vec![],
            runtime_version: "test".into(),
            storage_engine: StorageEngine::RocksDb,
            rpc_port: rpc.local_addr().unwrap().port(),
            p2p_port: p2p.local_addr().unwrap().port(),
            ws_port: None,
        })
        .unwrap();
    state
        .repository
        .update_node_status(&node.id, status, pid)
        .unwrap();
    state
        .nodes()
        .into_iter()
        .find(|current| current.id == node.id)
        .unwrap()
}

pub(super) fn immediate_watchdog(state: &EngineState) -> Watchdog {
    let policy = RestartPolicy {
        enabled: true,
        max_restart_attempts: 2,
        base_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(1),
    };
    state.repository.save_watchdog_policy(policy).unwrap();
    Watchdog::new(policy)
}

pub(super) fn due_now(state: &EngineState, id: &str) {
    let mut recovery = state
        .repository
        .load_node_recoveries()
        .unwrap()
        .remove(id)
        .unwrap();
    if recovery.next_attempt_at_unix_ms.is_some() {
        recovery.next_attempt_at_unix_ms = Some(0);
        state
            .repository
            .save_workspace_section(
                &format!("watchdog.recovery.{id}"),
                &serde_json::to_string(&recovery).unwrap(),
            )
            .unwrap();
    }
}

#[test]
fn a_pending_restart_is_cancelled_by_an_explicit_stop() {
    let (_dir, state) = fixture();
    let node = node(&state, "cancelled", NodeStatus::Crashed, None);
    let mut engine = LoopState::bootstrap(&state);
    engine.watchdog = immediate_watchdog(&state);
    engine.schedule_restart(&state, &node, "test crash");
    due_now(&state, &node.id);
    stop_node(&state, &node).unwrap();
    engine.run_due_restarts(&state);
    assert_eq!(state.nodes()[0].status, NodeStatus::Stopped);
    assert!(!engine.watchdog.has_pending_restart());
    assert!(!state
        .repository
        .list_events_after(0, 30)
        .unwrap()
        .iter()
        .any(|event| event.kind == EventKind::NodeStartFailed));
}

#[test]
fn automatic_launch_failures_retry_to_the_limit_and_remain_error() {
    let (_dir, state) = fixture();
    let node = node(&state, "missing-binary", NodeStatus::Crashed, None);
    let mut engine = LoopState::bootstrap(&state);
    engine.watchdog = immediate_watchdog(&state);
    engine.schedule_restart(&state, &node, "test crash");
    due_now(&state, &node.id);
    engine.run_due_restarts(&state);
    assert!(
        engine.watchdog.has_pending_restart(),
        "a failed launch must not silently end the retry chain"
    );
    assert_eq!(state.nodes()[0].status, NodeStatus::Error);
    due_now(&state, &node.id);
    engine.run_due_restarts(&state);
    assert!(!engine.watchdog.has_pending_restart());
    assert_eq!(state.nodes()[0].status, NodeStatus::Error);
    assert!(matches!(
        engine.watchdog.status(&node.id, Instant::now()),
        crate::watchdog::WatchdogStatus::Exhausted { attempts: 2 }
    ));
}

#[test]
fn an_old_exit_cannot_overwrite_the_pid_of_a_new_start() {
    let (_dir, state) = fixture();
    let node = node(&state, "restarted", NodeStatus::Running, Some(4_000_001));
    let mut engine = LoopState::bootstrap(&state);
    for code in [Some(0), Some(1)] {
        engine.reconcile_node_exit(
            &state,
            &crate::supervisor::ProcessExit {
                process_id: node.id.clone(),
                node_id: node.id.clone(),
                pid: 4_000_000,
                exit_code: code,
            },
        );
        assert_eq!(state.nodes()[0], node);
    }
    assert!(!engine.watchdog.has_pending_restart());
}

#[test]
fn a_stale_stop_request_cannot_clear_a_replacement_pid() {
    let (_dir, state) = fixture();
    let old = node(
        &state,
        "replaced-before-stop",
        NodeStatus::Running,
        Some(4_000_000),
    );
    state
        .repository
        .update_node_status(&old.id, NodeStatus::Running, Some(4_000_001))
        .unwrap();
    assert!(stop_node(&state, &old).is_err());
    assert_eq!(state.nodes()[0].pid, Some(4_000_001));
    assert_eq!(state.nodes()[0].status, NodeStatus::Running);
}

#[test]
fn external_pid_reuse_blocks_restart_and_missing_process_is_crashed() {
    let (_dir, state) = fixture();
    let unrelated = node(
        &state,
        "reused",
        NodeStatus::Running,
        Some(std::process::id()),
    );
    let missing = node(&state, "missing", NodeStatus::Running, Some(4_000_000));
    let mut engine = LoopState::bootstrap(&state);
    engine.watch_external_processes(&state);
    let nodes = state.nodes();
    let unrelated = nodes.iter().find(|node| node.id == unrelated.id).unwrap();
    assert_eq!(unrelated.status, NodeStatus::Error);
    assert_eq!(unrelated.pid, Some(std::process::id()));
    assert!(crate::supervisor::process_is_live(std::process::id()));
    let missing = nodes.iter().find(|node| node.id == missing.id).unwrap();
    assert_eq!(missing.status, NodeStatus::Crashed);
    assert_eq!(missing.pid, None);
}

#[test]
fn a_stale_rpc_result_does_not_write_health_or_postpone_the_new_process_probe() {
    let (_dir, state) = fixture();
    let node = node(
        &state,
        "rpc-restarted",
        NodeStatus::Running,
        Some(4_000_000),
    );
    let mut engine = LoopState::bootstrap(&state);
    engine
        .rpc_last_probe
        .insert(node.id.clone(), Instant::now());
    state
        .repository
        .update_node_status(&node.id, NodeStatus::Running, Some(4_000_001))
        .unwrap();
    engine.record_rpc_health(
        &state,
        node.clone(),
        crate::rpc_health::RpcHealthReport {
            endpoint: "http://127.0.0.1:0".into(),
            status: crate::rpc_health::RpcHealthStatus::Unreachable,
            version: None,
            block_count: None,
            syncing: None,
            network: Default::default(),
            methods: vec![],
        },
        Instant::now(),
    );
    assert!(state
        .repository
        .latest_rpc_health(&node.id)
        .unwrap()
        .is_none());
    assert!(!engine.rpc_last_probe.contains_key(&node.id));
}

#[test]
fn wrong_network_alerts_even_when_rpc_was_already_degraded_and_deduplicates() {
    use crate::rpc_health::{
        RpcHealthReport, RpcHealthStatus, RpcIdentityKind, RpcNetworkObservation,
    };
    let (_dir, state) = fixture();
    let node = node(&state, "wrong-chain", NodeStatus::Running, Some(4_000_000));
    let mut engine = LoopState::bootstrap(&state);
    let mut report = RpcHealthReport {
        endpoint: "http://127.0.0.1:0".into(),
        status: RpcHealthStatus::Degraded,
        version: None,
        block_count: None,
        syncing: None,
        methods: vec![],
        network: RpcNetworkObservation {
            identity_kind: Some(RpcIdentityKind::N3NetworkMagic),
            actual_identity: Some(894_710_606),
            expected_identity: Some(894_710_606),
            peer_count: Some(0),
            peers_expected: true,
        },
    };
    state.repository.record_rpc_health(&node, &report).unwrap();
    report.network.actual_identity = Some(860_833_102);
    engine.record_rpc_health(&state, node.clone(), report.clone(), Instant::now());
    engine.record_rpc_health(&state, node.clone(), report.clone(), Instant::now());
    let events = state.repository.list_recent_events(20).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].severity, EventSeverity::Critical);
    assert!(events[0].message.contains("wrong network"));
    report.network.actual_identity = report.network.expected_identity;
    engine.record_rpc_health(&state, node, report, Instant::now());
    assert_eq!(state.repository.list_recent_events(20).unwrap().len(), 2);
}

#[test]
fn rpc_batches_prioritize_the_unprobed_tail_even_when_old_nodes_are_due_again() {
    let (_dir, state) = fixture();
    for index in 0..9 {
        node(
            &state,
            &format!("rpc-{index}"),
            NodeStatus::Running,
            Some(4_000_000 + index),
        );
    }
    state.repository.save_rpc_health_monitor_policy(crate::rpc_health::RpcHealthMonitorPolicy::enabled_default()).unwrap();
    let mut engine = LoopState::bootstrap(&state);
    engine.probe_rpc_health(&state);
    assert_eq!(engine.rpc_last_probe.len(), MAX_RPC_PROBES_PER_TICK);
    let tail = state
        .nodes()
        .into_iter()
        .find(|node| !engine.rpc_last_probe.contains_key(&node.id))
        .unwrap();
    for last in engine.rpc_last_probe.values_mut() {
        *last = Instant::now() - Duration::from_secs(60);
    }
    engine.probe_rpc_health(&state);
    assert!(state
        .repository
        .latest_rpc_health(&tail.id)
        .unwrap()
        .is_some());
    assert_eq!(engine.rpc_last_probe.len(), 9);
}

#[test]
fn skipping_an_event_clears_its_persisted_retry_budget() {
    let (_dir, state) = fixture();
    event(&state, 100);
    state
        .repository
        .save_alert_progress(0, &BTreeMap::from([(1, 2)]))
        .unwrap();
    let mut engine = NotificationWorker::bootstrap(&state);
    engine.tick(&state); // default policy is disabled
    assert_eq!(engine.last_routed_event, 1);
    assert!(engine.alert_failures.is_empty());
    assert!(NotificationWorker::bootstrap(&state)
        .alert_failures
        .is_empty());
}
