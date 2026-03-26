# NeoNexus Branding & Messaging Guide

## Official Identity

### Primary Name
**NeoNexus Node Manager**

### Tagline Options

| Context | Tagline |
|---------|---------|
| **Primary** | "Self-hosted Neo N3 node management, simplified" |
| **Technical** | "Deploy, monitor, and manage Neo N3 nodes from one dashboard" |
| **Short** | "Your nodes, your control" |
| **Enterprise** | "Production-grade Neo N3 node orchestration" |

---

## Positioning

### What NeoNexus Is
A **self-hosted node management platform** that automates the deployment, configuration, and monitoring of Neo N3 blockchain nodes (neo-cli and neo-go).

### The Problem It Solves
Running a Neo N3 node requires:
- Manual binary downloads and configuration
- Complex command-line operations
- Constant monitoring and maintenance
- Security and access control management

**NeoNexus eliminates this complexity** with a web-based dashboard.

---

## Messaging by Audience

### For Node Operators (Primary Users)

**Pain Points:**
- "I want to run a Neo node but don't want to spend hours on CLI setup"
- "Managing multiple nodes is tedious and error-prone"
- "I need to monitor node health without constant SSH checks"

**Key Messages:**
- Deploy Neo nodes in minutes, not hours
- Manage all your nodes from a single dashboard
- Real-time metrics and health monitoring
- Automatic port allocation and conflict resolution

### For Developers

**Pain Points:**
- "I need a local Neo node for development testing"
- "Switching between mainnet and testnet is annoying"
- "I want to test with both neo-cli and neo-go"

**Key Messages:**
- One-click deployment of development nodes
- Easy network switching (mainnet/testnet/private)
- Support for both official node implementations
- Plugin management for neo-cli

### For Enterprise/Organizations

**Pain Points:**
- "We need reliable Neo infrastructure without cloud dependencies"
- "Compliance requires self-hosted solutions"
- "We need audit logs and access control"

**Key Messages:**
- Fully self-hosted — no cloud dependencies
- JWT-based authentication with role-based access
- Comprehensive audit trail
- Runs on your own hardware or VMs

---

## Feature Descriptions

### Core Features

| Feature | User-Facing Description | Technical Description |
|---------|------------------------|----------------------|
| **One-Click Deploy** | "Deploy Neo nodes in minutes without touching the command line" | Automated download, configuration, and setup of neo-cli and neo-go binaries |
| **Multi-Node Management** | "Run multiple nodes simultaneously with automatic port allocation" | Isolated node directories with dynamic port assignment starting at 10332 |
| **Real-Time Monitoring** | "Track block height, peers, CPU, and memory from your browser" | WebSocket-based metrics streaming with systeminformation integration |
| **Public Dashboard** | "Share node status without exposing sensitive configuration" | Read-only public API with sanitized node metrics |
| **Plugin Manager** | "Install and configure neo-cli plugins through the UI" | Download and configure official Neo plugins (ApplicationLogs, LevelDBStore, etc.) |

### Security Features

| Feature | Description |
|---------|-------------|
| **Self-Hosted** | All data stays on your infrastructure |
| **JWT Authentication** | Secure API access with token-based auth |
| **Default Credentials Warning** | Prompts password change on first login |
| **Public View Mode** | Separate read-only access for monitoring without management rights |
| **Isolated Node Storage** | Each node has isolated data, config, and logs |

---

## Comparison Table

| Feature | NeoNexus | Manual Setup | Cloud Providers |
|---------|----------|--------------|-----------------|
| **Setup Time** | 5 minutes | 2+ hours | 10 minutes |
| **Web UI** | ✅ Built-in | ❌ CLI only | ✅ Yes |
| **Self-Hosted** | ✅ Yes | ✅ Yes | ❌ No |
| **Multi-Node** | ✅ Easy | ⚠️ Complex | 💰 Expensive |
| **Cost** | Free | Free (labor) | $$ Monthly |
| **Data Control** | Full | Full | Limited |
| **Plugin Support** | ✅ UI-managed | ⚠️ Manual | ❌ No |

---

## Terminology Glossary

Use these terms consistently across documentation:

| Term | Usage |
|------|-------|
| **Node** | A running instance of neo-cli or neo-go |
| **Deployment** | The process of downloading and configuring a node |
| **Network** | The blockchain network (mainnet, testnet, private) |
| **Sync Mode** | How the node synchronizes (full, light) |
| **Dashboard** | The web interface for managing nodes |
| **Public View** | Read-only access without authentication |
| **Metrics** | Performance data (CPU, memory, block height, peers) |

