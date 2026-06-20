use super::*;

#[test]
fn loads_persists_and_backs_up_sidecar_execution_policy_settings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    assert!(!repository
        .load_private_network_allow_external_sidecars()
        .unwrap());

    repository
        .save_private_network_allow_external_sidecars(true)
        .unwrap();

    assert!(repository
        .load_private_network_allow_external_sidecars()
        .unwrap());

    let settings = repository.list_workspace_settings_for_backup().unwrap();
    assert!(settings.iter().any(|setting| {
        setting.key == "private_network.sidecars.allow_external" && setting.value == "true"
    }));
}

#[test]
fn loads_and_persists_remote_federation_monitor_policy_settings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    assert_eq!(
        repository.load_remote_federation_monitor_policy().unwrap(),
        RemoteFederationMonitorPolicy::enabled_default()
    );

    let custom = RemoteFederationMonitorPolicy {
        enabled: false,
        interval_seconds: 600,
    };
    repository
        .save_remote_federation_monitor_policy(custom)
        .unwrap();

    assert_eq!(
        repository.load_remote_federation_monitor_policy().unwrap(),
        custom
    );

    repository
        .save_remote_federation_monitor_policy(RemoteFederationMonitorPolicy {
            enabled: true,
            interval_seconds: 1,
        })
        .unwrap();
    let normalized = repository.load_remote_federation_monitor_policy().unwrap();

    assert!(normalized.enabled);
    assert_eq!(
        normalized.interval_seconds,
        RemoteFederationMonitorPolicy::MIN_INTERVAL_SECONDS
    );
}
