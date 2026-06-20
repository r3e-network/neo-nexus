use std::path::Path;

use anyhow::Result;

use super::{write_bundle_file, SupportBundleContext, SupportBundleFile};

pub(super) fn write_report_files(
    bundle_dir: &Path,
    context: &SupportBundleContext,
    files: &mut Vec<SupportBundleFile>,
) -> Result<()> {
    write_bundle_file(
        bundle_dir,
        "readiness.txt",
        &context.readiness_report.to_text(),
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "readiness.json",
        &context.readiness_report.to_json_text()?,
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "integrity.txt",
        &context.integrity_report.to_cli_text(),
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "integrity.json",
        &context.integrity_report.to_json_text()?,
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "event-journal.txt",
        &context.event_report.to_text(),
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "event-journal.json",
        &context.event_report.to_json_text()?,
        files,
    )?;
    Ok(())
}
