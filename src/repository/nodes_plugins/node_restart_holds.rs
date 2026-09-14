//! Which nodes the watchdog has been told to leave alone.
//!
//! There is one workspace watchdog policy, read for every node. So stopping the
//! watchdog from relaunching the node you are in the middle of editing meant
//! turning automatic restart off **for the whole fleet** — and remembering to
//! turn it back on, on a page the node you were working on does not link to.
//!
//! An absent row means "follow the workspace policy". A hold is deliberately
//! per node and explicit: a fleet-wide switch and a per-node exemption are
//! different decisions, and conflating them is how a crash loop on one node
//! leaves every other node unsupervised.

use anyhow::{Context, Result};

use super::*;

impl Repository {
    /// Stop the watchdog restarting this node until the hold is lifted.
    pub fn hold_node_restarts(&self, node_id: &str, reason: &str, at_unix: u64) -> Result<()> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO node_restart_holds (node_id, held_at_unix, reason)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(node_id) DO UPDATE SET
                    held_at_unix = excluded.held_at_unix,
                    reason = excluded.reason",
                rusqlite::params![node_id, at_unix, reason],
            )
            .with_context(|| format!("failed to hold restarts for node {node_id}"))?;
        Ok(())
    }

    /// Let the watchdog manage this node again.
    pub fn release_node_restarts(&self, node_id: &str) -> Result<()> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        connection
            .execute(
                "DELETE FROM node_restart_holds WHERE node_id = ?1",
                rusqlite::params![node_id],
            )
            .with_context(|| format!("failed to release restarts for node {node_id}"))?;
        Ok(())
    }

    /// When this node's hold was placed, and why, if it is held.
    pub fn node_restart_hold(&self, node_id: &str) -> Result<Option<(u64, String)>> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT held_at_unix, reason FROM node_restart_holds WHERE node_id = ?1",
                rusqlite::params![node_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .context("failed to read the restart hold")
    }

    /// Every held node, so the supervision tick can ask once per pass rather
    /// than once per node.
    pub fn held_node_ids(&self) -> Result<std::collections::BTreeSet<String>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare("SELECT node_id FROM node_restart_holds")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.collect::<rusqlite::Result<std::collections::BTreeSet<_>>>()
            .context("failed to read restart holds")
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/repository/node_restart_holds/tests.rs"]
mod tests;
