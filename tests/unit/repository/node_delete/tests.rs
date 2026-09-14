use super::*;

/// Only a token that speaks for this node **and nothing else** is revoked.
///
/// A token that also carries a fleet-wide grant was issued for something
/// broader; removing it because one node went away would silently break
/// whatever else holds it.
#[test]
fn only_a_token_confined_to_this_node_is_revoked() {
    assert!(grants_only_this_node("hermes_agent:node-1", "node-1"));
    assert!(grants_only_this_node(
        "hermes_agent:node-1,hermes_agent:node-1",
        "node-1"
    ));

    assert!(!grants_only_this_node("hermes_agent:node-2", "node-1"));
    assert!(!grants_only_this_node(
        "hermes_agent:node-1,read_fleet",
        "node-1"
    ));
    assert!(!grants_only_this_node("admin_all", "node-1"));
}

/// A token whose grants cannot be read is not one to delete on a guess.
#[test]
fn an_unreadable_grant_set_is_not_treated_as_confined() {
    assert!(!grants_only_this_node("", "node-1"));
    assert!(!grants_only_this_node("   ", "node-1"));
    assert!(!grants_only_this_node(",,,", "node-1"));
}

/// A node id that is a prefix of another must not match it. `node-1` and
/// `node-10` are different instances, and revoking the wrong credential is
/// worse than leaving an orphan.
#[test]
fn a_prefix_of_another_node_id_does_not_match() {
    assert!(!grants_only_this_node("hermes_agent:node-10", "node-1"));
    assert!(!grants_only_this_node("hermes_agent:node-1", "node-10"));
}
