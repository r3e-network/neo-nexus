# NeoNexus API Reference

Complete API documentation for NeoNexus Node Manager.

**Base URL:** `http://localhost:8080/api`

## Authentication

Most endpoints require authentication via JWT Bearer token.

### Login

Authenticate and receive an access token.

```http
POST /auth/login
```

**Request:**
```json
{
  "username": "admin",
  "password": "admin"
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
  ],
  "count": 1
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
  "version": "v3.9.2"
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

**Response:**
```json
{
  "node": {
    "id": "node-xyz789",
    "name": "My Test Node",
    "type": "neo-cli",
    "network": "testnet",
    "status": "stopped",
    ...
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
  "success": true,
  "message": "Node started",
  "pid": 12345
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
  "success": true,
  "message": "Node stopped"
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
GET /nodes/:id/logs?lines=100&since=1234567890
Authorization: Bearer YOUR_TOKEN
```

**Query Parameters:**
- `lines` — Number of lines to return (default: 100)
- `since` — Timestamp to fetch logs from

**Response:**
```json
{
  "logs": [
    "[2024-03-25 10:00:00] Block 5000000",
    "[2024-03-25 10:00:15] Connected to 10 peers"
  ]
}
```

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
PUT /nodes/:id/plugins/:pluginId/config
Authorization: Bearer YOUR_TOKEN
Content-Type: application/json
```

**Request:**
```json
{
  "Path": "Logs_App",
  "ConsoleOutput": false
}
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
      "lastUpdate": 1774400000000
    }
  ],
  "count": 1
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
  "status": "healthy",
  "totalNodes": 5,
  "runningNodes": 4,
  "errorNodes": 0,
  "timestamp": 1774400000000
}
```

---

## WebSocket API

Real-time updates via WebSocket connection.

### Connect

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

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
| `DOWNLOAD_FAILED` | Failed to download node binary |
| `PORT_UNAVAILABLE` | Required port is in use |
| `PLUGIN_NOT_FOUND` | The specified plugin doesn't exist |

---

## Rate Limiting

API endpoints have rate limits to prevent abuse:

| Endpoint | Limit |
|----------|-------|
| Authentication | 5 requests per 15 minutes |
| Node Control (start/stop) | 10 requests per minute |
| Other endpoints | 100 requests per minute |

Rate limit headers are included in responses:

```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
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

*For questions or issues, see the [GitHub repository](https://github.com/yourusername/neo-nexus).*
