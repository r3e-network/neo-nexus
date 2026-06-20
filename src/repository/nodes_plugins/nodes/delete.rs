use super::*;

impl Repository {
    pub fn delete_node(&self, id: &str) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
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
        if deleted == 0 {
            anyhow::bail!("node {id} was not found");
        }
        transaction.commit()?;
        Ok(())
    }
}
