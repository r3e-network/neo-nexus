use super::{filter::*, *};

impl Repository {
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
