# Kill and Recovery Validation Guide

## Overview

This document describes the **manual validation procedure** for testing NeoNexus's ability to recover nodes after out-of-band process termination. This is **Row 6** of the acceptance matrix defined in [client-acceptance.md](client-acceptance.md).

### Why Manual?

The acceptance script `scripts/client-acceptance-matrix.sh` deliberately does not perform automated kill/recovery tests because:

1. **Safety**: Killing production consensus nodes can disrupt network participation
2. **Complexity**: Requires precise PID tracking and state observation across multiple processes
3. **Verification**: Human oversight needed to confirm proper fencing and recovery semantics

---

## Prerequisites

Before beginning, ensure:

- ✅ A node is registered and **Running** with a known PID
- ✅ You have shell access (Linux) or PowerShell (Windows) on the host
- ✅ The node database path is known (used for status checks)
- ✅ Node is stable (not syncing, no active crashes)

---

## Step-by-Step Procedure

### Step 1: Capture Initial State

**Record current node status:**

```bash
./neo-nexus --node-status <database.db> <node-name>
```

Expected output should include:
- `status: Running`
- `pid: <PID>` (e.g., `pid: 12345`)
- `log_path`: Absolute path to node log file
- `health_history`: Recent RPC health checks showing healthy

**Save this information:**

```bash
INITIAL_PID=<the_pid_from_status_output>
DB_PATH=/path/to/database.db
NODE_NAME=my-node
```

---

### Step 2: Verify Node is Healthy Before Kill

**Test RPC endpoint before termination:**

```bash
# For N3 nodes
curl -X POST -d '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}' http://<rpc-endpoint>/

# For Neo X nodes  
curl -X POST -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://<rpc-endpoint>/
```

Expected: Response returns block number (not error)

---

### Step 3: Terminate the Node Process

#### Linux (SIGKILL)

```bash
kill -9 $INITIAL_PID
```

⚠️ **Warning**: SIGKILL cannot be caught by the process; it terminates immediately.

#### Windows (TerminateProcess)

```powershell
Get-Process -Id $INITIAL_PID | Stop-Process -Force
```

Or use Task Manager → Right-click process → "End Process Tree"

---

### Step 4: Wait for Watchdog Detection

NeoNexus supervisor runs continuously (either via systemd, Windows service, or watchtower). It should detect the crashed node within seconds.

**Check system logs:**

```bash
# Linux - systemd journal
sudo journalctl -u neo-nexus.service -f | grep -i "$NODE_NAME"

# Windows - Event Viewer or service output logs
Get-EventLog -LogName Application -Newest 50 | Select-String $NODE_NAME
```

Expected: Log entries indicate crash detection and recovery scheduling

---

### Step 5: Monitor Status Transition

Poll node status every 5 seconds:

```bash
for i in {1..20}; do
  ./neo-nexus --node-status $DB_PATH $NODE_NAME
  sleep 5
done
```

**Expected progression:**

```
Status: Crashed      # Immediate detection (within ~10s)
↓
Status: Starting     # Recovery scheduled (after backoff delay)
↓
Status: Running      # New PID assigned, node restarted
```

**Recovery timing constraints:**

| Scenario | Backoff Delay | Expected Total Time |
|----------|--------------|---------------------|
| First attempt | 5 seconds | ~15-20 seconds from crash |
| Second failure | 10 seconds | Additional 10-15 seconds |
| Third failure | 20 seconds | Additional 20-30 seconds |
| Max retries | Configured per node | Should exhaust within ~2 minutes |

---

### Step 6: Verify New PID (Fencing Check)

After recovery completes:

```bash
./neo-nexus --node-status $DB_PATH $NODE_NAME
```

**Critical verification points:**

✅ **NEW PID**: Record shows different PID than original (`$INITIAL_PID + 1` or higher)

✅ **No Overlap**: Original PID's process group was terminated cleanly

✅ **No Collateral Damage**: Other processes at that PID address were not affected

✅ **Generation Increment**: Operation ledger shows new generation counter

**Confirm old PID is gone:**

```bash
# Linux
ps -p $INITIAL_PID

# Windows
Get-Process -Id $INITIAL_PID -ErrorAction SilentlyContinue
```

Expected: No process found (old PID either died or reassigned safely)

---

### Step 7: Test PID Reuse Safety

This is the **most critical fence check**. If your system recycled the same PID quickly:

**Simulate scenario (optional advanced test):**

```bash
# Start a harmless dummy process that will occupy the freed PID
(while true; do sleep 1; done) &
DUMMY_PID=$!

# Then try to restart the node
./neo-nexus --node-start $DB_PATH $NODE_NAME

# Check outcome
./neo-nexus --node-status $DB_PATH $NODE_NAME
```

