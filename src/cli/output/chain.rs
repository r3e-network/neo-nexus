use anyhow::Result;
use serde::Serialize;

use crate::chain_state::{GovernanceSnapshot, RoleDesignation};

use super::json_text;

#[derive(Debug, Serialize)]
struct DesignationJsonReport<'a> {
    schema_version: u32,
    /// Whether the node's own key holds the designation. `null` when no key was
    /// supplied to compare — deliberately distinct from `false`, so a script
    /// cannot read "we did not check" as "not designated".
    node_designated: Option<bool>,
    report: &'a RoleDesignation,
}

pub(in crate::cli) fn designation_json_text(report: &RoleDesignation) -> Result<String> {
    json_text(&DesignationJsonReport {
        schema_version: 1,
        node_designated: report.includes_node_key,
        report,
    })
}

#[derive(Debug, Serialize)]
struct GovernanceJsonReport<'a> {
    schema_version: u32,
    committee_size: usize,
    validator_count: usize,
    candidate_count: usize,
    report: &'a GovernanceSnapshot,
}

pub(in crate::cli) fn governance_json_text(report: &GovernanceSnapshot) -> Result<String> {
    json_text(&GovernanceJsonReport {
        schema_version: 1,
        committee_size: report.committee.len(),
        validator_count: report.next_validators.len(),
        candidate_count: report.candidates.len(),
        report,
    })
}
