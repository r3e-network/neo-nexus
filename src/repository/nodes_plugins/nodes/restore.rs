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
        let active: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id = ?1 AND (pid IS NOT NULL OR lower(status) IN ('running','starting')))",
            params![node.id], |row| row.get(0))?;
        if active {
            anyhow::bail!(
                "stop node {} before restoring its configuration; its process state was preserved",
                node.id
            );
        }
        validate_restore_dependencies(&transaction, &node.id)?;
        // Restore is always a stopped inventory operation, even when called
        // directly with a NodeConfig that came from a live source workspace.
        if node.pid.is_some() || node.status != NodeStatus::Stopped {
            anyhow::bail!("restored nodes must be stopped and have no recorded PID");
        }
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
        Self::sync_node_recovery_status(&transaction, &node.id, NodeStatus::Stopped, None)?;
        transaction.commit()?;

        Ok(if existed {
            RestoreNodeOutcome::Updated
        } else {
            RestoreNodeOutcome::Created
        })
    }
}

fn validate_restore_dependencies(connection: &Connection, node_id: &str) -> Result<()> {
    let recovery: Option<String> = connection
        .query_row(
            "SELECT value FROM workspace_settings WHERE key = ?1",
            params![format!("watchdog.recovery.{node_id}")],
            |row| row.get(0),
        )
        .optional()?;
    if let Some(recovery) = recovery {
        let recovery: crate::watchdog::RecoveryState = serde_json::from_str(&recovery)
            .context("invalid node recovery record; stop the node before restoring")?;
        recovery.validate()?;
        if recovery.next_attempt_at_unix_ms.is_some() || recovery.claim.is_some() {
            anyhow::bail!(
                "stop node {node_id} to cancel its pending automatic recovery before restoring"
            );
        }
    }
    let mut statement = connection.prepare("SELECT record FROM managed_agents")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    for row in rows {
        let agent: crate::agents::AgentRecord =
            serde_json::from_str(&row?).context("invalid managed agent record")?;
        if agent.profile.node_id.as_deref() == Some(node_id)
            && (agent.pid.is_some()
                || agent.desired_running
                || matches!(
                    agent.status,
                    crate::agents::AgentStatus::Running | crate::agents::AgentStatus::Starting
                ))
        {
            anyhow::bail!(
                "stop the agent associated with node {node_id} before restoring that node"
            );
        }
    }
    Ok(())
}
