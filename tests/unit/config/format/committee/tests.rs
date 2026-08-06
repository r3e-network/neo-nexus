use super::{standby_committee, PUBLIC_COMMITTEE_SIZE};
use crate::types::Network;

/// neo-go refuses to start when `StandbyCommittee` is absent, and refuses again
/// when it holds fewer keys than `ValidatorsCount` (7 on both public networks).
/// A dropped key during transcription would produce exactly that failure.
#[test]
fn both_public_networks_carry_a_full_committee() {
    for network in [Network::Mainnet, Network::Testnet] {
        let committee = standby_committee(network);
        assert_eq!(
            committee.len(),
            PUBLIC_COMMITTEE_SIZE,
            "{network} committee lost or gained a key",
        );
    }
}

#[test]
fn every_key_is_a_compressed_secp256r1_point() {
    for network in [Network::Mainnet, Network::Testnet] {
        for key in standby_committee(network) {
            assert_eq!(key.len(), 66, "{key} is not a 33-byte compressed key");
            assert!(
                key.starts_with("02") || key.starts_with("03"),
                "{key} has no compressed-point prefix",
            );
            assert!(
                key.chars().all(|c| c.is_ascii_hexdigit()),
                "{key} is not hex",
            );
        }
    }
}

#[test]
fn committee_keys_are_distinct_within_a_network() {
    for network in [Network::Mainnet, Network::Testnet] {
        let mut keys = standby_committee(network);
        keys.sort();
        let total = keys.len();
        keys.dedup();
        assert_eq!(
            keys.len(),
            total,
            "{network} has a duplicated committee key"
        );
    }
}

#[test]
fn the_two_public_networks_have_different_committees() {
    assert_ne!(
        standby_committee(Network::Mainnet),
        standby_committee(Network::Testnet),
    );
}

/// A private network's committee comes from its own generated profile, so there
/// is no constant to fall back to.
#[test]
fn a_private_network_has_no_standby_committee() {
    assert!(standby_committee(Network::Private).is_empty());
}
