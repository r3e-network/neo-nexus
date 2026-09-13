//! Running one round of classes against one node.
//!
//! The rule that shapes this file: **a class that fails does not void the
//! round.** A round that read the height but could not read the mempool is a
//! good round, and the mempool column says why it is empty. The probe this
//! replaces derived a node's whole status from "how many of two calls
//! answered", so one unimplemented method condemned a node that was working.

use serde_json::{json, Value};

use super::{neo_n3, neox, NodeSample, SampleClass};
use crate::{
    observe::{
        client::{self, TimedCall},
        evidence::{Evidence, NotSampled, Observation},
    },
    types::{ChainFamily, NodeConfig},
};

/// Where this workspace reaches a node.
///
/// Every node is a local child process today, so this is loopback. It is a
/// function rather than an interpolation at each call site because the host a
/// node runs on is the next thing to become real, and there are otherwise a
/// dozen places that would each need finding.
fn endpoint_for(node: &NodeConfig) -> String {
    format!("http://127.0.0.1:{}", node.rpc_port)
}

/// Ask one node the classes that are due.
pub(crate) fn sample_node(
    agent: &ureq::Agent,
    node: &NodeConfig,
    due: &[SampleClass],
    now_unix: u64,
) -> NodeSample {
    // Nothing to ask. Not a failure, and it must not read as one: an operator
    // who turned RPC off should see that decision reflected, not an outage.
    if node.rpc_port == 0 {
        return NodeSample::not_observable(&node.id, now_unix);
    }

    let endpoint = endpoint_for(node);
    let family = node.node_type.family();
    let mut sample =
        NodeSample::unreachable(&node.id, now_unix, &endpoint, NotSampled::NeverSampled);

    for class in due {
        match class {
            SampleClass::Head => read_head(agent, &endpoint, family, now_unix, &mut sample),
            SampleClass::HeadTime => {
                read_head_time(agent, &endpoint, family, now_unix, &mut sample)
            }
            SampleClass::Peers => read_peers(agent, &endpoint, family, now_unix, &mut sample),
            SampleClass::Identity => read_identity(agent, &endpoint, family, now_unix, &mut sample),
            SampleClass::Pool => read_pool(agent, &endpoint, family, now_unix, &mut sample),
        }
    }
    sample
}

/// Convert a call's raw answer into a typed reading.
///
/// The reason for absence is carried across unchanged: a `-32601` stays a
/// capability fact, a transport error stays a failure, and a reply that parsed
/// but did not contain what was expected becomes its own failure rather than
/// silently vanishing.
fn read<T>(
    call: &TimedCall,
    method: &'static str,
    parse: impl FnOnce(&Value) -> Option<T>,
) -> Observation<T> {
    match &call.value {
        Observation::Known(value, evidence) => match parse(value) {
            Some(parsed) => Observation::Known(parsed, evidence.clone()),
            None => Observation::Unknown(NotSampled::CallFailed {
                method,
                detail: format!("reply was not in the expected shape: {}", evidence.value()),
            }),
        },
        Observation::Unknown(reason) => Observation::Unknown(reason.clone()),
        Observation::Unanswerable(reason) => Observation::Unanswerable(reason),
    }
}

/// The liveness class. This one decides whether the node answered at all.
fn read_head(
    agent: &ureq::Agent,
    endpoint: &str,
    family: ChainFamily,
    now: u64,
    sample: &mut NodeSample,
) {
    match family {
        ChainFamily::NeoN3 => {
            let count = client::call(agent, endpoint, neo_n3::BLOCK_COUNT, json!([]), now);
            sample.head_ok = count.value.is_known();
            sample.head_latency_ms = count.latency_ms;
            sample.block_height = read(&count, neo_n3::BLOCK_COUNT, neo_n3::block_count);

            // Headers run ahead of blocks while a Neo N3 node syncs, so the gap
            // between them is a sync signal that needs no reference node.
            let headers = client::call(agent, endpoint, neo_n3::HEADER_COUNT, json!([]), now);
            sample.header_height = read(&headers, neo_n3::HEADER_COUNT, neo_n3::block_count);
        }
        ChainFamily::NeoX => {
            let number = client::call(agent, endpoint, neox::BLOCK_NUMBER, json!([]), now);
            sample.head_ok = number.value.is_known();
            sample.head_latency_ms = number.latency_ms;
            sample.block_height = read(&number, neox::BLOCK_NUMBER, neox::block_count);

            let syncing = client::call(agent, endpoint, neox::SYNCING, json!([]), now);
            sample.syncing = read(&syncing, neox::SYNCING, neox::syncing);
            // A syncing node names the head it is chasing, which is a reference
            // available even to a workspace holding a single node.
            if let Observation::Known(value, evidence) = &syncing.value {
                if let Some(target) = neox::sync_target(value) {
                    sample.header_height = Observation::Known(target, evidence.clone());
                }
            }
        }
    }
}

fn read_head_time(
    agent: &ureq::Agent,
    endpoint: &str,
    family: ChainFamily,
    now: u64,
    sample: &mut NodeSample,
) {
    match family {
        ChainFamily::NeoN3 => {
            // The newest block's index is one below the count.
            let Some(height) = sample.block_height.value().copied() else {
                return;
            };
            let index = height.saturating_sub(1);
            let header = client::call(
                agent,
                endpoint,
                neo_n3::BLOCK_HEADER,
                json!([index, true]),
                now,
            );
            sample.head_block_time_unix =
                read(&header, neo_n3::BLOCK_HEADER, neo_n3::block_time_unix);
        }
        ChainFamily::NeoX => {
            let block = client::call(
                agent,
                endpoint,
                neox::BLOCK_BY_NUMBER,
                json!(["latest", false]),
                now,
            );
            sample.head_block_time_unix =
                read(&block, neox::BLOCK_BY_NUMBER, neox::block_time_unix);
        }
    }
}

