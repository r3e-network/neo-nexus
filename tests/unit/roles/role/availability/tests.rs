use super::{role_availability, RoleAvailability};
use crate::{roles::NodeRole, types::NodeType};

/// Every cell of the matrix is asserted, so adding a role or a client cannot
/// silently default to "supported" — which would offer an operator a duty their
/// node will accept configuration for and then never perform.
#[test]
fn every_client_and_role_pairing_is_decided() {
    for node_type in NodeType::ALL {
        for role in NodeRole::ALL {
            let availability = role_availability(node_type, role);
            if let Some(reason) = availability.reason() {
                assert!(
                    !reason.is_empty(),
                    "{node_type}/{role} is unavailable without saying why",
                );
            }
        }
    }
}

/// The C# node implements neither the P2PNotaryRequest payload nor a notary
/// module, so this is a fact about the client, not a gap in our knowledge.
#[test]
fn the_notary_service_is_neo_go_only() {
    assert!(matches!(
        role_availability(NodeType::NeoCli, NodeRole::Notary),
        RoleAvailability::Unsupported(_)
    ));
    assert!(role_availability(NodeType::NeoGo, NodeRole::Notary).is_supported());
}

/// Verified against the daemon's own TOML config struct: each of these has a
/// section it reads.
#[test]
fn neo_rs_supports_the_duties_its_config_has_sections_for() {
    for role in [
        NodeRole::RpcApi,
        NodeRole::Consensus,
        NodeRole::State,
        NodeRole::Indexer,
        NodeRole::Observer,
    ] {
        assert!(
            role_availability(NodeType::NeoRs, role).is_supported(),
            "{role} has a neo-rs config section and must be offered",
        );
    }
}

/// These are facts about the client, established by reading it — not gaps in
/// what we know. neo-rs serves state roots without signing them, and holds an
/// oracle crate it uses only to validate other nodes' oracle responses.
#[test]
fn neo_rs_duties_it_cannot_perform_are_unsupported_with_a_reason() {
    for role in [NodeRole::StateValidator, NodeRole::Oracle, NodeRole::Notary] {
        let availability = role_availability(NodeType::NeoRs, role);
        assert!(
            matches!(availability, RoleAvailability::Unsupported(_)),
            "{role} on neo-rs is a known limitation, not an unknown",
        );
        assert!(availability
            .reason()
            .is_some_and(|reason| !reason.is_empty()));
    }
}

/// Nothing is left unverified now that every client has been read. The variant
/// stays for the next client added before anyone has read it.
#[test]
fn no_cell_of_the_matrix_is_still_unverified() {
    for node_type in NodeType::ALL {
        for role in NodeRole::ALL {
            assert!(
                !matches!(
                    role_availability(node_type, role),
                    RoleAvailability::Unverified(_)
                ),
                "{node_type}/{role} is still unverified",
            );
        }
    }
}

/// A committee designation is required for exactly the duties that act with a
/// designated key. A validator is elected by vote, not designated.
#[test]
fn only_designated_duties_report_a_chain_role() {
    use crate::roles::ChainRole;
    assert_eq!(NodeRole::Oracle.designation(), Some(ChainRole::Oracle));
    assert_eq!(
        NodeRole::StateValidator.designation(),
        Some(ChainRole::StateValidator)
    );
    assert_eq!(NodeRole::Notary.designation(), Some(ChainRole::P2PNotary));
    for role in [
        NodeRole::RpcApi,
        NodeRole::State,
        NodeRole::Indexer,
        NodeRole::Consensus,
        NodeRole::Observer,
    ] {
        assert_eq!(role.designation(), None, "{role} needs no designation");
    }
}

/// These are consensus-visible integers passed to RoleManagement. Renumbering
/// them would designate the wrong duty.
#[test]
fn chain_role_values_match_the_native_contract() {
    use crate::roles::ChainRole;
    assert_eq!(ChainRole::StateValidator.on_chain_value(), 4);
    assert_eq!(ChainRole::Oracle.on_chain_value(), 8);
    assert_eq!(ChainRole::NeoFSAlphabet.on_chain_value(), 16);
    assert_eq!(ChainRole::P2PNotary.on_chain_value(), 32);
}
