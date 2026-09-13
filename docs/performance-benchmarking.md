# NeoNexus Performance Benchmarking Framework

## Overview

This benchmark suite provides comprehensive performance metrics for tracking regressions, establishing baselines, and optimizing performance across all aspects of NeoNexus node management operations.

## Installation Prerequisites

Before running benchmarks, ensure you have the Rust nightly toolchain installed:

```powershell
rustup toolchain install nightly
```

Criterion requires nightly Rust for some advanced features.

## Usage

### Running All Benchmarks

```powershell
cargo bench
```

Or specifically for this benchmark file:

```powershell
cargo bench --bench node_manager_bench
```

### Running Specific Benchmark Groups

```powershell
# Run only startup latency benchmarks
cargo bench startup

# Run memory allocation benchmarks
cargo bench memory_allocation

# Run connection pooling benchmarks
cargo bench connection_pooling

# Run log parsing benchmarks
cargo bench log_parsing

# Run event journal benchmarks
cargo bench event_journal

# Run integration workflow benchmarks
cargo bench integration_workflows
```

### Profiling Mode

Profile benchmarks for detailed CPU analysis:

```powershell
cargo bench -- --profile-time 30
```

This profiles for 30 seconds, collecting more data points for statistical significance.

### Generating HTML Reports

Generate interactive HTML reports with charts and visualizations:

```powershell
cargo bench --bench node_manager_bench -- --html-output docs/benchmark-report/
```

The HTML report will be generated in `target/criterion/report/index.html`.

Open the report in a browser:

```powershell
start target/criterion/report/index.html
```

### Comparison Mode

Compare current results against a previous baseline:

```powershell
# First, save current results as baseline
cargo bench --bench node_manager_bench --save-baseline main

# Later, compare against baseline
cargo bench --bench node_manager_bench --baseline main
```

## Benchmark Categories

### 1. Startup Latency Benchmarks

**Purpose**: Measure overhead from `NodeManager::start_node()` call to PID return (excluding actual binary startup).

**Baseline Target**: `<100ms` overhead (95th percentile)

**Tested Scenarios**:
- Cold start with adapter selection for each node type:
  - NeoCli
  - NeoGo  
  - NeoRs
  - NeoXGeth
  - NeoXReth
- Warm start with cached adapters
- Cross-node comparison of adapter initialization costs

**What This Measures**:
- Adapter lookup overhead
- Database write latency for event logging
- Process spawning abstraction layer costs
- Thread scheduling overhead

**Expected Results**:
- Cold start: 50-100ms median
- Warm start: <20ms (cached lookup)
- P95 should remain below 150ms

### 2. Memory Allocation Profiling

**Purpose**: Track heap usage and allocation patterns during critical operations.

**Baseline Targets**:
- Log parsing: `<500 bytes/line`
- Event Journal append: `<1KB/event`
- Connection pool: `<50KB per pooled connection`

**Tested Scenarios**:
- Allocation footprint per parsed log line
- Concurrent writer memory footprint
- Heap fragmentation detection
- GC pressure monitoring under repeated iteration

**Metrics Collected**:
- Bytes allocated per operation
- Number of allocations per operation
- Average allocation size
- Peak heap usage

**Tools Integration**:
- Built-in criterion allocation statistics
- Optional jemalloc profiling (`export JE_MALLOC_CONF=stats_print:true`)

### 3. Connection Pooling Efficiency

**Purpose**: Optimize HTTP client connection reuse and TLS handshake costs.

**Baseline Targets**:
- New TLS connection: `<50ms`
- Pooled connection reuse: `<5ms`
- Max concurrent connections: `>=100 idle limit maintained`

**Tested Scenarios**:
- Fresh TLS handshake time
- Connection pool reuse efficiency
- Concurrent request handling (50 simultaneous connections)
- Connection exhaustion stress testing

**Metrics Collected**:
- Time to first byte (TTFB)
- Connection establishment vs reuse differential
- Pool utilization rate
- Error rates under load

**Best Practices Validated**:
- Reusing `reqwest::Client` instances
- Proper timeout configuration
- Keep-alive header settings
- Max idle connections limits

### 4. Log Parsing Throughput

**Purpose**: Measure parsing performance across different log formats and complexity levels.

**Baseline Targets**:
- Plain text format: `>1M lines/sec`
- JSON format: `>500K lines/sec`
- CPU utilization: `<80% during intensive parsing`

**Tested Scenarios**:
- NeoCLI bracket format parsing
- Go log format parsing
- Rust tracing format parsing
- JSON structured log parsing
- Mixed-format log streams

**Metrics Collected**:
- Lines per second throughput
- CPU cycles per line
- Memory bandwidth utilization
- Regex compilation overhead

**Optimization Opportunities**:
- Pattern caching for frequently used regex
- SIMD-accelerated string matching
- Parallel parsing with Rayon
- Zero-copy parsing techniques

### 5. Event Journal Performance

**Purpose**: Evaluate SQLite-backed event storage under varying loads.

