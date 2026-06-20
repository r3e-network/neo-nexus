use std::path::PathBuf;

use anyhow::Result;
use serde::Serialize;

use super::manifest::WorkspaceSupportBundleManifest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceSupportBundleExport {
    pub schema_version: u32,
    pub status: String,
    pub bundle_dir: PathBuf,
    pub archive_path: PathBuf,
    pub archive_sha256: String,
    pub archive_bytes: u64,
    pub manifest_path: PathBuf,
    pub manifest: WorkspaceSupportBundleManifest,
}

impl WorkspaceSupportBundleExport {
    pub fn to_cli_text(&self) -> String {
        format!(
            "support-bundle: {status}\nbundle: {bundle}\narchive: {archive}\narchive-sha256: {archive_sha256}\narchive-bytes: {archive_bytes}\nmanifest: {manifest}\nprivacy: {privacy}\nreadiness: {readiness}\nintegrity: {integrity}\nnodes: {nodes}\nmatched-events: {matched_events}\nexported-events: {exported_events}\nfiles: {files}\n",
            status = self.status,
            bundle = self.bundle_dir.display(),
            archive = self.archive_path.display(),
            archive_sha256 = self.archive_sha256,
            archive_bytes = self.archive_bytes,
            manifest = self.manifest_path.display(),
            privacy = self.manifest.privacy_policy,
            readiness = self.manifest.readiness_status,
            integrity = self.manifest.integrity_status,
            nodes = self.manifest.node_count,
            matched_events = self.manifest.matched_event_count,
            exported_events = self.manifest.exported_event_count,
            files = self.manifest.files.len(),
        )
    }

    pub fn to_json_text(&self) -> Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(self)?))
    }
}
