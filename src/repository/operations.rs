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

/// A claimed node operation that closes itself when the scope ends.
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
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE operations SET state='abandoned', updated_at_unix=?2
                 WHERE state='running' AND updated_at_unix <= ?2 - ?3",
                params![now_unix, now_unix, OPERATION_STALE_SECS as i64],
            )
            .context("failed to retire stale node operations")?;
        let id = format!("op-{}-{}", kind, Uuid::new_v4().simple());
        let claimed = connection
            .execute(
                "INSERT INTO operations (id, kind, node_id, state, created_at_unix, updated_at_unix)
                 SELECT ?1, ?2, ?3, 'running', ?4, ?4
                 WHERE NOT EXISTS (
                     SELECT 1 FROM operations WHERE node_id = ?3 AND state = 'running'
                 )",
                params![id, kind, node_id, now_unix as i64],
            )
            .context("failed to record the node operation")?;
        if claimed == 0 {
            anyhow::bail!(
                "node {node_id} already has a running operation; wait for it to finish or go stale"
            );
        }
        Ok(id)
    }

    /// Marks a claimed operation finished. Marking an already-abandoned claim
    /// is not an error: the caller's work is done either way.
    pub fn finish_node_operation(&self, operation_id: &str, now_unix: u64) -> Result<()> {
        let connection = self.connection()?;
        connection
            .execute(
                "UPDATE operations SET state='done', updated_at_unix=?2
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
