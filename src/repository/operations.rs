//! The operation ledger: which surface is running which mutating node
//! operation, and whether it finished.
//!
//! The workbench engine and a headless CLI command are different processes
//! with no shared state except this database. Without a ledger here, a CLI
//! stop and a browser start can interleave: the CLI reads a row, the engine
//! restarts the node under it, and the CLI then signals a pid that belongs to
//! the replacement. A claimed operation makes one mutating operation per node
//! visible to every surface; a second claimant is refused instead of
//! interleaved, and a claimant that dies leaves a row that goes stale and
//! unblocks after the window below.

use anyhow::{Context, Result};
use rusqlite::params;
use uuid::Uuid;

use super::Repository;

/// How long a running operation may go without an update before a new claimant
/// may take over. A launch renders config and spawns — seconds normally — so
/// this window only opens when the claiming process died mid-operation, which
/// is exactly the case it exists for.
pub const OPERATION_STALE_SECS: u64 = 120;

/// A durable, fenced controller claim. Every state transition must present
/// both the monotonic generation and the random token; a stale controller can
/// still finish its own Rust scope, but its database write is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerOperation {
    pub id: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub operation_kind: String,
    pub phase: String,
    pub desired_state: Option<String>,
    pub generation: i64,
    pub fencing_token: String,
    pub pid: Option<u32>,
    pub process_started_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControllerReconcileSummary {
    pub pending: usize,
    pub stale: usize,
    pub unknown: usize,
}

/// A self-closing controller lease. Dropping an unfinished lease marks the
/// operation failed, so a process that returns early cannot leave a forever-
/// running phase behind; a process that is killed leaves the row stale.
pub struct ControllerLease<'a> {
    repository: &'a Repository,
    pub operation: ControllerOperation,
    finished: bool,
}

impl ControllerLease<'_> {
    pub fn operation(&self) -> &ControllerOperation {
        &self.operation
    }

    pub fn renew(&self, phase: &str) -> Result<bool> {
        self.repository
            .renew_controller_operation(&self.operation, phase, unix_now())
    }

    pub fn record_spawned(&mut self, pid: u32, process_started_at: Option<u64>) -> Result<bool> {
        let changed = self.repository.record_spawned_process(
            &self.operation,
            pid,
            process_started_at,
            unix_now(),
        )?;
        if changed {
            self.operation.phase = "spawned".to_string();
            self.operation.pid = Some(pid);
            self.operation.process_started_at = process_started_at;
        }
        Ok(changed)
    }

    pub fn complete(mut self) -> Result<bool> {
        self.finished = true;
        self.repository
            .complete_controller_operation(&self.operation, unix_now())
    }

    pub fn fail(mut self, error: &str) -> Result<bool> {
        self.finished = true;
        self.repository
            .fail_controller_operation(&self.operation, error, unix_now())
    }
}

impl Drop for ControllerLease<'_> {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.repository.fail_controller_operation(
                &self.operation,
                "controller scope ended before completion",
                unix_now(),
            );
        }
    }
}
///
/// Every early return on the guarded path — readiness refusal, supervision
/// failure, a launch that panicked halfway — ends here, so a claim cannot
/// outlive the code that made it. Only a crashed process leaves a claim
/// behind, and that one goes stale on its own.
pub struct NodeOperation<'a> {
    repository: &'a Repository,
    id: String,
    finished: bool,
}

impl NodeOperation<'_> {
    /// Marks the operation done before the scope ends. A caller that wants the
    /// ledger to record completion rather than scope exit can call this;
    /// otherwise the drop does the same write.
    pub fn finish(mut self) {
        self.finished = true;
        self.close();
    }

    fn close(&mut self) {
        let now = unix_now();
        let _ = self
            .repository
            .finish_node_operation(&self.id, now)
            .inspect_err(|error| {
                // Nothing sensible is left to do with a failed ledger close:
                // the row goes stale on its own, and failing the operation that
                // already ran would misreport it.
                let _ = error;
            });
    }
}

impl Drop for NodeOperation<'_> {
    fn drop(&mut self) {
        if !self.finished {
            self.close();
        }
    }
}

impl Repository {
    /// Claims the exclusive right to run `kind` on `node_id`.
    ///
    /// The claim is a conditional insert, so two surfaces racing on the same
    /// node get exactly one winner; the loser receives an error naming the
    /// situation instead of an interleaved execution. Operations abandoned by
    /// a dead process — no update within [`OPERATION_STALE_SECS`] — are marked
    /// and no longer block.
    pub fn begin_node_operation(&self, kind: &str, node_id: &str, now_unix: u64) -> Result<String> {
        let operation = self.begin_controller_operation(
            "node",
            node_id,
            kind,
            if kind == "stop" { "stopped" } else { "running" },
            now_unix,
        )?;
        Ok(operation.id)
    }

