use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct NodeProcessMetrics {
    pub node_id: String,
    pub node_name: String,
    pub pid: u32,
    pub cpu_usage_percent: f32,
    pub memory_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub run_time_seconds: u64,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MissingProcessMetric {
    pub node_id: String,
    pub node_name: String,
    pub pid: u32,
}
