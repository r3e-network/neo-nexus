use super::*;

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
