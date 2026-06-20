use anyhow::Result;
use serde::Serialize;

use crate::private_network::PrivateNetworkLaunchPackSidecarReport;

use super::json_text;

#[derive(Debug, Serialize)]
struct LaunchPackSidecarsJsonReport<'a> {
    schema_version: u32,
    status: &'static str,
    report: &'a PrivateNetworkLaunchPackSidecarReport,
}

pub(in crate::cli) fn launch_pack_sidecars_json_text(
    report: &PrivateNetworkLaunchPackSidecarReport,
) -> Result<String> {
    json_text(&LaunchPackSidecarsJsonReport {
        schema_version: 1,
        status: report.status_label(),
        report,
    })
}
