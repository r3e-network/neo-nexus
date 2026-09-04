mod call;
mod endpoint;
mod methods;
mod summary;

use std::time::{Duration, Instant};

use crate::types::{ChainFamily, Network, NodeConfig};

use super::{RpcHealthReport, RpcHealthStatus, RpcIdentityKind, RpcNetworkObservation};
use call::call_method;
use endpoint::normalize_endpoint;
use methods::probe_methods;
use summary::{method_health, summarize_version};

pub fn node_rpc_endpoint(node: &NodeConfig) -> String {
    endpoint::node_rpc_endpoint(node)
}

/// Probes a managed node, asking the methods its own chain family answers.
pub fn probe_node_rpc(node: &NodeConfig, timeout: Duration) -> RpcHealthReport {
    probe_rpc_endpoint_for_network(
        node.node_type.family(),
        Some(node.network),
        &node_rpc_endpoint(node),
        timeout,
    )
}

/// Probes a bare endpoint with no node behind it — a remote federation peer,
/// or an address typed at the CLI. Those are Neo N3 by default; a Neo X
/// endpoint has to say so, because no probe can tell from a URL alone.
pub fn probe_rpc_endpoint(endpoint: &str, timeout: Duration) -> RpcHealthReport {
    probe_rpc_endpoint_for(ChainFamily::NeoN3, endpoint, timeout)
}

/// Probes an endpoint known to belong to `family`.
pub fn probe_rpc_endpoint_for(
    family: ChainFamily,
    endpoint: &str,
    timeout: Duration,
) -> RpcHealthReport {
    probe_rpc_endpoint_for_network(family, None, endpoint, timeout)
}

fn probe_rpc_endpoint_for_network(
    family: ChainFamily,
    network: Option<Network>,
    endpoint: &str,
    timeout: Duration,
) -> RpcHealthReport {
    let methods = probe_methods(family);
    let started = Instant::now();
    let deadline = started.checked_add(timeout).unwrap_or(started);
    let normalized_endpoint = normalize_endpoint(endpoint);
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();

    let version_health = call_method(&agent, &normalized_endpoint, methods.version, deadline)
        .and_then(|value| {
            let valid = match family {
                ChainFamily::NeoN3 => {
                    value.is_object()
                        && summarize_version(&value).is_some_and(|text| !text.trim().is_empty())
                }
                ChainFamily::NeoX => value.as_str().is_some_and(|text| !text.trim().is_empty()),
            };
            if valid {
                Ok(value)
            } else {
                anyhow::bail!("invalid client version response")
            }
        });
    let block_health = call_method(&agent, &normalized_endpoint, methods.height, deadline)
        .and_then(|value| {
            if methods.block_count(&value).is_some() {
                Ok(value)
            } else {
                anyhow::bail!("invalid block count response")
            }
        });
    let sync_health = methods.syncing.map(|method| {
        call_method(&agent, &normalized_endpoint, method, deadline).and_then(|value| {
            if methods::syncing_verdict(&value).is_some() {
                Ok(value)
            } else {
                anyhow::bail!("invalid synchronization response")
            }
        })
    });
    let syncing = sync_health
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .and_then(methods::syncing_verdict);

    let identity_health = methods.identity.map(|method| {
        call_method(&agent, &normalized_endpoint, method, deadline).and_then(|value| {
            if methods::hex_quantity(&value).is_some() {
                Ok(value)
            } else {
                anyhow::bail!("invalid chain identity response")
            }
        })
    });
    let peer_health =
        call_method(&agent, &normalized_endpoint, methods.peers, deadline).and_then(|value| {
            if methods.peer_count(&value).is_some() {
                Ok(value)
            } else {
                anyhow::bail!("invalid peer count response")
            }
        });
    let actual_identity = match family {
        ChainFamily::NeoN3 => version_health
            .as_ref()
            .ok()
            .and_then(|value| value.pointer("/protocol/network"))
            .and_then(serde_json::Value::as_u64)
            .filter(|number| u32::try_from(*number).is_ok()),
        ChainFamily::NeoX => identity_health
            .as_ref()
            .and_then(|result| result.as_ref().ok())
            .and_then(methods::hex_quantity),
    };
    let observation = RpcNetworkObservation {
        identity_kind: Some(match family {
            ChainFamily::NeoN3 => RpcIdentityKind::N3NetworkMagic,
            ChainFamily::NeoX => RpcIdentityKind::EvmChainId,
        }),
        actual_identity,
        // Private profiles and bare endpoints have no inferred identity anchor.
        expected_identity: network
            .and_then(|network| super::expected_public_identity(family, network)),
        peer_count: peer_health
            .as_ref()
            .ok()
            .and_then(|value| methods.peer_count(value)),
        peers_expected: matches!(network, Some(Network::Mainnet | Network::Testnet)),
    };

    let version = version_health.as_ref().ok().and_then(summarize_version);
    let block_count = block_health
        .as_ref()
        .ok()
        .and_then(|value| methods.block_count(value));

    let mut method_reports = vec![
        method_health(methods.version, &version_health),
        method_health(methods.height, &block_health),
    ];
    let ok_count = method_reports.iter().filter(|method| method.ok).count();
    let mut status = match ok_count {
        2 => RpcHealthStatus::Healthy,
        1 => RpcHealthStatus::Degraded,
        _ => RpcHealthStatus::Unreachable,
    };
    if let (Some(method), Some(result)) = (methods.syncing, &sync_health) {
        method_reports.push(method_health(method, result));
    }
    if let (Some(method), Some(result)) = (methods.identity, &identity_health) {
        method_reports.push(method_health(method, result));
    }
    method_reports.push(method_health(methods.peers, &peer_health));
    if (syncing == Some(true)
        || sync_health.as_ref().is_some_and(Result::is_err)
        || observation.requires_attention())
        && status == RpcHealthStatus::Healthy
    {
        // A reachable node that is still catching up is not Healthy: both
        // scored methods answer while the chain it serves is behind.
        status = RpcHealthStatus::Degraded;
    }

    RpcHealthReport {
        endpoint: normalized_endpoint,
        status,
        version,
        block_count,
        syncing,
        network: observation,
        methods: method_reports,
    }
}
