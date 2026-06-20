use crate::types::NodeType;

use super::{RuntimeInstallation, RuntimePlatform, RuntimeRelease};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeInstallationFilter {
    pub node_type: Option<NodeType>,
    pub signed: Option<bool>,
    pub compatible: Option<bool>,
    pub query: String,
}

impl RuntimeInstallationFilter {
    pub fn new(
        node_type: Option<NodeType>,
        signed: Option<bool>,
        compatible: Option<bool>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            node_type,
            signed,
            compatible,
            query: query.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RuntimeReleaseFilter {
    pub node_type: Option<NodeType>,
    pub compatible: Option<bool>,
    pub query: String,
}

impl RuntimeReleaseFilter {
    pub fn new(
        node_type: Option<NodeType>,
        compatible: Option<bool>,
        query: impl Into<String>,
    ) -> Self {
        Self {
            node_type,
            compatible,
            query: query.into(),
        }
    }
}

pub fn filter_runtime_installations(
    installations: &[RuntimeInstallation],
    platform: &RuntimePlatform,
    filter: &RuntimeInstallationFilter,
) -> Vec<RuntimeInstallation> {
    let query = filter.query.trim().to_lowercase();
    installations
        .iter()
        .filter(|item| {
            filter
                .node_type
                .is_none_or(|node_type| item.node_type == node_type)
        })
        .filter(|item| {
            filter
                .signed
                .is_none_or(|signed| item.signature_verified == signed)
        })
        .filter(|item| {
            filter
                .compatible
                .is_none_or(|compatible| (item.platform == *platform) == compatible)
        })
        .filter(|item| query.is_empty() || installation_matches(item, &query))
        .cloned()
        .collect()
}

pub fn filter_runtime_releases(
    releases: &[RuntimeRelease],
    platform: &RuntimePlatform,
    filter: &RuntimeReleaseFilter,
) -> Vec<RuntimeRelease> {
    let query = filter.query.trim().to_lowercase();
    releases
        .iter()
        .filter(|release| {
            filter
                .node_type
                .is_none_or(|node_type| release.node_type == node_type)
        })
        .filter(|release| {
            filter
                .compatible
                .is_none_or(|compatible| release.platform_matches(platform) == compatible)
        })
        .filter(|release| query.is_empty() || release_matches(release, &query))
        .cloned()
        .collect()
}

fn installation_matches(item: &RuntimeInstallation, query: &str) -> bool {
    text_matches(&item.package_id, query)
        || text_matches(&item.label, query)
        || text_matches(&item.node_type.to_string(), query)
        || text_matches(&item.version, query)
        || text_matches(&item.platform.to_string(), query)
        || text_matches(&item.binary_path.display().to_string(), query)
        || text_matches(&item.sha256, query)
        || item
            .signer_public_key
            .as_deref()
            .is_some_and(|public_key| text_matches(public_key, query))
        || text_matches(trust_label(item.signature_verified), query)
        || text_matches(&item.bytes.to_string(), query)
}

fn release_matches(release: &RuntimeRelease, query: &str) -> bool {
    text_matches(&release.id, query)
        || text_matches(&release.label, query)
        || text_matches(&release.node_type.to_string(), query)
        || text_matches(&release.version, query)
        || text_matches(&release.platform.to_string(), query)
        || text_matches(&release.url, query)
        || text_matches(&release.file_name, query)
        || text_matches(&release.executable_name, query)
        || text_matches(&release.expected_sha256, query)
        || text_matches(&release.max_bytes.to_string(), query)
}

fn trust_label(signed: bool) -> &'static str {
    if signed {
        "signed"
    } else {
        "hash"
    }
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
mod tests;
