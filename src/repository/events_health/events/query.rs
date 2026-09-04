use super::{filter::*, *};

impl Repository {
    pub fn list_node_events(&self, node_id: &str, limit: usize) -> Result<Vec<RuntimeEvent>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, occurred_at_unix, node_id, node_name, kind, severity, message
             FROM runtime_events WHERE node_id=?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = statement.query_map(params![node_id, limit.min(200) as i64], event_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load node events")
    }

    /// Consume the journal by insertion id, independently of wall-clock changes.
    pub fn list_events_after(&self, after_id: i64, limit: usize) -> Result<Vec<RuntimeEvent>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, occurred_at_unix, node_id, node_name, kind, severity, message
             FROM runtime_events WHERE id > ?1 ORDER BY id ASC LIMIT ?2",
        )?;
        let rows =
            statement.query_map(params![after_id, limit.min(1000) as i64], event_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to read journal backlog")
    }

    pub fn latest_event_id(&self) -> Result<i64> {
        self.connection()?
            .query_row(
                "SELECT COALESCE(MAX(id), 0) FROM runtime_events",
                [],
                |row| row.get(0),
            )
            .context("failed to read journal position")
    }

    pub fn list_recent_events(&self, limit: usize) -> Result<Vec<RuntimeEvent>> {
        self.list_events(RuntimeEventFilter::new(None, "", limit))
    }

    pub fn list_events(&self, filter: RuntimeEventFilter) -> Result<Vec<RuntimeEvent>> {
        let connection = self.connection()?;
        let binding = EventFilterBinding::from_filter(filter);
        let sql = format!(
            "SELECT id, occurred_at_unix, node_id, node_name, kind, severity, message
             FROM runtime_events
             WHERE {EVENT_FILTER_WHERE_SQL}
             ORDER BY occurred_at_unix DESC, id DESC
             LIMIT ?4"
        );
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(
            params![
                binding.severity,
                binding.query,
                binding.pattern,
                binding.limit,
            ],
            event_from_row,
        )?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load runtime events")
    }

    pub fn count_events(&self, filter: &RuntimeEventFilter) -> Result<usize> {
        let connection = self.connection()?;
        let binding = EventFilterBinding::from_filter_ref(filter);
        let sql = format!(
            "SELECT COUNT(*)
             FROM runtime_events
             WHERE {EVENT_FILTER_WHERE_SQL}"
        );
        let count = connection.query_row(
            &sql,
            params![binding.severity, binding.query, binding.pattern],
            |row| row.get::<_, usize>(0),
        )?;
        Ok(count)
    }
}
