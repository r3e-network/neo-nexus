# NeoNexus API Reference

Reference documentation for the currently documented NeoNexus API surface.

This document is maintained to match the implemented route shapes, but it is not an exhaustive compatibility guarantee for every internal or newly added endpoint.

**Base URL:** `http://localhost:8080/api`

## Authentication

Most endpoints require authentication via JWT Bearer token.

### Initial Setup

Create the first admin account. This endpoint only works while no users exist.

```http
POST /auth/setup
Content-Type: application/json
```

**Request:**
```json
{
  "username": "admin",
  "password": "strong-password-here"
}
```

**Response:**
```json
{
  "message": "Setup completed successfully",
  "user": {
    "id": "uuid",
    "username": "admin",
    "role": "admin"
  },
  "token": "eyJhbGciOiJIUzI1NiIs..."
}
```

### Login

Authenticate and receive an access token.

```http
POST /auth/login
```

**Request:**
```json
{
  "username": "admin",
  "password": "strong-password-here"
}
```

**Response:**
```json
{
  "user": {
    "id": "uuid",
    "username": "admin",
    "role": "admin"
  },
  "token": "eyJhbGciOiJIUzI1NiIs..."
}
```

**Error Responses:**
- `401 Unauthorized` — Invalid credentials
- `400 Bad Request` — Missing username or password

### Logout

Invalidate the current session token.

```http
POST /auth/logout
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "message": "Logged out successfully"
}
```

### Check Setup Status

Check if initial setup is required (no authentication needed).

```http
GET /auth/setup-status
```

**Response:**
```json
{
  "needsSetup": false
}
```

---

## Node Management

### List All Nodes

Get all nodes with their current status and metrics.

```http
GET /nodes
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "nodes": [
    {
      "id": "node-abc123",
      "name": "My Neo Node",
      "type": "neo-cli",
      "network": "testnet",
      "syncMode": "full",
      "version": "v3.9.2",
      "ports": {
        "rpc": 10332,
        "p2p": 10333,
        "websocket": 10334,
        "metrics": 2112
      },
      "process": {
        "status": "running",
        "pid": 12345,
        "uptime": 3600
      },
      "metrics": {
        "blockHeight": 5000000,
        "headerHeight": 5000000,
        "connectedPeers": 10,
        "unconnectedPeers": 5,
        "syncProgress": 100,
        "cpuUsage": 25.5,
        "memoryUsage": 512,
        "lastUpdate": 1774400000000
      },
      "plugins": ["ApplicationLogs"],
      "createdAt": 1774400000000,
      "updatedAt": 1774400000000
    }
  ]
}
```

### Get Node Details

Get detailed information about a specific node.

```http
GET /nodes/:id
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "node": {
    "id": "node-abc123",
    "name": "My Neo Node",
    "type": "neo-cli",
    "network": "testnet",
    "syncMode": "full",
    "version": "v3.9.2",
    "ports": {
      "rpc": 10332,
      "p2p": 10333,
      "websocket": 10334,
      "metrics": 2112
    },
    "paths": {
      "base": "/home/user/.neonexus/nodes/node-abc123",
      "data": "/home/user/.neonexus/nodes/node-abc123/data",
      "logs": "/home/user/.neonexus/nodes/node-abc123/logs",
      "config": "/home/user/.neonexus/nodes/node-abc123/config"
    },
    "settings": {},
    "process": {
      "status": "running"
    },
    "metrics": { ... },
    "plugins": []
  }
}
```

### Import Existing Node

Import an existing Neo node installation.

```http
POST /nodes/import
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "name": "My Existing Node",
  "type": "neo-cli",
  "existingPath": "/home/user/neo-cli",
  "pid": 12345,
  "network": "testnet",
  "version": "v3.9.2",
  "ports": {
    "rpc": 10332,
    "p2p": 10333
  }
}
```

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Node display name |
| `type` | string | No | `neo-cli` or `neo-go` (auto-detected if not specified) |
| `existingPath` | string | Yes | Full path to existing node directory |
| `pid` | number | No | Process ID if node is currently running |
| `network` | string | No | `mainnet`, `testnet`, or `private` (auto-detected) |
| `version` | string | No | Node version (auto-detected) |
| `ports` | object | No | Port configuration (auto-detected) |

