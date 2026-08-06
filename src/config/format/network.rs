use crate::types::Network;

use super::RuntimeConfigProfile;

pub(super) fn network_magic(network: Network) -> u32 {
    match network {
        Network::Mainnet => 860_833_102,
        Network::Testnet => 894_710_606,
        Network::Private => 1_230_000,
    }
}

pub(in crate::config) fn effective_network_magic(
    network: Network,
    profile: Option<&RuntimeConfigProfile>,
) -> u32 {
    profile.map_or_else(|| network_magic(network), |profile| profile.network_magic)
}

pub(in crate::config) fn effective_seed_nodes(
    network: Network,
    profile: Option<&RuntimeConfigProfile>,
) -> Vec<String> {
    profile.map_or_else(|| seed_nodes(network), |profile| profile.seed_nodes.clone())
}

pub(in crate::config) fn effective_validators_count(
    network: Network,
    profile: Option<&RuntimeConfigProfile>,
) -> u8 {
    profile.map_or_else(
        || validators_count(network),
        |profile| profile.validators_count,
    )
}

/// The committee a generated config should carry: the private network's own
/// keys when a profile supplies them, otherwise the public network's standby
/// committee. It must never be empty for mainnet or testnet — neo-go treats an
/// absent or short committee as a fatal configuration error.
pub(in crate::config) fn effective_committee_public_keys(
    network: Network,
    profile: Option<&RuntimeConfigProfile>,
) -> Vec<String> {
    match profile {
        Some(profile) if !profile.committee_public_keys.is_empty() => {
            profile.committee_public_keys.clone()
        }
        _ => super::committee::standby_committee(network),
    }
}

pub(super) fn seed_nodes(network: Network) -> Vec<String> {
    match network {
        Network::Mainnet => [
            "seed1.neo.org:10333",
            "seed2.neo.org:10333",
            "seed3.neo.org:10333",
            "seed4.neo.org:10333",
            "seed5.neo.org:10333",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect(),
        Network::Testnet => [
            "seed1t5.neo.org:20333",
            "seed2t5.neo.org:20333",
            "seed3t5.neo.org:20333",
            "seed4t5.neo.org:20333",
            "seed5t5.neo.org:20333",
        ]
        .into_iter()
        .map(ToString::to_string)
        .collect(),
        Network::Private => Vec::new(),
    }
}

pub(in crate::config) fn broadcast_history_limit(network: Network) -> usize {
    match network {
        Network::Mainnet => 100_000,
        Network::Testnet | Network::Private => 50_000,
    }
}

pub(in crate::config) fn max_transactions_per_block(network: Network) -> u32 {
    match network {
        Network::Mainnet => 200,
        Network::Testnet | Network::Private => 5_000,
    }
}

pub(super) fn validators_count(network: Network) -> u8 {
    match network {
        Network::Mainnet | Network::Testnet => 7,
        Network::Private => 1,
    }
}
