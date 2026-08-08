use std::str::FromStr;

use super::*;

/// The JSON token a client serialises to must be the token the parser accepts.
///
/// `--runtime-smoke-json` and `--validate-node-config-json` put `node_type` into
/// their output, and an operator script that reads one back and passes it to
/// another invocation has to get the same client. A derived `kebab-case` rename
/// gives `neo-x-geth`, which `FromStr` rejects — so this broke on Neo X and
/// only on Neo X, which is exactly the kind of gap a fleet-wide script finds
/// months later.
#[test]
fn every_client_survives_a_json_round_trip() {
    for node_type in NodeType::ALL {
        let json = serde_json::to_string(&node_type).expect("serialises");
        let token = json.trim_matches('"');
        assert_eq!(
            NodeType::from_str(token).ok(),
            Some(node_type),
            "{node_type} serialises to `{token}`, which FromStr does not accept",
        );
    }
}

/// The same token an operator types is the one a machine reads, so there is one
/// spelling of each client rather than a human one and a wire one.
#[test]
fn the_json_token_and_the_displayed_name_agree() {
    for node_type in NodeType::ALL {
        let json = serde_json::to_string(&node_type).expect("serialises");
        assert_eq!(json.trim_matches('"'), node_type.to_string(), "{node_type}");
    }
}

#[test]
fn an_unknown_client_is_rejected_rather_than_defaulted() {
    for unknown in ["", "neo", "neox", "neo-x-geth", "geth", "NEO-CLI"] {
        assert!(NodeType::from_str(unknown).is_err(), "{unknown}");
    }
}

/// Every client belongs to exactly one chain, and both families are represented
/// — a family with no clients would be a picker entry that selects nothing.
#[test]
fn both_chain_families_have_clients() {
    for family in ChainFamily::ALL {
        assert!(
            NodeType::ALL
                .into_iter()
                .any(|node_type| node_type.family() == family),
            "{family} has no client",
        );
    }
}

/// A client's default engine must be one it actually supports, or a freshly
/// created node is invalid the moment it exists.
#[test]
fn every_default_storage_engine_is_supported_by_its_client() {
    for node_type in NodeType::ALL {
        let default = node_type.default_storage_engine();
        assert!(
            node_type.supports_storage_engine(default),
            "{node_type} defaults to an engine it rejects",
        );
    }
}

/// Neo X storage is not an operator choice, so the label names what the client
/// really uses instead of echoing the placeholder the field holds.
#[test]
fn a_neox_storage_label_never_echoes_the_placeholder_engine() {
    for node_type in [NodeType::NeoXGeth, NodeType::NeoXReth] {
        let label = node_type.storage_label(StorageEngine::RocksDb);
        assert!(!label.contains("rocksdb"), "{node_type}: {label}");
        assert!(label.contains("built in"), "{node_type}: {label}");
    }
    assert_eq!(
        NodeType::NeoGo.storage_label(StorageEngine::LevelDb),
        StorageEngine::LevelDb.to_string(),
    );
}
