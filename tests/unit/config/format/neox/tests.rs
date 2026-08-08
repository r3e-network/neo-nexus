use super::*;

/// The published EIP-155 ids. A wrong chain id makes every signed transaction
/// replayable on another chain, so these are transcribed, never derived.
#[test]
fn the_public_chain_ids_are_the_published_ones() {
    assert_eq!(neox_chain_id(Network::Mainnet, None), 47_763);
    assert_eq!(neox_chain_id(Network::Testnet, None), 12_227_332);
}

/// Neo X chain ids must never collide with the Neo N3 network magics, or an
/// operator reading "47763" somewhere would have no way to tell which chain a
/// node is on.
#[test]
fn neox_chain_ids_are_distinct_from_the_n3_magics() {
    let n3 = [860_833_102_u64, 894_710_606];
    for network in [Network::Mainnet, Network::Testnet] {
        assert!(!n3.contains(&neox_chain_id(network, None)));
    }
}

#[test]
fn every_public_bootnode_is_a_complete_enode_url() {
    for network in [Network::Mainnet, Network::Testnet] {
        let bootnodes = neox_bootnodes(network);
        assert_eq!(bootnodes.len(), 2, "{network} should have two bootnodes");
        for bootnode in bootnodes {
            assert!(bootnode.starts_with("enode://"), "{bootnode}");
            let (id, endpoint) = bootnode
                .trim_start_matches("enode://")
                .split_once('@')
                .expect("an enode URL carries node-id@host:port");
            // A secp256k1 public key without its prefix byte: 64 bytes of hex.
            assert_eq!(id.len(), 128, "node id in {bootnode}");
            assert!(id.chars().all(|c| c.is_ascii_hexdigit()), "{bootnode}");
            assert!(endpoint.contains(':'), "{bootnode}");
        }
    }
}

/// A line continuation that dropped a character would still look like a valid
/// enode URL, so the mainnet ids are pinned exactly.
#[test]
fn the_mainnet_bootnode_endpoints_are_pinned() {
    let bootnodes = neox_bootnodes(Network::Mainnet);
    assert!(bootnodes[0].ends_with("@34.42.6.58:30303"));
    assert!(bootnodes[1].ends_with("@34.87.188.162:30303"));
    assert!(bootnodes[0].starts_with("enode://92eec46dd8b67ea8"));
    assert!(bootnodes[1].starts_with("enode://f289fb5c83ed39cf"));
}

/// Testnet peers listen on 30304, not 30303: dialling the mainnet port would
/// silently sync a testnet node against mainnet peers that reject it.
#[test]
fn the_testnet_bootnodes_use_their_own_port() {
    for bootnode in neox_bootnodes(Network::Testnet) {
        assert!(bootnode.ends_with(":30304"), "{bootnode}");
    }
}

/// Inventing peers for a private network sends the node hunting for hosts the
/// operator does not own.
#[test]
fn a_private_network_has_no_bootnodes_and_no_genesis_hash() {
    assert!(neox_bootnodes(Network::Private).is_empty());
    assert!(neox_genesis_hash(Network::Private).is_none());
    assert!(neox_reth_chain(Network::Private).is_none());
}

#[test]
fn the_genesis_hashes_are_32_byte_hex() {
    for network in [Network::Mainnet, Network::Testnet] {
        let hash = neox_genesis_hash(network).expect("a public network has a genesis hash");
        assert_eq!(hash.len(), 66, "0x + 64 hex digits");
        assert!(hash[2..].chars().all(|c| c.is_ascii_hexdigit()));
    }
    assert_ne!(
        neox_genesis_hash(Network::Mainnet),
        neox_genesis_hash(Network::Testnet)
    );
}

/// The names neox-rs accepts for `--chain`. Anything else aborts at startup.
#[test]
fn the_reth_chain_names_match_the_client() {
    assert_eq!(neox_reth_chain(Network::Mainnet), Some("neox-mainnet"));
    assert_eq!(neox_reth_chain(Network::Testnet), Some("neox-testnet"));
}

/// A private chain id follows the operator's own network number so one private
/// network stays internally consistent across both chain families.
#[test]
fn a_private_chain_id_follows_the_operator_profile() {
    let profile = RuntimeConfigProfile {
        network_magic: 776_655,
        ..profile_defaults()
    };
    assert_eq!(
        neox_chain_id(Network::Private, Some(&profile)),
        776_655,
        "the profile's own network number is the private chain id"
    );
    assert_eq!(neox_chain_id(Network::Private, None), 1_230_000);
}

#[test]
fn neox_blocks_are_three_times_faster_than_n3() {
    assert_eq!(neox_block_period_secs(Network::Mainnet), 5);
    assert_eq!(neox_validator_count(Network::Mainnet), 7);
    assert_eq!(neox_validator_count(Network::Private), 1);
}

fn profile_defaults() -> RuntimeConfigProfile {
    RuntimeConfigProfile {
        network_magic: 0,
        seed_nodes: Vec::new(),
        validators_count: 1,
        committee_public_keys: Vec::new(),
        consensus_enabled: false,
    }
}
