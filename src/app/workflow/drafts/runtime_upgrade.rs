use crate::runtime::RuntimeUpgradePolicy;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::app) struct RuntimeUpgradePolicyDraft {
    pub(in crate::app) enabled: bool,
    pub(in crate::app) catalog_profile_id: Option<String>,
    pub(in crate::app) interval_minutes: u64,
    pub(in crate::app) require_signed_catalog: bool,
    pub(in crate::app) max_nodes_per_run: usize,
    pub(in crate::app) maintenance_window_enabled: bool,
    pub(in crate::app) maintenance_window_start_minute_utc: u16,
    pub(in crate::app) maintenance_window_end_minute_utc: u16,
    pub(in crate::app) wave_delay_minutes: u64,
}

impl RuntimeUpgradePolicyDraft {
    pub(in crate::app) fn from_policy(policy: &RuntimeUpgradePolicy) -> Self {
        Self {
            enabled: policy.enabled,
            catalog_profile_id: policy.catalog_profile_id.clone(),
            interval_minutes: policy.interval_minutes,
            require_signed_catalog: policy.require_signed_catalog,
            max_nodes_per_run: policy.max_nodes_per_run,
            maintenance_window_enabled: policy.maintenance_window_enabled,
            maintenance_window_start_minute_utc: policy.maintenance_window_start_minute_utc,
            maintenance_window_end_minute_utc: policy.maintenance_window_end_minute_utc,
            wave_delay_minutes: policy.wave_delay_minutes,
        }
    }

    pub(in crate::app) fn to_policy(&self, active: &RuntimeUpgradePolicy) -> RuntimeUpgradePolicy {
        RuntimeUpgradePolicy {
            enabled: self.enabled,
            catalog_profile_id: self.catalog_profile_id.clone(),
            interval_minutes: self.interval_minutes,
            require_signed_catalog: self.require_signed_catalog,
            max_nodes_per_run: self.max_nodes_per_run,
            maintenance_window_enabled: self.maintenance_window_enabled,
            maintenance_window_start_minute_utc: self.maintenance_window_start_minute_utc,
            maintenance_window_end_minute_utc: self.maintenance_window_end_minute_utc,
            wave_delay_minutes: self.wave_delay_minutes,
            last_checked_at_unix: active.last_checked_at_unix,
            last_applied_at_unix: active.last_applied_at_unix,
        }
    }

    pub(in crate::app) fn validation_message(&self) -> Option<&'static str> {
        if self.enabled
            && self
                .catalog_profile_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
        {
            return Some("Enabled runtime upgrade policy needs a catalog profile");
        }
        if self.interval_minutes < RuntimeUpgradePolicy::MIN_INTERVAL_MINUTES {
            return Some("Runtime upgrade interval is below the safe minimum");
        }
        if self.interval_minutes > RuntimeUpgradePolicy::MAX_INTERVAL_MINUTES {
            return Some("Runtime upgrade interval is above the supported maximum");
        }
        if self.max_nodes_per_run == 0 {
            return Some("Runtime upgrade policy must allow at least one node per run");
        }
        if self.max_nodes_per_run > RuntimeUpgradePolicy::MAX_NODES_PER_RUN {
            return Some("Runtime upgrade policy batch size is too large");
        }
        if self.maintenance_window_enabled {
            if self.maintenance_window_start_minute_utc >= RuntimeUpgradePolicy::MINUTES_PER_DAY
                || self.maintenance_window_end_minute_utc >= RuntimeUpgradePolicy::MINUTES_PER_DAY
            {
                return Some("Runtime upgrade maintenance window minute is out of range");
            }
            if self.maintenance_window_start_minute_utc == self.maintenance_window_end_minute_utc {
                return Some("Runtime upgrade maintenance window must have duration");
            }
        }
        if self.wave_delay_minutes > RuntimeUpgradePolicy::MAX_WAVE_DELAY_MINUTES {
            return Some("Runtime upgrade wave delay is above the supported maximum");
        }
        None
    }

    pub(in crate::app) fn differs_from(&self, policy: &RuntimeUpgradePolicy) -> bool {
        self != &Self::from_policy(policy)
    }
}
