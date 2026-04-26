# NeoNexus Node Manager

> Self-hosted Neo N3 node management, simplified

[![Version](https://img.shields.io/badge/version-2.3.0-green.svg)](https://github.com/r3e-network/neo-nexus)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Node.js](https://img.shields.io/badge/node-%3E%3D20.0.0-brightgreen.svg)](https://nodejs.org)
[![Tests](https://img.shields.io/badge/tests-322%20passing-brightgreen.svg)](#)

NeoNexus is a **self-hosted node management platform** for Neo N3. Deploy, monitor, and manage [neo-cli](https://github.com/neo-project/neo-node) and [neo-go](https://github.com/nspcc-dev/neo-go) nodes from a single web dashboard.

## Features

- **One-Click Deploy** — Deploy Neo nodes in minutes without CLI setup
- **Real-Time Monitoring** — Track block height, sync progress, peers, CPU, memory via WebSocket
- **Crash Recovery** — Automatic restart with exponential backoff when nodes crash
- **Plugin Management** — Install and configure neo-cli plugins through the UI
- **Multi-Network** — Mainnet, testnet, and private networks with correct protocol configs
- **Multi-Server** — Monitor multiple NeoNexus instances from one control panel
- **Config Audit** — Detect stale configs, missing plugins, hardfork mismatches
- **Backup/Restore** — JSON export/import of all node configurations
- **Audit Logging** — Track all state-changing operations
- **Secure Signers** — TEE key protection via Intel SGX, AWS Nitro, or custom endpoints
- **SaaS Integrations** — Optional Grafana Cloud, Datadog, Better Stack, Sentry, Slack, Discord, Telegram, and more — just add a token

## Quick Start

### Requirements

- **Node.js** 20+
- **npm** 9+
- **.NET 10+** (for neo-cli nodes)

### Installation

```bash
git clone https://github.com/r3e-network/neo-nexus.git
cd neo-nexus

npm install
npm run build
npm start
```

Open http://localhost:8080 — login with `admin` / `admin`, then change the password.

### Deploy Your First Node

1. Click **Create Node**
2. Select **Type** (neo-cli or neo-go) and **Network** (mainnet/testnet/private)
3. Click **Deploy** — the binary is downloaded, configured, and ready to start

### Use Local neo-node Builds

To use plugins built from a local [neo-node](https://github.com/neo-project/neo-node) checkout:

```bash
cd ~/git/neo-node && git checkout v3.9.2
dotnet build neo-node.sln -c Release

# Start NeoNexus with local plugin path
NEO_PLUGIN_BUILD_DIR=~/git/neo-node/plugins npm start
```

## Architecture

```
                        Web Browser
                            |
                     HTTP / WebSocket
                            |
              +-------------+-------------+
              |     NeoNexus Server        |
              |  Express + SQLite + ws     |
              +--+-------+-------+--------+
                 |       |       |
           +-----+  +---+---+  ++--------+
           |     |  |       |  |         |
        neo-cli  neo-go   neo-cli     neo-go
        Node 1   Node 2   Node 3     Node N
```

**Backend:** TypeScript, Express, better-sqlite3, ws
**Frontend:** React, TanStack Query, Tailwind CSS, Vite
**Processes:** Managed via `child_process.spawn` with PTY support for neo-cli

## Supported Software

| Software | Versions | Networks |
|----------|----------|----------|
| neo-cli | v3.6.0 — v3.9.2 | Mainnet, Testnet, Private |
| neo-go | v0.104.0+ | Mainnet, Testnet, Private |

## Production Features

### Crash Recovery

Nodes that crash are automatically restarted with exponential backoff (2s, 4s, 8s... up to 30s). After 5 consecutive failures the watchdog gives up and alerts. Backoff resets after 5 minutes of stable running.

### Sync Progress

NeoNexus queries seed nodes to determine the network's current block height, then computes `syncProgress = localHeight / networkHeight` for each running node. Stalled nodes (no new blocks for 5 minutes) are flagged.

### Config Audit

`GET /api/nodes/:id/config-audit` compares the on-disk config against the expected generated config and reports:
- Missing or mismatched critical fields (network magic, committee, hardforks)
- Missing plugin DLLs and config files
- Port conflicts
- Binary availability

### Process Management

- **Graceful shutdown:** SIGTERM/SIGINT handlers stop all nodes before exit
- **Zombie detection:** On startup, reconciles DB state with actual running processes
- **PID tracking:** Writes `~/.neonexus/neonexus.pid` for process management
- **Resource limits:** Set per-node memory limits via `settings.resourceLimits.maxMemoryMB`

### Observability

- **Disk monitoring:** Tracks growth rate, alerts at 90%/95% usage, predicts days until full
- **Log retention:** Auto-prunes to 50K rows per node (configurable via `LOG_RETENTION_MAX_ROWS`)
- **Audit log:** All state-changing operations logged to `audit_log` table, queryable via API
- **WebSocket:** Real-time system metrics, node metrics, and log streaming

## Plugin Support (neo-cli)

Install and manage official neo-cli plugins through the UI:

| Plugin | Category | Description |
|--------|----------|-------------|
| LevelDBStore | Storage | Default storage backend |
| RocksDBStore | Storage | Alternative storage backend |
| RpcServer | API | JSON-RPC endpoint |
| RestServer | API | REST API endpoint |
| ApplicationLogs | Core | Transaction and execution logs |
| DBFTPlugin | Core | dBFT consensus |
| TokensTracker | API | NEP-11/NEP-17 token tracking |
| StateService | Core | State root service |
| OracleService | Core | Oracle data integration |
| SQLiteWallet | Tooling | Wallet storage |
| SignClient | Tooling | Remote signing via secure signer |
| StorageDumper | Tooling | Storage export |

## Secure Signers / TEE Key Protection

Neo-cli nodes can reference a secure signing endpoint instead of raw private-key material:

- **Modes:** Software, Intel SGX, AWS Nitro Enclave, Custom
- **Integration:** Auto-wires through the Neo `SignClient` plugin
- **Orchestration:** Generate deploy/unlock/status commands for local signer instances
- **Safety:** NeoNexus never stores WIF, plaintext private keys, or unlock passphrases

## API Reference

### Authentication

```bash
# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}'

# Use token for authenticated requests
curl http://localhost:8080/api/nodes \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Key Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Health check (public) |
| GET | `/api/nodes` | List all nodes |
| POST | `/api/nodes` | Create node |
| POST | `/api/nodes/:id/start` | Start node |
| POST | `/api/nodes/:id/stop` | Stop node |
| GET | `/api/nodes/:id/logs` | Get node logs |
| GET | `/api/nodes/:id/config-audit` | Audit config |
| GET | `/api/nodes/:id/plugins/available` | List available plugins |
| POST | `/api/nodes/:id/plugins` | Install plugin |
| GET | `/api/metrics/system` | System metrics |
| GET | `/api/metrics/network` | Network heights |
| GET | `/api/system/export` | Export configuration |
| POST | `/api/system/restore` | Restore configuration |
| GET | `/api/system/audit-log` | Query audit log |
| GET | `/api/servers` | List remote servers |
| GET | `/api/secure-signers` | List signer profiles |

### WebSocket

Connect to `ws://localhost:8080/ws?token=YOUR_TOKEN` for real-time events:
- `system` — CPU, memory, disk metrics (every 5s)
- `metrics` — Per-node block height, peers, sync progress
- `log` — Live node log entries
- `status` — Node status changes

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PORT` | `8080` | Server port |
| `HOST` | `0.0.0.0` | Bind address |
| `JWT_SECRET` | random (dev) | JWT signing key (required in production) |
| `JWT_EXPIRES_IN` | `24h` | Token expiration |
| `CORS_ORIGIN` | — | Allowed CORS origins (comma-separated) |
| `HTTPS_ENABLED` | `false` | Enable HTTPS |
| `HTTPS_KEY_PATH` | — | TLS key file |
| `HTTPS_CERT_PATH` | — | TLS cert file |
| `LOG_RETENTION_MAX_ROWS` | `50000` | Max log rows per node |
| `NEO_PLUGIN_BUILD_DIR` | — | Local neo-node plugins build directory |

## Development

```bash
# Development mode (hot reload)
npm run dev

# Run tests (272 tests)
npm test

# Type checking
npm run typecheck

# Production build
npm run build
```

## License

MIT

---

<p align="center">
  Built for the Neo community
</p>
