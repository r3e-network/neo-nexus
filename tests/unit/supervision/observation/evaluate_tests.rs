use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use super::*;
use crate::{
    events::{EventKind, RuntimeEventFilter},
    observe::{Evidence, HealthState, Observation},
    repository::Repository,
    signing::SignerRegistry,
    supervisor::ProcessSupervisor,
    types::{ChainFamily, Network, NewNode, NodeStatus, NodeType, StorageEngine},
};

const NOW: u64 = 1_770_000_000;

fn workspace(directory: &std::path::Path) -> EngineState {
    let repository = Repository::open(directory.join("workspace.db")).unwrap();
    EngineState {
        repository,
        data_dir: directory.to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        signer_registry: SignerRegistry::empty(),
    }
}

fn node(state: &EngineState, name: &str, rpc_port: u16, status: NodeStatus) -> NodeConfig {
    let node = state
        .repository
        .create_node(NewNode {
            name: name.to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Private,
            binary_path: PathBuf::from("/opt/neo/neo-go"),
            args: Vec::new(),
            runtime_version: "0.122".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port,
            p2p_port: rpc_port + 1,
            ws_port: None,
        })
        .unwrap();
    state
        .repository
        .update_node_status(&node.id, status, Some(4321))
        .unwrap();
    state
        .repository
        .list_nodes()
        .unwrap()
        .into_iter()
        .find(|stored| stored.id == node.id)
        .unwrap()
}

/// A round in which the node answered at `height`, with the protocol constants
/// a `getversion` would have supplied.
fn answered(node: &NodeConfig, height: u64, at: u64, magic: u64) -> NodeSample {
    let endpoint = format!("http://127.0.0.1:{}", node.rpc_port);
    let evidence = |field: &'static str, value: String| {
        Evidence::recorded("getblockcount", field, value, &endpoint, at)
    };
    let mut sample = NodeSample::not_observable(&node.id, at);
    sample.endpoint = endpoint.clone();
    sample.head_ok = true;
    sample.head_latency_ms = Some(9);
    sample.block_height = Observation::Known(height, evidence("block_height", height.to_string()));
    sample.peers_connected = Observation::Known(6, evidence("peers_connected", "6".to_string()));
    sample.observed_magic = Observation::Known(magic, evidence("magic", magic.to_string()));
    sample.ms_per_block = Observation::Known(15_000, evidence("ms_per_block", "15000".to_string()));
    sample
}

/// Run the evaluator enough times to get past the two-evaluation debounce.
fn settle(state: &EngineState, nodes: &[NodeConfig], tracker: &mut HealthTracker, now_unix: u64) {
    let scheduler = Scheduler::default();
    evaluate_fleet(state, &scheduler, nodes, tracker, now_unix);
    evaluate_fleet(state, &scheduler, nodes, tracker, now_unix);
}

/// One late round must not put an entry in a node's timeline and a webhook in
/// someone's inbox. A new state has to be seen twice before it replaces the
/// stored one.
#[test]
fn a_new_state_is_held_until_it_has_been_seen_twice() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = node(&state, "rpc-1", 30332, NodeStatus::Running);
    let nodes = vec![node.clone()];
    let scheduler = Scheduler::default();
    let mut tracker = HealthTracker::default();

    state
        .repository
        .record_node_sample(&answered(&node, 8_421, NOW, 1_230_000))
        .unwrap();

    evaluate_fleet(&state, &scheduler, &nodes, &mut tracker, NOW);
    assert!(
        state
            .repository
            .load_node_health(&node.id)
            .unwrap()
            .is_none(),
        "one sighting is not a verdict"
    );

    evaluate_fleet(&state, &scheduler, &nodes, &mut tracker, NOW);
    let health = state
        .repository
        .load_node_health(&node.id)
        .unwrap()
        .unwrap();
    assert_eq!(health.state, HealthState::Healthy);
}

