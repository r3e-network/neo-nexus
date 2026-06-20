use super::super::*;

mod alerts;

#[test]
fn remote_federation_monitor_policy_action_persists_and_audits() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.remote_federation_monitor_policy_draft = RemoteFederationMonitorPolicyDraft {
        enabled: false,
        interval_seconds: 600,
    };
    app.save_remote_federation_monitor_policy();

    assert!(!app.remote_federation_monitor_policy.enabled);
    assert_eq!(app.remote_federation_monitor_policy.interval_seconds, 600);
    assert_eq!(
        app.repository
            .load_remote_federation_monitor_policy()?
            .interval_seconds,
        600
    );
    let events = app.repository.list_events(RuntimeEventFilter::new(
        None,
        "remote-federation-monitor",
        10,
    ))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::RemoteFederationMonitorPolicyUpdated));

    Ok(())
}

#[test]
fn sidecar_execution_policy_action_persists_audits_and_reloads() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("neonexus.db");
    let repository = Repository::open(&db_path)?;
    let mut app = NeoNexusApp::new(repository);

    assert!(!app.private_network_allow_external_sidecars);

    app.private_network_allow_external_sidecars = true;
    app.save_private_network_sidecar_execution_policy();

    assert!(app.private_network_allow_external_sidecars);
    assert!(app
        .repository
        .load_private_network_allow_external_sidecars()?);
    assert!(app.notice.as_deref().is_some_and(|notice| {
        notice.contains("Sidecar execution policy saved") && notice.contains("external allowed")
    }));
    let events = app.repository.list_events(RuntimeEventFilter::new(
        None,
        "sidecar execution policy",
        10,
    ))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::PrivateNetworkSignerSidecarPolicyUpdated));

    let reloaded = NeoNexusApp::new(Repository::open(db_path)?);
    assert!(reloaded.private_network_allow_external_sidecars);

    Ok(())
}

#[test]
fn runtime_upgrade_policy_manual_run_records_missing_catalog_profile_failure() -> anyhow::Result<()>
{
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.runtime_upgrade_policy = RuntimeUpgradePolicy {
        enabled: true,
        catalog_profile_id: Some("missing-profile".to_string()),
        interval_minutes: RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES,
        require_signed_catalog: true,
        max_nodes_per_run: 1,
        maintenance_window_enabled: false,
        maintenance_window_start_minute_utc: 0,
        maintenance_window_end_minute_utc: 6 * 60,
        wave_delay_minutes: 0,
        last_checked_at_unix: None,
        last_applied_at_unix: None,
    };

    app.run_runtime_upgrade_policy_now();

    assert!(app.notice.as_deref().is_some_and(|notice| {
        notice.contains("Runtime upgrade policy manual run failed")
            && notice.contains("missing-profile")
    }));
    let saved = app.repository.load_runtime_upgrade_policy()?;
    assert!(saved.last_checked_at_unix.is_some());
    assert!(saved.last_applied_at_unix.is_none());
    let events = app.repository.list_events(RuntimeEventFilter::new(
        Some(EventSeverity::Warning),
        "missing-profile",
        10,
    ))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::RuntimeUpgradePolicyRun));

    Ok(())
}
