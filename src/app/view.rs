#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum View {
    Summary,
    Operations,
    Monitor,
    Alerts,
    Federation,
    Settings,
    Runtimes,
    Wallets,
    Nodes,
    Roles,
    Snapshots,
    Plugins,
    Config,
    Logs,
}

impl View {
    pub(super) const ALL: [Self; 14] = [
        Self::Summary,
        Self::Operations,
        Self::Monitor,
        Self::Alerts,
        Self::Federation,
        Self::Settings,
        Self::Runtimes,
        Self::Wallets,
        Self::Nodes,
        Self::Roles,
        Self::Snapshots,
        Self::Plugins,
        Self::Config,
        Self::Logs,
    ];

    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Operations => "Operations",
            Self::Monitor => "Monitor",
            Self::Alerts => "Alerts",
            Self::Federation => "Federation",
            Self::Settings => "Settings",
            Self::Runtimes => "Runtimes",
            Self::Wallets => "Wallets",
            Self::Nodes => "Nodes",
            Self::Roles => "Roles",
            Self::Snapshots => "Sync",
            Self::Plugins => "Plugins",
            Self::Config => "Config",
            Self::Logs => "Logs",
        }
    }

    pub(super) fn short_label(self) -> &'static str {
        match self {
            Self::Summary => "Sum",
            Self::Operations => "Ops",
            Self::Monitor => "Mon",
            Self::Alerts => "Alert",
            Self::Federation => "Fed",
            Self::Settings => "Set",
            Self::Runtimes => "Run",
            Self::Wallets => "Keys",
            Self::Nodes => "Nodes",
            Self::Roles => "Role",
            Self::Snapshots => "Sync",
            Self::Plugins => "Plug",
            Self::Config => "Cfg",
            Self::Logs => "Logs",
        }
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Summary => "Overview",
            Self::Operations => "Operations",
            Self::Monitor => "Resource Monitor",
            Self::Alerts => "Alert Routing",
            Self::Federation => "Federation",
            Self::Settings => "Settings",
            Self::Runtimes => "Runtime Manager",
            Self::Wallets => "Wallet Profiles",
            Self::Nodes => "Node Studio",
            Self::Roles => "Role Planner",
            Self::Snapshots => "Fast Sync",
            Self::Plugins => "Plugin Manager",
            Self::Config => "Configuration",
            Self::Logs => "Runtime Logs",
        }
    }

    pub(super) fn subtitle(self) -> &'static str {
        match self {
            Self::Summary => "Fleet health, lifecycle posture, and native workspace state.",
            Self::Operations => "Readiness checks, port safety, and operator action queue.",
            Self::Monitor => "System pressure and managed node process telemetry.",
            Self::Alerts => "Route critical node and Federation events to operator endpoints.",
            Self::Federation => "Remote NeoNexus public endpoint profiles and probes.",
            Self::Settings => "Native runtime policy and local workspace paths.",
            Self::Runtimes => "Install verified local node runtimes and apply upgrades.",
            Self::Wallets => "Import encrypted Neo wallet metadata for signer operations.",
            Self::Nodes => "Create and tune local Neo node definitions.",
            Self::Roles => "Apply runtime roles and plan private-network topology.",
            Self::Snapshots => "Register, verify, and cache fast sync snapshots.",
            Self::Plugins => "Enable runtime capabilities for the selected node.",
            Self::Config => "Inspect generated settings without leaving the application.",
            Self::Logs => "Inspect captured process output in a fixed native workspace.",
        }
    }
}

#[cfg(test)]
mod tests;
