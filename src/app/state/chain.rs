//! The last answers read from the chain.
//!
//! Chain reads are operator-triggered, not periodic: a designation changes when
//! the committee votes, not every second. Results are held so the panel shows
//! what was read and when, instead of firing an RPC call on every repaint.

use crate::app::domain::{DesignationStatus, GovernanceSnapshot};

#[derive(Debug, Default)]
pub(in crate::app) struct ChainStateUi {
    /// Designation for the selected node's duty, and the node it was read for —
    /// so a stale answer is never shown against a different node.
    pub(in crate::app) designation: Option<(String, DesignationStatus)>,
    pub(in crate::app) governance: Option<GovernanceSnapshot>,
    /// Why the last governance read failed, if it did.
    pub(in crate::app) governance_error: Option<String>,
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
}
