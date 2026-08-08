//! The launch flags a Neo X client needs that its config file cannot carry.
//!
//! Both clients default their data directory to a path outside the workspace —
//! `~/.ethereum` for Neo X Geth, `~/Library/Application Support/reth` for
//! neox-rs — so a managed node left at the defaults writes its chain somewhere
//! NeoNexus does not manage, and two managed nodes silently share one database.
//!
//! neox-rs goes further. Reth keeps the chain, the data directory and every
//! listening port on the command line: its config file holds pipeline and
//! peering tuning and nothing else. Launched with only `--config`, neox-rs
//! takes its defaults for all of them.
//!
//! Its `--chain` default is not Ethereum: `NeoXChainSpecParser::SUPPORTED_CHAINS`
//! begins with `"mainnet"`, and Reth's `ChainSpecParser::default_value` returns
//! the first entry, which `parse` maps to `NeoXChainSpec::mainnet()` — Neo X
//! MainNet, chain id 47763. No Ethereum spec is reachable from the binary.
//!
//! That default is the hazard, not a wrong network family: a node an operator
//! labelled **Private** would come up on real Neo X MainNet, in the managed
//! datadir, with discovery off — an isolated node holding MainNet's genesis, on
//! which any key they treat as throwaway signs MainNet-valid transactions.
//! NeoNexus cannot invent a genesis to prevent that, so a private Neo X node is
//! blocked at readiness until the operator supplies `--chain <their spec>`
//! themselves; see `diagnostics::checks::chain`.
//!
//! Flags verified against `NodeCommand` in `crates/cli/commands/src/node.rs`
//! and the `RpcServerArgs` / `NetworkArgs` / `DatadirArgs` definitions in
//! `crates/node/core/src/args/` of r3e-network/neox-rs.

use std::path::{Path, PathBuf};

use crate::{
    config::neox_reth_chain,
    types::{Network, NodeConfig},
};

/// Where a managed Neo X node keeps its chain, under the node's own work dir.
const DATA_DIR: &str = "data";

/// Neo X Geth: the config file carries the chain id, the ports and the peers,
/// so only the data directory has to be forced onto the command line.
pub(super) fn geth_args(
    args: &mut Vec<String>,
    working_dir: &Path,
    managed_config_path: PathBuf,
) -> Option<PathBuf> {
    push_missing(args, "--datadir", &data_dir(working_dir));
    if has_flag(args, "--config") {
        return None;
    }
    push(args, "--config", &managed_config_path.display().to_string());
    Some(managed_config_path)
}

/// neox-rs: everything that decides *which chain* and *which ports* is a flag.
pub(super) fn reth_args(
    node: &NodeConfig,
    args: &mut Vec<String>,
    working_dir: &Path,
    managed_config_path: PathBuf,
) -> Option<PathBuf> {
    if args.first().is_none_or(|arg| arg != "node") {
        args.insert(0, "node".to_string());
    }

    // A private network has no built-in spec, and NeoNexus must not guess one.
    // Omitting the flag is not neutral — the client's own default is Neo X
    // MainNet — so the operator's own `--chain` is the only safe value here,
    // and its absence is a readiness blocker rather than something to paper
    // over with a default.
    if let Some(chain) = neox_reth_chain(node.network) {
        push_missing(args, "--chain", chain);
    }
    push_missing(args, "--datadir", &data_dir(working_dir));

    if !has_flag(args, "--http") {
        args.push("--http".to_string());
    }
    push_missing(args, "--http.addr", "127.0.0.1");
    push_missing(args, "--http.port", &node.rpc_port.to_string());
    push_missing(args, "--port", &node.p2p_port.to_string());

    // A private network has no bootnodes to find, so discovery would only
    // spray UDP at hosts the operator does not own.
    if node.network == Network::Private && !has_flag(args, "--disable-discovery") {
        args.push("--disable-discovery".to_string());
    }

    if has_flag(args, "--config") {
        return None;
    }
    push(args, "--config", &managed_config_path.display().to_string());
    Some(managed_config_path)
}

fn data_dir(working_dir: &Path) -> String {
    working_dir.join(DATA_DIR).display().to_string()
}

/// Adds a flag only when the operator has not already set it themselves.
///
/// Their value wins: someone who typed `--http.port 9999` into the runtime args
/// meant it, and silently overriding it would make the node unreachable at the
/// port they are watching.
fn push_missing(args: &mut Vec<String>, flag: &str, value: &str) {
    if !has_flag(args, flag) {
        push(args, flag, value);
    }
}

fn push(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.to_string());
    args.push(value.to_string());
}

/// Matches both `--flag value` and `--flag=value`.
fn has_flag(args: &[String], flag: &str) -> bool {
    let with_equals = format!("{flag}=");
    args.iter()
        .any(|arg| arg == flag || arg.starts_with(&with_equals))
}

#[cfg(test)]
#[path = "../../tests/unit/launch/neox/tests.rs"]
mod tests;
