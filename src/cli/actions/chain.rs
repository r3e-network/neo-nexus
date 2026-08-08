use super::*;

use crate::core::node_chain::{
    designation_status, governance_snapshot, ChainRole, GovernanceSnapshot, RoleDesignation,
};

/// Chain reads take the same three seconds as the RPC health probe: an operator
/// waiting at a terminal should get an answer or a failure quickly.
const CHAIN_TIMEOUT: Duration = Duration::from_secs(3);

pub(in crate::cli::actions) fn designation_text(args: &[String]) -> Result<String> {
    Ok(designation_report(args)?.to_cli_text())
}

pub(in crate::cli::actions) fn designation_json_action(args: &[String]) -> Result<CliAction> {
    let report = designation_report(args)?;
    // Exit non-zero when the key is not designated, so a deployment script can
    // gate on it. An unreadable chain is a different failure and already
    // returns an error.
    let designated = report.is_designated();
    Ok(CliAction::PrintWithExitCode {
        text: designation_json_text(&report)?,
        exit_code: i32::from(!designated),
    })
}

fn designation_report(args: &[String]) -> Result<RoleDesignation> {
    let option = args.get(1).map_or("--designation", String::as_str);
    if args.len() < 4 {
        anyhow::bail!(
            "usage: neo-nexus {option} <rpc-endpoint> \
             <state-validator|oracle|neofs-alphabet|p2p-notary> [public-key]"
        );
    }
    let role = parse_chain_role(&args[3])?;
    let public_key = args.get(4).map(String::as_str);
    designation_status(&args[2], role, public_key, CHAIN_TIMEOUT)
        .map_err(|error| anyhow::anyhow!("{}", error.message()))
}

/// Accepts the operator-facing spellings rather than the on-chain integers: a
/// mistyped number would silently query a different duty.
fn parse_chain_role(value: &str) -> Result<ChainRole> {
    let normalized = value.trim().to_ascii_lowercase().replace(['_', ' '], "-");
    // Matched two ways: the contract's own CamelCase label with the separators
    // stripped, and the hyphenated spelling an operator is more likely to type.
    let compact = normalized.replace('-', "");
    ChainRole::ALL
        .into_iter()
        .find(|role| {
            let label = role.label().to_ascii_lowercase();
            label == compact || label == normalized
        })
        .with_context(|| {
            format!(
                "unsupported chain role: {value}; expected one of {}",
                ChainRole::ALL
                    .map(|role| role.label().to_string())
                    .join(", ")
            )
        })
}

pub(in crate::cli::actions) fn governance_text(args: &[String]) -> Result<String> {
    Ok(governance_report(args)?.to_cli_text())
}

pub(in crate::cli::actions) fn governance_json_action(args: &[String]) -> Result<CliAction> {
    let report = governance_report(args)?;
    Ok(CliAction::Print(governance_json_text(&report)?))
}

fn governance_report(args: &[String]) -> Result<GovernanceSnapshot> {
    let option = args.get(1).map_or("--governance", String::as_str);
    require_arg_count(args, 3, option)?;
    governance_snapshot(&args[2], CHAIN_TIMEOUT)
        .map_err(|error| anyhow::anyhow!("{}", error.message()))
}

#[cfg(test)]
#[path = "../../../tests/unit/cli/chain/tests.rs"]
mod tests;
