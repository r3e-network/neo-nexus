mod defaults;
mod labels;
mod schedule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeUpgradePolicy {
    pub enabled: bool,
    pub catalog_profile_id: Option<String>,
    pub interval_minutes: u64,
    pub require_signed_catalog: bool,
    pub max_nodes_per_run: usize,
    pub maintenance_window_enabled: bool,
    pub maintenance_window_start_minute_utc: u16,
    pub maintenance_window_end_minute_utc: u16,
    pub wave_delay_minutes: u64,
    pub last_checked_at_unix: Option<u64>,
    pub last_applied_at_unix: Option<u64>,
}
