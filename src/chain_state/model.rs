use serde::Serialize;

use crate::roles::ChainRole;

/// Why a chain query could not be answered. Kept separate from a *negative*
/// answer: "the node is unreachable" and "your key is not designated" call for
/// completely different operator responses, and collapsing them into one error
/// state is how a manager ends up lying about chain state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainQueryError {
    /// The RPC endpoint could not be reached or returned a transport error.
    Unreachable(String),
    /// The node answered, but not in a shape this version understands.
    Unexpected(String),
}

impl ChainQueryError {
    pub fn message(&self) -> &str {
        match self {
            Self::Unreachable(message) | Self::Unexpected(message) => message,
        }
    }
}

/// Who currently holds a `RoleManagement` designation, and whether this node's
/// key is among them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RoleDesignation {
    pub role: ChainRole,
    /// Public keys the committee has designated, in the order the contract
    /// returned them.
    pub designated: Vec<String>,
    /// Whether the key this node signs with appears above. `None` when no key
    /// was supplied to compare against, which is different from `Some(false)`.
    pub includes_node_key: Option<bool>,
}

impl RoleDesignation {
    pub fn is_designated(&self) -> bool {
        self.includes_node_key.unwrap_or(false)
    }

    /// A one-line operator summary that never overstates what is known.
    pub fn summary(&self) -> String {
        match self.includes_node_key {
            Some(true) => format!(
                "designated for {} ({} key(s) hold this role)",
                self.role.label(),
                self.designated.len()
            ),
            Some(false) => format!(
                "not designated for {}; {} key(s) currently hold it",
                self.role.label(),
                self.designated.len()
            ),
            None => format!(
                "{} key(s) hold {}; this node has no key to compare",
                self.designated.len(),
                self.role.label()
            ),
        }
    }
}

/// The result of asking a node about a designation.
pub type DesignationStatus = Result<RoleDesignation, ChainQueryError>;

/// Where a public key stands in the candidate vote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CandidateStanding {
    pub public_key: String,
    /// Votes in NEO, as returned by the contract.
    pub votes: i64,
}

/// A read-only picture of Neo governance at the height the node is at.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GovernanceSnapshot {
    /// The 21 committee members.
    pub committee: Vec<String>,
    /// The validators producing blocks for the next round.
    pub next_validators: Vec<String>,
    /// Registered candidates and their vote totals, highest first.
    pub candidates: Vec<CandidateStanding>,
}

impl GovernanceSnapshot {
    /// Whether a key sits on the committee.
    pub fn is_committee_member(&self, public_key: &str) -> bool {
        self.committee.iter().any(|key| key == public_key)
    }

    /// Whether a key is producing blocks this round.
    pub fn is_validator(&self, public_key: &str) -> bool {
        self.next_validators.iter().any(|key| key == public_key)
    }

    pub fn candidate_standing(&self, public_key: &str) -> Option<&CandidateStanding> {
        self.candidates
            .iter()
            .find(|candidate| candidate.public_key == public_key)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/chain_state/model/tests.rs"]
mod tests;
