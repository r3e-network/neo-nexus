use super::RemoteServerProfile;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoteServerProfileFilter {
    pub enabled: Option<bool>,
    pub query: String,
}

impl RemoteServerProfileFilter {
    pub fn new(enabled: Option<bool>, query: impl Into<String>) -> Self {
        Self {
            enabled,
            query: query.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.query.trim().is_empty()
    }
}

pub fn filter_remote_server_profiles(
    profiles: &[RemoteServerProfile],
    filter: &RemoteServerProfileFilter,
) -> Vec<RemoteServerProfile> {
    let query = filter.query.trim().to_lowercase();
    profiles
        .iter()
        .filter(|profile| {
            filter
                .enabled
                .is_none_or(|enabled| profile.enabled == enabled)
        })
        .filter(|profile| query.is_empty() || profile_matches(profile, &query))
        .cloned()
        .collect()
}

fn profile_matches(profile: &RemoteServerProfile, query: &str) -> bool {
    text_matches(&profile.id, query)
        || text_matches(&profile.name, query)
        || text_matches(&profile.base_url, query)
        || text_matches(&profile.description, query)
        || text_matches(enabled_label(profile.enabled), query)
        || text_matches(&profile.created_at_unix.to_string(), query)
        || text_matches(&profile.updated_at_unix.to_string(), query)
}

fn enabled_label(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
mod tests {
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

    fn assert_ids(
        profiles: &[RemoteServerProfile],
        filter: RemoteServerProfileFilter,
        ids: &[&str],
    ) {
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
}