**Baseline Targets**:
- Single writer append: `>10,000 events/sec`
- Concurrent writers (10x): `>5,000 events/sec`
- Query performance (1M entries): `<100ms median`

**Tested Scenarios**:
- Single-threaded append speed
- Multi-writer contention (1x, 5x, 10x parallel writers)
- Date range query efficiency
- Index effectiveness on large datasets
- Transaction batching performance

**Database Schema**:
```sql
CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    timestamp TEXT NOT NULL,
    level TEXT NOT NULL,
    node_type TEXT NOT NULL,
    message TEXT NOT NULL,
    INDEX idx_timestamp (timestamp),
    INDEX idx_node_type (node_type),
    INDEX idx_level (level)
);
```

**Query Optimization Validated**:
- Composite index usage
- Covering indexes for common queries
- Partition strategies by date
- WAL mode benefits

### 6. Integration Workflows

**Purpose**: End-to-end performance validation of user-facing workflows.

**Tested Scenarios**:
- Node startup → Metrics collection → Event logging flow
- REST endpoint response times (p50, p95, p99 percentiles)
- CLI command vs Web handler equivalence
- Multi-node batch operations (sequential vs parallel)
- Export/import workflows for backup/restore

**Metrics Collected**:
- Total workflow duration
- Per-step timing breakdowns
- Resource utilization peaks
- Error/retry rates

## Statistical Significance

All benchmarks use rigorous statistical methods:

### Confidence Intervals
- 95% confidence intervals reported for all mean values
- Standard error of the mean (SEM) calculated
- Outliers removed using IQR method (Q1 - 1.5×IQR, Q3 + 1.5×IQR)

### P-Value Thresholds
- **Statistical significance**: p < 0.05
- **Highly significant**: p < 0.01
- **Extremely significant**: p < 0.001

### Sample Size Requirements
- Minimum iterations per benchmark: 100
- For high-variance operations: auto-scaling based on coefficient of variation (CV)
- Batch sizes selected to minimize noise while maintaining reproducibility

### Regression Detection
Alert threshold configured at **5% degradation** from baseline:

| Degradation Range | Alert Level | Action Required |
|-------------------|-------------|-----------------|
| 0-5%              | Monitor     | No action        |
| 5-10%             | Warning     | Investigate      |
| 10-20%            | Critical    | Hotfix priority  |
| >20%              | Blocker     | Release blocked  |

## Interpreting Results

### Reading HTML Reports

1. **Distribution Charts**: Show median, quartiles, and outliers
   - Red box = interquartile range (IQR)
   - Black horizontal line = median
   - Whiskers = 1.5×IQR range
   - Dots = outliers

2. **Benchmark Summary Table**:
   - `time`: Median execution time
   - `change`: Percentage difference from baseline
   - `stddev`: Standard deviation
   - `mean`: Arithmetic mean
   - `min/max`: Best and worst cases

3. **Regression Indicators**:
   - 🟢 Green arrow = improvement
   - 🔴 Red arrow = regression
   - ⚪ Gray circle = statistically insignificant change (<1%)

### Identifying Performance Issues

#### High Variance (>20% CV)
Indicates unstable test conditions or real-world non-determinism:
- Background processes interfering
- Cache effects (warm vs cold starts)
- Garbage collection interference

**Solution**: Increase iterations, add stabilization periods

#### Consistent Slowdowns
When entire distribution shifts right:
- Dependency version change
- Algorithmic complexity increase
- Resource contention (CPU/memory/disk I/O)

**Solution**: Profile with `cargo flamegraph`, check git diff for algorithmic changes

#### Increased Memory Usage
Higher allocation counts or larger average allocation sizes:
- New object creation without pooling
- String conversions not optimized
- Collection growth without capacity hints

**Solution**: Use `.collect_with_capacity()`, implement object pools, enable zero-copy parsing

## CI/CD Integration

### Automated Nightly Runs

Add to GitHub Actions `.github/workflows/benchmarks.yml`:

```yaml
name: Performance Benchmarks

on:
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM UTC
  push:
    branches: [main]
    paths:
      - 'src/**'
      - 'benches/**'

jobs:
  benchmark:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@nightly
      
      - name: Run benchmarks
        run: cargo bench --bench node_manager_bench -- --html-output benchmark-report
      
      - name: Upload benchmark artifacts
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-report
          path: target/criterion/
          
      - name: Compare with baseline
        if: github.event_name == 'push'
        run: |
          # Use tools like `criterion-compare` to detect regressions
          echo "Checking for performance regressions..."
```

### PR Gate Configuration

Block merges when performance degrades beyond threshold:

```yaml
- name: Check Performance Impact
  run: |
    if [[ $(git diff HEAD^ --stat src/) =~ \.rs$ ]]; then
      cargo bench --bench node_manager_bench --baseline main --save-baseline pr
      RESULT=$(cargo criterion-compare main pr | grep -E '\(regression|improvement\)')
      
      if echo "$RESULT" | grep -q "regression.*[5-9]\|[1-9][0-9]%"; then
        echo "❌ Significant performance regression detected!"
        exit 1
      fi
    fi
```

## Custom Benchmarking Guide

