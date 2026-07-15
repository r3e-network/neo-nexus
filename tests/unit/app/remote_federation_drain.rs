use super::*;
use crate::app::RemoteFederationProbeResult;

#[test]
fn remote_federation_drain_clears_pending_for_a_profile_deleted_mid_probe() -> anyhow::Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let repository = Repository::open(temp_dir.path().join("neonexus.db"))?;
    let mut app = NeoNexusApp::new(repository);

    let profile = app
        .repository
        .create_remote_server(NewRemoteServerProfile {
            name: "Ops Main".to_string(),
            base_url: "https://ops.example".to_string(),
            description: "primary endpoint".to_string(),
            enabled: true,
        })?;
    app.reload_remote_servers();

    // A probe is in flight: the profile is pending and a result is queued.
    app.async_bus.remote_federation_pending.insert(profile.id.clone());
    app.async_bus.remote_federation_sender
        .send(RemoteFederationProbeResult {
            profile: profile.clone(),
            report: Err("connection refused".to_string()),
        })
        .expect("federation probe channel should accept the queued result");

    // The profile is deleted before its in-flight result is drained.
    app.repository.delete_remote_server(&profile.id)?;
    app.reload_remote_servers();

    // Draining a result for a now-deleted profile must clear the pending marker,
    // not panic, and not resurrect probe records for the deleted profile.
    app.drain_remote_federation_results();

    assert!(!app.async_bus.remote_federation_pending.contains(&profile.id));
    assert!(app
        .repository
        .latest_remote_server_probe(&profile.id)?
        .is_none());

    Ok(())
}
