use crate::types::{Network, NodeConfig};

use super::super::super::{
    super::format::{neox_bootnodes, neox_chain_id, RuntimeConfigProfile},
    checks::*,
    model::ConfigValidationReport,
};

pub(super) fn check(
    node: &NodeConfig,
    profile: Option<&RuntimeConfigProfile>,
    report: &mut ConfigValidationReport,
    value: &toml::Value,
) {
    let chain_id = neox_chain_id(node.network, profile);
    check_toml_u64_eq(report, value, &["Eth", "NetworkId"], chain_id, "Chain id");
    check_toml_string(report, value, &["Eth", "SyncMode"], "snap", "Sync mode");

    check_toml_u16(
        report,
        value,
        &["Node", "HTTPPort"],
        node.rpc_port,
        "RPC port",
    );
    check_toml_string(
        report,
        value,
        &["Node", "HTTPHost"],
        "127.0.0.1",
        "RPC bind",
    );
    check_toml_string(
        report,
        value,
        &["Node", "P2P", "ListenAddr"],
        &format!(":{}", node.p2p_port),
        "P2P bind",
    );

    // `DataDir = ""` is not a missing setting in geth: it starts an ephemeral
    // in-memory node whose chain is discarded on exit.
    check_toml_absent(report, value, &["Node", "DataDir"], "Data directory");

    bootnodes(node.network, report, value);
    genesis_note(node.network, report);
}

fn bootnodes(network: Network, report: &mut ConfigValidationReport, value: &toml::Value) {
    let path = ["Node", "P2P", "BootstrapNodes"];
    let expected = neox_bootnodes(network);
    let actual: Vec<&str> = value
        .get("Node")
        .and_then(|node| node.get("P2P"))
        .and_then(|p2p| p2p.get("BootstrapNodes"))
        .and_then(toml::Value::as_array)
        .map(|items| items.iter().filter_map(toml::Value::as_str).collect())
        .unwrap_or_default();

    if expected.is_empty() {
        if actual.is_empty() {
            report.pass("Bootnodes", "A private network carries no bootnodes.");
        } else {
            report.critical(
                "Bootnodes",
                "A private network must not carry public Neo X bootnodes.",
            );
        }
        return;
    }

    if actual.len() < expected.len() {
        report.critical(
            "Bootnodes",
            format!(
                "{} has {} peer(s); {network} publishes {}.",
                path.join("."),
                actual.len(),
                expected.len()
            ),
        );
        return;
    }
    match actual.iter().find(|peer| !is_enode(peer)) {
        Some(peer) => report.critical("Bootnodes", format!("`{peer}` is not an enode URL.")),
        None => report.pass(
            "Bootnodes",
            format!("{} peer(s) for {network}.", actual.len()),
        ),
    }
}

/// An `enode://` URL is `enode://<128 hex>@host:port`. A truncated one is a
/// fatal parse error inside geth, long after NeoNexus has said the config is
/// fine, so it is checked here instead.
fn is_enode(peer: &str) -> bool {
    let Some(rest) = peer.strip_prefix("enode://") else {
        return false;
    };
    let Some((id, endpoint)) = rest.split_once('@') else {
        return false;
    };
    id.len() == 128 && id.chars().all(|c| c.is_ascii_hexdigit()) && endpoint.contains(':')
}

/// Geth has no Neo X network preset, so the config alone cannot put a node on
/// the chain — the data directory has to be initialised from the published
/// genesis. That is an operator step NeoNexus cannot perform for them, and a
/// silent config is how a node ends up syncing Ethereum instead.
fn genesis_note(network: Network, report: &mut ConfigValidationReport) {
    match super::super::super::super::format::neox_genesis_hash(network) {
        Some(hash) => report.warning(
            "Genesis",
            format!(
                "Initialise the data directory from the published Neo X genesis before first \
                 launch; block 0 must hash to {hash}."
            ),
        ),
        None => report.warning(
            "Genesis",
            "A private Neo X network needs its own genesis file, shared by every member."
                .to_string(),
        ),
    }
}
