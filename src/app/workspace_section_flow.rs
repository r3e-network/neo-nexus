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
const KEY_NODES: &str = "workspace.section.nodes";

impl NeoNexusApp {
    /// Persists the active sub-tab of each dense workspace page when it changes,
    /// so the workspace reopens with the operator's last selection. Called once
    /// per frame; each SQLite write only happens on an actual change, tracked
    /// through the paired `persisted_*_section` shadow fields.
    pub(in crate::app) fn persist_active_sections_if_changed(&mut self) {
        persist_section(
            &self.repository,
            &mut self.session.notice,
            self.session.node_workspace_tab,
            &mut self.session.persisted_node_workspace_tab,
            KEY_NODES,
            NodeWorkspaceTab::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.session.notice,
            self.operations_ui.section,
            &mut self.operations_ui.persisted_section,
            KEY_OPERATIONS,
            OperationsSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.session.notice,
            self.sections.settings,
            &mut self.sections.persisted_settings,
            KEY_SETTINGS,
            SettingsSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.session.notice,
            self.sections.runtimes,
            &mut self.sections.persisted_runtimes,
            KEY_RUNTIMES,
            RuntimesSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.session.notice,
            self.sections.snapshots,
            &mut self.sections.persisted_snapshots,
            KEY_SNAPSHOTS,
            SnapshotsSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.session.notice,
            self.sections.monitor,
            &mut self.sections.persisted_monitor,
            KEY_MONITOR,
            MonitorSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.session.notice,
            self.sections.federation,
            &mut self.sections.persisted_federation,
            KEY_FEDERATION,
            FederationSection::persist_key,
        );
        persist_section(
            &self.repository,
            &mut self.session.notice,
            self.sections.roles,
            &mut self.sections.persisted_roles,
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
