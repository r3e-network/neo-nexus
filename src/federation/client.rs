use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde_json::Value;

use super::{
    parsing::{format_remote_probe_message, parse_public_status, remote_probe_status},
    RemoteProbeStatus, RemoteServerProbeReport, RemoteServerProfile,
};

pub struct RemoteFederationClient;

impl RemoteFederationClient {
    pub fn probe(
        profile: &RemoteServerProfile,
        timeout: Duration,
    ) -> Result<RemoteServerProbeReport> {
        let checked_at_unix = current_unix_time()?;
        if !profile.enabled {
            return Ok(RemoteServerProbeReport {
                profile_id: profile.id.clone(),
                profile_name: profile.name.clone(),
                base_url: profile.base_url.clone(),
                checked_at_unix,
                status: RemoteProbeStatus::Disabled,
                total_nodes: None,
                running_nodes: None,
                syncing_nodes: None,
                error_nodes: None,
                total_blocks: None,
                total_peers: None,
                public_node_count: None,
                message: "remote profile is disabled".to_string(),
            });
        }

        let agent = ureq::AgentBuilder::new().timeout(timeout).build();
        let status_body = agent
            .get(&profile.public_status_url())
            .call()
            .with_context(|| format!("failed to reach {}", profile.public_status_url()))?
            .into_string()
            .context("failed to read remote status response body")
            .and_then(|text| {
                serde_json::from_str::<Value>(&text)
                    .context("remote status endpoint did not return JSON")
            })?;
        let status = parse_public_status(&status_body);
        let public_node_count = fetch_public_node_count(&agent, profile).ok().flatten();
        let probe_status = remote_probe_status(status.error_nodes, status.running_nodes);
        let message =
            format_remote_probe_message(&profile.name, probe_status, &status, public_node_count);

        Ok(RemoteServerProbeReport {
            profile_id: profile.id.clone(),
            profile_name: profile.name.clone(),
            base_url: profile.base_url.clone(),
            checked_at_unix,
            status: probe_status,
            total_nodes: status.total_nodes,
            running_nodes: status.running_nodes,
            syncing_nodes: status.syncing_nodes,
            error_nodes: status.error_nodes,
            total_blocks: status.total_blocks,
            total_peers: status.total_peers,
            public_node_count,
            message,
        })
    }
}

fn fetch_public_node_count(
    agent: &ureq::Agent,
    profile: &RemoteServerProfile,
) -> Result<Option<u64>> {
    let body = agent
        .get(&profile.public_nodes_url())
        .call()
        .with_context(|| format!("failed to reach {}", profile.public_nodes_url()))?
        .into_string()
        .context("failed to read remote nodes response body")
        .and_then(|text| {
            serde_json::from_str::<Value>(&text)
                .context("remote nodes endpoint did not return JSON")
        })?;

    if let Some(nodes) = body.get("nodes").and_then(Value::as_array) {
        return Ok(Some(nodes.len() as u64));
    }
    if let Some(nodes) = body.as_array() {
        return Ok(Some(nodes.len() as u64));
    }
    Ok(None)
}

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs())
}
