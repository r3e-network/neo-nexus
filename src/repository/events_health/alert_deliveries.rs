use super::*;

impl Repository {
    pub fn record_alert_delivery(&self, report: &AlertDeliveryReport) -> Result<AlertDelivery> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO alert_deliveries (
                event_id, attempted_at_unix, route_label, target, status, http_status, message
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                report.event_id,
                current_unix_time()?,
                report.route_label,
                report.target,
                report.status.to_string(),
                report.http_status,
                report.message,
            ],
        )?;
        let id = connection.last_insert_rowid();
        self.get_alert_delivery(&connection, id)
    }

    pub fn list_alert_deliveries(&self, limit: usize) -> Result<Vec<AlertDelivery>> {
        let connection = self.connection()?;
        let limit = limit.clamp(1, 500) as i64;
        let mut statement = connection.prepare(
            "SELECT id, event_id, attempted_at_unix, route_label, target, status, http_status,
                    message
             FROM alert_deliveries
             ORDER BY attempted_at_unix DESC, id DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map(params![limit], alert_delivery_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to list alert deliveries")
    }

    pub fn prune_alert_deliveries_keep_recent(&self, keep_recent: usize) -> Result<usize> {
        let connection = self.connection()?;
        let deleted = connection.execute(
            "DELETE FROM alert_deliveries
             WHERE id NOT IN (
                 SELECT id FROM alert_deliveries
                 ORDER BY attempted_at_unix DESC, id DESC
                 LIMIT ?1
             )",
            params![keep_recent as i64],
        )?;
        Ok(deleted)
    }
}
