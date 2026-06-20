use super::*;

#[test]
fn persists_runtime_upgrade_policy_settings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    assert_eq!(
        repository.load_runtime_upgrade_policy().unwrap(),
        RuntimeUpgradePolicy::disabled()
    );

    let policy = RuntimeUpgradePolicy {
        enabled: true,
        catalog_profile_id: Some("official-neo-rs".to_string()),
        interval_minutes: 60,
        require_signed_catalog: true,
        max_nodes_per_run: 4,
        maintenance_window_enabled: true,
        maintenance_window_start_minute_utc: 90,
        maintenance_window_end_minute_utc: 180,
        wave_delay_minutes: 240,
        last_checked_at_unix: Some(1_800_000_100),
        last_applied_at_unix: Some(1_800_000_200),
    };

    repository.save_runtime_upgrade_policy(&policy).unwrap();
    let loaded = repository.load_runtime_upgrade_policy().unwrap();

    assert_eq!(loaded, policy);

    let disabled_without_profile = RuntimeUpgradePolicy {
        enabled: false,
        catalog_profile_id: None,
        interval_minutes: RuntimeUpgradePolicy::DEFAULT_INTERVAL_MINUTES,
        require_signed_catalog: false,
        max_nodes_per_run: 1,
        maintenance_window_enabled: false,
        maintenance_window_start_minute_utc: 0,
        maintenance_window_end_minute_utc: 360,
        wave_delay_minutes: 0,
        last_checked_at_unix: None,
        last_applied_at_unix: None,
    };
    repository
        .save_runtime_upgrade_policy(&disabled_without_profile)
        .unwrap();

    assert_eq!(
        repository.load_runtime_upgrade_policy().unwrap(),
        disabled_without_profile
    );
}
