use super::*;

pub(in crate::cli::actions) fn export_node_configs_text(args: &[String]) -> Result<String> {
    let export = export_node_configs(args, "--export-node-configs")?;
    Ok(export.to_cli_text())
}

pub(in crate::cli::actions) fn export_node_configs_json_text(args: &[String]) -> Result<String> {
    let export = export_node_configs(args, "--export-node-configs-json")?;
    export.report.to_json_text()
}

fn export_node_configs(args: &[String], option: &str) -> Result<WorkspaceConfigExport> {
    require_arg_count(args, 4, option)?;
    let db_path = PathBuf::from(&args[2]);
    if !db_path.is_file() {
        anyhow::bail!(
            "workspace database {} does not exist; pass an existing neonexus.db",
            db_path.display()
        );
    }
    let repository = Repository::open(&db_path)
        .with_context(|| format!("failed to open workspace database {}", db_path.display()))?;
    let nodes = repository
        .list_nodes()
        .with_context(|| format!("failed to load nodes from {}", db_path.display()))?;
    let nodes_with_plugins = nodes
        .into_iter()
        .map(|node| {
            let plugins = repository
                .list_plugin_states(&node.id)
                .with_context(|| format!("failed to load plugin state for {}", node.name))?;
            Ok((node, plugins))
        })
        .collect::<Result<Vec<_>>>()?;

    WorkspaceConfigExporter::write(
        PathBuf::from(&args[3]),
        &db_path,
        &nodes_with_plugins,
        env!("CARGO_PKG_VERSION"),
    )
}
