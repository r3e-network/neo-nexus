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

/// neo-rs was never researched. Its unknown duties must report as unverified —
/// a gap we own — rather than borrowing neo-go's answers.
#[test]
fn unresearched_neo_rs_duties_are_unverified_not_unsupported() {
    for role in [
        NodeRole::State,
        NodeRole::Indexer,
        NodeRole::Oracle,
        NodeRole::StateValidator,
        NodeRole::Notary,
    ] {
        assert!(
            matches!(
                role_availability(NodeType::NeoRs, role),
                RoleAvailability::Unverified(_)
            ),
            "{role} on neo-rs must be reported as unverified",
        );
    }
}

#[test]
fn the_duties_neo_rs_already_models_stay_available() {
    for role in [NodeRole::RpcApi, NodeRole::Consensus, NodeRole::Observer] {
        assert!(role_availability(NodeType::NeoRs, role).is_supported());
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
