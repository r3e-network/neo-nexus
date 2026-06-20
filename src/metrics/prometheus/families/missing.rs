use super::{
    super::{
        labels,
        text::{push_header, push_sample},
    },
    MetricsSnapshot,
};

pub(in crate::metrics::prometheus) fn push_missing_process_metrics(
    output: &mut String,
    snapshot: &MetricsSnapshot,
) {
    push_header(
        output,
        "neonexus_node_missing_process",
        "Managed node marked running whose process no longer exists.",
    );
    for missing in &snapshot.missing_processes {
        let labels = labels::missing_process(missing);
        push_sample(output, "neonexus_node_missing_process", &labels, 1);
    }
}
