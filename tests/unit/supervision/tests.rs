use std::{path::PathBuf, sync::Arc};

use super::*;
use crate::types::{Network, NewNode, NodeType, StorageEngine};

#[path = "upgrade/tests.rs"]
mod upgrade;

#[test]
fn failed_watchdog_attempt_schedules_the_next_attempt() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();
    let created = repository
        .create_node(NewNode {
            name: "retry node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/missing/neo-go"),
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();
    repository
        .update_node_status(&created.id, NodeStatus::Error, None)
        .unwrap();
    let node = repository.list_nodes().unwrap().into_iter().next().unwrap();
    let policy =
        RestartPolicy::with_enabled(true, 3, Duration::from_secs(1), Duration::from_secs(4));
    let now = Instant::now();
    let mut loop_state = LoopState {
        watchdog: Watchdog::new(policy),
        applied_policy: policy,
        rpc_last_probe: BTreeMap::new(),
        federation_last_probe: BTreeMap::new(),
        last_routed_event: 0,
    };
    loop_state.watchdog.record_failure(&node.id, now);
    assert_eq!(
        loop_state
            .watchdog
            .due_restarts(now + Duration::from_secs(1))[0]
            .attempt,
        1
    );
    let state = EngineState {
        repository,
        data_dir: directory.path().to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        signer_registry: SignerRegistry::empty(),
    };

    loop_state.schedule_restart(&state, &node, "attempt one failed");
    assert!(matches!(
        loop_state.watchdog.status(&node.id, now),
        crate::watchdog::WatchdogStatus::Pending { attempt: 2, .. }
    ));
}

#[test]
fn launch_refuses_ports_owned_by_another_active_node() {
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();
    let binary = std::env::current_exe().unwrap();
    let active = repository
        .create_node(NewNode {
            name: "active node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: binary.clone(),
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 30332,
            p2p_port: 30333,
            ws_port: None,
        })
        .unwrap();
    repository
        .update_node_status(&active.id, NodeStatus::Running, Some(4242))
        .unwrap();
    let target = repository
        .create_node(NewNode {
            name: "target node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: binary,
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 30332,
            p2p_port: 31333,
            ws_port: None,
        })
        .unwrap();
    let state = EngineState {
        repository,
        data_dir: directory.path().to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        signer_registry: SignerRegistry::empty(),
    };

    let error = launch_node(&state, &target, LaunchAction::Start)
        .expect_err("an active inventory conflict must block launch before spawn");
    assert!(error
        .to_string()
        .contains("overlaps with active active node RPC"));
    assert!(!state.supervisor().is_managing(&target.id));
}
