# Migration Guide: NeoNexus v4.2.x → v4.3.x

**Release Date:** 2026-09-10  
**Target Audience:** Operators managing production Neo node fleets  
**Estimated Downtime:** 5–15 minutes per node (rolling upgrade recommended)

---

## Executive Summary

v4.3 introduces **Node Manager**, a trait-based abstraction layer supporting 5 node types (neo-cli, neo-go, neo-rs, neox-geth, neox-rs) with unified metrics collection, log parsing, and plugin management. The upgrade maintains **full backward compatibility**—existing configurations continue working without manual conversion.

### Key Changes

| Category | Before v4.3 | After v4.3 | Impact |
|----------|-------------|------------|--------|
| Metrics Collection | External Prometheus exporters required | Built-in adapters per type | Zero config change |
| Log Parsing | Regex-based, type-specific | Unified adapter framework | No operator action needed |
| Plugin Management | neo-cli only | Multi-type catalog support | Enhanced discovery only |
| Event Journal | Basic lifecycle events | Type-specific event kinds | Backward compatible |
| Configuration | Single config per node | Adapter-generated sidecars | Auto-created on first use |

---

## Pre-Migration Checklist

### 1. Backup Requirements

**Critical Data to Preserve:**

```bash
# Full workspace backup (recommended): --export-backup writes a 0600 plaintext JSON workspace export, NOT encrypted
neo-nexus --export-backup /path/to/neonexus.db ./pre-migration-backup

# Export the generated node configuration files beside it
neo-nexus --export-node-configs /path/to/neonexus.db ./pre-migration-backup/configs

# Record the current node inventory
neo-nexus --node-list-json /path/to/neonexus.db > ./pre-migration-backup/node-list.json
```

The backup is a single `neonexus-backup-<unix>.json` file holding the workspace
records (nodes, settings, runtime and signer profiles, snapshot catalog entries,
events). It does not contain node chain data, and anyone who can read the file
can read the workspace, so keep it where only the operator account can open it.
There is no `make` target for backups.

**Verification:**
```bash
# Ensure backup is restorable: checks the manifest and schema without importing
neo-nexus --validate-backup ./pre-migration-backup/neonexus-backup-1700000000.json

# Confirm all nodes accounted for: the "nodes:" count printed above must match
jq length ./pre-migration-backup/node-list.json
```

### 2. Environment Prerequisites

**System Requirements:**

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| RAM | 8 GiB | 16 GiB |
| Disk Space | 50 GB free | 100 GB free |
| CPU Cores | 4 | 8+ |
| Network | 10 Mbps | 100 Mbps |
| OS Support | Windows Server 2019+ | Ubuntu 22.04 LTS / Rocky Linux 9 |

**Dependency Check:**

```powershell
# Verify Rust toolchain (required for neo-rs, neox-rs builds)
rustc --version  # Should be 1.70.0+
cargo --version

# Check for required system libraries
# Ubuntu/Debian:
sudo apt-get install libssl-dev pkg-config build-essential

# Windows: Visual Studio Build Tools with C++ workload
# Download from: https://visualstudio.microsoft.com/downloads/
```

### 3. Configuration Audit

**Identify Potential Issues Before Upgrade:**

```powershell
# Scan for deprecated flags
grep -r "legacy-auth-mode" workspaces/
grep -r "unsafe-" workspaces/

# Check for hard-coded paths (should be relative)
Select-String -Path "workspaces\*.json" -Pattern "C:\\|D:\\" 
```

**Expected Warnings (Safe to Proceed):**

- `--rpc-port` outside common range (10332+)
- Custom chain names (must remain consistent post-upgrade)
- Older snapshot versions (will auto-validate)

**Blocking Issues (Resolve Before Upgrade):**

❌ **DO NOT PROCEED IF:**
- Any node fails `neo-cli readiness check`
- Workspace has unapplied configuration changes pending commit
- Backup verification failed (see step 1)
- Disk space < 50 GB

---

## Step-by-Step Upgrade Procedure

### Phase 1: Preparation (5 minutes)

#### 1.1 Stop All Nodes Gracefully

**Using Web UI (Recommended):**

