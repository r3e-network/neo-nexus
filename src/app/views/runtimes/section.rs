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
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/runtimes/section/tests.rs"]
mod tests;
