use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use ed25519_dalek::SigningKey;
use neo_nexus::{
    alerts::{AlertDeliveryReport, AlertDeliveryStatus, AlertProvider, AlertRoutingPolicy},
    catalog::PluginId,
    events::{EventKind, EventSeverity, NewRuntimeEvent, RuntimeEventFilter},
    federation::{
        NewRemoteServerProfile, RemoteFederationMonitorPolicy, RemoteProbeStatus,
        RemoteServerProbeReport,
    },
    plugins::PluginInstallation,
    repository::Repository,
    rpc_health::{RpcHealthMonitorPolicy, RpcHealthReport, RpcHealthStatus},
    runtime::{
        RuntimeCatalogLoad, RuntimeCatalogProfile, RuntimeReleaseCatalog, RuntimeSignerProfile,
        RuntimeUpgradePolicy,
    },
    types::{Network, NewNode, NodeStatus, NodeType, StorageEngine},
    wallet::NeoWalletProfile,
    watchdog::{default_restart_policy, RestartPolicy},
};

use std::{path::PathBuf, time::Duration};

#[path = "repository/basics_settings.rs"]
mod basics_settings;
#[path = "repository/nodes_health_plugins.rs"]
mod nodes_health_plugins;
#[path = "repository/runtime_events.rs"]
mod runtime_events;
