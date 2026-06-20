use crate::types::NodeConfig;

use super::super::super::{
    super::format::{
        effective_committee_public_keys, effective_validators_count, RuntimeConfigProfile,
    },
    checks::*,
    model::ConfigValidationReport,
};

pub(super) fn check(
    node: &NodeConfig,
    profile: Option<&RuntimeConfigProfile>,
    report: &mut ConfigValidationReport,
    value: &toml::Value,
) {
    let consensus_expected = profile.is_some_and(|profile| profile.consensus_enabled);
    check_toml_bool(
        report,
        value,
        &["consensus", "enabled"],
        consensus_expected,
        "Consensus",
    );
    check_toml_bool(
        report,
        value,
        &["consensus", "auto_start"],
        consensus_expected,
        "Consensus auto start",
    );

    let expected_keys = effective_committee_public_keys(profile);
    let expected_validators = if profile.is_some() {
        expected_keys.len().max(if consensus_expected {
            effective_validators_count(node.network, profile) as usize
        } else {
            0
        })
    } else {
        0
    };
    check_toml_array_len_at_least(
        report,
        value,
        &["consensus", "validators"],
        expected_validators,
        "Consensus validators",
    );
    check_toml_string_array_exact(
        report,
        value,
        &["consensus", "validators"],
        &expected_keys,
        "Consensus validator keys",
    );
}
