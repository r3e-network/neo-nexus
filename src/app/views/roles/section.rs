/// Focused regions of the Roles page, surfaced one at a time through a
/// segmented control so the role planner and the private-network planner each
/// get the full workspace instead of sharing a cramped split.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum RolesSection {
    Presets,
    Plan,
    PrivateNetwork,
}

impl RolesSection {
    pub(in crate::app) const ALL: [Self; 3] = [Self::Presets, Self::Plan, Self::PrivateNetwork];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Presets => "Presets",
            Self::Plan => "Role Plan",
            Self::PrivateNetwork => "Private Network",
        }
    }

    /// Stable identifier used to persist the active sub-tab across restarts.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Presets => "presets",
            Self::Plan => "plan",
            Self::PrivateNetwork => "private_network",
        }
    }

    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|section| section.persist_key() == key)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/roles/section/tests.rs"]
mod tests;
