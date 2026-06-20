use super::RuntimeUpgradePolicy;

impl RuntimeUpgradePolicy {
    pub fn interval_seconds(&self) -> u64 {
        self.interval_minutes.saturating_mul(60)
    }

    pub fn wave_delay_seconds(&self) -> u64 {
        self.wave_delay_minutes.saturating_mul(60)
    }

    pub fn is_due(&self, now_unix: u64) -> bool {
        if !self.enabled {
            return false;
        }
        if !self.is_in_maintenance_window(now_unix) {
            return false;
        }
        if self.wave_delay_minutes > 0 {
            if let Some(last_applied) = self.last_applied_at_unix {
                if now_unix < last_applied.saturating_add(self.wave_delay_seconds()) {
                    return false;
                }
            }
        }
        let Some(last_checked) = self.last_checked_at_unix else {
            return true;
        };
        now_unix >= last_checked.saturating_add(self.interval_seconds())
    }

    pub fn is_in_maintenance_window(&self, now_unix: u64) -> bool {
        if !self.maintenance_window_enabled {
            return true;
        }

        let minute = ((now_unix / 60) % u64::from(Self::MINUTES_PER_DAY)) as u16;
        let start = self.maintenance_window_start_minute_utc;
        let end = self.maintenance_window_end_minute_utc;
        if start <= end {
            minute >= start && minute < end
        } else {
            minute >= start || minute < end
        }
    }

    pub fn with_checked_at(&self, checked_at_unix: u64) -> Self {
        let mut updated = self.clone();
        updated.last_checked_at_unix = Some(checked_at_unix);
        updated
    }

    pub fn with_applied_at(&self, applied_at_unix: u64) -> Self {
        let mut updated = self.with_checked_at(applied_at_unix);
        updated.last_applied_at_unix = Some(applied_at_unix);
        updated
    }
}