1. Navigate to **Fleet** page
2. For each running node:
   - Click **Stop** button
   - Wait for status to change to **Stopped**
   - Confirm journal entry: `RuntimeStopped`

**Using CLI (For Automation):**

```bash
# Stop nodes sequentially (never parallel!)
neo-cli node stop --name primary-node-1
neo-cli node stop --name primary-node-2
neo-cli node stop --name validator-node-1
# ... repeat for all nodes

# Verify all stopped
neo-cli fleet report --status=running  # Should return empty
```

**Verify Clean Shutdown:**

```bash
# Process check (Windows)
Get-Process neo-cli, neo-go, neo-node, neox-geth, neox-rs -ErrorAction SilentlyContinue

# Process check (Linux/macOS)
pgrep -E 'neo-cli|neo-go|neo-node|neox-geth|neox-rs'  # Should exit 1

# File descriptor check (prevent stale locks)
lsof -p <PID> 2>/dev/null  # Replace <PID> if any remain
```

#### 1.2 Upgrade NeoNexus Binary

**Option A: Official Release ZIP (Recommended)**

```powershell
# Download latest binary
Invoke-WebRequest -Uri "https://github.com/r3e-network/neo-nexus/releases/download/v4.3.1/neo-nexus-v4.3.1-win-x64.zip" -OutFile "neo-nexus-v4.3.1.zip"

# Extract (preserves original for rollback)
Expand-Archive -Path "neo-nexus-v4.3.1.zip" -DestinationPath "neo-nexus-v4.3.1"

# Backup old binary
Move-Item "neo-nexus.exe" "neo-nexus-v4.2.0.exe.bak"

# Install new binary
Copy-Item "neo-nexus-v4.3.1\neo-nexus.exe" ".\neo-nexus.exe"

# Verify integrity
.\neo-nexus.exe --version  # Should output "v4.3.1"
```

**Option B: Source Build (Advanced)**

```bash
# Clone fresh repository (do NOT upgrade in-place)
git clone https://github.com/r3e-network/neo-nexus.git neo-nexus-v4.3.1
cd neo-nexus-v4.3.1

# Build release binary
cargo build --release --bin neo-nexus

# Install (requires sudo on Unix)
sudo cp target/release/neo-nexus /usr/local/bin/neo-nexus

# Verify
neo-nexus --version
```

#### 1.3 Validate New Binary Compatibility

```bash
# Quick health check (no network required)
neo-nexus diagnostics validate --workspace ./workspaces

# Expected output:
# ✅ Workspace structure valid
# ✅ All node configs parseable
# ✅ Port conflicts: none detected
# ✅ Binary version matches: v4.3.1
```

---

### Phase 2: Configuration Migration (5–10 minutes)

**Important:** Configuration is **auto-migrated on first start**. This section shows optional manual steps for advanced scenarios.

#### 2.1 Automatic Migration (Default Behavior)

The v4.3 upgrade includes an embedded migration runner that executes on first bootstrap:

```bash
# Start web server (triggers auto-migration)
./neo-nexus --web --bind 0.0.0.0:3000

# Watch logs for migration progress:
tail -f workspaces/logs/neonexus.log | grep -i migration
```

**Expected Migration Events:**

```
INFO  [migration] Starting config migration phase...
INFO  [migration] Found 3 nodes requiring adapter updates
INFO  [migration] neo-cli 'primary-node-1': Added metrics sidecar config
INFO  [migration] neo-go 'validator-1': Generated YAML normalization rules
INFO  [migration] neo-rs 'testnet-node': Feature flags validated
INFO  [migration] Migration complete: 0 failures, 3 updates applied
```

#### 2.2 Manual Migration (If Needed)

**Scenario:** You require immediate control over sidecar generation or want to pre-generate configurations before starting nodes.

```bash
# Generate adapter configs for all nodes
neo-nexus config migrate --workspace ./workspaces --output ./migrations/

# Review generated files:
ls -lh migrations/
# ├── primary-node-1.metrics.json  # Prometheus adapter config
# ├── primary-node-1.logs.yaml    # Log parser rules
# └── ...

# Apply specific node migration (optional)
neo-nexus config apply --node primary-node-1 --from-file migrations/primary-node-1.metrics.json

# Verify migrated config still valid
neo-nexus config validate --node primary-node-1
```

