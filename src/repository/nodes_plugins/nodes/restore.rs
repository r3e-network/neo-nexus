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
        let existing_runtime = transaction
            .query_row(
                "SELECT status, pid FROM nodes WHERE id = ?1",
                params![node.id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<u32>>(1)?)),
            )
            .optional()?;
        if let Some((status_raw, pid)) = &existing_runtime {
            let status = NodeStatus::from_str(status_raw)?;
            if status.is_active() || pid.is_some() {
                anyhow::bail!(
                    "stop node {} and confirm its process exited before restoring over it",
                    node.id
                );
            }
        }
        let existed = existing_runtime.is_some();

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
                "",
                "",
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
            "INSERT INTO node_runtime_quarantine (
                node_id, imported_binary_path, imported_args
             ) VALUES (?1, ?2, ?3)
             ON CONFLICT(node_id) DO UPDATE SET
                imported_binary_path = excluded.imported_binary_path,
                imported_args = excluded.imported_args",
            params![
                node.id,
                node.binary_path.to_string_lossy(),
                encode_args(&node.args),
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

    pub fn quarantined_runtime_spec(
        &self,
        node_id: &str,
    ) -> Result<Option<QuarantinedRuntimeSpec>> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT imported_binary_path, imported_args
                 FROM node_runtime_quarantine
                 WHERE node_id = ?1",
                params![node_id],
                |row| {
                    let args: String = row.get(1)?;
                    Ok(QuarantinedRuntimeSpec {
                        binary_path: PathBuf::from(row.get::<_, String>(0)?),
                        args: decode_args(&args),
                    })
                },
            )
            .optional()
            .with_context(|| format!("failed to load quarantined runtime for node {node_id}"))
    }
}
