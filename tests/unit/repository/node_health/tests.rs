use std::path::PathBuf;

use crate::{
    observe::{HealthState, HealthTransition, NextStep, NodeHealth, StallScope},
    repository::Repository,
    types::{Network, NewNode, NodeType, StorageEngine},
};

fn workspace() -> (tempfile::TempDir, Repository) {
    let dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(dir.path().join("n.db")).unwrap();
    (dir, repository)
}

fn node(repository: &Repository, name: &str, rpc_port: u16) -> String {
    repository
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
        .unwrap()
        .id
}

fn health(node_id: &str, state: HealthState, since_unix: u64) -> NodeHealth {
    NodeHealth {
        node_id: node_id.to_string(),
        state,
        since_unix,
        evaluated_at_unix: since_unix + 30,
        reason: format!("{} for a reason worth reading", state.label()),
        scope: None,
        cause: None,
        next: NextStep::here("Open the node", format!("/nodes/{node_id}")),
    }
}

/// The whole verdict survives, including the parts a surface cannot recompute:
/// when the state was entered, what it was blamed on, and where to go next.
#[test]
fn a_verdict_round_trips_with_its_scope_its_cause_and_its_next_step() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    let stalled = NodeHealth {
        scope: Some(StallScope::Chain),
        cause: Some("The node has no connected peers.".to_string()),
        next: NextStep::here("Read the log", format!("/logs?node={id}")),
        ..health(&id, HealthState::Stalled, 1_770_000_000)
    };
    repository.save_node_health(&stalled).unwrap();

    assert_eq!(repository.load_node_health(&id).unwrap(), Some(stalled));
}

/// `since_unix` is when the state was *entered*, and it is the number that
/// decides whether to act. "Stalled for 12 minutes" and "stalled, checked 15
/// seconds ago" are different sentences and the row has to carry both.
#[test]
fn the_row_distinguishes_how_long_a_state_has_held_from_how_fresh_it_is() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    let stalled = NodeHealth {
        since_unix: 1_770_000_000,
        evaluated_at_unix: 1_770_000_705,
        ..health(&id, HealthState::Stalled, 1_770_000_000)
    };
    repository.save_node_health(&stalled).unwrap();

    let stored = repository.load_node_health(&id).unwrap().unwrap();
    assert_eq!(stored.held_for_seconds(1_770_000_720), 720);
    assert_eq!(stored.evaluated_seconds_ago(1_770_000_720), 15);
}

/// A step with nowhere to go must stay a sentence rather than becoming a link
/// that leads nowhere — the distinction is what keeps advice from turning into
/// a dead end at three in the morning.
#[test]
fn a_step_this_console_cannot_take_survives_as_advice_not_as_a_link() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    let elsewhere = NodeHealth {
        next: NextStep::external("This needs a committee vote on chain."),
        ..health(&id, HealthState::Degraded, 1_770_000_000)
    };
    repository.save_node_health(&elsewhere).unwrap();

    let stored = repository.load_node_health(&id).unwrap().unwrap();
    assert_eq!(
        stored.next,
        NextStep::external("This needs a committee vote on chain.")
    );
}

/// Re-evaluating replaces the verdict rather than accumulating verdicts. One
/// row per node is what makes "the current state of the fleet" a single read.
#[test]
fn re_evaluating_replaces_the_verdict() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    repository
        .save_node_health(&health(&id, HealthState::Healthy, 1_770_000_000))
        .unwrap();
    repository
        .save_node_health(&health(&id, HealthState::Stalled, 1_770_000_600))
        .unwrap();

    let stored = repository.load_node_health(&id).unwrap().unwrap();
    assert_eq!(stored.state, HealthState::Stalled);
    assert_eq!(stored.since_unix, 1_770_000_600);
    assert_eq!(repository.list_node_health().unwrap().len(), 1);
}

