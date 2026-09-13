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

/// ============================================================================
/// SECTION 2: STORAGE ENGINE VALIDITY CONTRACT
/// ============================================================================

#[test]
fn neo_go_only_accepts_level_db_as_storage_engine() {
    // neo-go's native database is LevelDB, no other choice allowed
    assert!(NodeType::NeoGo.supports_storage_engine(StorageEngine::LevelDb));
    assert!(
        !NodeType::NeoGo.supports_storage_engine(StorageEngine::RocksDb),
        "neo-go should not support RocksDB",
    );
}

#[test]
fn neo_rs_only_accepts_rocks_db_as_storage_engine() {
    // neo-rs is built on Rust ecosystem with RocksDB as primary
    assert!(NodeType::NeoRs.supports_storage_engine(StorageEngine::RocksDb));
    assert!(
        !NodeType::NeoRs.supports_storage_engine(StorageEngine::LevelDb),
        "neo-rs should not support LevelDB",
    );
}

#[test]
fn neo_cli_supports_multiple_storage_engines() {
    // neo-cli offers choice between engines
    assert!(NodeType::NeoCli.supports_storage_engine(StorageEngine::LevelDb));
    assert!(NodeType::NeoCli.supports_storage_engine(StorageEngine::RocksDb));
}

#[test]
fn neo_x_clients_have_fixed_storages_but_documented_reasonably() {
    // Both Neo X variants use fixed storage (Pebble/MDBX) but can't select it
    for node_type in &[NodeType::NeoXGeth, NodeType::NeoXReth] {
        assert!(
            node_type.supports_storage_engine(StorageEngine::RocksDb),
            "{node_type} claims support for RocksDb default"
        );
        assert!(
            node_type.default_storage_engine() == StorageEngine::RocksDb,
            "{node_type} must default to RocksDb"
        );
        let label = node_type.storage_label(StorageEngine::RocksDb);
        assert!(
            label.contains("built") && label.contains("in"),
            "{node_type} must document that storage is fixed: {label}"
        );
    }
}

/// ============================================================================
/// SECTION 3: CROSS-MODULE CONSISTENCY TESTS
/// ============================================================================

#[test]
fn chain_family_matches_binary_name_origin() {
    // Verify binary naming convention aligns with chain family origins
    for node_type in NodeType::ALL {
        let family = node_type.family();

        match family {
            ChainFamily::NeoN3 => {
                // N3 clients have traditional names
                let name = node_type.default_binary_name();
                assert!(
                    name.starts_with("neo-"),
                    "{node_type}: {name} should start with 'neo-'"
                );
            }
            ChainFamily::NeoX => {
                // Neo X clients reflect their implementation origin
                let name = node_type.default_binary_name();
                assert!(
                    name.starts_with("neox-") || name.starts_with("neo-"),
                    "{node_type}: {name} should indicate Neo X or variant"
                );
            }
        }
    }
}

#[test]
fn config_format_determines_parser_requirements() {
    // Each config format requires different parser strategy
    for node_type in NodeType::ALL {
        let format = node_type.config_format();
        let path = node_type.config_path();

        // Verify path matches expected format handling
        match format {
            ConfigFormat::Json => {
                // JSON parsers handle flexible whitespace, comments
                assert!(path.extension().is_some_and(|e| e == "json"));
            }
            ConfigFormat::Yaml => {
                // YAML needs indented structure validation
                assert!(path.extension().is_some_and(|e| e == "yml" || e == "yaml"));
            }
            _ => unreachable!("Unexpected config format"),
        }
    }
}

/// ============================================================================
/// SECTION 4: WORKSPACE GENERATION INTEGRITY
/// ============================================================================

#[test]
fn all_node_types_produce_valid_workspace_paths() {
    // Every node type must generate paths that work in real filesystem
    for node_type in NodeType::ALL {
        let config_path = node_type.config_path();
        let plugin_dir = node_type.plugin_directory();

        // Config path must be joinable
        let _full_config = std::env::current_dir().unwrap().join(&config_path);

        // Plugin directory (if present) must also be joinable
        if let Some(ref plugin) = plugin_dir {
            let full_plugin = std::env::current_dir().unwrap().join(plugin);
            assert!(!full_plugin.exists()); // Won't exist yet - good
        }
    }
}

#[test]
fn plugin_enabled_nodes_require_plugins_directory() {
    // When plugins are supported, workspace generator must create Plugins/ dir
    for node_type in NodeType::ALL {
        if node_type.supports_plugins() {
            let plugin_dir = node_type.plugin_directory();
            assert!(
                plugin_dir.is_some(),
                "{node_type}: missing plugin directory"
            );

            let dir = plugin_dir.unwrap();
            assert!(
                !dir.is_absolute(),
                "{node_type}: plugin dir should be relative"
            );

            // Should resolve to something like "Plugins"
            assert!(
                dir.components().count() <= 1,
                "{node_type}: plugin dir too complex"
            );
        }
    }
}

#[test]
fn node_type_infers_cleanly_from_strings_and_paths() {
    let cases = [
        // Exact names
        ("neo-cli", Some(NodeType::NeoCli)),
        ("neo-go", Some(NodeType::NeoGo)),
        ("neo-rs", Some(NodeType::NeoRs)),
        ("neox-geth", Some(NodeType::NeoXGeth)),
        ("neox-rs", Some(NodeType::NeoXReth)),
        // Windows executables and dlls
        ("neo-cli.exe", Some(NodeType::NeoCli)),
        ("neo-cli.dll", Some(NodeType::NeoCli)),
        ("neo-node.exe", Some(NodeType::NeoRs)),
        ("neox-geth.exe", Some(NodeType::NeoXGeth)),
        ("reth.exe", Some(NodeType::NeoXReth)),
        ("geth.exe", Some(NodeType::NeoXGeth)),
        // Full paths (POSIX and Windows)
        (r"C:\Program Files\Neo\neo-cli.exe", Some(NodeType::NeoCli)),
        ("/usr/local/bin/neo-go", Some(NodeType::NeoGo)),
        ("/opt/runtimes/neo-node", Some(NodeType::NeoRs)),
        (
            r"D:\runtimes\neox-geth\neox-geth.exe",
            Some(NodeType::NeoXGeth),
        ),
        ("/var/lib/neox/neox-rs", Some(NodeType::NeoXReth)),
        ("/usr/bin/reth", Some(NodeType::NeoXReth)),
        ("/usr/bin/geth", Some(NodeType::NeoXGeth)),
        // Empty or unknown
        ("", None),
        ("   ", None),
        ("unknown_binary", None),
    ];

    for (input, expected) in cases {
        assert_eq!(
            NodeType::infer_from_str(input),
            expected,
            "failed inferring from str: {input}"
        );
        let path = std::path::Path::new(input);
        assert_eq!(
            NodeType::infer_from_path(path),
            expected,
            "failed inferring from path: {input}"
        );
    }
}
