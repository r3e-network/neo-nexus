use crate::metrics::{
    formatter::clean_usage_percent,
    types::{MetricsSnapshot, NodeProcessMetrics},
};

impl MetricsSnapshot {
    pub fn node_process(&self, node_id: &str) -> Option<&NodeProcessMetrics> {
        self.node_processes
            .iter()
            .find(|metrics| metrics.node_id == node_id)
    }

    pub fn total_node_memory_bytes(&self) -> u64 {
        self.node_processes
            .iter()
            .map(|metrics| metrics.memory_bytes)
            .sum()
    }

    pub fn total_node_cpu_usage_percent(&self) -> f32 {
        clean_usage_percent(
            self.node_processes
                .iter()
                .map(|metrics| metrics.cpu_usage_percent)
                .sum(),
        )
    }

    pub fn system_cpu_usage_percent(&self) -> f32 {
        clean_usage_percent(self.system.cpu_usage_percent)
    }

    pub fn system_memory_usage_percent(&self) -> f32 {
        clean_usage_percent(self.system.memory_usage_percent)
    }

    pub(in crate::metrics) fn node_cpu_usage_percent(metrics: &NodeProcessMetrics) -> f32 {
        clean_usage_percent(metrics.cpu_usage_percent)
    }

    pub fn status_label(&self) -> &'static str {
        if self.missing_processes.is_empty() {
            "ok"
        } else {
            "missing-processes"
        }
    }

    pub fn is_success(&self) -> bool {
        self.missing_processes.is_empty()
    }

    pub fn exit_code(&self) -> i32 {
        if self.is_success() {
            0
        } else {
            1
        }
    }
}
