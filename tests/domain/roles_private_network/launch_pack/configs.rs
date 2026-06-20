use std::path::Path;

use crate::*;
use neo_nexus::types::NodeConfig;

pub(super) fn assert_managed_configs(workspace_root: &Path, created: &[NodeConfig]) {
    let validator_config_path = ConfigExporter::managed_target_path(
        workspace_root.join("nodes").join(&created[0].id),
        &created[0],
    );
    let validator_config: toml::Value =
        toml::from_str(&std::fs::read_to_string(validator_config_path).unwrap()).unwrap();
    assert_eq!(
        validator_config["network"]["network_magic"].as_integer(),
        Some(1_230_307)
    );
    assert_eq!(
        validator_config["p2p"]["seed_nodes"][0].as_str(),
        Some("127.0.0.1:30333")
    );
    assert_eq!(
        validator_config["consensus"]["enabled"].as_bool(),
        Some(true)
    );
    assert_eq!(
        validator_config["consensus"]["validators"][0].as_str(),
        Some(committee_public_key("02", '1').as_str())
    );

    let rpc_node = created
        .iter()
        .find(|node| node.name == "neo-rs-rpc-api-5")
        .unwrap();
    let rpc_config_path = ConfigExporter::managed_target_path(
        workspace_root.join("nodes").join(&rpc_node.id),
        rpc_node,
    );
    let rpc_config: toml::Value =
        toml::from_str(&std::fs::read_to_string(rpc_config_path).unwrap()).unwrap();
    assert_eq!(rpc_config["consensus"]["enabled"].as_bool(), Some(false));
    assert_eq!(
        rpc_config["consensus"]["validators"][3].as_str(),
        Some(committee_public_key("03", '4').as_str())
    );
}
