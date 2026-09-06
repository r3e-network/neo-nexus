# RFC: Prometheus Metrics Exporter for NeoNexus

**RFC Number:** 001  
**Status:** Draft  
**Created:** September 6, 2026  
**Author:** Qoder (Technical Strategy)  
**Target Version:** v4.2.0 (experimental feature flag)

---

## 📋 Executive Summary

This RFC proposes adding a Prometheus-compatible metrics exporter to NeoNexus to enable enterprise-grade observability. The implementation will expose real-time metrics about node health, resource utilization, event processing, and web interactions through HTTP endpoint `/metrics`.

### Key Benefits
- **Operational Visibility:** Real-time monitoring of node status and performance
- **Alerting Integration:** Native compatibility with Alertmanager for proactive monitoring
- **Dashboard Ecosystem:** Ready-to-use Grafana dashboards out-of-the-box
- **Kubernetes Native:** Supports liveness/readiness probes and custom metrics adapter

---

## 🎯 Requirements & Goals

### Must-Have Features (v4.2.0 MVP)
1. Export node lifecycle metrics (up/down status, sync progress)
2. Resource utilization metrics (CPU, memory, disk I/O)
3. Event journal statistics (total events, purge rate)
4. Web UI interaction counters (requests per handler)
5. JSON logging format with ELK stack compatibility
6. Feature-flagged behind `--experimental-metrics`

### Nice-to-Have (Future Versions)
7. Distributed tracing integration (OpenTelemetry)
8. Custom metrics adapter for Kubernetes HPA
9. Security metrics (authentication attempts, authorization violations)

### Out of Scope for v4.2.0
10. Metric aggregation or time-series database backend
11. Multi-tenant isolation for metrics
12. Historical trend analysis within NeoNexus

---

## 🏗️ Architecture Design

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────┐
│                      NeoNexus Core                          │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  Node    │  │ Event    │  │  Web     │  │ Backup   │   │
│  │ Lifecycle│  │ Journal  │  │  Server  │  │ System   │   │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘   │
│       │             │             │             │          │
│       └─────────────┴─────────────┴─────────────┘          │
│                           │                                │
│                   ┌───────▼────────┐                       │
│                   │  MetricsBridge │                       │
│                   │  (New Module)  │                       │
│                   └───────┬────────┘                       │
└───────────────────────────┼───────────────────────────────┘
                            ▼
              ┌────────────────────────────┐
              │ Prometheus Client Crate    │
              │ • Registry creation        │
              │ • Counter/Gauge/Histogram  │
              │ • Collector registration   │
              └──────────────┬─────────────┘
                             ▼
            ┌───────────────────────────────┐
            │ /metrics HTTP Endpoint (Port) │
            │ Port: 9090 (configurable)     │
            │ Content-Type: text/plain      │
            └───────────────────────────────┘
```

### Module Structure

**New crate structure:**
```rust
// src/metrics/
├── collector.rs         # Metric definitions and types
├── registry.rs          # Global metric registry singleton
├── bridge.rs            # Bridge between core modules and prometheus
├── endpoint.rs          # HTTP handler for /metrics endpoint
└── json_logger.rs       # Structured JSON logging formatter
```

**Integration points:**
```rust
// src/core/lifecycle.rs - add metrics collection
use crate::metrics::{bridge::MetricsBridge, counter::node_lifecycle_started};

