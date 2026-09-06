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

#[test]
fn a_new_workspace_creates_no_local_signer_custody_tables() {
    let home = tempfile::tempdir().expect("temporary workspace");
    let database = home.path().join("neonexus.db");
    Repository::open(&database).expect("workspace opens");
    let connection = rusqlite::Connection::open(&database).expect("database reopens");

    for table in [
        "signer_keys",
        "signer_policies",
        "signer_callers",
        "signer_audit",
        "signer_value_events",
    ] {
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                [table],
                |row| row.get(0),
            )
            .expect("schema query");
        assert_eq!(
            count, 0,
            "{table} makes neo-nexus a second custody database"
        );
    }
}