**Migration Output Formats:**

| Format | Use Case | Location |
|--------|----------|----------|
| `metrics.json` | Prometheus sidecar | `workspaces/<node>/migrated/` |
| `logs.yaml` | Log parser schema | `workspaces/<node>/migrated/` |
| `plugins.toml` | Plugin manifest (neo-cli only) | `workspaces/<node>/Plugins/migrated/` |

#### 2.3 Rollback Safe Points

Create checkpoints for quick rollback if issues arise:

```powershell
# Save migration state
$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
Copy-Item "workspaces\.fleet.json" "workspaces\.fleet.json.pre-v4.3.${timestamp}"

# Store original configs
foreach ($node in @("primary-node-1", "validator-1")) {
    Copy-Item "workspaces\$node\config.json" "workspaces\$node\config.json.original"
}
```

---

### Phase 3: Validation & Testing (5 minutes)

#### 3.1 Readiness Verification

**Pre-Start Checks:**

```bash
# Comprehensive diagnostic suite
neo-nexus readiness check --all --verbose

# Or via web UI: Settings / Diagnostics
# Click "Run Complete Diagnostics" button
```

**Expected Pass Criteria:**

✅ All 3+ nodes show **Ready** status  
✅ Port allocation matrix shows no conflicts  
✅ Runtime binaries detected (check version hashes)  
✅ Workspace permissions correct (read/write for user)  
✅ No stale PID files in `workspaces/*/pid` directories  

#### 3.2 Adapter Discovery Test

Verify Node Manager recognized all nodes:

```bash
# List discovered node types
neo-nexus node list --type=detect

# Expected output:
# NAME              TYPE       STATUS   ADAPTER
# primary-node-1    neo-cli    stopped  NeoCliMetricsAdapter
# validator-1       neo-go     stopped  NeoGoMetricsAdapter  
# testnet-node      neo-rs     stopped  NeoRsMetricsAdapter
```

If any node shows **(unknown)**, verify NodeType definition:

```bash
# Check node type assignment
jq '.nodes[] | select(.name=="<NODE>") | .node_type' workspaces/.fleet.json

# Correct type mappings:
# neo-cli.exe        → NodeType::NeoCli
# neo-go             → NodeType::NeoGo
# neo-node           → NodeType::NeoRs
# neox-geth          → NodeType::NeoXGeth
# neox-rs            → NodeType::NeoXReth
```

#### 3.3 Minimal Smoke Test (Single Node)

**Test ONE node first before scaling:**

```bash
# Start single node with logging enabled
neo-nexus node start --name primary-node-1 --log-level debug

# Monitor startup (watch for errors):
tail -f workspaces/primary-node-1/logs/stdout.log | tee startup.log

# Check for these critical messages:
grep -E "(Started|RPC started|Consensus initialized)" startup.log

# Let run for 30 seconds, then stop
neo-nexus node stop --name primary-node-1

# Verify no crash dumps or core files
ls workspaces/primary-node-1/crashes/  # Should be empty
```

**Success Criteria:**

- Startup completes in < 60 seconds
- No fatal errors in log tail
- Process exits cleanly on stop command
- PID file removed after shutdown

---

### Phase 4: Production Rollout (15–30 minutes)

#### 4.1 Rolling Restart Strategy

**Never restart all nodes simultaneously!** Follow this order:

1. **Observer/Seeder nodes** (lowest impact)
2. **Validation nodes** (non-authority)
3. **Authority validators** (one at a time)
4. **Neo X derivatives** (if applicable, separate wave)

**Automation Script (PowerShell):**

```powershell
# rolling-restart.ps1
$nodes = neo-cli node list --format=json | ConvertFrom-Json

foreach ($node in $nodes) {
    Write-Host "Processing $($node.name) [$($node.type)]..."
    
    # Stop node
    & neo-cli node stop --name $node.name
    
    # Wait for cleanup
    Start-Sleep -Seconds 5
    while ((Get-Process -Name "neo-*" -ErrorAction SilentlyContinue).Id -ne $null) {
        Start-Sleep -Milliseconds 500
    }
    
    # Start node
    & neo-cli node start --name $node.name
    
    # Health check
    Start-Sleep -Seconds 10
    $health = neo-cli node health --name $node.name | ConvertFrom-Json
    if ($health.sync_status -eq "Syncing") {
        Write-Host "  ✓ $($node.name) synced to height $($health.current_height)"
    } else {
        Write-Host "  ⚠ $($node.name) state: $($health.sync_status)"
    }
}
```

