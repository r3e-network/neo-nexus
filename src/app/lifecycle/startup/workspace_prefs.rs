use super::super::super::*;

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

/// Host-level workspace preferences that an operator tunes once and expects to
/// survive a restart: theme, inspector layout, the active workspace view, and
/// the active sub-tab of each dense page. These are UI state, not workspace
/// content, so their keys are excluded from workspace backups.
pub(super) struct StartupWorkspacePrefs {
    pub(super) theme: Theme,
    pub(super) inspector_visible: bool,
    pub(super) view: View,
    pub(super) operations: OperationsSection,
    pub(super) settings: SettingsSection,
    pub(super) runtimes: RuntimesSection,
    pub(super) snapshots: SnapshotsSection,
    pub(super) monitor: MonitorSection,
    pub(super) federation: FederationSection,
    pub(super) roles: RolesSection,
    pub(super) notice: Option<String>,
}

impl StartupWorkspacePrefs {
    pub(super) fn load(repository: &Repository) -> Self {
        let (theme, theme_notice) = load_theme(repository);
        let (inspector_visible, inspector_notice) = load_inspector_visible(repository);
        let (view, view_notice) = load_view(repository);
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

        Self {
            theme,
            inspector_visible,
            view,
            operations,
            settings,
            runtimes,
            snapshots,
            monitor,
            federation,
            roles,
            notice: first_notice([
                theme_notice,
                inspector_notice,
                view_notice,
                operations_notice,
                settings_notice,
                runtimes_notice,
                snapshots_notice,
                monitor_notice,
                federation_notice,
                roles_notice,
            ]),
        }
    }
}

fn load_theme(repository: &Repository) -> (Theme, Option<String>) {
    match repository.load_app_dark_mode() {
        Ok(dark) => (Theme::from_dark_mode(dark), None),
        Err(error) => (
            Theme::default(),
            Some(format!("Using default theme: {error}")),
        ),
    }
}

fn load_inspector_visible(repository: &Repository) -> (bool, Option<String>) {
    match repository.load_app_inspector_visible() {
        Ok(visible) => (visible, None),
        Err(error) => (
            false,
            Some(format!("Using default inspector layout: {error}")),
        ),
    }
}

fn load_view(repository: &Repository) -> (View, Option<String>) {
    match repository.load_workspace_last_view() {
        Ok(stored) => (
            stored
                .as_deref()
                .and_then(View::from_persist_key)
                .unwrap_or(View::Summary),
            None,
        ),
        Err(error) => (
            View::Summary,
            Some(format!("Using default workspace view: {error}")),
        ),
    }
}

/// Loads a dense-page sub-tab, falling back to `default_section` for a fresh
/// workspace, an unreadable value, or an unknown persist key.
fn load_section<T>(
    repository: &Repository,
    setting_key: &str,
    decode: fn(&str) -> Option<T>,
    default_section: T,
) -> (T, Option<String>) {
    match repository.load_workspace_section(setting_key) {
        Ok(Some(stored)) => match decode(&stored) {
            Some(section) => (section, None),
            None => (
                default_section,
                Some(format!(
                    "Ignoring unknown {setting_key} value {stored:?}; using {setting_key} default"
                )),
            ),
        },
        Ok(None) => (default_section, None),
        Err(error) => (
            default_section,
            Some(format!("Using default sub-tab: {error}")),
        ),
    }
}

fn first_notice(notices: [Option<String>; 10]) -> Option<String> {
    notices.into_iter().flatten().next()
}
