use anyhow::Result;
use rusqlite::{params, Connection};

/// Create the api_tokens table if it doesn't exist.
/// This stores API authentication tokens for programmatic access.
pub(in crate::repository::schema) fn create_api_token_table(connection: &Connection) -> Result<()> {
    connection.execute_batch(
        "CREATE TABLE IF NOT EXISTS api_tokens (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            permissions TEXT NOT NULL DEFAULT '',
            created_at_unix INTEGER NOT NULL,
            expires_at_unix INTEGER,
            secret_hash BLOB NOT NULL CHECK(length(secret_hash) = 32)
        );",
    )?;

    // Add indexes for common queries
    connection.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_api_tokens_created_desc
         ON api_tokens (created_at_unix DESC);",
    )?;

    Ok(())
}
