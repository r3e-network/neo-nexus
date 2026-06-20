use super::{super::text::push_gauge, MetricsSnapshot};

pub(in crate::metrics::prometheus) fn push_system_metrics(
    output: &mut String,
    snapshot: &MetricsSnapshot,
) {
    push_gauge(
        output,
        "neonexus_system_cpu_usage_percent",
        "Host CPU usage percentage observed by NeoNexus.",
        &[],
        snapshot.system_cpu_usage_percent(),
    );
    push_gauge(
        output,
        "neonexus_system_memory_usage_percent",
        "Host memory usage percentage observed by NeoNexus.",
        &[],
        snapshot.system_memory_usage_percent(),
    );
    push_gauge(
        output,
        "neonexus_system_total_memory_bytes",
        "Host total memory in bytes observed by NeoNexus.",
        &[],
        snapshot.system.total_memory_bytes,
    );
    push_gauge(
        output,
        "neonexus_system_used_memory_bytes",
        "Host used memory in bytes observed by NeoNexus.",
        &[],
        snapshot.system.used_memory_bytes,
    );
    push_gauge(
        output,
        "neonexus_system_available_memory_bytes",
        "Host available memory in bytes observed by NeoNexus.",
        &[],
        snapshot.system.available_memory_bytes,
    );
    push_gauge(
        output,
        "neonexus_system_processes",
        "Host process count observed by NeoNexus.",
        &[],
        snapshot.system.process_count,
    );
}
