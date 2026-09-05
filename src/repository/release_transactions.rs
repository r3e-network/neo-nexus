//! Durable release transaction records.
//!
//! Filesystem publication cannot participate in SQLite's transaction, so the
//! database records the intent and every phase. A manager restart can inspect
//! the phase and the backup pointer rather than guessing whether a half-upgrade
//! happened. The orchestrator owns the compensating rollback.

use anyhow::{Context, Result};
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use super::Repository;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseTransaction {
    pub id: String,
    pub node_id: String,
    pub phase: String,
    pub previous_version: String,
    pub previous_binary_path: String,
    pub target_version: String,
    pub target_binary_path: String,
    pub backup_dir: String,
    pub last_error: Option<String>,
    pub created_at_unix: u64,
    pub updated_at_unix: u64,
}

impl Repository {
    /// Records the intent of an upgrade before any file write. The arguments
    /// mirror the durable record exactly, so callers cannot drop a field.
    #[allow(clippy::too_many_arguments)]
    pub fn begin_release_transaction(
        &self,
        node_id: &str,
        previous_version: &str,
        previous_binary_path: &str,
        target_version: &str,
        target_binary_path: &str,
        backup_dir: &str,
        now_unix: u64,
    ) -> Result<ReleaseTransaction> {
        let transaction = ReleaseTransaction {
            id: format!("release-{}", Uuid::new_v4().simple()),
            node_id: node_id.to_string(),
            phase: "requested".to_string(),
            previous_version: previous_version.to_string(),
            previous_binary_path: previous_binary_path.to_string(),
            target_version: target_version.to_string(),
            target_binary_path: target_binary_path.to_string(),
            backup_dir: backup_dir.to_string(),
            last_error: None,
            created_at_unix: now_unix,
            updated_at_unix: now_unix,
        };
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO release_transactions
             (id,node_id,phase,previous_version,previous_binary_path,target_version,
              target_binary_path,backup_dir,last_error,created_at_unix,updated_at_unix)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,NULL,?9,?9)",
            params![
                transaction.id,
                transaction.node_id,
                transaction.phase,
                transaction.previous_version,
                transaction.previous_binary_path,
                transaction.target_version,
                transaction.target_binary_path,
                transaction.backup_dir,
                now_unix as i64,
            ],
        )?;
        Ok(transaction)
    }

    /// Advances a transaction only from the phase the caller actually read.
    /// Two controllers observing `preflight` cannot both publish `applied`.
    pub fn advance_release_transaction(
        &self,
        id: &str,
        expected_phase: &str,
        next_phase: &str,
        error: Option<&str>,
        now_unix: u64,
    ) -> Result<bool> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE release_transactions SET phase=?1,last_error=?2,updated_at_unix=?3
             WHERE id=?4 AND phase=?5",
            params![next_phase, error, now_unix as i64, id, expected_phase],
        )?;
        Ok(changed == 1)
    }

    pub fn get_release_transaction(&self, id: &str) -> Result<Option<ReleaseTransaction>> {
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT id,node_id,phase,previous_version,previous_binary_path,
                        target_version,target_binary_path,backup_dir,last_error,
                        created_at_unix,updated_at_unix
                 FROM release_transactions WHERE id=?1",
                params![id],
                |row| {
                    Ok(ReleaseTransaction {
                        id: row.get(0)?,
                        node_id: row.get(1)?,
                        phase: row.get(2)?,
                        previous_version: row.get(3)?,
                        previous_binary_path: row.get(4)?,
                        target_version: row.get(5)?,
                        target_binary_path: row.get(6)?,
                        backup_dir: row.get(7)?,
                        last_error: row.get(8)?,
                        created_at_unix: row.get(9)?,
                        updated_at_unix: row.get(10)?,
                    })
                },
            )
            .optional()
            .context("failed to load release transaction")
    }

    pub fn list_pending_release_transactions(&self) -> Result<Vec<ReleaseTransaction>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id,node_id,phase,previous_version,previous_binary_path,
                    target_version,target_binary_path,backup_dir,last_error,
                    created_at_unix,updated_at_unix
             FROM release_transactions
             WHERE phase NOT IN ('committed','rolled-back','failed')
             ORDER BY created_at_unix,id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(ReleaseTransaction {
                id: row.get(0)?,
                node_id: row.get(1)?,
                phase: row.get(2)?,
                previous_version: row.get(3)?,
                previous_binary_path: row.get(4)?,
                target_version: row.get(5)?,
                target_binary_path: row.get(6)?,
                backup_dir: row.get(7)?,
                last_error: row.get(8)?,
                created_at_unix: row.get(9)?,
                updated_at_unix: row.get(10)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to list pending release transactions")
    }
}
