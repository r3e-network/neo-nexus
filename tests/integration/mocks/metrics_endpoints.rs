//! Mock HTTP endpoints for metrics collection simulation
//!
//! Provides realistic Prometheus-format metric servers and timeout scenarios
//! for testing NodeManager's metrics adapters across different node types.

use anyhow::Result;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

/// Mock metrics server that returns Prometheus-format data
pub struct MockMetricsServer {
    addr: SocketAddr,
    handle: JoinHandle<()>,
}

impl MockMetricsServer {
    /// Create a mock server with predefined metric responses
    pub async fn spawn(mock_data: MockMetricData) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let mock_data = Arc::new(Mutex::new(mock_data));

        let handle = tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let data = Arc::clone(&mock_data);
                        tokio::spawn(async move {
                            Self::handle_connection(stream, data).await;
                        });
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self { addr, handle })
    }

    /// Handle incoming HTTP connection
    async fn handle_connection(
        mut stream: tokio::net::TcpStream,
        data: Arc<Mutex<MockMetricData>>,
    ) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        // Read the request head; HTTP clients keep the write side open, so a
        // read_to_end here would hang forever waiting for EOF.
        let mut buf = vec![0u8; 4096];
        let n = match stream.read(&mut buf).await {
            Ok(n) => n,
            Err(_) => return,
        };

        let request = String::from_utf8_lossy(&buf[..n]).to_string();
        let is_metrics = request.contains("/metrics");

        // Copy the payload out before any await point - the MutexGuard is not
        // Send and must not be held across an await.
        let (content, delay_ms) = {
            let guard = data.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            (guard.content.clone(), guard.error_delay_ms)
        };

        if let Some(delay) = delay_ms {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }

        let response = if is_metrics {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                content.len(),
                content
            )
        } else {
            "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found"
                .to_string()
        };

        let _ = stream.write_all(response.as_bytes()).await;
        let _ = stream.shutdown().await;
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub async fn shutdown(self) {
        self.handle.abort();
    }
}

/// Structured mock metric data
#[derive(Debug, Clone)]
pub struct MockMetricData {
    pub content: String,
    pub error_delay_ms: Option<u64>, // Simulate slow response
}

impl MockMetricData {
    /// Create NeoCli-style metrics
    pub fn neo_cli_metrics() -> Self {
        Self {
            content: r#"# HELP neo_block_height Current blockchain height
# TYPE neo_block_height gauge
neo_block_height{node="test-node"} 2847392
# HELP neo_peers_connected Number of connected peers
# TYPE neo_peers_connected gauge
neo_peers_connected{node="test-node"} 42
# HELP neo_sync_progress Sync progress percentage
# TYPE neo_sync_progress gauge
neo_sync_progress{node="test-node"} 98.5
"#
            .to_string(),
            error_delay_ms: None,
        }
    }

    /// Create NeoGo-style metrics
    pub fn neo_go_metrics() -> Self {
        Self {
            content: r#"# HELP go_gc_duration_seconds A summary of the GC invocation durations.
# TYPE go_gc_duration_seconds summary
go_gc_duration_seconds{quantile="0"} 2.3e-06
go_gc_duration_seconds_sum 2.3e-06
go_gc_duration_seconds_count 1
# HELP neo_node_height Current block height
# TYPE neo_node_height gauge
neo_node_height 2847395
# HELP neo_peer_count Number of peers
# TYPE neo_peer_count gauge
neo_peer_count 38
"#
            .to_string(),
            error_delay_ms: None,
        }
    }

    /// Create NeoRs-style metrics
    pub fn neo_rs_metrics() -> Self {
        Self {
            content: r#"# TYPE rust_memory_usage_gauge gauge
rust_memory_usage_gauge 419430400
# TYPE rust_processor_threads gauge
rust_processor_threads 16
# HELP neox_node_block_height Block height
# TYPE neox_node_block_height gauge
neox_node_block_height 523847
# HELP neox_active_connections Active peer connections
# TYPE neox_active_connections gauge
neox_active_connections 24
"#
            .to_string(),
            error_delay_ms: None,
        }
    }

    /// Create metrics with simulated delay (for timeout testing)
    pub fn delayed_response(delay_ms: u64) -> Self {
        Self {
            content: "# DELAYED_RESPONSE\n".to_string(),
            error_delay_ms: Some(delay_ms),
        }
    }

    /// Create metrics with empty response (edge case)
    pub fn empty_metrics() -> Self {
        Self {
            content: String::new(),
            error_delay_ms: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_neo_cli_metrics_server() {
        let mock = MockMetricData::neo_cli_metrics();
        let server = MockMetricsServer::spawn(mock).await.unwrap();

        let client = reqwest::Client::new();
        let url = format!("http://{}/metrics", server.addr());

        let response = client.get(&url).send().await.unwrap();
        let body = response.text().await.unwrap();

        assert!(body.contains("neo_block_height"));
        assert!(body.contains("2847392"));

        server.shutdown().await;
    }

    #[tokio::test]
    async fn test_neo_go_metrics_server() {
        let mock = MockMetricData::neo_go_metrics();
        let server = MockMetricsServer::spawn(mock).await.unwrap();

        let client = reqwest::Client::new();
        let url = format!("http://{}/metrics", server.addr());

        let response = client.get(&url).send().await.unwrap();
        assert!(response.status().is_success());

        let body = response.text().await.unwrap();
        assert!(body.contains("neo_node_height"));
        assert!(body.contains("neo_peer_count"));

        server.shutdown().await;
    }

    #[tokio::test]
    async fn test_empty_metrics_response() {
        let mock = MockMetricData::empty_metrics();
        let server = MockMetricsServer::spawn(mock).await.unwrap();

        let client = reqwest::Client::new();
        let url = format!("http://{}/metrics", server.addr());

        let response = client.get(&url).send().await.unwrap();
        assert!(response.status().is_success());

        let body = response.text().await.unwrap();
        assert!(body.is_empty());

        server.shutdown().await;
    }

    #[tokio::test]
    async fn test_non_metrics_path_returns_404() {
        let mock = MockMetricData::neo_cli_metrics();
        let server = MockMetricsServer::spawn(mock).await.unwrap();

        let client = reqwest::Client::new();
        let url = format!("http://{}/health", server.addr());

        let response = client.get(&url).send().await.unwrap();
        assert_eq!(response.status().as_u16(), 404);

        server.shutdown().await;
    }

    #[tokio::test]
    async fn test_multiple_requests() {
        let mock = MockMetricData::neo_rs_metrics();
        let server = MockMetricsServer::spawn(mock).await.unwrap();

        let client = reqwest::Client::new();
        let url = format!("http://{}/metrics", server.addr());

        // Multiple concurrent requests
        let futures = (0..5).map(|_| {
            let client = client.clone();
            let url = url.clone();
            async move {
                let response = client.get(&url).send().await?;
                response.text().await
            }
        });

        let results = futures::future::join_all(futures).await;

        for result in results {
            let body = result.unwrap();
            assert!(body.contains("rust_memory_usage_gauge"));
        }

        server.shutdown().await;
    }
}
