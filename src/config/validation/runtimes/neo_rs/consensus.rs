use crate::types::NodeConfig;

use super::super::super::{
    super::format::RuntimeConfigProfile, checks::*, model::ConfigValidationReport,
};

/// `[consensus]` (which the daemon also accepts as `[dbft]`) takes `enabled`,
/// `auto_start`, `private_key_hex` and an optional HSM block — and nothing
/// else.
///
/// This used to assert a `validators` array. neo-rs has no such field, and
/// because it ignores unknown keys rather than rejecting them the check passed
/// on a list the node silently discarded: the committee's public keys were
/// written into a file that never read them. The validator set comes from the
/// protocol preset; the only key this node contributes is its own.
pub(super) fn check(
    _node: &NodeConfig,
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
    // A private key in a generated config would be a plaintext secret on disk.
    check_toml_absent(
        report,
        value,
        &["consensus", "private_key_hex"],
        "Consensus signing key",
    );
}
