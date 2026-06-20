use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};

use crate::{repository::Repository, snapshots::sha256_file};

use super::{
    context::SupportBundleContext,
    files::{publish_support_bundle_archive, write_support_bundle_directory},
    model::WorkspaceSupportBundleExport,
    render::bundle_status,
};

pub struct WorkspaceSupportBundleExporter;

impl WorkspaceSupportBundleExporter {
    pub fn write(
        repository: &Repository,
        database: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        application_version: impl Into<String>,
    ) -> Result<WorkspaceSupportBundleExport> {
        Self::write_at(
            repository,
            database,
            output_dir,
            application_version,
            current_unix_time()?,
        )
    }

    pub fn write_at(
        repository: &Repository,
        database: impl AsRef<Path>,
        output_dir: impl AsRef<Path>,
        application_version: impl Into<String>,
        generated_at_unix: u64,
    ) -> Result<WorkspaceSupportBundleExport> {
        let database = database.as_ref();
        let output_dir = output_dir.as_ref();
        fs::create_dir_all(output_dir).with_context(|| {
            format!(
                "failed to create support bundle directory {}",
                output_dir.display()
            )
        })?;

        let context = SupportBundleContext::collect(
            repository,
            database,
            application_version.into(),
            generated_at_unix,
        )?;

        let bundle_id = format!("neo-nexus-support-bundle-{generated_at_unix}");
        let bundle_dir = output_dir.join(&bundle_id);
        if bundle_dir.exists() {
            fs::remove_dir_all(&bundle_dir).with_context(|| {
                format!(
                    "failed to replace existing support bundle {}",
                    bundle_dir.display()
                )
            })?;
        }
        fs::create_dir_all(&bundle_dir)
            .with_context(|| format!("failed to create support bundle {}", bundle_dir.display()))?;

        let (manifest_path, manifest) = write_support_bundle_directory(&bundle_dir, &context)?;
        let archive_path = output_dir.join(format!("{bundle_id}.zip"));
        publish_support_bundle_archive(&bundle_dir, &archive_path)?;
        let (archive_sha256, archive_bytes) = sha256_file(&archive_path)?;

        let status = bundle_status(
            &context.diagnostics,
            &context.integrity_report,
            &context.log_diagnosis_report,
            &context.metrics_snapshot,
        )
        .to_string();
        Ok(WorkspaceSupportBundleExport {
            schema_version: 1,
            status,
            bundle_dir,
            archive_path,
            archive_sha256,
            archive_bytes,
            manifest_path,
            manifest,
        })
    }
}

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before UNIX_EPOCH")?
        .as_secs())
}
