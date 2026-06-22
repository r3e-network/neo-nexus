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
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/roles/section/tests.rs"]
mod tests;
