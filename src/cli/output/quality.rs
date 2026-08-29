use anyhow::Result;
use serde::Serialize;

use crate::core::quality::{CiPolicyReport, SourcePurityReport, SourceQualityReport};

use super::json_text;

#[derive(Debug, Serialize)]
struct SourcePurityJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    report: &'a SourcePurityReport,
}

#[derive(Debug, Serialize)]
struct SourceQualityJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    report: &'a SourceQualityReport,
}

#[derive(Debug, Serialize)]
struct CiPolicyJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    success: bool,
    report: &'a CiPolicyReport,
}

pub(in crate::cli) fn source_purity_json_text(report: &SourcePurityReport) -> Result<String> {
    json_text(&SourcePurityJsonReport {
        schema_version: 1,
        status: report.status_label(),
        success: report.is_success(),
        report,
    })
}

pub(in crate::cli) fn source_quality_json_text(report: &SourceQualityReport) -> Result<String> {
    json_text(&SourceQualityJsonReport {
        schema_version: 1,
        status: report.status_label(),
        success: report.is_success(),
        report,
    })
}

pub(in crate::cli) fn ci_policy_json_text(report: &CiPolicyReport) -> Result<String> {
    json_text(&CiPolicyJsonReport {
        schema_version: 1,
        status: report.status_label(),
        success: report.is_success(),
        report,
    })
}
