# Endpoint Addressing Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Centralize endpoint public address generation and stop presenting planned addresses as live routes while an endpoint is still provisioning or stopped.

**Architecture:** Introduce pure helpers for planned endpoint address generation and route display state. Use the address generator from endpoint creation so domain patterns are not hardcoded inline, then use the display helper in list/details/overview surfaces so users see clear “planned” or “not ready” messaging instead of raw live-looking URLs during non-active states.

**Tech Stack:** Next.js, TypeScript, Prisma, Vitest

### Task 1: Planned Address Generation

**Files:**
- Create: `dashboard/src/services/endpoints/EndpointAddressing.ts`
- Create: `dashboard/src/services/endpoints/EndpointAddressing.test.ts`
- Modify: `dashboard/src/app/app/endpoints/actions.ts`
- Modify: `dashboard/.env.example`

**Step 1: Write the failing test**

```ts
it('builds dedicated and shared planned endpoint urls from a root domain', () => {
  expect(buildPlannedEndpointAddress(...).httpsUrl).toBe('https://node-fsn1-abc123.neonexus.cloud/v1');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/endpoints/EndpointAddressing.test.ts`
Expected: FAIL because the helper does not exist.

**Step 3: Write minimal implementation**

Implement a pure helper that:
- uses a configurable root domain with a safe default
- generates dedicated/shared planned addresses consistently
- optionally derives WSS when the protocol/runtime supports it

Then switch `createEndpointAction` to use it and stop explicitly writing cluster-era `k8s*` fields.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/endpoints/EndpointAddressing.test.ts`
Expected: PASS

### Task 2: Honest Route Display State

**Files:**
- Modify: `dashboard/src/services/endpoints/EndpointConnectionDisplay.ts`
- Modify: `dashboard/src/services/endpoints/EndpointConnectionDisplay.test.ts`
- Modify: `dashboard/src/app/app/endpoints/[id]/EndpointDetailsClient.tsx`
- Modify: `dashboard/src/app/app/endpoints/EndpointsList.tsx`
- Modify: `dashboard/src/app/app/OverviewClient.tsx`

**Step 1: Write the failing test**

```ts
it('marks non-active endpoints as planned rather than live', () => {
  const display = deriveEndpointConnectionDisplay({
    status: 'Provisioning',
    url: 'https://node-fsn1-abc.neonexus.cloud/v1',
  });

  expect(display.readiness).toBe('planned');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/endpoints/EndpointConnectionDisplay.test.ts`
Expected: FAIL because the helper only returns raw URLs today.

**Step 3: Write minimal implementation**

Return structured display state:
- `live` for active endpoints
- `planned` for provisioning/syncing endpoints
- `offline` for stopped/error endpoints

Update the endpoint list, overview, and detail pages to render honest labels/copy behavior from that state.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/endpoints/EndpointConnectionDisplay.test.ts`
Expected: PASS

### Task 3: Full Verification

**Files:**
- Modify docs only if the new root-domain env var needs explanation

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/endpoints/EndpointAddressing.test.ts src/services/endpoints/EndpointConnectionDisplay.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS

**Step 3: Review the diff**

Run: `git diff -- dashboard/src/services/endpoints dashboard/src/app/app/endpoints/actions.ts dashboard/src/app/app/endpoints/[id]/EndpointDetailsClient.tsx dashboard/src/app/app/endpoints/EndpointsList.tsx dashboard/src/app/app/OverviewClient.tsx dashboard/.env.example docs/plans/2026-03-17-endpoint-addressing.md`
Expected: Only endpoint addressing and honest display-state changes.
