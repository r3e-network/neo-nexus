use anyhow::Result;
use serde::Serialize;

use crate::metrics::MetricsSnapshot;

#[derive(Debug, Serialize)]
struct SupportBundleMetricsJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    snapshot: &'a MetricsSnapshot,
}

pub(in crate::support_bundle) fn support_metrics_json_text(
    snapshot: &MetricsSnapshot,
) -> Result<String> {
    let report = SupportBundleMetricsJsonReport {
        schema_version: 1,
        status: snapshot.status_label(),
        success: snapshot.is_success(),
        snapshot,
    };
    Ok(format!("{}\n", serde_json::to_string_pretty(&report)?))
}
