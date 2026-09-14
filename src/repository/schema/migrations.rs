use super::*;

pub(in crate::repository::schema) fn apply_migrations(connection: &Connection) -> Result<()> {
    add_column_if_missing(
        connection,
        "nodes",
        "runtime_version",
        "TEXT NOT NULL DEFAULT 'latest'",
    )?;
    add_column_if_missing(
        connection,
        "nodes",
        "storage_engine",
        "TEXT NOT NULL DEFAULT 'leveldb'",
    )?;
    add_column_if_missing(
        connection,
        "nodes",
        "rpc_port",
        "INTEGER NOT NULL DEFAULT 10332",
    )?;
    add_column_if_missing(
        connection,
        "nodes",
        "p2p_port",
        "INTEGER NOT NULL DEFAULT 10333",
    )?;
    add_column_if_missing(connection, "nodes", "ws_port", "INTEGER")?;
    add_column_if_missing(connection, "fast_sync_snapshots", "source_url", "TEXT")?;
    add_column_if_missing(
        connection,
        "fast_sync_snapshots",
        "download_file_name",
        "TEXT",
    )?;
    add_column_if_missing(
        connection,
        "fast_sync_snapshots",
        "download_max_bytes",
        "INTEGER NOT NULL DEFAULT 68719476736",
    )?;
    add_column_if_missing(
        connection,
        "runtime_installations",
        "signature_verified",
        "INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column_if_missing(
        connection,
        "runtime_installations",
        "signer_public_key",
        "TEXT",
    )?;
    enforce_signer_lease_exclusivity(connection)?;
    enforce_unique_node_names(connection)?;
    record_node_timestamps(connection)?;
    Ok(())
}

/// Make "one signer key, one instance" a property of the database.
///
/// The rule was enforced only by a read-then-write check in application code.
/// That is the right check to keep — it produces a readable refusal — but it
/// cannot speak for a workspace edited by hand, restored from an older release,
/// or written by a future caller that forgets to ask. Two instances sharing one
/// consensus key is the double-signing hazard the whole lease model exists to
/// prevent, so the constraint belongs where it cannot be bypassed.
/// A node name is a join key, so it has to be unique.
///
/// `nodes.name` had no UNIQUE constraint, while the private-network launch-pack
/// exporter keys members **by name** — uniqueness was checked at plan time and
/// never enforced by the schema, so two nodes sharing a name silently wrote one
/// member's config twice. Every CLI command that takes `<node-name>` has the
/// same ambiguity.
///
/// Duplicates are renamed rather than removed: a node is an operator's, and
/// losing one to a migration is worse than any naming problem. The rename is
/// deterministic, visible in the list, and journalled so it is not a surprise.
fn enforce_unique_node_names(connection: &Connection) -> Result<()> {
    let duplicates: Vec<(String, String)> = {
        // Oldest row wins its name; the rest are suffixed. `rowid` is the
        // insertion order, which is the closest thing to "who had it first"
        // that the table records.
        let mut statement = connection.prepare(
            "SELECT id, name FROM nodes
              WHERE rowid NOT IN (SELECT MIN(rowid) FROM nodes GROUP BY name)",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };

    for (id, name) in duplicates {
        // The id suffix is short, stable, and already how the console
        // disambiguates two identically-named rows to the eye.
        let suffix = id.rsplit('-').next().unwrap_or(&id);
        let renamed = format!("{name}-{}", &suffix[..suffix.len().min(6)]);
        connection.execute(
            "UPDATE nodes SET name = ?2 WHERE id = ?1",
            params![id, renamed],
        )?;
        let now = crate::repository::helpers::current_unix_time()?;
        connection.execute(
            "INSERT INTO runtime_events
                (occurred_at_unix, node_id, node_name, kind, severity, message)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                now,
                id,
                renamed,
                crate::events::EventKind::NodeUpdated.to_string(),
                crate::events::EventSeverity::Warning.to_string(),
                format!(
                    "Two nodes were named {name}. A node name is a join key — the launch-pack exporter selects members by it — so this one was renamed to {renamed}. Rename it to something you recognise."
                ),
            ],
        )?;
    }

    connection.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_nodes_unique_name ON nodes (name)",
        [],
    )?;
    Ok(())
}

/// When a node was added and when it last changed.
///
/// `nodes` had neither, unlike every other entity in the schema, so "what
/// version was this on last Tuesday" and "when did this node join the fleet"
/// were unanswerable. Existing rows are stamped with the migration's own time:
/// a made-up creation date would be worse than one that says "at least this
/// old", and they are equal for a fresh workspace.
fn record_node_timestamps(connection: &Connection) -> Result<()> {
    let now = crate::repository::helpers::current_unix_time()?;
    add_column_if_missing(
        connection,
        "nodes",
        "created_at_unix",
        &format!("INTEGER NOT NULL DEFAULT {now}"),
    )?;
    add_column_if_missing(
        connection,
        "nodes",
        "updated_at_unix",
        &format!("INTEGER NOT NULL DEFAULT {now}"),
    )?;
    Ok(())
}

fn enforce_signer_lease_exclusivity(connection: &Connection) -> Result<()> {
    // A workspace may already be in the state the index forbids. Releasing the
    // contested key from *every* claimant is the only safe resolution: there is
    // nothing in the row to say which instance was meant to have it, and a node
    // with a signing duty and no binding refuses to launch, which is the
    // outcome we want while the operator decides. Silently picking one would
    // start a validator on a key its operator never confirmed.
    let mut contested = connection.prepare(
        "SELECT backend_id, key_id, group_concat(node_id, ', '), count(*)
           FROM node_signer_bindings
          GROUP BY backend_id, key_id
         HAVING count(*) > 1",
    )?;
    let conflicts: Vec<(String, String, String)> = contested
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    drop(contested);

    for (backend_id, key_id, node_ids) in conflicts {
        connection.execute(
            "DELETE FROM node_signer_bindings WHERE backend_id = ?1 AND key_id = ?2",
            params![backend_id, key_id],
        )?;
        let now = crate::repository::helpers::current_unix_time()?;
        connection.execute(
            "INSERT INTO runtime_events
                (occurred_at_unix, node_id, node_name, kind, severity, message)
             VALUES (?1, NULL, NULL, ?2, ?3, ?4)",
            params![
                now,
                crate::events::EventKind::NodeSignerBound.to_string(),
                crate::events::EventSeverity::Critical.to_string(),
                format!(
                    "Signer lease {backend_id}/{key_id} was held by more than one instance ({node_ids}), which is the double-signing state the lease model forbids. The lease has been released from all of them; re-bind it to exactly one instance before starting a signing duty."
                ),
            ],
        )?;
    }

    connection.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_node_signer_bindings_exclusive_lease
         ON node_signer_bindings (backend_id, key_id)",
        [],
    )?;
    Ok(())
}
