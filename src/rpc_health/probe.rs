mod call;
mod endpoint;
mod summary;

use std::time::Duration;

use crate::types::NodeConfig;

use super::{RpcHealthReport, RpcHealthStatus};
use call::call_method;
use endpoint::normalize_endpoint;
use summary::{method_health, parse_block_count, summarize_version};

pub fn node_rpc_endpoint(node: &NodeConfig) -> String {
    endpoint::node_rpc_endpoint(node)
}

pub fn probe_node_rpc(node: &NodeConfig, timeout: Duration) -> RpcHealthReport {
    probe_rpc_endpoint(&node_rpc_endpoint(node), timeout)
}

pub fn probe_rpc_endpoint(endpoint: &str, timeout: Duration) -> RpcHealthReport {
    let normalized_endpoint = normalize_endpoint(endpoint);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();

    let version_health = call_method(&agent, &normalized_endpoint, "getversion");
    let block_health = call_method(&agent, &normalized_endpoint, "getblockcount");

    let version = version_health.as_ref().ok().and_then(summarize_version);
    let block_count = block_health.as_ref().ok().and_then(parse_block_count);

    let methods = vec![
        method_health("getversion", &version_health),
        method_health("getblockcount", &block_health),
    ];
    let ok_count = methods.iter().filter(|method| method.ok).count();
    let status = match ok_count {
        2 => RpcHealthStatus::Healthy,
        1 => RpcHealthStatus::Degraded,
        _ => RpcHealthStatus::Unreachable,
    };

    RpcHealthReport {
        endpoint: normalized_endpoint,
        status,
        version,
        block_count,
        methods,
    }
}
