use anyhow::Result;
use serde::Serialize;

use crate::core::runtime::RuntimeSmokeReport;

use super::super::json_text;

#[derive(Debug, Serialize)]
struct RuntimeSmokeJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    report: &'a RuntimeSmokeReport,
}

pub(in crate::cli) fn runtime_smoke_json_text(report: &RuntimeSmokeReport) -> Result<String> {
    json_text(&RuntimeSmokeJsonReport {
        schema_version: 1,
        status: report.status_label(),
        success: report.status.is_success(),
        report,
    })
}
