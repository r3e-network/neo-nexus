mod model;

use anyhow::{Context, Result};

use crate::types::{Network, NodeConfig};

use super::super::super::format::{neox_bootnodes, neox_chain_id, RuntimeConfigProfile};
use super::header;

use self::model::{GethConfig, GethEth, GethNode, GethP2p};

/// The JSON-RPC namespaces a managed node exposes over HTTP.
///
/// `admin`, `debug` and `personal` are deliberately absent: they can add peers,
/// rewrite chain state and unlock accounts, and NeoNexus binds the listener to
/// loopback rather than relying on nobody finding it.
const HTTP_MODULES: [&str; 4] = ["eth", "net", "web3", "txpool"];

pub(super) fn geth_toml(
    node: &NodeConfig,
    profile: Option<&RuntimeConfigProfile>,
) -> Result<String> {
    let bootstrap_nodes = neox_bootnodes(node.network);
    // A private network has no published peers to discover, so discovery would
    // only spray UDP at nothing. Its peers are whatever the operator wires up.
    let private = node.network == Network::Private;

    let config = GethConfig {
        eth: GethEth {
            network_id: neox_chain_id(node.network, profile),
            sync_mode: "snap".to_string(),
            state_scheme: "path".to_string(),
        },
        node: GethNode {
            http_host: "127.0.0.1".to_string(),
            http_port: node.rpc_port,
            http_modules: modules(),
            http_virtual_hosts: vec!["localhost".to_string()],
            ws_host: node.ws_port.map(|_| "127.0.0.1".to_string()),
            ws_port: node.ws_port,
            ws_modules: if node.ws_port.is_some() {
                modules()
            } else {
                Vec::new()
            },
            p2p: GethP2p {
                listen_addr: format!(":{}", node.p2p_port),
                max_peers: 50,
                no_discovery: private,
                bootstrap_nodes,
                static_nodes: Vec::new(),
                trusted_nodes: Vec::new(),
            },
        },
    };

    let body = toml::to_string_pretty(&config).context("failed to render Neo X Geth config")?;
    Ok(format!("{}{body}", header(&preamble(node.network))))
}

fn modules() -> Vec<String> {
    HTTP_MODULES.iter().map(ToString::to_string).collect()
}

/// What the file cannot express, said where the operator will read it.
///
/// Neo X Geth carries no built-in Neo X chain: `cmd/utils.NetworkFlags` still
/// lists only Ethereum's mainnet and testnets. The chain comes from the
/// published genesis JSON, applied once with `geth init`, and NeoNexus does not
/// generate that file — an allocation table it invented would produce a
/// perfectly healthy chain of one.
fn preamble(network: Network) -> String {
    let genesis = match network {
        Network::Mainnet => "config/genesis_mainnet.json",
        Network::Testnet => "config/genesis_testnet.json",
        Network::Private => return PRIVATE_PREAMBLE.to_string(),
    };
    format!(
        "Initialise the data directory once before first launch:\n\
         #   geth init --datadir <datadir> {genesis}\n\
         # from the Neo X Geth distribution. Neo X is not a built-in geth\n\
         # network, so a datadir initialised from any other genesis will sync a\n\
         # different chain that still reports this NetworkId."
    )
}

const PRIVATE_PREAMBLE: &str = "This is a private network: it has no published genesis and no \
                                bootnodes.\n# Initialise every member from the same genesis file, \
                                or they will fork\n# at block 0.";

#[cfg(test)]
#[path = "../../../../tests/unit/config/generator/neox/geth/tests.rs"]
mod tests;
