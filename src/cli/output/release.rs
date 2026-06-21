use anyhow::Result;
use serde::Serialize;

use crate::core::distribution::ReleasePackageVerification;

use super::json_text;

#[derive(Debug, Serialize)]
struct ReleasePackageVerificationJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    verification: &'a ReleasePackageVerification,
}

#[derive(Debug, Serialize)]
struct ReleasePackageVerificationFailureJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    message: &'a str,
}

pub(in crate::cli) fn release_package_verification_json_text(
    verification: &ReleasePackageVerification,
) -> Result<String> {
    json_text(&ReleasePackageVerificationJsonReport {
        schema_version: 1,
        status: "ok",
        verification,
    })
}

pub(in crate::cli) fn release_package_verification_failure_json_text(
    message: &str,
) -> Result<String> {
    json_text(&ReleasePackageVerificationFailureJsonReport {
        schema_version: 1,
        status: "failed",
        message,
    })
}
