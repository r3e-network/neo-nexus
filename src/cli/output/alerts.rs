use anyhow::Result;
use serde::Serialize;

use crate::alerts::AlertPreviewReport;

use super::json_text;

#[derive(Serialize)]
struct AlertPreviewJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    report: &'a AlertPreviewReport,
}

pub(in crate::cli) fn alert_preview_json_text(report: &AlertPreviewReport) -> Result<String> {
    json_text(&AlertPreviewJsonReport {
        schema_version: 1,
        status: report.status,
        success: true,
        report,
    })
}
