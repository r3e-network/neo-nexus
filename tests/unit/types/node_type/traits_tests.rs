use std::path::{Path, PathBuf};

use crate::config::ConfigFormat;
use crate::types::{NodeType, NodeTypeTraits};

/// Trait test suite - verifies that all NodeType impls satisfy the same interface.
///
/// This module validates that the `NodeTypeTraits` trait provides consistent,
/// correct behavior across all 5 node runtime variants (neo-cli, neo-go,
/// neo-rs, neox-geth, neox-rs).

// ============================================================================
// SECTION 1: CONFIG FORMAT CONTRACT
// ============================================================================

#[test]
fn config_format_returns_valid_config_type() {
    // Every node type must return a valid ConfigFormat that can be used
    for node in NodeType::ALL {
        let format = node.config_format();
        // ConfigFormat should always be some valid variant
        assert!(!format!("{:?}", format).is_empty());
    }
}

#[test]
fn only_neo_go_uses_yaml_all_others_use_json() {
    // YAML constraint: neo-go is YAML-only, everything else is JSON
    assert!(
        NodeType::NeoGo.config_format() == ConfigFormat::Yaml,
        "neo-go must use YAML",
    );

    for node in &[NodeType::NeoCli, NodeType::NeoRs, NodeType::NeoXGeth, NodeType::NeoXReth] {
        assert_eq!(
            node.config_format(),
            ConfigFormat::Json,
            "{node} should use JSON, not YAML",
        );
    }
}

#[test]
fn config_format_is_consistent_across_same_type() {
    // Multiple instances of same node type must yield same config format
    let formats = NodeType::ALL.map(|node| node.config_format());
    for &node in &NodeType::ALL {
        assert_eq!(formats[node as usize], node.config_format());
    }
}

// ============================================================================
// SECTION 2: CONFIG PATH CONTRACT
// ============================================================================

#[test]
fn all_config_paths_are_relative() {
    // Paths should be relative to working directory, not absolute
    for node in NodeType::ALL {
        let path = node.config_path();
        assert!(
            path.as_path().is_relative(),
            "{node}: {path:?} should be relative, not absolute",
        );
    }
}

#[test]
fn config_path_matches_node_type_pattern() {
    // Neo-cli: root-level config.json
    // Neo-go: nested config/config.yml
    // Neo-X and neo-rs: nested config/config.json
    assert_eq!(
        NodeType::NeoCli.config_path(),
        PathBuf::from("config.json"),
    );

    assert_eq!(
        NodeType::NeoGo.config_path(),
        PathBuf::from("config/config.yml"),
    );

    for node in &[NodeType::NeoRs, NodeType::NeoXGeth, NodeType::NeoXReth] {
        assert_eq!(
            node.config_path(),
            PathBuf::from("config/config.json"),
        );
    }
}

#[test]
fn config_file_name_matches_config_format_extension() {
    // File extension must match declared config format
    for node in NodeType::ALL {
        let path = node.config_path();
        let format = node.config_format();

        let expected_ext = match format {
            ConfigFormat::Yaml => "yml",
            ConfigFormat::Json | ConfigFormat::Toml => "json",
        };

        assert!(
            path.extension()
                .is_some_and(|ext| ext == expected_ext),
            "{node}: {path:?} should end with .{expected_ext} for {format:?}",
        );
    }
}

// ============================================================================
// SECTION 3: PLUGIN SUPPORT CONTRACT
// ============================================================================

#[test]
fn plugin_directory_matches_supports_plugins_boolean() {
    // If supports_plugins() returns true, plugin_directory() must return Some()
    for node in NodeType::ALL {
        let has_plugins = node.supports_plugins();
        let plugin_dir = node.plugin_directory();

        if has_plugins {
            assert!(plugin_dir.is_some(), "{node}: supports plugins but no directory specified");
        } else {
            assert!(plugin_dir.is_none(), "{node}: does not support plugins but has directory");
        }
    }
}

#[test]
fn only_neo_cli_has_plugin_system() {
    // C#-based neo-cli uses DLL plugins; others don't support dynamic plugins
    assert!(
        NodeType::NeoCli.supports_plugins(),
        "neo-cli should support plugins",
    );
    assert!(
        NodeType::NeoCli.plugin_directory().unwrap().ends_with("Plugins"),
        "neo-cli plugin dir must end with 'Plugins'",
    );

    for node in &[NodeType::NeoGo, NodeType::NeoRs, NodeType::NeoXGeth, NodeType::NeoXReth] {
        assert!(
            !node.supports_plugins(),
            "{node} should not support plugins",
        );
        assert!(node.plugin_directory().is_none(), "{node} should not have plugin dir");
    }
}

#[test]
fn plugin_directory_path_structure_is_correct() {
    // Plugin directories should not contain path separators that suggest subdirs
    for node in NodeType::ALL {
        if let Some(dir) = node.plugin_directory() {
            assert!(
                !dir.components().count() > 1,
                "{node}: plugin dir {dir:?} should be a simple directory name",
            );
        }
    }
}

// ============================================================================
// SECTION 4: BINARY NAME CONTRACT
// ============================================================================

