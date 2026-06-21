use super::super::*;

use std::{thread, time::Duration};

#[test]
fn abnormal_launch_pack_sidecar_exit_is_restarted_by_watchdog() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let launch_pack_root = temp_dir.path().join("private-pack");
    write_app_launch_pack_sidecar_manifest_with_endpoint_binary_and_args(
        &launch_pack_root,
        "http://127.0.0.1:9021",
        "false",
        &[],
    )?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    app.watchdog.update_policy(RestartPolicy::new(
        1,
        Duration::from_secs(1),
        Duration::from_secs(1),
    ));
    app.private_network_last_export_root = Some(launch_pack_root.clone());
    app.refresh_private_network_launch_pack_sidecars();
    app.private_network_allow_external_sidecars = true;

    app.start_private_network_launch_pack_sidecars();

    let exited = reconcile_app_processes_until(&mut app, Duration::from_secs(5), |app| {
        app.private_network_sidecar_pids.is_empty()
    });
    if !exited {
        let log_path = launch_pack_root
            .join("signers")
            .join("committee-signer-1")
            .join("committee-signer-1.supervisor.log");
        let log_text = std::fs::read_to_string(&log_path).unwrap_or_default();
        anyhow::bail!(
            "crashing sidecar did not exit; pids {:?}; log:\n{}",
            app.private_network_sidecar_pids,
            log_text
        );
    }
    assert!(app.notice.as_deref().is_some_and(|notice| {
        notice.contains("watchdog restart attempt 1")
            && notice.contains("signer-sidecar:committee-signer-1")
    }));
    let scheduled =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "watchdog restart", 20))?;
    assert!(scheduled.iter().any(|event| {
        event.kind == EventKind::WatchdogScheduled
            && event.message.contains("signer-sidecar:committee-signer-1")
    }));

    thread::sleep(Duration::from_millis(1_100));
    app.reconcile_supervised_processes();
    app.run_due_watchdog_restarts();

    assert!(app
        .private_network_sidecar_pids
        .contains_key("signer:committee-signer-1"));
    assert!(app.notice.as_deref().is_some_and(|notice| {
        notice.contains("signer-sidecar:committee-signer-1 restarted by watchdog attempt 1")
    }));
    let restarted =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "watchdog attempt 1", 20))?;
    assert!(restarted.iter().any(|event| {
        event.kind == EventKind::WatchdogRestarted
            && event.message.contains("signer-sidecar:committee-signer-1")
    }));

    app.stop_private_network_launch_pack_sidecars();

    Ok(())
}
