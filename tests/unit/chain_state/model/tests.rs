use super::{CandidateStanding, ChainQueryError, GovernanceSnapshot, RoleDesignation};
use crate::roles::ChainRole;

fn designation(includes: Option<bool>) -> RoleDesignation {
    RoleDesignation {
        role: ChainRole::Oracle,
        designated: vec!["03aa".to_string(), "02bb".to_string()],
        includes_node_key: includes,
    }
}

/// "Not designated" and "we do not know this node's key" are different facts.
/// Collapsing them is how a manager ends up asserting something about a node it
/// has no basis for.
#[test]
fn an_unknown_node_key_is_not_reported_as_undesignated() {
    assert!(!designation(None).is_designated());
    let summary = designation(None).summary();
    assert!(summary.contains("no key to compare"), "{summary}");
    assert!(!designation(Some(false)).is_designated());
    assert!(designation(Some(false))
        .summary()
        .contains("not designated"));
    assert!(designation(Some(true)).is_designated());
}

#[test]
fn a_summary_always_reports_how_many_keys_hold_the_role() {
    for includes in [None, Some(false), Some(true)] {
        assert!(
            designation(includes).summary().contains('2'),
            "summary must say how many keys hold the role",
        );
    }
}

/// Unreachable and unexpected call for different operator responses: check the
/// node versus check the version. They stay distinct.
#[test]
fn query_failures_keep_their_kind() {
    let unreachable = ChainQueryError::Unreachable("connection refused".to_string());
    let unexpected = ChainQueryError::Unexpected("no result".to_string());
    assert_ne!(unreachable, unexpected);
    assert_eq!(unreachable.message(), "connection refused");
    assert_eq!(unexpected.message(), "no result");
}

fn snapshot() -> GovernanceSnapshot {
    GovernanceSnapshot {
        committee: vec!["03aa".to_string(), "02bb".to_string()],
        next_validators: vec!["03aa".to_string()],
        candidates: vec![
            CandidateStanding {
                public_key: "03aa".to_string(),
                votes: 5_000,
            },
            CandidateStanding {
                public_key: "02cc".to_string(),
                votes: 10,
            },
        ],
    }
}

/// Sitting on the committee and producing blocks are different things: the
/// committee is 21, the validators are 7 of them.
#[test]
fn committee_membership_and_block_production_are_distinguished() {
    let snapshot = snapshot();
    assert!(snapshot.is_committee_member("03aa"));
    assert!(snapshot.is_validator("03aa"));
    assert!(snapshot.is_committee_member("02bb"));
    assert!(!snapshot.is_validator("02bb"));
    assert!(!snapshot.is_committee_member("02cc"));
}

#[test]
fn a_key_can_be_looked_up_in_the_candidate_vote() {
    let snapshot = snapshot();
    assert_eq!(snapshot.candidate_standing("03aa").unwrap().votes, 5_000);
    assert!(snapshot.candidate_standing("039999").is_none());
}
