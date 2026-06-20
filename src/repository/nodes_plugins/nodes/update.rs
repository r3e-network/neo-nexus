use super::*;

impl Repository {
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
        let connection = self.connection()?;
        connection.execute(
            "UPDATE nodes SET status = ?1, pid = ?2 WHERE id = ?3",
            params![status.to_string(), pid, id],
        )?;
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
