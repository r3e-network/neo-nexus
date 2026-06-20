use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use uuid::Uuid;

use crate::{
    alerts::{
        normalized_webhook_url, AlertDelivery, AlertDeliveryReport, AlertProvider,
        AlertRoutingPolicy,
    },
    catalog::{PluginId, PluginState},
    events::{EventSeverity, NewRuntimeEvent, RuntimeEvent, RuntimeEventFilter},
    federation::{
        normalized_remote_input, validate_remote_server_profile, NewRemoteServerProfile,
        RemoteFederationMonitorPolicy, RemoteServerProbeRecord, RemoteServerProbeReport,
        RemoteServerProfile,
    },
    plugins::PluginInstallation,
    rpc_health::{RpcHealthMonitorPolicy, RpcHealthRecord, RpcHealthReport},
    runtime::{
        validate_runtime_catalog_profile, validate_runtime_signer_profile,
        validate_runtime_upgrade_policy, RuntimeCatalogLoad, RuntimeCatalogProfile,
        RuntimeInstallation, RuntimeSignerProfile, RuntimeUpgradePolicy,
    },
    snapshots::{
        normalize_sha256, validate_snapshot_input, FastSyncSnapshot, NewFastSyncSnapshot,
        SnapshotCache, SnapshotVerification,
    },
    types::{Network, NewNode, NodeConfig, NodeStatus, NodeType, StorageEngine},
    wallet::{validate_neo_wallet_profile, NeoWalletProfile},
    watchdog::{default_restart_policy, RestartPolicy},
};

mod events_health;
mod helpers;
mod model;
mod nodes_plugins;
mod policies;
mod remote_servers;
mod rows;
mod runtime_assets;
mod schema;
mod settings_keys;

use self::helpers::*;
pub(crate) use self::helpers::{validate_backup_setting_key, validate_node_config};
pub use self::model::{RestoreNodeOutcome, RestoredRuntimeEvent, WorkspaceSetting};
use self::rows::*;
use self::settings_keys::*;

#[derive(Debug, Clone)]
pub struct Repository {
    db_path: PathBuf,
}

impl Repository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db_path = path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create data directory {}", parent.display()))?;
        }

        let repository = Self { db_path };
        repository.initialize()?;
        Ok(repository)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}
