use std::{fs, path::Path};

use anyhow::{Context, Result};

use crate::{catalog::PluginState, types::NodeConfig};

use super::{
    nodes::export_node_configs,
    report::{build_workspace_config_report, write_report_files},
    time::current_unix_time,
    WorkspaceConfigExporter,
};
use crate::config::export::model::WorkspaceConfigExport;

impl WorkspaceConfigExporter {
    pub fn write(
        output_dir: impl AsRef<Path>,
        database: impl AsRef<Path>,
        nodes: &[(NodeConfig, Vec<PluginState>)],
        application_version: impl Into<String>,
    ) -> Result<WorkspaceConfigExport> {
        Self::write_at(
            output_dir,
            database,
            nodes,
            application_version,
            current_unix_time()?,
        )
    }

    pub fn write_at(
        output_dir: impl AsRef<Path>,
        database: impl AsRef<Path>,
        nodes: &[(NodeConfig, Vec<PluginState>)],
        application_version: impl Into<String>,
        generated_at_unix: u64,
    ) -> Result<WorkspaceConfigExport> {
        let output_dir = output_dir.as_ref();
        create_export_dir(output_dir)?;

        let database = database.as_ref();
        let node_reports = export_node_configs(output_dir, nodes)?;
        let report = build_workspace_config_report(
            database,
            output_dir,
            nodes.len(),
            node_reports,
            application_version.into(),
            generated_at_unix,
        );
        let (text_path, json_path) = write_report_files(output_dir, generated_at_unix, &report)?;

        Ok(WorkspaceConfigExport {
            output_dir: output_dir.to_path_buf(),
            text_path,
            json_path,
            report,
        })
    }
}

fn create_export_dir(output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir).with_context(|| {
        format!(
            "failed to create node config export directory {}",
            output_dir.display()
        )
    })
}
