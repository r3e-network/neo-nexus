use super::{
    super::{
        labels,
        text::{push_header, push_sample},
    },
    MetricsSnapshot,
};

pub(in crate::metrics::prometheus) fn push_node_process_metrics(
    output: &mut String,
    snapshot: &MetricsSnapshot,
) {
    push_header(
        output,
        "neonexus_node_process_cpu_usage_percent",
        "Managed node process CPU usage percentage.",
    );
    push_header(
        output,
        "neonexus_node_process_memory_bytes",
        "Managed node process resident memory in bytes.",
    );
    push_header(
        output,
        "neonexus_node_process_virtual_memory_bytes",
        "Managed node process virtual memory in bytes.",
    );
    push_header(
        output,
        "neonexus_node_process_uptime_seconds",
        "Managed node process uptime in seconds.",
    );
    for process in &snapshot.node_processes {
        let labels = labels::node_process(process);
        push_sample(
            output,
            "neonexus_node_process_cpu_usage_percent",
            &labels,
            MetricsSnapshot::node_cpu_usage_percent(process),
        );
        push_sample(
            output,
            "neonexus_node_process_memory_bytes",
            &labels,
            process.memory_bytes,
        );
        push_sample(
            output,
            "neonexus_node_process_virtual_memory_bytes",
            &labels,
            process.virtual_memory_bytes,
        );
        push_sample(
            output,
            "neonexus_node_process_uptime_seconds",
            &labels,
            process.run_time_seconds,
        );
    }
}
