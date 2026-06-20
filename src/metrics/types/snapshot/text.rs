use crate::metrics::{
    formatter::format_bytes,
    prometheus,
    types::{MetricsSnapshot, NodeProcessMetrics},
};

impl MetricsSnapshot {
    pub fn to_cli_text(&self) -> String {
        let mut text = self.cli_header_text();
        self.push_node_process_text(&mut text);
        self.push_missing_process_text(&mut text);
        text
    }

    pub fn to_prometheus_text(&self) -> String {
        prometheus::snapshot_to_text(self)
    }

    fn cli_header_text(&self) -> String {
        format!(
            "workspace-metrics: {}\ncaptured-at-unix: {}\nsystem-cpu: {:.1}% ({})\nsystem-memory: {} / {} ({:.1}%, {})\nprocesses: {}\nnode-processes: {}\nnode-cpu-total: {:.1}%\nnode-memory-total: {}\nmissing-processes: {}\n",
            self.status_label(),
            self.captured_at_unix,
            self.system_cpu_usage_percent(),
            self.system.cpu_pressure().label(),
            format_bytes(self.system.used_memory_bytes),
            format_bytes(self.system.total_memory_bytes),
            self.system_memory_usage_percent(),
            self.system.memory_pressure().label(),
            self.system.process_count,
            self.node_processes.len(),
            self.total_node_cpu_usage_percent(),
            format_bytes(self.total_node_memory_bytes()),
            self.missing_processes.len(),
        )
    }

    fn push_node_process_text(&self, text: &mut String) {
        for process in &self.node_processes {
            text.push_str(&node_process_text(process));
        }
    }

    fn push_missing_process_text(&self, text: &mut String) {
        for missing in &self.missing_processes {
            text.push_str(&format!(
                "missing: {} | pid {} | node-id {}\n",
                missing.node_name, missing.pid, missing.node_id
            ));
        }
    }
}

fn node_process_text(process: &NodeProcessMetrics) -> String {
    format!(
        "node: {} | pid {} | cpu {:.1}% | memory {} | uptime {}s | status {}\n",
        process.node_name,
        process.pid,
        MetricsSnapshot::node_cpu_usage_percent(process),
        format_bytes(process.memory_bytes),
        process.run_time_seconds,
        process.status,
    )
}
