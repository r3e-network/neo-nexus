//! Where the fleet's current verdict lives.
//!
//! One row per node, replaced in place, plus an append-only record of the
//! changes. Health is computed once, by the observation loop, and read by every
//! surface — a list row and a detail page that each derive it separately
//! eventually disagree, and an operator comparing them has no way to tell which
//! is wrong.

use rusqlite::Row;

use super::*;
use crate::observe::{HealthState, HealthTransition, NextStep, NodeHealth, StallScope};

/// How many transitions are kept per node.
///
/// Enough for a node detail page's timeline to cover a long incident without
/// the table growing without bound on a workspace that flaps.
pub(crate) const TRANSITIONS_KEPT_PER_NODE: usize = 100;

impl Repository {
    /// Write the current verdict for a node, replacing any previous one.
    pub fn save_node_health(&self, health: &NodeHealth) -> Result<()> {
        crate::types::validate_node_id(&health.node_id)?;
        let (next_label, next_href) = match &health.next {
            NextStep::Here { label, href } => (label.clone(), Some(href.clone())),
            // No href means no button: the label carries the whole sentence,
            // which is how "this needs a committee vote" stays actionable
            // advice rather than becoming a link that goes nowhere.
            NextStep::External { text } => (text.clone(), None),
        };
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO node_health_state (
                    node_id, state, since_unix, evaluated_at_unix, reason,
                    scope, cause, next_label, next_href
                 )
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(node_id) DO UPDATE SET
                    state = excluded.state,
                    since_unix = excluded.since_unix,
                    evaluated_at_unix = excluded.evaluated_at_unix,
                    reason = excluded.reason,
                    scope = excluded.scope,
                    cause = excluded.cause,
                    next_label = excluded.next_label,
                    next_href = excluded.next_href",
                params![
                    health.node_id,
                    health.state.persist_key(),
                    health.since_unix,
                    health.evaluated_at_unix,
                    health.reason,
                    health.scope.map(StallScope::persist_key),
                    health.cause,
                    next_label,
                    next_href,
                ],
            )
            .with_context(|| format!("failed to save health for node {}", health.node_id))?;
        Ok(())
    }

    pub fn load_node_health(&self, node_id: &str) -> Result<Option<NodeHealth>> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        connection
            .query_row(
                "SELECT node_id, state, since_unix, evaluated_at_unix, reason,
                        scope, cause, next_label, next_href
                 FROM node_health_state
                 WHERE node_id = ?1",
                params![node_id],
                node_health_from_row,
            )
            .optional()
            .context("failed to load node health")?
            .transpose()
    }

    /// Every node's current verdict, worst first.
    ///
    /// Ordered by the state enum's own discriminant, which is the guard chain's
    /// precedence — so "what should I look at" is answered by reading from the
    /// top rather than by a sort the caller has to get right. Within a state,
    /// the one that has been there longest comes first.
    pub fn list_node_health(&self) -> Result<Vec<NodeHealth>> {
        let connection = self.connection()?;
        let mut statement = connection.prepare(
            "SELECT node_id, state, since_unix, evaluated_at_unix, reason,
                    scope, cause, next_label, next_href
             FROM node_health_state",
        )?;
        let rows = statement.query_map([], node_health_from_row)?;
        let mut health = Vec::new();
        for row in rows {
            health.push(row??);
        }
        health.sort_by(|left, right| {
            left.state
                .cmp(&right.state)
                .then(left.since_unix.cmp(&right.since_unix))
                .then(left.node_id.cmp(&right.node_id))
        });
        Ok(health)
    }

    pub fn record_health_transition(&self, transition: &HealthTransition) -> Result<()> {
        crate::types::validate_node_id(&transition.node_id)?;
        let connection = self.connection()?;
        connection
            .execute(
                "INSERT INTO node_health_transitions (node_id, at_unix, from_state, to_state, reason)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    transition.node_id,
                    transition.at_unix,
                    transition.from.map(HealthState::persist_key),
                    transition.to.persist_key(),
                    transition.reason,
                ],
            )
            .with_context(|| {
                format!(
                    "failed to record a health transition for node {}",
                    transition.node_id
                )
            })?;
        Ok(())
    }

    /// A node's recent changes of state, newest first.
    pub fn recent_health_transitions(
        &self,
        node_id: &str,
        limit: usize,
    ) -> Result<Vec<HealthTransition>> {
        crate::types::validate_node_id(node_id)?;
        let connection = self.connection()?;
        let limit = limit.clamp(1, TRANSITIONS_KEPT_PER_NODE) as i64;
        let mut statement = connection.prepare(
            "SELECT node_id, at_unix, from_state, to_state, reason
             FROM node_health_transitions
             WHERE node_id = ?1
             ORDER BY at_unix DESC, id DESC
             LIMIT ?2",
        )?;
        let rows = statement.query_map(params![node_id, limit], health_transition_from_row)?;
        let mut transitions = Vec::new();
        for row in rows {
            transitions.push(row??);
        }
        Ok(transitions)
    }

    pub fn prune_health_transitions_keep_recent_per_node(
        &self,
        keep_recent: usize,
    ) -> Result<usize> {
        let mut connection = self.connection()?;
        let node_ids = {
            let mut statement =
                connection.prepare("SELECT DISTINCT node_id FROM node_health_transitions")?;
            let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .context("failed to load node ids with health transitions")?
        };

        let transaction = connection.transaction()?;
        let mut deleted = 0;
        for node_id in node_ids {
            deleted += transaction.execute(
                "DELETE FROM node_health_transitions
                 WHERE node_id = ?1
                   AND id NOT IN (
                       SELECT id FROM node_health_transitions
                       WHERE node_id = ?1
                       ORDER BY at_unix DESC, id DESC
                       LIMIT ?2
                   )",
                params![node_id, keep_recent as i64],
            )?;
        }
        transaction.commit()?;
        Ok(deleted)
    }
}

/// A stored state whose string does not name a state this build knows.
///
/// Refused rather than defaulted. A workspace written by a newer build could
/// carry a state this one has never heard of, and mapping it to `Healthy` would
/// turn an unrecognised incident into a pass — while mapping it to `Unknown`
/// would silently discard a real verdict. Neither is something to do quietly.
fn state_from_key(key: &str) -> Result<HealthState> {
    HealthState::from_persist_key(key)
        .ok_or_else(|| anyhow::anyhow!("stored health state {key:?} is not one this build knows"))
}

fn node_health_from_row(row: &Row<'_>) -> rusqlite::Result<Result<NodeHealth>> {
    let state: String = row.get(1)?;
    let scope: Option<String> = row.get(5)?;
    let next_label: String = row.get(7)?;
    let next_href: Option<String> = row.get(8)?;
    let node_id: String = row.get(0)?;
    let since_unix: u64 = row.get(2)?;
    let evaluated_at_unix: u64 = row.get(3)?;
    let reason: String = row.get(4)?;
    let cause: Option<String> = row.get(6)?;
    Ok(state_from_key(&state).map(|state| NodeHealth {
        node_id,
        state,
        since_unix,
        evaluated_at_unix,
        reason,
        // An unreadable scope is dropped rather than refused: it qualifies a
        // stall, and losing the qualifier is better than losing the stall.
        scope: scope.as_deref().and_then(StallScope::from_persist_key),
        cause,
        next: match next_href {
            Some(href) => NextStep::here(next_label, href),
            None => NextStep::external(next_label),
        },
    }))
}

fn health_transition_from_row(row: &Row<'_>) -> rusqlite::Result<Result<HealthTransition>> {
    let node_id: String = row.get(0)?;
    let at_unix: u64 = row.get(1)?;
    let from: Option<String> = row.get(2)?;
    let to: String = row.get(3)?;
    let reason: String = row.get(4)?;
    Ok(state_from_key(&to).and_then(|to| {
        Ok(HealthTransition {
            node_id,
            at_unix,
            from: from.as_deref().map(state_from_key).transpose()?,
            to,
            reason,
        })
    }))
}

#[cfg(test)]
#[path = "../../../tests/unit/repository/node_health/tests.rs"]
mod tests;
