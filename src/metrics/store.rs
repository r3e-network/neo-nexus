//! One long-lived collector, shared, with the recent past kept.
//!
//! Every consumer used to build its own `MetricsCollector::new(Duration::ZERO)`
//! and immediately call `refresh` — the Health page, the Metrics page, the
//! fleet overview, the JSON API and the support bundle. That is two `sysinfo`
//! refreshes back to back, and `sysinfo` refuses to recompute CPU inside its
//! 200 ms minimum interval, so **every CPU figure in the product was the
//! constructor's first sample**: the since-boot average on Linux and
//! effectively zero on macOS. The consequences were not subtle — the "High CPU"
//! filter compared against 50% and therefore never matched, the CPU sort was
//! inert, and `neonexus_system_cpu_usage_percent` could not track load at all.
//!
//! Here the collector is created once and refreshed on the supervision tick, so
//! consecutive samples are a real interval apart and the delta means something.
//!
//! Keeping the samples is what makes a chart possible. The Health page drew a
//! fixed SVG path — the same curve on every host at every moment — captioned
//! "1h Window · 1m Period", with only its right-hand endpoint bound to
//! anything. A ring of real readings replaces it, and the axis can say what it
//! actually covers.

use std::{
    collections::VecDeque,
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::types::NodeConfig;

use super::{collector::MetricsCollector, types::MetricsSnapshot};

/// How often the host is sampled.
///
/// Comfortably above `sysinfo`'s 200 ms minimum, and slow enough that watching
/// the fleet costs less than running it.
pub const SAMPLE_INTERVAL: Duration = Duration::from_secs(10);

/// How many readings are kept.
///
/// At the sample interval above this is one hour, which is what the chart's
/// axis claims and now what it holds.
pub const HISTORY_SAMPLES: usize = 360;

/// One reading of the host, small enough to keep hundreds of.
///
/// Deliberately not a whole `MetricsSnapshot`: the per-process rows are large,
/// they churn as nodes come and go, and nothing plots them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HostSample {
    pub at_unix: u64,
    pub cpu_usage_percent: f32,
    pub memory_usage_percent: f32,
}

/// The shared collector and the readings it has taken.
pub struct MetricsStore {
    inner: Mutex<Inner>,
}

struct Inner {
    collector: MetricsCollector,
    latest: Option<MetricsSnapshot>,
    history: VecDeque<HostSample>,
}

impl Default for MetricsStore {
    fn default() -> Self {
        Self {
            inner: Mutex::new(Inner {
                collector: MetricsCollector::new(SAMPLE_INTERVAL),
                latest: None,
                history: VecDeque::with_capacity(HISTORY_SAMPLES),
            }),
        }
    }
}

impl MetricsStore {
    /// Take a reading if one is due. Called from the supervision tick.
    pub fn refresh_if_due(&self, nodes: &[NodeConfig]) {
        let mut inner = self.lock();
        if let Some(snapshot) = inner.collector.refresh_if_due(nodes, Instant::now()) {
            inner.remember(snapshot);
        }
    }

    /// The most recent reading, taking one if there is none.
    ///
    /// A page served before the engine's first tick — or in a CLI process with
    /// no engine at all — still gets a snapshot rather than nothing. Its CPU
    /// figure is the constructor's sample and therefore weak, which is why the
    /// surfaces render `captured_at_unix` beside it instead of implying the
    /// reading is instantaneous.
    pub fn snapshot(&self, nodes: &[NodeConfig]) -> MetricsSnapshot {
        let mut inner = self.lock();
        if let Some(latest) = inner.latest.clone() {
            return latest;
        }
        let snapshot = inner.collector.refresh(nodes, Instant::now());
        inner.remember(snapshot.clone());
        snapshot
    }

    /// Every reading kept, oldest first.
    pub fn history(&self) -> Vec<HostSample> {
        self.lock().history.iter().copied().collect()
    }

    /// A poisoned lock is recovered rather than propagated: a panicked reader
    /// must not take host telemetry down with it for the life of the process.
    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl Inner {
    fn remember(&mut self, snapshot: MetricsSnapshot) {
        if self.history.len() == HISTORY_SAMPLES {
            self.history.pop_front();
        }
        self.history.push_back(HostSample {
            at_unix: snapshot.captured_at_unix,
            cpu_usage_percent: snapshot.system.cpu_usage_percent,
            memory_usage_percent: snapshot.system.memory_usage_percent,
        });
        self.latest = Some(snapshot);
    }
}

#[cfg(test)]
#[path = "../../tests/unit/metrics/store/tests.rs"]
mod tests;
