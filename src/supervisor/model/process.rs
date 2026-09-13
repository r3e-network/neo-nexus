//! The process model the supervisor records: what a managed process is, what a
//! start/stop/exit looks like, and the plugin and lifecycle traits a node type
//! can specialise.

use std::{
    fmt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{config::GenerationContext, launch::LaunchPlan, roles::NodeRole, types::NodeConfig};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

/// Trait for managing plugins/extensions across heterogeneous node types.
/// Neo-cli uses C# DLLs, neo-go uses Go modules, neo-rs uses Cargo features.
pub trait PluginSystemAdapter: std::fmt::Debug + Send + Sync {
    /// Discover available plugins in the node's plugin directory.
    fn discover_plugins(&self, node_dir: &Path) -> Result<Vec<PluginMetadata>>;

    /// Install a plugin binary/dynamic library to the plugin directory.
    fn install_plugin(&self, plugin_id: &str, target_dir: &Path) -> Result<()>;

    /// Toggle plugin enabled state by modifying loader configuration.
    fn toggle_plugin(&self, plugin_id: &str, enabled: bool, ctx: &GenerationContext) -> Result<()>;
}

/// Trait for type-specific lifecycle operations (start/stop/restart).
pub trait LifecycleAdapter: std::fmt::Debug + Send + Sync {
    /// Build command-line arguments specific to this node type.
    fn build_process_args(&self, config_path: &Path, role: Option<NodeRole>) -> Vec<String>;

    /// Get health check endpoint URL if applicable.
    /// Many node types expose /health or /status RPC endpoints.
    fn health_endpoint(&self, rpc_port: u16, p2p_port: u16) -> Option<String>;

    /// Parse logs for error patterns specific to this runtime type.
    fn parse_log_for_errors(&self, log_content: &str) -> Vec<String>;

    /// Get metrics endpoint URL if metrics are exposed separately.
    fn metrics_endpoint(&self, rpc_port: u16) -> Option<String>;
}

#[derive(Debug, Default, Clone)]
pub struct NoOpLifecycleAdapter;

impl LifecycleAdapter for NoOpLifecycleAdapter {
    fn build_process_args(&self, _config_path: &Path, _role: Option<NodeRole>) -> Vec<String> {
        vec![
            "--config".to_string(),
            _config_path.to_string_lossy().to_string(),
        ]
    }

    fn health_endpoint(&self, _rpc_port: u16, _p2p_port: u16) -> Option<String> {
        None
    }

    fn parse_log_for_errors(&self, _log_content: &str) -> Vec<String> {
        vec![]
    }

    fn metrics_endpoint(&self, _rpc_port: u16) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStart {
    pub pid: u32,
    pub log_path: PathBuf,
}

/// What a freshly spawned process was doing at the end of its settle window.
///
/// `spawn` succeeding only proves the kernel accepted the executable. A node
/// that rejects its arguments, cannot bind its ports, or cannot parse its
/// config is already gone milliseconds later — so reporting "launched with PID
/// n" straight off the spawn tells the operator something that is no longer
/// true by the time they read it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchConfirmation {
    /// Still running when the window closed. This is not a health check: the
    /// node may still be syncing, unreachable, or wedged. It only means the
    /// process survived its own startup.
    Survived,
    /// Exited inside the window. Whatever it printed before dying is the most
    /// useful thing an operator can be shown, so it is carried here.
    ExitedDuringStartup {
        exit_code: Option<i32>,
        /// The child's own output, header excluded, already trimmed to the
        /// last few meaningful lines.
        output: String,
    },
}

impl LaunchConfirmation {
    /// The sentence an operator or a journal entry should carry. `None` when
    /// the process survived and there is nothing to explain.
    pub fn failure_summary(&self, node_name: &str) -> Option<String> {
        let Self::ExitedDuringStartup { exit_code, output } = self else {
            return None;
        };
        let code = exit_code.map_or_else(
            || "terminated by signal".to_string(),
            |code| format!("exit code {code}"),
        );
        let detail = if output.trim().is_empty() {
            "it wrote nothing to its log".to_string()
        } else {
            output.trim().to_string()
        };
        Some(format!(
            "{node_name} exited during startup ({code}): {detail}"
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStop {
    pub process_id: String,
    pub pid: u32,
    pub log_path: PathBuf,
    pub graceful: bool,
    pub forced: bool,
    pub exit_code: Option<i32>,
}

impl ProcessStop {
    pub fn operator_summary(&self) -> String {
        let mode = if self.forced {
            "forced"
        } else if self.graceful {
            "graceful"
        } else {
            "stopped"
        };
        match self.exit_code {
            Some(code) => format!("pid {} {mode}, exit {code}", self.pid),
            None => format!("pid {} {mode}", self.pid),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessExit {
    pub process_id: String,
    pub node_id: String,
    pub pid: u32,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ManagedProcessKind {
    Node,
    Sidecar,
    Helper,
}

impl ManagedProcessKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Sidecar => "sidecar",
            Self::Helper => "helper",
        }
    }
}

impl fmt::Display for ManagedProcessKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ManagedProcessSpec {
    pub id: String,
    pub kind: ManagedProcessKind,
    pub label: String,
    pub binary_path: PathBuf,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub display_command: String,
}

impl ManagedProcessSpec {
    pub fn for_node(node: &NodeConfig, plan: &LaunchPlan) -> Self {
        Self {
            id: node.id.clone(),
            kind: ManagedProcessKind::Node,
            label: node.name.clone(),
            binary_path: plan.binary_path.clone(),
            args: plan.args.clone(),
            working_dir: plan.working_dir.clone(),
            display_command: plan.display_command.clone(),
        }
    }
}

/// Returns the current Unix timestamp in seconds.
///
/// Intentionally returns `0` when the system clock is before the Unix epoch.
/// This is acceptable because the value is only used in informational log
/// headers (launch/stop markers) where a failed timestamp should not abort
/// the operation. For contexts requiring a strict guarantee, see
/// `repository::helpers::current_unix_time` which returns a `Result`.
pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
