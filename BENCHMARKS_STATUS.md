# Benchmark Infrastructure Status

## Current State: Fake NodeManager Facade

The benchmark suite at `benches/node_manager_startup.rs` contains deprecated benchmarks that measure nothing meaningful because `NodeManager.start_node()` returns fake PID 12345 without any actual process spawning or I/O operations.

**Status**: All fake benchmarks have been removed from the default criterion group and are marked with `#[allow(dead_code, deprecated)]` to prevent accidental execution.

**Real Benchmarks Still Available**:
- `bench_async_runtime_overhead` - Measures tokio block_on cost (~10-50μs)
- `bench_node_manager_creation` - Measures NodeManager struct allocation time
- `bench_adapter_init_*` - Measures config generation throughput (not real adapter loading)

## What REAL Benchmarks Would Require

To replace the fake benchmarks with meaningful measurements of actual node startup latency, we need:

### 1. Containerized Test Harness
- Docker container with mock/neoclus binaries available
- Ability to control process lifecycle precisely
- Isolated filesystem for test data
- Configurable RPC endpoints and P2P ports

### 2. Supervision Engine Integration
```rust
// Replace this fake call:
nm.start_node(&config, &plan); // Returns PID 12345, no work

// With real supervision engine calls:
supervision.Engine::launch_node(
    config.clone(),
    plan.clone(),
    metrics_callbacks.clone()
).await?; // Actual process spawn, binary execution, I/O
```

### 3. Metrics Collection Infrastructure  
- Real metrics adapters for each node type (NeoCli, NeoGo, NeoRs, NeoXGeth, NeoXReth)
- Prometheus/gRPC endpoints accessible during benchmarks
- Log parser to extract sync progress, block heights, peer counts
- Backoff policy validation for retry scenarios

### 4. Chain Environment
- Testnet/private network with working nodes
- RPC endpoints for health checks
- Actual blockchain state (block height, transactions)
- Network topology with multiple peers

### 5. Measurement Targets
```rust
// Example of what we want to measure:
benchmark_group!("node_lifecycle/startup");
group.bench_function("Neocli/cold_start", |b| {
    b.iter(|| {
        let start = Instant::now();
        
        // Spawn real binary via supervision.Engine::launch_node()
        let pid = supervision::Engine::spawn_node(config).unwrap();
        
        // Wait for RPC endpoint to become available
        wait_for_rpc_endpoint(pid, timeout(Duration::from_secs(60)))
            .expect("Node should start within 60s");
        
        // Measure actual wall-clock time
        println!(
            "Cold start took {:.2?}", 
            start.elapsed()
        );
        
        black_box(pid);
    })
});
```

### 6. Comparison Benchmarks
We should compare:
- Cold vs warm start times (after initial adapter load)
- Single-node vs multi-node sequential startup
- Different node types (NeoCli, NeoGo, NeoRs, etc.)
- Various configuration complexity levels
- Network topology impacts (peer count, gossip protocol)

## TODO Items

1. **Create test harness infrastructure**:
   - [ ] Docker compose setup for mock nodes
   - [ ] Mock supervision.Engine implementation
   - [ ] Fake but realistic metrics collector

2. **Replace fake benchmarks**:
   - [ ] Remove all `#[allow(deprecated)]` functions calling `start_node()`
   - [ ] Implement new benchmarks using `supervision.Engine::launch_node()`
   - [ ] Add integration tests for realistic startup scenarios

3. **Add performance expectations**:
   - [ ] Document baseline metrics for cold/warm starts
   - [ ] Set performance regression thresholds
   - [ ] Create CI gate for significant slowdowns

## Related Files

- `benches/node_manager_startup.rs` - Deprecated fake benchmarks (marked, not run by default)
- `benches/metrics_log_parsing.rs` - ✅ Valid log parsing benchmarks
- `benches/error_handling_bench.rs` - ✅ Valid error handling benchmarks  
- `src/web/control.rs` - Real `start_node` implementation using supervision.Engine
- `src/cli/actions/node_control.rs` - Alternative real implementation path

## References

- See `docs/node-manager-architecture.md` for understanding `start_node()` limitations
- See `src/supervision/engine.rs` for real process spawning logic
- See memory task "NeoNexus v4.3.0 known gaps catalog" for more technical debt items
