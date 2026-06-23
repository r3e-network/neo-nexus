use super::*;

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

impl NeoNexusApp {
    /// Persists the active sub-tab of each dense workspace page when it changes,
    /// so the workspace reopens with the operator's last selection. Called once
    /// per frame; each SQLite write only happens on an actual change, tracked
    /// through the paired `persisted_*_section` shadow fields.
    pub(in crate::app) fn persist_active_sections_if_changed(&mut self) {
        persist_section(
            &self.repository,
            &mut self.notice,
            self.operations_section,
            &mut self.persisted_operations_section,
            KEY_OPERATIONS,
            OperationsSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.notice,
            self.settings_section,
            &mut self.persisted_settings_section,
            KEY_SETTINGS,
            SettingsSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.notice,
            self.runtimes_section,
            &mut self.persisted_runtimes_section,
            KEY_RUNTIMES,
            RuntimesSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.notice,
            self.snapshots_section,
            &mut self.persisted_snapshots_section,
            KEY_SNAPSHOTS,
            SnapshotsSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.notice,
            self.monitor_section,
            &mut self.persisted_monitor_section,
            KEY_MONITOR,
            MonitorSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.notice,
            self.federation_section,
            &mut self.persisted_federation_section,
            KEY_FEDERATION,
            FederationSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.notice,
            self.roles_section,
            &mut self.persisted_roles_section,
            KEY_ROLES,
            RolesSection::persist_key,
        );
    }
}

/// Writes the current section to the workspace settings only when it differs
/// from its shadow, then advances the shadow so the next frame is a cheap
/// comparison. `key_of` resolves the concrete enum value to its stable persist
/// key, passed as a function reference so the call stays generic without
/// monomorphising over the persistence logic.
fn persist_section<T: Copy + PartialEq>(
    repository: &Repository,
    notice: &mut Option<String>,
    current: T,
    persisted: &mut T,
    setting_key: &str,
    key_of: fn(T) -> &'static str,
) {
    if current == *persisted {
        return;
    }
    *persisted = current;
    if let Err(error) = repository.save_workspace_section(setting_key, key_of(current)) {
        *notice = Some(format!("Workspace sub-tab not saved: {error}"));
    }
}