/// A stop is a fact, not an inference. Making an operator who has just pressed
/// Stop wait two monitoring intervals while the console says `Unreachable` is
/// the console arguing with something they did on purpose.
#[test]
fn stopping_a_node_is_reflected_without_waiting_to_be_confirmed() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = node(&state, "rpc-1", 30332, NodeStatus::Stopped);
    let nodes = vec![node.clone()];

    evaluate_fleet(
        &state,
        &Scheduler::default(),
        &nodes,
        &mut HealthTracker::default(),
        NOW,
    );
    let health = state
        .repository
        .load_node_health(&node.id)
        .unwrap()
        .unwrap();
    assert_eq!(health.state, HealthState::Stopped);
}

/// A crashed node is evaluated even though nothing polls it, and reads as
/// unreachable rather than as unchecked. Sampling only ever visits running
/// nodes, so a node that died would otherwise keep the verdict it held while
/// it was alive.
#[test]
fn a_node_that_died_is_judged_even_though_nothing_samples_it() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = node(&state, "rpc-1", 30332, NodeStatus::Running);
    let mut nodes = vec![node.clone()];
    let mut tracker = HealthTracker::default();

    state
        .repository
        .record_node_sample(&answered(&node, 8_421, NOW, 1_230_000))
        .unwrap();
    settle(&state, &nodes, &mut tracker, NOW);
    assert_eq!(
        state
            .repository
            .load_node_health(&node.id)
            .unwrap()
            .unwrap()
            .state,
        HealthState::Healthy
    );

    state
        .repository
        .update_node_status(&node.id, NodeStatus::Error, None)
        .unwrap();
    nodes[0].status = NodeStatus::Error;
    settle(&state, &nodes, &mut tracker, NOW + 60);

    let health = state
        .repository
        .load_node_health(&node.id)
        .unwrap()
        .unwrap();
    assert_eq!(health.state, HealthState::Unreachable);
    assert!(health.reason.contains("no process"));
}

/// The moment a state was entered is what an operator reads as "stalled for
/// twelve minutes", and it must survive every re-confirmation of that state.
/// Refreshing it each round would turn a long incident into a permanently
/// fresh-looking one.
#[test]
fn re_confirming_a_state_keeps_the_time_it_was_entered() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = node(&state, "rpc-1", 30332, NodeStatus::Running);
    let nodes = vec![node.clone()];
    let mut tracker = HealthTracker::default();

    state
        .repository
        .record_node_sample(&answered(&node, 8_421, NOW, 1_230_000))
        .unwrap();
    settle(&state, &nodes, &mut tracker, NOW);
    let entered = state
        .repository
        .load_node_health(&node.id)
        .unwrap()
        .unwrap()
        .since_unix;

    // Five minutes later, still healthy, with the height moving as it should.
    for round in 1..=20u64 {
        state
            .repository
            .record_node_sample(&answered(&node, 8_421 + round, NOW + round * 15, 1_230_000))
            .unwrap();
    }
    evaluate_fleet(
        &state,
        &Scheduler::default(),
        &nodes,
        &mut tracker,
        NOW + 300,
    );

    let health = state
        .repository
        .load_node_health(&node.id)
        .unwrap()
        .unwrap();
    assert_eq!(health.state, HealthState::Healthy);
    assert_eq!(health.since_unix, entered, "the state did not change");
    assert_eq!(
        health.evaluated_at_unix,
        NOW + 300,
        "but the judgement is current"
    );
    assert_eq!(health.held_for_seconds(NOW + 300), 300);
}

/// A change of state is recorded once, in the timeline and in the journal, so
/// "it was fine an hour ago" is a question the workspace answers.
#[test]
fn a_change_of_state_is_written_to_the_timeline_and_the_journal() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = node(&state, "rpc-1", 30332, NodeStatus::Running);
    let nodes = vec![node.clone()];
    let mut tracker = HealthTracker::default();

    state
        .repository
        .record_node_sample(&answered(&node, 8_421, NOW, 1_230_000))
        .unwrap();
    settle(&state, &nodes, &mut tracker, NOW);

    // The height has not moved for well over the stall threshold.
    state
        .repository
        .record_node_sample(&answered(&node, 8_421, NOW + 600, 1_230_000))
        .unwrap();
    settle(&state, &nodes, &mut tracker, NOW + 600);

    let timeline = state
        .repository
        .recent_health_transitions(&node.id, 10)
        .unwrap();
    assert_eq!(timeline.len(), 2, "first verdict, then the change");
    assert_eq!(timeline[0].from, Some(HealthState::Healthy));
    assert_eq!(timeline[0].to, HealthState::Stalled);
    assert_eq!(
        timeline[1].from, None,
        "the first verdict came from nowhere"
    );

    let journalled = state
        .repository
        .list_events(RuntimeEventFilter::default())
        .unwrap()
        .into_iter()
        .filter(|event| event.kind == EventKind::NodeHealthChanged)
        .count();
    assert_eq!(journalled, 2);
}

