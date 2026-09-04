use super::*;
use crate::agents::{AgentProfile, AgentRecord, AgentStatus};

impl Repository {
    pub fn list_agents(&self) -> Result<Vec<AgentRecord>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT record FROM managed_agents ORDER BY id")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| serde_json::from_str(&row?).context("invalid managed agent record"))
            .collect()
    }

    pub(crate) fn put_agent(&self, record: &AgentRecord) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        check_agent_node(&transaction, &record.profile)?;
        transaction.execute("INSERT INTO managed_agents(id, record) VALUES (?1, ?2) ON CONFLICT(id) DO UPDATE SET record = excluded.record",
            params![record.profile.id, serde_json::to_string(record)?])?;
        transaction.commit()?;
        Ok(())
    }

    /// Restore metadata only. Existing profiles (including live processes) are
    /// never replaced by a backup, and imported profiles cannot auto-start.
    pub(crate) fn restore_agent_profile(&self, profile: &AgentProfile) -> Result<bool> {
        let record = AgentRecord {
            profile: profile.clone(),
            status: AgentStatus::Stopped,
            pid: None,
            process_started_at: None,
            desired_running: false,
            restart_attempts: 0,
            restart_after: None,
            healthy: None,
            last_health_at: 0,
        };
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        check_agent_node(&transaction, profile)?;
        let created = transaction.execute(
            "INSERT INTO managed_agents(id, record) VALUES (?1, ?2) ON CONFLICT(id) DO NOTHING",
            params![profile.id, serde_json::to_string(&record)?],
        )? > 0;
        transaction.commit()?;
        Ok(created)
    }

    pub(crate) fn remove_agent(&self, id: &str) -> Result<()> {
        self.connection()?
            .execute("DELETE FROM managed_agents WHERE id = ?1", params![id])?;
        Ok(())
    }
}

fn check_agent_node(transaction: &rusqlite::Transaction<'_>, profile: &AgentProfile) -> Result<()> {
    if let Some(id) = &profile.node_id {
        let exists: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE id = ?1)",
            params![id],
            |row| row.get(0),
        )?;
        if !exists {
            anyhow::bail!("associated node no longer exists");
        }
    }
    Ok(())
}
