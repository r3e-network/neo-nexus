use std::fmt;

/// Which Neo chain a node belongs to.
///
/// Neo N3 and Neo X are separate chains with separate clients, and almost
/// nothing crosses between them: N3 identifies a network by a 4-byte magic and
/// carries a 21-key standby committee, Neo X is EVM and identifies a network by
/// chain id; N3 nodes load plugin assemblies, Neo X nodes do not; N3 speaks its
/// own JSON-RPC, Neo X speaks Ethereum's.
///
/// The family is derived from the client rather than stored, because a client
/// only ever speaks one of them — there is no neo-cli that joins Neo X.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChainFamily {
    /// Neo N3: dBFT, native contracts, plugin assemblies, Neo JSON-RPC.
    NeoN3,
    /// Neo X: EVM execution with dBFT finality and an Anti-MEV pipeline.
    NeoX,
}

impl ChainFamily {
    pub const ALL: [Self; 2] = [Self::NeoN3, Self::NeoX];

    pub fn label(self) -> &'static str {
        match self {
            Self::NeoN3 => "Neo N3",
            Self::NeoX => "Neo X",
        }
    }

    /// Stable identifier for persistence and CLI arguments.
    pub fn slug(self) -> &'static str {
        match self {
            Self::NeoN3 => "neo-n3",
            Self::NeoX => "neo-x",
        }
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|family| family.slug() == slug)
    }

    /// Whether nodes of this family load plugin assemblies. Only the C# N3 node
    /// does; the plugin catalogue is meaningless everywhere else.
    pub fn has_plugins(self) -> bool {
        matches!(self, Self::NeoN3)
    }

    /// Whether NeoNexus can plan a private network from a template.
    ///
    /// The templates build a Neo N3 standby committee: a roster of consensus
    /// public keys written into every member's config, which is all an N3
    /// private chain needs. Neo X takes its validator set from a genesis
    /// allocation instead, and NeoNexus does not generate a Neo X genesis — an
    /// allocation it invented would produce a healthy-looking chain of one. So
    /// a "private Neo X network" planned from a template would be a fleet that
    /// never reaches consensus, and the picker must not offer one.
    pub fn has_committee_templates(self) -> bool {
        matches!(self, Self::NeoN3)
    }

    /// Whether this family speaks Ethereum JSON-RPC rather than Neo JSON-RPC.
    /// Decides which methods a health probe may call: `eth_blockNumber` on a
    /// Neo N3 node is as meaningless as `getblockcount` on a Neo X one.
    pub fn is_evm(self) -> bool {
        matches!(self, Self::NeoX)
    }
}

impl fmt::Display for ChainFamily {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

#[cfg(test)]
#[path = "../../tests/unit/types/chain_family/tests.rs"]
mod tests;