### Creating New Benchmarks

1. **Define Benchmark Function**:

```rust
fn my_custom_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom_operations");
    
    group.bench_function("specific_operation", |b| {
        b.iter(|| {
            // Your code here
            my_function(input_data)
        })
    });
    
    group.finish();
}
```

2. **Use `batched` for setup-heavy operations**:

```rust
group.bench_function("setup_then_measure", |b| {
    b.iter_batched(
        || MyFixture::new(),           // Setup
        |fixture| fixture.do_work(),   // Measure
        criterion::BatchSize::SmallInput
    )
});
```

3. **Protect against optimization elimination**:

```rust
// BAD - compiler might optimize away
b.iter(|| expensive_computation());

// GOOD - black_box prevents optimization
b.iter(|| black_box(expensive_computation()));
```

4. **Group related benchmarks**:

```rust
criterion_group!(
    custom_group,
    my_custom_benchmark,
    another_custom_benchmark,
);

criterion_main!(custom_group);
```

### Advanced Techniques

#### Micro-benchmarking Individual Operations

For fine-grained performance insights:

```rust
use criterion::black_box;

#[inline(never)]
fn process_single_event(event: &Event) -> Result<(), Error> {
    // Implementation
}

fn micro_benchmark(c: &mut Criterion) {
    let event = build_test_event();
    
    c.bench_function("process_event_micro", |b| {
        b.iter(|| black_box(process_single_event(&event)))
    });
}
```

#### Plotters Integration for Custom Charts

Create publication-quality plots:

```rust
use plotters::prelude::*;

fn plot_performance_trends(results: Vec<(String, f64)>) {
    let root = BitMapBackend::new("performance-trend.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    
    let mut chart = ChartBuilder::on(&root)
        .caption("Performance Over Time", ("Courier", 30))
        .margin(Margin::TOP, 50)
        .x_label_area_size(Size::from(30))
        .y_label_area_size(Size::from(30))
        .build_cartesian_2d(0f64..100f64, 0f64..1000f64)
        .unwrap();
    
    chart.configure_mesh().draw().unwrap();
    
    chart.draw_series(results.into_iter().map(|(label, value)| {
        AreaSeries::new(vec![(0, value)], vec![(100, value)], RED.faded())
    })).unwrap();
    
    root.present().unwrap();
}
```

## Troubleshooting

### Common Issues

#### "thread 'main' panicked at 'Failed to open DB'"
Ensure rusqlite is in dependencies with bundled feature:
```toml
rusqlite = { version = "0.37", features = ["bundled"] }
```

#### "compilation failed" with criterion errors
Check you're using nightly Rust:
```powershell
rustup default nightly
```

#### Benchmarks run too slowly
Reduce sample size:
```rust
criterion!(
    sampling_mode: SamplingMode::Flat,
    commitment_interval: 10,  // Report every 10 iterations
);
```

#### Benchmarks show high variance (>30%)
Add warm-up period or stabilize environment:
```rust
b.iter_batched(
    || {
        // Warm up cache/state
        std::thread::sleep(Duration::from_millis(10));
        Fixture::new()
    },
    |fixture| { /* measure */ },
    BatchSize::LargeInput
);
```

### Performance Profiling Tools

#### Flame Graphs

Generate CPU flame graphs:

```powershell
cargo install cargo-flamegraph
cargo flamegraph --bench node_manager_bench
```

#### Memory Profiling

Use `jeprof` with jemalloc:

```bash
# Enable jemalloc
export MALLOC_CONF="prof:true,prof_active:true"

# Run benchmark
cargo bench --bench node_manager_bench

# Analyze heap dump
jeprof ./target/debug/deps/node_manager_bench-* heap.prof
```

#### Async Task Tracing

Enable tokio console for async operations:

```rust
tokio::task::local_set().block_on(async {
    console_subscriber::init();
    // Your async code
});
```

## Baseline Values

Current baseline performance targets (v4.3.1):

| Metric | Target | Acceptable Range |
|--------|--------|------------------|
| Startup latency (cold) | <75ms | <100ms |
| Startup latency (warm) | <15ms | <20ms |
| Log parse throughput | >800K lines/s | >600K lines/s |
| Event Journal appends | >12K events/s | >8K events/s |
| New TLS connection | <40ms | <50ms |
| Pooled connection | <3ms | <5ms |
| Memory per log line | <400 bytes | <500 bytes |

These baselines will be updated after each major release based on accumulated measurement data.

## Contributing

When adding new features that impact performance:

1. **Write corresponding benchmarks** alongside your implementation
2. **Establish new baselines** before committing code
3. **Document expected performance characteristics** in feature documentation
4. **Run full benchmark suite** before creating pull requests
5. **Report both positive and negative impacts** - honesty builds trust

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/criterion_rs.html)
- [Rust Microbenchmarking Guidelines](https://nnethercote.github.io/perf-book/introduction.html)
- [Plotters Charting Library](https://docs.rs/plotters/latest/plotters/)
- [Tokio Console for Async Debugging](https://github.com/tokio-rs/console)

## License

Same license as the main project (MIT).
