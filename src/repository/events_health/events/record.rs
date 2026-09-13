use super::*;

impl Repository {
    /// Commit a small observation batch atomically. Callers retain their input
    /// checkpoint until this returns successfully; a failed insert rolls back
    /// the whole batch rather than duplicating its successful prefix on retry.
    pub(crate) fn record_event_batch(&self, events: &[NewRuntimeEvent]) -> Result<()> {
        anyhow::ensure!(events.len() <= 64, "event batch exceeds 64 entries");
        if events.is_empty() {
            return Ok(());
        }
        let mut connection = self.connection()?;
        connection.busy_timeout(Duration::from_millis(100))?;
        let transaction = connection.transaction()?;
        let occurred_at_unix = current_unix_time()?;
        {
            let mut insert = transaction.prepare(
                "INSERT INTO runtime_events (
                    occurred_at_unix, node_id, node_name, kind, severity, message
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for event in events {
                insert.execute(params![
                    occurred_at_unix,
                    event.node_id,
                    event.node_name,
                    event.kind.to_string(),
                    event.severity.to_string(),
                    event.message,
                ])?;
            }
        }
        transaction.commit().context("failed to commit event batch")
    }

    pub fn record_event(&self, event: NewRuntimeEvent) -> Result<RuntimeEvent> {
        self.record_event_at(event, current_unix_time()?)
    }

    pub fn record_event_at(
        &self,
        event: NewRuntimeEvent,
        occurred_at_unix: u64,
    ) -> Result<RuntimeEvent> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO runtime_events (
                occurred_at_unix, node_id, node_name, kind, severity, message
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                occurred_at_unix,
                event.node_id,
                event.node_name,
                event.kind.to_string(),
                event.severity.to_string(),
                event.message,
            ],
        )?;

        let id = connection.last_insert_rowid();
        self.get_event(&connection, id)
    }
}
