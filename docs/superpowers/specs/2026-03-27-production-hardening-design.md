# Production Hardening Design Spec

**Date:** 2026-03-27
**Goal:** Close the 10 highest-impact gaps preventing NeoNexus from being used as unattended blockchain infrastructure.

## 1. Crash Auto-Restart (WatchdogManager)

**Problem:** A crashed node stays down until manual restart.

**Design:** New class `src/core/WatchdogManager.ts` that wraps node lifecycle with automatic recovery.

- Listens to `nodeStatus` events from `NodeManager`
- When a node transitions to `error` or exits unexpectedly (status was `running` → process gone), triggers restart
- Exponential backoff: 2s, 4s, 8s, 16s, 30s cap
- Max 5 consecutive failures before marking node as `failed` and emitting a `nodeWatchdogExhausted` event
- Backoff resets after 5 minutes of stable running
- Persists restart count in a `node_watchdog` table: `(node_id, consecutive_failures, last_restart_at, backoff_ms, enabled)`
- User can disable auto-restart per node via API: `PUT /api/nodes/:id { settings: { autoRestart: false } }`
- Default: enabled for all nodes

**Integration:** NodeManager calls `watchdog.onNodeStarted(nodeId)` and `watchdog.onNodeExited(nodeId, wasExpected)`. WatchdogManager schedules restart via `setTimeout`.

## 2. Sync Progress Tracking

**Problem:** `syncProgress` is hardcoded to 0. Operators can't tell if nodes are synced.

**Design:** Extend the existing metrics polling interval in `server.ts`.

- Every 30 seconds (existing interval), for each running node:
  - Query the node's RPC `getblockcount` for local height
  - Compare against a cached `networkHeight` for that network
- `networkHeight` is fetched every 60 seconds by querying a known seed node's RPC endpoint (try each seed until one responds)
  - Mainnet seeds: `seed1.neo.org:10332` ... `seed5.neo.org:10332`
  - Testnet seeds: `seed1t5.neo.org:20332` ... `seed5t5.neo.org:20332`
- `syncProgress` = `Math.min(localHeight / networkHeight, 1.0)` (0 if networkHeight unknown)
- Stale detection: if `localHeight` hasn't changed for 5 minutes while `networkHeight` has, emit `nodeStalled` event
- Store `network_height` in memory (not DB — ephemeral), keyed by network name
- Add `syncProgress`, `networkHeight`, `syncLag` fields to the node metrics response

## 3. Graceful Shutdown

**Problem:** Killing the server orphans node processes and risks DB corruption.

**Design:** Add signal handlers in `src/server.ts` `startServer()` function.

```
SIGTERM/SIGINT →
  1. Set shutting_down flag (reject new requests with 503)
  2. Stop accepting new WebSocket connections
  3. For each running node: call nodeManager.stopNode(id) with 10s timeout
  4. If any node doesn't stop in 10s: force kill (SIGKILL)
  5. Close all WebSocket connections
  6. Close HTTP server
  7. Close database
  8. process.exit(0)
```

- Also write a PID file at startup (`~/.neonexus/neonexus.pid`), remove on clean exit
- Timeout the entire shutdown at 30 seconds, then force exit

## 4. Log Retention

**Problem:** Log table grows unbounded, eventually fills disk.

**Design:** Add a periodic cleanup to the existing metrics interval.

- Every 10 minutes, run: `DELETE FROM logs WHERE id NOT IN (SELECT id FROM logs WHERE node_id = ? ORDER BY timestamp DESC LIMIT ?)` for each node
- Default max: 50,000 rows per node (configurable via `LOG_RETENTION_MAX_ROWS` env var)
- Also add a global cap: total log rows across all nodes capped at 500,000
- On cleanup, emit count of deleted rows to server log
- Add `DELETE FROM logs WHERE timestamp < ?` endpoint at `POST /api/system/logs/clean` (already exists, enhance with row-based cleanup)

## 5. Zombie Process Detection

