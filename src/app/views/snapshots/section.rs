/// Focused regions of the Snapshots page, surfaced one at a time through a
/// segmented control so authoring, browsing, and verifying snapshots each get
/// the full workspace instead of four cramped columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum SnapshotsSection {
    Manifest,
    Catalog,
    Registry,
    Verify,
}

impl SnapshotsSection {
    pub(in crate::app) const ALL: [Self; 4] =
        [Self::Manifest, Self::Catalog, Self::Registry, Self::Verify];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Manifest => "Manifest",
            Self::Catalog => "Catalog",
            Self::Registry => "Registry",
            Self::Verify => "Verify & Cache",
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/snapshots/section/tests.rs"]
mod tests;
