/// Focused regions of the Runtimes page, surfaced one at a time through a
/// segmented control so installing, browsing, and applying runtimes each get
/// the full workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::app) enum RuntimesSection {
    Install,
    Catalog,
    Installed,
    Applied,
}

impl RuntimesSection {
    pub(in crate::app) const ALL: [Self; 4] =
        [Self::Install, Self::Catalog, Self::Installed, Self::Applied];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Install => "Install",
            Self::Catalog => "Catalog",
            Self::Installed => "Installed",
            Self::Applied => "Applied",
        }
    }

    /// Stable identifier used to persist the active sub-tab across restarts.
    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Install => "install",
            Self::Catalog => "catalog",
            Self::Installed => "installed",
            Self::Applied => "applied",
        }
    }

    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|section| section.persist_key() == key)
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/runtimes/section/tests.rs"]
mod tests;
