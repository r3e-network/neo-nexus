use super::{RemoteProbeStatus, RemoteServerProbeRecord};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteProbeHistoryFilter {
    pub status: Option<RemoteProbeStatus>,
    pub query: String,
}

impl RemoteProbeHistoryFilter {
    pub fn new(status: Option<RemoteProbeStatus>, query: impl Into<String>) -> Self {
        Self {
            status,
            query: query.into(),
        }
    }
}

pub fn filter_remote_probe_history(
    records: &[RemoteServerProbeRecord],
    filter: &RemoteProbeHistoryFilter,
) -> Vec<RemoteServerProbeRecord> {
    let query = filter.query.trim().to_lowercase();
    let mut rows = records
        .iter()
        .filter(|record| filter.status.is_none_or(|status| record.status == status))
        .filter(|record| query.is_empty() || record_matches(record, &query))
        .cloned()
        .collect::<Vec<_>>();
    rows.sort_by(record_order);
    rows
}

fn record_order(
    left: &RemoteServerProbeRecord,
    right: &RemoteServerProbeRecord,
) -> std::cmp::Ordering {
    right
        .checked_at_unix
        .cmp(&left.checked_at_unix)
        .then_with(|| right.id.cmp(&left.id))
}

fn record_matches(record: &RemoteServerProbeRecord, query: &str) -> bool {
    text_matches(&record.remote_server_id, query)
        || text_matches(&record.remote_server_name, query)
        || text_matches(&record.base_url, query)
        || text_matches(record.status.label(), query)
        || text_matches(&record.checked_at_unix.to_string(), query)
        || text_matches(&record.message, query)
        || matches_u64(record.total_nodes, query)
        || matches_u64(record.running_nodes, query)
        || matches_u64(record.syncing_nodes, query)
        || matches_u64(record.error_nodes, query)
        || matches_u64(record.total_blocks, query)
        || matches_u64(record.total_peers, query)
        || matches_u64(record.public_node_count, query)
}

fn matches_u64(value: Option<u64>, query: &str) -> bool {
    value.is_some_and(|value| text_matches(&value.to_string(), query))
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "../../tests/unit/federation/probe_filter/tests.rs"]
mod tests;
