/// Focused regions of the Settings page, surfaced one at a time through a
/// segmented control so each policy group gets the full workspace instead of
/// six cramped panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum SettingsSection {
    Watchdog,
    Upgrades,
    Monitors,
    Alerts,
    Storage,
    Release,
}

impl SettingsSection {
    pub(in crate::app) const ALL: [Self; 6] = [
        Self::Watchdog,
        Self::Upgrades,
        Self::Monitors,
        Self::Alerts,
        Self::Storage,
        Self::Release,
    ];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Watchdog => "Watchdog",
            Self::Upgrades => "Upgrades",
            Self::Monitors => "Monitors",
            Self::Alerts => "Alerts",
            Self::Storage => "Storage",
            Self::Release => "Release",
        }
    }

    /// Stable identifier used to persist the active sub-tab across restarts.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Watchdog => "watchdog",
            Self::Upgrades => "upgrades",
            Self::Monitors => "monitors",
            Self::Alerts => "alerts",
            Self::Storage => "storage",
            Self::Release => "release",
        }
    }

    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|section| section.persist_key() == key)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/settings/section/tests.rs"]
mod tests;