fn read_peers(
    agent: &ureq::Agent,
    endpoint: &str,
    family: ChainFamily,
    now: u64,
    sample: &mut NodeSample,
) {
    sample.peers_connected = match family {
        ChainFamily::NeoN3 => {
            let call = client::call(agent, endpoint, neo_n3::CONNECTION_COUNT, json!([]), now);
            read(&call, neo_n3::CONNECTION_COUNT, |value| {
                value.as_u64().and_then(|count| u32::try_from(count).ok())
            })
        }
        ChainFamily::NeoX => {
            let call = client::call(agent, endpoint, neox::PEER_COUNT, json!([]), now);
            read(&call, neox::PEER_COUNT, neox::peer_count)
        }
    };
}

/// The protocol constants every threshold derives from.
fn read_identity(
    agent: &ureq::Agent,
    endpoint: &str,
    family: ChainFamily,
    now: u64,
    sample: &mut NodeSample,
) {
    match family {
        ChainFamily::NeoN3 => {
            let call = client::call(agent, endpoint, neo_n3::VERSION, json!([]), now);
            let Observation::Known(value, evidence) = &call.value else {
                let absent = read(&call, neo_n3::VERSION, |_: &Value| None::<u64>);
                sample.observed_magic = absent.clone();
                sample.ms_per_block = absent.clone();
                sample.mempool_capacity = absent;
                return;
            };
            let protocol = neo_n3::protocol(value);
            let keep = |field: &'static str, raw: String| {
                Evidence::recorded(neo_n3::VERSION, field, raw, endpoint, now)
            };
            sample.observed_magic = to_observation(
                protocol.magic,
                "protocol.network",
                &keep,
                neo_n3::VERSION,
                evidence,
            );
            sample.ms_per_block = to_observation(
                protocol.ms_per_block,
                "protocol.msperblock",
                &keep,
                neo_n3::VERSION,
                evidence,
            );
            sample.mempool_capacity = to_observation(
                protocol.mempool_capacity,
                "protocol.memorypoolmaxtransactions",
                &keep,
                neo_n3::VERSION,
                evidence,
            );
            sample.validators_count = to_observation(
                protocol.validators_count,
                "protocol.validatorscount",
                &keep,
                neo_n3::VERSION,
                evidence,
            );
            sample.client_version = to_observation(
                protocol.user_agent,
                "useragent",
                &keep,
                neo_n3::VERSION,
                evidence,
            );
        }
        ChainFamily::NeoX => {
            let chain_id = client::call(agent, endpoint, neox::CHAIN_ID, json!([]), now);
            sample.observed_magic = read(&chain_id, neox::CHAIN_ID, neox::hex_quantity);
            let version = client::call(agent, endpoint, neox::CLIENT_VERSION, json!([]), now);
            sample.client_version = read(&version, neox::CLIENT_VERSION, |value| {
                value.as_str().map(str::to_string)
            });
        }
    }
}

/// Lift a parsed field into an observation, preserving the reason a sibling
/// field is missing rather than reporting the whole call as failed.
fn to_observation<T: std::fmt::Display>(
    parsed: Option<T>,
    field: &'static str,
    keep: &impl Fn(&'static str, String) -> Evidence,
    method: &'static str,
    _call_evidence: &Evidence,
) -> Observation<T> {
    match parsed {
        Some(value) => {
            let evidence = keep(field, value.to_string());
            Observation::Known(value, evidence)
        }
        None => Observation::Unknown(NotSampled::CallFailed {
            method,
            detail: format!("reply carried no {field}"),
        }),
    }
}

fn read_pool(
    agent: &ureq::Agent,
    endpoint: &str,
    family: ChainFamily,
    now: u64,
    sample: &mut NodeSample,
) {
    match family {
        ChainFamily::NeoN3 => {
            let call = client::call(agent, endpoint, neo_n3::RAW_MEMPOOL, json!([true]), now);
            match &call.value {
                Observation::Known(value, _) => {
                    let (verified, unverified) = neo_n3::mempool_counts(value);
                    sample.mempool_verified = verified;
                    sample.mempool_unverified = unverified;
                }
                _ => {
                    let absent = read(&call, neo_n3::RAW_MEMPOOL, |_: &Value| None::<u64>);
                    sample.mempool_verified = absent.clone();
                    sample.mempool_unverified = absent;
                }
            }
        }
        ChainFamily::NeoX => {
            let call = client::call(agent, endpoint, neox::TXPOOL_STATUS, json!([]), now);
            match &call.value {
                Observation::Known(value, evidence) => {
                    let (pending, queued) = neox::pool_counts(value);
                    sample.mempool_verified = pending.map_or_else(
                        || {
                            Observation::Unknown(NotSampled::CallFailed {
                                method: neox::TXPOOL_STATUS,
                                detail: "reply carried no pending count".to_string(),
                            })
                        },
                        |count| Observation::Known(count, evidence.clone()),
                    );
                    sample.mempool_unverified = queued.map_or_else(
                        || {
                            Observation::Unknown(NotSampled::CallFailed {
                                method: neox::TXPOOL_STATUS,
                                detail: "reply carried no queued count".to_string(),
                            })
                        },
                        |count| Observation::Known(count, evidence.clone()),
                    );
                }
                _ => {
                    let absent = read(&call, neox::TXPOOL_STATUS, |_: &Value| None::<u64>);
                    sample.mempool_verified = absent.clone();
                    sample.mempool_unverified = absent;
                }
            }
        }
    }
}
