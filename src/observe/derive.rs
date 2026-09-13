//! The formulas, kept pure.
//!
//! No I/O, no clock, no database — `now_unix` is an argument. That is what
//! lets the state machine above be tested with state vectors rather than with
//! log lines copied out of the parser they are meant to check.
//!
//! Every function returns `Option`, and a `None` propagates rather than
//! becoming a zero. The distinction is not pedantic: a head lag of `None`
//! means "there is nothing to compare against", and a head lag of `0` means
//! "this node is exactly at the chain head". Rendering the first as the second
//! tells an operator their single-node private chain is perfectly in sync with
//! a network that does not exist.

use super::sample::NodeSample;

/// Where the chain's head is believed to be, and how confident that belief is.
///
/// Resolving this properly needs the fleet grouped by the chain each node
/// actually joined; until that lands, a workspace reports [`Self::SelfOnly`]
/// and lag is simply not computed. That is deliberately usable: a stall is
/// still detected, because the newest block's own timestamp ages whether or
/// not there is anything to compare heights against.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReferenceHead {
    Known { height: u64, source: String },
    SelfOnly,
}

/// What the samples add up to.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Derived {
    /// Seconds since this node's height last increased. The witness that needs
    /// no reference and no trust in the local clock.
    pub height_unchanged_seconds: Option<u64>,
    /// Seconds between now and the newest block's own timestamp. The witness
    /// that is immediate — a node that starts and syncs to a three-day-old head
    /// is stale on its very first sample — but that depends on the local clock.
    pub chain_lag_seconds: Option<i64>,
    /// Blocks behind the reference head.
    pub head_lag: Option<u64>,
    /// Blocks this node is behind its own header chain. On Neo N3 headers run
    /// ahead of blocks while syncing, so this is a sync signal available with
    /// nothing else to compare against.
    pub header_gap: Option<u64>,
    pub blocks_per_minute: Option<f64>,
    /// The node's own reported block interval, in seconds, which every
    /// threshold derives from instead of a compile-time constant.
    pub block_interval_seconds: Option<u64>,
    pub mempool_utilisation: Option<f64>,
    /// The newest block is dated in the future, so a clock somewhere is wrong
    /// and `chain_lag_seconds` must not be trusted.
    pub clock_suspect: bool,
    /// The height went backwards. A deep reorg, a restored archive or a
    /// swapped data directory — a real incident, and never a negative rate.
    pub height_regressed: Option<(u64, u64)>,
}

/// How far into the future a block may be dated before the clock is suspect.
///
/// Block timestamps are set by the producing node, so a little skew across a
/// network is normal and not worth flagging.
const CLOCK_SKEW_TOLERANCE_SECONDS: i64 = 30;

/// Compute everything derivable from a node's samples.
///
/// `history` is newest-first and includes `latest` at index 0.
pub fn derive(history: &[NodeSample], reference: &ReferenceHead, now_unix: u64) -> Derived {
    let Some(latest) = history.first() else {
        return Derived::default();
    };
    let mut derived = Derived {
        block_interval_seconds: latest.ms_per_block.value().map(|ms| (ms / 1000).max(1)),
        ..Derived::default()
    };

    let height = latest.block_height.value().copied();

    // Headers ahead of blocks means the node knows about work it has not done.
    if let (Some(headers), Some(blocks)) = (latest.header_height.value(), height.as_ref()) {
        derived.header_gap = Some(headers.saturating_sub(*blocks));
    }

    if let (Some(height), ReferenceHead::Known { height: head, .. }) = (height, reference) {
        // Within a block or two of the reference is propagation jitter, not lag.
        derived.head_lag = Some(head.saturating_sub(height));
    }

    if let Some(block_time) = latest.head_block_time_unix.value().copied() {
        let lag = now_unix as i64 - block_time as i64;
        if lag < -CLOCK_SKEW_TOLERANCE_SECONDS {
            // A block from the future. Whatever is wrong, chain lag computed
            // from it would be a negative number presented as freshness.
            derived.clock_suspect = true;
        } else {
            derived.chain_lag_seconds = Some(lag.max(0));
        }
    }

    derived.height_unchanged_seconds = height_unchanged_seconds(history, now_unix);
    derived.height_regressed = height_regression(history);
    derived.blocks_per_minute = blocks_per_minute(history);
    derived.mempool_utilisation = mempool_utilisation(latest);
    derived
}

/// Seconds since the height last increased.
///
/// Walks back to the newest sample whose height is lower than the current one.
/// `None` when every sample in the window shares a height — the node may have
/// been stalled for longer than the window, which is not the same as "it just
/// changed", and the caller has to know the difference.
fn height_unchanged_seconds(history: &[NodeSample], now_unix: u64) -> Option<u64> {
    let current = history.first()?.block_height.value().copied()?;
    for sample in history.iter().skip(1) {
        let Some(height) = sample.block_height.value().copied() else {
            continue;
        };
        if height < current {
            // This sample predates the current height, so the change happened
            // no earlier than here.
            return Some(now_unix.saturating_sub(sample.sampled_at_unix));
        }
    }
    // Every observed height matches: report the span covered so far, so a node
    // stalled since the workspace started still accumulates a duration.
    let oldest = history
        .iter()
        .rev()
        .find(|sample| sample.block_height.is_known())?;
    Some(now_unix.saturating_sub(oldest.sampled_at_unix))
}

/// A height that went backwards, as `(from, to)`.
fn height_regression(history: &[NodeSample]) -> Option<(u64, u64)> {
    let current = history.first()?.block_height.value().copied()?;
    let previous = history
        .iter()
        .skip(1)
        .find_map(|sample| sample.block_height.value().copied())?;
    (current < previous).then_some((previous, current))
}

/// Blocks per minute across the sampled window.
///
/// Requires a real span and a non-decreasing height, so a regression produces
/// `None` rather than a confident negative rate.
fn blocks_per_minute(history: &[NodeSample]) -> Option<f64> {
    let newest = history.first()?;
    let newest_height = newest.block_height.value().copied()?;
    let oldest = history
        .iter()
        .rev()
        .find(|sample| sample.block_height.is_known())?;
    let oldest_height = oldest.block_height.value().copied()?;
    if newest_height < oldest_height {
        return None;
    }
    let span = newest.sampled_at_unix.checked_sub(oldest.sampled_at_unix)?;
    if span < 30 {
        // Too short a window to divide by without amplifying one late block
        // into an alarming rate.
        return None;
    }
    Some((newest_height - oldest_height) as f64 * 60.0 / span as f64)
}

/// How full the transaction pool is, against the capacity the node reported.
///
/// `None` when the capacity is unknown, which replaces the compile-time 500 and
/// 2000 thresholds that read "Elevated" on a chain configured for 5,000-tx
/// blocks.
fn mempool_utilisation(latest: &NodeSample) -> Option<f64> {
    let capacity = latest.mempool_capacity.value().copied()?;
    if capacity == 0 {
        return None;
    }
    let verified = latest.mempool_verified.value().copied()?;
    let unverified = latest.mempool_unverified.value().copied().unwrap_or(0);
    Some((verified + unverified) as f64 / capacity as f64)
}

#[cfg(test)]
#[path = "../../tests/unit/observe/derive_tests.rs"]
mod tests;
