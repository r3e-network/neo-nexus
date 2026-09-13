//! What is asked of a node, and how often.
//!
//! Sampling is split into **classes** rather than run as one probe, for three
//! reasons that each cost the previous design something:
//!
//! - **Cadence differs by two orders of magnitude.** A node's height is worth
//!   reading every fifteen seconds; the protocol constants behind it change
//!   when the node restarts. One period for everything means either paying for
//!   `getversion` four times a minute or learning the block interval once an
//!   hour.
//! - **A class that fails does not void the round.** A round with the height
//!   read and the mempool unreadable is a good round. The old probe derived
//!   status from "how many of two calls answered", so one unimplemented method
//!   condemned the node.
//! - **Cost is attributable.** When an operator asks why their node is being
//!   polled, the answer is a table.
//!
//! `getversion` deserves particular mention: it is the highest-value call in
//! the product and its answer was being thrown away. `summarize_version` kept
//! only `useragent`, discarding the `protocol` block that carries the block
//! interval, the mempool capacity, the validator count and — the one that
//! matters most — **the network magic the node actually joined**, which is how
//! a node configured "private" but dialling MainNet seeds becomes visible.

mod collect;
pub(crate) mod neo_n3;
pub(crate) mod neox;
mod round;

pub(crate) use collect::sample_node;
pub use round::{NodeSample, SampleRound};

use std::time::Duration;

/// One kind of question, with its own period and failure semantics.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SampleClass {
    /// How far the node has synced. The heartbeat of the whole layer.
    Head,
    /// When the newest block was produced — the witness that works with no
    /// reference node at all.
    HeadTime,
    /// How many peers are connected.
    Peers,
    /// Protocol constants and the magic actually joined.
    Identity,
    /// Transaction pool depth against the chain's configured capacity.
    Pool,
}

impl SampleClass {
    pub const ALL: [Self; 5] = [
        Self::Head,
        Self::HeadTime,
        Self::Peers,
        Self::Identity,
        Self::Pool,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Self::Head => "head",
            Self::HeadTime => "head_time",
            Self::Peers => "peers",
            Self::Identity => "identity",
            Self::Pool => "pool",
        }
    }

    /// How often this class is worth asking, before policy overrides.
    ///
    /// `Identity` is fifteen minutes because protocol constants change only
    /// when a node restarts onto a different config, and `Pool` is two minutes
    /// because mempool depth is a trend, not a tripwire.
    pub fn default_period(self) -> Duration {
        match self {
            Self::Head | Self::Peers => Duration::from_secs(15),
            Self::HeadTime => Duration::from_secs(60),
            Self::Pool => Duration::from_secs(120),
            Self::Identity => Duration::from_secs(900),
        }
    }

    /// Whether a failure here means the node is not answering at all.
    ///
    /// Only `Head` does. Every other class failing is a gap in what is known
    /// about a node that is otherwise responding, and must not be allowed to
    /// read as an outage.
    pub fn is_liveness(self) -> bool {
        matches!(self, Self::Head)
    }
}

#[cfg(test)]
#[path = "../../tests/unit/observe/sample_tests.rs"]
mod tests;
