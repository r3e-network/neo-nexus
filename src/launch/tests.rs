use std::path::PathBuf;

use crate::types::{Network, NodeConfig, NodeStatus, NodeType, StorageEngine};

use super::LaunchPlanner;

#[test]
fn launch_plan_redacts_sensitive_display_command_without_changing_spawn_args() {
    let node = NodeConfig {
        id: "node-1".to_string(),
        name: "validator".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::from("/opt/neo-rs/neo-node"),
        args: vec![
            "--api-key".to_string(),
            "raw-api-key".to_string(),
            "--wallet-password=raw-password".to_string(),
        ],
        runtime_version: "v0.8.0".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };

    let plan = LaunchPlanner::plan(&node, "/tmp/managed.toml", "/tmp/work");

    assert_eq!(
        plan.args,
        [
            "--api-key",
            "raw-api-key",
            "--wallet-password=raw-password",
            "--config",
            "/tmp/managed.toml"
        ]
    );
    assert!(plan.display_command.contains("<redacted>"));
    assert!(!plan.display_command.contains("raw-api-key"));
    assert!(!plan.display_command.contains("raw-password"));
}
