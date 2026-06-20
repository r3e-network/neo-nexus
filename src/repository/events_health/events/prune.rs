use super::*;

impl Repository {
    pub fn prune_events_keep_recent(&self, keep_recent: usize) -> Result<usize> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "DELETE FROM alert_deliveries
             WHERE event_id NOT IN (
                 SELECT id FROM runtime_events
                 ORDER BY occurred_at_unix DESC, id DESC
                 LIMIT ?1
             )",
            params![keep_recent as i64],
        )?;
        let deleted = transaction.execute(
            "DELETE FROM runtime_events
             WHERE id NOT IN (
                 SELECT id FROM runtime_events
                 ORDER BY occurred_at_unix DESC, id DESC
                 LIMIT ?1
             )",
            params![keep_recent as i64],
        )?;
        transaction.commit()?;
        Ok(deleted)
    }
}
