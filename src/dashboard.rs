use anyhow::Result;

use crate::{catalog::PluginId, repository::Repository, types::NodeStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardSummary {
    pub total_nodes: usize,
    pub running_nodes: usize,
    pub stopped_nodes: usize,
    pub error_nodes: usize,
    pub rpc_enabled_nodes: usize,
    pub health_percent: usize,
}

impl DashboardSummary {
    pub fn load(repository: &Repository) -> Result<Self> {
        let nodes = repository.list_nodes()?;
        let total_nodes = nodes.len();
        let running_nodes = nodes
            .iter()
            .filter(|node| node.status == NodeStatus::Running)
            .count();
        let stopped_nodes = nodes
            .iter()
            .filter(|node| node.status == NodeStatus::Stopped)
            .count();
        let error_nodes = nodes
            .iter()
            .filter(|node| node.status == NodeStatus::Error)
            .count();
        let mut rpc_enabled_nodes = 0;

        for node in &nodes {
            if repository
                .list_plugin_states(&node.id)?
                .iter()
                .any(|state| state.plugin_id == PluginId::RpcServer && state.enabled)
            {
                rpc_enabled_nodes += 1;
            }
        }

        let health_percent = if total_nodes == 0 {
            100
        } else {
            (running_nodes * 100) / total_nodes
        };

        Ok(Self {
            total_nodes,
            running_nodes,
            stopped_nodes,
            error_nodes,
            rpc_enabled_nodes,
            health_percent,
        })
    }
}
