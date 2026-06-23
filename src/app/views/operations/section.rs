/// Focused regions of the Operations page, surfaced one at a time through a
/// segmented control so the workspace stays calm instead of crowding five
/// panels onto one screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum OperationsSection {
    Readiness,
    ActionQueue,
    Ports,
    Safety,
    Journal,
}

impl OperationsSection {
    pub(in crate::app) const ALL: [Self; 5] = [
        Self::Readiness,
        Self::ActionQueue,
        Self::Ports,
        Self::Safety,
        Self::Journal,
    ];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Readiness => "Readiness",
            Self::ActionQueue => "Action Queue",
            Self::Ports => "Ports",
            Self::Safety => "Safety",
            Self::Journal => "Journal",
        }
    }

    /// Stable identifier used to persist the active sub-tab across restarts.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Readiness => "readiness",
            Self::ActionQueue => "action_queue",
            Self::Ports => "ports",
            Self::Safety => "safety",
            Self::Journal => "journal",
        }
    }

    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|section| section.persist_key() == key)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/operations/section/tests.rs"]
mod tests;
