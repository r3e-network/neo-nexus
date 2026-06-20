use serde::Serialize;

use super::{MissingProcessMetric, NodeProcessMetrics, SystemMetrics};

mod summary;
mod text;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MetricsSnapshot {
    pub captured_at_unix: u64,
    pub system: SystemMetrics,
    pub node_processes: Vec<NodeProcessMetrics>,
    pub missing_processes: Vec<MissingProcessMetric>,
}

impl MetricsSnapshot {
    pub fn empty() -> Self {
        Self {
            captured_at_unix: 0,
            system: SystemMetrics {
                cpu_usage_percent: 0.0,
                total_memory_bytes: 0,
                used_memory_bytes: 0,
                available_memory_bytes: 0,
                memory_usage_percent: 0.0,
                process_count: 0,
            },
            node_processes: Vec::new(),
            missing_processes: Vec::new(),
        }
    }
}