/// Lag is measured against the chain a node **actually joined**, read from its
/// own `getversion`, not the network it was configured with.
///
/// A node set to a private network that fell back to compiled-in MainNet
/// defaults is on MainNet. Comparing its height against the other private
/// nodes' would report a lag of several hundred million blocks instead of the
/// configuration error that produced it — so the two do not share a reference
/// head at all.
#[test]
fn nodes_are_compared_only_against_the_chain_they_actually_joined() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let ahead = node(&state, "private-1", 30332, NodeStatus::Running);
    let behind = node(&state, "private-2", 30432, NodeStatus::Running);
    let stray = node(&state, "stray", 30532, NodeStatus::Running);
    let nodes = vec![ahead.clone(), behind.clone(), stray.clone()];

    const PRIVATE: u64 = 1_230_000;
    const MAINNET: u64 = 860_833_102;
    for (node, height, magic) in [
        (&ahead, 9_000, PRIVATE),
        (&behind, 8_000, PRIVATE),
        (&stray, 6_500_000, MAINNET),
    ] {
        state
            .repository
            .record_node_sample(&answered(node, height, NOW, magic))
            .unwrap();
    }

    let histories: std::collections::BTreeMap<String, Vec<NodeSample>> = nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                state.repository.recent_node_samples(&node.id, 8).unwrap(),
            )
        })
        .collect();
    let heads = reference_heads(&nodes, &histories);

    assert_eq!(
        heads.get(&behind.id),
        Some(&ReferenceHead::Known {
            height: 9_000,
            source: "private-1".to_string()
        }),
        "the private pair share a head"
    );
    assert!(
        !heads.contains_key(&stray.id),
        "a node alone on its chain has nothing to be compared against"
    );
}

/// A node alone on its chain reports no reference head at all.
///
/// Comparing a node against itself always puts it exactly at the head, and
/// "0 blocks behind" on a single-node private chain is a number that means
/// nothing while reading as reassurance.
#[test]
fn a_node_alone_on_its_chain_is_not_compared_against_itself() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let only = node(&state, "solo", 30332, NodeStatus::Running);
    let nodes = vec![only.clone()];

    state
        .repository
        .record_node_sample(&answered(&only, 42, NOW, 1_230_000))
        .unwrap();
    let histories: std::collections::BTreeMap<String, Vec<NodeSample>> = nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                state.repository.recent_node_samples(&node.id, 8).unwrap(),
            )
        })
        .collect();

    assert!(reference_heads(&nodes, &histories).is_empty());
    assert_eq!(
        NodeSample::not_observable(&only.id, NOW).chain_key(ChainFamily::NeoN3),
        None,
        "and a node that has not reported its magic cannot be grouped either"
    );
}

/// Deleting a node must not leave the debounce holding its id for the life of
/// the process.
#[test]
fn a_deleted_node_is_dropped_from_the_debounce() {
    let dir = tempfile::tempdir().unwrap();
    let state = workspace(dir.path());
    let node = node(&state, "rpc-1", 30332, NodeStatus::Running);
    let nodes = vec![node.clone()];
    let mut tracker = HealthTracker::default();

    state
        .repository
        .record_node_sample(&answered(&node, 8_421, NOW, 1_230_000))
        .unwrap();
    evaluate_fleet(&state, &Scheduler::default(), &nodes, &mut tracker, NOW);
    assert_eq!(tracker.pending.len(), 1, "held, awaiting confirmation");

    tracker.retain(&[]);
    assert!(tracker.pending.is_empty());
}