**Response:**
```json
{
  "node": {
    "id": "node-imported-xyz789",
    "name": "My Existing Node",
    "type": "neo-cli",
    "network": "testnet",
    "status": "running",
    ...
  }
}
```

### Detect Node Configuration

Analyze a directory to detect node configuration without importing.

```http
POST /nodes/detect
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "path": "/home/user/neo-cli"
}
```

**Response:**
```json
{
  "detected": {
    "type": "neo-cli",
    "network": "testnet",
    "version": "v3.9.2",
    "ports": {
      "rpc": 10332,
      "p2p": 10333
    },
    "dataPath": "/home/user/neo-cli/Data",
    "configPath": "/home/user/neo-cli/config.json",
    "isRunning": true
  },
  "validation": {
    "valid": true,
    "errors": []
  },
  "canImport": true
}
```

### Scan Directory for Nodes

Scan a directory for multiple node installations.

```http
POST /nodes/scan
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "path": "/home/user/nodes"
}
```

**Response:**
```json
{
  "path": "/home/user/nodes",
  "nodes": [
    {
      "path": "/home/user/nodes/neo-cli-mainnet",
      "type": "neo-cli"
    },
    {
      "path": "/home/user/nodes/neo-go-testnet",
      "type": "neo-go"
    }
  ],
  "count": 2
}
```

### Create Node

Deploy a new Neo node.

```http
POST /nodes
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "name": "My Test Node",
  "type": "neo-cli",
  "network": "testnet",
  "syncMode": "full",
  "version": "v3.9.2",
  "settings": {
    "storageEngine": "rocksdb",
    "syncStrategy": "fast-sync"
  }
}
```

**Parameters:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Node display name |
| `type` | string | Yes | `neo-cli` or `neo-go` |
| `network` | string | Yes | `mainnet`, `testnet`, or `private` |
| `syncMode` | string | No | `full` (default) or `light` |
| `version` | string | No | Node version (auto-detected if not specified) |
| `settings.storageEngine` | string | No | `leveldb` (default) or `rocksdb` |
| `settings.syncStrategy` | string | No | `full`, `light`, or `fast-sync` |

**Response:**
```json
{
  "node": {
    "id": "node-xyz789",
    "name": "My Test Node",
    "type": "neo-cli",
    "network": "testnet",
    "process": {
      "status": "stopped"
    },
    "settings": {
      "storageEngine": "rocksdb",
      "syncStrategy": "fast-sync"
    }
  }
}
```

**Error Responses:**
- `400 Bad Request` — Invalid node type or network
- `409 Conflict` — Node name already exists

### Delete Node

Remove a node and all its data.

```http
DELETE /nodes/:id
Authorization: Bearer YOUR_TOKEN
```

**Error Responses:**
- `400 Bad Request` — Cannot delete running node (stop first)
- `404 Not Found` — Node does not exist

### Start Node

Start a stopped node.

```http
POST /nodes/:id/start
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "node": {
    "id": "node-abc123",
    "process": {
      "status": "running",
      "pid": 12345
    }
  }
}
```

### Stop Node

Stop a running node gracefully.

```http
POST /nodes/:id/stop
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "node": {
    "id": "node-abc123",
    "process": {
      "status": "stopped"
    }
  }
}
```

### Restart Node

Restart a node (stop then start).

```http
POST /nodes/:id/restart
Authorization: Bearer YOUR_TOKEN
```

### Get Node Logs

Retrieve node log output.

```http
GET /nodes/:id/logs?count=100
Authorization: Bearer YOUR_TOKEN
```

**Query Parameters:**
- `count` — Number of entries to return, clamped to 1-1000 (default: 100)

**Response:**
```json
{
  "logs": [
    "[2024-03-25 10:00:00] Block 5000000",
    "[2024-03-25 10:00:15] Connected to 10 peers"
  ]
}
```

### Additional Node Operations

| Method | Path | Description |
|--------|------|-------------|
| POST | `/nodes/scan` | Scan a directory for importable Neo node installations |
| POST | `/nodes/:id/ownership` | Change an imported node ownership mode |
| GET | `/nodes/:id/storage` | Get storage usage for a node |
| POST | `/nodes/:id/storage/clean` | Remove old log files for a node |
| GET | `/nodes/:id/config-audit` | Compare a node config against generated expectations |
| GET | `/nodes/:id/signer-health` | Check the secure signer bound to a node |

