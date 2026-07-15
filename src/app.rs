use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    thread,
    time::Instant,
};

use anyhow::Context;
use uuid::Uuid;

mod appearance_flow;
mod constants;
mod domain;
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
mod workspace_section_flow;

use constants::*;
use domain::*;
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
use theme::{Theme, UiDensity};
use views::{
    FederationSection, MonitorSection, NodeWorkspaceTab, OperationsSection, RolesSection,
    RuntimesSection, SettingsSection, SnapshotsSection,
};
use workflow::{
    committee_keys_with_wallet_profile, current_unix_time, data_dir, exit_notice, format_duration,
    non_empty_text, optional_text, preflight_notice, rpc_health_notice,
    runtime_smoke_event_severity, runtime_smoke_notice, signer_refs_has_public_key,
    signer_refs_with_wallet_profile, AlertRoutingPolicyDraft, RemoteFederationMonitorPolicyDraft,
    RemoteFederationProbeResult, RpcHealthMonitorPolicyDraft, RpcHealthProbeResult,
    RuntimeUpgradePolicyDraft, StartMode, WatchdogPolicyDraft,
};

pub use state::NeoNexusApp;
pub(in crate::app) use state::{
    render_toast_strip, AsyncProbeBus, FleetUi, OperationsUi, SessionUi, WorkspaceSections,
};
pub use view::View;

#[cfg(test)]
#[path = "../tests/unit/app/tests.rs"]
mod tests;
