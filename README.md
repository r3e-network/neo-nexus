# NeoNexus Node Manager

> Self-hosted Neo N3 node management, simplified

[![Version](https://img.shields.io/badge/version-2.0.0-green.svg)](https://github.com/r3e-network/neonexus)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Node.js](https://img.shields.io/badge/node-%3E%3D20.0.0-brightgreen.svg)](https://nodejs.org)

NeoNexus is a **self-hosted node management platform** for Neo N3. Deploy, monitor, and manage [neo-cli](https://github.com/neo-project/neo-node) and [neo-go](https://github.com/nspcc-dev/neo-go) nodes from a single web dashboard — no command-line expertise required.

## Features

- 🚀 **One-Click Deploy** — Deploy Neo nodes in minutes without CLI setup
- 📊 **Real-Time Monitoring** — Track block height, peers, CPU, memory, and live node events
- 🔔 **In-App Alerts** — Receive realtime notifications for node errors, warnings, and status changes
- 🔌 **Plugin Management** — Install and configure neo-cli plugins through the UI
- 🔐 **Self-Hosted** — Your nodes, your hardware, your control
- 🌐 **Multi-Server Monitoring** — Track multiple NeoNexus instances from one control panel
- 👁️ **Public Dashboard** — Share node status without exposing sensitive config
- 🧭 **Multi-Network** — Switch between mainnet, testnet, and private networks
- ⚡ **Multi-Node** — Run multiple nodes with automatic port allocation

## Quick Start

### Requirements

- **Node.js** 20+
- **npm** 9+
- **.NET 8+** (for neo-cli nodes)
- **Git** (optional, for development)

### Installation

```bash
# Clone the repository
git clone https://github.com/r3e-network/neonexus.git
cd neo-nexus

# Install dependencies
npm install

# Build the application
npm run build

# Start the server
npm start
```

### First Login

1. Open http://localhost:8080 in your browser
2. Login with default credentials:
   - **Username:** `admin`
   - **Password:** `admin`
3. **⚠️ Change the default password** in Settings → Change Password

### Deploy Your First Node

1. Click **"Create Node"**
2. Select **Node Type:** neo-cli or neo-go
3. Choose **Network:** Mainnet, Testnet, or Private
4. Click **Deploy**

Your node will be automatically downloaded, configured, and ready to start.

## Documentation

- [API Reference](docs/api.md)
- [Remote Access Setup](docs/REMOTE_ACCESS.md)

## What Makes NeoNexus Different?

| Feature            | NeoNexus      | Manual CLI Setup | Cloud Providers |
| ------------------ | ------------- | ---------------- | --------------- |
| **Setup Time**     | 5 minutes     | 2+ hours         | 10 minutes      |
| **Web UI**         | ✅ Built-in   | ❌ CLI only      | ✅ Yes          |
| **Self-Hosted**    | ✅ Yes        | ✅ Yes           | ❌ No           |
| **Multi-Node**     | ✅ Easy       | ⚠️ Complex       | 💰 Expensive    |
| **Cost**           | Free          | Free (labor)     | $$ Monthly      |
| **Plugin Support** | ✅ UI-managed | ⚠️ Manual        | ❌ No           |

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Web Browser                          │
└────────────────────┬────────────────────────────────────┘
                     │ HTTP / WebSocket
┌────────────────────▼────────────────────────────────────┐
│              NeoNexus Node Manager                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │   Express    │  │   Node.js    │  │   SQLite     │  │
│  │    API       │  │   Backend    │  │   Database   │  │
│  └──────────────┘  └──────────────┘  └──────────────┘  │
└────────────────────┬────────────────────────────────────┘
                     │ Process Management
        ┌────────────┼────────────┐
        ▼            ▼            ▼
   ┌─────────┐  ┌─────────┐  ┌─────────┐
   │ neo-cli │  │ neo-go  │  │  ...    │
   │  Node 1 │  │  Node 2 │  │  Node N │
   └─────────┘  └─────────┘  └─────────┘
```

## API Reference

### Authentication

```bash
# Login
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin"}'
```

### Node Management

```bash
# List all nodes (requires auth)
curl http://localhost:8080/api/nodes \
  -H "Authorization: Bearer YOUR_TOKEN"

# Create a new node
curl -X POST http://localhost:8080/api/nodes \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Neo Node",
    "type": "neo-cli",
    "network": "testnet"
  }'

# Start a node
curl -X POST http://localhost:8080/api/nodes/NODE_ID/start \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### Public API (No Authentication)

```bash
# View public node status
curl http://localhost:8080/api/public/nodes

# Check system metrics
curl http://localhost:8080/api/public/metrics/system

# Node health check
curl http://localhost:8080/api/public/nodes/NODE_ID/health
```

See [API Documentation](docs/api.md) for complete reference.

## Supported Node Software

| Software | Version         | Networks         |
| -------- | --------------- | ---------------- |
| neo-cli  | v3.6.0 - v3.9.2 | Mainnet, Testnet |
| neo-go   | v0.104.0+       | Mainnet, Testnet |

## Plugin Support (neo-cli)

NeoNexus can install and configure official neo-cli plugins:

- ApplicationLogs
- LevelDBStore
- RocksDBStore
- RpcNep17Tracker
- RpcSecurity
- RpcServer
- StatesDumper
- StorageDumper
- TokensTracker
- and more...

## Secure Signers / TEE Key Protection

NeoNexus now supports **secure signer profiles** so `neo-cli` nodes can reference a signing endpoint instead of relying on raw private-key material inside NeoNexus-managed configuration.

- **Supported signer modes:** software, Intel SGX, AWS Nitro Enclave, and custom compatible endpoints
- **Upstream integration:** NeoNexus wires secure signers through the Neo `SignClient` plugin
- **Profile data stored by NeoNexus:** endpoint, signer mode, public key / address metadata, encrypted-wallet path reference, unlock strategy, health-check status, and optional local orchestration metadata such as workspace path and Nitro KMS blob path
- **Data NeoNexus does not store:** raw WIF, plaintext private keys, or plaintext unlock material

### Current support boundary

- **`neo-cli`:** supported through auto-wired `SignClient` configuration
- **`neo-go`:** not yet wired to remote secure signers; remains on the standard wallet path until an upstream-compatible remote signing flow is added

### `secure-sign-service-rs` deployment pattern

NeoNexus is designed to work with external signer services such as `secure-sign-service-rs`:

- **Software mode:** local or remote TCP signer for development and testing
- **SGX mode:** HTTP or HTTPS signer backed by an Intel SGX enclave
- **Nitro mode:** `vsock://CID:PORT` signer backed by an AWS Nitro Enclave

Register the signer profile in **Settings**, then attach it to a `neo-cli` node from the node creation or configuration screen.

### Orchestration support

NeoNexus can now help operators with signer lifecycle tasks when the signer runs on the same host and a local `secure-sign-service-rs` workspace is configured:

- generate deploy / unlock / status command templates for software, SGX, and Nitro signer modes
- run safe readiness checks through `secure-sign-tools status` for localhost or vsock-compatible profiles
- fetch Nitro recipient attestation documents through `secure-sign-tools recipient-attestation`
- start Nitro signer unlock with `CiphertextForRecipient` through `secure-sign-tools start-recipient`
- surface bound signer readiness on the node detail page

### Explicit safety boundary

NeoNexus intentionally does **not** accept wallet passphrases or plaintext private keys through the web UI. Manual passphrase unlock remains a host-side terminal workflow. The web UI only orchestrates non-secret operations or Nitro recipient-ciphertext startup paths that avoid exposing plaintext unlock material to NeoNexus.

## Security

- **Self-Hosted:** All data stays on your infrastructure
- **JWT Authentication:** Secure API access with token-based auth
- **Public View Mode:** Separate read-only access for monitoring
- **Isolated Storage:** Each node has isolated data, config, and logs
- **Automatic Warnings:** Prompts password change on first login

## Remote Access

Access your NeoNexus dashboard remotely:

1. **SSH Tunnel:** `ssh -L 8080:localhost:8080 user@your-server`
2. **Nginx Reverse Proxy** with SSL
3. **Cloudflare Tunnel** for secure public access

See [Remote Access Guide](docs/REMOTE_ACCESS.md) for detailed setup.

## Development

```bash
# Run in development mode (hot reload)
npm run dev

# Run tests
npm test

# Build for production
npm run build
npm start
```

## Functionality Status

### 🔐 Authentication & Security

- [x] Login with username/password
- [x] JWT token generation and validation
- [x] Protected routes require authentication
- [x] Session management (24h expiry)
- [x] Default password warning
- [x] Change password functionality
- [x] Logout functionality

### 📊 Dashboard

- [x] System metrics display (CPU, Memory, Disk)
- [x] Node statistics (Total, Running, Errors, Blocks)
- [x] Real-time metrics via WebSocket
- [x] Node list with status badges
- [x] Responsive card layouts

### 🖥️ Node Management

- [x] List all nodes in table view
- [x] View node details
- [x] Node status display
- [x] Node metrics (block height, peers, CPU, memory)
- [x] Node logs viewer
- [x] Start/Stop/Restart node (UI functional)
- [x] Delete node (UI functional)
- [x] Create new node
- [x] Import existing node
- [x] Edit node configuration

### 🔌 Plugin Management

- [x] List available plugins
- [x] Plugin categories and descriptions
- [x] Install/uninstall plugins
- [x] Configure plugin settings

### ⚙️ Settings

- [x] System resources display
- [x] Storage management UI
- [x] Clean old logs
- [x] Export configuration
- [x] Stop all nodes
- [x] Reset all data

### 🌐 Multi-Server Management

- [x] Create/update/delete remote server profiles
- [x] Monitor remote NeoNexus public status and metrics
- [x] View remote node summaries from one dashboard

**Status:** 44/49 features complete (90%)

## Roadmap

- [x] neo-cli support
- [x] neo-go support
- [x] Plugin management UI
- [x] Public dashboard
- [x] Real-time metrics
- [x] Node control operations
- [x] Alert notifications
- [x] Backup/restore
- [x] Multi-server management

## Contributing

Contributions are welcome! Please open an issue or pull request on GitHub.

## License

MIT License

## Support

- 🐛 [Issue Tracker](https://github.com/r3e-network/neonexus/issues)

---

<p align="center">
  Built with ❤️ for the Neo community
</p>
