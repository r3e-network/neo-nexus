use std::path::Path;

use anyhow::Result;

use super::{
    support_metrics_json_text, support_nodes_json, support_nodes_text, write_bundle_file,
    SupportBundleContext, SupportBundleFile,
};

pub(super) fn write_inventory_files(
    bundle_dir: &Path,
    context: &SupportBundleContext,
    files: &mut Vec<SupportBundleFile>,
) -> Result<()> {
    write_bundle_file(
        bundle_dir,
        "metrics.txt",
        &context.metrics_snapshot.to_cli_text(),
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "metrics.json",
        &support_metrics_json_text(&context.metrics_snapshot)?,
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "metrics.prom",
        &context.metrics_snapshot.to_prometheus_text(),
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "nodes.json",
        &support_nodes_json(&context.nodes)?,
        files,
    )?;
    write_bundle_file(
        bundle_dir,
        "nodes.txt",
        &support_nodes_text(&context.nodes),
        files,
    )?;
    Ok(())
}
