use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};

use super::*;
use crate::agents::HermesAgentAssociation;

impl Repository {
    pub fn save_hermes_agent(&self, assoc: &HermesAgentAssociation) -> Result<()> {
        crate::types::validate_node_id(&assoc.node_id)?;
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO node_hermes_agents (node_id, enabled, autonomous_healing, last_heartbeat, agent_version)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(node_id) DO UPDATE SET
                enabled = excluded.enabled,
                autonomous_healing = excluded.autonomous_healing,
                last_heartbeat = excluded.last_heartbeat,
                agent_version = excluded.agent_version",
            params![
                assoc.node_id,
                assoc.enabled as i32,
                assoc.autonomous_healing as i32,
                assoc.last_heartbeat_unix,
                assoc.agent_version,
            ],
        )?;
        Ok(())
    }

    pub fn load_hermes_agent(&self, node_id: &str) -> Result<Option<HermesAgentAssociation>> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        let row = connection
            .query_row(
                "SELECT node_id, enabled, autonomous_healing, last_heartbeat, agent_version
                 FROM node_hermes_agents
                 WHERE node_id = ?1",
                params![node_id],
                |row| {
                    Ok(HermesAgentAssociation {
                        node_id: row.get(0)?,
                        enabled: row.get::<_, i32>(1)? != 0,
                        autonomous_healing: row.get::<_, i32>(2)? != 0,
                        last_heartbeat_unix: row.get(3)?,
                        agent_version: row.get(4)?,
                    })
                },
            )
            .optional()
            .with_context(|| format!("failed to load Hermes agent for node {node_id}"))?;
        Ok(row)
    }

    /// Record a liveness report from an already-enrolled guest agent.
    ///
    /// Returns whether there was an enabled association to record against. A
    /// heartbeat is a report, not an enrolment: this used to `INSERT` a fully
    /// enabled association — autonomous healing included — for any instance
    /// that sent one, so possession of a credential silently granted a machine
    /// the right to restart a node the operator had never switched on.
    pub fn record_hermes_heartbeat(
        &self,
        node_id: &str,
        agent_version: Option<&str>,
    ) -> Result<bool> {
        crate::types::validate_node_id(node_id)?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        let connection = self.connection()?;
        let version = agent_version.unwrap_or("0.5.0");
        let updated = connection.execute(
            "UPDATE node_hermes_agents
                SET last_heartbeat = ?2, agent_version = ?3
              WHERE node_id = ?1 AND enabled = 1",
            params![node_id, now, version],
        )?;
        Ok(updated > 0)
    }

    pub fn list_hermes_agents(&self) -> Result<Vec<HermesAgentAssociation>> {
        let connection = self.connection()?;
        let mut stmt = connection.prepare(
            "SELECT node_id, enabled, autonomous_healing, last_heartbeat, agent_version
             FROM node_hermes_agents",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(HermesAgentAssociation {
                node_id: row.get(0)?,
                enabled: row.get::<_, i32>(1)? != 0,
                autonomous_healing: row.get::<_, i32>(2)? != 0,
                last_heartbeat_unix: row.get(3)?,
                agent_version: row.get(4)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }
}