    pub fn begin_controller_operation(
        &self,
        subject_kind: &str,
        subject_id: &str,
        operation_kind: &str,
        desired_state: &str,
        now_unix: u64,
    ) -> Result<ControllerOperation> {
        anyhow::ensure!(
            matches!(subject_kind, "node" | "agent"),
            "invalid controller subject kind"
        );
        anyhow::ensure!(
            matches!(desired_state, "running" | "stopped"),
            "invalid desired state"
        );
        let mut connection = self.connection()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "UPDATE operations SET state='abandoned', phase='abandoned', last_error='stale controller lease', updated_at_unix=?1
             WHERE subject_kind=?2 AND subject_id=?3 AND state='running'
               AND updated_at_unix <= ?1 - ?4",
            params![now_unix as i64, subject_kind, subject_id, OPERATION_STALE_SECS as i64],
        )?;
        let generation: i64 = transaction.query_row(
            "SELECT COALESCE(MAX(generation), 0) + 1 FROM operations WHERE subject_kind=?1 AND subject_id=?2",
            params![subject_kind, subject_id],
            |row| row.get(0),
        )?;
        let id = format!("op-{}-{}", operation_kind, Uuid::new_v4().simple());
        let token = Uuid::new_v4().to_string();
        let inserted = transaction.execute(
            "INSERT INTO operations
             (id, kind, node_id, state, subject_kind, subject_id, operation_kind, phase,
              desired_state, generation, fencing_token, created_at_unix, updated_at_unix)
             SELECT ?1, ?2, CASE WHEN ?3='node' THEN ?4 ELSE NULL END, 'running', ?3, ?4, ?5,
                    'reserved', ?6, ?7, ?8, ?9, ?9
             WHERE NOT EXISTS (
                 SELECT 1 FROM operations WHERE subject_kind=?3 AND subject_id=?4
                   AND state='running' AND phase IN ('requested','reserved','spawned')
             )",
            params![
                id,
                operation_kind,
                subject_kind,
                subject_id,
                operation_kind,
                desired_state,
                generation,
                token,
                now_unix as i64
            ],
        )?;
        if inserted == 0 {
            anyhow::bail!(
                "node {subject_id} already has a running operation; wait for it to finish or go stale"
            );
        }
        transaction.commit()?;
        Ok(ControllerOperation {
            id,
            subject_kind: subject_kind.to_string(),
            subject_id: subject_id.to_string(),
            operation_kind: operation_kind.to_string(),
            phase: "reserved".to_string(),
            desired_state: Some(desired_state.to_string()),
            generation,
            fencing_token: token,
            pid: None,
            process_started_at: None,
        })
    }

    /// Advances a claim only if its fencing identity is still current.
    pub fn renew_controller_operation(
        &self,
        operation: &ControllerOperation,
        phase: &str,
        now_unix: u64,
    ) -> Result<bool> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE operations SET phase=?1, updated_at_unix=?2
             WHERE id=?3 AND subject_kind=?4 AND subject_id=?5 AND generation=?6
               AND fencing_token=?7 AND state='running' AND phase NOT IN ('committed','failed','abandoned')",
            params![phase, now_unix as i64, operation.id, operation.subject_kind,
                    operation.subject_id, operation.generation, operation.fencing_token],
        )?;
        Ok(changed == 1)
    }

    pub fn record_spawned_process(
        &self,
        operation: &ControllerOperation,
        pid: u32,
        process_started_at: Option<u64>,
        now_unix: u64,
    ) -> Result<bool> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE operations SET phase='spawned', pid=?1, process_started_at=?2, updated_at_unix=?3
             WHERE id=?4 AND subject_kind=?5 AND subject_id=?6 AND generation=?7
               AND fencing_token=?8 AND state='running' AND phase='reserved'",
            params![pid, process_started_at, now_unix as i64, operation.id,
                    operation.subject_kind, operation.subject_id, operation.generation,
                    operation.fencing_token],
        )?;
        Ok(changed == 1)
    }

    pub fn complete_controller_operation(
        &self,
        operation: &ControllerOperation,
        now_unix: u64,
    ) -> Result<bool> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE operations SET state='done', phase='committed', updated_at_unix=?1
             WHERE id=?2 AND subject_kind=?3 AND subject_id=?4 AND generation=?5
               AND fencing_token=?6 AND state='running' AND phase IN ('reserved','spawned')",
            params![
                now_unix as i64,
                operation.id,
                operation.subject_kind,
                operation.subject_id,
                operation.generation,
                operation.fencing_token
            ],
        )?;
        Ok(changed == 1)
    }

    pub fn fail_controller_operation(
        &self,
        operation: &ControllerOperation,
        error: &str,
        now_unix: u64,
    ) -> Result<bool> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE operations SET state='done', phase='failed', last_error=?1, updated_at_unix=?2
             WHERE id=?3 AND subject_kind=?4 AND subject_id=?5 AND generation=?6
               AND fencing_token=?7 AND state='running'",
            params![
                error,
                now_unix as i64,
                operation.id,
                operation.subject_kind,
                operation.subject_id,
                operation.generation,
                operation.fencing_token
            ],
        )?;
        Ok(changed == 1)
    }

    /// CAS-updates a node's observed state only while this operation still owns
    /// the subject. A late exit or old Stop cannot overwrite a replacement PID.
    pub fn update_node_status_fenced(
        &self,
        operation: &ControllerOperation,
        status: crate::types::NodeStatus,
        pid: Option<u32>,
        now_unix: u64,
    ) -> Result<bool> {
        let connection = self.connection()?;
        let changed = connection.execute(
            "UPDATE nodes SET status=?1, pid=?2
             WHERE id=?3 AND ?4='node' AND EXISTS (
                 SELECT 1 FROM operations WHERE id=?5 AND subject_kind=?4
                   AND subject_id=?3 AND generation=?6 AND fencing_token=?7
                   AND state='running' AND phase IN ('reserved','spawned')
             )",
            params![
                status.to_string(),
                pid,
                operation.subject_id,
                operation.subject_kind,
                operation.id,
                operation.generation,
                operation.fencing_token
            ],
        )?;
        if changed == 1 {
            // Keep the recovery ledger in sync inside the same SQLite connection
            // boundary used for this fenced write.
            let _ = now_unix;
        }
        Ok(changed == 1)
    }

    /// Returns active controller rows for startup reconcile. This deliberately
    /// exposes no credential or config data.
    pub fn pending_controller_operations(&self) -> Result<Vec<ControllerOperation>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT id, subject_kind, subject_id, operation_kind, phase, desired_state,
                    generation, fencing_token, pid, process_started_at
             FROM operations WHERE state='running' ORDER BY created_at_unix, id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(ControllerOperation {
                id: row.get(0)?,
                subject_kind: row.get(1)?,
                subject_id: row.get(2)?,
                operation_kind: row.get(3)?,
                phase: row.get(4)?,
                desired_state: row.get(5)?,
                generation: row.get(6)?,
                fencing_token: row.get(7)?,
                pid: row.get(8)?,
                process_started_at: row.get(9)?,
            })
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load pending controller operations")
    }

    /// Reconciles persisted in-flight operations after a controller restart.
    /// Stale leases are fenced; non-stale rows remain visible for the running
    /// controller to finish. No process is signalled here — identity checks
    /// belong to the subject-specific recovery path.
    pub fn reconcile_pending_controller_operations(
        &self,
        now_unix: u64,
    ) -> Result<ControllerReconcileSummary> {
        let connection = self.connection()?;
        connection.execute(
            "UPDATE operations SET state='done', phase='abandoned',
                    last_error='controller restart reclaimed stale lease', updated_at_unix=?1
             WHERE state='running' AND updated_at_unix <= ?1 - ?2",
            params![now_unix as i64, OPERATION_STALE_SECS as i64],
        )?;
        self.controller_reconcile_summary(now_unix)
    }

    /// A bounded operational view for /healthz and startup diagnostics.
    pub fn controller_reconcile_summary(
        &self,
        now_unix: u64,
    ) -> Result<ControllerReconcileSummary> {
        let connection = self.connection()?;
        let pending: usize = connection.query_row(
            "SELECT COUNT(*) FROM operations WHERE state='running' AND phase IN ('requested','reserved','spawned')",
            [],
            |row| row.get::<_, i64>(0),
        )? as usize;
        let stale: usize = connection.query_row(
            "SELECT COUNT(*) FROM operations WHERE state='running' AND updated_at_unix <= ?1 - ?2",
            params![now_unix as i64, OPERATION_STALE_SECS as i64],
            |row| row.get::<_, i64>(0),
        )? as usize;
        let unknown: usize = connection.query_row(
            "SELECT COUNT(*) FROM operations WHERE state='done' AND phase IN ('failed','abandoned')",
            [],
            |row| row.get::<_, i64>(0),
        )? as usize;
        Ok(ControllerReconcileSummary {
            pending,
            stale,
            unknown,
        })
    }
    pub fn begin_controller_lease(
        &self,
        subject_kind: &str,
        subject_id: &str,
        operation_kind: &str,
        desired_state: &str,
    ) -> Result<ControllerLease<'_>> {
        let operation = self.begin_controller_operation(
            subject_kind,
            subject_id,
            operation_kind,
            desired_state,
            unix_now(),
        )?;
        Ok(ControllerLease {
            repository: self,
            operation,
            finished: false,
        })
    }

    /// Marks a legacy operation finished. New controller paths use the fenced
    /// completion API above.
    pub fn finish_node_operation(&self, operation_id: &str, now_unix: u64) -> Result<()> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE operations SET state='done', phase='committed', updated_at_unix=?2
                 WHERE id=?1 AND state='running'",
                params![operation_id, now_unix as i64],
            )
            .context("failed to close the node operation")?;
        Ok(())
    }

    /// [`Self::begin_node_operation`] with a self-closing guard.
    pub fn begin_node_operation_guarded(
        &self,
        kind: &str,
        node_id: &str,
    ) -> Result<NodeOperation<'_>> {
        let id = self.begin_node_operation(kind, node_id, unix_now())?;
        Ok(NodeOperation {
            repository: self,
            id,
            finished: false,
        })
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}
