use super::RuntimeUpgradePolicy;

impl RuntimeUpgradePolicy {
    pub fn describe(&self) -> String {
        if !self.enabled {
            return "disabled".to_string();
        }

        let window = if self.maintenance_window_enabled {
            format!(", window {}", self.maintenance_window_label())
        } else {
            String::new()
        };
        let wave = if self.wave_delay_minutes > 0 {
            format!(", wave delay {} min", self.wave_delay_minutes)
        } else {
            String::new()
        };

        format!(
            "every {} min, max {} stopped nodes{window}{wave}, {} catalog",
            self.interval_minutes,
            self.max_nodes_per_run,
            if self.require_signed_catalog {
                "signed"
            } else {
                "signed/local"
            }
        )
    }

    pub fn maintenance_window_label(&self) -> String {
        if !self.maintenance_window_enabled {
            return "any time".to_string();
        }
        format!(
            "{}-{} UTC",
            format_utc_minute(self.maintenance_window_start_minute_utc),
            format_utc_minute(self.maintenance_window_end_minute_utc)
        )
    }
}

fn format_utc_minute(minute: u16) -> String {
    format!("{:02}:{:02}", minute / 60, minute % 60)
}
