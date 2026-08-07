use super::services_for;
use crate::{
    config::format::{GenerationContext, ServiceWallet},
    roles::NodeRole,
};

fn wallet() -> ServiceWallet {
    ServiceWallet {
        path: "/opt/neo/wallets/node.json".to_string(),
        password: "hunter2".to_string(),
    }
}

fn signed(role: NodeRole) -> GenerationContext {
    GenerationContext::for_role(role).with_wallet(wallet())
}

/// A node with no assigned duty gets a plain relaying config: no service
/// section at all, rather than four disabled ones.
#[test]
fn a_node_without_a_duty_carries_no_service_sections() {
    let services = services_for(&GenerationContext::default());
    assert!(services.consensus.is_none());
    assert!(services.oracle.is_none());
    assert!(services.state_root.is_none());
    assert!(services.p2p_notary.is_none());
}

/// Each duty switches on exactly one section — selecting Consensus must not
/// also enable the oracle.
#[test]
fn each_duty_switches_on_only_its_own_section() {
    let consensus = services_for(&signed(NodeRole::Consensus));
    assert!(consensus.consensus.is_some());
    assert!(consensus.oracle.is_none() && consensus.state_root.is_none());

    let oracle = services_for(&signed(NodeRole::Oracle));
    assert!(oracle.oracle.is_some());
    assert!(oracle.consensus.is_none() && oracle.p2p_notary.is_none());

    let state = services_for(&signed(NodeRole::StateValidator));
    assert!(state.state_root.is_some());
    assert!(state.consensus.is_none() && state.oracle.is_none());

    let notary = services_for(&signed(NodeRole::Notary));
    assert!(notary.p2p_notary.is_some());
    assert!(notary.consensus.is_none() && notary.oracle.is_none());
}

/// The read-only duties are expressed by the base configuration, not by a
/// service section.
#[test]
fn local_duties_need_no_service_section() {
    for role in [
        NodeRole::RpcApi,
        NodeRole::State,
        NodeRole::Indexer,
        NodeRole::Observer,
    ] {
        let services = services_for(&signed(role));
        assert!(
            services.consensus.is_none()
                && services.oracle.is_none()
                && services.state_root.is_none()
                && services.p2p_notary.is_none(),
            "{role} should not switch on a signing service",
        );
    }
}

/// neo-go reads `UnlockWallet` as a value, not a pointer, so an enabled service
/// with an empty wallet path fails at startup. Without a wallet the section is
/// written disabled — recording the intent without producing a file that will
/// not boot.
#[test]
fn a_signing_service_stays_disabled_until_a_wallet_is_supplied() {
    let services = services_for(&GenerationContext::for_role(NodeRole::Consensus));
    let consensus = services.consensus.expect("section is still written");
    assert!(!consensus.enabled);
    assert!(consensus.unlock_wallet.is_none());
}

#[test]
fn a_supplied_wallet_enables_the_service_and_is_written_through() {
    let services = services_for(&signed(NodeRole::Consensus));
    let consensus = services.consensus.expect("consensus section");
    assert!(consensus.enabled);
    let unlocked = consensus.unlock_wallet.expect("wallet is written");
    assert_eq!(unlocked.path, wallet().path);
    assert_eq!(unlocked.password, wallet().password);
}

#[test]
fn the_oracle_follows_the_same_wallet_rule() {
    let without = services_for(&GenerationContext::for_role(NodeRole::Oracle));
    let oracle = without.oracle.expect("oracle section");
    assert!(!oracle.enabled);
    assert!(oracle.unlock_wallet.is_none());

    let with = services_for(&signed(NodeRole::Oracle));
    assert!(with.oracle.expect("oracle section").enabled);
}

/// An oracle that accepts no content type answers nothing.
#[test]
fn the_oracle_accepts_at_least_one_content_type() {
    let services = services_for(&signed(NodeRole::Oracle));
    let oracle = services.oracle.expect("oracle section");
    assert!(!oracle.allowed_content_types.is_empty());
    assert!(!oracle.allow_private_host);
}
