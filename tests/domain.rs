use std::{
    collections::BTreeMap,
    fs::File,
    io::{Cursor, Write},
    path::{Path, PathBuf},
    str::FromStr,
    time::{Duration, Instant},
};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use ed25519_dalek::{Signer, SigningKey};
use flate2::{write::GzEncoder, Compression};
use neo_nexus::{
    backup::{WorkspaceBackupExporter, WorkspaceBackupImporter, WorkspaceSettingBackup},
    catalog::{PluginCatalog, PluginId, PluginState},
    config::{
        ConfigExporter, ConfigFormat, ConfigGenerator, ConfigValidationSeverity, ConfigValidator,
        RuntimeConfigProfile, WorkspaceConfigExporter,
    },
    dashboard::DashboardSummary,
    diagnostics::{
        evaluate_fleet, evaluate_launch_readiness, evaluate_launch_readiness_with_port_probe,
        evaluate_node, evaluate_restart_readiness_with_port_probe, CheckSeverity,
    },
    events::{EventKind, EventSeverity, NewRuntimeEvent},
    federation::{normalize_remote_base_url, parse_public_status, NewRemoteServerProfile},
    launch::{runtime_args_include_config, LaunchPlanner},
    logs::LogReader,
    metrics::{
        format_bytes, MetricsSnapshot, MissingProcessMetric, NodeProcessMetrics, ResourcePressure,
        SystemMetrics,
    },
    plugins::{PluginInstallation, PluginPackageManager, PluginPackageManifest},
    private_network::{
        CommitteeRoster, CommitteeSidecarProcess, LaunchPackValidationStatus,
        PrivateNetworkDeploymentExporter, PrivateNetworkDeploymentRequest,
        PrivateNetworkLaunchPackSidecarReport, PrivateNetworkLaunchPackVerifier,
    },
    repository::Repository,
    roles::{NodeRole, PrivateNetworkPlanner, PrivateNetworkTemplate, RolePlanner},
    rpc_health::RpcHealthMonitorPolicy,
    runtime::{
        validate_catalog_load_request, validate_download_request, validate_https_redirect,
        validate_runtime_signer_profile, validate_runtime_upgrade_policy,
        RuntimeCatalogLoadRequest, RuntimeCatalogProfile, RuntimeDownloadRequest,
        RuntimeInstallation, RuntimePackageManager, RuntimePackageManifest, RuntimePlatform,
        RuntimeReleaseCatalog, RuntimeSignerProfile, RuntimeUpgradePolicy,
    },
    snapshots::{
        sha256_file, validate_snapshot_catalog_load_request, validate_snapshot_download_request,
        validate_snapshot_https_redirect, FastSyncSnapshot, FastSyncSnapshotCatalog,
        FastSyncSnapshotManager, NewFastSyncSnapshot, SnapshotCatalogLoadRequest,
        SnapshotDownloadRequest, SnapshotImportMode,
    },
    supervisor::{log_path_for, ManagedProcessKind, ManagedProcessSpec},
    types::{Network, NewNode, NodeConfig, NodeStatus, NodeType, StorageEngine},
    wallet::NeoWalletProfile,
    watchdog::{RestartOutcome, RestartPolicy, Watchdog, WatchdogStatus},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipWriter};

#[cfg(unix)]
use neo_nexus::{
    launch::LaunchPlan,
    supervisor::{ProcessStart, ProcessSupervisor},
};

#[cfg(unix)]
use std::thread;

fn create_repo() -> Repository {
    let temp_dir = tempfile::tempdir().unwrap();
    Repository::open(temp_dir.keep().join("neonexus.db")).unwrap()
}

fn create_node(repo: &Repository, name: &str, node_type: NodeType) -> String {
    repo.create_node(NewNode {
        name: name.to_string(),
        node_type,
        network: Network::Testnet,
        binary_path: PathBuf::from("/usr/local/bin/node"),
        args: Vec::new(),
        runtime_version: "latest".to_string(),
        storage_engine: node_type.default_storage_engine(),
        rpc_port: 10332,
        p2p_port: 10333,
        ws_port: Some(10334),
    })
    .unwrap()
    .id
}

fn planned_plugin_enabled(plan: &neo_nexus::roles::RolePlan, plugin_id: PluginId) -> Option<bool> {
    plan.plugin_changes
        .iter()
        .find(|change| change.plugin_id == plugin_id)
        .map(|change| change.enabled)
}

fn committee_public_key(prefix: &str, byte: char) -> String {
    format!("{prefix}{}", byte.to_string().repeat(64))
}

const VALID_NEP6_CONTRACT_PUBLIC_KEY: &str =
    "036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0";

fn write_fake_executable(path: &Path) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    File::create(path).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
}

fn valid_nep6_wallet_json() -> String {
    serde_json::json!({
        "name": "NeoNexus validator wallet",
        "version": "3.0",
        "scrypt": {
            "n": 16384,
            "r": 8,
            "p": 8
        },
        "accounts": [
            {
                "address": "AQLASLtT6pWbThcSCYU1biVqhMnzhTgLFq",
                "label": "validator-1",
                "isDefault": true,
                "lock": false,
                "key": "6PYWB8m1bCnu5bQkRUKAwbZp2BHNvQ3BQRLbpLdTuizpyLkQPSZbtZfoxx",
                "contract": {
                    "script": "21036dc4bf8f0405dcf5d12a38487b359cb4bd693357a387d74fc438ffc7757948b0ac",
                    "parameters": [],
                    "deployed": false
                },
                "extra": null
            }
        ],
        "extra": null
    })
    .to_string()
}

#[path = "domain/backup_diagnostics.rs"]
mod backup_diagnostics;
#[path = "domain/config_launch_supervisor.rs"]
mod config_launch_supervisor;
#[path = "domain/plugins_snapshots.rs"]
mod plugins_snapshots;
#[path = "domain/roles_private_network.rs"]
mod roles_private_network;
#[path = "domain/runtime_federation.rs"]
mod runtime_federation;
