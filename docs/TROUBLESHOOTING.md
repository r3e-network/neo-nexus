# NeoNexus Troubleshooting & Operations Runbook

**Version:** v4.3.1  
**Last Updated:** September 12, 2026  
**Scope:** All supported runtimes (`neo-cli`, `neo-go`, `neo-rs`, `neox-geth`, `neox-rs`)

---

## Table of Contents

1. [Emergency Incident Matrix](#emergency-incident-matrix)
2. [Diagnostic Data Collection](#diagnostic-data-collection)
3. [Runbook: Configuration Drift & Safe Reconciliation](#runbook-configuration-drift--safe-reconciliation)
4. [Runbook: P2P Network Isolation & Sparse Peering](#runbook-p2p-network-isolation--sparse-peering)
5. [Runbook: Mempool Backlog & Transaction Congestion](#runbook-mempool-backlog--transaction-congestion)
6. [Runbook: Port Conflicts & Multi-Node Allocation](#runbook-port-conflicts--multi-node-allocation)
7. [Runbook: Process Supervision & Crash Loops](#runbook-process-supervision--crash-loops)
8. [Runbook: Disaster Recovery & Quarantined Restore](#runbook-disaster-recovery--quarantined-restore)
9. [Log Pattern Matching Reference](#log-pattern-matching-reference)

---

## Emergency Incident Matrix

| Incident Symptom | Immediate Command | Next Runbook Section |
|---|---|---|
| Node configuration diverged from golden database | `cargo run -- --check-config-drift <db> <node> <path>` | [Configuration Drift](#runbook-configuration-drift--safe-reconciliation) |
| Reconcile diverged config with auto-backup | `cargo run -- --reconcile-node-config <db> <node> <path>` | [Configuration Drift](#runbook-configuration-drift--safe-reconciliation) |
| Block sync halted, 0 connected peers | `cargo run -- --peer-health <rpc_endpoint> [n3\|neox]` | [P2P Network Isolation](#runbook-p2p-network-isolation--sparse-peering) |
| Transaction backlog spikes or high memory | `cargo run -- --mempool-status <rpc_endpoint> [n3\|neox]` | [Mempool Congestion](#runbook-mempool-backlog--transaction-congestion) |
| Node fails to start due to port collision | `cargo run -- --workspace-readiness <db>` | [Port Conflicts](#runbook-port-conflicts--multi-node-allocation) |
| Process crashed or orphaned PID | `cargo run -- --node-status <db> <node>` | [Process Supervision](#runbook-process-supervision--crash-loops) |
| Corrupt workspace database | `cargo run -- --workspace-integrity <db>` | [Disaster Recovery](#runbook-disaster-recovery--quarantined-restore) |
| Prepare support escalation bundle | `cargo run -- --export-support-bundle <db> <dir>` | [Diagnostic Data Collection](#diagnostic-data-collection) |

---

## Diagnostic Data Collection

When diagnosing fleet instability or preparing an escalation bundle, follow this systematic order:

### 1. Workspace Readiness & Integrity Assessment
Evaluate overall workspace health, port collisions, and SQLite database integrity:

```bash
# Check port conflicts, missing runtimes, and configuration flaws
cargo run -- --workspace-readiness /path/to/neonexus.db
cargo run -- --workspace-readiness-json /path/to/neonexus.db

# Check workspace database internal consistency
cargo run -- --workspace-integrity /path/to/neonexus.db
```

### 2. Live Node Status & RPC Telemetry
Inspect live runtime process states and dual-family RPC endpoints:

```bash
# Inspect process supervisor status
cargo run -- --node-status /path/to/neonexus.db "node-01"

# Probe RPC health, block height, and response latency
cargo run -- --rpc-health 127.0.0.1:10332
cargo run -- --rpc-health-json 127.0.0.1:10332
```

### 3. Generate Diagnostics Support Bundle
Export sanitized logs, metrics, event journal entries, and configuration states into an incident bundle:

```bash
cargo run -- --export-support-bundle /path/to/neonexus.db /path/to/support_bundles
```

---

## Runbook: Configuration Drift & Safe Reconciliation

### Problem Description
Manual edits to node configuration files on disk or external tooling modifications cause the node to diverge from the workspace golden specification. This leads to subtle state forks, missing RPC methods, or unexpected port bindings.

### Step 1: Detect Configuration Drift
Run the drift auditor to compare the disk file with the database golden spec:

```bash
cargo run -- --check-config-drift /path/to/neonexus.db "node-01" /path/to/config.json
cargo run -- --check-config-drift-json /path/to/neonexus.db "node-01" /path/to/config.json
```

**Diagnostic Output**:
- If clean: `Config clean (hash: <sha256>)`
- If drifted:
  ```text
  Config drift detected for node "node-01":
    Disk SHA-256:   a1b2c3d4...
    Golden SHA-256: f6e5d4c3...
    Differences:
      - P2P port modified (expected 10333, found 10334)
      - MaxGasInvoke modified (expected 50, found 10)
  ```

### Step 2: Reconcile Drift with Automated Backup
If the drift is unintended, execute atomic reconciliation. NeoNexus writes a timestamped backup before atomically replacing the file:

```bash
cargo run -- --reconcile-node-config /path/to/neonexus.db "node-01" /path/to/config.json
```

**Outcome**:
- Backup created at `/path/to/config.json.drift-bak.<TIMESTAMP>`.
- Disk configuration atomically replaced using staged temporary write (`.stage-write`).
- Restart the node to apply reconciled configuration:
  ```bash
  cargo run -- --node-restart /path/to/neonexus.db "node-01"
  ```

---

## Runbook: P2P Network Isolation & Sparse Peering

### Problem Description
The node stops receiving new blocks and block height falls behind the network. The node may be network-isolated (0 peers) or connected to an insufficient number of peers (sparse).

### Step 1: Execute P2P Peer Health Probe
```bash
# For Neo N3 nodes:
cargo run -- --peer-health 127.0.0.1:10332 neo-n3

# For Neo X EVM nodes:
cargo run -- --peer-health 127.0.0.1:8545 neo-x
```

### Step 2: Interpret Peer Health State
- **`healthy`**: Connected peers $\ge 3$. P2P layer is healthy.
- **`sparse`**: Connected peers $> 0$ but $< 3$. Warning: high risk of sync stalls.
- **`isolated`**: Connected peers $= 0$. Critical alert: node is completely disconnected.

### Step 3: Remediation Workflow for `isolated` / `sparse`
1. **Verify P2P Firewall Rules**:
   - Neo N3 MainNet: Port `10333` TCP
   - Neo N3 TestNet: Port `20333` TCP
   - Neo X MainNet: Port `8551` / `30303` TCP
   - Windows PowerShell check:
     ```powershell
     Test-NetConnection -ComputerName seed1.neo.org -Port 10333
     ```
   - Linux check:
     ```bash
     nc -zv seed1.neo.org 10333
     ```
2. **Verify Bootnodes / Seed List**:
   Ensure `protocol.json` (Neo N3) or `config.toml` (Neo X) contains reachable seeds.
3. **Restart Node to Force Discovery**:
   ```bash
   cargo run -- --node-restart /path/to/neonexus.db "node-01"
   ```

---

## Runbook: Mempool Backlog & Transaction Congestion

### Problem Description
The node's transaction pool accumulates an abnormally large volume of unverified or pending transactions, causing elevated memory consumption or delays in transaction submission.

### Step 1: Inspect Mempool Status
```bash
# Query mempool depth and classification
cargo run -- --mempool-status 127.0.0.1:10332 neo-n3
cargo run -- --mempool-status 127.0.0.1:8545 neo-x
```

### Step 2: Interpret Congestion Level
- **`normal`**: Total transactions $< 500$.
- **`elevated`**: Total transactions between $500$ and $2,000$.
- **`congested`**: Total transactions $> 2,000$. Action required.

### Step 3: Remediation Steps
1. **Check Consensus Block Production**:
   Confirm whether validators are producing blocks regularly:
   ```bash
   cargo run -- --rpc-health 127.0.0.1:10332
   ```
   If the block height is not advancing, investigate validator nodes and network consensus state.
2. **Audit Network Fee Floor**:
   In periods of spam attacks, verify that the node's `FeePerByte` policy matches the network committee recommendations.
3. **Governance & Committee Verification**:
   Query active committee snapshot to confirm consensus nodes are online:
   ```bash
   cargo run -- --governance 127.0.0.1:10332
   ```

---

## Runbook: Port Conflicts & Multi-Node Allocation

### Problem Description
A node fails to launch with an error indicating that a P2P or RPC socket port is already bound by another process or another managed node.

### Step 1: Run Workspace Readiness Scan
```bash
cargo run -- --workspace-readiness /path/to/neonexus.db
```
The readiness scan uses the internal `PortPlanner` to detect overlapping ports across all nodes in the workspace and checks host socket availability.

### Step 2: Identify Conflicting Process
- On Windows:
  ```powershell
  Get-NetTCPConnection -LocalPort 10332,10333 -ErrorAction SilentlyContinue | Select-Object LocalAddress,LocalPort,OwningProcess
  ```
- On Linux / macOS:
  ```bash
  lsof -i :10332 -i :10333
  ```

### Step 3: Reassign Ports Cleanly
1. Re-generate configuration with non-colliding ports:
   ```bash
   cargo run -- --generate-node-config neo-rs testnet rocksdb 10334 10335 /path/to/config.toml
   ```
2. Start the node:
   ```bash
   cargo run -- --node-start /path/to/neonexus.db "node-01"
   ```

---

## Runbook: Process Supervision & Crash Loops

### Problem Description
A supervised node repeatedly terminates immediately after launch, or an existing node enters a crash loop triggering watchdog restart backoff.

### Step 1: Query Supervisor State
```bash
cargo run -- --node-status /path/to/neonexus.db "node-01"
```

### Step 2: Inspect Supervised Logs
Examine the tail of standard output and standard error:
```bash
# Logs live under <workspace>/nodes/<node-id>/logs/
tail -n 50 /path/to/workspace/nodes/node-01/logs/stdout.log
tail -n 50 /path/to/workspace/nodes/node-01/logs/stderr.log
```

### Step 3: Rebind Runtime Binary
If the runtime binary path was moved, deleted, or restored from backup quarantine:
```bash
cargo run -- --node-rebind-runtime /path/to/neonexus.db "node-01" /usr/local/bin/neo-node
```

### Step 4: Perform Runtime Smoke Check
Validate that the binary executes properly without crashing in the host environment:
```bash
cargo run -- --runtime-smoke neo-rs /usr/local/bin/neo-node
```

---

## Runbook: Disaster Recovery & Quarantined Restore

### Problem Description
Server migration, disk corruption, or catastrophic host loss requires restoring the workspace from a 0600 plaintext JSON workspace export, NOT encrypted.

### Step 1: Validate Backup Integrity
Verify the backup hash, manifest structure, and database schema before importing:
```bash
cargo run -- --validate-backup /path/to/backups/neonexus-backup-1700000000.json
cargo run -- --validate-backup-json /path/to/backups/neonexus-backup-1700000000.json
```

### Step 2: Import Backup with Execution Quarantine
Restore the workspace into a fresh target database:
```bash
cargo run -- --import-backup /path/to/target.db /path/to/backups/neonexus-backup-1700000000.json
```

> 🛡️ **Zero-Trust Quarantine Protection**: Imported nodes are marked `Quarantined`. Their execution paths and launch arguments are deactivated until an operator explicitly verifies the local environment.

### Step 3: Rebind Trusted Local Runtimes
For each node in the restored workspace, rebind a trusted local runtime binary:
```bash
cargo run -- --node-rebind-runtime /path/to/target.db "node-01" /local/path/to/neo-node
cargo run -- --node-start /path/to/target.db "node-01"
```

---

## Log Pattern Matching Reference

| Log Pattern | Severity | Probable Root Cause | Recommended Action |
|---|---|---|---|
| `fatal: database corrupted` | CRITICAL | Unclean power-off or disk failure | Restore database from fast-sync snapshot |
| `thread '.*' panicked` | CRITICAL | Unhandled exception in Rust runtime | Check backtrace in `stderr.log` |
| `panic: runtime error` | CRITICAL | Go runtime panic in `neo-go` | Check stack trace in `stderr.log` |
| `Geth CRIT` | CRITICAL | EVM engine panic or dirty shutdown | Rebind datadir or repair database |
| `Address already in use` | HIGH | Port collision on P2P/RPC socket | Follow [Port Conflicts](#runbook-port-conflicts--multi-node-allocation) runbook |
| `0 connected peers` | HIGH | Network isolation | Follow [P2P Isolation](#runbook-p2p-network-isolation--sparse-peering) runbook |
| `mempool depth exceeded` | MEDIUM | Transaction congestion | Follow [Mempool Congestion](#runbook-mempool-backlog--transaction-congestion) runbook |
| `consensus timeout` | MEDIUM | Inter-validator network latency | Check peer ping times and committee health |
