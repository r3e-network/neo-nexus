use super::*;

impl Repository {
    /// Reserve the launch only if the configuration/status snapshot is still
    /// current. SQLite serializes this write with backup's restore transaction.
    pub(crate) fn mark_node_starting(&self, node: &NodeConfig) -> Result<()> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE nodes SET status='starting',pid=NULL WHERE id=?1 AND name=?2
                AND node_type=?3 AND network=?4 AND binary_path=?5 AND args=?6
                AND runtime_version=?7 AND storage_engine=?8 AND rpc_port=?9
                AND p2p_port=?10 AND ws_port IS ?11 AND status=?12 AND pid IS ?13",
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
                node.pid
            ],
        )?;
        anyhow::ensure!(
            changed == 1,
            "node configuration or lifecycle changed before launch; reload and retry"
        );
        Ok(())
    }

    pub fn update_node(&self, id: &str, input: NewNode) -> Result<NodeConfig> {
        validate_node_input(&input)?;
        let connection = self.connection()?;
        let (status_raw, pid) = connection
            .query_row(
                "SELECT status, pid FROM nodes WHERE id = ?1",
                params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<u32>>(1)?)),
            )
            .with_context(|| format!("node {id} was not found"))?;
        let status = NodeStatus::from_str(&status_raw)?;
        let name = input.name.trim().to_string();
        let runtime_version = normalize_runtime_version(&input.runtime_version);

        connection.execute(
            "UPDATE nodes
             SET name = ?1,
                 node_type = ?2,
                 network = ?3,
                 binary_path = ?4,
                 args = ?5,
                 runtime_version = ?6,
                 storage_engine = ?7,
                 rpc_port = ?8,
                 p2p_port = ?9,
                 ws_port = ?10
             WHERE id = ?11",
            params![
                name,
                input.node_type.to_string(),
                input.network.to_string(),
                input.binary_path.to_string_lossy(),
                encode_args(&input.args),
                runtime_version,
                input.storage_engine.to_string(),
                input.rpc_port,
                input.p2p_port,
                input.ws_port,
                id,
            ],
        )?;

        Ok(NodeConfig {
            id: id.to_string(),
            name,
            node_type: input.node_type,
            network: input.network,
            binary_path: input.binary_path,
            args: input.args,
            runtime_version,
            storage_engine: input.storage_engine,
            rpc_port: input.rpc_port,
            p2p_port: input.p2p_port,
            ws_port: input.ws_port,
            status,
            pid,
        })
    }

    pub fn update_node_status(&self, id: &str, status: NodeStatus, pid: Option<u32>) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE nodes SET status = ?1, pid = ?2 WHERE id = ?3",
            params![status.to_string(), pid, id],
        )?;
        Self::sync_node_recovery_status(&transaction, id, status, pid)?;
        transaction.commit()?;
        Ok(())
    }

    pub fn clear_transient_runtime_state(&self) -> Result<usize> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE nodes
             SET status = ?1, pid = NULL
             WHERE status IN (?2, ?3)",
            params![
                NodeStatus::Stopped.to_string(),
                NodeStatus::Running.to_string(),
                NodeStatus::Starting.to_string(),
            ],
        )?;
        Ok(changed)
    }
}
