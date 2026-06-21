use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, Instant, SystemTime},
};

use anyhow::{Context, Result};

use crate::{
    ci_policy::CiPolicyChecker,
    core::{
        distribution::{ReleasePackageVerifier, ReleasePackager},
        node::{validate_node_ports, Network, NodeConfig, NodeStatus, NodeType, StorageEngine},
        operations::{
            evaluate_fleet, event_export_filter, preview_alert_route, probe_rpc_endpoint,
            AlertPreviewReport, AlertProvider, EventJournalReporter, EventKind, EventSeverity,
            FleetDiagnostics, MetricsCollector, MetricsSnapshot, RpcHealthReport, RpcHealthStatus,
            RuntimeEvent, DEFAULT_EVENT_EXPORT_LIMIT, MAX_EVENT_EXPORT_LIMIT,
        },
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
    },
    native_ui::NativeUiAuditor,
    source_purity::SourcePurityChecker,
    source_quality::SourceQualityChecker,
};

use super::{output::*, CliAction};

mod alerts;
mod backup;
mod basics;
mod config;
mod dispatcher;
mod health;
mod launch_pack;
mod quality;
mod release;
mod reports;
mod wallet;
mod workspace;

pub(super) use self::basics::{help_text, self_check_text, version_text};
pub(super) use self::config::GeneratedNodeConfigReport;
pub(super) use self::dispatcher::action_from_args_vec;
use self::{
    alerts::*, backup::*, config::*, health::*, launch_pack::*, quality::*, release::*, reports::*,
    wallet::*, workspace::*,
};

fn current_unix_time() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .context("system clock is before Unix epoch")?
        .as_secs())
}

fn require_arg_count(args: &[String], expected: usize, option: &str) -> Result<()> {
    if args.len() == expected {
        Ok(())
    } else {
        anyhow::bail!("{option} does not accept extra arguments")
    }
}
