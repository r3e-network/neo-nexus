/// Focused regions of the Federation page, surfaced one at a time through a
/// segmented control. Selection persists across segments, so picking a profile
/// and then switching to the editor or inspector stays in context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum FederationSection {
    Profiles,
    Editor,
    Inspector,
}

impl FederationSection {
    pub(in crate::app) const ALL: [Self; 3] = [Self::Profiles, Self::Editor, Self::Inspector];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Profiles => "Profiles",
            Self::Editor => "Editor",
            Self::Inspector => "Inspector",
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/federation/section/tests.rs"]
mod tests;
