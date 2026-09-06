use super::*;

impl Repository {
    pub fn update_node(&self, id: &str, input: NewNode) -> Result<NodeConfig> {
        crate::types::validate_node_id(id)?;
        validate_node_input(&input)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let (status_raw, pid) = transaction
            .query_row(
                "SELECT status, pid FROM nodes WHERE id = ?1",
                params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<u32>>(1)?)),
            )
            .with_context(|| format!("node {id} was not found"))?;
        let status = NodeStatus::from_str(&status_raw)?;
        if status.is_active() || pid.is_some() {
            anyhow::bail!(
                "stop node {id} before changing its runtime, arguments, network or ports"
            );
        }
        let name = input.name.trim().to_string();
        let runtime_version = normalize_runtime_version(&input.runtime_version);

        transaction.execute(
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
        transaction.execute(
            "DELETE FROM node_runtime_quarantine WHERE node_id = ?1",
            params![id],
        )?;
        transaction.commit()?;

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

    /// Replace a quarantined backup command with a runtime selected on this
    /// machine. Updating the active node and clearing quarantine are atomic.
    pub fn rebind_node_runtime(
        &self,
        id: &str,
        binary_path: PathBuf,
        args: Vec<String>,
    ) -> Result<NodeConfig> {
        crate::types::validate_node_id(id)?;
        let node = self
            .list_nodes()?
            .into_iter()
            .find(|node| node.id == id)
            .with_context(|| format!("node {id} was not found"))?;
        if node.status.is_active() {
            anyhow::bail!("stop node {} before rebinding its runtime", node.name);
        }

        self.update_node(
            id,
            NewNode {
                name: node.name,
                node_type: node.node_type,
                network: node.network,
                binary_path,
                args,
                runtime_version: node.runtime_version,
                storage_engine: node.storage_engine,
                rpc_port: node.rpc_port,
                p2p_port: node.p2p_port,
                ws_port: node.ws_port,
            },
        )
    }

    pub fn update_node_status(&self, id: &str, status: NodeStatus, pid: Option<u32>) -> Result<()> {
        crate::types::validate_node_id(id)?;
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE nodes SET status = ?1, pid = ?2 WHERE id = ?3",
            params![status.to_string(), pid, id],
        )?;
        ensure_affected_rows(changed, "node", id)?;
        Ok(())
    }

    /// Atomically move the observed runtime state when it is still the state a
    /// lifecycle operation inspected. Both the status and pid participate in
    /// the comparison: a pid change means another controller won the race and
    /// this caller must not overwrite its result.
    pub fn transition_node_status(
        &self,
        id: &str,
        expected_status: NodeStatus,
        expected_pid: Option<u32>,
        status: NodeStatus,
        pid: Option<u32>,
    ) -> Result<bool> {
        crate::types::validate_node_id(id)?;
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE nodes
             SET status = ?1, pid = ?2
             WHERE id = ?3 AND status = ?4 AND pid IS ?5",
            params![
                status.to_string(),
                pid,
                id,
                expected_status.to_string(),
                expected_pid,
            ],
        )?;
        Ok(changed == 1)
    }

    /// Claim a launch only if the complete node definition still matches the
    /// snapshot the planner used. Comparing status and pid alone would allow an
    /// editor transaction that committed moments earlier to be overwritten by
    /// a launch of stale binary/argument/port data.
    pub fn claim_node_launch(&self, node: &NodeConfig) -> Result<bool> {
        crate::types::validate_node_id(&node.id)?;
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE nodes
             SET status = ?1, pid = ?2
             WHERE id = ?3
               AND name = ?4
               AND node_type = ?5
               AND network = ?6
               AND binary_path = ?7
               AND args = ?8
               AND runtime_version = ?9
               AND storage_engine = ?10
               AND rpc_port = ?11
               AND p2p_port = ?12
               AND ws_port IS ?13
               AND status = ?14
               AND pid IS ?15",
            params![
                NodeStatus::Starting.to_string(),
                node.pid,
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
        Ok(changed == 1)
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
