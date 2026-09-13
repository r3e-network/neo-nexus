use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{atomic::Ordering, Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use super::{
    launch::launch_node,
    state::{EngineState, LoopState},
    Engine,
};
use crate::{
    core::{lifecycle::LaunchAction, node::NodeConfig},
    events::{EventKind, RuntimeEventFilter},
    repository::Repository,
    signing::SignerRegistry,
    supervisor::{log_path_for, ProcessSupervisor},
    types::{Network, NewNode, NodeStatus, NodeType, StorageEngine},
    watchdog::{RestartPolicy, Watchdog},
};

#[path = "upgrade/tests.rs"]
mod upgrade;

/// Wake the engine's log collector out of its 30-second `park_timeout` so a test
/// can drive the next sampling round deterministically instead of sleeping. The
/// handle is private to `supervision`, and this test module is a child of it.
fn wake_collector(engine: &Engine) {
    engine
        .log_collection_handle
        .as_ref()
        .expect("a running engine owns its collector handle")
        .thread()
        .unpark();
}

fn log_collection_state(directory: &std::path::Path) -> (EngineState, NodeConfig) {
    let repository = Repository::open(directory.join("workspace.db")).unwrap();
    let node = repository
        .create_node(NewNode {
            name: "log collection node".to_string(),
            node_type: NodeType::NeoCli,
            network: Network::Testnet,
            binary_path: std::env::current_exe().unwrap(),
            args: Vec::new(),
            runtime_version: "test".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();
    (
        EngineState {
            repository,
            data_dir: directory.to_path_buf(),
            supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
            signer_registry: SignerRegistry::empty(),
        },
        node,
    )
}

fn wait_for_log_event(repository: &Repository, kind: EventKind, text: &str) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let events = repository
            .list_events(RuntimeEventFilter::default())
            .unwrap();
        if events
            .iter()
            .any(|event| event.kind == kind && event.message.contains(text))
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "missing {kind}: {text}; events: {events:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn engine_collects_workspace_logs_and_joins_collector_without_tokio() {
    use std::io::Write;

    let directory = tempfile::tempdir().unwrap();
    let (state, node) = log_collection_state(directory.path());
    let log_path = log_path_for(directory.path().join("logs"), &node);
    std::fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    std::fs::write(
        &log_path,
        "height 10 of 80\nheight 50 of 80\nFATAL database unavailable\n",
    )
    .unwrap();

    let engine = Engine::start(state.clone()).unwrap();
    // Baseline sync event should be recorded immediately.
    wait_for_log_event(
        &state.repository,
        EventKind::SyncProgressRecorded,
        "50/80 blocks (62.5%)",
    );
    // Baseline suppresses historical fatals - no fatal events yet.

    drop(state.supervisor());

    // Append new content + new fatal for append-only detection
    let mut log = std::fs::OpenOptions::new()
        .append(true)
        .open(&log_path)
        .unwrap();
    writeln!(log, "FATAL database corruption").unwrap();
    wake_collector(&engine);
    thread::sleep(Duration::from_millis(300)); // Give round time to process

    // Confirm a fatal event was now emitted (append only)
    wait_for_log_event(
        &state.repository,
        EventKind::LogFatalErrorDetected,
        "FATAL/PANIC",
    );

    let stop = Arc::clone(&engine.stop);
    let shutdown = thread::spawn(move || {
        drop(engine); // sets stop=true, unparks worker, joins handle
    });
    // The join completes quickly because we unpark in Drop.
    shutdown
        .join()
        .expect("collector shutdown must complete within 1s");
    assert!(stop.load(Ordering::Relaxed));
}

#[test]
fn engine_log_collection_bounds_reads_and_tolerates_missing_logs() {
    let directory = tempfile::tempdir().unwrap();
    let (state, node) = log_collection_state(directory.path());
    let engine = Engine::start(state.clone()).unwrap();
    // File absent on start - collector parks and waits.
    // Write a large baseline + fatal in the file after start.
    let log_path = log_path_for(directory.path().join("logs"), &node);
    std::fs::create_dir_all(log_path.parent().unwrap()).unwrap();
    let mut content = String::from("FATAL old archived failure\n");
    content.push_str(&"old line\n".repeat(10_000));
    content.push_str("height 37 of 100\n");
    std::fs::write(&log_path, content).unwrap();

    wait_for_log_event(
        &state.repository,
        EventKind::SyncProgressRecorded,
        "37/100 blocks (37.0%)",
    );

    // Baseline suppresses historical fatals - confirm none recorded
    drop(engine);
    let events = state
        .repository
        .list_events(RuntimeEventFilter::default())
        .unwrap();
    assert!(!events
        .iter()
        .any(|event| event.kind == EventKind::LogFatalErrorDetected));
}

#[test]
fn registered_log_parsers_never_invent_sync_targets_or_percentages() {
    let supervisor = ProcessSupervisor::default();
    let cases = [
        (
            NodeType::NeoCli,
            "height 37 of 100 peers=3",
            "height 37",
            "height 37 of 0",
        ),
        (
            NodeType::NeoGo,
            "blockHeight=37 targetHeight=100 peers=3",
            "blockHeight=37",
            "blockHeight=37 targetHeight=0",
        ),
        (
            NodeType::NeoXGeth,
            "Chain imported block=37 of 100 peers=3",
            "Chain imported block=37",
            "Chain imported block=37 of 0",
        ),
        (
            NodeType::NeoXReth,
            "Block #37 state root targetHeight=100 peers=3",
            "Block #37 state root",
            "Block #37 state root targetHeight=0",
        ),
    ];
    for (node_type, measured, missing_target, zero_target) in cases {
        let parser = supervisor.adapters().get_log_parser(&node_type).unwrap();
        let progress = parser.extract_sync_progress(&[measured]).unwrap();
        assert_eq!((progress.current_height, progress.target_height), (37, 100));
        assert_eq!(progress.sync_percentage, 37.0);
        assert_eq!(progress.peers_connected, 3);
        assert!(parser.extract_sync_progress(&[missing_target]).is_none());
        assert!(parser.extract_sync_progress(&[zero_target]).is_none());
        assert!(parser
            .extract_sync_progress(&["ordinary log message"])
            .is_none());
    }
}

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
