use super::super::*;

#[test]
fn launch_pack_sidecar_start_blocks_external_binary_until_operator_allows_it() -> anyhow::Result<()>
{
    let temp_dir = tempfile::tempdir()?;
    let launch_pack_root = temp_dir.path().join("private-pack");
    write_app_launch_pack_sidecar_manifest_with_endpoint_binary_and_args(
        &launch_pack_root,
        "http://127.0.0.1:9021",
        "/bin/sh",
        &["-c", "echo app-sidecar-ready; sleep 5"],
    )?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    app.private_network_last_export_root = Some(launch_pack_root);
    app.refresh_private_network_launch_pack_sidecars();

    app.start_private_network_launch_pack_sidecars();

    assert!(app.private_network_sidecar_pids.is_empty());
    assert!(app.session.notice.as_deref().is_some_and(|notice| {
        notice.contains("blocked by sidecar execution policy")
            && notice.contains("committee-signer-1")
    }));
    let blocked_events =
        app.repository
            .list_events(RuntimeEventFilter::new(None, "execution policy", 20))?;
    assert!(blocked_events.iter().any(|event| {
        event.kind == EventKind::PrivateNetworkSignerSidecarExecutionBlocked
            && event.message.contains("/bin/sh")
    }));

    app.private_network_allow_external_sidecars = true;
    app.start_private_network_launch_pack_sidecars();

    assert_eq!(app.private_network_sidecar_pids.len(), 1);
    assert!(app.session
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("1 signer sidecar started")));

    app.stop_private_network_launch_pack_sidecars();

    Ok(())
}

#[test]
fn launch_pack_sidecar_actions_start_stop_and_audit_signer_processes() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let launch_pack_root = temp_dir.path().join("private-pack");
    let manifest_path = write_app_launch_pack_sidecar_manifest(&launch_pack_root)?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    app.private_network_last_export_root = Some(launch_pack_root.clone());

    app.refresh_private_network_launch_pack_sidecars();

    let sidecar_report = app
        .private_network_sidecar_report
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("sidecar report should be loaded"))?;
    assert_eq!(sidecar_report.manifest_path, manifest_path);
    assert_eq!(sidecar_report.sidecar_count, 1);
    assert!(app.private_network_sidecar_pids.is_empty());
    assert!(app.session
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("signer sidecar spec loaded")));

    app.start_private_network_launch_pack_sidecars();

    assert_eq!(app.private_network_sidecar_pids.len(), 1);
    assert!(app
        .private_network_sidecar_pids
        .contains_key("signer:committee-signer-1"));
    assert!(app.session
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("1 signer sidecar started")));
    let log_path = launch_pack_root
        .join("signers")
        .join("committee-signer-1")
        .join("committee-signer-1.supervisor.log");
    assert!(log_path.exists());
    let log_text = std::fs::read_to_string(&log_path)?;
    assert!(log_text.contains("process-kind: sidecar"));
    assert!(log_text.contains("app-sidecar-ready"));

    app.stop_private_network_launch_pack_sidecars();

    assert!(app.private_network_sidecar_pids.is_empty());
    assert!(app.session
        .notice
        .as_deref()
        .is_some_and(|notice| notice.contains("1 signer sidecar stopped")));

    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "signer-sidecar", 20))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::PrivateNetworkSignerSidecarStarted));
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::PrivateNetworkSignerSidecarStopped));

    Ok(())
}
