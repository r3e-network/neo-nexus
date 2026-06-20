use serde::Serialize;

use super::ResourcePressure;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SystemMetrics {
    pub cpu_usage_percent: f32,
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub memory_usage_percent: f32,
    pub process_count: usize,
}

impl SystemMetrics {
    pub fn memory_pressure(&self) -> ResourcePressure {
        ResourcePressure::from_percent(self.memory_usage_percent)
    }

    pub fn cpu_pressure(&self) -> ResourcePressure {
        ResourcePressure::from_percent(self.cpu_usage_percent)
    }
}
