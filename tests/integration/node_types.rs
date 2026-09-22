//! The public contracts every node type, node id and port has to satisfy
//! before anything is launched.

use neo_nexus::types::{
    validate_node_id, validate_node_port, ChainFamily, NodeType, NodeTypeTraits, StorageEngine,
};

#[test]
fn node_ids_that_are_safe_as_directory_names_are_accepted() {
    let longest = "a".repeat(128);
    for id in [
        "node-001",
        "test-node-abc123",
        "production-neocli-01",
        "test_underscore_123",
        "UPPERCASE-NODE-456",
        "a",
        longest.as_str(),
    ] {
        assert!(validate_node_id(id).is_ok(), "{id} should be accepted");
    }
}

#[test]
fn node_ids_that_could_escape_or_confuse_a_path_are_rejected() {
    let too_long = "a".repeat(129);
    for id in [
        "",
        "node with spaces",
        "node/slash",
        "node\\backslash",
        "..",
        "../escape",
        "-leading-dash",
        "_leading_underscore",
        "C:drive",
        "node@symbol",
        "node#hash",
        "nöde",
        too_long.as_str(),
    ] {
        assert!(validate_node_id(id).is_err(), "{id:?} should be rejected");
    }
}

#[test]
fn port_zero_is_the_only_invalid_port() {
    // Ports are u16, so 65536 and above cannot be expressed at all.
    for port in [1u16, 1023, 1024, 30333, 45678, 65535] {
        assert!(
            validate_node_port(port, "test").is_ok(),
            "{port} should be valid"
        );
    }
    assert!(validate_node_port(0, "test").is_err());
}

#[test]
fn node_type_names_are_distinct_and_round_trip() {
    let expected = ["neo-cli", "neo-go", "neo-rs", "neox-geth", "neox-rs"];
    assert_eq!(NodeType::ALL.len(), expected.len());
    for (node_type, name) in NodeType::ALL.into_iter().zip(expected) {
        assert_eq!(node_type.to_string(), name);
        let parsed: NodeType = name
            .parse()
            .unwrap_or_else(|error| unreachable!("{name} does not parse: {error}"));
        assert_eq!(parsed, node_type);
    }
}

#[test]
fn node_types_belong_to_their_chain_family() {
    for (node_type, family) in [
        (NodeType::NeoCli, ChainFamily::NeoN3),
        (NodeType::NeoGo, ChainFamily::NeoN3),
        (NodeType::NeoRs, ChainFamily::NeoN3),
        (NodeType::NeoXGeth, ChainFamily::NeoX),
        (NodeType::NeoXReth, ChainFamily::NeoX),
    ] {
        assert_eq!(node_type.family(), family, "{node_type}");
    }
}

#[test]
fn node_types_default_to_their_storage_engine() {
    for (node_type, engine) in [
        (NodeType::NeoCli, StorageEngine::RocksDb),
        (NodeType::NeoGo, StorageEngine::LevelDb),
        (NodeType::NeoRs, StorageEngine::RocksDb),
        (NodeType::NeoXGeth, StorageEngine::RocksDb),
        (NodeType::NeoXReth, StorageEngine::RocksDb),
    ] {
        assert_eq!(node_type.default_storage_engine(), engine, "{node_type}");
    }
}

#[test]
fn only_neo_cli_supports_plugins() {
    for node_type in NodeType::ALL {
        let neo_cli = node_type == NodeType::NeoCli;
        assert_eq!(node_type.supports_plugins(), neo_cli, "{node_type}");
        assert_eq!(
            node_type.plugin_directory(),
            neo_cli.then(|| "Plugins".into()),
            "{node_type}"
        );
    }
}

/// `NodeTypeTraits` is internally consistent: each type's declared config path
/// is relative and carries its declared format's extension.
#[test]
fn each_declared_config_path_carries_its_declared_format() {
    for node_type in NodeType::ALL {
        let format = node_type.config_format();
        let path = node_type.config_path();
        assert!(path.is_relative(), "{node_type}: {}", path.display());
        assert_eq!(
            path.extension().and_then(|extension| extension.to_str()),
            Some(format.extension()),
            "{node_type}: {}",
            path.display()
        );
    }
}
