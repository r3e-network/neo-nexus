use super::super::super::{
    super::format::{GenerationContext, RuntimeConfigProfile},
    checks::*,
    model::ConfigValidationReport,
};
use crate::roles::NodeRole;

/// `[consensus]` (which the daemon also accepts as `[dbft]`) takes `enabled`,
/// `auto_start`, `private_key_hex` and an optional HSM block — and nothing
/// else.
///
/// This used to assert a `validators` array. neo-rs has no such field, and
/// because it ignores unknown keys rather than rejecting them the check passed
/// on a list the node silently discarded: the committee's public keys were
/// written into a file that never read them. The validator set comes from the
/// protocol preset; the only key this node contributes is its own.
/// Two things can put this node on consensus duty, and the check has to honour
/// both or it rejects the generator's own output: a private-network profile that
/// marks the node a validator, and the Consensus duty assigned to it directly.
/// Only the profile was consulted before, so a neo-rs node holding the Consensus
/// duty generated `enabled = true`, was told `false` was expected, and every
/// export and Apply Config failed and wrote no file.
pub(super) fn check(
    profile: Option<&RuntimeConfigProfile>,
    context: &GenerationContext,
    report: &mut ConfigValidationReport,
    value: &toml::Value,
) {
    let consensus_expected = profile.is_some_and(|profile| profile.consensus_enabled)
        || context.role == Some(NodeRole::Consensus);
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
