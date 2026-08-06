//! Which body the inspector column is showing.
//!
//! The inspector is a fixed-height panel with no scrolling, and the facts an
//! operator wants about a node (definition, filesystem paths, live process,
//! runtime policy) total far more than fits. Previously they were simply
//! stacked and everything past the panel edge was painted invisibly. Splitting
//! them into switchable sections keeps every fact reachable while honouring the
//! fixed-panel model the rest of the workbench uses.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::app) enum InspectorSection {
    /// Definition facts and the lifecycle / open actions.
    #[default]
    Overview,
    /// Filesystem locations for the selected node.
    Paths,
    /// Live process telemetry and the application runtime facts.
    Process,
}

impl InspectorSection {
    pub(in crate::app) const ALL: [Self; 3] = [Self::Overview, Self::Paths, Self::Process];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Overview => "Overview",
            Self::Paths => "Paths",
            Self::Process => "Process",
        }
    }

    pub(in crate::app) fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|section| *section == self)
            .unwrap_or(0)
    }

    pub(in crate::app) fn from_index(index: usize) -> Self {
        Self::ALL.get(index).copied().unwrap_or_default()
    }
}

#[cfg(test)]
#[path = "../../../../../tests/unit/app/views/shell/inspector/section/tests.rs"]
mod tests;
