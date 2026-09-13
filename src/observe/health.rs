//! What state a node is in on the chain it joined, and what to do about it.
//!
//! This is the second of the three axes the console must never fuse. The
//! supervisor knows whether a **process** is running; this knows what the
//! **chain** says when asked; a third knows what the chain has **designated**
//! the node's key to do. `Running` + `Stalled` + `Elected` is a legal state,
//! it is expensive, and it is the failure this product exists to catch — so a
//! single badge cannot express it and nothing here accepts a `NodeStatus`.
//!
//! [`HealthState`] deliberately implements neither `Default` nor `From<bool>`.
//! Both were routes by which "the process is up" became "the node is healthy",
//! which is the error that let a node answering RPC promptly, at a height that
//! had not moved in an hour, read as two passing checks.

use std::fmt;

use super::evidence::Evidence;

/// Where a node stands on its chain.
///
/// The order of the variants is the order the guard chain evaluates them, and
/// that ordering is part of the definition. `Isolated` outranks `Stalled`
/// because zero peers is the *cause* of the stall and the operator needs the
/// cause. `Stalled` outranks `Syncing` because a node that is behind and not
/// moving is not syncing, however much it would like to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HealthState {
    /// No process, and the operator has not asked for one.
    Stopped,
    /// Spawned, inside its client's grace period, no successful sample yet.
    Starting,
    /// Expected to be running and not answering.
    Unreachable,
    /// Not enough is known to say. Never rendered as a pass.
    Unknown,
    /// Answering, with no peers. A node alone on the network cannot progress.
    Isolated,
    /// Answering promptly, and its height is not moving.
    Stalled,
    /// Behind the chain and catching up.
    Syncing,
    /// Working, with something worth knowing about.
    Degraded,
    /// Nothing above applies.
    Healthy,
}

impl HealthState {
    pub const ALL: [Self; 9] = [
        Self::Stopped,
        Self::Starting,
        Self::Unreachable,
        Self::Unknown,
        Self::Isolated,
        Self::Stalled,
        Self::Syncing,
        Self::Degraded,
        Self::Healthy,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Stopped => "Stopped",
            Self::Starting => "Starting",
            Self::Unreachable => "Unreachable",
            Self::Unknown => "Not checked",
            Self::Isolated => "Isolated",
            Self::Stalled => "Stalled",
            Self::Syncing => "Syncing",
            Self::Degraded => "Degraded",
            Self::Healthy => "Healthy",
        }
    }

    /// The stable string this state is stored and filtered by.
    pub fn persist_key(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Unreachable => "unreachable",
            Self::Unknown => "unknown",
            Self::Isolated => "isolated",
            Self::Stalled => "stalled",
            Self::Syncing => "syncing",
            Self::Degraded => "degraded",
            Self::Healthy => "healthy",
        }
    }

    pub fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|state| state.persist_key() == key)
    }

    /// Whether this state should pull an operator's attention.
    ///
    /// `Unknown` does not: not having looked is not the same as having found
    /// something, and a fleet that has just started would otherwise page on
    /// every node at once.
    pub fn needs_attention(self) -> bool {
        matches!(
            self,
            Self::Unreachable | Self::Isolated | Self::Stalled | Self::Degraded
        )
    }

    /// How this state is coloured. `Unknown` is grey, never green — the whole
    /// point of the type is that "we have not looked" cannot read as a pass.
    pub fn tone(self) -> HealthTone {
        match self {
            Self::Healthy => HealthTone::Good,
            Self::Syncing | Self::Starting => HealthTone::Working,
            Self::Degraded | Self::Isolated => HealthTone::Warning,
            Self::Unreachable | Self::Stalled => HealthTone::Bad,
            Self::Stopped | Self::Unknown => HealthTone::Neutral,
        }
    }
}

impl fmt::Display for HealthState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.label())
    }
}

/// The visual weight of a state. `Neutral` is the colour of "we do not know".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthTone {
    Good,
    Working,
    Warning,
    Bad,
    Neutral,
}

/// Whether a stall is this node's problem or the whole chain's.
///
/// The distinction decides who is paged. One node stuck while its peers
/// advance is an operator's problem; a chain that has stopped producing is
/// everybody's, and restarting one node will not fix it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StallScope {
    Node,
    Chain,
}

