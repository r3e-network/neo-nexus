use anyhow::Result;

use super::*;

use crate::roles::NodeRole;

impl Repository {
    /// Records the duty a node is being operated for.
    ///
    /// The role is what decides which service sections a generated config
    /// carries, so it has to outlive the session that chose it — a role held
    /// only in UI state can never reach the generator.
    pub fn set_node_role(&self, node_id: &str, role: Option<NodeRole>) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
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

#[cfg(test)]
#[path = "../../../tests/unit/repository/node_roles/tests.rs"]
mod tests;
