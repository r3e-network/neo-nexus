use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, Instant, SystemTime},
};

use anyhow::{Context, Result};

use crate::{
    alerts::{preview_alert_route, AlertPreviewReport, AlertProvider},
    backup::{
        WorkspaceBackupExport, WorkspaceBackupExporter, WorkspaceBackupImport,
        WorkspaceBackupImporter,
    },
    ci_policy::CiPolicyChecker,
    config::{
        ConfigExporter, ConfigFormat, ConfigValidationReport, ConfigValidationSeverity,
        ConfigValidator, WorkspaceConfigExport, WorkspaceConfigExporter,
    },
    diagnostics::{evaluate_fleet, FleetDiagnostics},
    event_journal_report::{
        event_export_filter, EventJournalReporter, DEFAULT_EVENT_EXPORT_LIMIT,
        MAX_EVENT_EXPORT_LIMIT,
    },
    events::{EventKind, EventSeverity, RuntimeEvent},
    metrics::{MetricsCollector, MetricsSnapshot},
    native_ui::NativeUiAuditor,
    private_network::PrivateNetworkLaunchPackVerifier,
    readiness_report::WorkspaceReadinessReporter,
    release_pack::{ReleasePackageVerifier, ReleasePackager},
    repository::Repository,
    rpc_health::{probe_rpc_endpoint, RpcHealthReport, RpcHealthStatus},
    runtime_smoke::{smoke_runtime_command, RuntimeSmokeReport},
    source_purity::SourcePurityChecker,
    source_quality::SourceQualityChecker,
    support_bundle::{WorkspaceSupportBundleExport, WorkspaceSupportBundleExporter},
    types::{validate_node_ports, Network, NodeConfig, NodeStatus, NodeType, StorageEngine},
    wallet::NeoWalletValidator,
    workspace_integrity::{WorkspaceIntegrityChecker, WorkspaceIntegrityReport},
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
