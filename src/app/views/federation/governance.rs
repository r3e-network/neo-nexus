//! Neo governance, read from a node.
//!
//! The committee (21 members) sets fees, designates roles and manages the
//! policy contract. The validators (7 of them) produce blocks. Both are elected
//! by NEO holders voting for candidates, and both change over time — so who
//! holds them is chain state, not configuration.
//!
//! Read-only. Registering a candidate costs GAS and voting spends a NEO
//! holder's weight; both are signed transactions this application does not
//! perform. What it can do is show an operator exactly where a key stands.

use eframe::egui;

use crate::rpc_health::node_rpc_endpoint;

use crate::app::{
    domain::GovernanceSnapshot,
    state::ChainReadKey,
    text::truncate_middle,
    theme,
    widgets::{callout, empty_state, grid_header, metric_grid, secondary_button, CalloutKind},
    NeoNexusApp,
};

/// How many candidates to list. The vote has a long tail of near-zero
/// registrations; the top of it is what tells an operator where they stand.
const CANDIDATE_ROWS: usize = 8;

/// A public key is 66 hex characters, far too wide for a column, so it is
/// middle-truncated to something an operator can still recognise.
const KEY_CHARS: usize = 20;

impl NeoNexusApp {
    pub(in crate::app::views::federation) fn render_governance(&mut self, ui: &mut egui::Ui) {
        let Some(node) = self.selected_node().cloned() else {
            empty_state(
                ui,
                "No node selected",
                "Governance is read through a node's RPC. Select one from Inventory.",
            );
            return;
        };

        // Both the answer and the error belong to the chain they came from. A
        // committee read through a testnet node must not be shown against a
        // mainnet node, and an error reading one chain is not the other's.
        let key = ChainReadKey::for_node(&node, &node_rpc_endpoint(&node));
        if let Some(error) = self
            .chain
            .governance_error_for(&key)
            .map(ToString::to_string)
        {
            callout(ui, CalloutKind::Danger, "Could not read the chain", &error);
            ui.add_space(theme::SM);
        }

        match self.chain.governance_for(&key).cloned() {
            Some(snapshot) => render_snapshot(ui, &snapshot),
            None => {
                ui.label(theme::muted_body(format!(
                    "Governance has not been read on {} yet.",
                    node.network
                )));
            }
        }

        ui.add_space(theme::SM);
        if secondary_button(ui, "Read governance")
            .on_hover_text("Query the committee, validators and candidate vote")
            .clicked()
        {
            self.refresh_governance();
        }
        ui.add_space(theme::XS);
        ui.label(theme::caption(
            "Read-only. Registering a candidate costs GAS and voting spends NEO; both are signed \
             transactions NeoNexus does not perform.",
        ));
    }
}

fn render_snapshot(ui: &mut egui::Ui, snapshot: &GovernanceSnapshot) {
    metric_grid(
        ui,
        &[
            ("Committee", snapshot.committee.len().to_string()),
            ("Validators", snapshot.next_validators.len().to_string()),
            ("Candidates", snapshot.candidates.len().to_string()),
            ("Listed below", listed(snapshot).to_string()),
        ],
    );
    if snapshot.candidates.is_empty() {
        return;
    }
    ui.add_space(theme::SM);
    ui.label(theme::label_caption("Candidate vote"));
    ui.add_space(theme::XS);
    egui::Grid::new("governance_candidates")
        .num_columns(3)
        .spacing([theme::MD, theme::XS])
        .show(ui, |ui| {
            grid_header(ui, &["Candidate", "Votes", "Standing"]);
            for candidate in snapshot.candidates.iter().take(CANDIDATE_ROWS) {
                ui.label(theme::body(truncate_middle(
                    &candidate.public_key,
                    KEY_CHARS,
                )));
                ui.label(theme::body(candidate.votes.to_string()));
                ui.label(theme::muted_body(standing(snapshot, &candidate.public_key)));
                ui.end_row();
            }
        });
}

fn listed(snapshot: &GovernanceSnapshot) -> usize {
    snapshot.candidates.len().min(CANDIDATE_ROWS)
}

/// Sitting on the committee and producing blocks are different things: the
/// validators are the top 7 of the 21, so the distinction is worth showing.
fn standing(snapshot: &GovernanceSnapshot, public_key: &str) -> &'static str {
    if snapshot.is_validator(public_key) {
        "validator"
    } else if snapshot.is_committee_member(public_key) {
        "committee"
    } else {
        "candidate"
    }
}
