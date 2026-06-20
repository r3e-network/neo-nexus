use std::path::Path;

use anyhow::Result;

use super::{
    render_privacy_note, render_readme, write_bundle_file, SupportBundleContext, SupportBundleFile,
};

pub(super) fn write_overview_files(
    bundle_dir: &Path,
    context: &SupportBundleContext,
    files: &mut Vec<SupportBundleFile>,
) -> Result<()> {
    write_bundle_file(
        bundle_dir,
        "README.txt",
        &render_readme(
            &context.diagnostics,
            &context.integrity_report,
            &context.event_report,
            &context.log_diagnosis_report,
            &context.metrics_snapshot,
        ),
        files,
    )?;
    write_bundle_file(bundle_dir, "privacy.txt", &render_privacy_note(), files)?;
    Ok(())
}
