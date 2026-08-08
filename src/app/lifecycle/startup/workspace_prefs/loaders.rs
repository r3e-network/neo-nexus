//! Reading one preference at a time.
//!
//! Each loader answers with the value *and* a notice when the stored setting
//! could not be read, so a corrupt row degrades to the default and says so
//! rather than failing startup or silently resetting.

use super::super::super::super::*;

pub(super) fn load_theme(repository: &Repository) -> (Theme, Option<String>) {
    match repository.load_app_dark_mode() {
        Ok(dark) => (Theme::from_dark_mode(dark), None),
        Err(error) => (
            Theme::default(),
            Some(format!("Using default theme: {error}")),
        ),
    }
}

pub(super) fn load_density(repository: &Repository) -> (UiDensity, Option<String>) {
    match repository.load_app_ui_density() {
        Ok(Some(stored)) => match UiDensity::from_persist_key(&stored) {
            Some(density) => (density, None),
            None => (
                UiDensity::default(),
                Some(format!(
                    "Ignoring unknown appearance.ui_density value {stored:?}; using comfortable"
                )),
            ),
        },
        Ok(None) => (UiDensity::default(), None),
        Err(error) => (
            UiDensity::default(),
            Some(format!("Using default density: {error}")),
        ),
    }
}

pub(super) fn load_inspector_visible(repository: &Repository) -> (bool, Option<String>) {
    match repository.load_app_inspector_visible() {
        Ok(visible) => (visible, None),
        Err(error) => (
            false,
            Some(format!("Using default inspector layout: {error}")),
        ),
    }
}

pub(super) fn load_view(repository: &Repository) -> (View, Option<String>) {
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
pub(super) fn load_section<T>(
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

pub(super) fn first_notice(notices: [Option<String>; 12]) -> Option<String> {
    notices.into_iter().flatten().next()
}
