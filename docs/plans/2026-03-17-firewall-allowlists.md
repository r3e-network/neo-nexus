# Firewall Allowlists Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make org-level IP and origin allowlists real, persisted, and enforced through APISIX route plugins.

**Architecture:** Add organization-scoped firewall records, validate and normalize allowlist entries in a pure service, derive APISIX route plugins from those rules, and resync all organization routes whenever allowlists change. Keep JSON-RPC method presets as non-live preview only, because APISIX native method restrictions do not map to JSON-RPC methods.

**Tech Stack:** Next.js server actions, Prisma, TypeScript, Vitest, APISIX Admin API

### Task 1: Firewall Rule Validation And Plugin Derivation

**Files:**
- Create: `dashboard/src/services/security/FirewallPolicy.ts`
- Create: `dashboard/src/services/security/FirewallPolicy.test.ts`
- Modify: `dashboard/src/services/apisix/ApisixService.ts`
- Modify: `dashboard/src/services/apisix/ApisixService.test.ts`

**Step 1: Write the failing test**

```ts
it('builds APISIX route plugins from ip and origin allowlists', () => {
  const plugins = buildRouteSecurityPlugins([
    { type: 'ip_allow', value: '203.0.113.10' },
    { type: 'origin_allow', value: 'https://app.example.com' },
  ]);

  expect(plugins['ip-restriction']).toBeDefined();
  expect(plugins.cors).toBeDefined();
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/security/FirewallPolicy.test.ts src/services/apisix/ApisixService.test.ts`
Expected: FAIL because the policy helper does not exist yet.

**Step 3: Write minimal implementation**

Implement:
- IP / CIDR validation
- origin URL normalization
- APISIX plugin derivation for `ip-restriction` and `cors`

Update route payload creation to merge security plugins with the base auth/rate-limit plugins.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/security/FirewallPolicy.test.ts src/services/apisix/ApisixService.test.ts`
Expected: PASS

### Task 2: Persist Organization-Scoped Firewall Rules

**Files:**
- Modify: `dashboard/prisma/schema.prisma`
- Modify: `infrastructure/database/schema.sql`
- Modify: `dashboard/src/app/app/security/actions.ts`
- Modify: `dashboard/src/app/app/security/page.tsx`
- Modify: `dashboard/src/app/app/security/SecurityClient.tsx`

**Step 1: Wire the data model**

Add `organizationId` to firewall rows and organization relation wiring.

**Step 2: Implement create/delete actions**

Create actions that:
- validate and normalize `ip_allow` / `origin_allow` values
- persist organization-scoped rules
- resync existing organization routes in APISIX

**Step 3: Implement the UI**

Replace the placeholder allowlist panel with:
- add IP/origin rule form
- actual existing org-level rules list
- remove action per rule

**Step 4: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/security/actions.ts src/app/app/security/page.tsx src/app/app/security/SecurityClient.tsx`
Expected: PASS

### Task 3: Apply Allowlists To New Routes

**Files:**
- Modify: `dashboard/src/services/provisioning/ProvisioningRunner.ts`

**Step 1: Update route creation**

When shared or dedicated routes are created, load the organization firewall rules and pass derived security plugins into APISIX route creation.

**Step 2: Verify**

Use the focused tests from Task 1 plus full verify.

### Task 4: Full Verification

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/security/FirewallPolicy.test.ts src/services/apisix/ApisixService.test.ts src/app/app/security/SecurityClient.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS
