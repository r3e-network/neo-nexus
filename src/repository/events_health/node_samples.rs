//! Storing what the chain said.
//!
//! Every column is nullable, and a null means **not read** rather than zero.
//! A round trip through SQLite must preserve that, so rows come back as
//! [`Observation`]s rather than as `Option`s that some later caller would
//! `unwrap_or(0)` — otherwise the type that makes fabrication unrepresentable
//! in memory would be undone by saving and reloading it.

use rusqlite::Row;

use super::*;
use crate::observe::{Evidence, NodeSample, NotSampled, Observation};

/// How many rounds are kept per node.
///
/// Enough to derive over a useful window — at the default fifteen-second head
/// period this is roughly fifty minutes — without the table becoming a
/// liability on a workspace nobody prunes. Rollups will carry longer history;
/// this is the raw tail that the derivations walk.
pub(crate) const SAMPLES_KEPT_PER_NODE: usize = 200;

impl Repository {
    /// Record one round.
    pub fn record_node_sample(&self, sample: &NodeSample) -> Result<()> {
        crate::types::validate_node_id(&sample.node_id)?;
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO node_samples (
                    node_id, sampled_at_unix, endpoint, head_ok, head_latency_ms,
                    block_height, header_height, syncing, head_block_time_unix,
                    peers_connected, observed_magic, ms_per_block, mempool_capacity,
                    mempool_verified, mempool_unverified, validators_count, client_version
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
                params![
                    sample.node_id,
                    sample.sampled_at_unix,
                    sample.endpoint,
                    i64::from(sample.head_ok),
                    sample.head_latency_ms,
                    sample.block_height.value().copied(),
                    sample.header_height.value().copied(),
                    sample.syncing.value().copied().map(i64::from),
                    sample.head_block_time_unix.value().copied(),
                    sample.peers_connected.value().copied(),
                    sample.observed_magic.value().copied(),
                    sample.ms_per_block.value().copied(),
                    sample.mempool_capacity.value().copied(),
                    sample.mempool_verified.value().copied(),
                    sample.mempool_unverified.value().copied(),
                    sample.validators_count.value().copied(),
                    sample.client_version.value().cloned(),
                ],
            )
            .with_context(|| format!("failed to record a sample for node {}", sample.node_id))?;
        Ok(())
    }

    /// The most recent rounds for one node, newest first.
    ///
    /// Newest-first is the order the derivations walk: they look back from the
    /// current head until they have the window they need, so they can stop
    /// early instead of loading a whole history in order to reverse it.
    pub fn recent_node_samples(&self, node_id: &str, limit: usize) -> Result<Vec<NodeSample>> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        let limit = limit.clamp(1, SAMPLES_KEPT_PER_NODE) as i64;
        let mut statement = connection.prepare(
            "SELECT node_id, sampled_at_unix, endpoint, head_ok, head_latency_ms,
                    block_height, header_height, syncing, head_block_time_unix,
                    peers_connected, observed_magic, ms_per_block, mempool_capacity,
                    mempool_verified, mempool_unverified, validators_count, client_version
             FROM node_samples
             WHERE node_id = ?1
             ORDER BY sampled_at_unix DESC, id DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![node_id, limit], node_sample_from_row)?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
            .context("failed to load node samples")
    }

    /// The newest round for one node, if any has been taken.
    pub fn latest_node_sample(&self, node_id: &str) -> Result<Option<NodeSample>> {
        Ok(self.recent_node_samples(node_id, 1)?.into_iter().next())
    }

    /// Drop all but the newest rounds for every node.
    ///
    /// Per node rather than globally: a global cap would let one chatty node's
    /// history evict a quiet node's entirely, and the quiet node is precisely
    /// the one whose last known height an operator will want after an outage.
    pub fn prune_node_samples_keep_recent_per_node(&self, keep_recent: usize) -> Result<usize> {
        let mut connection = self.connection()?;
        let node_ids = {
            let mut statement = connection.prepare("SELECT DISTINCT node_id FROM node_samples")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .context("failed to load node ids with samples")?
        };

        let transaction = connection.transaction()?;
        let mut deleted = 0;
        for node_id in node_ids {
            deleted += transaction.execute(
                "DELETE FROM node_samples
                 WHERE node_id = ?1
                   AND id NOT IN (
                       SELECT id FROM node_samples
                       WHERE node_id = ?1
                       ORDER BY sampled_at_unix DESC, id DESC
                       LIMIT ?2
                   )",
                params![node_id, keep_recent as i64],
            )?;
        }
        transaction.commit()?;
        Ok(deleted)
    }
}

/// Rebuild a round, keeping "not read" distinct from zero.
fn node_sample_from_row(row: &Row<'_>) -> rusqlite::Result<NodeSample> {
    let reader = StoredSample {
        endpoint: row.get(2)?,
        sampled_at_unix: row.get(1)?,
        head_ok: row.get::<_, i64>(3)? != 0,
    };
    Ok(NodeSample {
        node_id: row.get(0)?,
        sampled_at_unix: reader.sampled_at_unix,
        endpoint: reader.endpoint.clone(),
        head_ok: reader.head_ok,
        head_latency_ms: row.get(4)?,
        block_height: reader.number(row, 5, "block_height")?,
        header_height: reader.number(row, 6, "header_height")?,
        syncing: reader
            .number::<u64>(row, 7, "syncing")?
            .map(|flag| flag != 0),
        head_block_time_unix: reader.number(row, 8, "head_block_time_unix")?,
        peers_connected: reader.number(row, 9, "peers_connected")?,
        observed_magic: reader.number(row, 10, "observed_magic")?,
        ms_per_block: reader.number(row, 11, "ms_per_block")?,
        mempool_capacity: reader.number(row, 12, "mempool_capacity")?,
        mempool_verified: reader.number(row, 13, "mempool_verified")?,
        mempool_unverified: reader.number(row, 14, "mempool_unverified")?,
        validators_count: reader.number(row, 15, "validators_count")?,
        client_version: match row.get::<_, Option<String>>(16)? {
            Some(version) => {
                Observation::Known(version.clone(), reader.evidence("client_version", version))
            }
            None => Observation::Unknown(reader.absence()),
        },
    })
}

/// The parts of a stored row that every column's provenance shares.
struct StoredSample {
    endpoint: String,
    sampled_at_unix: u64,
    head_ok: bool,
}

impl StoredSample {
    /// Provenance for a value read back out of storage.
    ///
    /// The method is `node_samples` rather than the RPC call that originally
    /// produced the value: which call it was depends on the node's family, and
    /// claiming `getblockcount` for a row that may have come from
    /// `eth_blockNumber` would be a guess dressed as evidence. What the row can
    /// state truthfully is the endpoint it came from and when — which is what
    /// an operator needs in order to know whether they are looking at something
    /// current.
    fn evidence(&self, field: &'static str, value: impl Into<String>) -> Evidence {
        Evidence::recorded(
            "node_samples",
            field,
            value,
            self.endpoint.clone(),
            self.sampled_at_unix,
        )
    }

    /// Why a null column is null, read off the row rather than guessed.
    ///
    /// No reason is stored per column — that would be a sentence per null on
    /// every row — but the row still distinguishes the three cases an operator
    /// reads differently, because the round itself recorded enough:
    ///
    /// * an empty endpoint is a node with no RPC port, which is the one shape
    ///   [`NodeSample::not_observable`] writes and the only way a row can have
    ///   no endpoint at all;
    /// * a recorded endpoint with `head_ok = 0` is a round that reached for the
    ///   node and got nothing;
    /// * `head_ok = 1` with a null column is a round that did reach the node,
    ///   so that column is absent because its class was not due this time.
    ///
    /// This matters after a restart, when no live round exists yet and the
    /// stored row is all an operator has. Collapsing the three into one blank
    /// would make a node that has no RPC port look identical to one that is
    /// down.
    fn absence(&self) -> NotSampled {
        match (self.endpoint.is_empty(), self.head_ok) {
            (true, _) => NotSampled::SamplingDisabled,
            (false, false) => NotSampled::NodeDidNotAnswer,
            (false, true) => NotSampled::NotRecorded,
        }
    }

    fn number<T>(
        &self,
        row: &Row<'_>,
        index: usize,
        field: &'static str,
    ) -> rusqlite::Result<Observation<T>>
    where
        T: rusqlite::types::FromSql + ToString + Copy,
    {
        Ok(match row.get::<_, Option<T>>(index)? {
            Some(value) => Observation::Known(value, self.evidence(field, value.to_string())),
            None => Observation::Unknown(self.absence()),
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/repository/node_samples/tests.rs"]
mod tests;
