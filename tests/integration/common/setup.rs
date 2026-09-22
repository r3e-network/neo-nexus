//! Shared test setup utilities for integration tests
//!
//! Provides common helpers for creating NodeManager instances, temporary
//! node configurations, launch plans, and log fixtures across all
//! integration tests.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

pub use anyhow::Result;
pub use neo_nexus::launch::LaunchPlan;
pub use neo_nexus::types::{Network, NodeConfig, NodeStatus, NodeType};
pub use std::fs;
pub use std::path::{Path, PathBuf};
pub use tempfile::TempDir;

use neo_nexus::config::ConfigFormat;
use neo_nexus::supervisor::log_path_for;
use neo_nexus::supervisor::model::{LogParserAdapter, NodeAdapters};
use neo_nexus::types::{validate_node_id, NodeTypeTraits};

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

/// A stand-in LaunchPlan for tests that only need a well-formed plan value.
pub fn test_launch_plan() -> LaunchPlan {
    LaunchPlan {
        binary_path: PathBuf::from("bin").join("neo-node"),
        args: Vec::new(),
        working_dir: PathBuf::from("."),
        managed_config_path: None,
        display_command: "neo-node --test".to_string(),
    }
}

/// PID returned by the harness's simulated process spawn. Fixed so tests can
/// assert on it; this is the process-spawn double, not a product constant.
pub const HARNESS_SPAWN_PID: u32 = 12345;

/// Node-manager facade used by the integration suite.
///
/// Process spawning is simulated (see [`HARNESS_SPAWN_PID`]); metrics and log
/// parsing delegate to the real [`NodeAdapters`] registry so the suite
/// exercises production parsers and exporters rather than a second copy of
/// their logic.
#[derive(Debug)]
pub struct NodeManager {
    adapters: NodeAdapters,
    data_dir: PathBuf,
    running: Mutex<HashMap<String, u32>>,
}

impl NodeManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            adapters: NodeAdapters::initialized(),
            data_dir,
            running: Mutex::new(HashMap::new()),
        }
    }

    /// Record `node` as running and return the simulated spawn pid.
    pub async fn start_node(&self, node: &NodeConfig, plan: &LaunchPlan) -> Result<u32> {
        validate_node_id(&node.id)?;
        if plan.binary_path.as_os_str().is_empty() {
            anyhow::bail!("launch plan has no binary path for node {}", node.id);
        }
        let mut running = self
            .running
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if running.contains_key(&node.id) {
            anyhow::bail!("node {} is already running", node.id);
        }
        running.insert(node.id.clone(), HARNESS_SPAWN_PID);
        Ok(HARNESS_SPAWN_PID)
    }

    /// Drop `id` from the running set. Errors when the id was never started,
    /// so a test that stops a node it did not start cannot pass silently.
    pub fn stop_node(&self, id: &str) -> Result<()> {
        validate_node_id(id)?;
        let mut running = self
            .running
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match running.remove(id) {
            Some(_) => Ok(()),
            None => anyhow::bail!("node {id} is not running"),
        }
    }

    pub async fn restart_node(&self, node: &NodeConfig, plan: &LaunchPlan) -> Result<u32> {
        let already_running = self
            .running
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .contains_key(&node.id);
        if already_running {
            self.stop_node(&node.id)?;
        }
        self.start_node(node, plan).await
    }

    /// Summarise metrics-exporter configuration for `node` through the real
    /// metrics adapter registry. The summary names the node type so callers can
    /// assert which client produced it.
    pub fn collect_metrics(&self, node: &NodeConfig) -> Result<String> {
        let adapter = self
            .adapters
            .get_metrics_adapter(&node.node_type)
            .ok_or_else(|| {
                anyhow::anyhow!("no metrics adapter registered for {}", node.node_type)
            })?;

        let config_bytes = adapter.generate_config(node)?;
        let normalized = adapter.normalize_metrics(&config_bytes)?;
        let exporter = adapter.exporter_package().unwrap_or("none");
        let url = adapter
            .metrics_url(node.rpc_port)
            .unwrap_or_else(|| "unavailable".to_string());

        Ok(format!(
            "# node_type: {}\n# exporter: {exporter}\n# metrics_url: {url}\n{normalized}",
            node.node_type
        ))
    }

    /// Parse up to `limit` structured entries from the node's log file using
    /// the real per-type [`LogParserAdapter`]. A missing log file is an empty
    /// observation set, not an error.
    pub fn parse_logs(&self, node: &NodeConfig, limit: usize) -> Result<Vec<String>> {
        let parser: &dyn LogParserAdapter =
            self.adapters.get_log_parser(&node.node_type).ok_or_else(|| {
                anyhow::anyhow!("no log parser registered for {}", node.node_type)
            })?;

        let log_path = log_path_for(self.data_dir.join("logs"), node);
        if !log_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&log_path)?;
        let mut observed = Vec::new();
        for line in content.lines() {
            if observed.len() >= limit {
                break;
            }
            if let Some(entry) = parser.parse_line(line) {
                observed.push(format!("{}: {}", entry.level, entry.message));
            }
        }
        Ok(observed)
    }
}