**Problem:** Stale neo-cli/neo-go processes from previous runs block ports.

**Design:** On `NodeManager` initialization:

- Read all nodes from DB that have `status = 'running'` with a stored PID
- For each, check if that PID is still alive (`process.kill(pid, 0)`) and matches expected command
- If alive and matches: adopt the process (re-attach event handlers)
- If alive but doesn't match (different process at same PID): reset status to `stopped`
- If dead: reset status to `stopped`
- Also scan for processes matching `neo-cli|neo-go` via `pgrep` that aren't tracked — log a warning

**PID storage:** Add `pid` column to `nodes` table (already exists as part of process status updates — store in a dedicated `last_pid` column).

## 6. Disk Space Monitoring

**Problem:** Silent disk exhaustion leads to node crashes and DB corruption.

**Design:** Extend `MetricsCollector.getSystemMetrics()`.

- Track disk usage percentage and absolute free space
- On each metrics broadcast (every 30s), check thresholds:
  - < 10% free: emit `diskWarning` WebSocket message
  - < 5% free: emit `diskCritical` WebSocket message
- Calculate growth rate: store last 10 disk readings, compute bytes/hour
- Calculate `daysUntilFull` = freeBytes / bytesPerHour / 24
- Add to system metrics response: `disk.daysUntilFull`, `disk.growthRatePerHour`

## 7. Download Checksum Verification

**Problem:** Downloaded binaries are not verified — supply chain risk.

**Design:** After downloading a binary:

- Fetch the release's checksum file from GitHub (if available)
- Neo-go releases include SHA256 checksums
- Compute SHA256 of downloaded file
- Compare — reject if mismatch
- If no checksum available: log a warning but proceed (don't break existing flow)
- Store verified hash in `downloads` metadata for re-validation

## 8. Process Resource Limits

**Problem:** Node processes can consume unlimited CPU/memory.

**Design:** In `spawnProcess()`:

- Accept optional `maxMemoryMB` parameter
- For neo-go: set via `GOMEMLIMIT` environment variable
- For neo-cli: set via `DOTNET_GCHeapHardLimit` environment variable (hex bytes)
- Default: no limit (preserve current behavior)
- Configurable per-node via `settings.resourceLimits: { maxMemoryMB?: number }`
- Monitor actual usage via `/proc/{pid}/status` on Linux (VmRSS field)
- If usage exceeds 90% of limit for 5 minutes: emit warning

## 9. Audit Logging

**Problem:** No trail of who did what — compliance gap.

**Design:** New `audit_log` table:

```sql
CREATE TABLE audit_log (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  timestamp INTEGER NOT NULL,
  user_id TEXT,
  username TEXT,
  action TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id TEXT,
  details TEXT,
  ip_address TEXT
);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp);
```

- Log all state-changing API operations: node create/delete/start/stop/restart, plugin install/uninstall, config changes, auth events, system actions
- Middleware approach: wrap route handlers to capture before/after state
- Keep last 100,000 audit entries (prune with log retention)
- Expose via `GET /api/system/audit-log?limit=100&offset=0`

## 10. Dynamic Hardfork Heights

**Problem:** Hardcoded hardfork heights require code changes for network upgrades.

**Design:** Already partially implemented — `generateNeoCliConfig` reads from downloaded binary's bundled config files.

- Extend to neo-go: when generating `protocol.yml`, check if neo-go binary ships a default config
  - neo-go doesn't bundle config files, so this uses ConfigManager's hardcoded values
- Add a new endpoint `GET /api/system/network-info/:network` that returns current hardfork data
- When the user updates a node's version, regenerate config and run config-audit automatically
- On node version change: show diff of what changed in hardfork heights / protocol settings

## Architecture Notes

- All new code follows existing patterns (TypeScript, express routes, better-sqlite3)
- No new dependencies required
- WatchdogManager, audit logging, and disk monitoring are independent modules
- Graceful shutdown integrates into existing server lifecycle
- All features are backward-compatible (existing nodes continue to work)
