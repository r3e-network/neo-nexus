//! Persisted in-page section tabs for dense multi-surface workspaces.

use crate::app::views::{
    FederationSection, MonitorSection, RolesSection, RuntimesSection, SettingsSection,
    SnapshotsSection,
};

#[derive(Debug, Clone, Copy)]
pub(in crate::app) struct WorkspaceSections {
    pub(in crate::app) settings: SettingsSection,
    pub(in crate::app) persisted_settings: SettingsSection,
    pub(in crate::app) runtimes: RuntimesSection,
    pub(in crate::app) persisted_runtimes: RuntimesSection,
    pub(in crate::app) snapshots: SnapshotsSection,
    pub(in crate::app) persisted_snapshots: SnapshotsSection,
    pub(in crate::app) monitor: MonitorSection,
    pub(in crate::app) persisted_monitor: MonitorSection,
    pub(in crate::app) federation: FederationSection,
    pub(in crate::app) persisted_federation: FederationSection,
    pub(in crate::app) roles: RolesSection,
    pub(in crate::app) persisted_roles: RolesSection,
}

impl WorkspaceSections {
    pub(in crate::app) fn new(
        settings: SettingsSection,
        runtimes: RuntimesSection,
        snapshots: SnapshotsSection,
        monitor: MonitorSection,
        federation: FederationSection,
        roles: RolesSection,
    ) -> Self {
        Self {
            settings,
            persisted_settings: settings,
            runtimes,
            persisted_runtimes: runtimes,
            snapshots,
            persisted_snapshots: snapshots,
            monitor,
            persisted_monitor: monitor,
            federation,
            persisted_federation: federation,
            roles,
            persisted_roles: roles,
        }
    }
}
