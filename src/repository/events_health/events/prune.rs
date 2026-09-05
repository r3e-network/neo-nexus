use super::*;
use std::fs::{self, OpenOptions};
use std::io::Write;

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

    /// Export events older than max_age_days to a JSON lines file.
    pub fn export_events_before(&self, max_age_days: u64, output_path: std::path::PathBuf) -> Result<usize> {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let cutoff_unix = now_unix - (max_age_days * 24 * 60 * 60);

        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, occurred_at_unix, node_id, node_name, kind, severity, message
             FROM runtime_events WHERE occurred_at_unix <= ?1",
        )?;

        let rows = statement.query_map(params![cutoff_unix as i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut exported_count = 0;
        let file_exists = output_path.exists();
        
        for record in rows {
            let (id, occurred, node_id, node_name, kind, severity, message) = record?;
            let json_line = format!(
                r#"{{"id":{}, "occurred_at_unix":{}, "node_id":"{}", "node_name":"{}", "kind":"{}", "severity":"{}", "message":"{}"}}"#,
                id, occurred, node_id.unwrap_or_default(), node_name.unwrap_or_default(), kind, severity, sanitize_json(&message)
            );
            
            if !file_exists {
                fs::write(&output_path, format!("{}\n", json_line))?;
            } else {
                OpenOptions::new()
                    .append(true)
                    .open(&output_path)?
                    .write_all(format!("{}\n", json_line).as_bytes())?;
            }
            exported_count += 1;
        }

        println!("Exported {} events older than {} days to {}", exported_count, max_age_days, output_path.display());
        Ok(exported_count)
    }

    /// Purge events older than max_age_days after export.
    pub fn purge_events_before(&self, max_age_days: u64) -> Result<usize> {
        let now_unix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let cutoff_unix = now_unix - (max_age_days * 24 * 60 * 60);

        let connection = self.connection()?;
        let deleted = connection.execute(
            "DELETE FROM runtime_events WHERE occurred_at_unix <= ?1",
            params![cutoff_unix as i64],
        )?;

        println!("Purged {} old event records", deleted);
        Ok(deleted as usize)
    }
}

fn sanitize_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}
