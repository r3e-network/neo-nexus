use std::path::PathBuf;

use crate::{
    observe::{Evidence, NodeSample, NotSampled, Observation},
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

/// A round that answered, with a deliberate mix: one value present, one value
/// present *and zero*, and one value never read.
fn answered(node_id: &str, at: u64, height: u64) -> NodeSample {
    let evidence = |field: &'static str, value: String| {
        Evidence::recorded("getblockcount", field, value, "http://127.0.0.1:30332", at)
    };
    let mut sample = NodeSample::unreachable(
        node_id,
        at,
        "http://127.0.0.1:30332",
        NotSampled::CallFailed {
            method: "getblockcount",
            detail: "connection refused".to_string(),
        },
    );
    sample.head_ok = true;
    sample.head_latency_ms = Some(0);
    sample.block_height = Observation::Known(height, evidence("block_height", height.to_string()));
    sample.peers_connected = Observation::Known(0, evidence("peers_connected", "0".to_string()));
    sample
}

/// The reason this table exists. The console it replaces printed `0` where it
/// had no number, and an operator cannot tell a node with no peers — which is
/// an incident — from a client that was never asked for its peer count.
///
/// Zero has to survive as zero and absence has to survive as absence across a
/// write and a read, or the type that makes that distinction in memory is
/// undone the moment anything is saved.
#[test]
fn a_zero_stays_a_zero_and_an_absence_stays_an_absence() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    repository
        .record_node_sample(&answered(&id, 1_770_000_000, 8_421))
        .unwrap();
    let stored = repository.latest_node_sample(&id).unwrap().unwrap();

    assert_eq!(stored.peers_connected.value().copied(), Some(0));
    assert_eq!(stored.block_height.value().copied(), Some(8_421));

    // The mempool was never read this round, and must not read as an empty one.
    assert!(!stored.mempool_verified.is_known());
    assert_ne!(
        stored.mempool_verified.render(|depth| depth.to_string()),
        "0"
    );

    // A latency that was measured as zero is not a latency that was not
    // measured; `Option<u32>` carries that and the column must preserve it.
    assert_eq!(stored.head_latency_ms, Some(0));
}

/// Which absence it is survives too, reconstructed from the row rather than
/// guessed. After a restart there is no live round, so the stored row is the
/// whole of what an operator has: a node with no RPC port must not read the
/// same as a node that is down.
#[test]
fn the_three_kinds_of_absence_read_back_as_three_different_sentences() {
    let (_dir, repository) = workspace();
    let disabled = node(&repository, "no-rpc", 0);
    let down = node(&repository, "down", 30432);
    let partial = node(&repository, "partial", 30532);

    repository
        .record_node_sample(&NodeSample::not_observable(&disabled, 1_770_000_000))
        .unwrap();
    repository
        .record_node_sample(&NodeSample::unreachable(
            &down,
            1_770_000_000,
            "http://127.0.0.1:30432",
            NotSampled::CallFailed {
                method: "getblockcount",
                detail: "connection refused".to_string(),
            },
        ))
        .unwrap();
    repository
        .record_node_sample(&answered(&partial, 1_770_000_000, 12))
        .unwrap();

    let read = |id: &str| {
        repository
            .latest_node_sample(id)
            .unwrap()
            .unwrap()
            .mempool_verified
            .render(|depth| depth.to_string())
    };

    let sentences = [read(&disabled), read(&down), read(&partial)];
    assert!(sentences[0].contains("RPC is disabled"));
    assert!(sentences[1].contains("did not answer"));
    assert!(sentences[2].contains("not recorded"));

    for sentence in &sentences {
        assert!(
            sentence.parse::<u64>().is_err(),
            "{sentence} must not read as a measurement"
        );
    }
    assert_eq!(
        sentences
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        3,
        "each kind of absence must be distinguishable: {sentences:?}"
    );
}

/// Derivations walk backwards from the head — "has the height moved in the last
/// five minutes" — so the newest round has to come first without the caller
/// sorting anything.
#[test]
fn rounds_come_back_newest_first() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    for (at, height) in [(1_770_000_030, 3), (1_770_000_000, 1), (1_770_000_015, 2)] {
        repository
            .record_node_sample(&answered(&id, at, height))
            .unwrap();
    }

    let heights: Vec<u64> = repository
        .recent_node_samples(&id, 10)
        .unwrap()
        .iter()
        .filter_map(|sample| sample.block_height.value().copied())
        .collect();
    assert_eq!(heights, vec![3, 2, 1]);
}

/// Two rounds can land in the same second on a fast local node. Ordering by
/// time alone would then make "the latest sample" whichever row SQLite happened
/// to return, so the insertion order breaks the tie.
#[test]
fn rounds_in_the_same_second_keep_the_order_they_were_written() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);

    for height in [10, 11, 12] {
        repository
            .record_node_sample(&answered(&id, 1_770_000_000, height))
            .unwrap();
    }

    let latest = repository.latest_node_sample(&id).unwrap().unwrap();
    assert_eq!(latest.block_height.value().copied(), Some(12));
}

/// Pruning is per node. A single global cap would let one node sampled every
/// fifteen seconds evict the history of a node sampled every two minutes — and
/// the quiet node is exactly the one whose last known height gets looked up
/// after an outage.
#[test]
fn pruning_keeps_each_nodes_own_history() {
    let (_dir, repository) = workspace();
    let chatty = node(&repository, "chatty", 30332);
    let quiet = node(&repository, "quiet", 30432);

    for round in 0..10u64 {
        repository
            .record_node_sample(&answered(&chatty, 1_770_000_000 + round, round))
            .unwrap();
    }
    repository
        .record_node_sample(&answered(&quiet, 1_770_000_000, 500))
        .unwrap();

    let removed = repository
        .prune_node_samples_keep_recent_per_node(3)
        .unwrap();
    assert_eq!(removed, 7);

    let kept: Vec<u64> = repository
        .recent_node_samples(&chatty, 10)
        .unwrap()
        .iter()
        .filter_map(|sample| sample.block_height.value().copied())
        .collect();
    assert_eq!(kept, vec![9, 8, 7], "pruning must keep the newest rounds");

    assert_eq!(
        repository.recent_node_samples(&quiet, 10).unwrap().len(),
        1,
        "a quiet node must keep its history when a chatty one is pruned"
    );
}

/// A node that has never been sampled is not a node reporting nothing. The
/// caller has to be able to tell those apart to render "not checked yet".
#[test]
fn a_node_that_was_never_sampled_has_no_latest_round() {
    let (_dir, repository) = workspace();
    let id = node(&repository, "rpc-1", 30332);
    assert!(repository.latest_node_sample(&id).unwrap().is_none());
    assert!(repository.recent_node_samples(&id, 10).unwrap().is_empty());
}

/// Node ids come from URLs and API payloads, so the same validation every other
/// node-scoped table applies guards this one — a read for one node must not be
/// expressible as a read across all of them.
#[test]
fn a_malformed_node_id_is_refused_rather_than_queried() {
    let (_dir, repository) = workspace();
    assert!(repository.recent_node_samples("' OR 1=1 --", 10).is_err());
    assert!(repository
        .record_node_sample(&NodeSample::not_observable("' OR 1=1 --", 1_770_000_000))
        .is_err());
}
