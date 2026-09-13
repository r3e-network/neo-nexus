use super::parse_chain_role;
use crate::core::node_chain::ChainRole;

/// Operators type role names, not the on-chain integers. A mistyped number
/// would silently query a different duty, so the CLI never takes one.
#[test]
fn every_role_is_reachable_by_its_operator_spelling() {
    for (input, expected) in [
        ("state-validator", ChainRole::StateValidator),
        ("StateValidator", ChainRole::StateValidator),
        ("oracle", ChainRole::Oracle),
        ("Oracle", ChainRole::Oracle),
        ("neofs-alphabet", ChainRole::NeoFSAlphabet),
        ("NeoFSAlphabet", ChainRole::NeoFSAlphabet),
        ("p2p-notary", ChainRole::P2PNotary),
        ("P2PNotary", ChainRole::P2PNotary),
    ] {
        assert_eq!(
            parse_chain_role(input).unwrap(),
            expected,
            "{input} should name {expected:?}",
        );
    }
}

#[test]
fn underscores_and_spaces_are_accepted() {
    assert_eq!(
        parse_chain_role("state_validator").unwrap(),
        ChainRole::StateValidator
    );
    assert_eq!(
        parse_chain_role(" p2p notary ").unwrap(),
        ChainRole::P2PNotary
    );
}

/// A bare integer is refused even though it is what the contract takes: a typo
/// there queries a real but different duty and the answer looks plausible.
#[test]
fn a_raw_on_chain_value_is_refused() {
    assert!(parse_chain_role("8").is_err());
    assert!(parse_chain_role("4").is_err());
}

#[test]
fn an_unknown_role_lists_the_ones_that_exist() {
    let error = parse_chain_role("committee").expect_err("committee is not a designated role");
    let message = error.to_string();
    for role in ChainRole::ALL {
        assert!(
            message.contains(role.label()),
            "the error should list {}: {message}",
            role.label(),
        );
    }
}

#[test]
fn chain_family_parsing_supports_n3_and_neox() {
    use super::parse_chain_family;
    use crate::core::node::ChainFamily;

    assert_eq!(parse_chain_family(None).unwrap(), ChainFamily::NeoN3);
    assert_eq!(
        parse_chain_family(Some("neo-n3")).unwrap(),
        ChainFamily::NeoN3
    );
    assert_eq!(
        parse_chain_family(Some("neon3")).unwrap(),
        ChainFamily::NeoN3
    );
    assert_eq!(parse_chain_family(Some("n3")).unwrap(), ChainFamily::NeoN3);
    assert_eq!(
        parse_chain_family(Some("neo-x")).unwrap(),
        ChainFamily::NeoX
    );
    assert_eq!(parse_chain_family(Some("neox")).unwrap(), ChainFamily::NeoX);
    assert_eq!(parse_chain_family(Some("x")).unwrap(), ChainFamily::NeoX);
    assert!(parse_chain_family(Some("invalid")).is_err());
}
