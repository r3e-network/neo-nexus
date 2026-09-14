use super::*;

use crate::roles::role_availability;

/// **The dead end this gate exists to close.**
///
/// `role_availability(NeoXGeth, Consensus)` says supported — correctly, both
/// Neo X clients ship dBFT block production — and `/roles` let an operator
/// apply it. Launch then refused every way it could be reached: no key is a
/// signing duty without one; a local wallet is Neo N3 NEP-6 material a Neo X
/// client cannot consume; the local signer and the remote service are
/// neo-cli-consensus only. No `--validator` or `--miner` flag is emitted
/// anywhere either.
#[test]
fn neox_consensus_is_supported_by_the_client_and_not_by_this_product() {
    for node_type in [NodeType::NeoXGeth, NodeType::NeoXReth] {
        assert!(
            role_availability(node_type, NodeRole::Consensus).is_supported(),
            "{node_type} does ship dBFT block production"
        );
        let launch = launch_support(node_type, NodeRole::Consensus);
        assert!(
            !launch.is_launchable(),
            "{node_type} consensus must not be offered as launchable"
        );
        assert!(
            launch
                .reason()
                .is_some_and(|reason| reason.contains("secp256k1")),
            "the reason has to name the actual obstacle: {launch:?}"
        );
    }
}

/// neo-rs takes its consensus key as plaintext hex in its config, which this
/// workspace will not write.
#[test]
fn neo_rs_consensus_is_refused_for_a_reason_that_names_the_obstacle() {
    let launch = launch_support(NodeType::NeoRs, NodeRole::Consensus);
    assert!(!launch.is_launchable());
    assert!(launch
        .reason()
        .is_some_and(|reason| reason.contains("plaintext")));
}

/// The two Neo N3 clients that take a wallet can perform every signing duty
/// they support, so the gate must not narrow them.
#[test]
fn the_clients_that_take_a_wallet_stay_launchable() {
    for node_type in [NodeType::NeoGo, NodeType::NeoCli] {
        for role in NodeRole::ALL {
            assert!(
                launch_support(node_type, role).is_launchable(),
                "{node_type} {role:?} must stay launchable"
            );
        }
    }
}

/// A duty that signs nothing is configuration only — a plugin or a service
/// section — which every client path can write. Narrowing those would remove
/// working capability to fix a signing problem.
#[test]
fn a_duty_that_does_not_sign_is_launchable_everywhere() {
    for node_type in NodeType::ALL {
        for role in NodeRole::ALL.iter().filter(|role| !role.requires_signer()) {
            assert!(
                launch_support(node_type, *role).is_launchable(),
                "{node_type} {role:?} signs nothing and must stay launchable"
            );
        }
    }
}

/// The gate is narrower than the client matrix, never wider: it may refuse a
/// duty the client supports, but it must never offer one the client cannot do.
#[test]
fn the_gate_never_offers_more_than_the_client_supports() {
    for node_type in NodeType::ALL {
        for role in NodeRole::ALL {
            if !role_availability(node_type, role).is_supported() {
                // Whatever this gate says, the picker checks the client matrix
                // first — so the pair can only ever narrow.
                continue;
            }
            let launch = launch_support(node_type, role);
            assert!(
                launch.is_launchable() || launch.reason().is_some(),
                "{node_type} {role:?} is refused with no reason to show the operator"
            );
        }
    }
}
