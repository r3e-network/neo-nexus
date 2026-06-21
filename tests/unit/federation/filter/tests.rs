use super::*;

#[test]
fn remote_profile_filter_matches_operational_fields() {
    let profiles = [
        profile(
            "ops-main",
            "Ops Main",
            "https://ops.example",
            "primary",
            true,
        ),
        profile(
            "lab-backup",
            "Lab Backup",
            "https://lab.example",
            "standby",
            false,
        ),
    ];

    assert_ids(
        &profiles,
        RemoteServerProfileFilter::new(None, "ops.example"),
        &["ops-main"],
    );
    assert_ids(
        &profiles,
        RemoteServerProfileFilter::new(None, "standby"),
        &["lab-backup"],
    );
    assert_ids(
        &profiles,
        RemoteServerProfileFilter::new(None, "disabled"),
        &["lab-backup"],
    );
}

#[test]
fn remote_profile_filter_combines_enabled_and_query() {
    let profiles = [
        profile("ops-main", "Ops Main", "https://ops.example", "main", true),
        profile("ops-lab", "Ops Lab", "https://lab.example", "lab", false),
        profile("seed-lab", "Seed Lab", "https://seed.example", "lab", true),
    ];
    let filter = RemoteServerProfileFilter::new(Some(true), "lab");

    assert_ids(&profiles, filter, &["seed-lab"]);
}

fn assert_ids(profiles: &[RemoteServerProfile], filter: RemoteServerProfileFilter, ids: &[&str]) {
    let filtered = filter_remote_server_profiles(profiles, &filter);
    let actual = filtered
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(actual.as_slice(), ids);
}

fn profile(
    id: &str,
    name: &str,
    base_url: &str,
    description: &str,
    enabled: bool,
) -> RemoteServerProfile {
    RemoteServerProfile {
        id: id.to_string(),
        name: name.to_string(),
        base_url: base_url.to_string(),
        description: description.to_string(),
        enabled,
        created_at_unix: 1_800_000_000,
        updated_at_unix: 1_800_000_100,
    }
}
