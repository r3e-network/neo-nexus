/// In-page tabs for the Nodes primary surface. Secondary node tools that used
/// to be top-level nav entries (Config / Logs / Plugins) live here so operators
/// stay in one node context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(in crate::app) enum NodeWorkspaceTab {
    #[default]
    Studio,
    Config,
    Logs,
    Plugins,
    Health,
}

impl NodeWorkspaceTab {
    pub(in crate::app) const ALL: [Self; 5] = [
        Self::Studio,
        Self::Config,
        Self::Logs,
        Self::Plugins,
        Self::Health,
    ];

    pub(in crate::app) fn label(self) -> &'static str {
        match self {
            Self::Studio => "Studio",
            Self::Config => "Config",
            Self::Logs => "Logs",
            Self::Plugins => "Plugins",
            Self::Health => "Health",
        }
    }

    pub(in crate::app) fn persist_key(self) -> &'static str {
        match self {
            Self::Studio => "studio",
            Self::Config => "config",
            Self::Logs => "logs",
            Self::Plugins => "plugins",
            Self::Health => "health",
        }
    }

    pub(in crate::app) fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|tab| tab.persist_key() == key)
    }

    /// Map a legacy top-level view into the node workspace tab it now owns.
    pub(in crate::app) fn from_legacy_view(view: crate::app::view::View) -> Option<Self> {
        match view {
            crate::app::view::View::Config => Some(Self::Config),
            crate::app::view::View::Logs => Some(Self::Logs),
            crate::app::view::View::Plugins => Some(Self::Plugins),
            crate::app::view::View::Monitor => Some(Self::Health),
            crate::app::view::View::Nodes => Some(Self::Studio),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "../../../../tests/unit/app/views/nodes/workspace/tests.rs"]
mod tests;
