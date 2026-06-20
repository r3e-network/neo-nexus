use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Instant,
};

use anyhow::Context;
use uuid::Uuid;

mod constants;
mod draft;
mod event_recording_flow;
mod frame;
mod health_events;
mod lifecycle;
mod managed_config_flow;
mod monitor_flow;
mod node_lifecycle_flow;
mod operations_flow;
mod paging;
mod plugin_flow;
mod policy_alert_flow;
mod private_network_flow;
mod remote_federation_flow;
mod rpc_health_flow;
mod runtime_draft;
mod runtime_profile_flow;
mod runtime_release_flow;
mod runtime_upgrade;
mod runtime_upgrade_policy;
mod shortcuts;
mod sidecar_health;
mod snapshot_draft;
mod snapshot_flow;
mod state;
mod supervision_flow;
mod text;
mod theme;
mod view;
mod views;
mod widgets;
mod workflow;

use crate::{
    alerts::{
        alert_target_label, deliver_webhook_alert, preview_alert_route, should_route_alert,
        AlertDeliveryReport, AlertDeliveryStatus, AlertPreviewReport, AlertRoutingPolicy,
    },
    backup::{WorkspaceBackupExporter, WorkspaceBackupImporter, WorkspaceBackupValidation},
    catalog::{
        filter_plugin_definitions, PluginCatalog, PluginCategory, PluginDefinition,
        PluginDefinitionFilter, PluginId, PluginState,
    },
    config::ConfigExporter,
    diagnostics::{
        evaluate_launch_readiness, evaluate_restart_readiness, filter_diagnostic_checks,
        filter_port_matrix_rows, filter_readiness_actions, CheckSeverity, DiagnosticCheck,
        DiagnosticCheckFilter, DiagnosticCheckKey, FleetDiagnostics, NodeDiagnostics,
        PortMatrixFilter, PortMatrixRow, ReadinessAction, ReadinessActionFilter,
        ReadinessActionKey,
    },
    event_journal_report::{EventJournalReporter, DEFAULT_EVENT_EXPORT_LIMIT},
    events::{EventKind, EventSeverity, NewRuntimeEvent, RuntimeEvent, RuntimeEventFilter},
    federation::{
        filter_remote_probe_history, filter_remote_server_profiles, NewRemoteServerProfile,
        RemoteFederationClient, RemoteFederationMonitorPolicy, RemoteProbeHistoryFilter,
        RemoteProbeStatus, RemoteServerProbeRecord, RemoteServerProbeReport, RemoteServerProfile,
        RemoteServerProfileFilter,
    },
    launch::{LaunchPlan, LaunchPlanner},
    logs::{LogDiagnosisStatus, LogReader},
    metrics::{
        filter_process_rows, format_bytes, MetricsCollector, MetricsSnapshot, ProcessFilter,
        ProcessRow, ProcessStateFilter,
    },
    plugins::{PluginPackageManager, PluginPackageManifest},
    port_planner::{plan_available_node_ports, PortAssignment, DEFAULT_RPC_PORT},
    preflight::inspect_node_binary,
    private_network::{
        CommitteeRoster, PrivateNetworkDeploymentExporter, PrivateNetworkDeploymentRequest,
        PrivateNetworkLaunchPackSidecarReport, PrivateNetworkLaunchPackValidation,
        PrivateNetworkLaunchPackVerifier,
    },
    readiness_report::WorkspaceReadinessReporter,
    release_pack::{
        ReleasePackage, ReleasePackageVerification, ReleasePackageVerifier, ReleasePackager,
    },
    repository::Repository,
    roles::{NodeRole, PrivateNetworkPlanner, PrivateNetworkTemplate, RolePlanner},
    rpc_health::{probe_node_rpc, RpcHealthMonitorPolicy},
    runtime::{
        RuntimeCatalogLoadRequest, RuntimeCatalogProfile, RuntimeInstallation,
        RuntimePackageManager, RuntimePlatform, RuntimeReleaseCatalog, RuntimeSignerProfile,
        RuntimeUpgradePolicy,
    },
    runtime_smoke::smoke_node_binary,
    snapshots::{
        filter_snapshot_catalog_entries, filter_snapshots, sha256_file, FastSyncSnapshot,
        FastSyncSnapshotCatalog, FastSyncSnapshotCatalogEntry, FastSyncSnapshotManager,
        SnapshotCatalogEntryFilter, SnapshotCatalogLoadRequest, SnapshotFilter,
    },
    supervisor::{log_path_for, ProcessExit, ProcessSupervisor},
    support_bundle::WorkspaceSupportBundleExporter,
    types::{
        filter_nodes, Network, NewNode, NodeConfig, NodeInventoryFilter, NodeStatus, NodeType,
    },
    wallet::{
        filter_neo_wallet_profiles, NeoWalletProfile, NeoWalletProfileFilter, NeoWalletValidator,
    },
    watchdog::{default_restart_policy, RestartOutcome, Watchdog, WatchdogStatus},
    workspace_integrity::{WorkspaceIntegrityChecker, WorkspaceIntegrityReport},
};

use constants::*;
use draft::NodeDraft;
use health_events::{
    remote_probe_event_severity, remote_probe_failure_report, rpc_health_event_severity,
    should_record_remote_probe_event, should_record_rpc_health_event,
};
use paging::clamp_page;
use runtime_draft::RuntimePackageDraft;
use sidecar_health::{
    launch_pack_validation_severity, private_launch_pack_validation_notice,
    probe_sidecar_endpoint_health, sidecar_execution_policy_finding,
    sidecar_execution_policy_findings, sidecar_execution_policy_label, sidecar_health_notice,
    SidecarEndpointHealthReport,
};
use snapshot_draft::SnapshotDraft;
use text::short_path;
use view::View;
use workflow::{
    committee_keys_with_wallet_profile, current_unix_time, data_dir, exit_notice, format_duration,
    non_empty_text, optional_text, preflight_notice, rpc_health_notice,
    runtime_smoke_event_severity, runtime_smoke_notice, signer_refs_has_public_key,
    signer_refs_with_wallet_profile, AlertRoutingPolicyDraft, RemoteFederationMonitorPolicyDraft,
    RemoteFederationProbeResult, RpcHealthMonitorPolicyDraft, RpcHealthProbeResult,
    RuntimeUpgradePolicyDraft, StartMode, WatchdogPolicyDraft,
};

pub use state::NeoNexusApp;

#[cfg(test)]
mod tests;
