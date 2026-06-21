use crate::types::{Network, NodeType};

use super::{FastSyncSnapshot, FastSyncSnapshotCatalogEntry};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SnapshotFilter {
    pub network: Option<Network>,
    pub node_type: Option<NodeType>,
    pub verified: Option<bool>,
    pub cached: Option<bool>,
    pub query: String,
}

impl SnapshotFilter {
    pub fn new(
        network: Option<Network>,
        node_type: Option<NodeType>,
        verified: Option<bool>,
        cached: Option<bool>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            network,
            node_type,
            verified,
            cached,
            query: query.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SnapshotCatalogEntryFilter {
    pub network: Option<Network>,
    pub node_type: Option<NodeType>,
    pub query: String,
}

impl SnapshotCatalogEntryFilter {
    pub fn new(
        network: Option<Network>,
        node_type: Option<NodeType>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            network,
            node_type,
            query: query.into(),
        }
    }
}

pub fn filter_snapshots(
    snapshots: &[FastSyncSnapshot],
    filter: &SnapshotFilter,
) -> Vec<FastSyncSnapshot> {
    let query = filter.query.trim().to_lowercase();
    snapshots
        .iter()
        .filter(|snapshot| {
            filter
                .network
                .is_none_or(|network| snapshot.network == network)
        })
        .filter(|snapshot| {
            filter
                .node_type
                .is_none_or(|kind| snapshot.node_type == kind)
        })
        .filter(|snapshot| {
            filter
                .verified
                .is_none_or(|verified| snapshot_is_verified(snapshot) == verified)
        })
        .filter(|snapshot| {
            filter
                .cached
                .is_none_or(|cached| snapshot.cached_path.is_some() == cached)
        })
        .filter(|snapshot| query.is_empty() || snapshot_matches(snapshot, &query))
        .cloned()
        .collect()
}

pub fn filter_snapshot_catalog_entries(
    entries: &[FastSyncSnapshotCatalogEntry],
    filter: &SnapshotCatalogEntryFilter,
) -> Vec<FastSyncSnapshotCatalogEntry> {
    let query = filter.query.trim().to_lowercase();
    entries
        .iter()
        .filter(|entry| {
            filter
                .network
                .is_none_or(|network| entry.network == network)
        })
        .filter(|entry| filter.node_type.is_none_or(|kind| entry.node_type == kind))
        .filter(|entry| query.is_empty() || catalog_entry_matches(entry, &query))
        .cloned()
        .collect()
}

fn snapshot_is_verified(snapshot: &FastSyncSnapshot) -> bool {
    snapshot.verified_sha256.as_ref() == Some(&snapshot.expected_sha256)
}

fn snapshot_matches(snapshot: &FastSyncSnapshot, query: &str) -> bool {
    text_matches(&snapshot.id, query)
        || text_matches(&snapshot.label, query)
        || text_matches(&snapshot.network.to_string(), query)
        || text_matches(&snapshot.node_type.to_string(), query)
        || text_matches(&snapshot.source_path.display().to_string(), query)
        || snapshot
            .source_url
            .as_deref()
            .is_some_and(|url| text_matches(url, query))
        || snapshot
            .download_file_name
            .as_deref()
            .is_some_and(|file| text_matches(file, query))
        || text_matches(&snapshot.expected_sha256, query)
        || snapshot
            .verified_sha256
            .as_deref()
            .is_some_and(|sha| text_matches(sha, query))
        || snapshot
            .cached_path
            .as_ref()
            .is_some_and(|path| text_matches(&path.display().to_string(), query))
        || text_matches(verified_label(snapshot_is_verified(snapshot)), query)
        || text_matches(cache_label(snapshot.cached_path.is_some()), query)
}

fn catalog_entry_matches(entry: &FastSyncSnapshotCatalogEntry, query: &str) -> bool {
    text_matches(&entry.id, query)
        || text_matches(&entry.label, query)
        || text_matches(&entry.network.to_string(), query)
        || text_matches(&entry.node_type.to_string(), query)
        || text_matches(&entry.url, query)
        || text_matches(&entry.file_name, query)
        || text_matches(&entry.expected_sha256, query)
        || text_matches(&entry.max_bytes.to_string(), query)
}

fn verified_label(verified: bool) -> &'static str {
    if verified {
        "verified"
    } else {
        "unverified"
    }
}

fn cache_label(cached: bool) -> &'static str {
    if cached {
        "cached"
    } else {
        "uncached"
    }
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
#[path = "../../tests/unit/snapshots/filter/tests.rs"]
mod tests;
