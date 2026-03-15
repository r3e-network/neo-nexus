# NeoNexus

**The Ultimate Industrial-grade Neo N3 & Neo X Infrastructure Platform.**

NeoNexus is a comprehensive Node-as-a-Service (NaaS) and Web3 cloud infrastructure provider exclusively built for the Neo ecosystem. It combines the seamless developer experience of platforms like Vercel with the robust, enterprise-grade scalability of Chainstack and QuickNode.

---

## 🌟 Project Architecture (Monorepo)

The repository is structured into three main layers:

1. **`/website` (Marketing & Documentation)**
   - Built with Next.js 15 (App Router), Tailwind CSS, and Framer Motion.
   - Fully static generation for maximum SEO performance.
   - Includes Pricing, Developer Hub, Documentation, and Auth flows.

2. **`/dashboard` (Control Console)**
   - The core NaaS interface built with Next.js 15, Tailwind, and Recharts.
   - Connected to **Supabase** via Server Components and Actions.
   - Includes: Endpoint creation wizard, real-time Analytics, Firewall security settings, and Marketplace add-ons.

3. **`/infrastructure` (DevOps & Control Plane)**
   - **Helm Charts (`/helm`)**: Production-ready Kubernetes manifests to deploy `neo-go` and `neo-cli` stateful nodes with persistent volumes.
   - **Database (`/database`)**: Complete PostgreSQL schema with Row-Level Security (RLS) for Supabase integration.
   - **Docker (`/docker`)**: Local observability stack (neo-go + Prometheus + Grafana) for testing metrics.

---

## 🚀 Getting Started

### 1. Launch the Marketing Website
```bash
cd website
npm install
npm run dev
# Visit: http://localhost:3000
```

### 2. Launch the Dashboard
```bash
cd dashboard
npm install
npm run dev
# Visit: http://localhost:3001
```

*(Note: The Dashboard uses a graceful fallback. If you don't provide Supabase environment variables, it will render using high-quality mock data automatically.)*

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

## 🤝 Open Source
NeoNexus is designed to accelerate the growth of the Neo N3 and Neo X blockchains by removing infrastructure hurdles.
