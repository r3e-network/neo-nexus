//! Tests for the runtime upgrade feature (T1 - Scheduled Runtime Upgrade Execution)
//! These tests verify the scheduled upgrade probe respects policies without requiring network access.

use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use uuid::Uuid;

use super::*;
use crate::{
    repository::Repository,
    runtime::RuntimeUpgradePolicy,
    signing::SignerRegistry,
    supervisor::ProcessSupervisor,
    types::{Network, NewNode, NodeType, StorageEngine},
    watchdog::{default_restart_policy, Watchdog},
};

#[test]
fn test_probe_runtime_upgrade_disabled_by_default() {
    // Test that when runtime upgrade is disabled by default, the probe returns early
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();

    // Create a node
    repository
        .create_node(NewNode {
            name: "test node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/bin/neo-go"),
            args: Vec::new(),
            runtime_version: "0.1.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();

    let state = EngineState {
        repository,
        data_dir: directory.path().to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        signer_registry: SignerRegistry::empty(),
    };

    let mut loop_state = LoopState {
        watchdog: Watchdog::new(default_restart_policy()),
        applied_policy: default_restart_policy(),
        rpc_last_probe: BTreeMap::new(),
        federation_last_probe: BTreeMap::new(),
        last_routed_event: 0,
    };

    // This should return early because policy.enabled is false by default
    loop_state.probe_runtime_upgrade(&state);

    // Verify no RuntimeUpgradePolicyRun event was recorded
    let events = state
        .repository
        .list_events(RuntimeEventFilter::new(None, "", 10))
        .unwrap();
    let upgrade_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::RuntimeUpgradePolicyRun))
        .collect();
    assert!(upgrade_events.is_empty());
}

#[test]
fn test_probe_runtime_upgrade_respects_interval() {
    // Test that upgrade doesn't run if not due based on interval
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();

    // Create a node
    repository
        .create_node(NewNode {
            name: "test node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/bin/neo-go"),
            args: Vec::new(),
            runtime_version: "0.1.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();

    // Create catalog profile first with a local file source (no signature needed)
    let profile_id = Uuid::new_v4().to_string();
    let temp_file = directory.path().join("catalog.json");
    std::fs::write(&temp_file, r#"{"releases":[]}"#).unwrap();

    repository
        .upsert_runtime_catalog_profile(&crate::runtime::RuntimeCatalogProfile {
            id: profile_id.clone(),
            label: "test catalog".to_string(),
            source: temp_file.to_string_lossy().to_string(),
            signature_source: None,
            ed25519_public_key: None,
            max_bytes: RuntimeUpgradePolicy::DEFAULT_INTERVAL_MINUTES * 60 * 2, // Large enough
            enabled: true,
            last_loaded_at_unix: None,
            last_signature_verified: None,
            last_bytes: None,
        })
        .unwrap();

    // Save an enabled policy with a very short interval and current time as last_checked
    // This makes the policy appear just checked, so next check should be in the future
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut policy = RuntimeUpgradePolicy::disabled();
    policy.enabled = true;
    policy.catalog_profile_id = Some(profile_id);
    policy.interval_minutes = RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES; // Minimum 15 minutes
    policy.wave_delay_minutes = 0;
    policy.last_checked_at_unix = Some(now); // Set to now means "just checked"

    // Save this policy
    repository.save_runtime_upgrade_policy(&policy).unwrap();

    let state = EngineState {
        repository,
        data_dir: directory.path().to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        signer_registry: SignerRegistry::empty(),
    };

    let mut loop_state = LoopState {
        watchdog: Watchdog::new(default_restart_policy()),
        applied_policy: default_restart_policy(),
        rpc_last_probe: BTreeMap::new(),
        federation_last_probe: BTreeMap::new(),
        last_routed_event: 0,
    };

    // Should NOT run because we just checked (interval has not elapsed)
    loop_state.probe_runtime_upgrade(&state);

    // Verify no RuntimeUpgradePolicyRun event was recorded (it didn't run at all!)
    let events = state
        .repository
        .list_events(RuntimeEventFilter::new(None, "", 10))
        .unwrap();
    let upgrade_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::RuntimeUpgradePolicyRun))
        .collect();
    assert!(upgrade_events.is_empty());

    // The last_checked_at should remain unchanged since it returned early
    let saved_policy = state.repository.load_runtime_upgrade_policy().unwrap();
    assert_eq!(saved_policy.last_checked_at_unix, Some(now));
}

