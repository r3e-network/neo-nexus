# Billing Overview Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace placeholder invoice and payment-method sections on the billing page with real recent billing activity from Stripe and on-chain billing records.

**Architecture:** Add a server-side billing overview service that merges recent `BillingTransaction` rows with optional Stripe customer data (recent invoices and card summary) into a serializable view model. Keep checkout, billing-portal, and crypto verification flows intact, but render the billing page from real data instead of static placeholders.

**Tech Stack:** Next.js App Router, Prisma, Stripe SDK, TypeScript, Vitest

### Task 1: Billing Overview View Model

**Files:**
- Create: `dashboard/src/services/billing/BillingOverviewService.ts`
- Create: `dashboard/src/services/billing/BillingOverviewService.test.ts`
- Modify: `dashboard/src/services/billing/StripeService.ts`

**Step 1: Write the failing test**

```ts
it('combines stripe invoices and crypto transactions into a unified billing history', () => {
  const overview = buildBillingOverview(...);
  expect(overview.items).toHaveLength(2);
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/billing/BillingOverviewService.test.ts`
Expected: FAIL because the service does not exist.

**Step 3: Write minimal implementation**

Create:
- a pure transformer for Stripe invoices + crypto billing rows
- Stripe helper methods to fetch recent invoices and a card summary when a customer exists

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/billing/BillingOverviewService.test.ts`
Expected: PASS

### Task 2: Billing Page

**Files:**
- Modify: `dashboard/src/app/app/billing/page.tsx`
- Modify: `dashboard/src/app/app/billing/BillingClient.tsx`

**Step 1: Wire real data**

Load:
- recent `BillingTransaction` rows for the organization
- Stripe invoices/payment method summary when `stripeCustomerId` exists

Render them in place of:
- `No recent invoices. Your payment history will appear here.`
- `No fiat card on file`

**Step 2: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/billing/page.tsx src/app/app/billing/BillingClient.tsx`
Expected: PASS

### Task 3: Full Verification

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/billing/BillingOverviewService.test.ts src/app/app/UxBehavior.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS
