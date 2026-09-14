use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use super::*;
use crate::{
    observe::{NotSampled, Observation},
    repository::Repository,
    rpc_health::RpcHealthMonitorPolicy,
    signing::SignerRegistry,
    supervisor::ProcessSupervisor,
    types::{ChainFamily, Network, NewNode, NodeStatus, NodeType, StorageEngine},
};

fn workspace(directory: &std::path::Path) -> EngineState {
    let repository = Repository::open(directory.join("workspace.db")).unwrap();
    EngineState {
        repository,
        data_dir: directory.to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        signer_registry: SignerRegistry::empty(),
        metrics: Arc::new(crate::metrics::MetricsStore::default()),
    }
}

/// A node marked running, so the pass will consider it. `rpc_port` decides
/// whether it can be asked anything at all.
fn running_node(
    state: &EngineState,
    name: &str,
    node_type: NodeType,
    rpc_port: u16,
    p2p_port: u16,
) -> NodeConfig {
    let node = state
        .repository
        .create_node(NewNode {
            name: name.to_string(),
            node_type,
            network: Network::Private,
            binary_path: PathBuf::from("/opt/neo/node"),
            args: Vec::new(),
            runtime_version: "0.122".to_string(),
            // Each family keeps its own supported engine; the workspace
            // refuses the wrong pairing at creation.
            storage_engine: match node_type.family() {
                ChainFamily::NeoN3 => StorageEngine::LevelDb,
                ChainFamily::NeoX => StorageEngine::RocksDb,
            },
            rpc_port,
            p2p_port,
            ws_port: None,
        })
        .unwrap();
    state
        .repository
        .update_node_status(&node.id, NodeStatus::Running, Some(4321))
        .unwrap();
    state
        .repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|stored| stored.id == node.id)
        .unwrap()
}

fn answered_head(node: &NodeConfig, height: u64) -> NodeSample {
    let mut sample = NodeSample::not_observable(&node.id, 1_770_000_000);
    sample.endpoint = format!("http://127.0.0.1:{}", node.rpc_port);
    sample.head_ok = true;
    sample.head_latency_ms = Some(7);
    sample.block_height = Observation::Known(
        height,
        crate::observe::Evidence::recorded(
            head_method(node.node_type.family()),
            "result",
            height.to_string(),
            &sample.endpoint,
            1_770_000_000,
        ),
    );
    sample
}

/// The bug that made this rewrite necessary, pinned in the compatibility layer.
///
/// The probe this replaces called `getversion` and `getblockcount` and derived
/// status from how many of the two answered. A Neo X node implements neither
/// spelling, so a node serving every request put to it read as `Degraded`
/// forever. Here the node answered its liveness call, and that is the whole
/// question — the classes that were not due carry no verdict.
#[test]
fn a_node_that_answered_is_healthy_even_when_nothing_else_was_asked() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = running_node(&state, "neox-1", NodeType::NeoXGeth, 8545, 30303);

    let sample = answered_head(&node, 4_002);
    assert!(
        !sample.client_version.is_known(),
        "the identity class was not due this round"
    );

    let report = legacy_report(&node, &sample);
    assert_eq!(report.status, RpcHealthStatus::Healthy);
    assert_eq!(report.block_count, Some(4_002));
    assert_eq!(report.version, None);
}

/// A failure report has to name the call that was actually made. Neo X is asked
/// `eth_blockNumber`; blaming `getblockcount` would send an operator to check a
/// method their client does not have.
#[test]
fn a_failure_names_the_method_the_family_was_actually_asked() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let neox = running_node(&state, "neox-1", NodeType::NeoXGeth, 8545, 30303);
    let neo_n3 = running_node(&state, "n3-1", NodeType::NeoGo, 10332, 10333);

    let down = |node: &NodeConfig| {
        let sample = NodeSample::unreachable(
            &node.id,
            1_770_000_000,
            format!("http://127.0.0.1:{}", node.rpc_port),
            NotSampled::CallFailed {
                method: head_method(node.node_type.family()),
                detail: "connection refused".to_string(),
            },
        );
        legacy_report(node, &sample)
    };

    let evm = down(&neox);
    assert_eq!(evm.status, RpcHealthStatus::Unreachable);
    assert_eq!(evm.methods[0].method, "eth_blockNumber");
    assert!(
        evm.message().contains("connection refused"),
        "the sampler's own reason must survive: {}",
        evm.message()
    );

    let n3 = down(&neo_n3);
    assert_eq!(n3.methods[0].method, "getblockcount");
}

