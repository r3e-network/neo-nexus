use super::*;
use crate::chain_progress::ProgressMarker;

impl Repository {
    pub(crate) fn load_chain_progress_marker(
        &self,
        node_id: &str,
    ) -> Result<Option<ProgressMarker>> {
        self.connection()?.query_row(
            "SELECT observed_pid,identity,last_observation_id,last_checked_at_unix,stalled_block_count
             FROM chain_progress_markers WHERE node_id=?1", params![node_id], |row| {
                Ok(ProgressMarker { observed_pid:row.get(0)?, identity:row.get(1)?,
                    last_observation_id:row.get(2)?, last_checked_at_unix:row.get(3)?, stalled_block_count:row.get(4)? })
            }).optional().context("failed to read chain progress marker")
    }

    pub(crate) fn commit_chain_progress(
        &self,
        node: &NodeConfig,
        previous: Option<&ProgressMarker>,
        next: &ProgressMarker,
        event: Option<&NewRuntimeEvent>,
        now_unix: u64,
    ) -> Result<bool> {
        let mut connection = self.connection()?;
        let transaction =
            connection.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        // Refuse a computed transition if another tick, a new observation, or a
        // node lifecycle/edit operation won the race while history was read.
        let position = transaction
            .query_row(
                "SELECT last_observation_id FROM chain_progress_markers WHERE node_id=?1",
                params![node.id],
                |row| row.get::<_, i64>(0),
            )
            .optional()?;
        if position != previous.map(|marker| marker.last_observation_id) {
            return Ok(false);
        }
        let current_node = transaction.query_row(
            "SELECT COUNT(*) FROM nodes WHERE id=?1 AND status='running' AND pid=?2 AND node_type=?3 AND network=?4 AND rpc_port=?5 AND runtime_version=?6",
            params![node.id, next.observed_pid, node.node_type.to_string(), node.network.to_string(), node.rpc_port, node.runtime_version], |row| row.get::<_, u64>(0))?;
        let latest_id = transaction.query_row(
            "SELECT id FROM rpc_health_checks WHERE node_id=?1 ORDER BY checked_at_unix DESC,id DESC LIMIT 1",
            params![node.id], |row| row.get::<_, i64>(0)).optional()?;
        if current_node != 1 || latest_id != Some(next.last_observation_id) {
            return Ok(false);
        }
        transaction.execute("INSERT INTO chain_progress_markers(node_id,observed_pid,identity,last_observation_id,last_checked_at_unix,stalled_block_count)
            VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(node_id) DO UPDATE SET observed_pid=excluded.observed_pid,
            identity=excluded.identity,last_observation_id=excluded.last_observation_id,
            last_checked_at_unix=excluded.last_checked_at_unix,stalled_block_count=excluded.stalled_block_count",
            params![node.id,next.observed_pid,next.identity,next.last_observation_id,next.last_checked_at_unix,next.stalled_block_count])?;
        if let Some(event) = event {
            transaction.execute("INSERT INTO runtime_events(occurred_at_unix,node_id,node_name,kind,severity,message)
                VALUES(?1,?2,?3,?4,?5,?6)", params![now_unix,event.node_id,event.node_name,event.kind.to_string(),event.severity.to_string(),event.message])?;
        }
        transaction.commit()?;
        Ok(true)
    }
}