/// Create a NodeManager backed by an isolated temporary directory.
///
/// Returns the manager plus the TempDir guard; the directory is removed
/// when the guard is dropped. The manager resolves log files under this
/// directory, so [`write_mock_node_log`] and [`NodeManager::parse_logs`] agree
/// on where a node's log lives.
pub async fn spawn_supervised_server(test_name: &str) -> Result<(NodeManager, TempDir)> {
    let temp_dir = tempfile::Builder::new()
        .prefix(&format!("neonexus-{test_name}-"))
        .tempdir()?;

    let manager = NodeManager::new(temp_dir.path().to_path_buf());
    Ok((manager, temp_dir))
}

/// Write a mock log file for `node` where the manager's log discovery
/// expects it (`<data_dir>/logs/<type>-<id>.log`).
pub fn write_mock_node_log(data_dir: &Path, node: &NodeConfig, content: &str) -> Result<PathBuf> {
    let log_dir = data_dir.join("logs");
    fs::create_dir_all(&log_dir)?;

    let log_path = log_path_for(&log_dir, node);
    fs::write(&log_path, content)?;

    Ok(log_path)
}

/// Clear process-wide test state left behind by a test.
///
/// Unsets `NEONEXUS_DATA_DIR` under [`env_lock`] so a concurrent test that is
/// holding the var cannot race the cleanup.
pub fn ensure_clean_environment() -> Result<()> {
    let _guard = env_lock();
    std::env::remove_var("NEONEXUS_DATA_DIR");
    Ok(())
}

/// Create a throwaway workspace holding a node config for `node_type`.
///
/// The config is written in the type's real [`ConfigFormat`] at the type's
/// real [`NodeTypeTraits::config_path`], and names the private network and
/// RPC port so tests can assert on its contents.
pub fn create_mock_config(node_type: NodeType) -> Result<(PathBuf, PathBuf)> {
    let workspace = tempfile::Builder::new()
        .prefix("neonexus-mock-config-")
        .tempdir()?;
    let workspace_path = workspace.keep();

    let config_rel = node_type.config_path();
    let config_path = workspace_path.join(&config_rel);
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let body = match node_type.config_format() {
        ConfigFormat::Json => format!(
            r#"{{"ApplicationConfiguration": {{"Network": "Private", "RpcPort": 30333}}}}"#
        ),
        ConfigFormat::Yaml => {
            "ApplicationConfiguration:\n  Network: Private\n  RpcPort: 30333\n".to_string()
        }
        ConfigFormat::Toml => {
            "[ApplicationConfiguration]\nNetwork = \"Private\"\nRpcPort = 30333\n".to_string()
        }
    };
    fs::write(&config_path, body)?;

    Ok((workspace_path, config_path))
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

    #[tokio::test]
    async fn test_spawn_supervised_server() {
        let (_manager, temp_dir) = spawn_supervised_server("spawn_test").await.unwrap();
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
