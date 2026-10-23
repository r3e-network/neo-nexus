use super::*;

mod api_tokens;
mod connection;
mod getters;
mod indexes;
mod migrations;
mod tables;

/// Version of the schema built by [`create_schema`].
///
/// **Bump this whenever `create_tables`, `create_api_token_table`,
/// `create_indexes`, or `apply_migrations` changes.** It is the only thing that
/// makes `initialize` skip the bootstrap on an existing workspace, so forgetting
/// to bump it ships a schema change that never reaches an existing database.
const SCHEMA_VERSION: u32 = 1;

/// Journal mode for every workspace.
///
/// WAL was the previous default. It suits concurrent readers, but this product
/// opens a connection per operation instead of holding a pool, and a WAL
/// database removes its `-shm` file when the last connection closes. Under
/// concurrent writes that transition intermittently made the next writer see
/// `SQLITE_READONLY` — "attempt to write a readonly database" — on Windows,
/// reachable both from the web suite and from a CLI command inspecting a live
/// server's workspace. The workload is a management console: a handful of small
/// writes per second, not a read-heavy analytics store. The reader/writer
/// exclusion of `TRUNCATE` costs nothing that matters here and buys a
/// deterministic attach.
const JOURNAL_MODE: &str = "TRUNCATE";

impl Repository {
    /// Prepare a workspace for use, writing only when it is not already prepared.
    ///
    /// This used to set the journal mode and run the full DDL on every `open`,
    /// including opens of a database a running server is writing. Those
    /// statements take write locks, so a second handle — a CLI inspection
    /// command, or a test harness — could fail intermittently with
    /// `SQLITE_READONLY` while the server was busy. An up-to-date workspace now
    /// answers one read-only `PRAGMA user_version` and returns.
    pub(in crate::repository) fn initialize(&self) -> Result<()> {
        let connection = self.connection()?;
        if schema_version(&connection)? == SCHEMA_VERSION {
            return Ok(());
        }
        connection
            .execute_batch(&format!("PRAGMA journal_mode = {JOURNAL_MODE};"))
            .context("failed to configure the SQLite journal mode")?;
        create_schema(&connection)?;
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION)
            .context("failed to record the workspace schema version")?;
        Ok(())
    }
}

fn schema_version(connection: &Connection) -> Result<u32> {
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .context("failed to read the workspace schema version")
}

/// Build the whole schema on a connection.
///
/// Taking a connection rather than a path is what lets the integrity checker
/// construct a **reference workspace in memory** and compare a real one against
/// it, instead of keeping a second hand-maintained declaration of the same
/// tables — which had already drifted, silently omitting the table whose unique
/// index prevents two nodes sharing a signing key.
pub(crate) fn create_schema(connection: &Connection) -> Result<()> {
    tables::create_tables(connection)?;
    api_tokens::create_api_token_table(connection)?;
    indexes::create_indexes(connection)?;
    migrations::apply_migrations(connection)?;
    Ok(())
}
