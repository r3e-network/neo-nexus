use anyhow::Result;
use serde::Serialize;

use crate::metrics::MetricsSnapshot;

use super::super::json_text;

#[derive(Debug, Serialize)]
struct WorkspaceMetricsJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    snapshot: &'a MetricsSnapshot,
}

pub(in crate::cli) fn workspace_metrics_json_text(snapshot: &MetricsSnapshot) -> Result<String> {
    json_text(&WorkspaceMetricsJsonReport {
        schema_version: 1,
        status: snapshot.status_label(),
        success: snapshot.is_success(),
        snapshot,
    })
}
