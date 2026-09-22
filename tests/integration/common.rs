//! The shared harness: a throwaway workspace backed by a real SQLite repository,
//! and a stand-in node runtime the real supervisor can launch and stop on every
//! platform CI runs on.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use neo_nexus::{
    config::ConfigExporter,
    launch::{LaunchPlan, LaunchPlanner},
    repository::Repository,
    supervisor::log_path_for,
    types::{node_workspace_path, Network, NewNode, NodeConfig, NodeType},
};
use tempfile::TempDir;

#[path = "../support/stub_runtime.rs"]
mod stub_runtime;

pub use stub_runtime::stub_runtime_binary;

/// A workspace in its own temporary directory, removed when dropped.
pub struct Workspace {
    // Declared before `directory` so the database closes before the directory
    // holding it is removed: Windows refuses to delete an open file.
    repository: Repository,
    directory: TempDir,
}

/// What the headless `--node-start` path derives before it launches a node
/// (`src/cli/actions/node_control.rs`): the plan, the managed config path it
/// renders first, and the log the supervisor writes.
pub struct LaunchInputs {
    pub plan: LaunchPlan,
    pub config_path: PathBuf,
    pub log_path: PathBuf,
}

impl Workspace {
    pub fn new() -> Result<Self> {
        let directory = tempfile::Builder::new()
            .prefix("neonexus-integration-")
            .tempdir()?;
        let repository = Repository::open(directory.path().join("neonexus.db"))?;
        Ok(Self {
            repository,
            directory,
        })
    }

    pub fn repository(&self) -> &Repository {
        &self.repository
    }

    pub fn path(&self) -> &Path {
        self.directory.path()
    }

    /// Register a node whose runtime is the stand-in binary.
    pub fn add_node(&self, name: &str, node_type: NodeType, rpc_port: u16) -> Result<NodeConfig> {
        self.add_node_with_runtime(name, node_type, rpc_port, stub_runtime_binary())
    }

    pub fn add_node_with_runtime(
        &self,
        name: &str,
        node_type: NodeType,
        rpc_port: u16,
        binary_path: PathBuf,
    ) -> Result<NodeConfig> {
        self.repository.create_node(NewNode {
            name: name.to_string(),
            node_type,
            network: Network::Testnet,
            binary_path,
            args: Vec::new(),
            runtime_version: "integration".to_string(),
            storage_engine: node_type.default_storage_engine(),
            rpc_port,
            p2p_port: rpc_port + 1,
            ws_port: None,
        })
    }

    /// The node as the workspace records it right now.
    pub fn stored(&self, node_id: &str) -> Result<NodeConfig> {
        self.repository
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == node_id)
            .with_context(|| format!("node {node_id} is not registered"))
    }

    pub fn launch_inputs(&self, node: &NodeConfig) -> Result<LaunchInputs> {
        let work_dir = node_workspace_path(self.path().join("nodes"), &node.id)?;
        let config_path = ConfigExporter::managed_target_path(&work_dir, node);
        let log_path = log_path_for(self.path().join("logs"), node);
        let plan = LaunchPlanner::plan(node, &config_path, &work_dir);
        Ok(LaunchInputs {
            plan,
            config_path,
            log_path,
        })
    }
}
