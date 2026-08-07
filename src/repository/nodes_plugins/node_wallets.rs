use anyhow::Result;

use super::*;

impl Repository {
    /// Records which imported wallet profile a node signs with.
    ///
    /// Only the profile reference is stored. The wallet's password is never
    /// persisted: both clients read it in plaintext from their config file, and
    /// this application fails validation on any wallet that carries a plaintext
    /// secret — keeping one in the workspace database would contradict the
    /// boundary it enforces on everyone else.
    pub fn set_node_wallet(&self, node_id: &str, profile_id: Option<&str>) -> Result<()> {
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        match profile_id {
            Some(profile_id) => transaction.execute(
                "INSERT INTO node_wallets (node_id, wallet_profile_id) VALUES (?1, ?2)
                 ON CONFLICT(node_id) DO UPDATE SET wallet_profile_id = excluded.wallet_profile_id",
                rusqlite::params![node_id, profile_id],
            )?,
            None => transaction.execute(
                "DELETE FROM node_wallets WHERE node_id = ?1",
                rusqlite::params![node_id],
            )?,
        };
        transaction.commit()?;
        Ok(())
    }

    /// The wallet profile assigned to a node, or `None` when it signs with
    /// nothing.
    pub fn load_node_wallet(&self, node_id: &str) -> Result<Option<String>> {
        let connection = self.connection()?;
        let mut statement =
            connection.prepare("SELECT wallet_profile_id FROM node_wallets WHERE node_id = ?1")?;
        let mut rows = statement.query(rusqlite::params![node_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        Ok(Some(row.get(0)?))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/repository/node_wallets/tests.rs"]
mod tests;
