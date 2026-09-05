//! The operation ledger: one mutating operation per node, across processes.
//!
//! The engine and a headless CLI share nothing but this database, so the
//! ledger — not an in-memory lock — is what keeps their operations from
//! interleaving. These tests exercise the SQL semantics directly.

use super::*;

/// The tempdir has to outlive the repository: the connection is opened per
/// call, so a deleted workspace directory fails every later statement.
fn open_repo() -> (tempfile::TempDir, Repository) {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();
    (temp_dir, repository)
}

#[test]
fn a_second_claim_on_the_same_node_is_refused() {
    let (_home, repo) = open_repo();
    let _first = repo
        .begin_node_operation("start", "node-1", 1_000)
        .expect("first claim wins");

    let error = repo
        .begin_node_operation("stop", "node-1", 1_010)
        .expect_err("a running operation excludes a second claim");
    assert!(error
        .to_string()
        .contains("already has a running operation"));
}

#[test]
fn different_nodes_do_not_block_each_other() {
    let (_home, repo) = open_repo();
    let _first = repo
        .begin_node_operation("start", "node-1", 1_000)
        .expect("claim on node-1");
    repo.begin_node_operation("start", "node-2", 1_000)
        .expect("node-2 is independent");
}

#[test]
fn a_finished_operation_frees_the_node() {
    let (_home, repo) = open_repo();
    let claim = repo
        .begin_node_operation("start", "node-1", 1_000)
        .expect("claim");
    repo.finish_node_operation(&claim, 1_050).expect("finish");
    repo.begin_node_operation("stop", "node-1", 1_060)
        .expect("a finished operation no longer blocks");
}

/// The stale window is what keeps a crashed claimant from blocking the node
/// forever: past it, the next claimant retires the dead operation and
/// proceeds.
#[test]
fn a_stale_running_operation_is_retired_by_the_next_claimant() {
    let (_home, repo) = open_repo();
    let _dead = repo
        .begin_node_operation("start", "node-1", 1_000)
        .expect("the claim that will be abandoned");

    // Well past the stale window: the new claimant wins instead of failing.
    repo.begin_node_operation("stop", "node-1", 1_000 + 120 + 5)
        .expect("a stale claim no longer blocks");
}

#[test]
fn a_claim_within_the_stale_window_still_blocks() {
    let (_home, repo) = open_repo();
    let _dead = repo
        .begin_node_operation("start", "node-1", 1_000)
        .expect("the claim");
    let error = repo
        .begin_node_operation("stop", "node-1", 1_000 + 119)
        .expect_err("just inside the stale window the claim still holds");
    assert!(error
        .to_string()
        .contains("already has a running operation"));
}
