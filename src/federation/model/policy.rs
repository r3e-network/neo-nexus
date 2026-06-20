use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteFederationMonitorPolicy {
    pub enabled: bool,
    pub interval_seconds: u64,
}

impl RemoteFederationMonitorPolicy {
    pub const MIN_INTERVAL_SECONDS: u64 = 30;
    pub const DEFAULT_INTERVAL_SECONDS: u64 = 120;
    pub const MAX_INTERVAL_SECONDS: u64 = 7_200;

    pub fn enabled_default() -> Self {
        Self {
            enabled: true,
            interval_seconds: Self::DEFAULT_INTERVAL_SECONDS,
        }
    }

    pub fn interval_duration(self) -> Duration {
        Duration::from_secs(self.interval_seconds)
    }

    pub fn normalized(self) -> Self {
        Self {
            enabled: self.enabled,
            interval_seconds: self
                .interval_seconds
                .clamp(Self::MIN_INTERVAL_SECONDS, Self::MAX_INTERVAL_SECONDS),
        }
    }

    pub fn validation_message(self) -> Option<&'static str> {
        if self.interval_seconds < Self::MIN_INTERVAL_SECONDS {
            return Some("Federation probe interval is too short");
        }
        if self.interval_seconds > Self::MAX_INTERVAL_SECONDS {
            return Some("Federation probe interval is too long");
        }
        None
    }

    pub fn describe(self) -> String {
        if self.enabled {
            format!("enabled every {}s", self.interval_seconds)
        } else {
            format!("disabled; interval {}s", self.interval_seconds)
        }
    }
}
