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
        let fields = [
            ("support-bundle", self.status.clone()),
            ("bundle", self.bundle_dir.display().to_string()),
            ("archive", self.archive_path.display().to_string()),
            ("archive-sha256", self.archive_sha256.clone()),
            ("archive-bytes", self.archive_bytes.to_string()),
            ("manifest", self.manifest_path.display().to_string()),
            ("privacy", self.manifest.privacy_policy.to_string()),
            ("readiness", self.manifest.readiness_status.to_string()),
            ("integrity", self.manifest.integrity_status.to_string()),
            ("nodes", self.manifest.node_count.to_string()),
            (
                "matched-events",
                self.manifest.matched_event_count.to_string(),
            ),
            (
                "exported-events",
                self.manifest.exported_event_count.to_string(),
            ),
            ("files", self.manifest.files.len().to_string()),
        ];

        let mut lines = fields
            .map(|(label, value)| format!("{label}: {value}"))
            .to_vec();
        lines.push(String::new());
        lines.join("\n")
    }

    pub fn to_json_text(&self) -> Result<String> {
        Ok(format!("{}\n", serde_json::to_string_pretty(self)?))
    }
}
