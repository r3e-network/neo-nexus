use crate::types::NodeConfig;

pub(super) fn node_rpc_endpoint(node: &NodeConfig) -> String {
    format!("http://127.0.0.1:{}", node.rpc_port)
}

pub(super) fn normalize_endpoint(endpoint: &str) -> String {
    let trimmed = endpoint.trim();
    if trimmed.chars().all(|character| character.is_ascii_digit()) {
        format!("http://127.0.0.1:{trimmed}")
    } else if !trimmed.contains("://") {
        format!("http://{trimmed}")
    } else {
        trimmed.to_string()
    }
}
