# Platform Messaging Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Remove cluster-era and over-promising messaging from docs and marketing so the product story matches the current VM-first Hetzner-primary / DigitalOcean-backup platform.

**Architecture:** Update documentation and marketing copy only. Keep the message consistent across infrastructure docs, repo README, landing page, pricing, and developer surfaces: dedicated nodes are managed VMs, shared endpoints are APISIX-routed upstreams, plugin installation happens from dedicated endpoints, and optional Kubernetes assets are only for shared services / experiments.

**Tech Stack:** Markdown, Next.js App Router, TypeScript, ESLint, Next build

### Task 1: Infrastructure Docs

**Files:**
- Modify: `docs/DEPLOY_INFRASTRUCTURE.md`
- Modify: `docs/ARCHITECTURE.md`
- Modify: `README.md`

**Step 1: Update the docs**

Replace Kubernetes-primary wording with control-plane / optional shared-services wording, and remove claims that imply dedicated nodes are cluster-managed.

**Step 2: Verify**

Run: `npm run lint --workspace=dashboard`
Expected: PASS

### Task 2: Marketing Surfaces

**Files:**
- Modify: `dashboard/src/app/(marketing)/page.tsx`
- Modify: `dashboard/src/app/(marketing)/pricing/page.tsx`
- Modify: `dashboard/src/app/(marketing)/developers/page.tsx`

**Step 1: Update the copy**

Replace:
- AWS/GCP wording with Hetzner primary / DigitalOcean fallback wording
- one-click / zero-devops overstatements with honest managed-platform language
- cluster/K8s deployment claims with managed VM language

**Step 2: Verify**

Run: `npm run lint --workspace=dashboard -- 'src/app/(marketing)/page.tsx' 'src/app/(marketing)/pricing/page.tsx' 'src/app/(marketing)/developers/page.tsx'`
Expected: PASS

### Task 3: Full Verification

**Step 1: Run the full verification suite**

Run: `npm run verify`
Expected: PASS

**Step 2: Review the diff**

Run: `git diff -- README.md docs/DEPLOY_INFRASTRUCTURE.md docs/ARCHITECTURE.md 'dashboard/src/app/(marketing)/page.tsx' 'dashboard/src/app/(marketing)/pricing/page.tsx' 'dashboard/src/app/(marketing)/developers/page.tsx' docs/plans/2026-03-17-platform-messaging.md`
Expected: Only platform messaging alignment changes.
