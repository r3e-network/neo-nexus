#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
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
    pub const ALL: [Self; 14] = [
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
            Self::Summary => "Home",
            Self::Operations => "Operations",
            Self::Monitor => "Monitor",
            Self::Alerts => "Alerts",
            Self::Federation => "Network",
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

    /// Whether the node inventory column is relevant to this page. Workspace-
    /// level pages (wallets, settings, alerts, …) hide it so they get the full
    /// width instead of an irrelevant node list.
    pub(super) fn shows_inventory(self) -> bool {
        matches!(
            self,
            Self::Summary
                | Self::Operations
                | Self::Monitor
                | Self::Nodes
                | Self::Plugins
                | Self::Config
                | Self::Logs
        )
    }

    pub(super) fn title(self) -> &'static str {
        match self {
            Self::Summary => "Home",
            Self::Operations => "Operations",
            Self::Monitor => "Resource Monitor",
            Self::Alerts => "Alert Routing",
            Self::Federation => "Network",
            Self::Settings => "Settings",
            Self::Runtimes => "Runtime Manager",
            Self::Wallets => "Wallet Profiles",
            Self::Nodes => "Nodes",
            Self::Roles => "Role Planner",
            Self::Snapshots => "Fast Sync",
            Self::Plugins => "Plugin Manager",
            Self::Config => "Configuration",
            Self::Logs => "Runtime Logs",
        }
    }

    /// Stable identifier used to persist the active view across restarts.
    /// Independent of `label()` so display text can change without breaking the
    /// saved preference.
    pub fn persist_key(self) -> &'static str {
        match self {
            Self::Summary => "summary",
            Self::Operations => "operations",
            Self::Monitor => "monitor",
            Self::Alerts => "alerts",
            Self::Federation => "federation",
            Self::Settings => "settings",
            Self::Runtimes => "runtimes",
            Self::Wallets => "wallets",
            Self::Nodes => "nodes",
            Self::Roles => "roles",
            Self::Snapshots => "snapshots",
            Self::Plugins => "plugins",
            Self::Config => "config",
            Self::Logs => "logs",
        }
    }

    pub fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|view| view.persist_key() == key)
    }

    pub(super) fn subtitle(self) -> &'static str {
        match self {
            Self::Summary => "Fleet health, selection context, and operator next steps.",
            Self::Operations => "Readiness checks, port safety, and operator action queue.",
            Self::Monitor => "System pressure and managed node process telemetry.",
            Self::Alerts => "Route critical node and Federation events to operator endpoints.",
            Self::Federation => "Remote endpoints, private networks, and wallet profiles.",
            Self::Settings => "Native runtime policy, alerts, and local workspace paths.",
            Self::Runtimes => "Install verified runtimes, upgrades, and fast sync snapshots.",
            Self::Wallets => "Import encrypted Neo wallet metadata for signer operations.",
            Self::Nodes => "Define, configure, and operate a selected local Neo node.",
            Self::Roles => "Apply runtime roles and plan private-network topology.",
            Self::Snapshots => "Register, verify, and cache fast sync snapshots.",
            Self::Plugins => "Enable runtime capabilities for the selected node.",
            Self::Config => "Inspect generated settings without leaving the application.",
            Self::Logs => "Inspect captured process output in a fixed native workspace.",
        }
    }

    /// The six primary navigation destinations shown in the sidebar.
    pub(super) const PRIMARY: [Self; 6] = [
        Self::Summary,
        Self::Nodes,
        Self::Runtimes,
        Self::Federation,
        Self::Operations,
        Self::Settings,
    ];

    /// Map any view (including legacy top-level tools) onto the primary nav
    /// item that should appear selected in the sidebar.
    pub(super) fn primary_nav(self) -> Self {
        match self {
            Self::Summary => Self::Summary,
            Self::Nodes | Self::Config | Self::Logs | Self::Plugins | Self::Monitor => Self::Nodes,
            Self::Runtimes | Self::Snapshots => Self::Runtimes,
            Self::Federation | Self::Roles | Self::Wallets => Self::Federation,
            Self::Operations => Self::Operations,
            Self::Settings | Self::Alerts => Self::Settings,
        }
    }

    /// Whether this view is one of the six primary sidebar destinations.
    #[allow(dead_code)] // used by unit tests and upcoming command palette filtering
    pub(super) fn is_primary(self) -> bool {
        Self::PRIMARY.contains(&self)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/app/view/tests.rs"]
mod tests;
