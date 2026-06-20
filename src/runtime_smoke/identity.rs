use crate::{redaction::redact_sensitive_text, types::NodeType};

use super::text::single_line;

pub(super) fn output_contains_runtime_identity(node_type: NodeType, output: &str) -> bool {
    let normalized = output.to_ascii_lowercase();
    match node_type {
        NodeType::NeoCli => {
            normalized.contains("neo-cli")
                || normalized.contains("neo cli")
                || normalized.contains("neo.console")
                || normalized.contains("neo.cli")
        }
        NodeType::NeoGo => normalized.contains("neo-go") || normalized.contains("neo go"),
        NodeType::NeoRs => {
            normalized.contains("neo-node")
                || normalized.contains("neo-rs")
                || normalized.contains("neo node")
        }
    }
}

pub(super) fn success_message(node_type: NodeType, output: &str) -> String {
    let first_line = output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("runtime responded");
    format!(
        "{node_type} probe responded: {}",
        single_line(&redact_sensitive_text(first_line))
    )
}
