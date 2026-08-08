use std::path::PathBuf;

use super::*;
use crate::app::domain::{NodeStatus, NodeType, StorageEngine};
use crate::chain_state::RoleDesignation;
use crate::roles::ChainRole;

fn node(name: &str, network: Network) -> NodeConfig {
    NodeConfig {
        id: name.to_string(),
        name: name.to_string(),
        node_type: NodeType::NeoGo,
        network,
        binary_path: PathBuf::from("/opt/neo-go"),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::LevelDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

fn snapshot(committee: usize) -> GovernanceSnapshot {
    GovernanceSnapshot {
        committee: (0..committee).map(|index| format!("key{index}")).collect(),
        next_validators: Vec::new(),
        candidates: Vec::new(),
    }
}

fn key(name: &str, network: Network) -> ChainReadKey {
    ChainReadKey::for_node(&node(name, network), "http://127.0.0.1:10332")
}

/// Governance is chain state, so an answer read through one mainnet node applies
/// to every mainnet node. Discarding it when the operator selects a sibling would
/// throw away a valid read for no reason.
#[test]
fn a_public_answer_is_shared_by_every_node_on_that_network() {
    let mut chain = ChainStateUi::default();
    chain.record_governance(key("a", Network::Mainnet), snapshot(21));

    let sibling = key("b", Network::Mainnet);
    assert_eq!(
        chain.governance_for(&sibling).map(|s| s.committee.len()),
        Some(21),
    );
}

/// The bug this keying exists for: a committee read through a testnet node was
/// shown against a mainnet node, so an operator saw a mismatch that was not real.
#[test]
fn an_answer_is_never_shown_against_another_network() {
    let mut chain = ChainStateUi::default();
    chain.record_governance(key("testnet-node", Network::Testnet), snapshot(21));

    assert!(chain
        .governance_for(&key("mainnet-node", Network::Mainnet))
        .is_none());
    assert!(chain
        .governance_for(&key("private-node", Network::Private))
        .is_none());
}

/// Every private network shares one `Network::Private` value and is not one
/// chain, so two private nodes on different endpoints must not share an answer.
#[test]
fn private_networks_are_told_apart_by_endpoint() {
    let mine = ChainReadKey::for_node(&node("a", Network::Private), "http://127.0.0.1:30332");
    let theirs = ChainReadKey::for_node(&node("b", Network::Private), "http://10.0.0.9:30332");
    assert_ne!(mine, theirs);

    let mut chain = ChainStateUi::default();
    chain.record_governance(mine.clone(), snapshot(4));
    assert!(chain.governance_for(&mine).is_some());
    assert!(chain.governance_for(&theirs).is_none());
}

/// Two private nodes behind the same endpoint are the same chain.
#[test]
fn one_private_endpoint_is_one_chain() {
    let first = ChainReadKey::for_node(&node("a", Network::Private), "http://127.0.0.1:30332");
    let second = ChainReadKey::for_node(&node("b", Network::Private), "http://127.0.0.1:30332");
    assert_eq!(first, second);
}

/// A failed read must drop the answer it failed to refresh. Keeping it drew the
/// error above a snapshot from before the chain went unreachable, with nothing
/// saying the numbers were stale.
#[test]
fn a_failed_read_clears_the_answer_it_could_not_refresh() {
    let mut chain = ChainStateUi::default();
    let mainnet = key("a", Network::Mainnet);
    chain.record_governance(mainnet.clone(), snapshot(21));
    chain.record_governance_error(mainnet.clone(), "connection refused".to_string());

    assert!(chain.governance_for(&mainnet).is_none());
    assert_eq!(
        chain.governance_error_for(&mainnet),
        Some("connection refused"),
    );
}

/// An error reading one chain says nothing about another, and must not appear
/// over a good answer for it.
#[test]
fn an_error_on_one_chain_leaves_another_chains_answer_alone() {
    let mut chain = ChainStateUi::default();
    let mainnet = key("a", Network::Mainnet);
    let testnet = key("b", Network::Testnet);
    chain.record_governance(mainnet.clone(), snapshot(21));
    chain.record_governance_error(testnet.clone(), "timed out".to_string());

    assert_eq!(
        chain.governance_for(&mainnet).map(|s| s.committee.len()),
        Some(21),
        "a testnet failure discarded the mainnet answer",
    );
    assert!(chain.governance_error_for(&mainnet).is_none());
    assert_eq!(chain.governance_error_for(&testnet), Some("timed out"));
}

/// A successful read clears the error it succeeded past.
#[test]
fn a_successful_read_clears_the_previous_error() {
    let mut chain = ChainStateUi::default();
    let mainnet = key("a", Network::Mainnet);
    chain.record_governance_error(mainnet.clone(), "connection refused".to_string());
    chain.record_governance(mainnet.clone(), snapshot(21));

    assert!(chain.governance_error_for(&mainnet).is_none());
    assert!(chain.governance_for(&mainnet).is_some());
}

/// Designation stays keyed by node, not by chain: it is a claim about one node's
/// key, and two nodes on one network do not share it.
#[test]
fn designation_remains_a_per_node_answer() {
    let chain = ChainStateUi {
        designation: Some((
            "node-a".to_string(),
            Ok(RoleDesignation {
                role: ChainRole::StateValidator,
                designated: vec!["key0".to_string()],
                includes_node_key: Some(false),
            }),
        )),
        ..ChainStateUi::default()
    };
    assert!(chain.designation_for("node-a").is_some());
    assert!(chain.designation_for("node-b").is_none());
}