**Expected behavior:**
- If dummy process already owns PID → Node starts normally with NEW PID
- Status remains `Stopped` if collision detected (not forced kill of innocent process)
- Error message includes: `"pid XXX belongs to a different process"`

---

### Step 8: Verify Operational Ledger

Check the operation history shows proper fenced lifecycle:

```bash
./neo-nexus --export-event-journal $DB_PATH /tmp/journal-export info node
```

Look for events like:
- `NodeCrashed` - Original termination
- `RecoveryScheduled` - Watchdog decision
- `NodeStartStarted` - Recovery launch began
- `NodeStartCompleted` - Successfully restarted
- All with matching `generation` counter increments

---

### Step 9: Validate RPC Health After Recovery

Confirm node fully recovered and is responsive:

```bash
# N3 node
curl -X POST -d '{"jsonrpc":"2.0","method":"getblockcount","params":[],"id":1}' http://<rpc-endpoint>/

# Neo X node
curl -X POST -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' http://<rpc-endpoint>/
```

Expected: Same or greater block count than pre-crash, no errors

**Cross-check with RPC health CLI:**

```bash
./neo-nexus --rpc-health-json <rpc-endpoint> neo-n3
```

Expected JSON:
```json
{
  "report": {
    "status": "healthy",
    "version_info": { /* ... */ },
    "network_observation": { "peer_count": >0 }
  }
}
```

---

### Step 10: Stop Node Gracefully (Cleanup)

After successful recovery test:

```bash
./neo-nexus --node-stop $DB_PATH $NODE_NAME
```

Verify status transitions back to `Stopped` cleanly.

---

## Acceptance Criteria Summary

| Criterion | Pass Condition | Severity |
|-----------|---------------|----------|
| Crash detection | Node status changes to `Crashed` within 30s | P0 |
| Recovery initiation | New start operation begins within 60s total | P0 |
| PID change | New PID ≠ Old PID (unless intentional reuse) | P0 |
| Fencing integrity | No other process killed during PID reuse | P0 |
| RPC responsiveness | Node responds to queries post-recovery | P0 |
| Ledger consistency | Operation history shows complete chain | P1 |
| No data loss | Config, wallet, chain state preserved | P0 |

---

## Known Edge Cases

### Case 1: PID Reuse Collision
**Scenario**: OS reuses exact PID for another process before recovery

**Behavior**:
- Node stays `Stopped` with error message about PID collision
- Operator must manually investigate which process occupies PID
- No unsafe killing of unrelated process occurs ✅

### Case 2: Slow Restart Due to Resource Contention
**Scenario**: Disk I/O bottleneck delays binary loading or config parsing

**Behavior**:
- Operation remains in `reserved` phase until resource available
- Stale operation timeout = 120 seconds
- After timeout, abandoned and re-triable

### Case 3: Crash During Configuration Rewrite
**Scenario**: Node killed mid-write to managed config file

**Behavior**:
- Backup exists from transaction phase
- Recovery may need manual intervention (config restore from backup)
- Not automatically handled (manual step)

### Case 4: Consensus Signer Dependency
**Scenario**: Node requires external signer (HSM/SignClient) which also crashed

**Behavior**:
- Node restart fails readiness check (signer unavailable)
- Remains in `Starting` → `Failed` loop
- Requires signer restoration before node can succeed

---

## Reporting Results

After completing validation:

**Successful run:**
```text
✓ Node crashed detected within 15 seconds
✓ Recovery initiated after 5-second backoff
✓ New PID assigned (old=12345, new=12346)
✓ RPC healthy post-recovery (block height maintained)
✓ Operation ledger shows complete lifecycle
✓ No collateral process damage
Result: PASS
```

**Failed run (document specifics):**
```text
✗ Issue: [Description]
✗ Symptoms: [What happened instead of expected]
✗ Logs: [Relevant journal/event log snippets]
✗ Root cause hypothesis: [If determinable]
Result: FAIL
```

---

## References

- [client-acceptance.md](client-acceptance.md#6-kill--recovery) - Matrix specification
- [os-service-deployment.md](os-service-deployment.md) - Service manager configuration
- [`src/repository/operations.rs`](src/repository/operations.rs#390-402) - Stale operation cleanup logic
- [`src/supervision.rs`](src/supervision.rs) - Process watchdog implementation

---

## Maintenance Notes

- Update backoff values in `RETRY_BACKOFF_SECS` constant if observed performance needs tuning
- Monitor PID reuse patterns on heavily-loaded systems where recycling happens rapidly
- Document any platform-specific behaviors (Linux vs Windows termination semantics)

---

*Last updated: 2026-09-06*  
*Author: NeoNexus Engineering Team*
