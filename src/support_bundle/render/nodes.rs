use anyhow::Result;

use crate::{
    launch::runtime_args_include_config, redaction::redact_sensitive_args,
    support_bundle::SupportBundleNode, types::NodeConfig,
};

pub(in crate::support_bundle) fn support_nodes_json(nodes: &[NodeConfig]) -> Result<String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(
            &nodes
                .iter()
                .map(support_node)
                .collect::<Vec<SupportBundleNode>>()
        )?
    ))
}

pub(in crate::support_bundle) fn support_nodes_text(nodes: &[NodeConfig]) -> String {
    let mut lines = vec![format!("nodes: {}", nodes.len())];
    if nodes.is_empty() {
        lines.push("node: none".to_string());
    } else {
        for node in nodes {
            let external_config = runtime_args_include_config(node.node_type, &node.args);
            lines.push(format!(
                "node: {} | {} | {} | {} | status={} | rpc={} | p2p={} | ws={} | args={} | external-config={}",
                node.name,
                node.node_type,
                node.network,
                node.runtime_version,
                node.status,
                node.rpc_port,
                node.p2p_port,
                node.ws_port.map_or_else(|| "-".to_string(), |port| port.to_string()),
                node.args.len(),
                external_config,
            ));
        }
    }
    lines.push(String::new());
    lines.join("\n")
}

fn support_node(node: &NodeConfig) -> SupportBundleNode {
    SupportBundleNode {
        id: node.id.clone(),
        name: node.name.clone(),
        node_type: node.node_type.to_string(),
        network: node.network.to_string(),
        runtime_version: node.runtime_version.clone(),
        storage_engine: node.storage_engine.to_string(),
        status: node.status.to_string(),
        pid: node.pid,
        binary_path: node.binary_path.display().to_string(),
        argument_count: node.args.len(),
        redacted_args: redact_sensitive_args(&node.args),
        external_config_arg: runtime_args_include_config(node.node_type, &node.args),
        rpc_port: node.rpc_port,
        p2p_port: node.p2p_port,
        ws_port: node.ws_port,
    }
}
