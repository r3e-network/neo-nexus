//! A verdict, in the form that survives a restart.
//!
//! [`Verdict`] is what the guard chain produces and it carries its working —
//! the [`Evidence`](super::Evidence) behind every number. That working belongs
//! to the round that produced it and is not stored: a sentence per column per
//! evaluation would cost more than it is worth, and the round it came from is
//! one table away.
//!
//! What is stored is the judgement and the one thing a verdict cannot
//! recompute from a single round — **when this state was entered**. The
//! difference matters at three in the morning: "stalled, checked 15s ago" is a
//! freshness report, and "stalled for 12 minutes" is the number that decides
//! whether to act.

use super::health::{HealthState, NextStep, StallScope, Verdict};

/// The current judgement on one node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeHealth {
    pub node_id: String,
    pub state: HealthState,
    /// When this state was entered.
    pub since_unix: u64,
    /// When it was last confirmed.
    pub evaluated_at_unix: u64,
    pub reason: String,
    pub scope: Option<StallScope>,
    /// What the same round's evidence suggested was behind it, already in
    /// sentence form. The structured
    /// [`Cause`](super::Cause) stays with the live verdict; what an operator
    /// reads is this.
    pub cause: Option<String>,
    pub next: NextStep,
}

impl NodeHealth {
    /// Record a verdict as entered at `since_unix` and confirmed now.
    pub fn from_verdict(
        node_id: impl Into<String>,
        verdict: &Verdict,
        since_unix: u64,
        evaluated_at_unix: u64,
    ) -> Self {
        Self {
            node_id: node_id.into(),
            state: verdict.state(),
            since_unix,
            evaluated_at_unix,
            reason: verdict.reason().to_string(),
            scope: verdict.scope(),
            cause: verdict.suspected_cause().map(super::Cause::describe),
            next: verdict.next_action().clone(),
        }
    }

    /// How long this state has held, in seconds.
    pub fn held_for_seconds(&self, now_unix: u64) -> u64 {
        now_unix.saturating_sub(self.since_unix)
    }

    /// How old the judgement itself is.
    ///
    /// A surface has to distinguish this from [`Self::held_for_seconds`]: a
    /// node stalled for an hour whose last evaluation was two minutes ago is
    /// telling an operator two different things, and only one of them is about
    /// the node.
    pub fn evaluated_seconds_ago(&self, now_unix: u64) -> u64 {
        now_unix.saturating_sub(self.evaluated_at_unix)
    }
}

/// One change of state, kept so "it was fine an hour ago" is a question the
/// workspace can answer rather than one reconstructed from log files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HealthTransition {
    pub node_id: String,
    pub at_unix: u64,
    /// `None` for the first verdict ever recorded on a node. Rendering that as
    /// a transition from `Stopped` would put an event in the timeline that
    /// never happened.
    pub from: Option<HealthState>,
    pub to: HealthState,
    pub reason: String,
}

impl HealthTransition {
    /// One line for a timeline.
    pub fn summary(&self) -> String {
        match self.from {
            Some(from) => format!("{} → {}: {}", from.label(), self.to.label(), self.reason),
            None => format!("{}: {}", self.to.label(), self.reason),
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/observe/record_tests.rs"]
mod tests;
