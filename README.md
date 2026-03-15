# NeoNexus

**The Ultimate Industrial-grade Neo N3 & Neo X Infrastructure Platform.**

NeoNexus is a comprehensive Node-as-a-Service (NaaS) and Web3 cloud infrastructure provider exclusively built for the Neo ecosystem. It combines the seamless developer experience of platforms like Vercel with the robust, enterprise-grade scalability of Chainstack and QuickNode.

---

## 🌟 Project Architecture

The repository is structured into two main layers:

1. **`/dashboard` (Unified Frontend & Control Console)**
   - Built with Next.js 16 (App Router), Tailwind CSS, and Framer Motion.
   - Contains both the public Marketing Website (`/(marketing)`) and the authenticated Control Console (`/app`).
   - Connected to **Neon Serverless Postgres** via Prisma ORM and NextAuth.
   - Includes: Endpoint creation wizard, real-time Analytics (SWR), Firewall security settings, and Marketplace add-ons.

2. **`/infrastructure` (DevOps & Control Plane)**
   - **Helm Charts (`/helm`)**: Production-ready Kubernetes manifests to deploy `neo-go`, `neo-cli`, and `neo-x-geth` stateful nodes with persistent volumes.
   - **Database (`/database`)**: Complete PostgreSQL schema.
   - **Docker (`/docker`)**: Local observability stack (neo-go + Prometheus + Grafana) for testing metrics.

---

## 🚀 Getting Started

### 1. Launch the Platform
```bash
npm install
npm run dev
# Visit: http://localhost:3000
```

If you do not provide database or auth credentials, the marketing site will still load, but authenticated dashboard features will be limited.

### 2. Verify the Dashboard
```bash
npm run verify
```

This runs the dashboard workspace lint, typecheck, and production build in one command.

### 3. Deploy a Local Observability Stack
Want to see the metrics engine in action? You can spin up a local Neo N3 node alongside Prometheus and Grafana:
```bash
cd infrastructure/docker
docker-compose up -d
```

---

## 🛠️ Infrastructure Capabilities

* **Dual-Engine Support**: Choose between the lightning-fast `neo-go` or the official reference `neo-cli` (C#).
* **Multi-Cloud Readiness**: Helm values configured to support mapping onto AWS (EKS) and Google Cloud (GKE) storage classes.
* **Sync Modes**: Provisions both lightweight Full nodes (RPC) and deep Archive nodes for indexers.
* **Marketplace Integrations**: Architected to support sidecar containers for Phala TEE Oracles and Account Abstraction Bundlers.

## ⚠️ Current Gaps

- The analytics dashboard is now backed by persisted organization data, but it is still current-state reporting rather than a full Prometheus/VictoriaMetrics time-series pipeline.
- Crypto billing now verifies a real N3 transaction hash against the configured treasury and amount, but wallet initiation itself is still manual in the UI.
- Secret storage now uses authenticated encryption via `VAULT_ENCRYPTION_KEY`; if you need HSM/KMS-backed custody, you still need to integrate an external secret manager.

## 🤝 Open Source
NeoNexus is designed to accelerate the growth of the Neo N3 and Neo X blockchains by removing infrastructure hurdles.