impl StallScope {
    pub const ALL: [Self; 2] = [Self::Node, Self::Chain];

    pub fn label(self) -> &'static str {
        match self {
            Self::Node => "this node",
            Self::Chain => "the whole chain",
        }
    }

    /// The stable string this scope is stored by.
    pub fn persist_key(self) -> &'static str {
        match self {
            Self::Node => "node",
            Self::Chain => "chain",
        }
    }

    pub fn from_persist_key(key: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|scope| scope.persist_key() == key)
    }
}

/// What the same round of evidence suggests is behind a stall.
///
/// Attributed from measurements already in hand. Telling an operator to "check
/// disk space and peers" when the disk measurement is one column away in the
/// same row is a design failure, not a limitation.
#[derive(Clone, Debug, PartialEq)]
pub enum Cause {
    DiskNearFull { free_percent: f64, free_bytes: u64 },
    NoPeers,
    PeersBelowExpected { connected: u32, expected: u32 },
    ClockSuspect,
}

impl Cause {
    pub fn describe(&self) -> String {
        match self {
            Self::DiskNearFull {
                free_percent,
                free_bytes,
            } => format!(
                "Disk is {:.1}% full ({} free).",
                100.0 - free_percent,
                crate::core::operations::format_bytes(*free_bytes)
            ),
            Self::NoPeers => "The node has no connected peers.".to_string(),
            Self::PeersBelowExpected {
                connected,
                expected,
            } => format!("The node has {connected} peers; {expected} were expected."),
            Self::ClockSuspect => {
                "The node's newest block is dated in the future, so this host's clock or the node's is wrong."
                    .to_string()
            }
        }
    }
}

/// Where an operator goes next.
///
/// Not an `Option`. A verdict with no known remediation cannot be built,
/// therefore cannot be ranked into an attention queue, therefore cannot reach
/// an operator at 03:00 as a dead end. Where this console genuinely cannot fix
/// something — a committee-witnessed designation, say — `External` says so and
/// says what can.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NextStep {
    Here { label: String, href: String },
    External { text: String },
}

impl NextStep {
    pub fn here(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self::Here {
            label: label.into(),
            href: href.into(),
        }
    }

    pub fn external(text: impl Into<String>) -> Self {
        Self::External { text: text.into() }
    }
}

/// A state, the numbers that produced it, and one action.
///
/// Built only by [`super::classify`]. `evidence` is non-empty for every state
/// except those that are *about* absence — a verdict that cannot show its
/// working is indistinguishable from the literals this layer replaced.
#[derive(Clone, Debug, PartialEq)]
pub struct Verdict {
    state: HealthState,
    scope: Option<StallScope>,
    reason: String,
    evidence: Vec<Evidence>,
    suspected_cause: Option<Cause>,
    next_action: NextStep,
}

impl Verdict {
    pub(crate) fn new(
        state: HealthState,
        reason: impl Into<String>,
        evidence: Vec<Evidence>,
        next_action: NextStep,
    ) -> Self {
        Self {
            state,
            scope: None,
            reason: reason.into(),
            evidence,
            suspected_cause: None,
            next_action,
        }
    }

    #[must_use]
    pub(crate) fn with_scope(mut self, scope: StallScope) -> Self {
        self.scope = Some(scope);
        self
    }

    #[must_use]
    pub(crate) fn with_cause(mut self, cause: Option<Cause>) -> Self {
        self.suspected_cause = cause;
        self
    }

    pub fn state(&self) -> HealthState {
        self.state
    }

    pub fn scope(&self) -> Option<StallScope> {
        self.scope
    }

    /// One sentence, containing the numbers that decided it.
    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }

    pub fn suspected_cause(&self) -> Option<&Cause> {
        self.suspected_cause.as_ref()
    }

    pub fn next_action(&self) -> &NextStep {
        &self.next_action
    }

    /// Everything an operator needs in one line: what, why, and what next.
    pub fn summary(&self) -> String {
        let cause = self
            .suspected_cause
            .as_ref()
            .map(|cause| format!(" {}", cause.describe()))
            .unwrap_or_default();
        let next = match &self.next_action {
            NextStep::Here { label, .. } => format!(" → {label}"),
            NextStep::External { text } => format!(" → {text}"),
        };
        format!("{}: {}{cause}{next}", self.state.label(), self.reason)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/observe/health_tests.rs"]
mod tests;
