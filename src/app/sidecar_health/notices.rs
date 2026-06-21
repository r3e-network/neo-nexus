use crate::app::domain::{EventSeverity, PrivateNetworkLaunchPackValidation};

use super::{SidecarEndpointHealthReport, SidecarEndpointHealthStatus};

pub(in crate::app) fn sidecar_health_notice(report: &SidecarEndpointHealthReport) -> String {
    let first_issue = report.results.iter().find(|result| {
        result.status == SidecarEndpointHealthStatus::Unreachable
            || result.status == SidecarEndpointHealthStatus::MissingEndpoint
    });
    let first_label = report
        .results
        .first()
        .map_or("none", |result| result.signer_label.as_str());
    let first_endpoint = report
        .results
        .first()
        .map_or("-", |result| result.endpoint.as_str());
    let status_detail = first_issue.map_or_else(
        || format!("first signer {first_label} at {first_endpoint}"),
        |result| {
            format!(
                "{} ({}) {} at {}: {}",
                result.signer_label,
                result.process_id,
                result.status.label(),
                result.endpoint,
                result.message
            )
        },
    );
    format!(
        "{}/{} signer endpoints reachable across {} sidecars; {} missing; {}",
        report.reachable_count,
        report.endpoint_count,
        report.sidecar_count,
        report.missing_endpoint_count,
        status_detail
    )
}

pub(in crate::app) fn private_launch_pack_validation_notice(
    validation: &PrivateNetworkLaunchPackValidation,
) -> String {
    let status = if validation.failed_count == 0 {
        if validation.warning_count == 0 {
            "ready"
        } else {
            "ready with warnings"
        }
    } else {
        "blocked"
    };
    format!(
        "launch pack validation {status}: {} passed, {} warnings, {} failed",
        validation.passed_count, validation.warning_count, validation.failed_count
    )
}

pub(in crate::app) fn launch_pack_validation_severity(
    validation: &PrivateNetworkLaunchPackValidation,
) -> EventSeverity {
    if validation.failed_count > 0 {
        EventSeverity::Critical
    } else if validation.warning_count > 0 {
        EventSeverity::Warning
    } else {
        EventSeverity::Info
    }
}
