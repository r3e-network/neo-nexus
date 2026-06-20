use super::*;

#[test]
fn persists_remote_server_profiles_with_normalized_urls() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repository = Repository::open(temp_dir.path().join("neonexus.db")).unwrap();

    let profile = repository
        .create_remote_server(NewRemoteServerProfile {
            name: " Remote Lab ".to_string(),
            base_url: "nexus.example.com/ops/".to_string(),
            description: "  read-only federation  ".to_string(),
            enabled: true,
        })
        .unwrap();

    assert!(profile.id.starts_with("remote-"));
    assert_eq!(profile.name, "Remote Lab");
    assert_eq!(profile.base_url, "https://nexus.example.com/ops");
    assert_eq!(profile.description, "read-only federation");

    let duplicate = repository.create_remote_server(NewRemoteServerProfile {
        name: "Duplicate URL".to_string(),
        base_url: "https://nexus.example.com/ops".to_string(),
        description: String::new(),
        enabled: true,
    });
    assert!(duplicate.is_err());

    let updated = repository
        .update_remote_server(
            &profile.id,
            NewRemoteServerProfile {
                name: "Remote Lab Renamed".to_string(),
                base_url: "http://localhost:9090/".to_string(),
                description: "local validation endpoint".to_string(),
                enabled: false,
            },
        )
        .unwrap();
    assert_eq!(updated.name, "Remote Lab Renamed");
    assert_eq!(updated.base_url, "http://localhost:9090");
    assert!(!updated.enabled);
    assert_eq!(updated.created_at_unix, profile.created_at_unix);
    assert!(updated.updated_at_unix >= updated.created_at_unix);

    let enabled = repository
        .set_remote_server_enabled(&profile.id, true)
        .unwrap();
    assert!(enabled.enabled);

    let profiles = repository.list_remote_servers().unwrap();
    assert_eq!(profiles.len(), 1);
    assert_eq!(profiles[0].id, profile.id);

    for sample in 0..3 {
        let report = RemoteServerProbeReport {
            profile_id: profile.id.clone(),
            profile_name: enabled.name.clone(),
            base_url: enabled.base_url.clone(),
            checked_at_unix: 1_800_000_000 + sample,
            status: if sample == 2 {
                RemoteProbeStatus::Degraded
            } else {
                RemoteProbeStatus::Healthy
            },
            total_nodes: Some(7),
            running_nodes: Some(6 - sample),
            syncing_nodes: Some(sample),
            error_nodes: Some((sample == 2) as u64),
            total_blocks: Some(1_000 + sample),
            total_peers: Some(20),
            public_node_count: Some(7),
            message: format!("sample {sample}"),
        };
        repository.record_remote_server_probe(&report).unwrap();
    }

    let latest = repository
        .latest_remote_server_probe(&profile.id)
        .unwrap()
        .unwrap();
    assert_eq!(latest.status, RemoteProbeStatus::Degraded);
    assert_eq!(latest.message, "sample 2");
    let history = repository
        .list_remote_server_probes(&profile.id, 10)
        .unwrap();
    assert_eq!(history.len(), 3);
    assert_eq!(history[0].checked_at_unix, 1_800_000_002);

    let pruned = repository
        .prune_remote_server_probes_keep_recent_per_profile(2)
        .unwrap();
    assert_eq!(pruned, 1);
    assert_eq!(
        repository
            .list_remote_server_probes(&profile.id, 10)
            .unwrap()
            .len(),
        2
    );

    repository.delete_remote_server(&profile.id).unwrap();
    assert!(repository.list_remote_servers().unwrap().is_empty());
    assert!(repository
        .list_remote_server_probes(&profile.id, 10)
        .unwrap()
        .is_empty());
}
