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
   - Includes: Endpoint creation wizard, real-time Analytics, Firewall security settings, and a managed plugin catalog for dedicated nodes.

2. **`/infrastructure` (DevOps & Control Plane)**
   - **Helm Charts (`/helm`)**: Optional shared-service and observability assets for cluster-based experiments or APISIX/monitoring deployments.
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

## 🛠️ Core Capabilities

* **Zero-Touch Provisioning**: Automated architecture templates (RPC, Consensus, Oracle) that intelligently spin up Hetzner/DigitalOcean VMs, configure firewalls, and install Docker engines without manual SSH.
* **Dual-Engine Support**: Choose between the lightning-fast `neo-go` or the official reference `neo-cli` (C#). Also natively supports `neo-x-geth` for the Neo X EVM sidechain.
* **Managed Plugin Runtime**: Natively supports all 12 official Neo-CLI plugins (including DBFT, OracleService, ApplicationLogs). Users can dynamically reconfigure plugins via web forms.
* **Enterprise Secret Custody**: AWS KMS-backed encryption ensures that Consensus and Oracle private keys are securely injected directly into isolated sidecar containers.
* **Vanity Domains with Auto-TLS**: Allows users to configure custom endpoint URLs (`rpc.mycompany.com`) backed by automated Let's Encrypt certificates managed by the APISIX edge gateway.
* **Native Crypto Billing**: Frictionless NeoLine web3 wallet integration for paying subscription tiers automatically in GAS.

## 🤝 Open Source
NeoNexus is designed to accelerate the growth of the Neo N3 and Neo X blockchains by removing infrastructure hurdles.
