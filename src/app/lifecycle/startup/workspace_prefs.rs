mod loaders;

use super::super::super::*;

use loaders::{
    first_notice, load_density, load_inspector_visible, load_section, load_theme, load_view,
};

// Workspace sub-tab persistence keys. These mirror the private constants in
// repository::settings_keys; the app names them by value because the
// architectural boundary keeps repository symbols out of the app's imports.
const KEY_OPERATIONS: &str = "workspace.section.operations";
const KEY_SETTINGS: &str = "workspace.section.settings";
const KEY_RUNTIMES: &str = "workspace.section.runtimes";
const KEY_SNAPSHOTS: &str = "workspace.section.snapshots";
const KEY_MONITOR: &str = "workspace.section.monitor";
const KEY_FEDERATION: &str = "workspace.section.federation";
const KEY_ROLES: &str = "workspace.section.roles";
const KEY_PRIVATE_NETWORK: &str = "workspace.section.private-network";
const KEY_NODES: &str = "workspace.section.nodes";

/// Host-level workspace preferences that an operator tunes once and expects to
/// survive a restart: theme, density, inspector layout, the active workspace
/// view, and the active sub-tab of each dense page.
pub(super) struct StartupWorkspacePrefs {
    pub(super) theme: Theme,
    pub(super) density: UiDensity,
    pub(super) inspector_visible: bool,
    pub(super) view: View,
    pub(super) nodes_tab: NodeWorkspaceTab,
    pub(super) operations: OperationsSection,
    pub(super) settings: SettingsSection,
    pub(super) runtimes: RuntimesSection,
    pub(super) snapshots: SnapshotsSection,
    pub(super) monitor: MonitorSection,
    pub(super) federation: FederationSection,
    pub(super) roles: RolesSection,
    pub(super) private_network: PrivateNetworkSection,
    pub(super) notice: Option<String>,
}

impl StartupWorkspacePrefs {
    pub(super) fn load(repository: &Repository) -> Self {
        let (theme, theme_notice) = load_theme(repository);
        let (density, density_notice) = load_density(repository);
        let (inspector_visible, inspector_notice) = load_inspector_visible(repository);
        let (view, view_notice) = load_view(repository);
        let (nodes_tab, nodes_notice) = load_section(
            repository,
            KEY_NODES,
            NodeWorkspaceTab::from_persist_key,
            NodeWorkspaceTab::Studio,
        );
        let (operations, operations_notice) = load_section(
            repository,
            KEY_OPERATIONS,
            OperationsSection::from_persist_key,
            OperationsSection::Readiness,
        );
        let (settings, settings_notice) = load_section(
            repository,
            KEY_SETTINGS,
            SettingsSection::from_persist_key,
            SettingsSection::Watchdog,
        );
        let (runtimes, runtimes_notice) = load_section(
            repository,
            KEY_RUNTIMES,
            RuntimesSection::from_persist_key,
            RuntimesSection::Install,
        );
        let (snapshots, snapshots_notice) = load_section(
            repository,
            KEY_SNAPSHOTS,
            SnapshotsSection::from_persist_key,
            SnapshotsSection::Manifest,
        );
        let (monitor, monitor_notice) = load_section(
            repository,
            KEY_MONITOR,
            MonitorSection::from_persist_key,
            MonitorSection::Pressure,
        );
        let (federation, federation_notice) = load_section(
            repository,
            KEY_FEDERATION,
            FederationSection::from_persist_key,
            FederationSection::Profiles,
        );
        let (roles, roles_notice) = load_section(
            repository,
            KEY_ROLES,
            RolesSection::from_persist_key,
            RolesSection::Presets,
        );

        let (private_network, private_network_notice) = load_section(
            repository,
            KEY_PRIVATE_NETWORK,
            PrivateNetworkSection::from_persist_key,
            PrivateNetworkSection::Plan,
        );

        Self {
            theme,
            density,
            inspector_visible,
            view,
            nodes_tab,
            operations,
            settings,
            runtimes,
            snapshots,
            monitor,
            federation,
            roles,
            private_network,
            notice: first_notice([
                theme_notice,
                density_notice,
                inspector_notice,
                view_notice,
                nodes_notice,
                operations_notice,
                settings_notice,
                runtimes_notice,
                snapshots_notice,
                monitor_notice,
                federation_notice,
                roles_notice,
                private_network_notice,
            ]),
        }
    }
}
