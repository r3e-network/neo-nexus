use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCategory {
    Storage,
    Api,
    Core,
    Indexing,
    /// Services that act with a committee-designated key: consensus and oracle
    /// duties. Kept apart from `Core` because enabling one is an on-chain
    /// commitment, not a local capability.
    Governance,
}

impl PluginCategory {
    pub const ALL: [Self; 5] = [
        Self::Api,
        Self::Core,
        Self::Governance,
        Self::Indexing,
        Self::Storage,
    ];
}

impl fmt::Display for PluginCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Storage => "Storage",
            Self::Api => "Network & API",
            Self::Core => "Core services",
            Self::Indexing => "Indexing",
            Self::Governance => "Chain duties",
        })
    }
}
