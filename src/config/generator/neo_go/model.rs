use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoConfig {
    #[serde(rename = "ProtocolConfiguration")]
    pub(super) protocol_configuration: NeoGoProtocolConfiguration,
    #[serde(rename = "ApplicationConfiguration")]
    pub(super) application_configuration: NeoGoApplicationConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoProtocolConfiguration {
    pub(super) magic: u32,
    pub(super) seed_list: Vec<String>,
    pub(super) standby_committee: Vec<String>,
    pub(super) time_per_block: String,
    pub(super) max_transactions_per_block: u32,
    pub(super) validators_count: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoApplicationConfiguration {
    #[serde(rename = "DBConfiguration")]
    pub(super) db_configuration: NeoGoDbConfiguration,
    #[serde(rename = "P2P")]
    pub(super) p2p: NeoGoP2pConfiguration,
    #[serde(rename = "RPC")]
    pub(super) rpc: NeoGoRpcConfiguration,
    #[serde(rename = "Pprof")]
    pub(super) pprof: NeoGoPprofConfiguration,
    #[serde(rename = "Node")]
    pub(super) node: NeoGoNodeConfiguration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoDbConfiguration {
    #[serde(rename = "Type")]
    pub(super) db_type: String,
    #[serde(rename = "LevelDBOptions")]
    pub(super) leveldb_options: NeoGoLevelDbOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoLevelDbOptions {
    pub(super) data_directory_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoP2pConfiguration {
    pub(super) address: String,
    pub(super) port: u16,
    pub(super) dial_timeout: String,
    pub(super) proto_tick_interval: String,
    pub(super) ping_interval: String,
    pub(super) ping_timeout: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoRpcConfiguration {
    pub(super) enabled: bool,
    pub(super) enable_cors_workaround: bool,
    pub(super) max_gas_invoke: i64,
    pub(super) session_enabled: bool,
    pub(super) session_expiration_time: u32,
    pub(super) address: String,
    pub(super) port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoPprofConfiguration {
    pub(super) enabled: bool,
    pub(super) address: String,
    pub(super) port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct NeoGoNodeConfiguration {
    pub(super) relay: bool,
    pub(super) user_agent: String,
}
