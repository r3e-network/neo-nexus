use super::*;
use std::{
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, MutexGuard},
};

pub(in crate::repository) enum RepositoryConnection<'a> {
    Owned(Connection),
    Restore(MutexGuard<'a, Connection>),
}

impl Deref for RepositoryConnection<'_> {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        match self {
            Self::Owned(connection) => connection,
            Self::Restore(connection) => connection,
        }
    }
}

impl DerefMut for RepositoryConnection<'_> {
    fn deref_mut(&mut self) -> &mut Connection {
        match self {
            Self::Owned(connection) => connection,
            Self::Restore(connection) => connection,
        }
    }
}

pub(in crate::repository) enum WriteScope<'a> {
    Transaction(rusqlite::Transaction<'a>),
    Savepoint(rusqlite::Savepoint<'a>),
}

impl Deref for WriteScope<'_> {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        match self {
            Self::Transaction(transaction) => transaction,
            Self::Savepoint(savepoint) => savepoint,
        }
    }
}

impl WriteScope<'_> {
    pub(in crate::repository) fn commit(self) -> rusqlite::Result<()> {
        match self {
            Self::Transaction(transaction) => transaction.commit(),
            Self::Savepoint(savepoint) => savepoint.commit(),
        }
    }
}

impl RepositoryConnection<'_> {
    pub(in crate::repository) fn transaction(&mut self) -> rusqlite::Result<WriteScope<'_>> {
        if matches!(self, Self::Restore(_)) {
            Ok(WriteScope::Savepoint(self.deref_mut().savepoint()?))
        } else {
            Ok(WriteScope::Transaction(self.deref_mut().transaction()?))
        }
    }
}

impl Repository {
    pub(in crate::repository) fn connection(&self) -> Result<RepositoryConnection<'_>> {
        if let Some(connection) = &self.restore_connection {
            return Ok(RepositoryConnection::Restore(connection.lock().map_err(
                |_| anyhow::anyhow!("backup transaction lock poisoned"),
            )?));
        }
        Ok(RepositoryConnection::Owned(self.open_connection()?))
    }

    fn open_connection(&self) -> Result<Connection> {
        let connection = Connection::open(&self.db_path)
            .with_context(|| format!("failed to open database {}", self.db_path.display()))?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .context("failed to enable SQLite foreign-key enforcement")?;
        Ok(connection)
    }

    /// Reuse the regular repository restore methods in one SQLite transaction.
    /// Their local transactions become savepoints. An import is fully visible
    /// or fully rolled back, including profile/binding changes before a failure.
    pub(crate) fn in_restore_transaction<T>(
        &self,
        apply: impl FnOnce(&Repository) -> Result<T>,
    ) -> Result<T> {
        let connection = self.open_connection()?;
        connection.execute_batch("BEGIN IMMEDIATE")?;
        let repository = Self {
            db_path: self.db_path.clone(),
            restore_connection: Some(Arc::new(Mutex::new(connection))),
        };
        let result = apply(&repository);
        let connection = repository.connection()?;
        match result {
            Ok(value) => {
                connection.execute_batch("COMMIT")?;
                Ok(value)
            }
            Err(error) => {
                connection
                    .execute_batch("ROLLBACK")
                    .context("backup restore failed and transaction rollback failed")?;
                Err(error)
            }
        }
    }
}
