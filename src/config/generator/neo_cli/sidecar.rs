//! Per-plugin configuration files for neo-cli.
//!
//! neo-cli holds almost nothing about a capability in `config.json`. Each
//! plugin reads its own `Plugins/<Name>/<Name>.json`, and that is where the RPC
//! listener, the oracle service, the state service and dBFT consensus are
//! actually configured. A single-file export can therefore express none of
//! them — which is why the RPC port an operator set in NeoNexus never reached
//! the node.
//!
//! Every shape here is transcribed from the plugin's own shipped default in
//! neo-project/neo-node `plugins/<Name>/<Name>.json`. Two details are easy to
//! get wrong and are load-bearing:
//! - `Dependency` is a **sibling** of `PluginConfiguration`, not nested in it.
//! - neo-node 3.9.2's DBFTPlugin still requires `Network`; 3.10+ inherits it
//!   from the primary config. The runtime version controls that compatibility
//!   field.

use serde_json::{json, Value};

use crate::{
    catalog::PluginId,
    config::{ConsensusSigner, GenerationContext},
    types::NodeConfig,
};

/// One plugin configuration file, ready to be written beside the primary one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginSidecar {
    /// Path relative to the node directory, e.g. `Plugins/RpcServer/RpcServer.json`.
    pub relative_path: String,
    pub text: String,
}

/// Builds the configuration file for every enabled plugin that has one.
#[cfg(test)]
pub(super) fn sidecars_for(node: &NodeConfig, enabled: &[PluginId]) -> Vec<PluginSidecar> {
    sidecars_for_with_context(node, enabled, &GenerationContext::default())
}

pub(super) fn sidecars_for_with_context(
    node: &NodeConfig,
    enabled: &[PluginId],
    context: &GenerationContext,
) -> Vec<PluginSidecar> {
    let mut sidecars = enabled
        .iter()
        .filter_map(|plugin| {
            configuration(node, *plugin, context).map(|value| sidecar(*plugin, &value))
        })
        .collect::<Vec<_>>();
    if let Some(ConsensusSigner::SignClient {
        name,
        endpoint,
        public_key,
        network_magic,
    }) = &context.consensus_signer
    {
        sidecars.push(named_sidecar("SignClient", &sign_client(name, endpoint)));
        sidecars.push(PluginSidecar {
            relative_path: "Plugins/NeoNexus.SignerBootstrap/SignerBootstrap.json".to_string(),
            text: json_text(&signer_bootstrap(
                name,
                endpoint,
                public_key,
                *network_magic,
            )),
        });
    }
    sidecars
}

fn sidecar(plugin: PluginId, value: &Value) -> PluginSidecar {
    let name = plugin.to_string();
    named_sidecar(&name, value)
}

fn named_sidecar(name: &str, value: &Value) -> PluginSidecar {
    PluginSidecar {
        relative_path: format!("Plugins/{name}/{name}.json"),
        text: json_text(value),
    }
}

fn json_text(value: &Value) -> String {
    format!(
        "{}\n",
        serde_json::to_string_pretty(value).unwrap_or_default()
    )
}

/// The plugin's file contents, or `None` for plugins that ship no config of
/// their own (the two storage engines are selected in the primary config).
fn configuration(
    node: &NodeConfig,
    plugin: PluginId,
    context: &GenerationContext,
) -> Option<Value> {
    match plugin {
        PluginId::RpcServer => Some(rpc_server(node)),
        PluginId::RestServer => Some(rest_server(node)),
        PluginId::OracleService => Some(oracle_service()),
        PluginId::StateService => Some(state_service()),
        PluginId::DBFTPlugin => Some(dbft(node, context)),
        PluginId::ApplicationLogs => Some(application_logs()),
        PluginId::TokensTracker => Some(tokens_tracker()),
        PluginId::LevelDbStore | PluginId::RocksDbStore => None,
    }
}

/// The RPC listener. This is the only place the port an operator chose in
/// NeoNexus has any effect on a neo-cli node.
fn rpc_server(node: &NodeConfig) -> Value {
    json!({
        "PluginConfiguration": {
            "UnhandledExceptionPolicy": "Ignore",
            "Servers": [{
                "BindAddress": "127.0.0.1",
                "Port": node.rpc_port,
                "SslCert": "",
                "SslCertPassword": "",
                "TrustedAuthorities": [],
                "RpcUser": "",
                "RpcPass": "",
                "EnableCors": true,
                "AllowOrigins": [],
                "KeepAliveTimeout": 60,
                "RequestHeadersTimeout": 15,
                "MaxGasInvoke": 20,
                "MaxFee": 0.1,
                "MaxConcurrentConnections": 40,
                "MaxIteratorResultItems": 100,
                "MaxStackSize": 65535,
                "DisabledMethods": ["openwallet"],
                "SessionEnabled": false,
                "SessionExpirationTime": 60,
                "FindStoragePageSize": 50
            }]
        }
    })
}

