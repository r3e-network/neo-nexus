//! Operator-facing text for chain reads.
//!
//! Every line states what is known and, where it matters, what is not. A
//! designation report in particular must never let "we could not check this
//! node's key" read as "this node is not designated".

use super::model::{GovernanceSnapshot, RoleDesignation};

/// How many candidates to print. The vote has a long tail of near-zero
/// registrations; the top of it is what tells an operator where they stand.
const CANDIDATE_ROWS: usize = 10;

impl RoleDesignation {
    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("chain-role: {}", self.role.label()),
            format!("on-chain-value: {}", self.role.on_chain_value()),
            format!("designated-keys: {}", self.designated.len()),
            format!(
                "node-designated: {}",
                match self.includes_node_key {
                    Some(true) => "yes",
                    Some(false) => "no",
                    None => "unknown (no node key supplied)",
                }
            ),
            format!("summary: {}", self.summary()),
        ];
        for key in &self.designated {
            lines.push(format!("key: {key}"));
        }
        lines.join("\n")
    }
}

impl GovernanceSnapshot {
    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("committee: {}", self.committee.len()),
            format!("next-validators: {}", self.next_validators.len()),
            format!("candidates: {}", self.candidates.len()),
        ];
        for key in &self.committee {
            let producing = if self.is_validator(key) {
                " validator"
            } else {
                ""
            };
            lines.push(format!("committee-member: {key}{producing}"));
        }
        for candidate in self.candidates.iter().take(CANDIDATE_ROWS) {
            lines.push(format!(
                "candidate: {} votes={}",
                candidate.public_key, candidate.votes
            ));
        }
        if self.candidates.len() > CANDIDATE_ROWS {
            lines.push(format!(
                "candidates-omitted: {}",
                self.candidates.len() - CANDIDATE_ROWS
            ));
        }
        lines.join("\n")
    }
}

#[cfg(test)]
#[path = "../../tests/unit/chain_state/render/tests.rs"]
mod tests;
