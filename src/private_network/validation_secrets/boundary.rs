use super::*;

mod collect;
mod keys;
mod report;
mod strings;

use collect::collect_wallet_provisioning_secret_findings;
use report::summarize_secret_findings;

pub(super) fn check_wallet_provisioning_secret_boundary(
    checks: &mut Vec<LaunchPackValidationCheck>,
    value: &serde_json::Value,
) {
    let mut findings = Vec::new();
    collect_wallet_provisioning_secret_findings(value, "$", &mut findings);
    add_check(
        checks,
        "secret-provisioning",
        "wallet-provisioning-secret-boundary",
        if findings.is_empty() {
            LaunchPackValidationStatus::Pass
        } else {
            LaunchPackValidationStatus::Fail
        },
        if findings.is_empty() {
            "no private keys, wallet passwords, mnemonics, seeds, tokens, or inline sensitive argv material".to_string()
        } else {
            format!(
                "secret material markers found: {}",
                summarize_secret_findings(&findings)
            )
        },
    );
}
