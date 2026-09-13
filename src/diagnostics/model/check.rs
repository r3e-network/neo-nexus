use super::CheckSeverity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticResolution {
    ConfigWorkspace,
    Logs,
    Monitor,
    NodeStudio,
    Operations,
    PluginManager,
    RolePlanner,
    RuntimeManager,
    WalletProfiles,
}

impl DiagnosticResolution {
    pub const ALL: [Self; 9] = [
        Self::ConfigWorkspace,
        Self::Logs,
        Self::Monitor,
        Self::NodeStudio,
        Self::Operations,
        Self::PluginManager,
        Self::RolePlanner,
        Self::RuntimeManager,
        Self::WalletProfiles,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::ConfigWorkspace => "config",
            Self::Logs => "logs",
            Self::Monitor => "monitor",
            Self::NodeStudio => "node-studio",
            Self::Operations => "operations",
            Self::PluginManager => "plugins",
            Self::RolePlanner => "roles",
            Self::RuntimeManager => "runtime-manager",
            Self::WalletProfiles => "wallets",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::ConfigWorkspace => "Config",
            Self::Logs => "Logs",
            Self::Monitor => "Monitor",
            Self::NodeStudio => "Node Studio",
            Self::Operations => "Operations",
            Self::PluginManager => "Plugins",
            Self::RolePlanner => "Roles",
            Self::RuntimeManager => "Runtimes",
            Self::WalletProfiles => "Wallets",
        }
    }

    pub fn action_label(self) -> &'static str {
        match self {
            Self::ConfigWorkspace => "Open Config",
            Self::Logs => "Open Logs",
            Self::Monitor => "Open Monitor",
            Self::NodeStudio => "Open Node Studio",
            Self::Operations => "Open Operations",
            Self::PluginManager => "Open Plugins",
            Self::RolePlanner => "Open Roles",
            Self::RuntimeManager => "Open Runtimes",
            Self::WalletProfiles => "Open Wallets",
        }
    }

    pub fn hint(self) -> &'static str {
        match self {
            Self::ConfigWorkspace => "Inspect generated config and managed configuration evidence.",
            Self::Logs => "Inspect captured process output and runtime failure diagnosis.",
            Self::Monitor => "Inspect process state, telemetry, and reconciliation actions.",
            Self::NodeStudio => "Edit node definition, runtime args, ports, and storage settings.",
            Self::Operations => "Review readiness evidence and workspace safety actions.",
            Self::PluginManager => "Enable, disable, or install runtime plugin capabilities.",
            Self::RolePlanner => "Review role presets, signer references, and launch-pack posture.",
            Self::RuntimeManager => "Install, verify, or apply node runtime binaries.",
            Self::WalletProfiles => "Review encrypted wallet metadata and signer profile links.",
        }
    }

    /// Where an operator goes to resolve a finding on this node.
    ///
    /// The resolution has always known which surface fixes each finding, and
    /// the OpsCenter has always labelled its button from it — "Open Plugins",
    /// "Open Runtimes" — while sending every one of them to the instance page.
    /// A button that names a destination has to go there.
    ///
    /// `encode` percent-encodes a query value; callers pass their own so this
    /// stays free of the web layer.
    pub fn href(self, node_id: &str, encode: impl Fn(&str) -> String) -> String {
        let node = encode(node_id);
        match self {
            // Pages that select an instance from a query parameter.
            Self::PluginManager => format!("/plugins?node={node}"),
            Self::RolePlanner => format!("/roles?node={node}"),
            Self::Logs => format!("/logs?node={node}"),
            // The instance's own editor.
            Self::NodeStudio => format!("/nodes/{node}/edit"),
            // Fleet-wide surfaces with nothing per-instance to select.
            Self::ConfigWorkspace => "/config".to_string(),
            Self::Monitor => "/monitor".to_string(),
            Self::Operations => "/operations".to_string(),
            Self::RuntimeManager => "/runtimes".to_string(),
            Self::WalletProfiles => "/wallets".to_string(),
        }
    }

    pub fn matches_query(self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        [self.key(), self.label(), self.action_label(), self.hint()]
            .into_iter()
            .any(|value| value.to_lowercase().contains(&query))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCheck {
    pub severity: CheckSeverity,
    pub title: &'static str,
    pub detail: String,
    pub resolution: DiagnosticResolution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticCheckKey {
    pub severity: CheckSeverity,
    pub title: String,
    pub detail: String,
    pub resolution: DiagnosticResolution,
}

impl DiagnosticCheck {
    pub fn new(
        severity: CheckSeverity,
        title: &'static str,
        detail: impl Into<String>,
        resolution: DiagnosticResolution,
    ) -> Self {
        Self {
            severity,
            title,
            detail: detail.into(),
            resolution,
        }
    }

    pub fn key(&self) -> DiagnosticCheckKey {
        DiagnosticCheckKey {
            severity: self.severity,
            title: self.title.to_string(),
            detail: self.detail.clone(),
            resolution: self.resolution,
        }
    }
}

impl DiagnosticCheckKey {
    pub fn matches(&self, check: &DiagnosticCheck) -> bool {
        self.severity == check.severity
            && self.title == check.title
            && self.detail == check.detail
            && self.resolution == check.resolution
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/diagnostics/model/check/tests.rs"]
mod tests;