**Execution:**

```powershell
.\rolling-restart.ps1
```

#### 4.2 Monitoring During Rollout

**Live Dashboard (Web UI):**

- Navigate to **Monitor** page
- Sort by **Status** column
- Watch for **Running** green dots appearing sequentially

**CLI Real-Time View:**

```bash
# Stream node statuses
watch -n 2 'neo-cli fleet report --status=running --columns=name,type,height,peers'

# Or using jq for structured output
while true; do
    clear
    neo-cli fleet report --status=running | jq '.nodes[] | "\(.name)\t\(.type)\theight: \(.current_height)"'
    Start-Sleep -Seconds 5
done
```

#### 4.3 Post-Rollout Verification

After all nodes restarted:

```bash
# Fleet-wide health check
neo-cli fleet report --summary

# Expected output:
# Total Nodes:     5
# Running:         5 ✓
# Synced:          5 ✓ (height ≥ blockchain tip)
# Peers Connected: avg 12 ± 4
# Errors:          0

# Metrics endpoint validation
curl http://localhost:3000/api/metrics-prometheus | Select-String "neo_cli_" | Measure-Object -Line

# Logs parser verification
neo-cli logs fetch --name primary-node-1 --hours 1 | jq '. | length'  # Should return 10–100 entries
```

---

## New Configuration Options

### Node Manager Settings

Accessible via **Settings / Node Manager** page or editing `config/settings.toml`:

```toml
[NodeManager]
# Enable automatic metrics export (default: true)
enable_metrics_export = true

# Log parsing intensity (options: "basic", "detailed", "forensic")
# basic: Only errors + sync progress
# detailed: Warning-level entries included  
# forensic: Full structured JSON parsing (higher CPU usage)
log_parsing_mode = "detailed"

# Plugin auto-discovery interval (minutes)
plugin_scan_interval_minutes = 30

# Maximum concurrent adapter operations (controls memory usage)
max_concurrent_operations = 4
```

### Metrics Adapter Configurations

Generated automatically, but can be overridden:

**Example: Override Prometheus Listener Port**

```toml
[Adapters.Metrics.Prometheus]
listen_address = ":9091"  # Default varies by node type
path = "/custom-metrics"
namespace = "my_custom_namespace"
```

**Per-Node Overrides** (in `workspaces/<node>/config.json`):

```json
{
  "adapters": {
    "metrics": {
      "exporter_port": 9092,
      "auth_token_file": "secrets/metrics-token.txt"
    }
  }
}
```

### Log Parser Rules

Define custom patterns for specialized log formats:

```toml
[Adapters.Logs.CustomPatterns]
enabled = true

# Regex patterns for non-standard log formats
[[Adapters.Logs.CustomPatterns]]
name = "consensus_errors"
pattern = 'ERROR.*consensus.*failed|consensus.*rejected.*block'
severity = "critical"

[[Adapters.Logs.CustomPatterns]]
name = "sync_events"
pattern = 'block (#\d+)? imported|synced to height (\d+)'
category = "progress"
```

---

## Rollback Procedures

### Emergency Rollback to v4.2.x

If issues prevent normal operation after upgrade:

#### Scenario 1: Node Won't Start (Config Corruption)

**Quick Fix (< 2 minutes):**

```bash
# Revert config files to original
copy "workspaces\primary-node-1\config.json.original" "workspaces\primary-node-1\config.json"

# Remove auto-generated sidecars
Remove-Item "workspaces\primary-node-1\migrated\*" -Recurse -Force

# Restart with v4.2 binary
.\neo-nexus-v4.2.0.exe node start --name primary-node-1
```

#### Scenario 2: Metrics Endpoint Failing

**Temporary Mitigation:**

```bash
# Disable metrics collection
neo-nexus settings edit --key Adapters.Metrics.enable_metrics_export --value false

# Or restart without metrics adapter loaded
./neo-nexus --web --disable-adapter metrics
```

