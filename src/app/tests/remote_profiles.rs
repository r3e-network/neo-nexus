use super::*;
use crate::federation::RemoteServerProbeReport;

#[test]
fn remote_federation_profile_actions_persist_and_audit_events() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    app.remote_server_name = "Remote Lab".to_string();
    app.remote_server_base_url = "nexus.example.com/ops/".to_string();
    app.remote_server_description = "Public status endpoint".to_string();
    app.remote_server_enabled = true;
    app.create_remote_server_profile();

    assert_eq!(app.remote_servers.len(), 1);
    let profile_id = app.remote_servers[0].id.clone();
    assert_eq!(
        app.remote_servers[0].base_url,
        "https://nexus.example.com/ops"
    );

    app.remote_server_name = "Remote Lab Updated".to_string();
    app.remote_server_description = "Updated endpoint".to_string();
    app.update_selected_remote_server_profile();
    assert_eq!(app.remote_servers[0].name, "Remote Lab Updated");

    app.toggle_selected_remote_server_enabled();
    assert!(app
        .remote_servers
        .iter()
        .find(|profile| profile.id == profile_id)
        .is_some_and(|profile| !profile.enabled));

    app.probe_selected_remote_server();
    let Some(probe) = app.selected_remote_server_probe() else {
        anyhow::bail!("disabled probe should be recorded");
    };
    assert_eq!(probe.status, RemoteProbeStatus::Disabled);
    assert_eq!(
        app.repository
            .list_remote_server_probes(&profile_id, 10)?
            .len(),
        1
    );

    app.delete_selected_remote_server();
    assert!(app.remote_servers.is_empty());

    let events = app
        .repository
        .list_events(RuntimeEventFilter::new(None, "remote-server", 10))?;
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::RemoteServerCreated));
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::RemoteServerUpdated));
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::RemoteServerProbed));
    assert!(events
        .iter()
        .any(|event| event.kind == EventKind::RemoteServerDeleted));

    Ok(())
}

#[test]
fn remote_federation_profile_filter_limits_visible_profiles() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    repository.create_remote_server(NewRemoteServerProfile {
        name: "Ops Main".to_string(),
        base_url: "https://ops.example".to_string(),
        description: "primary endpoint".to_string(),
        enabled: true,
    })?;
    let disabled = repository.create_remote_server(NewRemoteServerProfile {
        name: "Ops Lab".to_string(),
        base_url: "https://lab.example".to_string(),
        description: "standby lab".to_string(),
        enabled: false,
    })?;
    repository.create_remote_server(NewRemoteServerProfile {
        name: "Seed Lab".to_string(),
        base_url: "https://seed.example".to_string(),
        description: "active lab".to_string(),
        enabled: true,
    })?;
    let mut app = NeoNexusApp::new(repository);
    app.remote_server_query = "lab".to_string();
    app.remote_server_enabled_filter = Some(false);

    let visible = app.filtered_remote_server_profiles();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, disabled.id);

    app.remote_server_page = 4;
    app.ensure_valid_remote_server_selection();
    assert_eq!(app.remote_server_page, 0);

    Ok(())
}

#[test]
fn remote_probe_history_filter_limits_visible_records_and_clamps_page() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let profile = repository.create_remote_server(NewRemoteServerProfile {
        name: "Ops Main".to_string(),
        base_url: "https://ops.example".to_string(),
        description: "primary endpoint".to_string(),
        enabled: true,
    })?;
    for (index, status, message) in [
        (0, RemoteProbeStatus::Healthy, "ok"),
        (1, RemoteProbeStatus::Degraded, "slow peers"),
        (2, RemoteProbeStatus::Healthy, "caught up"),
        (3, RemoteProbeStatus::Unreachable, "timeout from lab router"),
        (4, RemoteProbeStatus::Disabled, "operator paused profile"),
        (5, RemoteProbeStatus::Degraded, "syncing"),
        (6, RemoteProbeStatus::Healthy, "stable"),
    ] {
        repository.record_remote_server_probe(&probe_report(
            &profile.id,
            &profile.name,
            status,
            100 + index,
            message,
        ))?;
    }
    let mut app = NeoNexusApp::new(repository);

    assert_eq!(app.selected_remote_server_probe_history().len(), 7);
    app.remote_probe_history_status_filter = Some(RemoteProbeStatus::Unreachable);
    app.remote_probe_history_query = "timeout".to_string();
    app.remote_probe_history_page = 7;

    let visible = app.filtered_selected_remote_server_probe_history();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].status, RemoteProbeStatus::Unreachable);
    assert!(visible[0].message.contains("timeout"));

    app.clamp_remote_probe_history_page();
    assert_eq!(app.remote_probe_history_page, 0);

    Ok(())
}

fn probe_report(
    profile_id: &str,
    profile_name: &str,
    status: RemoteProbeStatus,
    checked_at_unix: u64,
    message: &str,
) -> RemoteServerProbeReport {
    RemoteServerProbeReport {
        profile_id: profile_id.to_string(),
        profile_name: profile_name.to_string(),
        base_url: "https://ops.example".to_string(),
        checked_at_unix,
        status,
        total_nodes: Some(4),
        running_nodes: Some(3),
        syncing_nodes: Some(1),
        error_nodes: Some(0),
        total_blocks: Some(12_000),
        total_peers: Some(32),
        public_node_count: Some(4),
        message: message.to_string(),
    }
}
