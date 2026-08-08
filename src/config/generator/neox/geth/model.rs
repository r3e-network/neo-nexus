//! The Neo X Geth config-file shape.
//!
//! Keys are Go struct field names, verbatim: `cmd/geth/config.go` installs a
//! `toml.Config` whose `NormFieldName` and `FieldToKey` are both the identity
//! function, so `HTTPPort` is `HTTPPort` and nothing else.
//!
//! Its `MissingField` hook returns an error for any key it does not recognise
//! (only a short list of deprecated `ethconfig` fields is tolerated), so an
//! invented key here is a **fatal startup error**, exactly like neo-go. Every
//! field below was read off `eth/ethconfig.Config`, `node.Config` and
//! `p2p.Config` in bane-labs/go-ethereum.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct GethConfig {
    #[serde(rename = "Eth")]
    pub(super) eth: GethEth,
    #[serde(rename = "Node")]
    pub(super) node: GethNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct GethEth {
    /// The EIP-155 chain id. Geth has no Neo X network preset — its `--mainnet`
    /// and testnet flags are still Ethereum's — so this is what puts the node
    /// on Neo X, together with a datadir initialised from the published
    /// genesis file.
    #[serde(rename = "NetworkId")]
    pub(super) network_id: u64,
    #[serde(rename = "SyncMode")]
    pub(super) sync_mode: String,
    #[serde(rename = "StateScheme")]
    pub(super) state_scheme: String,
}

/// `node.Config`. `DataDir` is deliberately absent: geth's own default is
/// applied before the file is read, the launch plan passes `--datadir`, and an
/// empty `DataDir` would silently start an **ephemeral in-memory node**.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct GethNode {
    #[serde(rename = "HTTPHost")]
    pub(super) http_host: String,
    #[serde(rename = "HTTPPort")]
    pub(super) http_port: u16,
    #[serde(rename = "HTTPModules")]
    pub(super) http_modules: Vec<String>,
    #[serde(rename = "HTTPVirtualHosts")]
    pub(super) http_virtual_hosts: Vec<String>,
    /// A non-empty `WSHost` is what *starts* geth's WebSocket server, so the
    /// host and the port travel together. Writing the host alone would open a
    /// listener on geth's default 8546 — a port NeoNexus never reserved and
    /// another managed node may already hold.
    #[serde(rename = "WSHost", skip_serializing_if = "Option::is_none")]
    pub(super) ws_host: Option<String>,
    #[serde(rename = "WSPort", skip_serializing_if = "Option::is_none")]
    pub(super) ws_port: Option<u16>,
    #[serde(rename = "WSModules", skip_serializing_if = "Vec::is_empty")]
    pub(super) ws_modules: Vec<String>,
    /// Serialised last: TOML puts every scalar before the first sub-table, and
    /// serde writes fields in declaration order.
    #[serde(rename = "P2P")]
    pub(super) p2p: GethP2p,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct GethP2p {
    #[serde(rename = "ListenAddr")]
    pub(super) listen_addr: String,
    #[serde(rename = "MaxPeers")]
    pub(super) max_peers: u32,
    #[serde(rename = "NoDiscovery")]
    pub(super) no_discovery: bool,
    /// Discovery peers, as `enode://` URLs. Geth parses each into an
    /// `enode.Node` through its `UnmarshalText`, so a truncated URL is a
    /// startup error rather than a silently dropped peer.
    #[serde(rename = "BootstrapNodes")]
    pub(super) bootstrap_nodes: Vec<String>,
    #[serde(rename = "StaticNodes")]
    pub(super) static_nodes: Vec<String>,
    #[serde(rename = "TrustedNodes")]
    pub(super) trusted_nodes: Vec<String>,
}
