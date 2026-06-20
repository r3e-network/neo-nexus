use super::*;

impl Repository {
    pub fn restore_node_with_plugins(
        &self,
        node: &NodeConfig,
        plugins: &[PluginState],
    ) -> Result<RestoreNodeOutcome> {
        validate_node_config(node)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let existed = transaction
            .query_row(
                "SELECT 1 FROM nodes WHERE id = ?1",
                params![node.id],
                |_| Ok(()),
            )
            .optional()?
            .is_some();

        transaction.execute(
            "INSERT INTO nodes (
                id, name, node_type, network, binary_path, args,
                runtime_version, storage_engine, rpc_port, p2p_port, ws_port, status, pid
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                node_type = excluded.node_type,
                network = excluded.network,
                binary_path = excluded.binary_path,
                args = excluded.args,
                runtime_version = excluded.runtime_version,
                storage_engine = excluded.storage_engine,
                rpc_port = excluded.rpc_port,
                p2p_port = excluded.p2p_port,
                ws_port = excluded.ws_port,
                status = excluded.status,
                pid = excluded.pid",
            params![
                node.id,
                node.name,
                node.node_type.to_string(),
                node.network.to_string(),
                node.binary_path.to_string_lossy(),
                encode_args(&node.args),
                node.runtime_version,
                node.storage_engine.to_string(),
                node.rpc_port,
                node.p2p_port,
                node.ws_port,
                node.status.to_string(),
                node.pid,
            ],
        )?;
        transaction.execute(
            "DELETE FROM plugin_states WHERE node_id = ?1",
            params![node.id],
        )?;
        for plugin in plugins {
            transaction.execute(
                "INSERT INTO plugin_states (node_id, plugin_id, enabled)
                 VALUES (?1, ?2, ?3)",
                params![node.id, plugin.plugin_id.to_string(), plugin.enabled],
            )?;
        }
        transaction.commit()?;

        Ok(if existed {
            RestoreNodeOutcome::Updated
        } else {
            RestoreNodeOutcome::Created
        })
    }
}