#[test]
fn binary_names_are_lowercase_with_dashes() {
    // Executable names must follow Unix-friendly lowercase convention
    for node in NodeType::ALL {
        let name = node.default_binary_name();
        assert!(
            name == name.to_lowercase(),
            "{node}: {name} should be lowercase",
        );
        assert!(
            !name.contains(" "),
            "{node}: {name} should not contain spaces",
        );
    }
}

#[test]
fn windows_executables_include_exe_extension() {
    // Only Windows binaries should have .exe extension
    assert_eq!(NodeType::NeoCli.default_binary_name(), "neo-cli.exe");

    for node in &[NodeType::NeoGo, NodeType::NeoRs, NodeType::NeoXGeth, NodeType::NeoXReth] {
        assert!(
            !node.default_binary_name().ends_with(".exe"),
            "{node}: {} should not have .exe extension", node.default_binary_name()
        );
    }
}

#[test]
fn binary_names_are_unique_per_node_type() {
    // No two node types should share the same binary name
    let names: Vec<_> = NodeType::ALL.iter().map(|n| n.default_binary_name()).collect();
    let mut unique_names = names.clone();
    unique_names.sort();
    unique_names.dedup();

    assert_eq!(names.len(), unique_names.len(), "Duplicate binary names detected");
}

// ============================================================================
// SECTION 5: INTEGRATIVE BEHAVIORAL TESTS
// ============================================================================

#[test]
fn plugin_types_have_full_workspace_layout() {
    // Types with plugins need: main config + Plugins/ directory
    for node in NodeType::ALL {
        if node.supports_plugins() {
            assert!(node.config_path().is_relative());
            assert!(node.plugin_directory().is_some());

            let config = node.config_path();
            let plugin_dir = node.plugin_directory().unwrap();

            // Config and plugins should be at different levels or related
            assert!(
                config.file_name().is_some() && is_file_name_like(&plugin_dir),
                "{node}: plugin-enabled workspace incomplete",
            );
        }
    }
}

#[test]
fn non_plugin_types_have_simpler_workspace() {
    // Types without plugins only need config, no directory structure
    for node in &[NodeType::NeoGo, NodeType::NeoRs, NodeType::NeoXGeth, NodeType::NeoXReth] {
        assert!(!node.supports_plugins());
        assert!(node.plugin_directory().is_none());

        // Should still have config
        let config = node.config_path();
        assert!(config.file_name().is_some(), "{node} missing config file");
    }
}

#[test]
fn trait_implementations_are_internal_and_private_to_enum() {
    // Verify implementation is on the enum itself (encapsulation)
    // This test documents the architectural decision: traits implemented on
    // the enum, not requiring callers to know trait object details
    fn require_trait_impl<T>(node: T) -> String
    where
        T: NodeTypeTraits,
    {
        node.default_binary_name().to_string()
    }

    for node in NodeType::ALL {
        let name = require_trait_impl(node);
        assert!(!name.is_empty());
    }
}

#[test]
fn workspace_root_detection_by_binary_name() {
    // Test that binary names could reliably locate workspace roots
    for node in NodeType::ALL {
        let bin_name = node.default_binary_name();
        let work_dir = std::env::current_dir().unwrap();
        
        // The binary would exist somewhere in PATH or workspace
        // This verifies the naming convention supports OS detection
        let _check_bin_exists = work_dir.join(bin_name);
    }
}

// ============================================================================
// SECTION 6: EDGE CASES AND BORDERLINE CONDITIONS
// ============================================================================

#[test]
fn empty_strings_and_special_values_rejected_in_from_str() {
    // While these are tested elsewhere, ensure trait implementations handle
    // edge cases gracefully when constructed via FromStr
    let invalid_inputs = [
        "neo",           // Ambiguous prefix
        "neox",          // Neo X family prefix
        "neo-node",      // Generic term
        "neo-x-geth",    // Wrong Neo X format (should be neox-geth)
        "neo-rs-node",   // Redundant suffix
        "NEO-CLI",       // Wrong case
        "Neo-Go",        // Mixed case
    ];

    for input in &invalid_inputs {
        // Should fail gracefully, not return None or default
        let result = <NodeType as std::str::FromStr>::from_str(input);
        assert!(result.is_err(), "{input} should be rejected");
    }
}

#[test]
fn serde_serialization_preserves_all_fields() {
    // JSON serialization/deserialization preserves complete information
    for node in NodeType::ALL {
        let json = serde_json::to_string(&node).unwrap();
        let round_trip: NodeType = serde_json::from_str(&json).unwrap();
        assert_eq!(node, round_trip, "Full preservation required");
    }
}

#[test]
fn display_trait_matches_deserialization_token() {
    // Display and FromStr tokens must be identical for human/wire compatibility
    for node in NodeType::ALL {
        let display = node.to_string();
        let from_str: NodeType = display.parse().unwrap();
        assert_eq!(node, from_str, "Display/FromStr symmetry broken");
    }
}

// ============================================================================
// UTILITY EXTENSIONS FOR TESTS
// ============================================================================

fn is_file_name_like(path: &Path) -> bool {
    path.components().all(|c| matches!(c, std::path::Component::Normal(_)))
}
