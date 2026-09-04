//! Conservative public-chain progress alarms. This monitor never restarts a
//! node: an idle/private chain or a failed observer is not evidence of a crash.
use anyhow::Result;

use crate::{
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    repository::Repository,
    rpc_health::{
        expected_public_identity, RpcHealthRecord, RpcHealthStatus, RpcIdentityKind,
        RpcIdentityStatus,
    },
    types::{ChainFamily, Network, NodeConfig, NodeStatus},
};

pub const STALL_WINDOW_SECONDS: u64 = 15 * 60;
/// Covers 15 minutes at the monitor's minimum supported interval of 10 seconds.
pub(crate) const OBSERVATION_HISTORY_LIMIT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProgressMarker {
    pub observed_pid: u32,
    pub identity: u64,
    pub last_observation_id: i64,
    pub last_checked_at_unix: u64,
    pub stalled_block_count: Option<u64>,
}

/// Evaluate newly persisted RPC observations. Markers and emitted events commit
/// together, so repeating a tick or restarting the workbench cannot duplicate it.
pub fn check(repository: &Repository, now_unix: u64) -> Result<usize> {
    let policy = repository.load_rpc_health_monitor_policy()?;
    if !policy.enabled {
        return Ok(0);
    }
    let max_age = policy.observation_max_age_seconds();
    let mut emitted = 0;
    for node in repository.list_nodes()? {
        if node.status != NodeStatus::Running || node.network == Network::Private {
            continue;
        }
        let Some(latest) = repository.latest_rpc_health(&node.id)? else {
            continue;
        };
        if !latest.is_fresh(now_unix, max_age) || !comparable(&node, &latest, &latest) {
            continue;
        }
        let (Some(pid), Some(identity), Some(height)) =
            (node.pid, latest.network.actual_identity, latest.block_count)
        else {
            continue;
        };
        let previous = repository.load_chain_progress_marker(&node.id)?;
        let same_process = previous
            .as_ref()
            .is_some_and(|marker| marker.observed_pid == pid && marker.identity == identity);
        if same_process
            && previous.as_ref().is_some_and(|marker| {
                latest.id <= marker.last_observation_id
                    || latest.checked_at_unix <= marker.last_checked_at_unix
            })
        {
            continue;
        }
        let mut marker = ProgressMarker {
            observed_pid: pid,
            identity,
            last_observation_id: latest.id,
            last_checked_at_unix: latest.checked_at_unix,
            stalled_block_count: previous
                .as_ref()
                .filter(|_| same_process)
                .and_then(|marker| marker.stalled_block_count),
        };
        let kind = if let Some(stalled_at) = marker.stalled_block_count {
            if height > stalled_at {
                marker.stalled_block_count = None;
                Some(EventKind::ChainProgressRecovered)
            } else {
                if height < stalled_at {
                    marker.stalled_block_count = None;
                }
                None
            }
        } else {
            let history = repository.list_rpc_health(&node.id, OBSERVATION_HISTORY_LIMIT)?;
            if stalled_window(&node, &latest, &history, max_age) {
                marker.stalled_block_count = Some(height);
                Some(EventKind::ChainProgressStalled)
            } else {
                None
            }
        };
        let event = kind.map(|kind| NewRuntimeEvent {
            node_id: Some(node.id.clone()), node_name: Some(node.name.clone()), kind,
            severity: if kind == EventKind::ChainProgressStalled { EventSeverity::Warning } else { EventSeverity::Info },
            message: if kind == EventKind::ChainProgressStalled {
                format!("Public chain has not advanced for at least 15 minutes across successful RPC observations (block count {height}). Check peers, synchronization and consensus; no automatic restart requested.")
            } else { format!("Public chain progress resumed; observed block count {height}.") },
        });
        let has_event = event.is_some();
        if repository.commit_chain_progress(
            &node,
            previous.as_ref(),
            &marker,
            event.as_ref(),
            now_unix,
        )? && has_event
        {
            emitted += 1;
        }
    }
    Ok(emitted)
}

fn comparable(node: &NodeConfig, record: &RpcHealthRecord, latest: &RpcHealthRecord) -> bool {
    record.status != RpcHealthStatus::Unreachable
        && record.block_count.is_some()
        && record.matches_process(node)
        && record.network.identity_status() == RpcIdentityStatus::Matched
        && record.network.identity_kind
            == Some(match node.node_type.family() {
                ChainFamily::NeoN3 => RpcIdentityKind::N3NetworkMagic,
                ChainFamily::NeoX => RpcIdentityKind::EvmChainId,
            })
        && record.network.expected_identity
            == expected_public_identity(node.node_type.family(), node.network)
        && record.network.actual_identity == latest.network.actual_identity
        && record.network.identity_kind == latest.network.identity_kind
        && record.endpoint == latest.endpoint
        && record.version == latest.version
}

fn stalled_window(
    node: &NodeConfig,
    latest: &RpcHealthRecord,
    history: &[RpcHealthRecord],
    max_gap: u64,
) -> bool {
    let mut previous_time = latest.checked_at_unix;
    let mut distinct = 0;
    for record in history {
        if !comparable(node, record, latest) || record.block_count != latest.block_count {
            break;
        }
        if record.checked_at_unix > previous_time {
            break;
        }
        if record.checked_at_unix == previous_time {
            if distinct == 0 {
                distinct = 1;
            }
            continue;
        }
        if previous_time - record.checked_at_unix > max_gap {
            break;
        }
        distinct += 1;
        previous_time = record.checked_at_unix;
        if distinct >= 2 && latest.checked_at_unix - previous_time >= STALL_WINDOW_SECONDS {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "../tests/unit/chain_progress.rs"]
mod tests;
