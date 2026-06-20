use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct SidecarExecutionPolicyFinding {
    pub signer_label: String,
    pub process_id: String,
    pub binary_path: PathBuf,
    pub reason: String,
}

impl SidecarExecutionPolicyFinding {
    pub(in crate::app) fn summary(&self) -> String {
        format!(
            "{} ({}) uses {}: {}",
            self.signer_label,
            self.process_id,
            self.binary_path.display(),
            self.reason
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum SidecarEndpointHealthStatus {
    Reachable,
    Unreachable,
    MissingEndpoint,
}

impl SidecarEndpointHealthStatus {
    pub(in crate::app::sidecar_health) fn label(self) -> &'static str {
        match self {
            Self::Reachable => "reachable",
            Self::Unreachable => "unreachable",
            Self::MissingEndpoint => "missing-endpoint",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct SidecarEndpointHealthResult {
    pub signer_label: String,
    pub process_id: String,
    pub endpoint: String,
    pub status: SidecarEndpointHealthStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct SidecarEndpointHealthReport {
    pub checked_at_unix: u64,
    pub sidecar_count: usize,
    pub endpoint_count: usize,
    pub reachable_count: usize,
    pub unreachable_count: usize,
    pub missing_endpoint_count: usize,
    pub results: Vec<SidecarEndpointHealthResult>,
}

impl SidecarEndpointHealthReport {
    pub(in crate::app) fn empty() -> Self {
        Self {
            checked_at_unix: 0,
            sidecar_count: 0,
            endpoint_count: 0,
            reachable_count: 0,
            unreachable_count: 0,
            missing_endpoint_count: 0,
            results: Vec::new(),
        }
    }

    pub(in crate::app) fn from_results(
        checked_at_unix: u64,
        sidecar_count: usize,
        results: Vec<SidecarEndpointHealthResult>,
    ) -> Self {
        let endpoint_count = results
            .iter()
            .filter(|result| result.status != SidecarEndpointHealthStatus::MissingEndpoint)
            .count();
        let reachable_count = results
            .iter()
            .filter(|result| result.status == SidecarEndpointHealthStatus::Reachable)
            .count();
        let unreachable_count = results
            .iter()
            .filter(|result| result.status == SidecarEndpointHealthStatus::Unreachable)
            .count();
        let missing_endpoint_count = results
            .iter()
            .filter(|result| result.status == SidecarEndpointHealthStatus::MissingEndpoint)
            .count();
        Self {
            checked_at_unix,
            sidecar_count,
            endpoint_count,
            reachable_count,
            unreachable_count,
            missing_endpoint_count,
            results,
        }
    }
}
