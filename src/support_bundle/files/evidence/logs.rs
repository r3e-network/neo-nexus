use std::path::Path;

use anyhow::Result;

use super::{
    support_log_diagnosis_json, support_log_diagnosis_text, write_bundle_file,
    SupportBundleContext, SupportBundleFile,
};

pub(super) fn write_log_diagnosis_files(
    bundle_dir: &Path,
    context: &SupportBundleContext,
    files: &mut Vec<SupportBundleFile>,
) -> Result<()> {
    write_bundle_file(
        bundle_dir,
        "log-diagnosis.json",
        &support_log_diagnosis_json(&context.log_diagnosis_report)?,
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "log-diagnosis.txt",
        &support_log_diagnosis_text(&context.log_diagnosis_report),
        files,
    )?;
    Ok(())
}
