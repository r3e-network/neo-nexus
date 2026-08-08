//! Neo X chain identity.
//!
//! Neo X is an EVM sidechain with dBFT finality, so none of the Neo N3 chain
//! constants apply to it: it has no network magic, no seed list of `host:port`
//! pairs, and no standby committee of public keys. It has an EIP-155 chain id,
//! `enode://` bootnodes, and a genesis block whose hash is the only thing that
//! proves a node joined the right chain.
//!
//! Every value here is transcribed from `crates/neox/chainspec/src/lib.rs` in
//! r3e-network/neox-rs, which is also what the Neo X Geth genesis files encode.

use crate::types::Network;

use super::RuntimeConfigProfile;

/// EIP-155 chain id. Neo X MainNet is 47763; TestNet T4 is 12227332.
pub fn neox_chain_id(network: Network, profile: Option<&RuntimeConfigProfile>) -> u64 {
    match network {
        Network::Mainnet => 47_763,
        Network::Testnet => 12_227_332,
        // A private Neo X network has no published id, so the operator's own
        // network number is used. It is the same number the N3 private profile
        // carries, which keeps one private network internally consistent
        // whichever family its nodes belong to.
        Network::Private => profile.map_or(1_230_000, |profile| u64::from(profile.network_magic)),
    }
}

/// The discovery bootnodes a Neo X node dials to find the network.
///
/// A private network has none: its peers are whatever the operator starts, and
/// a fabricated `enode://` would send the node hunting for a host that is not
/// theirs.
pub fn neox_bootnodes(network: Network) -> Vec<String> {
    match network {
        Network::Mainnet => MAINNET_BOOTNODES.iter().map(ToString::to_string).collect(),
        Network::Testnet => TESTNET_BOOTNODES.iter().map(ToString::to_string).collect(),
        Network::Private => Vec::new(),
    }
}

/// The canonical genesis block hash, or `None` for a private network.
///
/// This is the verification anchor for Neo X. NeoNexus never generates a Neo X
/// genesis file — an allocation table it invented would silently produce a
/// chain of one — so instead it records what the real chain's block 0 hashes
/// to, and a node that reports anything else is on the wrong chain.
pub fn neox_genesis_hash(network: Network) -> Option<&'static str> {
    match network {
        Network::Mainnet => {
            Some("0x2ee57478315c7d3182997a812d7885dafee48612cd88cb30b615847b0dd8dbd7")
        }
        Network::Testnet => {
            Some("0x221f7d0a47dd80fe10f476625d62303947c9cd336113e119c64d919f0e9beb71")
        }
        Network::Private => None,
    }
}

/// The `--chain` value neox-rs accepts.
///
/// neox-rs ships both Neo X chain specs compiled in (`SUPPORTED_CHAINS` in
/// `bin/neox-rs/src/main.rs`), so it needs no genesis file. Neo X Geth has no
/// such preset — its network flags are still Ethereum's — which is why a Geth
/// node has to be initialised from the published genesis JSON first.
pub fn neox_reth_chain(network: Network) -> Option<&'static str> {
    match network {
        Network::Mainnet => Some("neox-mainnet"),
        Network::Testnet => Some("neox-testnet"),
        Network::Private => None,
    }
}

/// Seconds per block. Neo X targets 5s, against Neo N3's 15s.
pub fn neox_block_period_secs(network: Network) -> u64 {
    match network {
        Network::Mainnet | Network::Testnet | Network::Private => 5,
    }
}

/// dBFT consensus node count on the public Neo X networks.
pub fn neox_validator_count(network: Network) -> u8 {
    match network {
        Network::Mainnet | Network::Testnet => 7,
        Network::Private => 1,
    }
}

const MAINNET_BOOTNODES: [&str; 2] = [
    "enode://92eec46dd8b67ea8d8999defe0bf2b43d4c4802ed42a430843fec97dafbdc912\
8849261bdf1a940d431fc61f06a1317f5fc7c0386e18a9bbf951d0ccd8bf4f98@34.42.6.58:30303",
    "enode://f289fb5c83ed39cf7d7aff2727afe70bf7951222c4a9aaef7bcbceef9fd0b53e\
4b6c9c0e08a50774dfd50d93e83b977932e4780934d379a6a0ac10cc44c6cfdb@34.87.188.162:30303",
];

const TESTNET_BOOTNODES: [&str; 2] = [
    "enode://60603db58ef8c90ed152531425910b0352e9304f04935d0f2b5ce149a8c70fb7\
a743a39020bb12161e56c17b34d9a6295b378436ac43a09b75bbdc954b48ca5d@34.42.6.58:30304",
    "enode://9d58aaeb46d51ab442cff90613e65e979fbd2084b46b25e46565b289baa007ea\
50e4abfad4e8655873e7f5a1f51b504df217a0d577fffa8278ad2105c0b8cfa9@34.87.188.162:30304",
];

#[cfg(test)]
#[path = "../../../tests/unit/config/format/neox/tests.rs"]
mod tests;
