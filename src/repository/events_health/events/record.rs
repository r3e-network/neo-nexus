use super::*;

impl Repository {
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