/// The REST listener sits on its own port, defaulted next to the WebSocket
/// port so a fleet on one host does not collide.
fn rest_server(node: &NodeConfig) -> Value {
    let port = node.ws_port.unwrap_or(node.rpc_port.saturating_add(7));
    json!({
        "PluginConfiguration": {
            "BindAddress": "127.0.0.1",
            "Port": port,
            "KeepAliveTimeout": 120,
            "EnableCors": true,
            "AllowOrigins": [],
            "EnableCompression": true,
            "EnableSwagger": true,
            "MaxPageSize": 50,
            "MaxConcurrentConnections": 40,
            "MaxGasInvoke": 200_000_000
        },
        "Dependency": ["RpcServer"]
    })
}

/// The oracle service. `AutoStart` stays false: an oracle node only responds
/// once the committee has designated its key, and starting before that logs
/// failures on every request. `Nodes` is the set of oracle peers responses are
/// broadcast to and is filled in by the operator.
fn oracle_service() -> Value {
    json!({
        "PluginConfiguration": {
            "Nodes": [],
            "MaxTaskTimeout": 432_000_000u64,
            "MaxOracleTimeout": 10_000,
            "AllowPrivateHost": false,
            "AllowedContentTypes": ["application/json"],
            "UnhandledExceptionPolicy": "Ignore",
            "Https": { "Timeout": 5_000 },
            "NeoFS": { "EndPoint": "http://127.0.0.1:8080", "Timeout": 15_000 },
            "AutoStart": false
        },
        "Dependency": ["RpcServer"]
    })
}

/// State roots and MPT proofs. `FullState` keeps the whole trie rather than the
/// latest state, which is what makes historical proofs answerable; `AutoVerify`
/// stays off because signing state roots needs a designated StateValidator key.
fn state_service() -> Value {
    json!({
        "PluginConfiguration": {
            "Path": "Data_MPT_{0}",
            "FullState": true,
            "AutoVerify": false,
            "MaxFindResultItems": 100,
            "UnhandledExceptionPolicy": "StopPlugin"
        },
        "Dependency": ["RpcServer"]
    })
}

/// dBFT consensus duties.
fn dbft(node: &NodeConfig, context: &GenerationContext) -> Value {
    let auto_start = matches!(context.consensus_signer, Some(ConsensusSigner::LocalWallet));
    let mut value = json!({
        "PluginConfiguration": {
            "RecoveryLogs": "ConsensusState",
            "IgnoreRecoveryLogs": false,
            "AutoStart": auto_start,
            "MaxBlockSize": 2_097_152,
            "MaxBlockSystemFee": 2_000_000_000u64,
            "UnhandledExceptionPolicy": "StopNode"
        }
    });
    if dbft_requires_network(&node.runtime_version) {
        value["PluginConfiguration"]["Network"] = json!(crate::config::network_magic(node.network));
    }
    value
}

fn dbft_requires_network(runtime_version: &str) -> bool {
    let version = runtime_version.trim().trim_start_matches('v');
    let mut parts = version.split('.');
    match (
        parts.next().and_then(|part| part.parse::<u32>().ok()),
        parts.next().and_then(|part| part.parse::<u32>().ok()),
    ) {
        (Some(major), Some(minor)) => major < 3 || (major == 3 && minor <= 9),
        _ => false,
    }
}

fn sign_client(name: &str, endpoint: &str) -> Value {
    json!({
        "PluginConfiguration": {
            "Name": name,
            "Endpoint": endpoint
        }
    })
}

fn signer_bootstrap(name: &str, endpoint: &str, public_key: &str, network_magic: u32) -> Value {
    json!({
        "PluginConfiguration": {
            "Network": network_magic,
            "PublicKey": public_key,
            "Endpoint": endpoint,
            "SignerName": name,
            "StartupTimeoutSeconds": 3_600
        }
    })
}

fn application_logs() -> Value {
    json!({
        "PluginConfiguration": {
            "Path": "ApplicationLogs_{0}",
            "MaxStackSize": 65535,
            "Debug": false,
            "UnhandledExceptionPolicy": "StopPlugin"
        },
        "Dependency": ["RpcServer"]
    })
}

fn tokens_tracker() -> Value {
    json!({
        "PluginConfiguration": {
            "DBPath": "TokenBalanceData",
            "TrackHistory": true,
            "MaxResults": 1000,
            "EnabledTrackers": ["NEP-11", "NEP-17"],
            "UnhandledExceptionPolicy": "StopPlugin"
        },
        "Dependency": ["RpcServer"]
    })
}

#[cfg(test)]
#[path = "../../../../tests/unit/config/generator/neo_cli/sidecar/tests.rs"]
mod tests;
