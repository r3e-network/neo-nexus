use std::{fs, path::Path};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::types::{NodeConfig, NodeType};

use super::{archive::SnapshotImport, SnapshotImportMode};

#[derive(Serialize)]
struct AppliedSnapshotManifest<'a> {
    snapshot_id: &'a str,
    label: &'a str,
    node_id: &'a str,
    node_name: &'a str,
    network: String,
    node_type: String,
    runtime_import_profile: &'static str,
    import_mode: &'static str,
    sha256: &'a str,
    bytes: u64,
    expanded_bytes: u64,
    imported_files: usize,
    source_cache_path: String,
    runtime_data_dir: String,
    applied_snapshot_path: String,
    applied_at_unix: u64,
}

pub(super) struct AppliedSnapshotManifestInput<'a> {
    pub(super) snapshot_id: &'a str,
    pub(super) label: &'a str,
    pub(super) node: &'a NodeConfig,
    pub(super) import_mode: SnapshotImportMode,
    pub(super) sha256: &'a str,
    pub(super) bytes: u64,
    pub(super) cached_path: &'a Path,
    pub(super) import_dir: &'a Path,
    pub(super) import: &'a SnapshotImport,
    pub(super) applied_at_unix: u64,
}

pub(super) fn write_applied_snapshot_manifest(
    manifest_path: &Path,
    input: AppliedSnapshotManifestInput<'_>,
) -> Result<()> {
    let manifest = AppliedSnapshotManifest {
        snapshot_id: input.snapshot_id,
        label: input.label,
        node_id: &input.node.id,
        node_name: &input.node.name,
        network: input.node.network.to_string(),
        node_type: input.node.node_type.to_string(),
        runtime_import_profile: runtime_import_profile(input.node),
        import_mode: input.import_mode.manifest_value(),
        sha256: input.sha256,
        bytes: input.bytes,
        expanded_bytes: input.import.expanded_bytes,
        imported_files: input.import.imported_files,
        source_cache_path: input.cached_path.display().to_string(),
        runtime_data_dir: input.import_dir.display().to_string(),
        applied_snapshot_path: input.import.snapshot_path.display().to_string(),
        applied_at_unix: input.applied_at_unix,
    };
    let manifest_text =
        serde_json::to_string_pretty(&manifest).context("failed to render snapshot manifest")?;
    fs::write(manifest_path, manifest_text.as_bytes()).with_context(|| {
        format!(
            "failed to write snapshot manifest {}",
            manifest_path.display()
        )
    })
}

fn runtime_import_profile(node: &NodeConfig) -> &'static str {
    match node.node_type {
        NodeType::NeoCli => "neo-cli managed chain data",
        NodeType::NeoGo => "neo-go managed chain data",
        NodeType::NeoRs => "neo-rs [storage].data_dir RocksDB data",
    }
}
