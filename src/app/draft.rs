use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::{
    app::domain::{
        validate_node_port, validate_node_ports, Network, NewNode, NodeConfig, NodeType,
        StorageEngine,
    },
    argv::{format_argv, parse_argv_text},
};

#[derive(Debug, Clone)]
pub(super) struct NodeDraft {
    pub(super) name: String,
    pub(super) node_type: NodeType,
    pub(super) network: Network,
    pub(super) binary_path: String,
    pub(super) args: String,
    pub(super) runtime_version: String,
    pub(super) storage_engine: StorageEngine,
    pub(super) rpc_port: String,
    pub(super) p2p_port: String,
    pub(super) ws_port: String,
}

impl NodeDraft {
    pub(super) fn ensure_storage_compatible(&mut self) {
        if !self.node_type.supports_storage_engine(self.storage_engine) {
            self.storage_engine = self.node_type.default_storage_engine();
        }
    }

    pub(super) fn to_new_node(&self) -> Result<NewNode> {
        let rpc_port = parse_port(&self.rpc_port, "RPC")?;
        let p2p_port = parse_port(&self.p2p_port, "P2P")?;
        let ws_port = parse_optional_port(&self.ws_port)?;
        validate_node_ports(rpc_port, p2p_port, ws_port)?;

        Ok(NewNode {
            name: self.name.clone(),
            node_type: self.node_type,
            network: self.network,
            binary_path: PathBuf::from(self.binary_path.trim()),
            args: parse_argv_text(&self.args)?,
            runtime_version: normalize_runtime_version(&self.runtime_version),
            storage_engine: self.storage_engine,
            rpc_port,
            p2p_port,
            ws_port,
        })
    }

    pub(super) fn load_from_node(&mut self, node: &NodeConfig) {
        self.name = node.name.clone();
        self.node_type = node.node_type;
        self.network = node.network;
        self.binary_path = node.binary_path.display().to_string();
        self.args = format_argv(&node.args);
        self.runtime_version = node.runtime_version.clone();
        self.storage_engine = node.storage_engine;
        self.rpc_port = node.rpc_port.to_string();
        self.p2p_port = node.p2p_port.to_string();
        self.ws_port = node
            .ws_port
            .map_or_else(String::new, |port| port.to_string());
    }
}

impl Default for NodeDraft {
    fn default() -> Self {
        Self {
            name: String::new(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: String::new(),
            args: String::new(),
            runtime_version: "latest".to_string(),
            storage_engine: NodeType::NeoGo.default_storage_engine(),
            rpc_port: "10332".to_string(),
            p2p_port: "10333".to_string(),
            ws_port: "10334".to_string(),
        }
    }
}

fn normalize_runtime_version(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        "latest".to_string()
    } else {
        trimmed.to_string()
    }
}

fn parse_port(raw: &str, label: &str) -> Result<u16> {
    let port = raw
        .trim()
        .parse::<u16>()
        .with_context(|| format!("{label} port must be a number from 1 to 65535"))?;
    validate_node_port(port, label)?;
    Ok(port)
}

fn parse_optional_port(raw: &str) -> Result<Option<u16>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        Ok(None)
    } else {
        let port = trimmed
            .parse::<u16>()
            .context("WebSocket port must be a number from 1 to 65535")?;
        validate_node_port(port, "WebSocket")?;
        Ok(Some(port))
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/draft/tests.rs"]
mod tests;
