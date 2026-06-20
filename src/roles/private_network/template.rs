use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivateNetworkTemplate {
    SingleValidator,
    FourValidators,
    SevenNodeLab,
}

impl PrivateNetworkTemplate {
    pub const ALL: [Self; 3] = [
        Self::SingleValidator,
        Self::FourValidators,
        Self::SevenNodeLab,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::SingleValidator => "Single validator",
            Self::FourValidators => "Four validators",
            Self::SevenNodeLab => "Seven-node lab",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::SingleValidator => "One local consensus node for minimal private testing.",
            Self::FourValidators => "Four consensus nodes for dBFT quorum practice.",
            Self::SevenNodeLab => "Four validators plus RPC, state, and indexer operators.",
        }
    }
}

impl fmt::Display for PrivateNetworkTemplate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}
