//! P2P peer topology and connectivity inspection.
//!
//! Evaluates whether a node is actively connected to the peer-to-peer network
//! or dangerously isolated (0 peers / network partition).

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
pub enum PeerConnectivity {
    Healthy,
    Sparse,
    Isolated,
}

impl PeerConnectivity {
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Sparse => "sparse",
            Self::Isolated => "isolated",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerEndpoint {
    pub address: String,
    pub port: Option<u16>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerTelemetry {
    pub endpoint: String,
    pub family: ChainFamily,
    pub connected_count: u64,
    pub unconnected_count: Option<u64>,
    pub bad_count: Option<u64>,
    pub connectivity: PeerConnectivity,
    pub sample_peers: Vec<PeerEndpoint>,
}

impl PeerTelemetry {
    pub fn to_cli_text(&self) -> String {
        let mut lines = vec![
            format!("peer-telemetry: {}", self.connectivity.label()),
            format!("endpoint: {}", self.endpoint),
            format!("family: {}", self.family),
            format!("connected-peers: {}", self.connected_count),
        ];
        if let Some(unconnected) = self.unconnected_count {
            lines.push(format!("unconnected-peers: {unconnected}"));
        }
        if let Some(bad) = self.bad_count {
            lines.push(format!("bad-peers: {bad}"));
        }
        for peer in &self.sample_peers {
            if let Some(port) = peer.port {
                lines.push(format!("peer: {}:{}", peer.address, port));
            } else {
                lines.push(format!("peer: {}", peer.address));
            }
        }
        lines.join("\n")
    }
}

/// Probes a node's P2P connectivity status and peer counts.
pub fn peer_telemetry(
    endpoint: &str,
    family: ChainFamily,
    timeout: Duration,
) -> Result<PeerTelemetry, ChainQueryError> {
    let agent = agent(timeout);
    match family {
        ChainFamily::NeoN3 => probe_neo_n3_peers(&agent, endpoint),
        ChainFamily::NeoX => probe_neox_peers(&agent, endpoint),
    }
}

fn probe_neo_n3_peers(
    agent: &ureq::Agent,
    endpoint: &str,
) -> Result<PeerTelemetry, ChainQueryError> {
    if let Ok(peers_val) = call(agent, endpoint, "getpeers", json!([])) {
        let connected_array = peers_val.get("connected").and_then(Value::as_array);
        let unconnected_array = peers_val.get("unconnected").and_then(Value::as_array);
        let bad_array = peers_val.get("bad").and_then(Value::as_array);

        let connected_count = connected_array.map_or(0, |arr| arr.len() as u64);
        let unconnected_count = unconnected_array.map(|arr| arr.len() as u64);
        let bad_count = bad_array.map(|arr| arr.len() as u64);

        let mut sample_peers = Vec::new();
        if let Some(arr) = connected_array {
            for item in arr.iter().take(8) {
                if let Some(addr) = item.get("address").and_then(Value::as_str) {
                    let port = item
                        .get("port")
                        .and_then(Value::as_u64)
                        .and_then(|p| u16::try_from(p).ok());
                    sample_peers.push(PeerEndpoint {
                        address: addr.to_string(),
                        port,
                    });
                }
            }
        }

        let connectivity = classify_connectivity(connected_count);
        return Ok(PeerTelemetry {
            endpoint: endpoint.to_string(),
            family: ChainFamily::NeoN3,
            connected_count,
            unconnected_count,
            bad_count,
            connectivity,
            sample_peers,
        });
    }

    // Fallback: call getconnectioncount
    let count_val = call(agent, endpoint, "getconnectioncount", json!([]))?;
    let connected_count = count_val
        .as_u64()
        .or_else(|| count_val.as_str().and_then(|s| s.parse::<u64>().ok()))
        .unwrap_or(0);

    let connectivity = classify_connectivity(connected_count);
    Ok(PeerTelemetry {
        endpoint: endpoint.to_string(),
        family: ChainFamily::NeoN3,
        connected_count,
        unconnected_count: None,
        bad_count: None,
        connectivity,
        sample_peers: Vec::new(),
    })
}

fn probe_neox_peers(agent: &ureq::Agent, endpoint: &str) -> Result<PeerTelemetry, ChainQueryError> {
    let count_val = call(agent, endpoint, "net_peerCount", json!([]))?;
    let connected_count = match &count_val {
        Value::String(hex_str) => parse_hex_u64(hex_str).unwrap_or(0),
        Value::Number(num) => num.as_u64().unwrap_or(0),
        _ => 0,
    };

    let connectivity = classify_connectivity(connected_count);
    Ok(PeerTelemetry {
        endpoint: endpoint.to_string(),
        family: ChainFamily::NeoX,
        connected_count,
        unconnected_count: None,
        bad_count: None,
        connectivity,
        sample_peers: Vec::new(),
    })
}

fn classify_connectivity(connected_count: u64) -> PeerConnectivity {
    match connected_count {
        0 => PeerConnectivity::Isolated,
        1..=2 => PeerConnectivity::Sparse,
        _ => PeerConnectivity::Healthy,
    }
}

fn parse_hex_u64(value: &str) -> Option<u64> {
    let clean = value.strip_prefix("0x").unwrap_or(value);
    u64::from_str_radix(clean, 16).ok()
}

#[cfg(test)]
#[path = "../../tests/unit/chain_state/peers/tests.rs"]
mod tests;
