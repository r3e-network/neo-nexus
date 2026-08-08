//! The last answers read from the chain.
//!
//! Chain reads are operator-triggered, not periodic: a designation changes when
//! the committee votes, not every second. Results are held so the panel shows
//! what was read and when, instead of firing an RPC call on every repaint.
//!
//! Every held answer carries what it was read for. A cached answer shown against
//! something it was not read for is worse than no answer: an operator comparing a
//! mainnet node against a testnet committee sees a mismatch that is not real, or
//! a match that is not either.

use crate::app::domain::{DesignationStatus, GovernanceSnapshot, Network, NodeConfig};

/// What identifies the chain a governance answer came from.
///
/// Governance is chain state, not node state, so an answer read through one
/// mainnet node applies to every mainnet node — discarding it when the operator
/// selects a sibling would throw away a valid read for no reason.
///
/// Private networks are the exception. They all share one `Network::Private`
/// value and are emphatically *not* one chain, so the app cannot tell two apart
/// from the network alone. Those are keyed by the endpoint the answer came
/// through instead, which is the only thing distinguishing them here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) enum ChainReadKey {
    Public(Network),
    Private(String),
}

impl ChainReadKey {
    pub(in crate::app) fn for_node(node: &NodeConfig, endpoint: &str) -> Self {
        match node.network {
            Network::Private => Self::Private(endpoint.to_string()),
            network => Self::Public(network),
        }
    }
}

#[derive(Debug, Default)]
pub(in crate::app) struct ChainStateUi {
    /// Designation for the selected node's duty, and the node it was read for —
    /// so a stale answer is never shown against a different node.
    pub(in crate::app) designation: Option<(String, DesignationStatus)>,
    /// The governance answer, and the chain it came from.
    pub(in crate::app) governance: Option<(ChainReadKey, GovernanceSnapshot)>,
    /// Why the last governance read failed, and which chain it failed for.
    pub(in crate::app) governance_error: Option<(ChainReadKey, String)>,
}

impl ChainStateUi {
    /// The designation answer for `node_id`, or `None` when the last read was
    /// for a different node.
    pub(in crate::app) fn designation_for(&self, node_id: &str) -> Option<&DesignationStatus> {
        self.designation
            .as_ref()
            .filter(|(read_for, _)| read_for == node_id)
            .map(|(_, status)| status)
    }

    /// The governance answer for the chain `key` names, or `None` when the last
    /// read was for a different chain.
    pub(in crate::app) fn governance_for(&self, key: &ChainReadKey) -> Option<&GovernanceSnapshot> {
        self.governance
            .as_ref()
            .filter(|(read_for, _)| read_for == key)
            .map(|(_, snapshot)| snapshot)
    }

    /// Why the read for this chain failed, if it did. An error from another
    /// chain is not this chain's problem and is not shown here.
    pub(in crate::app) fn governance_error_for(&self, key: &ChainReadKey) -> Option<&str> {
        self.governance_error
            .as_ref()
            .filter(|(read_for, _)| read_for == key)
            .map(|(_, message)| message.as_str())
    }

    /// Records a successful read, replacing whatever was held for that chain and
    /// clearing its error.
    pub(in crate::app) fn record_governance(
        &mut self,
        key: ChainReadKey,
        snapshot: GovernanceSnapshot,
    ) {
        self.governance_error = None;
        self.governance = Some((key, snapshot));
    }

    /// Records a failed read, and drops the answer it failed to refresh.
    ///
    /// Keeping it would draw the error above a snapshot from before the chain
    /// became unreachable, with nothing to say the numbers were stale.
    pub(in crate::app) fn record_governance_error(&mut self, key: ChainReadKey, message: String) {
        if self
            .governance
            .as_ref()
            .is_some_and(|(read_for, _)| *read_for == key)
        {
            self.governance = None;
        }
        self.governance_error = Some((key, message));
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/app/state/chain/tests.rs"]
mod tests;
