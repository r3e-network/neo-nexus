/// Focused regions of the Monitor page, surfaced one at a time through a
/// segmented control so each telemetry view gets the full workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum MonitorSection {
    Pressure,
    Telemetry,
    Processes,
}

impl MonitorSection {
    pub(in crate::app) const ALL: [Self; 3] = [Self::Pressure, Self::Telemetry, Self::Processes];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Pressure => "Pressure",
            Self::Telemetry => "Telemetry",
            Self::Processes => "Processes",
        }
    }

    /// Stable identifier used to persist the active sub-tab across restarts.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Pressure => "pressure",
            Self::Telemetry => "telemetry",
            Self::Processes => "processes",
        }
    }

    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|section| section.persist_key() == key)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/monitor/section/tests.rs"]
mod tests;
