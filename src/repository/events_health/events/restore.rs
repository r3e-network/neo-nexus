use super::*;

impl Repository {
    pub fn restore_runtime_events(&self, events: &[RestoredRuntimeEvent]) -> Result<usize> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let mut restored = 0;

        for restored_event in events {
            if restored_runtime_event_exists(&transaction, restored_event)? {
                continue;
            }

            insert_restored_runtime_event(&transaction, restored_event)?;
            restored += 1;
        }

        transaction.commit()?;
        Ok(restored)
    }
}

fn restored_runtime_event_exists(
    transaction: &rusqlite::Transaction<'_>,
    restored_event: &RestoredRuntimeEvent,
) -> Result<bool> {
    Ok(transaction
        .query_row(
            "SELECT id
             FROM runtime_events
             WHERE occurred_at_unix = ?1
               AND node_id IS ?2
               AND node_name IS ?3
               AND kind = ?4
               AND severity = ?5
               AND message = ?6
             LIMIT 1",
            params![
                restored_event.occurred_at_unix,
                restored_event.event.node_id.as_deref(),
                restored_event.event.node_name.as_deref(),
                restored_event.event.kind.to_string(),
                restored_event.event.severity.to_string(),
                restored_event.event.message.as_str(),
            ],
            |row| row.get::<_, i64>(0),
        )
        .optional()?
        .is_some())
}

fn insert_restored_runtime_event(
    transaction: &rusqlite::Transaction<'_>,
    restored_event: &RestoredRuntimeEvent,
) -> Result<()> {
    transaction.execute(
        "INSERT INTO runtime_events (
            occurred_at_unix, node_id, node_name, kind, severity, message
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            restored_event.occurred_at_unix,
            restored_event.event.node_id.as_deref(),
            restored_event.event.node_name.as_deref(),
            restored_event.event.kind.to_string(),
            restored_event.event.severity.to_string(),
            restored_event.event.message.as_str(),
        ],
    )?;
    Ok(())
}
