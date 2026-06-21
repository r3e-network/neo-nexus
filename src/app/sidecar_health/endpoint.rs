use std::time::Duration;

use crate::app::domain::CommitteeSidecarProcess;

use super::{SidecarEndpointHealthResult, SidecarEndpointHealthStatus};

pub(in crate::app) fn probe_sidecar_endpoint_health(
    sidecar: &CommitteeSidecarProcess,
    timeout: Duration,
) -> SidecarEndpointHealthResult {
    let Some(endpoint) = sidecar.signer_endpoint.as_deref() else {
        return SidecarEndpointHealthResult {
            signer_label: sidecar.signer_label.clone(),
            process_id: sidecar.process.id.clone(),
            endpoint: "-".to_string(),
            status: SidecarEndpointHealthStatus::MissingEndpoint,
            message: "signer endpoint is not configured".to_string(),
        };
    };

    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();
    let response = agent.get(endpoint).call();
    match response {
        Ok(response) => SidecarEndpointHealthResult {
            signer_label: sidecar.signer_label.clone(),
            process_id: sidecar.process.id.clone(),
            endpoint: endpoint.to_string(),
            status: SidecarEndpointHealthStatus::Reachable,
            message: format!("HTTP {}", response.status()),
        },
        Err(ureq::Error::Status(status, _response)) => SidecarEndpointHealthResult {
            signer_label: sidecar.signer_label.clone(),
            process_id: sidecar.process.id.clone(),
            endpoint: endpoint.to_string(),
            status: SidecarEndpointHealthStatus::Reachable,
            message: format!("HTTP {status}"),
        },
        Err(error) => SidecarEndpointHealthResult {
            signer_label: sidecar.signer_label.clone(),
            process_id: sidecar.process.id.clone(),
            endpoint: endpoint.to_string(),
            status: SidecarEndpointHealthStatus::Unreachable,
            message: error.to_string(),
        },
    }
}
