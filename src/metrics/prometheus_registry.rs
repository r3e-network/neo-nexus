/// Global Prometheus metric registry singleton for NeoNexus observability.
/// This module provides thread-safe access to metrics across all components.

use std::sync::{Arc, OnceLock};

use prometheus::{Gauge, Histogram, Registry, Counter, GaugeVec, CounterVec};

/// Global metric registry instance (lazy initialized)
static REGISTRY: OnceLock<Registry> = OnceLock::new();

/// Node status gauge - indicates whether each node is operational
pub static NODE_UP_GAUGE: OnceLock<Gauge> = OnceLock::new();

/// Event total counter - tracks event journal statistics by kind and severity
pub static EVENTS_TOTAL_COUNTER: OnceLock<CounterVec> = OnceLock::new();

/// Web request total counter - counts requests per web handler endpoint
pub static WEB_REQUESTS_TOTAL_COUNTER: OnceLock<CounterVec> = OnceLock::new();

/// Disk usage gauge - monitors backup storage usage per path
pub static DISK_USAGE_BYTES_GAUGE: OnceLock<GaugeVec> = OnceLock::new();

/// Sync progress histogram - measures time from last RPC call to current state update
pub static SYNC_PROGRESS_HISTOGRAM: OnceLock<Histogram> = OnceLock::new();

/// Initialize the global registry with all metrics
pub fn init_metrics() -> anyhow::Result<()> {
    let registry = Registry::new();
    
    // Register the registry first
    REGISTRY.set(registry.clone()).expect("Registry already initialized");
    
    // Register node status gauge
    let node_up = Gauge::new(
        "neo_nexus_node_up",
        "Whether the node is operational"
    )?;
    NODE_UP_GAUGE
        .set(node_up.clone())
        .expect("NodeUpGauge already set");
    registry.register(Box::new(node_up))?;
    
    // Register event total counter
    let events_total = CounterVec::new(
        prometheus::opts!(
            "neo_nexus_events_total",
            "Total events processed in runtime journal"
        ),
        &["kind", "severity", "chain"]
    )?;
    EVENTS_TOTAL_COUNTER
        .set(events_total.clone())
        .expect("EventsTotalCounter already set");
    registry.register(Box::new(events_total))?;
    
    // Register web requests counter
    let web_requests = CounterVec::new(
        prometheus::opts!(
            "neo_nexus_web_requests_total",
            "HTTP requests handled by web server"
        ),
        &["handler", "method", "status_code"]
    )?;
    WEB_REQUESTS_TOTAL_COUNTER
        .set(web_requests.clone())
        .expect("WebRequestsTotalCounter already set");
    registry.register(Box::new(web_requests))?;
    
    // Register disk usage gauge
    let disk_usage = GaugeVec::new(
        prometheus::opts!(
            "neo_nexus_disk_usage_bytes",
            "Current disk space used in bytes per path"
        ),
        &["path"]
    )?;
    DISK_USAGE_BYTES_GAUGE
        .set(disk_usage.clone())
        .expect("DiskUsageBytesGauge already set");
    registry.register(Box::new(disk_usage))?;
    
    // Register sync progress histogram
    let sync_progress = Histogram::with_opts(
        prometheus::HistogramOpts::new(
            "neo_nexus_sync_progress_seconds",
            "Time from last RPC call to current state update"
        )
        .buckets(vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0, 60.0])
    )?;
    SYNC_PROGRESS_HISTOGRAM
        .set(sync_progress.clone())
        .expect("SyncProgressHistogram already set");
    registry.register(Box::new(sync_progress))?;
    
    Ok(())
}

/// Get the global registry instance
pub fn get_registry() -> &'static Registry {
    REGISTRY.get().expect("Metrics registry not initialized")
}

/// Increment event counter for specific kind/severity/chain
pub fn increment_event(kind: &str, severity: &str, chain: &str) {
    if let Some(counter) = EVENTS_TOTAL_COUNTER.get() {
        let labels = counter.with_label_values(&[kind, severity, chain]);
        labels.inc();
    }
}

/// Set node up/down status
pub fn set_node_up(is_up: bool) {
    if let Some(gauge) = NODE_UP_GAUGE.get() {
        gauge.set(if is_up { 1.0 } else { 0.0 });
    }
}

/// Record web request
pub fn record_web_request(handler: &str, method: &str, status_code: u16) {
    if let Some(counter) = WEB_REQUESTS_TOTAL_COUNTER.get() {
        let labels = counter.with_label_values(&[handler, method, &status_code.to_string()]);
        labels.inc();
    }
}

/// Set disk usage for a path
pub fn set_disk_usage(path: &str, bytes: u64) {
    if let Some(gauge) = DISK_USAGE_BYTES_GAUGE.get() {
        gauge.with_label_values(&[path]).set(bytes as f64);
    }
}

/// Record sync progress duration
pub fn record_sync_progress(seconds: f64) {
    if let Some(histogram) = SYNC_PROGRESS_HISTOGRAM.get() {
        histogram.observe(seconds);
    }
}
