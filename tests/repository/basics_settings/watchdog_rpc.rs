use super::*;

#[test]
fn backup_restore_watchdog_jitter_old_backup_sets_default_false() {
    use neo_nexus::backup::{restored_workspace_setting, WorkspaceSettingBackup};

    let temp_dir = tempfile::tempdir().unwrap();
    let target = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    // Old backup lacks the jitter key entirely
    let old_backup_settings = [
        WorkspaceSettingBackup {
            key: "watchdog.enabled".to_string(),
            value: "false".to_string(),
        },
        WorkspaceSettingBackup {
            key: "watchdog.max_restart_attempts".to_string(),
            value: "7".to_string(),
        },
        WorkspaceSettingBackup {
            key: "watchdog.base_delay_seconds".to_string(),
            value: "5".to_string(),
        },
        WorkspaceSettingBackup {
            key: "watchdog.max_delay_seconds".to_string(),
            value: "60".to_string(),
        },
    ];

    let workspace_settings: Vec<_> = old_backup_settings
        .iter()
        .map(restored_workspace_setting)
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("valid restored settings");
    target
        .restore_workspace_settings(&workspace_settings)
        .expect("restore ok");

    let policy = target.load_watchdog_policy().expect("load policy");
    assert_eq!(policy.max_restart_attempts, 7);
    assert_eq!(policy.base_delay, Duration::from_secs(5));
    assert_eq!(policy.max_delay, Duration::from_secs(60));
    assert!(
        !policy.jitter_enabled(),
        "old backup missing jitter key must restore as false"
    );
}

#[test]
fn backup_restore_watchdog_jitter_new_backup_preserves_true_and_false() {
    use neo_nexus::backup::{restored_workspace_setting, WorkspaceSettingBackup};

    let temp_dir = tempfile::tempdir().unwrap();

    for enabled in &[true, false] {
        let target =
            Repository::open(temp_dir.path().join(format!("neonexus-{}.db", enabled))).unwrap();

        let new_backup_settings = [
            WorkspaceSettingBackup {
                key: "watchdog.enabled".to_string(),
                value: "true".to_string(),
            },
            WorkspaceSettingBackup {
                key: "watchdog.max_restart_attempts".to_string(),
                value: "5".to_string(),
            },
            WorkspaceSettingBackup {
                key: "watchdog.base_delay_seconds".to_string(),
                value: "3".to_string(),
            },
            WorkspaceSettingBackup {
                key: "watchdog.max_delay_seconds".to_string(),
                value: "45".to_string(),
            },
            WorkspaceSettingBackup {
                key: "watchdog.jitter_enabled".to_string(),
                value: enabled.to_string(),
            },
        ];

        let workspace_settings: Vec<_> = new_backup_settings
            .iter()
            .map(restored_workspace_setting)
            .collect::<Vec<_>>()
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("valid restored settings");
        target
            .restore_workspace_settings(&workspace_settings)
            .expect("restore ok");

        let policy = target.load_watchdog_policy().expect("load policy");
        assert!(policy.enabled);
        assert_eq!(policy.max_restart_attempts, 5);
        assert_eq!(policy.base_delay, Duration::from_secs(3));
        assert_eq!(policy.max_delay, Duration::from_secs(45));
        assert_eq!(
            policy.jitter_enabled(),
            *enabled,
            "new backup must preserve explicit jitter value"
        );
    }
}

#[test]
fn backup_restore_all_workspace_settings_count_equals_input() {
    use neo_nexus::backup::{restored_workspace_setting, WorkspaceSettingBackup};

    let temp_dir = tempfile::tempdir().unwrap();
    let target = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    let all_keys = vec![
        ("watchdog.enabled", "true"),
        ("watchdog.max_restart_attempts", "7"),
        ("watchdog.base_delay_seconds", "4"),
        ("watchdog.max_delay_seconds", "60"),
        ("watchdog.jitter_enabled", "false"),
        ("rpc_health_monitor.enabled", "true"),
        ("rpc_health_monitor.interval_seconds", "300"),
        ("remote_federation_monitor.enabled", "true"),
        ("remote_federation_monitor.interval_seconds", "120"),
        ("appearance.dark_mode", "false"),
        ("appearance.ui_density", "comfortable"),
    ];

    let workspace_settings: Vec<_> = all_keys
        .iter()
        .map(|(k, v)| WorkspaceSettingBackup {
            key: k.to_string(),
            value: v.to_string(),
        })
        .map(|backup| restored_workspace_setting(&backup))
        .collect::<Vec<_>>()
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("valid restored settings");

    let written = target
        .restore_workspace_settings(&workspace_settings)
        .expect("restore ok");
    assert_eq!(
        written,
        all_keys.len(),
        "returned count must equal input slice length"
    );

    for (key, value) in &all_keys {
        let stored = target.load_setting(key).expect("load setting");
        assert_eq!(
            stored.as_deref(),
            Some(*value),
            "all settings must be present after restore"
        );
    }
}

