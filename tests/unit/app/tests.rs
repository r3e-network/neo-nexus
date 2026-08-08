use crate::{
    alerts::{AlertProvider, AlertRoutingPolicy},
    catalog::{PluginCategory, PluginId},
    events::{EventKind, EventSeverity, NewRuntimeEvent, RuntimeEventFilter},
    federation::{NewRemoteServerProfile, RemoteFederationMonitorPolicy, RemoteProbeStatus},
    private_network::CommitteeRoster,
    repository::Repository,
    rpc_health::{RpcHealthMonitorPolicy, RpcHealthStatus},
    runtime::RuntimeUpgradePolicy,
    runtime::{RuntimeInstallation, RuntimePlatform, RuntimeRelease, RuntimeReleaseCatalog},
    snapshots::{FastSyncSnapshot, FastSyncSnapshotCatalog, FastSyncSnapshotCatalogEntry},
    types::{Network, NewNode, NodeStatus, NodeType, StorageEngine},
};
use std::path::PathBuf;

// Used only from the `#[cfg(unix)]` submodules below — the sidecar suites and the
// runtime-upgrade suites, which need a real forkable process. Imported without
// the gate they are unused on Windows, and CI builds with `-D warnings`.
#[cfg(unix)]
use super::sidecar_health::SidecarEndpointHealthStatus;
use super::{
    should_record_remote_probe_event, should_record_rpc_health_event, AlertRoutingPolicyDraft,
    NeoNexusApp, RemoteFederationMonitorPolicyDraft, RpcHealthMonitorPolicyDraft, View,
};
#[cfg(unix)]
use crate::{runtime::RuntimeCatalogProfile, watchdog::RestartPolicy};

#[cfg(unix)]
use process_fixtures::reconcile_app_processes_until;
#[cfg(unix)]
use sidecar_fixtures::{
    spawn_one_shot_http_server, write_app_launch_pack_sidecar_manifest,
    write_app_launch_pack_sidecar_manifest_with_endpoint_and_args,
    write_app_launch_pack_sidecar_manifest_with_endpoint_binary_and_args,
};
use wallet_fixtures::{valid_nep6_wallet_json, VALID_NEP6_CONTRACT_PUBLIC_KEY};

mod architecture;
mod monitor_filters;
mod nodes_runtime;
mod operations;
mod plugin_filters;
mod policies;
mod process_fixtures;
mod remote_federation_drain;
mod remote_profiles;
mod runtime_filters;
mod settings_wallet;
#[cfg(unix)]
mod sidecar_fixtures;
mod sidecars;
mod snapshot_filters;
mod wallet_fixtures;