pub fn start_node(...) {
    // Existing logic...
    
    // New: emit metric
    MetricsBridge::increment_counter(&node_lifecycle_started);
}
```

---

## 📊 Metric Schema Definition

### Core Node Metrics

#### 1. Node Status Gauge
```prometheus
neo_nexus_node_up{chain="cosmoshub", network="mainnet"} 1
neo_nexus_node_up{chain="osmo", network="testnet"} 0
```
**Labels:** `chain`, `network`  
**Meaning:** 1 = healthy and running, 0 = unhealthy/stopped

#### 2. Sync Progress Histogram
```prometheus
neo_nexus_sync_progress_seconds_bucket{le="0.1", chain="cosmoshub"} 124
neo_nexus_sync_progress_seconds_bucket{le="1.0", chain="cosmoshub"} 89
neo_nexus_sync_progress_seconds_bucket{le="+Inf", chain="cosmoshub"} 150
```
**Labels:** `chain`  
**Histogram:** Measures time from last RPC call to current state update

#### 3. Event Counters
```prometheus
neo_nexus_events_total{kind="lifecycle", severity="info", chain="cosmoshub"} 15234
neo_nexus_events_total{kind="health_check", severity="warning", chain="cosmoshub"} 23
neo_nexus_events_total{kind="config_change", severity="error", chain="cosmoshub"} 0
```
**Labels:** `kind`, `severity`, `chain`  
**Counter:** Total number of events in runtime journal

#### 4. Disk Usage Gauge
```prometheus
neo_nexus_disk_usage_bytes{/path="/var/lib/neonexus"} 5368709120
neo_nexus_disk_usage_bytes{/path="/var/lib/neonexus/data"} 2147483648
```
**Labels:** `path`  
**Gauge:** Current disk space used in bytes

#### 5. Web Request Counter
```prometheus
neo_nexus_web_requests_total{handler="node_start", method="POST", status_code="200"} 45
neo_nexus_web_requests_total{handler="node_stop", method="POST", status_code="403"} 3
```
**Labels:** `handler`, `method`, `status_code`  
**Counter:** Total requests per web handler endpoint

### Additional Metrics (v4.3.0+)

#### Snapshot Operations
```prometheus
neo_nexus_snapshot_duration_seconds{type="full", success="true"} 45.23
neo_nexus_snapshot_size_bytes{type="full", success="true"} 5368709120
neo_nexus_restore_duration_seconds{type="incremental", success="false", error="timeout"} 300.0
```

#### Backup Success Rate
```prometheus
neo_nexus_backup_success_rate{destination="local", window="1h"} 0.98
neo_nexus_backup_success_rate{destination="s3", window="1h"} 0.95
```

---

## 🔧 Implementation Strategy

### Phase 1: Foundation (Week 1-2)
1. Add `prometheus` crate dependency (`version = "0.13"`)
2. Create metrics module structure with empty collectors
3. Implement global registry initialization
4. Add basic counter/gauge wrappers

### Phase 2: Core Integration (Week 3-4)
1. Integrate node lifecycle metrics into `src/core/lifecycle.rs`
2. Add event journal statistics to `src/repository/events_health/events/prune.rs`
3. Hook up web request counters in `src/web/control.rs` and related handlers
4. Implement disk usage monitoring in backup system

### Phase 3: HTTP Endpoint (Week 5)
1. Add simple HTTP server using `hyper` or `actix-web`
2. Implement `/metrics` endpoint following OpenMetrics spec
3. Expose on configurable port (default 9090)
4. Add authentication token support for production security

### Phase 4: Testing & Documentation (Week 6)
1. Write unit tests for metric collection
2. Create integration test suite with mock nodes
3. Develop Grafana dashboard templates
4. Document configuration options and examples

### Feature Flag Design
```toml
# Cargo.toml
[features]
experimental-metrics = ["prometheus"]
```

```rust
// In main.rs or CLI parser
if config.experimental_metrics {
    MetricsBridge::init()?;
}
```

---

## ⚠️ Performance Considerations

### Expected Overhead
- **Memory:** ~2MB additional for metric registries (conservative estimate)
- **CPU:** < 1% overhead from metric collection calls
- **Network:** Minimal (metric endpoint only accessed by monitoring systems)
- **Disk:** No persistent storage for metrics themselves

### Optimization Strategies
1. Use atomic operations for counters (zero lock contention)
2. Batch histogram updates to reduce mutex contention
3. Limit label cardinality to prevent exponential growth
4. Implement automatic metric garbage collection for stale labels

### Cardinality Limits
**Maximum unique label combinations target:** < 10,000  
**Label key whitelist enforcement:** Only accept predefined labels

---

## 🔒 Security Implications

### Authentication & Authorization
- **Default:** No authentication (accessible locally via localhost only)
- **Production Mode:** Token-based authentication via environment variable `NEONEXUS_METRICS_TOKEN`
- **TLS Support:** Optional HTTPS endpoint with certificate paths configurable

### Data Privacy
- **DO NOT include:** PII, private keys, wallet addresses, sensitive configuration values
- **Mask automatically:** Database connection strings, API credentials in log contexts
- **Audit trail:** Track which metrics are exposed and their sensitivity level

### Attack Surface
- **Exposed endpoint risk:** `/metrics` can be exploited for DoS if not rate-limited
- **Mitigation:** Bind to 127.0.0.1 only by default, explicit binding required for external access
- **Rate limiting:** Max 100 requests/minute per IP address

---

## 📖 User Experience Impact

### Configuration Changes Required

**Minimal setup (most users):**
```bash
neo-nexus start --experimental-metrics
# Metrics available at http://127.0.0.1:9090/metrics
```

**Production setup with authentication:**
```bash
export NEONEXUS_METRICS_TOKEN="your-secret-token"
neo-nexus start --experimental-metrics \
  --metrics-bind-address="0.0.0.0:9090" \
  --metrics-auth-mode="token"
