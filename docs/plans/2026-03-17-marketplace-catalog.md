# Marketplace Catalog Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the mocked marketplace with an honest plugin catalog backed by the real supported plugin list and actual organization endpoint/plugin state.

**Architecture:** Build a pure marketplace view-model helper that merges `PluginCatalog` definitions with dedicated endpoint state to derive real card status and CTA targets. Use the server page to fetch organization dedicated endpoints and installed plugins, then render the client page from that derived data instead of a hardcoded fake marketplace dataset.

**Tech Stack:** Next.js App Router, Prisma, TypeScript, Vitest

### Task 1: Build A Real Marketplace View Model

**Files:**
- Create: `dashboard/src/services/plugins/MarketplaceCatalog.ts`
- Create: `dashboard/src/services/plugins/MarketplaceCatalog.test.ts`
- Modify: `dashboard/src/services/plugins/PluginCatalog.ts`

**Step 1: Write the failing test**

```ts
it('routes plugin cards to real dedicated endpoint actions', () => {
  const cards = buildMarketplacePluginCards({
    plugins: listSupportedPlugins(),
    dedicatedEndpoints: [{ id: 7, name: 'alpha', installedPluginIds: ['tee-oracle'] }],
  });

  expect(cards[0].actionHref).toBe('/app/endpoints/7');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/plugins/MarketplaceCatalog.test.ts`
Expected: FAIL because the marketplace helper does not exist yet.

**Step 3: Write minimal implementation**

Implement a pure helper that derives:
- category/filter values from the real plugin definitions
- installed counts from dedicated endpoints
- honest CTA labels/hrefs based on whether dedicated endpoints exist

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/plugins/MarketplaceCatalog.test.ts`
Expected: PASS

### Task 2: Replace The Mock Marketplace UI

**Files:**
- Modify: `dashboard/src/app/app/marketplace/page.tsx`
- Modify: `dashboard/src/app/app/marketplace/MarketplaceClient.tsx`

**Step 1: Write the failing test**

Use the Task 1 helper tests as the regression proof for marketplace card behavior.

**Step 2: Implement minimal UI replacement**

The UI should:
- stop using the hardcoded fake marketplace array
- render cards from the server-provided real catalog model
- remove fake pricing and fake installed/coming-soon states
- replace toast-only install with honest links like `Create Dedicated Node`, `Open Node`, or `Choose Node`

**Step 3: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/marketplace/MarketplaceClient.tsx src/app/app/marketplace/page.tsx`
Expected: PASS

### Task 3: Full Verification

**Files:**
- Modify docs only if route/page copy needs explanation

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/plugins/MarketplaceCatalog.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS

**Step 3: Review the diff**

Run: `git diff -- dashboard/src/app/app/marketplace dashboard/src/services/plugins/MarketplaceCatalog.ts dashboard/src/services/plugins/MarketplaceCatalog.test.ts docs/plans/2026-03-17-marketplace-catalog.md`
Expected: Only real marketplace catalog changes.
