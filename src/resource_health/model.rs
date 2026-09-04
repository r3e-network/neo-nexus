use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const MIB: u64 = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ResourcePolicy {
    pub enabled: bool,
    pub interval_seconds: u64,
    pub disk_warning_mib: u64,
    pub disk_critical_mib: u64,
    pub memory_warning_percent: u8,
    pub memory_critical_percent: u8,
    pub storage_paths: Vec<PathBuf>,
}

impl Default for ResourcePolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 30,
            disk_warning_mib: 5120,
            disk_critical_mib: 1024,
            memory_warning_percent: 10,
            memory_critical_percent: 5,
            storage_paths: vec![],
        }
    }
}

impl ResourcePolicy {
    pub fn validate(&self) -> Result<()> {
        if !(10..=3600).contains(&self.interval_seconds) {
            bail!("resource interval must be between 10 and 3600 seconds");
        }
        if self.disk_critical_mib == 0
            || self.disk_warning_mib <= self.disk_critical_mib
            || self.disk_warning_mib > 1024 * 1024 * 1024
        {
            bail!("disk warning must exceed a positive critical threshold (MiB)");
        }
        if self.memory_critical_percent == 0
            || self.memory_warning_percent <= self.memory_critical_percent
            || self.memory_warning_percent > 90
        {
            bail!("memory thresholds must satisfy 0 < critical < warning <= 90 percent available");
        }
        if self.storage_paths.len() > 16
            || self.storage_paths.iter().any(|path| {
                !path.is_absolute()
                    || path.as_os_str().len() > 4096
                    || path.to_string_lossy().chars().any(char::is_control)
            })
        {
            bail!("provide up to 16 absolute storage directory paths without control characters");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResourceStatus {
    Healthy,
    Warning,
    Critical,
    #[default]
    Unknown,
}

impl ResourceStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Healthy => "Healthy",
            Self::Warning => "Warning",
            Self::Critical => "Critical",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReading {
    pub id: String,
    pub label: String,
    pub capacity_bytes: Option<u64>,
    pub available_bytes: Option<u64>,
    pub available_inodes: Option<u64>,
    pub status: ResourceStatus,
    pub message: String,
    pub confirmed_status: ResourceStatus,
    pub pending_samples: u8,
    pub alerted: bool,
}

impl ResourceReading {
    pub(super) fn unknown(id: String, label: String, message: &str) -> Self {
        Self {
            id,
            label,
            capacity_bytes: None,
            available_bytes: None,
            available_inodes: None,
            status: ResourceStatus::Unknown,
            message: message.into(),
            confirmed_status: ResourceStatus::Unknown,
            pending_samples: 0,
            alerted: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceReport {
    pub checked_at_unix: u64,
    pub policy: ResourcePolicy,
    pub readings: Vec<ResourceReading>,
    pub sample_failed: bool,
}

impl ResourceReport {
    pub fn is_fresh(&self, now: u64) -> bool {
        self.policy.enabled
            && !self.sample_failed
            && self.checked_at_unix <= now.saturating_add(5)
            && now.saturating_sub(self.checked_at_unix)
                <= self
                    .policy
                    .interval_seconds
                    .saturating_mul(2)
                    .saturating_add(15)
    }
}
