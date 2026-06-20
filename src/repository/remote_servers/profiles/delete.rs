use super::*;

impl Repository {
    pub fn delete_remote_server(&self, id: &str) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "DELETE FROM remote_server_probe_records WHERE remote_server_id = ?1",
            params![id],
        )?;
        let deleted =
            transaction.execute("DELETE FROM remote_servers WHERE id = ?1", params![id])?;
        if deleted == 0 {
            anyhow::bail!("remote server {id} was not found");
        }
        transaction.commit()?;
        Ok(())
    }
}
