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
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/operations/section/tests.rs"]
mod tests;
