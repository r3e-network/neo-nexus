//! Mempool and transaction pool backlog inspection.
//!
//! Evaluates whether a node's transaction pool is healthy, processing normally,
//! or facing unverified transaction backlogs / spam congestion.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::types::ChainFamily;

use super::{
    model::ChainQueryError,
    rpc::{agent, call},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MempoolCongestion {
    Normal,
    Elevated,
    Congested,
}

impl MempoolCongestion {
    pub fn label(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Elevated => "elevated",
            Self::Congested => "congested",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MempoolTelemetry {
    pub endpoint: String,
    pub family: ChainFamily,
    pub total_count: u64,
    pub verified_count: Option<u64>,
    pub unverified_count: Option<u64>,
    pub congestion: MempoolCongestion,
}

impl MempoolTelemetry {
    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("mempool-telemetry: {}", self.congestion.label()),
            format!("endpoint: {}", self.endpoint),
            format!("family: {}", self.family),
            format!("total-transactions: {}", self.total_count),
        ];
        if let Some(verified) = self.verified_count {
            lines.push(format!("verified-transactions: {verified}"));
        }
        if let Some(unverified) = self.unverified_count {
            lines.push(format!("unverified-transactions: {unverified}"));
        }
        lines.join("\n")
    }
}

/// Probes a node's transaction pool depth and congestion level.
pub fn mempool_telemetry(
    endpoint: &str,
    family: ChainFamily,
    timeout: Duration,
) -> Result<MempoolTelemetry, ChainQueryError> {
    let agent = agent(timeout);
    match family {
        ChainFamily::NeoN3 => probe_neo_n3_mempool(&agent, endpoint),
        ChainFamily::NeoX => probe_neox_mempool(&agent, endpoint),
    }
}

fn probe_neo_n3_mempool(
    agent: &ureq::Agent,
    endpoint: &str,
) -> Result<MempoolTelemetry, ChainQueryError> {
    // Try detailed query first: getrawmempool [true]
    if let Ok(res) = call(agent, endpoint, "getrawmempool", json!([true])) {
        if let Some(obj) = res.as_object() {
            let verified_count = obj
                .get("verified")
                .and_then(Value::as_array)
                .map(|a| a.len() as u64);
            let unverified_count = obj
                .get("unverified")
                .and_then(Value::as_array)
                .map(|a| a.len() as u64);
            let total_count = verified_count.unwrap_or(0) + unverified_count.unwrap_or(0);
            let congestion = classify_congestion(total_count);
            return Ok(MempoolTelemetry {
                endpoint: endpoint.to_string(),
                family: ChainFamily::NeoN3,
                total_count,
                verified_count,
                unverified_count,
                congestion,
            });
        }
        if let Some(arr) = res.as_array() {
            let total_count = arr.len() as u64;
            let congestion = classify_congestion(total_count);
            return Ok(MempoolTelemetry {
                endpoint: endpoint.to_string(),
                family: ChainFamily::NeoN3,
                total_count,
                verified_count: None,
                unverified_count: None,
                congestion,
            });
        }
    }

    // Fallback: getrawmempool []
    let res = call(agent, endpoint, "getrawmempool", json!([]))?;
    let total_count = res.as_array().map_or(0, |a| a.len() as u64);
    let congestion = classify_congestion(total_count);
    Ok(MempoolTelemetry {
        endpoint: endpoint.to_string(),
        family: ChainFamily::NeoN3,
        total_count,
        verified_count: None,
        unverified_count: None,
        congestion,
    })
}

fn probe_neox_mempool(
    agent: &ureq::Agent,
    endpoint: &str,
) -> Result<MempoolTelemetry, ChainQueryError> {
    // Try eth_getBlockTransactionCountByNumber(["pending"])
    let total_count = if let Ok(res) = call(
        agent,
        endpoint,
        "eth_getBlockTransactionCountByNumber",
        json!(["pending"]),
    ) {
        match &res {
            Value::String(hex) => parse_hex_u64(hex).unwrap_or(0),
            Value::Number(num) => num.as_u64().unwrap_or(0),
            _ => 0,
        }
    } else {
        // Fallback: txpool_status
        let res = call(agent, endpoint, "txpool_status", json!([]))?;
        let pending = res
            .get("pending")
            .and_then(Value::as_str)
            .and_then(parse_hex_u64)
            .unwrap_or(0);
        let queued = res
            .get("queued")
            .and_then(Value::as_str)
            .and_then(parse_hex_u64)
            .unwrap_or(0);
        pending + queued
    };

    let congestion = classify_congestion(total_count);
    Ok(MempoolTelemetry {
        endpoint: endpoint.to_string(),
        family: ChainFamily::NeoX,
        total_count,
        verified_count: None,
        unverified_count: None,
        congestion,
    })
}

fn classify_congestion(total_count: u64) -> MempoolCongestion {
    if total_count > 2000 {
        MempoolCongestion::Congested
    } else if total_count >= 500 {
        MempoolCongestion::Elevated
    } else {
        MempoolCongestion::Normal
    }
}

fn parse_hex_u64(value: &str) -> Option<u64> {
    let clean = value.strip_prefix("0x").unwrap_or(value);
    u64::from_str_radix(clean, 16).ok()
}

#[cfg(test)]
#[path = "../../tests/unit/chain_state/mempool/tests.rs"]
mod tests;
