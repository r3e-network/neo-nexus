//! Metrics exporter adapters: one per node type, describing where the node
//! publishes Prometheus metrics and how its exporter sidecar is configured.

use anyhow::Result;

use crate::types::NodeConfig;

pub const EXPORTER_PROMETHEUS_NET: &str = "prometheus-net-adapter";
pub const EXPORTER_PROMETHEUS: &str = "prometheus-exporter";

/// Trait for type-specific metrics exporter configuration and normalization.
#[allow(dead_code)]
pub trait MetricsExporterAdapter: std::fmt::Debug + Send + Sync {
    /// Returns the package identifier for the metrics exporter.
    /// Returns None if metrics are built-in to the node type.
    fn exporter_package(&self) -> Option<&'static str>;

    /// Returns the full HTTP URL at which this node exposes its metrics, given
    /// the node's configured RPC port. Returns None if this adapter has no HTTP
    /// metrics endpoint (e.g. tokio-console-only runtimes).
    fn metrics_url(&self, rpc_port: u16) -> Option<String>;

    /// Generate configuration for the metrics exporter sidecar process.
    fn generate_config(&self, node: &NodeConfig) -> Result<Vec<u8>>;

    /// Normalize raw metrics from any exporter into a consistent text format.
    fn normalize_metrics(&self, raw: &[u8]) -> Result<String> {
        Ok(String::from_utf8_lossy(raw).to_string())
    }
}

#[derive(Debug, Clone)]
pub struct NeoCliMetricsAdapter;

impl MetricsExporterAdapter for NeoCliMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        Some(EXPORTER_PROMETHEUS_NET)
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        // neo-cli prometheus-net-adapter always listens on :9090 regardless of rpc_port
        Some(String::from("http://localhost:9090/metrics"))
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        Ok(r#"{
            "listen_address": ":9090",
            "path": "/metrics",
            "namespace": "neo_cli"
        }"#
        .as_bytes()
        .to_vec())
    }
}

#[derive(Debug, Clone)]
pub struct NeoGoMetricsAdapter;

impl MetricsExporterAdapter for NeoGoMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        Some(EXPORTER_PROMETHEUS)
    }

    fn metrics_url(&self, rpc_port: u16) -> Option<String> {
        if rpc_port == 0 {
            return None;
        }
        // neo-go exposes metrics on the same port as RPC
        Some(format!("http://localhost:{rpc_port}/metrics"))
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        Ok(r#"{
            "listen_address": ":8090",
            "path": "/metrics",
            "namespace": "neo_go"
        }"#
        .as_bytes()
        .to_vec())
    }
}

#[derive(Debug, Clone)]
pub struct NeoRsMetricsAdapter;

impl MetricsExporterAdapter for NeoRsMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        None // neo-rs uses tokio-console + native metrics
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        None // no HTTP prometheus endpoint
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}

#[derive(Debug, Clone)]
pub struct NeoXGethMetricsAdapter;

impl MetricsExporterAdapter for NeoXGethMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        None // neox-geth inherits ethereum geth metrics
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        // geth exposes metrics on a fixed port separate from RPC
        Some(String::from("http://localhost:8546/metrics"))
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        Ok(r#"{
            "metrics_enabled": true,
            "metrics_endpoint": ":8546",
            "metrics_namespace": "neox_geth"
        }"#
        .as_bytes()
        .to_vec())
    }
}

#[derive(Debug, Clone)]
pub struct NeoXRethMetricsAdapter;

impl MetricsExporterAdapter for NeoXRethMetricsAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        None // neox-rs uses Reth's native metrics
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        // reth exposes metrics on a fixed port separate from RPC
        Some(String::from("http://localhost:9091/metrics"))
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        Ok(r#"{
            "metrics_enabled": true,
            "metrics_port": 9091,
            "metrics_namespace": "neox_reth"
        }"#
        .as_bytes()
        .to_vec())
    }
}

#[derive(Debug, Default)]
pub struct NoOpMetricsExporterAdapter;

impl MetricsExporterAdapter for NoOpMetricsExporterAdapter {
    fn exporter_package(&self) -> Option<&'static str> {
        None
    }

    fn metrics_url(&self, _rpc_port: u16) -> Option<String> {
        None
    }

    fn generate_config(&self, _node: &NodeConfig) -> Result<Vec<u8>> {
        Ok(vec![])
    }
}
