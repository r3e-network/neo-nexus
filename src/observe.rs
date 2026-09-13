//! What the chain says when asked, and what that means.
//!
//! This module exists because the workspace previously had no observation
//! layer at all. A node's health was the answer to "how many of two JSON-RPC
//! calls replied" — `getversion` and `getblockcount`, or `web3_clientVersion`
//! and `eth_blockNumber`. Nothing compared one height to the next, compared a
//! height to anything else, timed a request, counted peers, or evaluated a
//! threshold. The consequence is the failure mode every node operator knows:
//! **a node that answers RPC promptly, at a height that has not moved for an
//! hour, was indistinguishable from a healthy one.**
//!
//! Three ideas carry the design.
//!
//! **Absence has a type.** [`Observation`] is `Known` with its [`Evidence`], or
//! `Unknown` with the reason, or `Unanswerable` because this client cannot be
//! asked. There is no variant that renders as a plausible default, because the
//! surface this replaces filled its gaps with exactly that.
//!
//! **Health is not process state.** [`HealthState`] has no `Default` and no
//! `From<bool>`, so `is_running()` cannot become `Healthy` by any path the
//! compiler will accept.
//!
//! **A verdict is not a dead end.** Every [`Verdict`] carries a [`NextStep`],
//! not an `Option<NextStep>`. Advice with nowhere to go is unrepresentable.
//!
//! `src/rpc_health/` stays, narrowed to what it was always good at: a one-shot
//! liveness probe of a bare endpoint, for the CLI and for federation. What it
//! stops being is the node's health.

mod classify;
mod client;
mod derive;
mod evidence;
mod health;
mod run;
mod sample;
mod schedule;

pub use classify::{classify, HealthInputs, HealthPolicy};
pub use derive::{derive, Derived, ReferenceHead};
pub use evidence::{Evidence, NotSampled, Observation};
pub use health::{Cause, HealthState, HealthTone, NextStep, StallScope, Verdict};
pub use run::{forget_missing, run_pass};
pub use sample::{head_method, NodeSample, SampleClass, SampleRound};
pub use schedule::{DueWork, ObservationPolicy, Scheduler};

/// How many rounds are retained per node.
///
/// Re-exported from the repository so the loop that writes rounds and the
/// reader that asks for them cannot disagree about how much history exists.
pub(crate) use crate::repository::SAMPLES_KEPT_PER_NODE;
