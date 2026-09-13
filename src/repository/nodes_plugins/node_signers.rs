use anyhow::{bail, Context, Result};
use rusqlite::{params, OptionalExtension};

use super::*;
use crate::signing::SignerKeyRef;

/// The signer registry profile explicitly assigned to a node.
///
/// Profiles themselves are resolved from the process signer registry. The
/// workspace stores only the stable, non-secret profile id so a node can never
/// drift onto a default backend when routes or profile order change.
impl Repository {
    pub fn set_node_signer_key(&self, node_id: &str, key: Option<&SignerKeyRef>) -> Result<()> {
        crate::types::validate_node_id(node_id)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let (status_raw, pid) = transaction
            .query_row(
                "SELECT status, pid FROM nodes WHERE id = ?1",
                params![node_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<u32>>(1)?)),
            )
            .with_context(|| format!("node {node_id} was not found"))?;
        let status = NodeStatus::from_str(&status_raw)?;
        if status.is_active() || pid.is_some() {
            bail!("stop node {node_id} before changing its signer profile");
        }

        match key {
            Some(key) => {
                // Reconstructing validates both components even when a caller
                // created the value in a future version with looser rules.
                let key = SignerKeyRef::new(&key.backend_id, &key.key_id)?;
                let existing_owner: Option<String> = transaction
                    .query_row(
                        "SELECT node_id FROM node_signer_bindings WHERE backend_id = ?1 AND key_id = ?2",
                        params![key.backend_id, key.key_id],
                        |row| row.get(0),
                    )
                    .optional()?;
                if let Some(owner) = existing_owner {
                    if owner != node_id {
                        bail!(
                            "IAM Isolation Violation: Signer key '{}/{}' is already exclusively allocated to instance '{owner}'. Cross-node key usage is strictly forbidden.",
                            key.backend_id,
                            key.key_id
                        );
                    }
                }
                transaction.execute(
                    "INSERT INTO node_signer_bindings (node_id, backend_id, key_id)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(node_id) DO UPDATE SET
                    backend_id = excluded.backend_id,
                    key_id = excluded.key_id",
                    params![node_id, key.backend_id, key.key_id],
                )?
            }
            None => transaction.execute(
                "DELETE FROM node_signer_bindings WHERE node_id = ?1",
                params![node_id],
            )?,
        };
        transaction.commit()?;
        Ok(())
    }

    pub fn list_all_signer_bindings(&self) -> Result<Vec<(String, SignerKeyRef)>> {
        let connection = self.connection()?;
        let mut stmt =
            connection.prepare("SELECT node_id, backend_id, key_id FROM node_signer_bindings")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            let (node_id, backend_id, key_id) = row?;
            if let Ok(k) = SignerKeyRef::new(backend_id, key_id) {
                results.push((node_id, k));
            }
        }
        Ok(results)
    }

    pub fn find_node_by_signer_key(
        &self,
        backend_id: &str,
        key_id: &str,
    ) -> Result<Option<String>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT node_id FROM node_signer_bindings WHERE backend_id = ?1 AND key_id = ?2",
                params![backend_id, key_id],
                |row| row.get(0),
            )
            .optional()
            .with_context(|| format!("failed to find owner for signer key {backend_id}/{key_id}"))
    }

    pub fn load_node_signer_key(&self, node_id: &str) -> Result<Option<SignerKeyRef>> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        let stored = connection
            .query_row(
                "SELECT backend_id, key_id
                 FROM node_signer_bindings
                 WHERE node_id = ?1",
                params![node_id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .with_context(|| format!("failed to load signer binding for node {node_id}"))?;
        stored
            .map(|(backend_id, key_id)| SignerKeyRef::new(backend_id, key_id))
            .transpose()
            .with_context(|| format!("node {node_id} has an invalid signer binding"))
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/repository/node_signers/tests.rs"]
mod tests;
