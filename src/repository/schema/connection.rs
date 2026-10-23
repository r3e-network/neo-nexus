use std::{thread, time::Duration};

use rusqlite::ErrorCode;

use super::*;

/// How many times a connection is re-established when SQLite reports a
/// transient failure at attach time.
const CONNECT_ATTEMPTS: usize = 5;

impl Repository {
    pub(in crate::repository) fn connection(&self) -> Result<Connection> {
        let mut attempt = 0;
        loop {
            attempt += 1;
            match self.attach() {
                Ok(connection) => return Ok(connection),
                Err(error) if is_transient(&error) && attempt < CONNECT_ATTEMPTS => {
                    // A failed attach leaves nothing to clean up: the handle is
                    // dropped inside `attach`. Backing off briefly is enough for
                    // the competing connection to finish and release the file.
                    thread::sleep(Duration::from_millis(20 * attempt as u64));
                }
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!("failed to open database {}", self.db_path.display())
                    });
                }
            }
        }
    }

    /// Open a connection and make it actually touch the database file.
    ///
    /// SQLite opens the file lazily, on the first statement. Leaving that to the
    /// caller means the first attach — the one step that can fail transiently —
    /// happens inside an operation the caller has already committed to, where it
    /// surfaces as "attempt to write a readonly database". Reading
    /// `schema_version` is cheap, takes no write lock, and forces the file (and,
    /// for a workspace still journaling in WAL, its `-shm`) to be attached, so a
    /// bad connection is discarded here instead.
    fn attach(&self) -> Result<Connection> {
        let connection = Connection::open(&self.db_path)?;
        connection.execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )?;
        connection.execute_batch("PRAGMA schema_version;")?;
        Ok(connection)
    }
}

/// Whether retrying a fresh connection can plausibly succeed.
///
/// `ReadOnly` is the transient attach failure described above — still reachable
/// on a workspace written before the journal mode changed, until its first
/// `TRUNCATE` open succeeds. `Busy` and `Locked` mean another connection held
/// the database past our busy timeout; a new attempt gets a new timeout window.
fn is_transient(error: &anyhow::Error) -> bool {
    error
        .downcast_ref::<rusqlite::Error>()
        .and_then(rusqlite::Error::sqlite_error_code)
        .is_some_and(|code| {
            matches!(
                code,
                ErrorCode::ReadOnly | ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
            )
        })
}
