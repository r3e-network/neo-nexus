use super::{
    reliability::{due_now, immediate_watchdog, node},
    *,
};
use crate::watchdog::{unix_millis, WatchdogStatus};

#[test]
fn pending_and_exhausted_budgets_survive_each_server_restart() {
    let (_dir, state) = fixture();
    let node = node(&state, "durable", NodeStatus::Running, Some(4_000_000));
    immediate_watchdog(&state);
    let mut engine = LoopState::bootstrap(&state);
    engine.reconcile_startup(&state);
    let deadline =
        state.repository.load_node_recoveries().unwrap()[&node.id].next_attempt_at_unix_ms;
    drop(engine);
    let mut resumed = LoopState::bootstrap(&state);
    assert_eq!(
        state.repository.load_node_recoveries().unwrap()[&node.id].next_attempt_at_unix_ms,
        deadline
    );
    for spent in 1..=2 {
        due_now(&state, &node.id);
        resumed.run_due_restarts(&state);
        assert_eq!(
            state.repository.load_node_recoveries().unwrap()[&node.id].attempts,
            spent
        );
        resumed = LoopState::bootstrap(&state);
    }
    assert_eq!(
        resumed.watchdog.status(&node.id, Instant::now()),
        WatchdogStatus::Exhausted { attempts: 2 }
    );
    resumed.run_due_restarts(&state);
    assert_eq!(
        state
            .repository
            .list_events_after(0, 100)
            .unwrap()
            .iter()
            .filter(|e| e.kind == EventKind::NodeStartFailed)
            .count(),
        2
    );
}

#[test]
fn interrupted_claim_is_not_reissued_or_refunded() {
    let (_dir, state) = fixture();
    let node = node(&state, "interrupted", NodeStatus::Crashed, None);
    immediate_watchdog(&state);
    state
        .repository
        .schedule_node_recovery(&node, NodeStatus::Crashed, 0)
        .unwrap();
    let first = state
        .repository
        .claim_node_recovery(&node.id, unix_millis())
        .unwrap()
        .unwrap();
    assert!(state
        .repository
        .claim_node_recovery(&node.id, unix_millis())
        .unwrap()
        .is_none());
    assert_eq!(first.attempt, 1);
    let mut resumed = LoopState::bootstrap(&state);
    let recovered = state.repository.load_node_recoveries().unwrap();
    assert_eq!(recovered[&node.id].attempts, 1);
    assert!(recovered[&node.id].claim.is_none());
    due_now(&state, &node.id);
    resumed.run_due_restarts(&state);
    let saved = &state.repository.load_node_recoveries().unwrap()[&node.id];
    assert_eq!(saved.attempts, 2);
    assert!(saved.exhausted);
}

#[test]
fn concurrent_controllers_cannot_claim_the_same_attempt() {
    let (_dir, state) = fixture();
    let node = node(&state, "concurrent-claim", NodeStatus::Crashed, None);
    state
        .repository
        .schedule_node_recovery(&node, NodeStatus::Crashed, 0)
        .unwrap();
    let barrier = std::sync::Barrier::new(2);
    let claims = thread::scope(|scope| {
        let jobs: Vec<_> = (0..2)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    state
                        .repository
                        .claim_node_recovery(&node.id, unix_millis())
                })
            })
            .collect();
        jobs.into_iter()
            .map(|job| job.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(
        claims
            .iter()
            .filter(|result| matches!(result, Ok(Some(_))))
            .count(),
        1
    );
    assert_eq!(
        state.repository.load_node_recoveries().unwrap()[&node.id].attempts,
        1
    );
}

#[test]
fn a_failed_stop_transaction_cannot_publish_a_half_cancelled_state() {
    let (dir, state) = fixture();
    let node = node(&state, "atomic-stop", NodeStatus::Crashed, None);
    state
        .repository
        .schedule_node_recovery(&node, NodeStatus::Crashed, 0)
        .unwrap();
    let connection = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_cancel BEFORE DELETE ON workspace_settings
        WHEN OLD.key GLOB 'watchdog.recovery.*' BEGIN SELECT RAISE(ABORT,'cancel unavailable'); END;").unwrap();
    assert!(stop_node(&state, &node).is_err());
    assert_eq!(state.nodes()[0].status, NodeStatus::Crashed);
    assert!(state.repository.load_node_recoveries().unwrap()[&node.id]
        .next_attempt_at_unix_ms
        .is_some());
}

