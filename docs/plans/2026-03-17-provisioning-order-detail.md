# Provisioning Order Detail Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add an operator-facing provisioning order detail page with current order state, endpoint state, and recent provisioning activity.

**Architecture:** Create a small helper that filters endpoint activities down to provisioning-related rows and sorts them for display. Use that helper in a server-rendered operator-only page under `/app/operations/provisioning/[id]`, and link the provisioning queue into it.

**Tech Stack:** Next.js App Router, Prisma, TypeScript, Vitest

### Task 1: Provisioning Activity Helper

**Files:**
- Create: `dashboard/src/services/provisioning/ProvisioningActivity.ts`
- Create: `dashboard/src/services/provisioning/ProvisioningActivity.test.ts`

**Step 1: Write the failing test**

Run: `npm run test --workspace=dashboard -- src/services/provisioning/ProvisioningActivity.test.ts`
Expected: FAIL because the helper does not exist.

**Step 2: Implement minimal helper**

Filter endpoint activities to category `provisioning` and sort descending by `createdAt`.

### Task 2: Order Detail Page

**Files:**
- Create: `dashboard/src/app/app/operations/provisioning/[id]/page.tsx`
- Modify: `dashboard/src/app/app/operations/provisioning/page.tsx`

**Step 1: Implement the page**

Load:
- provisioning order
- endpoint state
- recent provisioning endpoint activities

**Step 2: Link the queue**

Add a “View Details” link from each provisioning queue row.

### Task 3: Full Verification

Run: `npm run verify`
Expected: PASS
