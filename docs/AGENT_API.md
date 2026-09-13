# NeoNexus Agent Protocol & Automation API Reference

**Version**: 4.3.1  
**Last Updated**: September 12, 2026  
**Default Base URL**: `http://127.0.0.1:8080`

---

## Table of Contents

1. [Overview & Automation Surfaces](#overview--automation-surfaces)
2. [Authentication & RBAC](#authentication--rbac)
   - [Session Cookie Authentication](#session-cookie-authentication)
   - [Bearer Token Authentication](#bearer-token-authentication)
   - [Managing API Tokens (Web & CLI)](#managing-api-tokens-web--cli)
3. [Security & Rate Limiting Boundaries](#security--rate-limiting-boundaries)
4. [HTTP REST Endpoints](#http-rest-endpoints)
   - [Public Liveness: GET /healthz](#get-healthz)
   - [Public Status: GET /api/public/status](#get-apipublicstatus)
   - [Fleet Inventory: GET /api/fleet](#get-apifleet)
   - [Readiness Diagnostics: GET /api/readiness](#get-apireadiness)
   - [Prometheus Metrics: GET /api/metrics-prometheus](#get-apimetrics-prometheus)
   - [Cloud Node IaC: GET /api/nodes/{id}/iac](#get-apinodesidiac)
   - [Cloud Fleet Manifest: GET /api/fleet/iac](#get-apifleetiac)
   - [Hermes AI Copilot MCP: POST /api/nodes/{id}/mcp](#post-apinodesidmcp)
   - [Signer Relay: POST /signer/api/v1/...](#signer-relay-endpoints)
5. [Headless CLI & JSON Automation Reference](#headless-cli--json-automation-reference)
   - [Fleet & Node Lifecycle Supervision](#fleet--node-lifecycle-supervision)
   - [P2P Network Isolation & Health Probe](#p2p-network-isolation--health-probe)
   - [Mempool Depth & Congestion Telemetry](#mempool-depth--congestion-telemetry)
   - [Configuration Drift Detection & Reconciliation](#configuration-drift-detection--reconciliation)
   - [State Checkpoints, Quarantined Backup & Restore](#state-checkpoints-quarantined-backup--restore)
   - [Workspace Readiness & Integrity Probes](#workspace-readiness--integrity-probes)
6. [Error Handling Specification](#error-handling-specification)

---

## Overview & Automation Surfaces

NeoNexus provides two complementary, production-grade automation surfaces designed for continuous integration, infrastructure orchestration, and observability agents:

1. **HTTP REST & Metrics Service**: Fast, read-oriented API served directly from the embedded axum/tokio server. Supports role-based Bearer tokens, Prometheus scrapers, and browser sessions.
2. **Headless CLI JSON Automation**: Rich, scriptable CLI actions returning structured JSON with guaranteed zero-panic exit codes (`0` on success, non-zero on failure), executing identically in CI/CD pipelines, local terminals, and remote bastion servers without requiring a running web daemon.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           NeoNexus Core Facade                          │
├────────────────────────────────────┬────────────────────────────────────┤
│         HTTP Web & REST API        │        Headless CLI Engine         │
│  - /api/fleet                      │  - --peer-health-json              │
│  - /api/readiness                  │  - --mempool-status-json           │
│  - /api/metrics-prometheus         │  - --check-config-drift-json       │
│  - /signer/api/v1/...              │  - --reconcile-node-config-json    │
│  - Bearer Token RBAC Middleware    │  - --workspace-readiness-json      │
└────────────────────────────────────┴────────────────────────────────────┘
```

---

## Authentication & RBAC

NeoNexus enforces strict defense-in-depth authentication. Unauthenticated requests to protected endpoints return `401 Unauthorized`.

### Session Cookie Authentication

Browser-based requests use encrypted, short-lived HTTP session cookies:
- **Cookie Name**: `neonexus_session`
- **Flags**: `HttpOnly; SameSite=Strict; Path=/` (plus `Secure` when HTTPS origin is configured)
- **TTL**: 12-hour sliding window

### Bearer Token Authentication

Automated clients, CI/CD runners, and monitoring agents authenticate via standard HTTP `Authorization` headers:

```http
GET /api/fleet HTTP/1.1
Host: 127.0.0.1:8080
Authorization: Bearer <HEX_ENCODED_32_BYTE_SECRET>
```

#### Token Permission Scopes

| Scope | Allowed Operations | Typical Use Case |
|---|---|---|
| `read_fleet` | `/api/fleet`, `/api/logs`, `/api/plugins`, `/api/metrics-prometheus`, `/public-metrics`, `/api/nodes/{id}/{metrics,iac}`, `/api/fleet/iac` | Prometheus scrapers, Grafana dashboards |
| `read_readiness` | `/api/readiness` | Deployment gates, pre-flight sanity checks |
| `admin_all` | All API endpoints and headless token operations | Orchestration systems, automated operators |
| `hermes_agent:<node-id>` | **Only** `/api/nodes/<node-id>/*` | One instance's guest copilot |

#### Instance Confinement

A token whose *only* grants are `hermes_agent:<node-id>` is **confined** to that
instance, the way a cloud instance profile is. The authentication boundary
rejects it with `403` on any path outside `/api/nodes/<node-id>/`, including
every fleet-wide endpoint and any other instance's routes. This is decided from
the request path at the single authentication choke point, so an endpoint added
later is confined by default rather than reachable until it is explicitly gated.

Adding any fleet-wide grant (`read_fleet`, `read_readiness`, `admin_all`) to the
same token removes the confinement — the operator asked for something broader
and gets it.

> 🔒 **Security Guarantee**: NeoNexus never persists plaintext tokens. Only the SHA-256 cryptographic digest is stored in the workspace database. The plaintext secret is displayed **exactly once** upon creation.

### Managing API Tokens (Web & CLI)

Tokens can be minted, inspected, and revoked through either the Web UI (`/settings/api-tokens`) or directly via the headless CLI:

#### Create Scoped API Token

```bash
cargo run -- --create-api-token /path/to/neonexus.db "ci-orchestrator" admin_all
```

**Standard Output**:
```text
Created API token "ci-orchestrator" (admin_all)
ID:     019553f2-89ab-7000-8000-000000000001
Secret: 4a8b2c1d9e0f3a4b5c6d7e8f90123456789abcdef0123456789abcdef0123456

Store this secret safely. It will not be shown again.
```

#### List Active Tokens

```bash
cargo run -- --list-api-tokens /path/to/neonexus.db
cargo run -- --list-api-tokens-json /path/to/neonexus.db
```

**JSON Output Format**:
```json
[
  {
    "id": "019553f2-89ab-7000-8000-000000000001",
    "name": "ci-orchestrator",
    "permissions": ["admin_all"],
    "created_at": 1726135200,
    "last_used_at": 1726135320
  }
]
```

#### Revoke Token

```bash
cargo run -- --revoke-api-token /path/to/neonexus.db "019553f2-89ab-7000-8000-000000000001"
```

---

## Security & Rate Limiting Boundaries

NeoNexus rejects external tampering and resource exhaustion through kernel-level and middleware safeguards:

| Boundary | Enforcement Mechanism | Failure Behavior |
|---|---|---|
| **Signer Relay** | Max 1 MiB body (`MAX_REQUEST_BODY_BYTES`), 15s timeout | `413 Payload Too Large` / `408 Request Timeout` |
| **Plugin Uploads** | Bounded multipart streaming (up to 2 GiB), path traversal checks | Fails before writing files to disk |
| **Origin Hardening** | Exact match on `Origin` / `Referer` headers for state changes | `403 Forbidden` |
| **Brute-Force Guard** | Exponential delay on consecutive failed login attempts | Progressive backoff (up to 5m) |

---

## HTTP REST Endpoints

### GET /healthz

Public endpoint verifying process liveness and basic server responsiveness.

- **Auth**: None (Public)
- **Response Code**: `200 OK`
- **Response Body**: `ok`

---

### GET /api/public/status

Inventory-minimized public status endpoint for load balancers.

- **Auth**: None (Public)
- **Response Code**: `200 OK`
- **Response Format**:
```json
{
  "service": "neonexus",
  "status": "operational",
  "version": "4.3.1"
}
```

---

### GET /api/fleet

Returns real-time inventory and status of all configured nodes across Neo N3 and Neo X runtimes.

- **Auth**: Bearer token with `read_fleet` or active session
- **Response Code**: `200 OK`
- **Response Format**:
```json
{
  "nodes": [
    {
      "id": "node-01",
      "name": "N3-Validator-Alpha",
      "node_type": "neo-cli",
      "chain_family": "neo-n3",
      "network": "mainnet",
      "status": "running",
      "pid": 48210,
      "p2p_port": 10333,
      "rpc_port": 10332,
      "rpc_health": "healthy",
      "current_height": 6245100
    },
    {
      "id": "node-02",
      "name": "NeoX-EVM-Beta",
      "node_type": "neox-geth",
      "chain_family": "neo-x",
      "network": "testnet",
      "status": "running",
      "pid": 48215,
      "p2p_port": 20333,
      "rpc_port": 8545,
      "rpc_health": "healthy",
      "current_height": 1845120
    }
  ]
}
```

---

### GET /api/readiness

Aggregated fleet diagnostic score, configuration status, and operational blockers.

- **Auth**: Bearer token with `read_readiness` or active session
- **Response Code**: `200 OK`
- **Response Format**:
```json
{
  "readiness_score": 100,
  "ready_nodes": 2,
  "total_nodes": 2,
  "warning_count": 0,
  "blocker_count": 0,
  "port_conflicts": 0,
  "findings": []
}
```

---

### GET /api/metrics-prometheus

Exposes comprehensive fleet metrics formatted for Prometheus scrapers.

- **Auth**: Bearer token with `read_fleet` or active session
- **Response Code**: `200 OK`
- **Content-Type**: `text/plain; version=0.0.4; charset=utf-8`
- **Sample Output**:
```prometheus
# HELP neonexus_node_running Process running state (1 = running, 0 = stopped)
# TYPE neonexus_node_running gauge
neonexus_node_running{node="node-01",type="neo-cli",chain="neo-n3"} 1
neonexus_node_running{node="node-02",type="neox-geth",chain="neo-x"} 1

# HELP neonexus_node_block_height Latest observed block height
# TYPE neonexus_node_block_height gauge
neonexus_node_block_height{node="node-01",chain="neo-n3"} 6245100
neonexus_node_block_height{node="node-02",chain="neo-x"} 1845120

# HELP neonexus_node_rpc_latency_seconds RPC probe response time
# TYPE neonexus_node_rpc_latency_seconds gauge
neonexus_node_rpc_latency_seconds{node="node-01"} 0.0034
neonexus_node_rpc_latency_seconds{node="node-02"} 0.0028
```

---

### GET /api/nodes/{id}/iac

Exports declarative Infrastructure-as-Code (IaC) specifications and cloud launch templates for a specific node.

- **Auth**: Bearer token with `read_fleet` or active session
- **Query Parameters**:
  - `format`: `k8s` (Kubernetes Pod YAML), `docker` (Docker shell script), `json` (Declarative instance spec), `cli` (Reproducible CLI command). Default: `json`.
- **Response Code**: `200 OK`
- **Headers**:
  - `Content-Type`: `application/x-yaml`, `text/x-shellscript`, or `application/json`
  - `Content-Disposition`: `attachment; filename="node-<name>.<ext>"`

---

### GET /api/fleet/iac

Generates unified multi-node cluster deployment manifests for the entire fleet.

- **Auth**: Bearer token with `read_fleet` or active session
- **Query Parameters**:
  - `format`: `compose` (Docker Compose `compose.yaml`), `k8s` (Multi-document Kubernetes YAML). Default: `compose`.
- **Response Code**: `200 OK`
- **Headers**:
  - `Content-Type`: `application/x-yaml`
  - `Content-Disposition`: `attachment; filename="docker-compose.yml"` or `"k8s-fleet.yaml"`

---

### POST /api/nodes/{id}/mcp

Nous Hermes AI Copilot Model Context Protocol (MCP) JSON-RPC 2.0 endpoint for autonomous node supervision and self-healing.

- **Auth**: Bearer token with `hermes_agent:<node-id>` or active session
- **Protocol**: MCP JSON-RPC 2.0 (`tools/list`, `tools/call`)
- **Precondition**: the instance must have an **enabled** agent association.
  Holding a credential is not enrolment — an instance whose operator has not
  switched the copilot on exposes no tool surface and answers `403` with
  JSON-RPC error `-32002`. Provisioning a scoped token from the instance page
  enrols the agent (with autonomous healing left off).
- **Available Autonomous Tools**:
  - `get_node_status`: Inspect real-time health, height, peers, and sync progress
  - `get_node_config`: Inspect declarative role, network, and signer lease
  - `get_node_logs`: Stream recent log observation tail
  - `restart_node`: Self-healing restart. Requires the **autonomous healing**
    grant on the association — a missing association is not consent — and is
    bounded by a circuit breaker at 5 restarts/hour. `403` / `-32001` otherwise.
  - `stop_node`: Gracefully quiesce process and release ports
  - `start_node`: Supervised instance launch
  - `smoke_test_node`: SRE binary smoke sweep and health diagnostics
  - `get_node_iac`: Export cloud launch template, Kubernetes Pod YAML, or Docker run script
- **Operator-only tool**: `take_snapshot` writes a whole-workspace backup
  covering every instance, so it is neither listed for nor callable by a
  confined credential (`403` / `-32003`). A session or `admin_all` token gets it.

---

### Signer Relay Endpoints

The signer relay forwards transaction, consensus, and EIP-191 signatures to configured local-wallet, local-signer, or NeoOS custody services.

- `POST /signer/api/v1/sign/transaction`: Sign Neo N3 or Neo X transaction
- `POST /signer/api/v1/sign/consensus`: Sign Neo N3 dBFT consensus payload
- `POST /signer/api/v1/sign/eip191-fulfillment`: Sign Neo X EIP-191 personal message
- `GET /signer/api/v1/keys/{id}`: Inspect signer key metadata without revealing private keys

---

## Headless CLI & JSON Automation Reference

All operational capabilities are fully accessible via headless CLI commands with dual text and JSON formats.

### Fleet & Node Lifecycle Supervision

#### Query Fleet Status
```bash
cargo run -- --node-list-json /path/to/neonexus.db
```

#### Query Single Node
```bash
cargo run -- --node-status-json /path/to/neonexus.db "node-01"
```

#### Node Lifecycle Control
```bash
cargo run -- --node-start /path/to/neonexus.db "node-01"
cargo run -- --node-stop /path/to/neonexus.db "node-01"
cargo run -- --node-restart /path/to/neonexus.db "node-01"
```

---

### P2P Network Isolation & Health Probe

Probes peer connectivity and classifies network health for Neo N3 and Neo X nodes. Detects network isolation and insufficient peer density:

```bash
cargo run -- --peer-health-json 127.0.0.1:10332 neo-n3
cargo run -- --peer-health-json 127.0.0.1:8545 neo-x
```

**JSON Output Schema**:
```json
{
  "connected_peers": 14,
  "min_expected_peers": 3,
  "family": "neo-n3",
  "status": "healthy",
  "endpoint": "127.0.0.1:10332"
}
```

**Peer Health Status Classification**:
- `healthy`: Connected peers $\ge$ `min_expected_peers`.
- `sparse`: Connected peers $> 0$ but below `min_expected_peers` (under-peered warning).
- `isolated`: Connected peers $= 0$ (critical network isolation alert).

---

### Mempool Depth & Congestion Telemetry

Inspects transaction pool depth and evaluates backlog congestion across Neo N3 and Neo X:

```bash
cargo run -- --mempool-status-json 127.0.0.1:10332 neo-n3
cargo run -- --mempool-status-json 127.0.0.1:8545 neo-x
```

**JSON Output Schema**:
```json
{
  "verified_count": 85,
  "unverified_count": 12,
  "total_count": 97,
  "capacity": 50000,
  "family": "neo-n3",
  "status": "normal",
  "endpoint": "127.0.0.1:10332"
}
```

**Mempool Status Classification**:
- `normal`: Total count $< 500$ transactions.
- `elevated`: Total count between $500$ and $2,000$ transactions.
- `congested`: Total count $> 2,000$ transactions.

---

### Configuration Drift Detection & Reconciliation

Audits configuration divergence between disk files and workspace golden records, and performs atomic, zero-loss reconciliation with automated backups:

#### Check Configuration Drift
```bash
cargo run -- --check-config-drift-json /path/to/neonexus.db "node-01" /path/to/config.json
```

**JSON Output Schema**:
```json
{
  "node_id": "node-01",
  "config_path": "/path/to/config.json",
  "is_drifted": true,
  "disk_hash": "a1b2c3d4e5f6...32bytes",
  "golden_hash": "f6e5d4c3b2a1...32bytes",
  "differences": [
    "P2P port changed from 10333 to 10334",
    "RPC MaxGasInvoke changed from 50 to 10"
  ]
}
```

#### Reconcile Drifted Configuration
```bash
cargo run -- --reconcile-node-config-json /path/to/neonexus.db "node-01" /path/to/config.json
```

**JSON Output Schema**:
```json
{
  "node_id": "node-01",
  "config_path": "/path/to/config.json",
  "reconciled": true,
  "backup_path": "/path/to/config.json.drift-bak.1726135200"
}
```

---

### State Checkpoints, Quarantined Backup & Restore

#### Export Encrypted Backup Archive
```bash
cargo run -- --export-backup-json /path/to/neonexus.db /path/to/backups
```

#### Validate Backup Integrity & Manifest
```bash
cargo run -- --validate-backup-json /path/to/backups/backup.tar.gz
```

#### Restore Backup (Zero-Trust Quarantined)
```bash
cargo run -- --import-backup-json /path/to/target.db /path/to/backups/backup.tar.gz
```

> 🛡️ **Quarantine Contract**: Restored nodes have their binary and execution arguments quarantined until explicitly rebound with `--node-rebind-runtime` to prevent unauthorized execution of untrusted paths.

---

### Workspace Readiness & Integrity Probes

#### Workspace Readiness Check
Evaluates port conflicts across P2P, RPC, and sidecars, verifies runtime presence, and identifies configuration flaws:
```bash
cargo run -- --workspace-readiness-json /path/to/neonexus.db
```

#### Workspace Database Integrity
```bash
cargo run -- --workspace-integrity-json /path/to/neonexus.db
```

---

## Error Handling Specification

API errors and CLI JSON errors follow a consistent structure:

```json
{
  "error": {
    "code": "CONFIG_DRIFT_DETECTED",
    "message": "Disk configuration differs from database golden spec",
    "details": [
      "P2P port mismatch"
    ],
    "suggestion": "Run --reconcile-node-config to re-align disk file with golden configuration"
  }
}
```

### Common Error Codes

| Code | HTTP Status | Description | Action |
|---|---|---|---|
| `NODE_NOT_FOUND` | 404 | Node identifier not registered in database | Check `--node-list` |
| `PORT_CONFLICT` | 409 | Configured port is already bound | Reassign port via port planner |
| `PEER_ISOLATION` | 503 | Node has 0 connected peers | Check firewall and P2P seed nodes |
| `UNAUTHORIZED` | 401 | Missing or invalid Bearer token | Generate token with `--create-api-token` |
| `FORBIDDEN` | 403 | Token lacks required permission | Check token scope (`read_fleet`, `admin_all`) |
| `PAYLOAD_TOO_LARGE`| 413 | Request payload exceeds maximum buffer | Reduce payload size |
