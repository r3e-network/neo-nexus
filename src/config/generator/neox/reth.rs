//! The neox-rs config file.
//!
//! neox-rs is a Reth fork, and Reth splits its settings in two: the TOML file
//! holds pipeline and peering *tuning* (`[stages] [prune] [peers] [sessions]
//! [static_files]` — see `Config` in `crates/config/src/config.rs`), while the
//! chain, the data directory and every listening port are **command-line
//! flags**. There is no key in this file for an RPC port.
//!
//! So this generator writes what the file can actually hold, and the launch
//! plan carries the rest. Emitting a `port` here would look configured and do
//! nothing — the failure neo-rs already taught us, since serde ignores unknown
//! keys rather than rejecting them.

use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::{Network, NodeConfig};

use super::super::super::format::{neox_reth_chain, RuntimeConfigProfile};
use super::header;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RethConfig {
    peers: RethPeers,
}

/// `PeersConfig` carries `#[serde(default)]`, so a partial table is valid and
/// every unwritten field keeps the client's own default.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RethPeers {
    trusted_nodes: Vec<String>,
    trusted_nodes_only: bool,
    connection_info: RethConnections,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct RethConnections {
    max_outbound: usize,
    max_inbound: usize,
}

pub(super) fn reth_toml(
    node: &NodeConfig,
    _profile: Option<&RuntimeConfigProfile>,
) -> Result<String> {
    let config = RethConfig {
        peers: RethPeers {
            trusted_nodes: Vec::new(),
            // A private network with no trusted peers and this set would
            // refuse every connection, so it stays off until an operator
            // actually lists peers.
            trusted_nodes_only: false,
            connection_info: RethConnections {
                max_outbound: 100,
                max_inbound: 30,
            },
        },
    };

    let body = toml::to_string_pretty(&config).context("failed to render neox-rs config")?;
    Ok(format!("{}{body}", header(&preamble(node))))
}

/// The flags that carry everything this file cannot.
fn preamble(node: &NodeConfig) -> String {
    let chain = neox_reth_chain(node.network).map_or_else(
        || "<your chainspec>.json    # a private network needs its own genesis".to_string(),
        ToString::to_string,
    );
    let discovery = match node.network {
        Network::Private => "\n#   --disable-discovery",
        _ => "",
    };
    format!(
        "neox-rs takes its chain, data directory and ports as flags, not as\n\
         # config keys. The launch plan passes:\n\
         #   node --chain {chain}\n\
         #   --http --http.addr 127.0.0.1 --http.port {rpc}\n\
         #   --port {p2p}{discovery}\n\
         # The chain spec is compiled into the client, so unlike Neo X Geth\n\
         # this node needs no genesis file.",
        rpc = node.rpc_port,
        p2p = node.p2p_port,
    )
}

#[cfg(test)]
#[path = "../../../../tests/unit/config/generator/neox/reth/tests.rs"]
mod tests;