---

## Role Orchestration

All role orchestration endpoints require an authenticated admin. Roles let operators save or apply a complete node identity: storage engine, sync strategy, plugin desired state, node settings, data-context policy, warnings, and prerequisites.

### Built-in Roles

| Role ID | Name | Notes |
|---------|------|-------|
| `builtin-rpc-api` | RPC / API Node | Enables RPC and fast-sync oriented defaults |
| `builtin-state` | State Node | Enables StateService, RPC, RocksDB, and fast sync |
| `builtin-oracle` | Oracle Node | Enables OracleService, RPC, and fast sync |
| `builtin-consensus` | Consensus Node | Enables DBFTPlugin and full sync; warns about validator signing material |
| `builtin-indexer` | Indexer Node | Enables ApplicationLogs, TokensTracker, RPC, RocksDB |
| `builtin-secure-signer-client` | Secure Signer Client | Enables SignClient and requires a signer profile |

### List Roles

```http
GET /node-roles
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "roles": [
    {
      "id": "builtin-rpc-api",
      "name": "RPC / API Node",
      "kind": "builtin",
      "nodeTypes": ["neo-cli"],
      "profile": {
        "storageEngine": "leveldb",
        "plugins": [{ "id": "RpcServer", "enabled": true, "config": {} }],
        "dataContext": {
          "mode": "reuse-or-create",
          "labelTemplate": "rpc-{network}-{storageEngine}"
        },
        "sync": { "strategy": "fast-sync", "allowCheckpoint": true }
      }
    }
  ]
}
```

### Get Role

```http
GET /node-roles/:roleId
Authorization: Bearer YOUR_TOKEN
```

### Create Custom Role

```http
POST /node-roles
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "name": "Archive RPC",
  "description": "RPC node with application logs and full sync",
  "nodeTypes": ["neo-cli"],
  "profile": {
    "storageEngine": "rocksdb",
    "settings": {
      "relay": true,
      "maxConnections": 100
    },
    "plugins": [
      { "id": "RpcServer", "enabled": true, "config": {} },
      { "id": "ApplicationLogs", "enabled": true, "config": {} }
    ],
    "dataContext": {
      "mode": "reuse-or-create",
      "labelTemplate": "archive-rpc-{network}-{storageEngine}"
    },
    "sync": {
      "strategy": "full",
      "allowCheckpoint": false
    },
    "warnings": ["Requires enough disk for archive data."]
  }
}
```

Custom role plugins are supported for `neo-cli` roles. Supported plugin ids are `ApplicationLogs`, `DBFTPlugin`, `LevelDBStore`, `OracleService`, `RestServer`, `RocksDBStore`, `RpcServer`, `SignClient`, `SQLiteWallet`, `StateService`, `StorageDumper`, and `TokensTracker`.

### Preview Role Application

```http
POST /node-roles/:roleId/plan
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "nodeId": "node-abc123",
  "storageEngine": "rocksdb"
}
```

**Response:**
```json
{
  "plan": {
    "nodeId": "node-abc123",
    "roleId": "builtin-state",
    "roleName": "State Node",
    "requiresRestart": true,
    "changes": [
      { "type": "storage", "summary": "Switch storage engine to rocksdb" },
      { "type": "plugin", "summary": "Enable StateService" }
    ],
    "warnings": []
  }
}
```

### Apply Role

```http
POST /node-roles/:roleId/apply
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "nodeId": "node-abc123",
  "storageEngine": "rocksdb"
}
```

**Response:**
```json
{
  "application": {
    "id": "role-application-uuid",
    "nodeId": "node-abc123",
    "roleId": "builtin-state",
    "roleName": "State Node",
    "status": "applied"
  },
  "node": {
    "id": "node-abc123",
    "settings": {
      "storageEngine": "rocksdb",
      "syncStrategy": "fast-sync",
      "activeDataContextId": "ctx-state-testnet-rocksdb"
    }
  }
}
```

### List Role Application History

```http
GET /nodes/:id/role-applications
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "applications": [
    {
      "id": "role-application-uuid",
      "nodeId": "node-abc123",
      "roleId": "builtin-state",
      "roleName": "State Node",
      "status": "applied",
      "appliedAt": 1774400000000
    }
  ]
}
```

---

## Data Contexts

Data contexts isolate blockchain data for a node. Each context records storage engine, sync strategy, optional checkpoint metadata, and optional fast-sync snapshot id. The node must be stopped before creating the first active context or activating a different context.

### List Data Contexts

```http
GET /nodes/:id/data-contexts
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "contexts": [
    {
      "id": "ctx-state-testnet-rocksdb",
      "nodeId": "node-abc123",
      "label": "state-testnet-rocksdb",
      "storageEngine": "rocksdb",
      "syncStrategy": "fast-sync",
      "checkpointHeight": 5000000,
      "checkpointHash": "0xabc123",
      "snapshotId": "snapshot-uuid",
      "active": true
    }
  ],
  "activeContext": {
    "id": "ctx-state-testnet-rocksdb",
    "active": true
  }
}
```

### Create Data Context

```http
POST /nodes/:id/data-contexts
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "label": "rpc-mainnet-fast",
  "storageEngine": "leveldb",
  "syncStrategy": "fast-sync",
  "checkpointHeight": 5000000,
  "checkpointHash": "0xabc123",
  "snapshotId": "snapshot-uuid"
}
```

**Response:** `201 Created`
```json
{
  "context": {
    "id": "ctx-rpc-mainnet-fast",
    "label": "rpc-mainnet-fast",
    "storageEngine": "leveldb",
    "syncStrategy": "fast-sync",
    "active": false
  }
}
```

### Activate Data Context

```http
POST /nodes/:id/data-contexts/:contextId/activate
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "context": {
    "id": "ctx-rpc-mainnet-fast",
    "active": true
  },
  "node": {
    "id": "node-abc123",
    "settings": {
      "storageEngine": "leveldb",
      "syncStrategy": "fast-sync",
      "activeDataContextId": "ctx-rpc-mainnet-fast"
    }
  }
}
```

---

## Fast Sync Snapshots

Fast-sync snapshots are manifest records. NeoNexus can register local, HTTPS URL, or catalog sources, verify the SHA-256 checksum, and download HTTPS packages into the managed downloads directory. Clients cannot set `trusted`; verification records `lastVerifiedAt` and size metadata after the server validates the digest.

### List Snapshots

```http
GET /fast-sync/snapshots
Authorization: Bearer YOUR_TOKEN
```

### Register Snapshot

```http
POST /fast-sync/snapshots
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "name": "N3 testnet rocksdb 5M",
  "sourceType": "url",
  "source": "https://snapshots.example.com/n3-testnet-rocksdb-5000000.tar.zst",
  "chain": "n3",
  "network": "testnet",
  "nodeType": "neo-cli",
  "storageEngine": "rocksdb",
  "height": 5000000,
  "blockHash": "0xabc123",
  "sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  "sizeBytes": 1073741824,
  "signature": "optional-detached-signature"
}
```

**Response:** `201 Created`
```json
{
  "snapshot": {
    "id": "snapshot-uuid",
    "name": "N3 testnet rocksdb 5M",
    "sourceType": "url",
    "trusted": false,
    "height": 5000000
  }
}
```

Valid `sourceType` values are `local`, `url`, and `catalog`. `chain` can be `n3` or `x`; N3 snapshots use `mainnet`, `testnet`, or `private` with `neo-cli`/`neo-go`, while Neo X snapshots use `neox-mainnet` or `neox-testnet` with `neox-go`.

### Verify Snapshot

```http
POST /fast-sync/snapshots/:id/verify
Authorization: Bearer YOUR_TOKEN
```

### Download Snapshot

```http
POST /fast-sync/snapshots/:id/download
Authorization: Bearer YOUR_TOKEN
```

Download currently requires an HTTPS `source`. The returned `snapshot` includes updated verification metadata when checksum validation succeeds.

---

## Private Network Plans

Private network plans generate complete Neo N3 local network layouts. Supported templates are `single`, `four`, and `seven`. Plans include network magic, per-node ports, seed list, validator count, standby committee public keys, generated N3 addresses, storage engine, and neo-cli plugin presets for the selected roles.

### List Plans

```http
GET /private-networks/plans
Authorization: Bearer YOUR_TOKEN
```

### Create Plan

```http
POST /private-networks/plans
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "name": "local-single",
  "template": "single",
  "nodeType": "neo-cli",
  "storageEngine": "rocksdb",
  "networkMagic": 123456789,
  "baseRpcPort": 20332,
  "baseP2pPort": 20333,
  "baseWebsocketPort": 20334,
  "baseMetricsPort": 22112,
  "nodeNamePrefix": "local-single"
}
```

**Response:** `201 Created`
```json
{
  "plan": {
    "id": "private-plan-uuid",
    "name": "local-single",
    "template": "single",
    "networkMagic": 123456789,
    "status": "draft",
    "plan": {
      "seedList": ["127.0.0.1:20333"],
      "validatorsCount": 1,
      "nodes": [
        {
          "name": "local-single-1",
          "type": "neo-cli",
          "roleIds": ["builtin-consensus", "builtin-rpc-api"],
          "storageEngine": "rocksdb",
          "ports": {
            "rpc": 20332,
            "p2p": 20333,
            "websocket": 20334,
            "metrics": 22112
          },
          "publicKey": "02...",
          "address": "N..."
        }
      ]
    }
  }
}
```

`nodeType` defaults to `neo-cli`, `storageEngine` defaults to `leveldb`, `networkMagic` is generated when omitted, and generated ports increment by 10 for each node. `POST /private-networks/plan` is accepted as a compatibility alias for creating a plan.

### Get Plan

```http
GET /private-networks/plans/:id
Authorization: Bearer YOUR_TOKEN
```

### Preview Configuration Snapshot

```http
GET /private-networks/plans/:id/configuration-snapshot
Authorization: Bearer YOUR_TOKEN
```

The snapshot is compatible with the system restore shape and can be inspected before applying a plan.

### Apply Plan

```http
POST /private-networks/plans/:id/apply
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "replaceExisting": false
}
```

**Response:**
```json
{
  "result": {
    "restoredCount": 1,
    "skippedCount": 0,
    "failedCount": 0
  },
  "plan": {
    "id": "private-plan-uuid",
    "status": "applied"
  }
}
```

`POST /private-networks/:planId/apply` is accepted as a compatibility alias.

---

## Plugin Management

### List Available Plugins

Get list of installable plugins for a node.

```http
GET /nodes/:id/plugins/available
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "plugins": [
    {
      "name": "ApplicationLogs",
      "description": "Logs application events",
      "version": "3.9.2"
    }
  ]
}
```

### List Installed Plugins

Get currently installed plugins.

```http
GET /nodes/:id/plugins
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "plugins": [
    {
      "name": "ApplicationLogs",
      "enabled": true,
      "config": {
        "Path": "Logs_App"
      }
    }
  ]
}
```

### Install Plugin

Install a plugin on a node.

```http
POST /nodes/:id/plugins
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "pluginId": "ApplicationLogs"
}
```

### Uninstall Plugin

Remove a plugin from a node.

```http
DELETE /nodes/:id/plugins/:pluginId
Authorization: Bearer YOUR_TOKEN
```

### Configure Plugin

Update plugin configuration.

```http
PUT /nodes/:id/plugins/:pluginId
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "config": {
    "Path": "Logs_App",
    "ConsoleOutput": false
  }
}
```

**Response:**
```json
{
  "plugins": [
    {
      "name": "ApplicationLogs",
      "enabled": true,
      "config": {
        "Path": "Logs_App",
        "ConsoleOutput": false
      }
    }
  ]
}
```

### Enable Plugin

```http
POST /nodes/:id/plugins/:pluginId/enable
Authorization: Bearer YOUR_TOKEN
```

### Disable Plugin

```http
POST /nodes/:id/plugins/:pluginId/disable
Authorization: Bearer YOUR_TOKEN
```

---

## Metrics & Monitoring

### System Metrics

Get host system metrics.

```http
GET /metrics/system
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "metrics": {
    "cpu": {
      "usage": 45.5,
      "cores": 8
    },
    "memory": {
      "total": 16000000000,
      "used": 8000000000,
      "free": 8000000000,
      "percentage": 50
    },
    "disk": {
      "total": 1000000000000,
      "used": 500000000000,
      "free": 500000000000,
      "percentage": 50
    },
    "network": {
      "rx": 1024,
      "tx": 2048
    },
    "timestamp": 1774400000000
  }
}
```

### Node Metrics

Get metrics for a specific node.

```http
GET /metrics/nodes/:id
Authorization: Bearer YOUR_TOKEN
```

**Response:**
```json
{
  "metrics": {
    "blockHeight": 5000000,
    "headerHeight": 5000000,
    "connectedPeers": 10,
    "unconnectedPeers": 5,
    "syncProgress": 100,
    "cpuUsage": 25.5,
    "memoryUsage": 512,
    "lastUpdate": 1774400000000
  }
}
```

---

## Public API (No Authentication)

These endpoints are publicly accessible for monitoring without login.

### List Public Nodes

Get node status without sensitive configuration.

```http
GET /public/nodes
```

**Response:**
```json
{
  "nodes": [
    {
      "id": "node-abc123",
      "name": "Public Node",
      "type": "neo-cli",
      "network": "testnet",
      "status": "running",
      "version": "v3.9.2",
      "metrics": {
        "blockHeight": 5000000,
        "headerHeight": 5000000,
        "connectedPeers": 10,
        "syncProgress": 100
      },
      "uptime": 3600,
      "lastUpdate": 1774400000000
    }
  ]
}
```

> **Note:** Excludes `paths`, `settings`, and other sensitive data.

### Get Public Node Details

```http
GET /public/nodes/:id
```

### Check Node Health

Quick health check endpoint.

```http
GET /public/nodes/:id/health
```

**Response:**
```json
{
  "healthy": true,
  "status": "running",
  "blockHeight": 5000000,
  "peers": 10,
  "timestamp": 1774400000000
}
```

### System Status (Public)

```http
GET /public/metrics/system
```

**Response:**
```json
{
  "metrics": {
    "cpu": { "usage": 45.5, "cores": 8 },
    "memory": { "percentage": 50, "used": 8000000000, "total": 16000000000 },
    "disk": { "percentage": 50, "used": 500000000000, "total": 1000000000000 },
    "timestamp": 1774400000000
  }
}
```

### Overall System Status

```http
GET /public/status
```

**Response:**
```json
{
  "status": {
    "totalNodes": 5,
    "runningNodes": 4,
    "syncingNodes": 1,
    "errorNodes": 0,
    "totalBlocks": 25000000,
    "totalPeers": 42,
    "timestamp": 1774400000000
  }
}
```

### Node Metrics Summary (Public)

```http
GET /public/metrics/nodes
```

**Response:**
```json
{
  "metrics": [
    {
      "id": "node-abc123",
      "name": "Public Node",
      "status": "running",
      "blockHeight": 5000000,
      "peers": 10,
      "cpuUsage": 25.5,
      "memoryUsage": 512,
      "lastUpdate": 1774400000000
    }
  ]
}
```

---

## WebSocket API

Real-time updates via WebSocket connection.

### Connect

```javascript
const ws = new WebSocket('ws://localhost:8080/ws', ['neonexus.auth', 'YOUR_TOKEN']);

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(data);
};
```

### Message Types

**Node Metrics Update:**
```json
{
  "type": "metrics",
  "nodeId": "node-abc123",
  "data": {
    "blockHeight": 5000001,
    "connectedPeers": 10
  }
}
```

**Node Status Change:**
```json
{
  "type": "status",
  "nodeId": "node-abc123",
  "data": {
    "status": "running",
    "pid": 12345
  }
}
```

**System Metrics:**
```json
{
  "type": "system",
  "data": {
    "cpu": { "usage": 45.5 },
    "memory": { "percentage": 50 }
  }
}
```

---

## Admin & Integration Routes

These routes require an authenticated admin unless noted.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/auth/register` | Create an additional admin or viewer account |
| GET | `/auth/users` | List users |
| DELETE | `/auth/users/:id` | Delete a user account |
| PUT | `/auth/password` | Change the current user's password |
| POST | `/system/logs/clean` | Clean old managed node logs |
| GET | `/system/export` | Download a configuration snapshot |
| POST | `/system/restore` | Restore a configuration snapshot |
| POST | `/system/nodes/stop-all` | Stop all running managed nodes |
| POST | `/system/reset` | Delete all managed node definitions and managed directories |
| GET | `/system/audit-log` | Read the audit log |
| GET | `/servers` | List configured remote NeoNexus instances |
| POST | `/servers` | Add a remote NeoNexus instance |
| PUT | `/servers/:id` | Update a remote NeoNexus instance |
| DELETE | `/servers/:id` | Delete a remote NeoNexus instance |
| GET | `/secure-signers` | List secure signer profiles |
| POST | `/secure-signers` | Create a secure signer profile |
| POST | `/secure-signers/:id/test` | Test a secure signer profile |
| POST | `/secure-signers/:id/attestation` | Fetch signer attestation metadata |
| GET | `/integrations` | List SaaS integration configuration state |
| PUT | `/integrations/:id` | Save and enable/disable an integration |
| POST | `/integrations/:id/test` | Test an integration |
| DELETE | `/integrations/:id` | Remove an integration configuration |

Viewer accounts are limited to read-only routes. State-changing node, plugin, server, system, signer, and integration routes require admin access.

---

## Hermes Agent

In-app AI agent. Per-user API key (Anthropic / OpenAI / OpenAI-compatible). Tools inherit the user's role: viewers get read-only fleet inspection; admins additionally get start/stop/restart and plugin enable/disable.

Disabled by default. Set `NEONEXUS_ENABLE_HERMES_AGENT=true` on the server to turn on.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/agent/health` | Returns `{enabled: bool}` based on the feature flag |
| GET | `/agent/settings` | Current user's settings (API key redacted to last 4 chars) |
| PUT | `/agent/settings` | Save provider/model/apiKey/baseUrl |
| DELETE | `/agent/settings` | Remove the user's settings |
| GET | `/agent/conversations` | List the user's conversations |
| POST | `/agent/conversations` | Create a new conversation |
| GET | `/agent/conversations/:id` | Conversation header + full message list |
| DELETE | `/agent/conversations/:id` | Delete a conversation |
| POST | `/agent/conversations/:id/messages` | Non-streaming send (WS preferred for real-time) |
| POST | `/agent/conversations/:id/cancel` | Cancel the in-flight turn for that conversation |

### WebSocket message shape

Send (client → server):
```json
{ "type": "agent.send", "conversationId": "<id>", "text": "what nodes do I have?" }
{ "type": "agent.cancel", "conversationId": "<id>" }
```

Receive (server → client) — events use the prefix `agent.`:

- `agent.message_start` — `{ messageId, role }`
- `agent.delta` — `{ messageId, text }` for each streamed text chunk
- `agent.tool_use` — `{ messageId, toolUseId, name, input }` when the model calls a tool
- `agent.tool_result` — `{ messageId, toolUseId, output, isError }` after the tool runs
- `agent.message_end` — `{ messageId }`
- `agent.complete` — `{ conversationId }` once the turn finishes
- `agent.error` — `{ conversationId, error }` if the provider or a tool fails

### Tool surface

Read tools (any role): `list_nodes`, `get_node`, `get_node_logs`, `list_plugins`, `get_system_metrics`, `get_network_height`, `list_remote_servers`, `list_integrations`.

Control tools (admin only): `start_node`, `stop_node`, `restart_node`, `set_plugin_enabled`. Every control invocation writes to the audit log with action `agent.node.start` / `.stop` / `.restart` / `agent.plugin.toggle`.

Out of scope for v1: deleting nodes, importing/creating nodes, password changes, snapshot restore, on-chain transaction signing.

---

## Error Handling

All errors follow this format:

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE",
  "details": {}
}
```

### Common HTTP Status Codes

| Code | Meaning |
|------|---------|
| `200` | Success |
| `201` | Created |
| `400` | Bad Request — Invalid parameters |
| `401` | Unauthorized — Invalid or missing token |
| `403` | Forbidden — Insufficient permissions |
| `404` | Not Found — Resource doesn't exist |
| `409` | Conflict — Resource already exists |
| `500` | Internal Server Error |

### Error Codes

| Code | Description |
|------|-------------|
| `NODE_NOT_FOUND` | The specified node does not exist |
| `NODE_ALREADY_RUNNING` | Cannot start a node that's already running |
| `NODE_NOT_RUNNING` | Cannot stop a node that's not running |
| `INVALID_NODE_TYPE` | Unsupported node type |
| `INVALID_NETWORK` | Unsupported network |
| `INVALID_STORAGE_ENGINE` | Storage engine must be `leveldb` or `rocksdb` |
| `INVALID_SYNC_STRATEGY` | Sync strategy must be `full`, `light`, or `fast-sync` |
| `DOWNLOAD_FAILED` | Failed to download node binary |
| `PORT_UNAVAILABLE` | Required port is in use |
| `PLUGIN_NOT_FOUND` | The specified plugin doesn't exist |
| `NODE_NOT_STOPPED` | Node must be stopped before changing role or data context |
| `NODE_ROLE_NOT_FOUND` | The requested role does not exist |
| `NODE_ROLE_REQUEST_INVALID` | Custom role or role application request is invalid |
| `NODE_ROLE_INCOMPATIBLE` | Role does not support the target node type |
| `NODE_ROLE_PLUGIN_UNSUPPORTED` | Role plugin changes are not supported for the target node type |
| `NODE_ROLE_PREREQUISITE_MISSING` | A role prerequisite, such as signer configuration, is missing |
| `DATA_CONTEXT_INVALID` | Data context request is invalid |
| `FAST_SYNC_SNAPSHOT_INVALID` | Snapshot manifest is invalid |
| `FAST_SYNC_SNAPSHOT_NOT_FOUND` | Snapshot manifest does not exist |
| `FAST_SYNC_VERIFY_UNSUPPORTED` | Snapshot cannot be verified by the requested method |
| `FAST_SYNC_DOWNLOAD_UNSUPPORTED` | Snapshot source cannot be downloaded automatically |
| `FAST_SYNC_SNAPSHOT_HASH_MISMATCH` | Snapshot checksum did not match the manifest |
| `FAST_SYNC_DOWNLOAD_FAILED` | Snapshot download failed |
| `PRIVATE_NETWORK_PLAN_INVALID` | Private network plan request is invalid |
| `PRIVATE_NETWORK_PLAN_NOT_FOUND` | Private network plan does not exist |
| `PRIVATE_NETWORK_PLAN_CORRUPT` | Stored private network plan cannot be parsed safely |
| `REMOTE_SERVER_URL_PRIVATE_TARGET` | Remote server URL targets a private or local address |
| `SIGNER_ENDPOINT_PRIVATE_TARGET` | Secure signer endpoint targets a private or local address |
| `INTEGRATION_URL_PRIVATE_TARGET` | Integration URL targets a private or local address |

---

## Rate Limiting

API endpoints have rate limits to prevent abuse:

| Endpoint | Limit |
|----------|-------|
| Authentication | 5 requests per 15 minutes |
| Node Control (start/stop) | 10 requests per minute |
| Other endpoints, including public routes | 1000 requests per 15 minutes |

Rate limit headers are included in responses:

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1774400000
```

---

## SDK Examples

### JavaScript/TypeScript

```typescript
class NeoNexusClient {
  private baseUrl = 'http://localhost:8080/api';
  private token: string;

  async login(username: string, password: string) {
    const res = await fetch(`${this.baseUrl}/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ username, password })
    });
    const data = await res.json();
    this.token = data.token;
    return data;
  }

  async listNodes() {
    const res = await fetch(`${this.baseUrl}/nodes`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    return res.json();
  }

  async startNode(nodeId: string) {
    const res = await fetch(`${this.baseUrl}/nodes/${nodeId}/start`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    return res.json();
  }
}
```

### Python

```python
import requests

class NeoNexusClient:
    def __init__(self, base_url='http://localhost:8080/api'):
        self.base_url = base_url
        self.token = None

    def login(self, username, password):
        res = requests.post(f'{self.base_url}/auth/login', json={
            'username': username,
            'password': password
        })
        data = res.json()
        self.token = data['token']
        return data

    def list_nodes(self):
        res = requests.get(f'{self.base_url}/nodes', headers={
            'Authorization': f'Bearer {self.token}'
        })
        return res.json()

    def start_node(self, node_id):
        res = requests.post(f'{self.base_url}/nodes/{node_id}/start', headers={
            'Authorization': f'Bearer {self.token}'
        })
        return res.json()
```

### cURL

```bash
# Login and save token
TOKEN=$(curl -s -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}' | jq -r '.token')

# Use token for subsequent requests
curl -s http://localhost:8080/api/nodes \
  -H "Authorization: Bearer $TOKEN" | jq .
```

---

*For questions or issues, see the [GitHub repository](https://github.com/r3e-network/neonexus).*