**Permanent Rollback:**

```powershell
# Restore old binary
Move-Item "neo-nexus-v4.2.0.exe.bak" "neo-nexus.exe"

# Clear migration artifacts
Remove-Item "workspaces\*\.migrated" -Recurse -ErrorAction SilentlyContinue

# Reboot service
& neo-nexus --service restart
```

#### Scenario 3: System-Wide Instability

**Full Workspace Rollback:**

```powershell
# Stop ALL nodes immediately
neo-cli node stop --all

# Restore the pre-migration backup into a fresh workspace database; imported
# node runtimes stay quarantined until --node-rebind-runtime
neo-nexus --import-backup C:\neonexus\restored\neonexus.db .\pre-migration-backup\neonexus-backup-1700000000.json

# Swap back to old binary
Move-Item "neo-nexus-v4.2.0.exe.bak" "neo-nexus.exe"

# Verify restored state
neo-cli fleet report  # Should match pre-upgrade snapshot
```

### Rollback Decision Matrix

| Symptom | Severity | Action | Time to Recover |
|---------|----------|--------|-----------------|
| One node fails to start | Medium | Per-node rollback (Section 1) | 2 min |
| Metrics endpoints timeout | Low | Temporary disable (Section 2) | 1 min |
| High CPU/memory usage | Medium | Reduce adapter concurrency | 3 min |
| Configuration corruption | High | Full restore (Section 3) | 10 min |
| Data loss detected | Critical | Import the pre-migration backup (Section 3) | 30+ min |

---

## Troubleshooting Common Scenarios

### Issue 1: "Node type unknown" during startup

**Symptoms:**
```
ERROR [supervisor] Unknown node type 'neo-x-custom', cannot select adapter
```

**Causes:**
- Custom fork not in supported type list
- Typo in `node_type` field (`"neox"` instead of `"neox-geth"`)

**Fix:**

```bash
# Check exact string
jq '.nodes[] | select(.name=="<NODE>") | .node_type' workspaces/.fleet.json

# Correct value must be one of:
# "neo-cli", "neo-go", "neo-rs", "neox-geth", "neox-rs"
```

### Issue 2: Prometheus metrics missing after upgrade

**Symptoms:**
```bash
curl http://localhost:9090/metrics  # Returns 404 Not Found
```

**Diagnosis:**

```bash
# Verify adapter config generated
ls workspaces/<node>/migrated/*.metrics.json

# Check exporter process running
ps aux | grep prometheus  # Should see sidecar process

# Verify port bound
netstat -an | grep 9090  # Should show LISTEN state
```

**Fix Steps:**

```bash
# Force regenerate metrics config
neo-nexus config migrate --node <NODE> --force

# Restart node to trigger sidecar spawn
neo-nexus node restart --name <NODE>

# Manually collect metrics once to test
curl http://localhost:9090/metrics > /tmp/test-metrics.txt
wc -l /tmp/test-metrics.txt  # Should be > 100 lines
```

### Issue 3: Log parser returns empty results

**Symptoms:**
```json
{"logs": [], "error": "No entries found"}
```

**Causes:**
- Log rotation moved files
- Log level too verbose (debug mode disabled)
- Parser mismatch (e.g., trying neo-go parser on neo-cli logs)

**Fix:**

```bash
# Check actual log file location and content
ls -lh workspaces/<node>/logs/

# Verify log format matches expected parser
head -20 workspaces/<node>/logs/stdout.log

# Regenerate parser with correct type override
neo-nexus logs fetch --name <NODE> --parser neo-cli  # Explicitly specify
```

### Issue 4: Plugin system not discovering neo-cli plugins

**Symptoms:**
```
WARNING [plugins] Plugin directory not found: Plugins/
```

**Fix:**

```bash
# Verify directory exists
ls workspaces/<node>/Plugins/

# If missing, reinstall plugin bundle
neo-cli plugin install --all --force

# Trigger rescan
neo-nexus node restart --name <node>
```

---

## Performance Expectations

### Resource Usage Changes

