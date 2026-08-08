//! Operator-triggered reads of the chain state a node's duties depend on.
//!
//! These are deliberately manual, like the RPC health check: a designation
//! changes when the committee votes, and polling for it every second would put
//! load on a node to learn nothing. They also never sign — see `chain_state`
//! for why the app reports designation status but does not offer to change it.

use super::*;

use crate::{
    app::{
        domain::{designation_status, governance_snapshot},
        state::ChainReadKey,
    },
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
        let public_key = self.node_signing_key(&node.id);
        let status = designation_status(
            &endpoint,
            chain_role,
            public_key.as_deref(),
            CHAIN_STATE_TIMEOUT,
        );
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
        let endpoint = node_rpc_endpoint(&node);
        // Keyed by the chain the answer came from, so selecting a node on a
        // different network shows that network's committee or nothing — never
        // this one's. See `ChainReadKey`.
        let key = ChainReadKey::for_node(&node, &endpoint);
        match governance_snapshot(&endpoint, CHAIN_STATE_TIMEOUT) {
            Ok(snapshot) => {
                self.session.notice = Some(format!(
                    "Committee {} · validators {} · candidates {}",
                    snapshot.committee.len(),
                    snapshot.next_validators.len(),
                    snapshot.candidates.len()
                ));
                self.chain.record_governance(key, snapshot);
            }
            Err(error) => {
                let message = error.message().to_string();
                self.chain.record_governance_error(key, message.clone());
                self.session.notice = Some(message);
            }
        }
    }

    /// The public key this node signs with, from the wallet profile assigned to
    /// it.
    ///
    /// `None` when no wallet is assigned, and the difference matters: without a
    /// key the report says how many keys hold the duty but makes no claim about
    /// *this* node, rather than reporting it as undesignated.
    fn node_signing_key(&self, node_id: &str) -> Option<String> {
        let profile_id = self.repository.load_node_wallet(node_id).ok().flatten()?;
        self.neo_wallet_profiles
            .iter()
            .find(|profile| profile.id == profile_id)
            .and_then(|profile| profile.contract_public_keys.first().cloned())
    }
}
