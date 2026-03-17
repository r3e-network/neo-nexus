# Method Firewall Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Turn the current JSON-RPC method firewall preview into real route enforcement for selected Neo RPC method categories.

**Architecture:** Keep IP/origin allowlists on native APISIX plugins, but implement JSON-RPC method filtering through an APISIX `serverless-pre-function` access-phase Lua snippet generated from persisted organization policy. This is necessary because APISIX’s native `request-validation` plugin validates body schema, not JSON-RPC method semantics. The generated serverless plugin should parse request bodies, support single and batch JSON-RPC payloads, and return `403` for blocked methods.

**Tech Stack:** Next.js server actions, Prisma, TypeScript, Vitest, APISIX route plugins

### Task 1: Method Firewall Policy Generation

**Files:**
- Modify: `dashboard/src/services/security/FirewallPolicy.ts`
- Modify: `dashboard/src/services/security/FirewallPolicy.test.ts`
- Modify: `dashboard/src/services/apisix/ApisixService.test.ts`

**Step 1: Write the failing test**

```ts
it('builds a serverless-pre-function plugin for blocked json-rpc methods', () => {
  const plugins = buildRouteSecurityPlugins([
    { type: 'method_block', value: 'getstorage' },
  ]);

  expect(plugins['serverless-pre-function']).toBeDefined();
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/security/FirewallPolicy.test.ts src/services/apisix/ApisixService.test.ts`
Expected: FAIL because method-block rules are not turned into APISIX plugins yet.

**Step 3: Write minimal implementation**

Generate a `serverless-pre-function` plugin that:
- reads request body
- handles single JSON-RPC payloads and arrays
- blocks configured methods with `403`

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/security/FirewallPolicy.test.ts src/services/apisix/ApisixService.test.ts`
Expected: PASS

### Task 2: Persist Organization Method Policies

**Files:**
- Modify: `dashboard/src/app/app/security/SecurityClient.tsx`
- Modify: `dashboard/src/app/app/security/actions.ts`
- Modify: `dashboard/src/app/app/security/page.tsx`

**Step 1: Implement real method firewall controls**

Replace the current preview-only method presets with persisted org-level `method_block` rules derived from preset categories such as:
- write methods
- debug methods
- execution test methods

**Step 2: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/security/SecurityClient.tsx src/app/app/security/actions.ts src/app/app/security/page.tsx`
Expected: PASS

### Task 3: Route Sync And Verification

**Files:**
- Modify: `dashboard/src/services/security/RouteSecuritySync.ts`
- Modify: `dashboard/src/services/provisioning/ProvisioningRunner.ts`

**Step 1: Ensure route sync includes method plugins**

Existing route sync and new-route provisioning should automatically carry the generated method-firewall plugin.

**Step 2: Run full verification**

Run: `npm run verify`
Expected: PASS