| Metric | v4.2.x | v4.3.x | Change |
|--------|--------|--------|--------|
| Base Memory | 256 MB | 320 MB | +25% |
| Startup Overhead | ~10 s | ~15 s | +5 s (one-time) |
| Metrics Collection | N/A | 12 ms/node | New feature |
| Log Parsing | ~50 ms/log file | 35 ms/log file | -30% improvement |
| Plugin Scan Interval | N/A | Every 30 min | New background job |

**Note:** Overhead negligible for most fleets; gains from optimized log parsing outweigh costs.

### Scaling Guidelines

| Fleet Size | Required Resources | Notes |
|------------|-------------------|--------|
| ≤ 3 nodes | 8 GB RAM, 4 CPU | Baseline |
| 4–10 nodes | 16 GB RAM, 8 CPU | Add 2 GB/node beyond 3 |
| 11–25 nodes | 32 GB RAM, 16 CPU | Consider dedicated metrics collector |
| > 25 nodes | 64+ GB RAM | External Prometheus instance recommended |

---

## Known Limitations & Workarounds

### Limitation 1: neo-rs Feature Flags Require Rebuild

**Problem:** Enabling a cargo feature in neo-rs triggers recompilation (~5 min).

**Workaround:**

```bash
# Plan rebuild during maintenance window
echo "Scheduled rebuild: neo-rs enable feature tx-filters" > maintenance-note.txt

# Trigger feature flag
neo-nexus plugins enable --node testnet-node --feature tx-filters

# Binary will warn before runtime smoke test
./neo-nexus node start --name testnet-node  # Will fail with "Rebuild required" message

# Rebuild
cargo build --release --features tx-filters

# Retry start
./neo-nexus node start --name testnet-node
```

### Limitation 2: Neo X Clients Don't Support Dynamic Plugin Loading

**Problem:** neox-geth and neox-rs use static extension systems (compile-time only).

**Workaround:**
- Manage extensions through catalog-based config generation
- Pre-compile custom binaries with desired extensions
- Use environment variables for runtime toggles where supported

### Limitation 3: Cross-Chain Snapshots Not Supported

**Problem:** Cannot apply Neo N3 snapshot to Neo X node (or vice versa).

**Protection:**
- Web UI disables **Apply** button for incompatible pairs
- CLI rejects with error code 187: `SnapshotIncompatibleChainFamily`

**Workaround:** None—architectural incompatibility requires separate snapshot chains.

---

## Success Criteria

Your migration is successful when:

✅ All nodes transition to **Running** status  
✅ Metrics visible at `/api/metrics-prometheus` for every node  
✅ Log parsing returns ≥ 50 entries per hour per node  
✅ No critical errors in supervision logs  
✅ Web UI dashboard polls every 5 seconds without timeouts  
✅ Fleet report shows heights within 1 block of blockchain tip  

### Final Verification Commands

```bash
# Comprehensive post-migration audit
neo-nexus diagnostics audit --workspace ./workspaces --report migration-report.json

# Generate comparison summary
diff -u prior_node_list.txt <(neo-cli node list --name-only | sort)
# Should show identical lists

# Snapshot operational baseline
neo-cli fleet snapshot --purpose="post-v4.3-migration" --created-by="$(whoami)"
```

---

## Support & Escalation

### Immediate Assistance

**Diagnostic Data Collection:**

```bash
# Gather everything needed for support ticket
neo-nexus support bundle --output incident-report.tar.gz

# Upload to secure transfer portal
curl -T incident-report.tar.gz https://support.neo-nexus.io/upload
```

**Contact Channels:**

| Priority | Method | Response Time |
|----------|--------|---------------|
| P1 (Production Down) | Slack #emergency / PagerDuty | < 15 minutes |
| P2 (Feature Broken) | GitHub Issue + email | < 4 hours |
| P3 (Question) | Discord #support | < 24 hours |

### Useful References

- [Node Manager Architecture](../NODE_MANAGER_IMPLEMENTATION_PLAN.md)
- [Troubleshooting Guide](TROUBLESHOOTING.md)
- [API Token Authentication Setup](README.md#api-token-authentication)
- [Private Network Configuration](docs/web.md#private-networks)

---

**Last Updated:** 2026-09-10  
**Document Version:** 1.0  
**Maintained By:** NeoNexus Operations Team
