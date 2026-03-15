# NeoNexus Deployment Architecture

This document describes the recommended production architecture for deploying NeoNexus. 

To achieve maximum scalability, zero-downtime deployments, and strong security boundaries, NeoNexus is designed to be deployed across two separate environments: **The Frontend Edge (Vercel)** and **The Backend Control Plane (AWS/GCP Kubernetes)**.

## High-Level Architecture Diagram

```mermaid
graph TD
    %% Users
    User((Developer/User))
    DApp((DApp/Client))

    %% Edge Layer
    subgraph "Edge Network (Vercel)"
        Website[Marketing Website]
        Dashboard[NeoNexus Dashboard]
    end

    %% Auth & DB Layer
    subgraph "Managed Services"
        NeonDB[(Neon Postgres)]
        Auth[NextAuth / OAuth Providers]
        Stripe[Stripe Billing]
    end

    %% Infrastructure Layer
    subgraph "Infrastructure Layer (Kubernetes Cluster)"
        APISIX[Apache APISIX Ingress Gateway]
        Prometheus[Prometheus & Grafana]
        
        subgraph "Tenant Namespaces"
            Node1[neo-go (Node 1)]
            Node2[neo-cli (Node 2)]
            Node3[Shared Node Pool]
        end
    end

    %% Connections
    User -->|Visits| Website
    User -->|Manages| Dashboard
    DApp -->|RPC Requests| APISIX
    
    Website <--> Auth
    Dashboard <--> Auth
    Dashboard <--> NeonDB
    Dashboard <--> Stripe
    
    Dashboard -->|Kubernetes API (Deploy/Scale)| APISIX
    Dashboard -->|Kubernetes API (Deploy/Scale)| Node1
    
    APISIX -->|Routes traffic| Node1
    APISIX -->|Routes traffic| Node2
    APISIX -->|Routes traffic| Node3

    Prometheus -->|Scrapes Metrics| Node1
    Prometheus -->|Scrapes Metrics| Node2
```

## Why this Architecture?

1. **Separation of Concerns:** The Next.js frontend (Dashboard) handles the UI, authentication, database persistence, and billing. It runs on a globally distributed edge network (Vercel), ensuring lightning-fast load times.
2. **Heavy Lifting on K8s:** Blockchain nodes (neo-go/neo-cli) are highly stateful, requiring massive IOPS and memory. Kubernetes (EKS/GKE) is the only reliable way to orchestrate persistent volumes (PVCs) dynamically.
3. **API Gateway (APISIX):** Directly exposing K8s Services is dangerous. APISIX acts as a high-performance shield, validating API keys, enforcing rate limits, and routing traffic to the correct tenant's pod.

## Setup Requirements

Before you begin the deployment process, ensure you have accounts and access to the following:

- **Vercel Account:** For hosting the `website` and `dashboard` Next.js applications.
- **Neon Account:** For the Serverless PostgreSQL database.
- **GitHub/Google Developer Console:** For OAuth credentials.
- **Stripe Account:** For handling subscriptions and payments.
- **Cloud Provider (AWS/GCP):** For creating the Kubernetes cluster (EKS/GKE).
- **Domain Name:** To configure DNS records (e.g., `neonexus.com`).
