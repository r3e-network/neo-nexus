/// Focused regions of the Federation page, surfaced one at a time through a
/// segmented control. Selection persists across segments, so picking a profile
/// and then switching to the editor or inspector stays in context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum FederationSection {
    Profiles,
    Editor,
    Inspector,
    /// The committee, the validators, and the candidate vote.
    Governance,
}

impl FederationSection {
    pub(in crate::app) const ALL: [Self; 4] = [
        Self::Profiles,
        Self::Editor,
        Self::Inspector,
        Self::Governance,
    ];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Profiles => "Profiles",
            Self::Editor => "Editor",
            Self::Inspector => "Inspector",
            Self::Governance => "Governance",
        }
    }

    /// Stable identifier used to persist the active sub-tab across restarts.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Profiles => "profiles",
            Self::Editor => "editor",
            Self::Inspector => "inspector",
            Self::Governance => "governance",
        }
    }

    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|section| section.persist_key() == key)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/federation/section/tests.rs"]
mod tests;
