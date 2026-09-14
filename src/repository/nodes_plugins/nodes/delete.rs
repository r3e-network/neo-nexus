use super::*;

impl Repository {
    pub fn delete_node(&self, id: &str) -> Result<()> {
        crate::types::validate_node_id(id)?;
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        let (status_raw, pid) = transaction
            .query_row(
                "SELECT status, pid FROM nodes WHERE id = ?1",
                params![id],
                |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<u32>>(1)?)),
            )
            .with_context(|| format!("node {id} was not found"))?;
        let status = NodeStatus::from_str(&status_raw)?;
        if status.is_active() || pid.is_some() {
            anyhow::bail!("stop node {id} and confirm its process exited before deleting it");
        }
        transaction.execute(
            "DELETE FROM plugin_installations WHERE node_id = ?1",
            params![id],
        )?;
        transaction.execute(
            "DELETE FROM rpc_health_checks WHERE node_id = ?1",
            params![id],
        )?;
        transaction.execute("DELETE FROM plugin_states WHERE node_id = ?1", params![id])?;
        revoke_tokens_confined_to(&transaction, id)?;
        let deleted = transaction.execute("DELETE FROM nodes WHERE id = ?1", params![id])?;
        ensure_affected_rows(deleted, "node", id)?;
        transaction.commit()?;
        Ok(())
    }
}

/// Revoke the credentials that spoke only for this node.
///
/// `api_tokens` has no foreign key to `nodes` — the node scope lives inside the
/// permissions string as `hermes_agent:<node-id>` — so deleting a node left its
/// guest-agent token listed as Active forever. The surviving credential was
/// inert (uuids are never reused, and the confinement guard answers 404 for a
/// node that does not exist), so this is hygiene rather than access. But a
/// credential list that shows a live token for a node nobody can name is one an
/// operator cannot audit.
///
/// Only tokens confined to **this node and nothing else** are revoked. A token
/// that also carries a fleet-wide grant was issued for something broader, and
/// removing it because one node went away would silently break whatever else
/// holds it.
fn revoke_tokens_confined_to(transaction: &rusqlite::Transaction<'_>, node_id: &str) -> Result<()> {
    let confined: Vec<String> = {
        let mut statement = transaction.prepare("SELECT id, permissions FROM api_tokens")?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
            .into_iter()
            .filter(|(_, permissions)| grants_only_this_node(permissions, node_id))
            .map(|(token_id, _)| token_id)
            .collect()
    };
    for token_id in confined {
        transaction.execute("DELETE FROM api_tokens WHERE id = ?1", params![token_id])?;
    }
    Ok(())
}

/// Whether every grant on this token is a guest-agent grant for `node_id`.
///
/// An unparseable or empty permission set is **not** treated as confined: a
/// token whose grants cannot be read is not one to delete on a guess.
fn grants_only_this_node(permissions_csv: &str, node_id: &str) -> bool {
    let grants: Vec<&str> = permissions_csv
        .split(',')
        .map(str::trim)
        .filter(|grant| !grant.is_empty())
        .collect();
    !grants.is_empty()
        && grants
            .iter()
            .all(|grant| *grant == format!("hermes_agent:{node_id}"))
}

#[cfg(test)]
#[path = "../../../../tests/unit/repository/node_delete/tests.rs"]
mod tests;
