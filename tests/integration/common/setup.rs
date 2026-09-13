//! Shared test setup utilities for integration tests
//!
//! Provides common helpers for creating NodeManager instances, temporary
//! node configurations, launch plans, and log fixtures across all
//! integration tests.

use std::sync::{Mutex, MutexGuard, OnceLock};

pub use anyhow::Result;
pub use neo_nexus::launch::LaunchPlan;
pub use neo_nexus::node_manager::NodeManager;
pub use neo_nexus::types::{Network, NodeConfig, NodeStatus, NodeType};
pub use std::fs;
pub use std::path::{Path, PathBuf};
pub use tempfile::TempDir;

/// Serialize tests that mutate process-wide state (environment variables).
///
/// Integration tests run on parallel threads within one binary, and
/// `NEONEXUS_DATA_DIR` is process-global. Any test that sets it must hold
/// this guard for its full duration.
pub fn env_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Create a minimal NodeConfig for testing
pub fn make_test_node_config(id: &str, name: &str, node_type: NodeType, port: u16) -> NodeConfig {
    NodeConfig {
        id: id.to_string(),
        name: name.to_string(),
        node_type,
        network: Network::Private,
        binary_path: PathBuf::from("bin").join(node_type.to_string()),
        args: Vec::new(),
        runtime_version: "0.0.0-test".to_string(),
        storage_engine: node_type.default_storage_engine(),
        rpc_port: port,
        p2p_port: port + 1,
        ws_port: None,
        status: NodeStatus::Stopped,
        pid: None,
    }
}

/// Create a minimal LaunchPlan matching a test node configuration
pub fn make_test_launch_plan(node: &NodeConfig, working_dir: &Path) -> LaunchPlan {
    LaunchPlan {
        binary_path: node.binary_path.clone(),
        args: node.args.clone(),
        working_dir: working_dir.to_path_buf(),
        managed_config_path: None,
        display_command: format!("{} --test", node.binary_path.display()),
    }
}

/// Create a NodeManager backed by an isolated temporary directory.
///
/// Returns the manager plus the TempDir guard; the directory is removed
/// when the guard is dropped. The temp dir is NOT wired into the manager's
/// data-dir resolution — tests that need `NEONEXUS_DATA_DIR` must set it
/// themselves while holding [`env_lock`].
pub fn spawn_supervised_server(test_name: &str) -> Result<(NodeManager, TempDir)> {
    let temp_dir = tempfile::Builder::new()
        .prefix(&format!("neonexus-{test_name}-"))
        .tempdir()?;

    let manager = NodeManager::new();
    Ok((manager, temp_dir))
}

/// Write a mock log file for `node` where the manager's log discovery
/// expects it (`<data_dir>/logs/<type>-<id>.log`).
pub fn write_mock_node_log(data_dir: &Path, node: &NodeConfig, content: &str) -> Result<PathBuf> {
    let log_dir = data_dir.join("logs");
    fs::create_dir_all(&log_dir)?;

    let log_path = neo_nexus::supervisor::log_path_for(&log_dir, node);
    fs::write(&log_path, content)?;

    Ok(log_path)
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_make_test_node_config() {
        let node = make_test_node_config("test-001", "Test Node", NodeType::NeoCli, 30333);

        assert_eq!(node.id, "test-001");
        assert_eq!(node.name, "Test Node");
        assert_eq!(node.node_type, NodeType::NeoCli);
        assert_eq!(node.rpc_port, 30333);
        assert_eq!(node.p2p_port, 30334);
        assert_eq!(node.status, NodeStatus::Stopped);
        assert!(node.pid.is_none());
    }

    #[test]
    fn test_spawn_supervised_server() {
        let (_manager, temp_dir) = spawn_supervised_server("spawn_test").unwrap();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_make_test_launch_plan() {
        let node = make_test_node_config("plan-001", "Plan Node", NodeType::NeoGo, 30340);
        let temp_dir = TempDir::new().unwrap();
        let plan = make_test_launch_plan(&node, temp_dir.path());

        assert_eq!(plan.binary_path, node.binary_path);
        assert_eq!(plan.working_dir, temp_dir.path());
        assert!(plan.managed_config_path.is_none());
        assert!(plan.display_command.contains("--test"));
    }

    #[test]
    fn test_write_mock_node_log() {
        let temp_dir = TempDir::new().unwrap();
        let node = make_test_node_config("log-001", "Log Node", NodeType::NeoCli, 30350);
        let content = "[INFO] 2024-01-15T10:30:00Z Started\n[DEBUG] Initializing components";

        let log_path = write_mock_node_log(temp_dir.path(), &node, content).unwrap();

        assert!(log_path.exists());
        let written = fs::read_to_string(log_path).unwrap();
        assert_eq!(written, content);
    }
}
