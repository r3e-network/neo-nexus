use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use sysinfo::{Pid, System};

use crate::types::{NodeConfig, NodeStatus};

/// How long a one-shot caller waits between its two readings.
///
/// Comfortably above `sysinfo`'s 200 ms `MINIMUM_CPU_UPDATE_INTERVAL`, below
/// which a refresh is discarded and the CPU delta is never recomputed.
const CPU_SETTLE: Duration = Duration::from_millis(250);

use super::{
    formatter::{clean_usage_percent, percent},
    types::{MetricsSnapshot, MissingProcessMetric, NodeProcessMetrics, SystemMetrics},
};

pub struct MetricsCollector {
    system: System,
    refresh_interval: Duration,
    last_refresh: Option<Instant>,
}

impl MetricsCollector {
    /// `System::new_all` already takes a full reading, so this deliberately
    /// does **not** refresh again. `sysinfo` refuses to recompute CPU inside its
    /// 200 ms minimum interval, so a second refresh here would be discarded and
    /// the first `refresh` call would return the constructor's sample — which is
    /// the since-boot average on Linux and effectively zero on macOS, and is
    /// exactly why every CPU figure in this product used to be inert.
    pub fn new(refresh_interval: Duration) -> Self {
        let system = System::new_all();
        Self {
            system,
            refresh_interval,
            last_refresh: None,
        }
    }

    pub fn refresh(&mut self, nodes: &[NodeConfig], now: Instant) -> MetricsSnapshot {
        self.system.refresh_all();
        self.last_refresh = Some(now);
        snapshot_from_system(&self.system, nodes, unix_now())
    }

    /// Two readings, a real interval apart, for a caller with nowhere to keep a
    /// collector.
    ///
    /// `sysinfo` computes CPU as a delta between refreshes and discards any
    /// refresh inside its 200 ms minimum, so a single reading is not a CPU
    /// measurement at all — it is the since-boot average on Linux and zero on
    /// macOS. A one-shot CLI command or a support bundle pays a quarter of a
    /// second here and gets a figure that means something; a long-running
    /// server uses `MetricsStore` instead and pays nothing.
    pub fn sample_settled(nodes: &[NodeConfig]) -> MetricsSnapshot {
        let mut collector = Self::new(Duration::ZERO);
        std::thread::sleep(CPU_SETTLE);
        collector.refresh(nodes, Instant::now())
    }

    pub fn refresh_if_due(
        &mut self,
        nodes: &[NodeConfig],
        now: Instant,
    ) -> Option<MetricsSnapshot> {
        if self
            .last_refresh
            .is_none_or(|last_refresh| now.duration_since(last_refresh) >= self.refresh_interval)
        {
            Some(self.refresh(nodes, now))
        } else {
            None
        }
    }
}

fn snapshot_from_system(
    system: &System,
    nodes: &[NodeConfig],
    captured_at_unix: u64,
) -> MetricsSnapshot {
    let total_memory_bytes = system.total_memory();
    let used_memory_bytes = system.used_memory();
    let system_metrics = SystemMetrics {
        cpu_usage_percent: clean_usage_percent(system.global_cpu_usage()),
        total_memory_bytes,
        used_memory_bytes,
        available_memory_bytes: system.available_memory(),
        memory_usage_percent: percent(used_memory_bytes, total_memory_bytes),
        process_count: system.processes().len(),
    };

    let mut node_processes = Vec::new();
    let mut missing_processes = Vec::new();

    for node in nodes
        .iter()
        .filter(|node| node.status == NodeStatus::Running)
    {
        let Some(pid) = node.pid else {
            continue;
        };
        if let Some(process) = system.process(Pid::from_u32(pid)) {
            node_processes.push(NodeProcessMetrics {
                node_id: node.id.clone(),
                node_name: node.name.clone(),
                pid,
                cpu_usage_percent: clean_usage_percent(process.cpu_usage()),
                memory_bytes: process.memory(),
                virtual_memory_bytes: process.virtual_memory(),
                run_time_seconds: process.run_time(),
                status: format!("{:?}", process.status()).to_ascii_lowercase(),
            });
        } else {
            missing_processes.push(MissingProcessMetric {
                node_id: node.id.clone(),
                node_name: node.name.clone(),
                pid,
            });
        }
    }

    MetricsSnapshot {
        captured_at_unix,
        system: system_metrics,
        node_processes,
        missing_processes,
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
