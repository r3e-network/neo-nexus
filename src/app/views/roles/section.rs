/// The two halves of applying a role to the selected node: choosing a preset,
/// and reviewing the plugin changes it implies. Surfaced one at a time through
/// a segmented control so each gets the full workspace.
///
/// Private-network planning used to be a third section here. It is a *topology*
/// — how many validators, which committee — not a property of one node, so it
/// lives under Network with the remote endpoints instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum RolesSection {
    Presets,
    Plan,
}

impl RolesSection {
    pub(in crate::app) const ALL: [Self; 2] = [Self::Presets, Self::Plan];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Presets => "Presets",
            Self::Plan => "Role Plan",
        }
    }

    /// Stable identifier used to persist the active sub-tab across restarts.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Presets => "presets",
            Self::Plan => "plan",
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
