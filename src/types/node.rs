use std::path::{Path, PathBuf};

use anyhow::Result;

use super::{Network, NodeStatus, NodeType, StorageEngine};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewNode {
    pub name: String,
    pub node_type: NodeType,
    pub network: Network,
    pub binary_path: PathBuf,
    pub args: Vec<String>,
    pub runtime_version: String,
    pub storage_engine: StorageEngine,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeConfig {
    pub id: String,
    pub name: String,
    pub node_type: NodeType,
    pub network: Network,
    pub binary_path: PathBuf,
    pub args: Vec<String>,
    pub runtime_version: String,
    pub storage_engine: StorageEngine,
    pub rpc_port: u16,
    pub p2p_port: u16,
    pub ws_port: Option<u16>,
    pub status: NodeStatus,
    pub pid: Option<u32>,
}

/// Validate the stable identifier used as both a database key and a workspace
/// directory component.
///
/// NeoNexus-generated identifiers already use this grammar (`node-<uuid>`).
/// Keeping the accepted alphabet deliberately small means an identifier cannot
/// become an absolute path, a parent component, a Windows drive/UNC path, or a
/// second filename component when it crosses into filesystem code.
pub fn validate_node_id(id: &str) -> Result<()> {
    const MAX_NODE_ID_BYTES: usize = 128;

    if id.is_empty() {
        anyhow::bail!("node id is required");
    }
    if id.len() > MAX_NODE_ID_BYTES {
        anyhow::bail!("node id exceeds {MAX_NODE_ID_BYTES} bytes");
    }
    if !id.is_ascii() {
        anyhow::bail!("node id must contain ASCII letters, digits, '-' or '_'");
    }
    let mut bytes = id.bytes();
    let first = bytes.next().unwrap_or_default();
    if !first.is_ascii_alphanumeric()
        || !bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        anyhow::bail!(
            "node id must start with an ASCII letter or digit and contain only letters, digits, '-' or '_'"
        );
    }
    Ok(())
}

/// Build a node-owned directory below `root` after enforcing the identifier
/// boundary. This is the shared containment guard for launch and export paths.
pub fn node_workspace_path(root: impl AsRef<Path>, node_id: &str) -> Result<PathBuf> {
    validate_node_id(node_id)?;
    let root = root.as_ref();
    let path = root.join(node_id);
    if !path.starts_with(root) {
        anyhow::bail!("node workspace path escaped its configured root");
    }
    Ok(path)
}

#[cfg(test)]
#[path = "../../tests/unit/types/node/tests.rs"]
mod tests;
