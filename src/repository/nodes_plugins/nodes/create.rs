use super::*;

impl Repository {
    pub fn create_node(&self, input: NewNode) -> Result<NodeConfig> {
        self.create_nodes_with_plugins(vec![(input, Vec::new())])?
            .into_iter()
            .next()
            .context("failed to create node")
    }

    pub fn create_nodes_with_plugins(
        &self,
        inputs: Vec<(NewNode, Vec<PluginState>)>,
    ) -> Result<Vec<NodeConfig>> {
        let mut nodes = Vec::with_capacity(inputs.len());
        for (input, plugins) in inputs {
            nodes.push((new_node_config(input)?, plugins));
        }

        if nodes.is_empty() {
            return Ok(Vec::new());
        }

        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        for (node, plugins) in &nodes {
            transaction.execute(
                "INSERT INTO nodes (
                    id, name, node_type, network, binary_path, args,
                    runtime_version, storage_engine, rpc_port, p2p_port, ws_port, status, pid
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    &node.id,
                    &node.name,
                    node.node_type.to_string(),
                    node.network.to_string(),
                    node.binary_path.to_string_lossy(),
                    encode_args(&node.args),
                    &node.runtime_version,
                    node.storage_engine.to_string(),
                    node.rpc_port,
                    node.p2p_port,
                    node.ws_port,
                    node.status.to_string(),
                    node.pid,
                ],
            )?;

            for plugin in plugins {
                transaction.execute(
                    "INSERT INTO plugin_states (node_id, plugin_id, enabled)
                     VALUES (?1, ?2, ?3)",
                    params![&node.id, plugin.plugin_id.to_string(), plugin.enabled],
                )?;
            }
        }
        transaction.commit()?;

        Ok(nodes.into_iter().map(|(node, _)| node).collect())
    }
}