```

### Breaking Changes Assessment
**NONE** - This is an optional experimental feature, fully backward compatible. Existing users can continue without enabling it.

### Migration Path
No migration needed - new users simply opt-in via feature flag, existing deployments unaffected.

---

## 🧪 Testing Strategy

### Unit Tests
1. Metric registration and unregistration correctness
2. Counter increment and gauge set/get idempotency
3. Thread-safety under concurrent access (>10k ops/sec)

### Integration Tests
1. Verify metrics persist across node restarts
2. Test metric accuracy against known events
3. Validate histogram bucket distributions

### Performance Tests
1. Stress test: 10,000 metric queries over 1 minute
2. Memory profiling with 100+ concurrent connections
3. Latency benchmark: P99 response time < 100ms

### Security Tests
1. Unauthorized access rejection when auth enabled
2. Token validation with valid/invalid tokens
3. SQL injection prevention in metric labels

---

## 🚀 Rollout Plan

### Alpha Testing (v4.2.0-alpha)
- Internal testing only
- Limited to CI pipeline instrumentation
- Collect feedback from dev team

### Beta Testing (v4.2.0-beta)
- Open to select beta testers (community operators)
- Full documentation published
- Feature flag still required

### Stable Release (v4.3.0)
- Default stable version
- Production-ready security hardening
- Official Grafana dashboards included

### Deprecation Path
None - this is a core observability feature, no planned deprecation

---

## 📚 Documentation Deliverables

1. **Developer Guide:** How to add new metrics, best practices
2. **User Manual:** Configuration options, example setups
3. **Operator Dashboard:** Grafana templates for common scenarios
4. **API Reference:** Complete list of all exported metrics with descriptions
5. **Troubleshooting Guide:** Common issues and resolutions

---

## ✅ Success Criteria

| Metric | Target Value | Measurement Method |
|--------|-------------|---------------------|
| Query latency (P99) | < 100ms | Load test with 1000 req/min |
| Memory overhead | < 5MB | Heap profiler during stress test |
| CPU overhead | < 5% | Benchmark before/after comparison |
| Label cardinality | < 10k unique series | Monitor Prometheus ingestion stats |
| Documentation coverage | 100% metric definitions | Audit against schema |

---

## 🔄 Feedback Mechanism

**Community Input Channels:**
1. GitHub Discussion thread post-launch
2. Beta tester survey after first month
3. Slack/Discord channel #observability-feedback
4. Quarterly metrics improvement retrospective

**Bug Reporting Template:**
```yaml
Environment: [production/test/dev]
NeoNexus Version: X.Y.Z
Metric Name: neo_nexus_XXX
Expected Behavior: [...]
Actual Behavior: [...]
Reproduction Steps: [...]
Logs/Screenshots: [...]
```

---

## 📝 Appendix A: Example Output Format

```text
# HELP neo_nexus_node_up Whether the node is operational
# TYPE neo_nexus_node_up gauge
neo_nexus_node_up{chain="cosmoshub",network="mainnet"} 1
# HELP neo_nexus_events_total Total events processed
# TYPE neo_nexus_events_total counter
neo_nexus_events_total{kind="lifecycle",severity="info"} 15234.0
# HELP neo_nexus_web_requests_total Web HTTP requests
# TYPE neo_nexus_web_requests_total counter
neo_nexus_web_requests_total{handler="node_start",method="POST",status_code="200"} 45.0
```

---

## 📄 Appendix B: Related Resources

- [Prometheus Best Practices](https://prometheus.io/docs/practices/naming/)
- [OpenMetrics Specification](https://openmetrics.io/)
- [Grafana Dashboard Templates](https://grafana.com/grafana/dashboards/)
- [Rust Prometheus Crate Docs](https://docs.rs/prometheus/)

---

*This RFC is subject to revision based on community feedback and technical validation.*  
*Last Updated:* September 6, 2026  
*Next Review Cycle:* Post-v4.2.0-beta release