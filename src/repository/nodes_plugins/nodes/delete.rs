use super::*;

impl Repository {
    pub fn delete_node(&self, id: &str) -> Result<()> {
        crate::types::validate_node_id(id)?;
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
            anyhow::bail!("stop node {id} and confirm its process exited before deleting it");
        }
        transaction.execute(
            "DELETE FROM plugin_installations WHERE node_id = ?1",
            params![id],
        )?;
        transaction.execute(
            "DELETE FROM rpc_health_checks WHERE node_id = ?1",
            params![id],
        )?;
        transaction.execute("DELETE FROM plugin_states WHERE node_id = ?1", params![id])?;
        let deleted = transaction.execute("DELETE FROM nodes WHERE id = ?1", params![id])?;
        ensure_affected_rows(deleted, "node", id)?;
        transaction.commit()?;
        Ok(())
    }
}
