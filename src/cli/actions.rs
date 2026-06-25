use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, Instant, SystemTime},
};

use anyhow::{Context, Result};

use crate::core::{
    distribution::{ReleasePackageVerifier, ReleasePackager},
    node::{validate_node_ports, Network, NodeConfig, NodeStatus, NodeType, StorageEngine},
    operations::{
        evaluate_fleet, event_export_filter, preview_alert_route, probe_rpc_endpoint,
        AlertPreviewReport, AlertProvider, EventJournalReporter, EventKind, EventSeverity,
        FleetDiagnostics, MetricsCollector, MetricsSnapshot, RpcHealthReport, RpcHealthStatus,
        RuntimeEvent, RuntimeEventFilter, DEFAULT_EVENT_EXPORT_LIMIT, MAX_EVENT_EXPORT_LIMIT,
    },
    quality::{CiPolicyChecker, NativeUiAuditor, SourcePurityChecker, SourceQualityChecker},
    runtime::{smoke_runtime_command, RuntimeSmokeReport},
    security::NeoWalletValidator,
    workspace::{
        ConfigExporter, ConfigFormat, ConfigValidationReport, ConfigValidationSeverity,
        ConfigValidator, PrivateNetworkLaunchPackVerifier, Repository, WorkspaceBackupExport,
        WorkspaceBackupExporter, WorkspaceBackupImport, WorkspaceBackupImporter,
        WorkspaceConfigExport, WorkspaceConfigExporter, WorkspaceIntegrityChecker,
        WorkspaceIntegrityReport, WorkspaceReadinessReporter, WorkspaceSupportBundleExport,
        WorkspaceSupportBundleExporter,
    },
};

use super::{output::*, CliAction};

mod alerts;
mod backup;
mod basics;
mod completions;
mod config;
mod dispatcher;
mod health;
mod launch_pack;
mod node_control;
mod quality;
mod release;
mod reports;
mod suggest;
mod wallet;
mod workspace;

pub(super) use self::basics::{help_text, self_check_text, version_text};
pub(super) use self::config::GeneratedNodeConfigReport;
pub(super) use self::dispatcher::action_from_args_vec;
use self::{
    alerts::*, backup::*, completions::*, config::*, health::*, launch_pack::*, node_control::*,
    quality::*, release::*, reports::*, wallet::*, workspace::*,
};

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs())
}

fn require_arg_count(args: &[String], expected: usize, option: &str) -> Result<()> {
    match args.len().cmp(&expected) {
        std::cmp::Ordering::Equal => Ok(()),
        std::cmp::Ordering::Less => {
            anyhow::bail!("{option} is missing required arguments; run neo-nexus --help for usage")
        }
        std::cmp::Ordering::Greater => {
            anyhow::bail!("{option} does not accept extra arguments")
        }
    }
}
