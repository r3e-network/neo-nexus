use std::path::PathBuf;

use super::*;
use crate::{
    launch::LaunchPlan,
    types::{Network, NodeStatus, NodeType, StorageEngine},
};

#[test]
fn start_and_restart_refuse_an_unbound_imported_runtime_before_spawn() {
    let node = NodeConfig {
        id: "node-imported".to_string(),
        name: "imported".to_string(),
        node_type: NodeType::NeoRs,
        network: Network::Testnet,
        binary_path: PathBuf::new(),
        args: Vec::new(),
        runtime_version: "test".to_string(),
        storage_engine: StorageEngine::RocksDb,
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    };
    let plan = LaunchPlan {
        // If the guard used only the plan, this executable could be launched.
        binary_path: std::env::current_exe().expect("test executable path"),
        args: Vec::new(),
        working_dir: std::env::temp_dir(),
        managed_config_path: None,
        display_command: "test executable".to_string(),
    };
    let temp_dir = tempfile::tempdir().expect("temporary log directory");
    let log_path = temp_dir.path().join("unbound-node-must-not-launch.log");
    let mut supervisor = ProcessSupervisor::default();

    let start = supervisor
        .start(&node, &plan, &log_path)
        .expect_err("unbound node start must fail");
    assert!(start.to_string().contains("no trusted local runtime"));
    let restart = supervisor
        .restart(&node, &plan, &log_path)
        .expect_err("unbound node restart must fail");
    assert!(restart.to_string().contains("no trusted local runtime"));
    assert!(!log_path.exists());
}
