use super::RuntimeUpgradePolicy;

impl RuntimeUpgradePolicy {
    pub const MIN_INTERVAL_MINUTES: u64 = 15;
    pub const MAX_INTERVAL_MINUTES: u64 = 7 * 24 * 60;
    pub const DEFAULT_INTERVAL_MINUTES: u64 = 24 * 60;
    pub const DEFAULT_MAX_NODES_PER_RUN: usize = 3;
    pub const MAX_NODES_PER_RUN: usize = 50;
    pub const MINUTES_PER_DAY: u16 = 24 * 60;
    pub const MAX_WAVE_DELAY_MINUTES: u64 = 7 * 24 * 60;

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            catalog_profile_id: None,
            interval_minutes: Self::DEFAULT_INTERVAL_MINUTES,
            require_signed_catalog: true,
            max_nodes_per_run: Self::DEFAULT_MAX_NODES_PER_RUN,
            maintenance_window_enabled: false,
            maintenance_window_start_minute_utc: 0,
            maintenance_window_end_minute_utc: 6 * 60,
            wave_delay_minutes: 0,
            last_checked_at_unix: None,
            last_applied_at_unix: None,
        }
    }
}
