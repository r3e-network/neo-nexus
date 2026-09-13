use super::*;

use std::path::PathBuf;

use crate::types::{Network, NodeStatus, NodeType, StorageEngine};

fn node(id: &str, rpc_port: u16) -> NodeConfig {
    NodeConfig {
        id: id.to_string(),
        name: id.to_string(),
        node_type: NodeType::NeoGo,
        network: Network::Testnet,
        binary_path: PathBuf::from("/opt/neo/neo-go"),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port,
        p2p_port: rpc_port + 1,
        ws_port: None,
        status: NodeStatus::Running,
        pid: Some(4242),
    }
}

fn fleet(count: usize) -> Vec<NodeConfig> {
    (0..count)
        .map(|index| node(&format!("node-{index}"), 10332 + index as u16 * 2))
        .collect()
}

/// Nothing has been asked yet, so everything is due.
#[test]
fn a_fresh_scheduler_asks_every_class_once() {
    let scheduler = Scheduler::default();
    let due = scheduler.due(&fleet(1), Instant::now(), &ObservationPolicy::default());
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].classes.len(), SampleClass::ALL.len());
}

/// A tick is bounded. Sampling is blocking I/O, and without a ceiling a fleet
/// of unreachable nodes would hold the supervision loop for timeout × fleet
/// size — stalling restarts and alert routing behind it.
#[test]
fn one_pass_never_contacts_more_nodes_than_the_policy_allows() {
    let scheduler = Scheduler::default();
    let policy = ObservationPolicy::default();
    let due = scheduler.due(&fleet(50), Instant::now(), &policy);
    assert_eq!(due.len(), policy.max_nodes_per_tick);
    assert!(policy.max_nodes_per_tick < 50);
}

/// Each class has its own period, so a pass shortly after the last one asks
/// only what has come due — not everything again.
#[test]
fn classes_come_due_on_their_own_schedules() {
    let mut scheduler = Scheduler::default();
    let start = Instant::now();
    let fleet = fleet(1);
    let all: Vec<SampleClass> = SampleClass::ALL.into_iter().collect();
    scheduler.record("node-0", &all, start, true);

    // Twenty seconds later: the fifteen-second classes are due, the rest are not.
    let later = start + Duration::from_secs(20);
    let due = scheduler.due(&fleet, later, &ObservationPolicy::default());
    let classes = &due[0].classes;
    assert!(classes.contains(&SampleClass::Head));
    assert!(classes.contains(&SampleClass::Peers));
    assert!(!classes.contains(&SampleClass::HeadTime));
    assert!(!classes.contains(&SampleClass::Identity));

    // Two minutes later the slower classes have come round, except identity,
    // whose constants change only when a node restarts.
    let later = start + Duration::from_secs(125);
    let classes = &scheduler.due(&fleet, later, &ObservationPolicy::default())[0].classes;
    assert!(classes.contains(&SampleClass::HeadTime));
    assert!(classes.contains(&SampleClass::Pool));
    assert!(!classes.contains(&SampleClass::Identity));
}

/// A node that is not answering is asked less often, and asked only whether it
/// is back — its mempool depth is not the question.
#[test]
fn a_failing_node_is_backed_off_and_asked_only_about_liveness() {
    let mut scheduler = Scheduler::default();
    let start = Instant::now();
    let fleet = fleet(1);
    let all: Vec<SampleClass> = SampleClass::ALL.into_iter().collect();

    scheduler.record("node-0", &all, start, false);
    assert_eq!(scheduler.consecutive_failures("node-0"), 1);

    // At 20s the un-backed-off head period (15s) would be due; doubled, it is not.
    let due = scheduler.due(
        &fleet,
        start + Duration::from_secs(20),
        &ObservationPolicy::default(),
    );
    assert!(due.is_empty(), "a failing node was asked again too soon");

    let due = scheduler.due(
        &fleet,
        start + Duration::from_secs(31),
        &ObservationPolicy::default(),
    );
    assert_eq!(due[0].classes, vec![SampleClass::Head]);
}

/// Backoff is bounded: a node down for an hour costs a handful of requests,
/// and the operator still sees a recent "last checked" rather than a figure
/// that has aged for the whole outage.
#[test]
fn backoff_is_geometric_and_capped() {
    let mut scheduler = Scheduler::default();
    let start = Instant::now();
    let head = [SampleClass::Head];
    for _ in 0..20 {
        scheduler.record("node-0", &head, start, false);
    }
    let fleet = fleet(1);

    // Capped at ×8 of the fifteen-second head period.
    let just_under = start + Duration::from_secs(119);
    assert!(scheduler
        .due(&fleet, just_under, &ObservationPolicy::default())
        .is_empty());
    let just_over = start + Duration::from_secs(121);
    assert!(!scheduler
        .due(&fleet, just_over, &ObservationPolicy::default())
        .is_empty());
}

/// One success clears the backoff outright. A node that has come back should
/// be watched closely again immediately; the cost of being wrong is one extra
/// request.
#[test]
fn a_single_success_restores_the_full_sampling_rate() {
    let mut scheduler = Scheduler::default();
    let start = Instant::now();
    let head = [SampleClass::Head];
    for _ in 0..5 {
        scheduler.record("node-0", &head, start, false);
    }
    assert_eq!(scheduler.consecutive_failures("node-0"), 5);

    scheduler.record("node-0", &head, start, true);
    assert_eq!(scheduler.consecutive_failures("node-0"), 0);

    let due = scheduler.due(
        &fleet(1),
        start + Duration::from_secs(16),
        &ObservationPolicy::default(),
    );
    assert!(
        due[0].classes.len() > 1,
        "a recovered node should be asked the full set again"
    );
}

/// A node with no RPC port is asked once, so its row can say why it is empty,
/// and then never again — there is nothing to ask and no cost worth paying.
#[test]
fn a_node_with_rpc_disabled_is_asked_once_and_then_left_alone() {
    let mut scheduler = Scheduler::default();
    let start = Instant::now();
    let fleet = vec![node("node-0", 0)];

    let due = scheduler.due(&fleet, start, &ObservationPolicy::default());
    assert_eq!(due.len(), 1, "it should be recorded as unobservable once");
    scheduler.record("node-0", &due[0].classes, start, false);

    let later = start + Duration::from_secs(3600);
    assert!(
        scheduler
            .due(&fleet, later, &ObservationPolicy::default())
            .is_empty(),
        "a node with no RPC port must not be polled forever"
    );
}

#[test]
fn sampling_can_be_switched_off_entirely() {
    let scheduler = Scheduler::default();
    let policy = ObservationPolicy {
        enabled: false,
        ..ObservationPolicy::default()
    };
    assert!(scheduler.due(&fleet(5), Instant::now(), &policy).is_empty());
}

/// A deleted node must not keep its id alive in memory for the life of the
/// process.
#[test]
fn forgetting_a_node_clears_everything_remembered_about_it() {
    let mut scheduler = Scheduler::default();
    let start = Instant::now();
    scheduler.record("node-0", &[SampleClass::Head], start, false);
    assert_eq!(scheduler.consecutive_failures("node-0"), 1);

    scheduler.forget("node-0");
    assert_eq!(scheduler.consecutive_failures("node-0"), 0);
    assert!(
        !scheduler.due(&fleet(1), start, &ObservationPolicy::default())[0]
            .classes
            .is_empty()
    );
}
