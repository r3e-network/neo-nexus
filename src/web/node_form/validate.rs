//! The rules that turn a parsed draft into a node the repository accepts.
//!
//! Kept apart from `NodeDraft` so the draft stays readable: the struct carries
//! every field as the text the browser posted, while this module owns the
//! port/name/version checks that decide whether that text becomes a `NewNode`.

use std::path::PathBuf;

use crate::{
    core::node::{Network, NewNode, NodeConfig, NodeType, StorageEngine},
    types::NodeTypeTraits,
};

use super::{FieldErrors, NodeDraft};

/// Every field of a draft that has already been parsed and checked.
pub(super) struct Parsed<'a> {
    pub(super) name: &'a str,
    pub(super) node_type: Option<NodeType>,
    pub(super) network: Option<Network>,
    pub(super) storage: Option<StorageEngine>,
    pub(super) args: Option<Vec<String>>,
    pub(super) rpc_port: Option<u16>,
    pub(super) p2p_port: Option<u16>,
    pub(super) ws_port: Option<u16>,
}

impl Parsed<'_> {
    /// Assemble the accepted node. The `?` marks are a consistency check between
    /// the error map and this path, not a silent failure.
    pub(super) fn into_node(self, draft: &NodeDraft) -> Option<NewNode> {
        let node_type = self.node_type?;
        let binary_path = if draft.binary_path.trim().is_empty() {
            PathBuf::from(node_type.default_binary_name())
        } else {
            PathBuf::from(draft.binary_path.trim())
        };
        Some(NewNode {
            name: self.name.to_string(),
            node_type,
            network: self.network?,
            binary_path,
            args: self.args?,
            runtime_version: normalize_version(draft.runtime_version.trim()),
            storage_engine: self.storage?,
            rpc_port: self.rpc_port?,
            p2p_port: self.p2p_port?,
            ws_port: self.ws_port,
        })
    }
}

pub(super) fn parse_port(
    raw: &str,
    label: &str,
    key: &'static str,
    errors: &mut FieldErrors,
) -> Option<u16> {
    if raw.is_empty() {
        errors.insert(key, format!("A {label} port is required."));
        return None;
    }
    match raw.parse::<u16>() {
        Ok(0) => {
            errors.insert(key, format!("The {label} port must be above 0."));
            None
        }
        Ok(port) => Some(port),
        Err(_) => {
            errors.insert(key, format!("\"{raw}\" is not a port number (1-65535)."));
            None
        }
    }
}

/// Blank means "no WebSocket port", which is a valid choice, not an error. The
/// outer `Option` distinguishes those two states; `flatten()` collapses it once
/// validation has passed.
pub(super) fn parse_optional_port(raw: &str, errors: &mut FieldErrors) -> Option<Option<u16>> {
    if raw.is_empty() {
        return None;
    }
    match raw.parse::<u16>() {
        Ok(0) => {
            errors.insert("ws_port", "The WebSocket port must be above 0.".to_string());
            Some(None)
        }
        Ok(port) => Some(Some(port)),
        Err(_) => {
            errors.insert(
                "ws_port",
                format!("\"{raw}\" is not a port number (1-65535)."),
            );
            Some(None)
        }
    }
}

pub(super) fn has_port_error(errors: &FieldErrors) -> bool {
    ["rpc_port", "p2p_port", "ws_port"]
        .iter()
        .any(|key| errors.contains_key(*key))
}

/// `validate_node_ports` reports the first problem as a sentence. Attach it to
/// the field it names so the mark appears where the mistake is, not in a banner
/// above the form.
pub(super) fn add_port_error(errors: &mut FieldErrors, message: &str) {
    let key = if message.contains("WebSocket") {
        "ws_port"
    } else if message.contains("P2P") {
        "p2p_port"
    } else {
        "rpc_port"
    };
    errors.entry(key).or_insert_with(|| message.to_string());
}

pub(super) fn find_name<'a>(
    existing: &'a [NodeConfig],
    current_id: Option<&str>,
    name: &str,
) -> Option<&'a NodeConfig> {
    existing.iter().find(|node| {
        current_id.is_none_or(|current| node.id != current) && node.name.eq_ignore_ascii_case(name)
    })
}

/// Ports collide across nodes as well as within one node: a new node that takes
/// an existing node's P2P port for its own RPC port would start fine and then
/// fail to bind, so it is refused here rather than at launch.
pub(super) fn find_port_conflict<'a>(
    existing: &'a [NodeConfig],
    current_id: Option<&str>,
    rpc_port: u16,
    p2p_port: u16,
    ws_port: Option<u16>,
) -> Option<(u16, &'a NodeConfig)> {
    let wanted = [
        (rpc_port > 0).then_some(rpc_port),
        (p2p_port > 0).then_some(p2p_port),
        ws_port.filter(|&port| port > 0),
    ];
    for node in existing {
        if current_id.is_some_and(|current| current == node.id) {
            continue;
        }
        let held = [
            (node.rpc_port > 0).then_some(node.rpc_port),
            (node.p2p_port > 0).then_some(node.p2p_port),
            node.ws_port.filter(|&port| port > 0),
        ];
        for port in wanted.iter().flatten().copied() {
            if held.iter().flatten().copied().any(|held| held == port) {
                return Some((port, node));
            }
        }
    }
    None
}

/// A blank version means "whatever is current", which the workspace already
/// spells `latest`; an empty string would render as nothing.
pub(super) fn normalize_version(raw: &str) -> String {
    if raw.is_empty() {
        "latest".to_string()
    } else {
        raw.to_string()
    }
}
