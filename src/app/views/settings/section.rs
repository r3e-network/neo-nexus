/// Focused regions of the Settings page, surfaced one at a time through a
/// segmented control so each policy group gets the full workspace instead of
/// six cramped panels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum SettingsSection {
    Watchdog,
    Upgrades,
    Monitors,
    Storage,
    Release,
}

impl SettingsSection {
    pub(in crate::app) const ALL: [Self; 5] = [
        Self::Watchdog,
        Self::Upgrades,
        Self::Monitors,
        Self::Storage,
        Self::Release,
    ];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Watchdog => "Watchdog",
            Self::Upgrades => "Upgrades",
            Self::Monitors => "Monitors",
            Self::Storage => "Storage",
            Self::Release => "Release",
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/settings/section/tests.rs"]
mod tests;
