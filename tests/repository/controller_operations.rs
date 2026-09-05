use super::*;

fn open_repo() -> (tempfile::TempDir, Repository) {
    let home = tempfile::tempdir().unwrap();
    let repository = Repository::open(home.path().join("neonexus.db")).unwrap();
    (home, repository)
}

#[test]
fn controller_claim_carries_generation_and_random_fencing_token() {
    let (_home, repo) = open_repo();
    let operation = repo
        .begin_controller_operation("node", "node-1", "start", "running", 1_000)
        .expect("controller claim");
    assert_eq!(operation.subject_kind, "node");
    assert_eq!(operation.subject_id, "node-1");
    assert_eq!(operation.phase, "reserved");
    assert_eq!(operation.desired_state.as_deref(), Some("running"));
    assert_eq!(operation.generation, 1);
    assert!(!operation.fencing_token.is_empty());
}

#[test]
fn stale_reclaim_fences_the_old_controller() {
    let (_home, repo) = open_repo();
    let old = repo
        .begin_controller_operation("node", "node-1", "start", "running", 1_000)
        .expect("old claim");
    let new = repo
        .begin_controller_operation("node", "node-1", "recover", "running", 1_000 + 121)
        .expect("stale claim is reclaimed");
    assert_eq!(new.generation, old.generation + 1);
    assert_ne!(new.fencing_token, old.fencing_token);
    assert!(!repo
        .renew_controller_operation(&old, "spawned", 1_122)
        .expect("old renewal is a normal CAS miss"));
    assert!(repo
        .renew_controller_operation(&new, "spawned", 1_122)
        .expect("new owner renews"));
}

#[test]
fn old_token_cannot_commit_or_fail_a_new_generation() {
    let (_home, repo) = open_repo();
    let old = repo
        .begin_controller_operation("agent", "agent-1", "start", "running", 10)
        .expect("old claim");
    let new = repo
        .begin_controller_operation("agent", "agent-1", "recover", "running", 10 + 121)
        .expect("new claim");
    assert!(!repo
        .complete_controller_operation(&old, 200)
        .expect("old completion is a normal CAS miss"));
    assert!(!repo
        .fail_controller_operation(&old, "late", 201)
        .expect("old failure is a normal CAS miss"));
    assert!(repo
        .record_spawned_process(&new, 1234, Some(999), 202)
        .expect("new owner records its process"));
    assert!(repo
        .complete_controller_operation(&new, 203)
        .expect("new owner commits"));
}

#[test]
fn concurrent_connections_allow_only_one_active_claim() {
    let (home, repo) = open_repo();
    let left = repo.clone();
    let right = repo.clone();
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let left_barrier = barrier.clone();
    let right_barrier = barrier;
    let left_thread = std::thread::spawn(move || {
        left_barrier.wait();
        left.begin_controller_operation("node", "node-1", "start", "running", 2_000)
    });
    let right_thread = std::thread::spawn(move || {
        right_barrier.wait();
        right.begin_controller_operation("node", "node-1", "stop", "stopped", 2_000)
    });
    let outcomes = [
        left_thread.join().expect("left claimant thread"),
        right_thread.join().expect("right claimant thread"),
    ];
    assert_eq!(outcomes.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        repo.controller_reconcile_summary(2_000)
            .expect("summary")
            .pending,
        1
    );
    drop(home);
}

#[test]
fn a_reused_pid_cannot_be_written_by_a_fenced_claim() {
    let (_home, repo) = open_repo();
    let old = repo
        .begin_controller_operation("node", "node-reused", "start", "running", 1_000)
        .expect("old claim");
    // The old owner spawns and records a PID.
    repo.record_spawned_process(&old, 123, Some(1_010), 1_010)
        .expect("old spawn");
    // Then the manager dies and a fresh controller reclaims after the window.
    let fresh = repo
        .begin_controller_operation("node", "node-reused", "start", "running", 1_131)
        .expect("reclaim");
    assert_ne!(fresh.generation, old.generation);
    // The old token must not record the replacement PID.
    assert!(!repo
        .record_spawned_process(&old, 456, Some(1_200), 1_200)
        .expect("old spawn write is refused"));
}

#[test]
fn startup_reconcile_fences_a_stale_reserved_operation() {
    let (_home, repo) = open_repo();
    // A controller was killed after reserving but before spawning a process.
    let _stale = repo
        .begin_controller_operation("node", "node-orphan", "start", "running", 1_000)
        .expect("reserved operation");
    // A fresh controller starts after the lease window.
    let summary = repo
        .reconcile_pending_controller_operations(1_000 + 121)
        .expect("startup reconcile");
    assert_eq!(summary.pending, 0, "stale reserved operation is fenced");
    // The orphan no longer blocks a fresh start claim.
    let fresh = repo
        .begin_controller_operation("node", "node-orphan", "start", "running", 1_130)
        .expect("fresh start claim succeeds after reconcile");
    assert_eq!(fresh.generation, 2);
}

#[test]
fn reconcile_reclaims_a_spawned_orphan_after_the_window() {
    let (_home, repo) = open_repo();
    let orphan = repo
        .begin_controller_operation("node", "node-spawned", "start", "running", 1_000)
        .expect("claim");
    repo.record_spawned_process(&orphan, 777, Some(1_010), 1_010)
        .expect("spawned orphan");
    // The spawn advanced updated_at to 1_010; reconcile must treat it stale
    // only once it is 120s behind the reconcile time.
    let reclaimed = repo
        .reconcile_pending_controller_operations(1_130)
        .expect("reconcile");
    assert_eq!(reclaimed.pending, 0);
}

#[test]
fn spawned_phase_requires_the_reserved_owner_and_is_fenced() {
    let (_home, repo) = open_repo();
    let operation = repo
        .begin_controller_operation("node", "node-1", "start", "running", 10)
        .expect("claim");
    assert!(repo
        .record_spawned_process(&operation, 4321, Some(77), 11)
        .expect("reserved owner records spawn"));
    assert!(!repo
        .record_spawned_process(&operation, 4322, Some(78), 12)
        .expect("a second spawned transition is rejected"));
}
