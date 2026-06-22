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
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/monitor/section/tests.rs"]
mod tests;
