mod families;
mod labels;
mod text;

use super::types::MetricsSnapshot;

pub(super) fn snapshot_to_text(snapshot: &MetricsSnapshot) -> String {
    let mut output = String::new();
    families::push_workspace_metrics(&mut output, snapshot);
    families::push_system_metrics(&mut output, snapshot);
    families::push_node_process_metrics(&mut output, snapshot);
    families::push_missing_process_metrics(&mut output, snapshot);
    output
}
