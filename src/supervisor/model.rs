//! The supervisor's data model, split by responsibility so each adapter family
//! can be read on its own: `metrics` describes where a node publishes Prometheus
//! metrics, `log_parsers` turns raw log lines into structured observations,
//! `process` records what a managed process is, and `adapters` binds the
//! implementations to a node type.

use std::time::Duration;

pub(crate) mod adapters;
pub mod log_parsers;
pub(crate) mod metrics;
pub(crate) mod process;

pub use adapters::NodeAdapters;
pub use log_parsers::{
    FatalError, LogEntry, LogParserAdapter, NeoCliLogParser, NeoGoLogParser, NeoRsLogParser,
    NeoXGethLogParser, NeoXRethLogParser, SyncProgress,
};
pub use metrics::MetricsExporterAdapter;
pub use process::{
    unix_timestamp, LaunchConfirmation, ManagedProcessKind, ManagedProcessSpec, PluginMetadata,
    PluginSystemAdapter, ProcessExit, ProcessStart, ProcessStop,
};

pub(super) const DEFAULT_STOP_GRACE_PERIOD: Duration = Duration::from_secs(5);
