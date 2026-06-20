use std::path::Path;

use anyhow::Result;

use super::super::{
    context::SupportBundleContext,
    io::write_bundle_file,
    render::{
        render_privacy_note, render_readme, support_log_diagnosis_json, support_log_diagnosis_text,
        support_metrics_json_text, support_nodes_json, support_nodes_text,
    },
    SupportBundleFile,
};

mod inventory;
mod logs;
mod overview;
mod reports;

use self::{
    inventory::write_inventory_files, logs::write_log_diagnosis_files,
    overview::write_overview_files, reports::write_report_files,
};

pub(super) fn write_evidence_files(
    bundle_dir: &Path,
    context: &SupportBundleContext,
    files: &mut Vec<SupportBundleFile>,
) -> Result<()> {
    write_overview_files(bundle_dir, context, files)?;
    write_report_files(bundle_dir, context, files)?;
    write_inventory_files(bundle_dir, context, files)?;
    write_log_diagnosis_files(bundle_dir, context, files)?;
    Ok(())
}
