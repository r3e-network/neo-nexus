use super::super::types::{MissingProcessMetric, NodeProcessMetrics};

pub(super) fn node_process(process: &NodeProcessMetrics) -> Vec<(&'static str, String)> {
    vec![
        ("node_id", process.node_id.clone()),
        ("node_name", process.node_name.clone()),
        ("pid", process.pid.to_string()),
        ("status", process.status.clone()),
    ]
}

pub(super) fn missing_process(missing: &MissingProcessMetric) -> Vec<(&'static str, String)> {
    vec![
        ("node_id", missing.node_id.clone()),
        ("node_name", missing.node_name.clone()),
        ("pid", missing.pid.to_string()),
    ]
}