#[test]
fn load_watchdog_policy_missing_key_defaults_false() {
    let temp_dir = tempfile::tempdir().unwrap();
    let target = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    let policy = target.load_watchdog_policy().expect("load policy");
    assert!(policy.max_restart_attempts > 0);
    assert!(policy.base_delay > Duration::ZERO);
    assert!(policy.max_delay >= policy.base_delay);
    assert!(
        !policy.jitter_enabled(),
        "missing jitter key must default to false"
    );
}

#[test]
fn loads_and_persists_watchdog_policy_settings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    assert_eq!(
        repository.load_watchdog_policy().unwrap(),
        default_restart_policy()
    );

    let custom =
        RestartPolicy::with_enabled(false, 7, Duration::from_secs(5), Duration::from_secs(60));
    repository.save_watchdog_policy(custom).unwrap();

    assert_eq!(repository.load_watchdog_policy().unwrap(), custom);

    repository
        .save_watchdog_policy(RestartPolicy::with_enabled(
            true,
            99,
            Duration::from_secs(0),
            Duration::from_secs(0),
        ))
        .unwrap();
    let normalized = repository.load_watchdog_policy().unwrap();
    assert_eq!(normalized.max_restart_attempts, 20);
    assert_eq!(normalized.base_delay, Duration::from_secs(1));
    assert_eq!(normalized.max_delay, Duration::from_secs(1));
}

#[test]
fn verify_save_load_watchdog_policy_with_jitter() {
    use std::time::Duration;

    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    // Test with jitter enabled
    let policy_with_jitter =
        RestartPolicy::with_enabled(true, 5, Duration::from_secs(3), Duration::from_secs(45))
            .with_jitter(true);

    repository.save_watchdog_policy(policy_with_jitter).unwrap();
    let loaded = repository.load_watchdog_policy().unwrap();
    assert!(
        loaded.jitter_enabled(),
        "jitter should be preserved when enabled"
    );
    assert_eq!(loaded.max_restart_attempts, 5);
    assert_eq!(loaded.base_delay, Duration::from_secs(3));
    assert_eq!(loaded.max_delay, Duration::from_secs(45));

    // Test with jitter disabled
    let policy_without_jitter =
        RestartPolicy::with_enabled(true, 3, Duration::from_secs(2), Duration::from_secs(30))
            .with_jitter(false);

    repository
        .save_watchdog_policy(policy_without_jitter)
        .unwrap();
    let loaded = repository.load_watchdog_policy().unwrap();
    assert!(
        !loaded.jitter_enabled(),
        "jitter should be preserved when disabled"
    );
    assert_eq!(loaded.max_restart_attempts, 3);
    assert_eq!(loaded.base_delay, Duration::from_secs(2));
    assert_eq!(loaded.max_delay, Duration::from_secs(30));
}

#[test]
fn loads_and_persists_rpc_health_monitor_policy_settings() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    assert_eq!(
        repository.load_rpc_health_monitor_policy().unwrap(),
        RpcHealthMonitorPolicy::enabled_default()
    );

    let custom = RpcHealthMonitorPolicy {
        enabled: false,
        interval_seconds: 120,
    };
    repository.save_rpc_health_monitor_policy(custom).unwrap();

    assert_eq!(repository.load_rpc_health_monitor_policy().unwrap(), custom);

    repository
        .save_rpc_health_monitor_policy(RpcHealthMonitorPolicy {
            enabled: true,
            interval_seconds: 1,
        })
        .unwrap();
    let normalized = repository.load_rpc_health_monitor_policy().unwrap();

    assert!(normalized.enabled);
    assert_eq!(
        normalized.interval_seconds,
        RpcHealthMonitorPolicy::MIN_INTERVAL_SECONDS
    );
}

#[test]
fn loads_and_persists_app_dark_mode_preference() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    // A fresh workspace defaults to the light theme.
    assert!(!repository.load_app_dark_mode().unwrap());

    repository.save_app_dark_mode(true).unwrap();
    assert!(repository.load_app_dark_mode().unwrap());

    repository.save_app_dark_mode(false).unwrap();
    assert!(!repository.load_app_dark_mode().unwrap());
}

#[test]
fn loads_and_persists_workspace_ui_state() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    // A fresh workspace hides the inspector and has no remembered view.
    assert!(!repository.load_app_inspector_visible().unwrap());
    assert_eq!(repository.load_workspace_last_view().unwrap(), None);

    repository.save_app_inspector_visible(true).unwrap();
    assert!(repository.load_app_inspector_visible().unwrap());

    repository.save_workspace_last_view("operations").unwrap();
    assert_eq!(
        repository.load_workspace_last_view().unwrap().as_deref(),
        Some("operations")
    );
}
