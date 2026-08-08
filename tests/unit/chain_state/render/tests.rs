use crate::chain_state::{CandidateStanding, GovernanceSnapshot, RoleDesignation};
use crate::roles::ChainRole;

fn designation(includes: Option<bool>) -> RoleDesignation {
    RoleDesignation {
        role: ChainRole::Oracle,
        designated: vec!["03aa".to_string(), "02bb".to_string()],
        includes_node_key: includes,
    }
}

/// A script or an operator reading this must never take "we could not check
/// this node's key" for "this node is not designated".
#[test]
fn an_unchecked_node_is_reported_as_unknown_not_as_undesignated() {
    let text = designation(None).to_cli_text();
    assert!(text.contains("node-designated: unknown"), "{text}");
    assert!(designation(Some(false))
        .to_cli_text()
        .contains("node-designated: no"));
    assert!(designation(Some(true))
        .to_cli_text()
        .contains("node-designated: yes"));
}

#[test]
fn the_report_names_the_role_and_its_on_chain_value() {
    let text = designation(Some(true)).to_cli_text();
    assert!(text.contains("chain-role: Oracle"));
    assert!(text.contains("on-chain-value: 8"));
    assert!(text.contains("designated-keys: 2"));
    assert!(text.contains("key: 03aa"));
}

fn snapshot(candidates: usize) -> GovernanceSnapshot {
    GovernanceSnapshot {
        committee: vec!["03aa".to_string(), "02bb".to_string()],
        next_validators: vec!["03aa".to_string()],
        candidates: (0..candidates)
            .map(|index| CandidateStanding {
                public_key: format!("03{index:02}"),
                votes: (candidates - index) as i64,
            })
            .collect(),
    }
}

/// Committee membership and block production are different: 21 versus 7.
#[test]
fn a_committee_member_producing_blocks_is_marked() {
    let text = snapshot(2).to_cli_text();
    assert!(text.contains("committee-member: 03aa validator"), "{text}");
    assert!(text.contains("committee-member: 02bb\n") || text.ends_with("committee-member: 02bb"));
}

/// The vote has a long tail. Truncating silently would misreport the field, so
/// the count that was left out is stated.
#[test]
fn a_truncated_candidate_list_says_how_many_were_omitted() {
    let text = snapshot(25).to_cli_text();
    assert!(text.contains("candidates: 25"));
    assert!(text.contains("candidates-omitted: 15"), "{text}");
}

#[test]
fn a_short_candidate_list_is_not_marked_as_truncated() {
    let text = snapshot(3).to_cli_text();
    assert!(!text.contains("candidates-omitted"));
}
