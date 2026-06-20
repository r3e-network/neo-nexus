use anyhow::Result;
use serde::Serialize;

use crate::rpc_health::{RpcHealthReport, RpcHealthStatus};

use super::super::json_text;

#[derive(Debug, Serialize)]
struct RpcHealthJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    report: &'a RpcHealthReport,
}

pub(in crate::cli) fn rpc_health_json_text(report: &RpcHealthReport) -> Result<String> {
    json_text(&RpcHealthJsonReport {
        schema_version: 1,
        status: report.status_label(),
        success: report.status == RpcHealthStatus::Healthy,
        report,
    })
}
