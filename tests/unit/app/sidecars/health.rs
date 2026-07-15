use super::super::*;

#[test]
fn launch_pack_sidecar_health_action_probes_signer_endpoints_and_audits() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let launch_pack_root = temp_dir.path().join("private-pack");
    let endpoint = spawn_one_shot_http_server()?;
    write_app_launch_pack_sidecar_manifest_with_endpoint_and_args(
        &launch_pack_root,
        &endpoint,
        &["-c", "echo app-sidecar-ready; sleep 5"],
    )?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);
    app.private_network_last_export_root = Some(launch_pack_root);
    app.refresh_private_network_launch_pack_sidecars();

    app.check_private_network_launch_pack_sidecar_health();

    let report = app
        .private_network_sidecar_health_report
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("sidecar health report should be stored"))?;
    assert_eq!(report.endpoint_count, 1);
    assert_eq!(report.reachable_count, 1);
    assert_eq!(report.unreachable_count, 0);
    assert_eq!(report.results[0].signer_label, "committee-signer-1");
    assert_eq!(report.results[0].endpoint, endpoint);
    assert_eq!(
        report.results[0].status,
        SidecarEndpointHealthStatus::Reachable
    );
    assert!(app.session.notice.as_deref().is_some_and(|notice| {
        notice.contains("1/1 signer endpoints reachable") && notice.contains("committee-signer-1")
    }));
    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "sidecar health", 20))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::PrivateNetworkSignerSidecarHealthChecked));

    Ok(())
}
