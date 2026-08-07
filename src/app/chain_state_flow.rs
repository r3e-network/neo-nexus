//! Operator-triggered reads of the chain state a node's duties depend on.
//!
//! These are deliberately manual, like the RPC health check: a designation
//! changes when the committee votes, and polling for it every second would put
//! load on a node to learn nothing. They also never sign — see `chain_state`
//! for why the app reports designation status but does not offer to change it.

use super::*;

use crate::{
    app::domain::{designation_status, governance_snapshot},
    rpc_health::node_rpc_endpoint,
};

impl NeoNexusApp {
    /// Asks the selected node whether its key holds the designation its duty
    /// needs.
    pub(in crate::app) fn check_selected_designation(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice = Some("Select a node before checking designation".to_string());
            return;
        };
        let role = self.repository.load_node_role(&node.id).unwrap_or_default();
        let Some(role) = role else {
            self.session.notice = Some(format!(
                "{} has no duty assigned; apply a role before checking designation",
                node.name
            ));
            return;
        };
        let Some(chain_role) = role.designation() else {
            self.session.notice = Some(format!("The {role} duty needs no committee designation",));
            return;
        };

        let endpoint = node_rpc_endpoint(&node);
        let status = designation_status(&endpoint, chain_role, None, CHAIN_STATE_TIMEOUT);
        let message = match &status {
            Ok(designation) => format!("{}: {}", node.name, designation.summary()),
            Err(error) => format!("{}: {}", node.name, error.message()),
        };
        self.chain.designation = Some((node.id.clone(), status));
        self.session.notice = Some(message);
    }

    /// Reads the committee, the next round's validators, and the candidate vote
    /// from the selected node.
    pub(in crate::app) fn refresh_governance(&mut self) {
        let Some(node) = self.selected_node().cloned() else {
            self.session.notice =
                Some("Select a node to read governance state through".to_string());
            return;
        };
        match governance_snapshot(&node_rpc_endpoint(&node), CHAIN_STATE_TIMEOUT) {
            Ok(snapshot) => {
                self.session.notice = Some(format!(
                    "Committee {} · validators {} · candidates {}",
                    snapshot.committee.len(),
                    snapshot.next_validators.len(),
                    snapshot.candidates.len()
                ));
                self.chain.governance_error = None;
                self.chain.governance = Some(snapshot);
            }
            Err(error) => {
                let message = error.message().to_string();
                self.chain.governance_error = Some(message.clone());
                self.session.notice = Some(message);
            }
        }
    }
}