**Avoid:**
- "Installer" (implies one-time use; NeoNexus is for ongoing management)
- "Wallet" (NeoNexus manages nodes, not wallets)
- "Miner" (Neo N3 uses dBFT, not mining)

---

## Website Copy

### Hero Section

```
NeoNexus Node Manager
Self-hosted Neo N3 node management, simplified

Deploy neo-cli and neo-go nodes in minutes.
Monitor health, manage plugins, and control everything from one dashboard.

[Get Started] [View Documentation]
```

### Feature Grid

```
🚀 Quick Deploy
   One-click installation of Neo N3 nodes. No CLI required.

📊 Real-Time Metrics
   Monitor block height, peers, CPU, and memory usage live.

🔌 Plugin Management
   Install and configure neo-cli plugins through the UI.

🔐 Self-Hosted
   Your nodes, your hardware, your control. No cloud lock-in.

👁️ Public Dashboard
   Share node status with your community without exposing config.

🌐 Multi-Network
   Switch between mainnet, testnet, and private networks easily.
```

### Call to Action

```
Ready to run your own Neo N3 node?

Get started in 5 minutes:
1. Install NeoNexus
2. Open the web dashboard
3. Deploy your first node

[Installation Guide] [GitHub]
```

---

## README Copy

### Quick Start Section

```markdown
## Quick Start

### 1. Install
\`\`\`bash
npm install
npm run build
npm start
\`\`\`

### 2. Access Dashboard
Open http://localhost:8080 in your browser

### 3. Login
- Username: `admin`
- Password: `admin`
- ⚠️ Change the default password in Settings

### 4. Deploy Your First Node
Click "Create Node" → Select type (neo-cli/neo-go) → Choose network → Deploy
```

---

## Social Media / Announcement Copy

### Twitter/X Post

```
Introducing NeoNexus Node Manager 🚀

A self-hosted dashboard for deploying and managing Neo N3 nodes.

✅ One-click neo-cli/neo-go deployment
✅ Real-time monitoring
✅ Plugin management
✅ Public dashboard mode
✅ Completely self-hosted

Run your own Neo infrastructure without the headache.

→ github.com/r3e-network/neonexus

#NeoN3 #Blockchain #NodeManager
```

### LinkedIn Post

```
We're excited to introduce NeoNexus Node Manager — a new open-source tool for deploying and managing Neo N3 blockchain nodes.

NeoNexus eliminates the complexity of running Neo infrastructure:
• Deploy nodes in minutes, not hours
• Monitor health and performance from a web dashboard
• Manage neo-cli plugins without touching config files
• Share public node status with your community

Perfect for developers, node operators, and organizations building on Neo N3.

Fully self-hosted. No cloud dependencies. Your nodes, your control.

Check it out: [link]

#NeoN3 #BlockchainInfrastructure #OpenSource #Web3
```

---

## Documentation Tone & Voice

### Tone Guidelines

| ✅ Use | ❌ Avoid |
|--------|----------|
| Clear, direct instructions | Vague or marketing-heavy language |
| Technical accuracy | Oversimplification that loses meaning |
| Action-oriented verbs | Passive voice |
| "You can..." | "One might..." |
| Specific examples | Generic descriptions |

### Example: Good vs. Bad

**Bad:**
> "NeoNexus is a revolutionary platform that leverages cutting-edge technology to facilitate the deployment of blockchain nodes in a seamless manner."

**Good:**
> "NeoNexus deploys Neo N3 nodes (neo-cli or neo-go) and provides a web dashboard to manage them. Download binaries, configure ports, and monitor health — all without command-line tools."

---

## Logo & Visual Identity Suggestions

### Color Palette

| Use | Color | Hex |
|-----|-------|-----|
| Primary (Neo Green) | Neo brand green | `#00E599` |
| Secondary (Dark) | Slate/Dark mode | `#0F172A` |
| Accent | Bright cyan | `#00D4AA` |
| Warning | Amber | `#F59E0B` |
| Error | Red | `#EF4444` |

### Icon Concepts

1. **Network Node Icon** — Connected dots representing blockchain nodes
2. **Dashboard + Node** — Combination of chart/dashboard with a node symbol
3. **Neo "N" + Gear** — Neo's N with management/settings symbolism

---

## Version Naming

Use semantic versioning with codenames:

| Version | Codename | Focus |
|---------|----------|-------|
| 2.0.0 | "Foundation" | Initial release (current) |
| 2.1.0 | "Monitor" | Enhanced metrics & alerting |
| 2.2.0 | "Scale" | Multi-server node management |
| 3.0.0 | "Horizon" | Mainnet production features |

---

*This branding guide ensures consistent messaging across all NeoNexus materials.*
