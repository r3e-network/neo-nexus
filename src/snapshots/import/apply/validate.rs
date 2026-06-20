use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::{
    snapshots::{
        normalize_sha256, sha256_file, validation::verified_source_path, FastSyncSnapshot,
    },
    types::NodeConfig,
};

pub(super) struct VerifiedCachedSnapshot {
    pub(super) path: PathBuf,
    pub(super) sha256: String,
    pub(super) bytes: u64,
}

pub(super) fn verify_snapshot_matches_node(
    snapshot: &FastSyncSnapshot,
    node: &NodeConfig,
) -> Result<()> {
    if snapshot.network != node.network {
        anyhow::bail!(
            "snapshot network {} does not match node network {}",
            snapshot.network,
            node.network
        );
    }
    if snapshot.node_type != node.node_type {
        anyhow::bail!(
            "snapshot runtime {} does not match node runtime {}",
            snapshot.node_type,
            node.node_type
        );
    }
    Ok(())
}

pub(super) fn verified_cached_snapshot(
    snapshot: &FastSyncSnapshot,
) -> Result<VerifiedCachedSnapshot> {
    let cached_path = snapshot
        .cached_path
        .as_ref()
        .context("snapshot must be cached before it can be applied")?;
    let cached_path = verified_source_path(cached_path)?;
    let expected_sha256 = normalize_sha256(&snapshot.expected_sha256)?;
    let (sha256, bytes) = sha256_file(&cached_path)?;
    if sha256 != expected_sha256 {
        anyhow::bail!(
            "cached snapshot checksum mismatch: expected {expected_sha256}, got {sha256}"
        );
    }
    Ok(VerifiedCachedSnapshot {
        path: cached_path,
        sha256,
        bytes,
    })
}
