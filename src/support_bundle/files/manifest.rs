use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::super::{
    context::SupportBundleContext,
    model::{WorkspaceSupportBundleManifest, WorkspaceSupportBundleManifestInput},
    SupportBundleFile,
};
use super::evidence::write_evidence_files;

pub(in crate::support_bundle) fn write_support_bundle_directory(
    bundle_dir: &Path,
    context: &SupportBundleContext,
) -> Result<(PathBuf, WorkspaceSupportBundleManifest)> {
    let mut files = Vec::new();
    write_evidence_files(bundle_dir, context, &mut files)?;
    let manifest_path = bundle_dir.join("manifest.json");
    let manifest = context.manifest_with_files(files);
    fs::write(
        &manifest_path,
        format!("{}\n", serde_json::to_string_pretty(&manifest)?),
    )
    .with_context(|| {
        format!(
            "failed to write support bundle manifest {}",
            manifest_path.display()
        )
    })?;
    Ok((manifest_path, manifest))
}

impl SupportBundleContext {
    pub(super) fn manifest_with_files(
        &self,
        files: Vec<SupportBundleFile>,
    ) -> WorkspaceSupportBundleManifest {
        WorkspaceSupportBundleManifest::from_input(WorkspaceSupportBundleManifestInput {
            application_version: self.application_version.clone(),
            generated_at_unix: self.generated_at_unix,
            database: self.database.clone(),
            diagnostics: &self.diagnostics,
            integrity: &self.integrity_report,
            metrics: &self.metrics_snapshot,
            log_diagnosis: &self.log_diagnosis_report,
            running_nodes: self.running_nodes,
            matched_event_count: self.matched_event_count,
            exported_event_count: self.event_report.exported_event_count,
            files,
        })
    }
}