#[test]
fn stop_atomically_cancels_a_claim_and_stale_failure_cannot_restore_it() {
    let (_dir, state) = fixture();
    let node = node(&state, "stop-claimed", NodeStatus::Crashed, None);
    state
        .repository
        .schedule_node_recovery(&node, NodeStatus::Crashed, 0)
        .unwrap();
    let claim = state
        .repository
        .claim_node_recovery(&node.id, unix_millis())
        .unwrap()
        .unwrap();
    stop_node(&state, &node).unwrap();
    assert!(state.repository.validate_recovery_claim(&claim).is_err());
    assert!(state
        .repository
        .fail_node_recovery(&claim, unix_millis())
        .unwrap()
        .is_none());
    let resumed = LoopState::bootstrap(&state);
    assert!(!resumed.watchdog.has_pending_restart());
    assert!(state.repository.load_node_recoveries().unwrap().is_empty());
    assert_eq!(state.nodes()[0].status, NodeStatus::Stopped);
}

#[test]
fn policy_changes_keep_consumed_budget_and_do_not_resurrect_cancelled_work() {
    let (_dir, state) = fixture();
    let node = node(&state, "policy", NodeStatus::Crashed, None);
    immediate_watchdog(&state);
    state
        .repository
        .schedule_node_recovery(&node, NodeStatus::Crashed, 0)
        .unwrap();
    let claim = state
        .repository
        .claim_node_recovery(&node.id, unix_millis())
        .unwrap()
        .unwrap();
    state
        .repository
        .fail_node_recovery(&claim, unix_millis())
        .unwrap();
    let before = state.repository.load_node_recoveries().unwrap()[&node.id].clone();
    let mut policy = state.repository.load_watchdog_policy().unwrap();
    policy.base_delay = Duration::from_secs(20);
    policy.max_delay = Duration::from_secs(40);
    state.repository.save_watchdog_policy(policy).unwrap();
    assert_eq!(
        state.repository.load_node_recoveries().unwrap()[&node.id],
        before
    );
    policy.enabled = false;
    state.repository.save_watchdog_policy(policy).unwrap();
    policy.enabled = true;
    state.repository.save_watchdog_policy(policy).unwrap();
    let resumed = LoopState::bootstrap(&state);
    assert!(!resumed.watchdog.has_pending_restart());
    assert_eq!(
        state.repository.load_node_recoveries().unwrap()[&node.id].attempts,
        1
    );
    policy.max_restart_attempts = 1;
    state.repository.save_watchdog_policy(policy).unwrap();
    assert!(state.repository.load_node_recoveries().unwrap()[&node.id].exhausted);
}

#[test]
fn claimed_success_keeps_budget_and_manual_start_resets_it() {
    let (_dir, state) = fixture();
    let node = node(&state, "success-budget", NodeStatus::Crashed, None);
    state
        .repository
        .schedule_node_recovery(&node, NodeStatus::Crashed, 0)
        .unwrap();
    state
        .repository
        .claim_node_recovery(&node.id, unix_millis())
        .unwrap()
        .unwrap();
    state
        .repository
        .update_node_status(&node.id, NodeStatus::Running, Some(4_000_000))
        .unwrap();
    let saved = state.repository.load_node_recoveries().unwrap();
    assert_eq!(saved[&node.id].attempts, 1);
    assert!(saved[&node.id].claim.is_none());
    // Successful explicit control has no outstanding automatic claim.
    state
        .repository
        .update_node_status(&node.id, NodeStatus::Running, Some(4_000_001))
        .unwrap();
    assert!(state.repository.load_node_recoveries().unwrap().is_empty());
}

#[test]
fn invalid_or_unwritable_recovery_state_never_authorizes_a_launch() {
    let (dir, state) = fixture();
    let node = node(&state, "invalid-state", NodeStatus::Crashed, None);
    state
        .repository
        .save_workspace_section(&format!("watchdog.recovery.{}", node.id), "broken-state")
        .unwrap();
    let mut engine = LoopState::bootstrap(&state);
    engine.run_due_restarts(&state);
    assert!(!state
        .repository
        .list_events_after(0, 100)
        .unwrap()
        .iter()
        .any(|e| e.kind == EventKind::NodeStartFailed));
    stop_node(&state, &node).unwrap();
    state
        .repository
        .update_node_status(&node.id, NodeStatus::Crashed, None)
        .unwrap();
    let current = state.nodes()[0].clone();
    engine.schedule_restart(&state, &current, "fixture failure");
    due_now(&state, &node.id);
    let connection = rusqlite::Connection::open(dir.path().join("test.db")).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_claim BEFORE UPDATE ON workspace_settings
        WHEN NEW.key GLOB 'watchdog.recovery.*' BEGIN SELECT RAISE(ABORT,'durable state unavailable'); END;").unwrap();
    engine.run_due_restarts(&state);
    assert_eq!(
        state.repository.load_node_recoveries().unwrap()[&node.id].attempts,
        0
    );
    assert!(!state
        .repository
        .list_events_after(0, 100)
        .unwrap()
        .iter()
        .any(|e| e.kind == EventKind::NodeStartFailed));
}
