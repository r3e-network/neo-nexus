# API Key Consumer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make APISIX authentication track one consumer per API key so multiple keys work correctly and deletes clean up the matching gateway consumer.

**Architecture:** Replace the current org-level APISIX consumer mapping with per-key consumer identities derived from the `ApiKey.id`. Add pure helpers for consumer id/payload construction, update security actions to create/delete the matching APISIX consumer for each key, and fail the action when gateway sync does not succeed so the database and gateway do not silently drift.

**Tech Stack:** Next.js server actions, Prisma, TypeScript, Vitest

### Task 1: APISIX Consumer Helpers

**Files:**
- Modify: `dashboard/src/services/apisix/ApisixService.ts`
- Modify: `dashboard/src/services/apisix/ApisixService.test.ts`

**Step 1: Write the failing test**

```ts
it('builds a unique consumer id per api key', () => {
  expect(buildConsumerId('key-123')).toBe('api-key-key-123');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/apisix/ApisixService.test.ts`
Expected: FAIL because the helper does not exist yet.

**Step 3: Write minimal implementation**

Add:
- `buildConsumerId(apiKeyId)`
- `buildConsumerPayload(apiKey, plan)`
- `deleteConsumer(apiKeyId)`

Update `createConsumer` to take `apiKeyId` instead of `organizationId`.

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/apisix/ApisixService.test.ts`
Expected: PASS

### Task 2: Security Actions

**Files:**
- Modify: `dashboard/src/app/app/security/actions.ts`

**Step 1: Update the action flow**

Change API key create/delete so that:
- create stores the key row, syncs the matching APISIX consumer, and rolls back the key row if sync fails
- delete resolves the owned key first, removes the APISIX consumer, then deletes the key row

**Step 2: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/security/actions.ts`
Expected: PASS

### Task 3: Full Verification

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/apisix/ApisixService.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS
