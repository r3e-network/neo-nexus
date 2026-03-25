# NeoNexus Node Manager

> Self-hosted Neo N3 node management, simplified

[![Version](https://img.shields.io/badge/version-2.0.0-green.svg)](https://github.com/yourusername/neo-nexus)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Node.js](https://img.shields.io/badge/node-%3E%3D20.0.0-brightgreen.svg)](https://nodejs.org)

NeoNexus is a **self-hosted node management platform** for Neo N3. Deploy, monitor, and manage [neo-cli](https://github.com/neo-project/neo-node) and [neo-go](https://github.com/nspcc-dev/neo-go) nodes from a single web dashboard — no command-line expertise required.

![Dashboard Preview](docs/images/dashboard-preview.png)

## Features

- 🚀 **One-Click Deploy** — Deploy Neo nodes in minutes without CLI setup
- 📊 **Real-Time Monitoring** — Track block height, peers, CPU, and memory live
- 🔌 **Plugin Management** — Install and configure neo-cli plugins through the UI
- 🔐 **Self-Hosted** — Your nodes, your hardware, your control
- 👁️ **Public Dashboard** — Share node status without exposing sensitive config
- 🌐 **Multi-Network** — Switch between mainnet, testnet, and private networks
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
git clone https://github.com/yourusername/neo-nexus.git
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

## Screenshots

| Dashboard | Node Details | Public View |
|-----------|--------------|-------------|
| ![Dashboard](docs/images/dashboard.png) | ![Details](docs/images/details.png) | ![Public](docs/images/public.png) |

## Documentation

- [Installation Guide](docs/installation.md)
- [Configuration](docs/configuration.md)
- [API Reference](docs/api.md)
- [Remote Access Setup](docs/remote-access.md)
- [Troubleshooting](docs/troubleshooting.md)

## What Makes NeoNexus Different?

| Feature | NeoNexus | Manual CLI Setup | Cloud Providers |
|---------|----------|------------------|-----------------|
| **Setup Time** | 5 minutes | 2+ hours | 10 minutes |
| **Web UI** | ✅ Built-in | ❌ CLI only | ✅ Yes |
| **Self-Hosted** | ✅ Yes | ✅ Yes | ❌ No |
| **Multi-Node** | ✅ Easy | ⚠️ Complex | 💰 Expensive |
| **Cost** | Free | Free (labor) | $$ Monthly |
| **Plugin Support** | ✅ UI-managed | ⚠️ Manual | ❌ No |

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

| Software | Version | Networks |
|----------|---------|----------|
| neo-cli | v3.6.0 - v3.9.2 | Mainnet, Testnet |
| neo-go | v0.104.0+ | Mainnet, Testnet |

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

See [Remote Access Guide](docs/remote-access.md) for detailed setup.

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

## Roadmap

- [x] neo-cli support
- [x] neo-go support
- [x] Plugin management
- [x] Public dashboard
- [x] Real-time metrics
- [ ] Alert notifications
- [ ] Backup/restore
- [ ] Multi-server management
- [ ] REST API v2

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License — see [LICENSE](LICENSE) for details.

## Support

- 📖 [Documentation](https://docs.neo-nexus.io)
- 💬 [Discord Community](https://discord.gg/neo-nexus)
- 🐛 [Issue Tracker](https://github.com/yourusername/neo-nexus/issues)

---

<p align="center">
  Built with ❤️ for the Neo community
</p>