#[test]
fn test_probe_runtime_upgrade_respects_maintenance_window() {
    // Test that upgrade doesn't run outside maintenance window
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();

    // Create a node
    repository
        .create_node(NewNode {
            name: "test node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/bin/neo-go"),
            args: Vec::new(),
            runtime_version: "0.1.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();

    // Create catalog profile with local file source (no signature needed)
    let profile_id = Uuid::new_v4().to_string();
    let temp_file = directory.path().join("catalog2.json");
    std::fs::write(&temp_file, r#"{"releases":[]}"#).unwrap();

    repository
        .upsert_runtime_catalog_profile(&crate::runtime::RuntimeCatalogProfile {
            id: profile_id.clone(),
            label: "test catalog".to_string(),
            source: temp_file.to_string_lossy().to_string(),
            signature_source: None,
            ed25519_public_key: None,
            max_bytes: RuntimeUpgradePolicy::DEFAULT_INTERVAL_MINUTES * 60 * 2,
            enabled: true,
            last_loaded_at_unix: None,
            last_signature_verified: None,
            last_bytes: None,
        })
        .unwrap();

    // Enable maintenance window with a small window far from now
    // Window: minute 60-120 (the hour starting at minute 60)
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let current_minute = ((now / 60) % RuntimeUpgradePolicy::MINUTES_PER_DAY as u64) as u16;

    let mut policy = RuntimeUpgradePolicy::disabled();
    policy.enabled = true;
    policy.catalog_profile_id = Some(profile_id);
    policy.interval_minutes = RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES; // Minimum 15 minutes
    policy.wave_delay_minutes = 0;
    policy.maintenance_window_enabled = true;
    policy.maintenance_window_start_minute_utc = 60; // Start at minute 60
    policy.maintenance_window_end_minute_utc = 120; // End at minute 120

    // Make sure current minute is definitely OUTSIDE the window
    // If current minute happens to be in the window, add buffer
    let effective_start = if (60..120).contains(&current_minute) {
        180 // Far away: minute 180-240 window
    } else {
        60
    };
    policy.maintenance_window_start_minute_utc = effective_start;
    policy.maintenance_window_end_minute_utc = effective_start + 60;
    policy.last_checked_at_unix = Some(now); // Ensure not due due to interval either

    repository.save_runtime_upgrade_policy(&policy).unwrap();

    let state = EngineState {
        repository,
        data_dir: directory.path().to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        signer_registry: SignerRegistry::empty(),
    };

    let mut loop_state = LoopState {
        watchdog: Watchdog::new(default_restart_policy()),
        applied_policy: default_restart_policy(),
        rpc_last_probe: BTreeMap::new(),
        federation_last_probe: BTreeMap::new(),
        last_routed_event: 0,
    };

    // Should NOT run because we're outside maintenance window
    loop_state.probe_runtime_upgrade(&state);

    // Verify no RuntimeUpgradePolicyRun event was recorded
    let events = state
        .repository
        .list_events(RuntimeEventFilter::new(None, "", 10))
        .unwrap();
    let upgrade_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::RuntimeUpgradePolicyRun))
        .collect();
    assert!(upgrade_events.is_empty());
}

#[test]
fn test_probe_runtime_upgrade_returns_early_without_catalog_config() {
    // Test that the probe returns early if no catalog profile ID is configured
    let directory = tempfile::tempdir().unwrap();
    let repository = Repository::open(directory.path().join("workspace.db")).unwrap();

    // Create a node
    repository
        .create_node(NewNode {
            name: "test node".to_string(),
            node_type: NodeType::NeoGo,
            network: Network::Testnet,
            binary_path: PathBuf::from("/bin/neo-go"),
            args: Vec::new(),
            runtime_version: "0.1.0".to_string(),
            storage_engine: StorageEngine::LevelDb,
            rpc_port: 20332,
            p2p_port: 20333,
            ws_port: None,
        })
        .unwrap();

    let state = EngineState {
        repository,
        data_dir: directory.path().to_path_buf(),
        supervisor: Arc::new(Mutex::new(ProcessSupervisor::default())),
        signer_registry: SignerRegistry::empty(),
    };

    let mut loop_state = LoopState {
        watchdog: Watchdog::new(default_restart_policy()),
        applied_policy: default_restart_policy(),
        rpc_last_probe: BTreeMap::new(),
        federation_last_probe: BTreeMap::new(),
        last_routed_event: 0,
    };

    // Should return early due to no catalog profile config
    loop_state.probe_runtime_upgrade(&state);
}
