//! The neo-go YAML config shape.
//!
//! neo-go loads its config with `yaml.Decoder.KnownFields(true)`, so **any key
//! that is not a field of the target struct is a fatal startup error**, not a
//! warning. Every name here is taken from `pkg/config/*.go` on nspcc-dev/neo-go
//! and cross-checked against the shipped `config/protocol.mainnet.yml`; nothing
//! is inferred from the neo-cli spelling, which differs in several places.
//!
//! The traps this file exists to avoid:
//! - `P2P`, `RPC`, `Prometheus` and `Pprof` take `Addresses: ["host:port"]`.
//!   There is no `Address` + `Port` pair anywhere in neo-go.
//! - `Relay` is a flat key on `ApplicationConfiguration`. There is no `Node:`
//!   section, and `UserAgent` is not configurable at all.
//! - The logger keys are inline on `ApplicationConfiguration`, not nested under
//!   a `Logger:` map the way neo-cli nests them.
//! - Session expiry is `SessionLifetime`, a Go duration. `SessionExpirationTime`
//!   is the neo-cli name.

use serde::Serialize;

mod services;

pub(in crate::config::generator::neo_go) use services::{
    NeoGoInternalService, NeoGoOracle, NeoGoOracleNeoFs, NeoGoServices, NeoGoWallet,
};

/// A Go `time.Duration` as neo-go parses it from YAML: `"15s"`, `"3m"`, `"1h"`.
/// A bare number is *not* accepted for these fields, so the type exists to make
/// it impossible to emit one by accident.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub(super) struct GoDuration(String);

impl GoDuration {
    pub(super) fn seconds(value: u64) -> Self {
        Self(format!("{value}s"))
    }
}

/// A neo-go bind address. neo-go accepts `"[host]:port"` and treats an empty
/// host as "all interfaces", which is how the shipped configs express it.
pub(super) fn bind_address(host: &str, port: u16) -> String {
    format!("{host}:{port}")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoConfig {
    #[serde(rename = "ProtocolConfiguration")]
    pub(super) protocol_configuration: NeoGoProtocolConfiguration,
    #[serde(rename = "ApplicationConfiguration")]
    pub(super) application_configuration: NeoGoApplicationConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoProtocolConfiguration {
    #[serde(rename = "Magic")]
    pub(super) magic: u32,
    #[serde(rename = "SeedList")]
    pub(super) seed_list: Vec<String>,
    #[serde(rename = "StandbyCommittee")]
    pub(super) standby_committee: Vec<String>,
    #[serde(rename = "TimePerBlock")]
    pub(super) time_per_block: GoDuration,
    #[serde(rename = "MaxTransactionsPerBlock")]
    pub(super) max_transactions_per_block: u32,
    #[serde(rename = "ValidatorsCount")]
    pub(super) validators_count: u8,
    #[serde(rename = "VerifyTransactions")]
    pub(super) verify_transactions: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoApplicationConfiguration {
    #[serde(rename = "LogLevel")]
    pub(super) log_level: String,
    #[serde(rename = "LogEncoding")]
    pub(super) log_encoding: String,
    #[serde(rename = "LogPath", skip_serializing_if = "Option::is_none")]
    pub(super) log_path: Option<String>,
    #[serde(rename = "DBConfiguration")]
    pub(super) db_configuration: NeoGoDbConfiguration,
    #[serde(rename = "P2P")]
    pub(super) p2p: NeoGoP2pConfiguration,
    #[serde(rename = "Relay")]
    pub(super) relay: bool,
    #[serde(rename = "RPC")]
    pub(super) rpc: NeoGoRpcConfiguration,
    #[serde(rename = "Prometheus")]
    pub(super) prometheus: NeoGoBasicService,
    #[serde(rename = "Pprof")]
    pub(super) pprof: NeoGoPprofConfiguration,
    /// The duty sections. Flattened so each lands as a key of
    /// `ApplicationConfiguration`, and each is omitted when the node does not
    /// perform that duty.
    #[serde(flatten)]
    pub(super) services: NeoGoServices,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoDbConfiguration {
    #[serde(rename = "Type")]
    pub(super) db_type: String,
    #[serde(rename = "LevelDBOptions")]
    pub(super) leveldb_options: NeoGoLevelDbOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoLevelDbOptions {
    #[serde(rename = "DataDirectoryPath")]
    pub(super) data_directory_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoP2pConfiguration {
    #[serde(rename = "Addresses")]
    pub(super) addresses: Vec<String>,
    #[serde(rename = "DialTimeout")]
    pub(super) dial_timeout: GoDuration,
    #[serde(rename = "ProtoTickInterval")]
    pub(super) proto_tick_interval: GoDuration,
    #[serde(rename = "PingInterval")]
    pub(super) ping_interval: GoDuration,
    #[serde(rename = "PingTimeout")]
    pub(super) ping_timeout: GoDuration,
    #[serde(rename = "MinPeers")]
    pub(super) min_peers: u16,
    #[serde(rename = "MaxPeers")]
    pub(super) max_peers: u16,
    #[serde(rename = "AttemptConnPeers")]
    pub(super) attempt_conn_peers: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoRpcConfiguration {
    #[serde(rename = "Enabled")]
    pub(super) enabled: bool,
    #[serde(rename = "Addresses")]
    pub(super) addresses: Vec<String>,
    #[serde(rename = "EnableCORSWorkaround")]
    pub(super) enable_cors_workaround: bool,
    #[serde(rename = "MaxGasInvoke")]
    pub(super) max_gas_invoke: u32,
    #[serde(rename = "SessionEnabled")]
    pub(super) session_enabled: bool,
    #[serde(rename = "SessionLifetime")]
    pub(super) session_lifetime: GoDuration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoBasicService {
    #[serde(rename = "Enabled")]
    pub(super) enabled: bool,
    #[serde(rename = "Addresses")]
    pub(super) addresses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(super) struct NeoGoPprofConfiguration {
    #[serde(rename = "Enabled")]
    pub(super) enabled: bool,
    #[serde(rename = "Addresses")]
    pub(super) addresses: Vec<String>,
    #[serde(rename = "EnableBlock")]
    pub(super) enable_block: bool,
    #[serde(rename = "EnableMutex")]
    pub(super) enable_mutex: bool,
}
