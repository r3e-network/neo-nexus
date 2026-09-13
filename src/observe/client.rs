//! One timed JSON-RPC call, and an honest account of how it went.
//!
//! Three things this does that the liveness probe in `src/rpc_health/` does
//! not, each of which the observation layer depends on:
//!
//! 1. **It times the request.** `RpcHealthRecord` has no latency field and the
//!    probe never took one, which is why the node page could only ever print a
//!    constant where a round-trip figure belonged.
//! 2. **It tells "not implemented" apart from "not answering".** A JSON-RPC
//!    `-32601` means this client has no such method; a transport error means
//!    the node is down. Conflating them is what made a healthy Neo X node —
//!    which has no `getversion` — report as unreachable.
//! 3. **It bounds the response.** `getrawmempool(true)` on a congested chain
//!    is unbounded, and a node manager must not be the thing that runs its own
//!    host out of memory.

use std::{
    io::Read,
    time::{Duration, Instant},
};

use serde_json::{json, Value};

use super::evidence::{Evidence, NotSampled, Observation};

/// The most a single response may occupy.
///
/// Generous for the calls this layer makes — a mempool listing on a busy chain
/// is the only one that approaches it — and far below the point where reading
/// a reply threatens the host.
const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;

/// The JSON-RPC code for "this server has no such method".
///
/// A capability fact, not a fault. Every caller has to treat it differently
/// from a failure, so it is named here rather than repeated as a literal.
const METHOD_NOT_FOUND: i64 = -32601;

/// What one call produced.
pub(crate) struct TimedCall {
    /// The `result` field, or why there is none.
    pub(crate) value: Observation<Value>,
    /// Round-trip time. `None` when the call did not complete, because a
    /// failed request has no meaningful duration to report.
    pub(crate) latency_ms: Option<u32>,
}

/// Build the agent this layer calls with.
///
/// One agent per sampling round rather than per call: `ureq` pools connections
/// on the agent, and a node manager polling twenty nodes every fifteen seconds
/// should not open a fresh TCP connection for each.
pub(crate) fn agent(timeout: Duration) -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout(timeout)
        .timeout_connect(timeout)
        .build()
}

/// Call one method and record what came back.
///
/// `now_unix` is passed in rather than read here so that a caller replaying a
/// round — a test, or a rerun against stored input — produces identical
/// evidence.
pub(crate) fn call(
    agent: &ureq::Agent,
    endpoint: &str,
    method: &'static str,
    params: Value,
    now_unix: u64,
) -> TimedCall {
    let body = json!({
        "jsonrpc": "2.0",
        "id": "neonexus-observe",
        "method": method,
        "params": params,
    });

    let started = Instant::now();
    let response = agent
        .post(endpoint)
        .set("Content-Type", "application/json")
        .send_string(&body.to_string());
    let elapsed = started.elapsed();

    let response = match response {
        Ok(response) => response,
        // `ureq` reports a JSON-RPC server's 4xx/5xx as `Status`. The body may
        // still carry a structured error worth reading — a node behind a proxy
        // returning 500 with `-32601` is a capability fact, not an outage.
        Err(ureq::Error::Status(_, response)) => response,
        Err(error) => {
            return TimedCall {
                value: Observation::Unknown(NotSampled::CallFailed {
                    method,
                    detail: transport_detail(&error),
                }),
                latency_ms: None,
            };
        }
    };

    let mut text = String::new();
    if let Err(error) = response
        .into_reader()
        .take(MAX_RESPONSE_BYTES)
        .read_to_string(&mut text)
    {
        return TimedCall {
            value: Observation::Unknown(NotSampled::CallFailed {
                method,
                detail: format!("response could not be read: {error}"),
            }),
            latency_ms: None,
        };
    }

    let latency_ms = u32::try_from(elapsed.as_millis()).unwrap_or(u32::MAX);
    TimedCall {
        value: read_result(&text, method, endpoint, now_unix),
        latency_ms: Some(latency_ms),
    }
}

/// Turn a response body into a value or a reason there is none.
fn read_result(
    text: &str,
    method: &'static str,
    endpoint: &str,
    now_unix: u64,
) -> Observation<Value> {
    let parsed: Value = match serde_json::from_str(text) {
        Ok(parsed) => parsed,
        Err(error) => {
            return Observation::Unknown(NotSampled::CallFailed {
                method,
                detail: format!("reply was not JSON: {error}"),
            })
        }
    };

    if let Some(error) = parsed.get("error") {
        let code = error.get("code").and_then(Value::as_i64);
        if code == Some(METHOD_NOT_FOUND) {
            return Observation::Unknown(NotSampled::MethodUnsupported { method });
        }
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("no message");
        return Observation::Unknown(NotSampled::CallFailed {
            method,
            detail: match code {
                Some(code) => format!("{message} (code {code})"),
                None => message.to_string(),
            },
        });
    }

    match parsed.get("result") {
        Some(result) => Observation::Known(
            result.clone(),
            Evidence::recorded(method, "result", summarise(result), endpoint, now_unix),
        ),
        None => Observation::Unknown(NotSampled::CallFailed {
            method,
            detail: "reply carried neither a result nor an error".to_string(),
        }),
    }
}

/// The raw answer, short enough to store beside every sample.
///
/// Evidence is kept for every reading a node makes, so a `getpeers` array is
/// truncated rather than retained in full. What matters is that an operator can
/// see the shape of what was returned when a derived figure looks wrong.
fn summarise(value: &Value) -> String {
    const MAX: usize = 200;
    let text = match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    if text.chars().count() <= MAX {
        return text;
    }
    let head: String = text.chars().take(MAX).collect();
    format!("{head}… ({} bytes)", text.len())
}

/// A transport failure in the words an operator can act on.
///
/// `ureq`'s `Display` for a transport error is already a sentence; this drops
/// the URL from it, because the endpoint is recorded separately and repeating
/// it in every message makes a table of failures unreadable.
fn transport_detail(error: &ureq::Error) -> String {
    let text = error.to_string();
    text.split_once(": ")
        .map_or(text.clone(), |(_, detail)| detail.to_string())
}

#[cfg(test)]
#[path = "../../tests/unit/observe/client_tests.rs"]
mod tests;