/// One round, both tables, from the same evidence — so the new health surface
/// and the console's existing RPC panel cannot tell an operator two different
/// stories about the same moment.
#[test]
fn a_pass_writes_the_round_and_its_compatibility_row_together() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    // Port 1 is not listening, so the call is refused immediately rather than
    // spending the policy timeout.
    let node = running_node(&state, "n3-1", NodeType::NeoGo, 1, 10333);

    let mut observation = ObservationState::default();
    observe_once(&state, &mut observation);

    let sample = state
        .repository
        .latest_node_sample(&node.id)
        .unwrap()
        .expect("the pass records what it learned");
    assert!(!sample.head_ok);
    assert!(
        !sample.block_height.is_known(),
        "a refused call must not produce a height"
    );

    let legacy = state
        .repository
        .latest_rpc_health(&node.id)
        .unwrap()
        .expect("the compatibility row is written from the same round");
    assert_eq!(legacy.status, RpcHealthStatus::Unreachable);
    assert_eq!(legacy.block_count, None);

    assert_eq!(
        observation.scheduler.consecutive_failures(&node.id),
        1,
        "the pass must tell the scheduler to back off"
    );
}

/// "We cannot ask this node" is not "this node did not answer". The probe this
/// replaces skipped RPC-less nodes entirely, and writing an endpoint-less
/// `unreachable` row for one would turn a configuration fact into an outage on
/// every surface that still reads the old table.
#[test]
fn a_node_with_no_rpc_port_is_recorded_without_being_called_unreachable() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = running_node(&state, "consensus-1", NodeType::NeoGo, 0, 10333);

    let mut observation = ObservationState::default();
    observe_once(&state, &mut observation);

    let sample = state
        .repository
        .latest_node_sample(&node.id)
        .unwrap()
        .expect("the node is still recorded, so its row says why it is blank");
    assert!(sample.endpoint.is_empty());
    assert!(sample
        .block_height
        .render(|height| height.to_string())
        .contains("RPC is disabled"));

    assert!(
        state
            .repository
            .latest_rpc_health(&node.id)
            .unwrap()
            .is_none(),
        "an RPC-less node must not appear in the RPC health table at all"
    );
}

/// Turning monitoring off in Settings has to actually stop the traffic. A
/// policy the workspace offers but does not honour is worse than no policy.
#[test]
fn a_disabled_monitor_policy_stops_the_pass() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = running_node(&state, "n3-1", NodeType::NeoGo, 1, 10333);
    state
        .repository
        .save_rpc_health_monitor_policy(RpcHealthMonitorPolicy {
            enabled: false,
            interval_seconds: 30,
        })
        .unwrap();

    let mut observation = ObservationState::default();
    observe_once(&state, &mut observation);

    assert!(state
        .repository
        .latest_node_sample(&node.id)
        .unwrap()
        .is_none());
    assert!(state
        .repository
        .latest_rpc_health(&node.id)
        .unwrap()
        .is_none());
}

/// A stopped node is not asked. Its state is already known from the process
/// table, and polling it would spend a timeout per round to learn nothing.
#[test]
fn a_stopped_node_is_not_polled() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = running_node(&state, "n3-1", NodeType::NeoGo, 1, 10333);
    state
        .repository
        .update_node_status(&node.id, NodeStatus::Stopped, None)
        .unwrap();

    let mut observation = ObservationState::default();
    observe_once(&state, &mut observation);

    assert!(state
        .repository
        .latest_node_sample(&node.id)
        .unwrap()
        .is_none());
}

/// Deleting a node must not leave its id in the scheduler's maps for the life
/// of the process — and the cleanup has to happen even while monitoring is off,
/// which is precisely when nothing else would notice the node is gone.
#[test]
fn a_deleted_node_is_forgotten_even_with_monitoring_disabled() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = running_node(&state, "n3-1", NodeType::NeoGo, 1, 10333);

    let mut observation = ObservationState::default();
    observe_once(&state, &mut observation);
    assert_eq!(observation.known, vec![node.id.clone()]);

    state
        .repository
        .save_rpc_health_monitor_policy(RpcHealthMonitorPolicy {
            enabled: false,
            interval_seconds: 30,
        })
        .unwrap();
    state
        .repository
        .update_node_status(&node.id, NodeStatus::Stopped, None)
        .unwrap();
    state.repository.delete_node(&node.id).unwrap();
    observe_once(&state, &mut observation);

    assert!(observation.known.is_empty());
    assert_eq!(observation.scheduler.consecutive_failures(&node.id), 0);
}
