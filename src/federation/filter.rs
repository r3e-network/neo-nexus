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
mod tests;
