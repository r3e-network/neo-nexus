use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCategory {
    Storage,
    Api,
    Core,
    Indexing,
}

impl PluginCategory {
    pub const ALL: [Self; 4] = [Self::Api, Self::Core, Self::Indexing, Self::Storage];
}

impl fmt::Display for PluginCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Storage => "Storage",
            Self::Api => "Network & API",
            Self::Core => "Core services",
            Self::Indexing => "Indexing",
        })
    }
}
