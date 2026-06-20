use super::{context::LaunchPackWriteContext, *};

pub(super) struct NodeConfigWriteReport {
    pub(super) config_count: usize,
    pub(super) bytes_written: usize,
    pub(super) nodes: Vec<DeploymentNodeManifest>,
}

pub(super) fn write_node_configs(
    request: &PrivateNetworkDeploymentRequest,
    context: &LaunchPackWriteContext,
) -> Result<NodeConfigWriteReport> {
    let mut config_count = 0;
    let mut bytes_written = 0;
    let mut manifest_nodes = Vec::with_capacity(request.plan.nodes.len());
    let nodes_by_name = request
        .nodes
        .iter()
        .map(|node| (node.name.as_str(), node))
        .collect::<BTreeMap<_, _>>();

    for (start_order, planned) in request.plan.nodes.iter().enumerate() {
        let node = nodes_by_name
            .get(planned.name.as_str())
            .copied()
            .with_context(|| format!("planned node {} was not materialized", planned.name))?;
        let node_work_dir = request.node_root_dir.join(&node.id);
        let config_path = ConfigExporter::managed_target_path(&node_work_dir, node);
        let plugins = request
            .plugin_states
            .get(&node.id)
            .map_or(&[] as &[PluginState], Vec::as_slice);
        let profile = RuntimeConfigProfile {
            network_magic: context.network_magic,
            seed_nodes: context.seed_nodes.clone(),
            validators_count: context.validators_count,
            committee_public_keys: context.committee_public_keys.clone(),
            consensus_enabled: planned.role == NodeRole::Consensus,
        };
        let config_export = ConfigExporter::write_node_config_to_path_with_profile(
            &config_path,
            node,
            plugins,
            Some(&profile),
        )?;
        let (config_sha256, _) = sha256_file(&config_path).with_context(|| {
            format!(
                "failed to hash managed config for launch pack node {}",
                node.name
            )
        })?;
        config_count += 1;
        bytes_written += config_export.bytes_written;

        let launch_plan = LaunchPlanner::plan(node, config_path.clone(), node_work_dir.clone());
        manifest_nodes.push(DeploymentNodeManifest {
            start_order: start_order + 1,
            node_id: node.id.clone(),
            name: node.name.clone(),
            role: planned.role.label().to_string(),
            runtime: node.node_type.to_string(),
            storage: node.storage_engine.to_string(),
            rpc_port: node.rpc_port,
            p2p_port: node.p2p_port,
            ws_port: node.ws_port,
            binary_path: launch_plan.binary_path.display().to_string(),
            arguments: launch_plan.args,
            working_dir: node_work_dir.display().to_string(),
            config_path: config_path.display().to_string(),
            config_sha256,
            command: launch_plan.display_command,
        });
    }

    Ok(NodeConfigWriteReport {
        config_count,
        bytes_written,
        nodes: manifest_nodes,
    })
}
