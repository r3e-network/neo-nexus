use super::NeoWalletProfile;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NeoWalletProfileFilter {
    pub used: Option<bool>,
    pub query: String,
}

impl NeoWalletProfileFilter {
    pub fn new(used: Option<bool>, query: impl Into<String>) -> Self {
        Self {
            used,
            query: query.into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.used.is_none() && self.query.trim().is_empty()
    }
}

pub fn filter_neo_wallet_profiles(
    profiles: &[NeoWalletProfile],
    filter: &NeoWalletProfileFilter,
) -> Vec<NeoWalletProfile> {
    let query = filter.query.trim().to_lowercase();
    profiles
        .iter()
        .filter(|profile| {
            filter
                .used
                .is_none_or(|used| profile_is_used(profile) == used)
        })
        .filter(|profile| query.is_empty() || profile_matches(profile, &query))
        .cloned()
        .collect()
}

fn profile_is_used(profile: &NeoWalletProfile) -> bool {
    profile.last_used_at_unix.is_some()
}

fn profile_matches(profile: &NeoWalletProfile, query: &str) -> bool {
    text_matches(&profile.id, query)
        || text_matches(&profile.label, query)
        || text_matches(&profile.source_path, query)
        || text_matches(profile.wallet_version.as_deref().unwrap_or_default(), query)
        || text_matches(&profile.primary_address, query)
        || profile
            .contract_public_keys
            .iter()
            .any(|public_key| text_matches(public_key, query))
        || text_matches(&profile.wallet_sha256, query)
        || text_matches(&profile.account_count.to_string(), query)
        || text_matches(&profile.encrypted_account_count.to_string(), query)
        || text_matches(&profile.default_account_count.to_string(), query)
        || text_matches(&profile.watch_only_account_count.to_string(), query)
        || text_matches(used_label(profile_is_used(profile)), query)
}

fn used_label(used: bool) -> &'static str {
    if used {
        "used"
    } else {
        "unused"
    }
}

fn text_matches(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(query)
}

#[cfg(test)]
mod tests;
