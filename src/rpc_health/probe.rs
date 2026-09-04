mod call;
mod endpoint;
mod methods;
mod summary;

use std::time::Duration;

use crate::types::{ChainFamily, NodeConfig};

use super::{RpcHealthReport, RpcHealthStatus};
use call::call_method;
use endpoint::normalize_endpoint;
use methods::probe_methods;
use summary::{method_health, summarize_version};

pub fn node_rpc_endpoint(node: &NodeConfig) -> String {
    endpoint::node_rpc_endpoint(node)
}

/// Probes a managed node, asking the methods its own chain family answers.
pub fn probe_node_rpc(node: &NodeConfig, timeout: Duration) -> RpcHealthReport {
    probe_rpc_endpoint_for(node.node_type.family(), &node_rpc_endpoint(node), timeout)
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
    let methods = probe_methods(family);
    let normalized_endpoint = normalize_endpoint(endpoint);
    let agent = ureq::AgentBuilder::new()
        .redirects(0)
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();

    let version_health =
        call_method(&agent, &normalized_endpoint, methods.version).and_then(|value| {
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
    let block_health =
        call_method(&agent, &normalized_endpoint, methods.height).and_then(|value| {
            if methods.block_count(&value).is_some() {
                Ok(value)
            } else {
                anyhow::bail!("invalid block count response")
            }
        });
    let sync_health = methods.syncing.map(|method| {
        call_method(&agent, &normalized_endpoint, method).and_then(|value| {
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
    if (syncing == Some(true) || sync_health.as_ref().is_some_and(Result::is_err))
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
        methods: method_reports,
    }
}
