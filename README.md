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

*(Note: The Dashboard uses a graceful fallback. If you don't provide Prisma environment variables, it will prompt you or safely disable features.)*

### 2. Deploy a Local Observability Stack
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

## 🤝 Open Source
NeoNexus is designed to accelerate the growth of the Neo N3 and Neo X blockchains by removing infrastructure hurdles.