/// The fleet comes back worst first, so "what should I look at" is answered by
/// reading from the top rather than by a sort every caller has to get right.
#[test]
fn the_fleet_is_ordered_by_what_needs_attention_first() {
    let (_dir, repository) = workspace();
    let healthy = node(&repository, "healthy", 30332);
    let stalled = node(&repository, "stalled", 30432);
    let unreachable = node(&repository, "unreachable", 30532);
    let syncing = node(&repository, "syncing", 30632);

    repository
        .save_node_health(&health(&healthy, HealthState::Healthy, 1_770_000_000))
        .unwrap();
    repository
        .save_node_health(&health(&stalled, HealthState::Stalled, 1_770_000_000))
        .unwrap();
    repository
        .save_node_health(&health(
            &unreachable,
            HealthState::Unreachable,
            1_770_000_000,
        ))
        .unwrap();
    repository
        .save_node_health(&health(&syncing, HealthState::Syncing, 1_770_000_000))
        .unwrap();

    let states: Vec<HealthState> = repository
        .list_node_health()
        .unwrap()
        .iter()
        .map(|health| health.state)
        .collect();
    assert_eq!(
        states,
        vec![
            HealthState::Unreachable,
            HealthState::Stalled,
            HealthState::Syncing,
            HealthState::Healthy,
        ]
    );
}

/// Within one state, the node that has been there longest comes first — it is
/// the one that has been broken longest and the one most likely to still be
/// broken for the same reason nobody has looked at.
#[test]
fn nodes_in_the_same_state_are_ordered_oldest_first() {
    let (_dir, repository) = workspace();
    let recent = node(&repository, "recent", 30332);
    let ancient = node(&repository, "ancient", 30432);

    repository
        .save_node_health(&health(&recent, HealthState::Stalled, 1_770_000_600))
        .unwrap();
    repository
        .save_node_health(&health(&ancient, HealthState::Stalled, 1_770_000_000))
        .unwrap();

    let order: Vec<String> = repository
        .list_node_health()
        .unwrap()
        .iter()
        .map(|health| health.node_id.clone())
        .collect();
    assert_eq!(order, vec![ancient, recent]);
}

/// The first verdict on a node is not a transition. Rendering it as one would
/// put an event in the timeline that never happened.
#[test]
fn the_first_verdict_has_no_state_to_have_come_from() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    let first = HealthTransition {
        node_id: id.clone(),
        at_unix: 1_770_000_000,
        from: None,
        to: HealthState::Healthy,
        reason: "at height 8421, answering, and keeping up".to_string(),
    };
    repository.record_health_transition(&first).unwrap();

    let stored = repository.recent_health_transitions(&id, 10).unwrap();
    assert_eq!(stored, vec![first]);
    assert!(stored[0].summary().starts_with("Healthy:"));
    assert!(
        !stored[0].summary().contains('→'),
        "there was nothing to come from: {}",
        stored[0].summary()
    );
}

/// A timeline reads newest first, and every entry says what it came from.
#[test]
fn transitions_come_back_newest_first_and_name_both_ends() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    for (at, from, to) in [
        (1_770_000_000, None, HealthState::Healthy),
        (
            1_770_000_600,
            Some(HealthState::Healthy),
            HealthState::Stalled,
        ),
        (
            1_770_001_200,
            Some(HealthState::Stalled),
            HealthState::Healthy,
        ),
    ] {
        repository
            .record_health_transition(&HealthTransition {
                node_id: id.clone(),
                at_unix: at,
                from,
                to,
                reason: "because".to_string(),
            })
            .unwrap();
    }

    let stored = repository.recent_health_transitions(&id, 10).unwrap();
    assert_eq!(stored[0].at_unix, 1_770_001_200);
    assert_eq!(stored[0].summary(), "Stalled → Healthy: because");
    assert_eq!(stored.len(), 3);

    repository
        .prune_health_transitions_keep_recent_per_node(2)
        .unwrap();
    assert_eq!(
        repository.recent_health_transitions(&id, 10).unwrap().len(),
        2
    );
}

/// A state written by a newer build is refused, not defaulted.
///
/// Mapping an unrecognised state to `Healthy` would turn an incident into a
/// pass; mapping it to `Unknown` would discard a real verdict. Neither is
/// something to do quietly, so the read fails and says what it found.
#[test]
fn a_state_this_build_does_not_know_is_refused_rather_than_guessed() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);
    repository
        .save_node_health(&health(&id, HealthState::Healthy, 1_770_000_000))
        .unwrap();

    let raw = rusqlite::Connection::open(repository.db_path()).unwrap();
    raw.execute(
        "UPDATE node_health_state SET state = 'quarantined' WHERE node_id = ?1",
        rusqlite::params![id],
    )
    .unwrap();
    drop(raw);

    let error = repository.load_node_health(&id).unwrap_err().to_string();
    assert!(error.contains("quarantined"), "{error}");
}
