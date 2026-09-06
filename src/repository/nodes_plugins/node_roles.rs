use anyhow::{bail, Context, Result};

use super::*;

use crate::roles::{NodeRole, RolePlan};

impl Repository {
    /// Records the duty a node is being operated for.
    ///
    /// The role is what decides which service sections a generated config
    /// carries, so it has to outlive the session that chose it — a role held
    /// only in UI state can never reach the generator.
    pub fn set_node_role(&self, node_id: &str, role: Option<NodeRole>) -> Result<()> {
        crate::types::validate_node_id(node_id)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        ensure_role_change_is_safe(&transaction, node_id)?;
        match role {
            Some(role) => transaction.execute(
                "INSERT INTO node_roles (node_id, role) VALUES (?1, ?2)
                 ON CONFLICT(node_id) DO UPDATE SET role = excluded.role",
                rusqlite::params![node_id, role.persist_key()],
            )?,
            None => transaction.execute(
                "DELETE FROM node_roles WHERE node_id = ?1",
                rusqlite::params![node_id],
            )?,
        };
        transaction.commit()?;
        Ok(())
    }

    /// Atomically apply a planned duty and every plugin state it requires.
    /// The UI never writes one without the other, so a crash cannot leave a
    /// persisted Consensus role with the old relay plugin set.
    pub fn apply_node_role_plan(&self, node_id: &str, plan: &RolePlan) -> Result<()> {
        crate::types::validate_node_id(node_id)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let (node_type_raw, storage_raw) = ensure_role_change_is_safe(&transaction, node_id)?;
        let node_type = NodeType::from_str(&node_type_raw)?;
        let storage = StorageEngine::from_str(&storage_raw)?;
        if node_type != plan.node_type || storage != plan.storage_engine {
            bail!("node {node_id} changed after its role plan was prepared; reload the plan");
        }
        transaction.execute(
            "INSERT INTO node_roles (node_id, role) VALUES (?1, ?2)
             ON CONFLICT(node_id) DO UPDATE SET role = excluded.role",
            rusqlite::params![node_id, plan.role.persist_key()],
        )?;
        for change in &plan.plugin_changes {
            transaction.execute(
                "INSERT INTO plugin_states (node_id, plugin_id, enabled)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(node_id, plugin_id) DO UPDATE SET enabled = excluded.enabled",
                rusqlite::params![node_id, change.plugin_id.to_string(), change.enabled],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    /// The node's recorded duty, or `None` when it has not been assigned one.
    /// An unrecognised stored value also reads as `None` rather than failing:
    /// a role removed in a later version must not make the node unloadable.
    pub fn load_node_role(&self, node_id: &str) -> Result<Option<NodeRole>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT role FROM node_roles WHERE node_id = ?1")?;
        let mut rows = statement.query(rusqlite::params![node_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let key: String = row.get(0)?;
        Ok(NodeRole::from_persist_key(&key))
    }
}

fn ensure_role_change_is_safe(
    transaction: &rusqlite::Transaction<'_>,
    node_id: &str,
) -> Result<(String, String)> {
    let (node_type, storage, status_raw, pid) = transaction
        .query_row(
            "SELECT node_type, storage_engine, status, pid FROM nodes WHERE id = ?1",
            rusqlite::params![node_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, Option<u32>>(3)?,
                ))
            },
        )
        .with_context(|| format!("node {node_id} was not found"))?;
    let status = NodeStatus::from_str(&status_raw)?;
    if status.is_active() || pid.is_some() {
        bail!("stop node {node_id} before changing its duty");
    }
    Ok((node_type, storage))
}

#[cfg(test)]
#[path = "../../../tests/unit/repository/node_roles/tests.rs"]
mod tests;
