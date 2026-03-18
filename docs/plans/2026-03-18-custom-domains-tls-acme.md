# Custom Domains Auto-TLS (ACME) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** To truly match QuickNode and Chainstack, allowing users to specify a vanity domain (`rpc.mycompany.com`) is not enough. The Edge API Gateway (APISIX) must automatically provision and serve a valid TLS/SSL certificate (HTTPS) for that custom domain via Let's Encrypt / ACME.

**Architecture:**
1. **APISIX ACME Integration:** When we create or update a route in APISIX via `ApisixService.createRoute`, APISIX needs to be instructed to manage the SNI (Server Name Indication) and SSL for the custom domain. We need to create an APISIX SSL Object using the `public-api` / ACME plugin instead of just adding a route.
2. **ApisixService Expansion:** 
   - Add `syncSslCertificate(domain: string)` to `ApisixService.ts`.
   - Update `updateEndpointDomainAction` to call this new method when a vanity domain is configured.

**Tech Stack:** Next.js App Router, TypeScript, Apache APISIX REST API.

### Task 1: Update ApisixService for SSL Management

**Files:**
- Modify: `dashboard/src/services/apisix/ApisixService.ts`
- Modify: `dashboard/src/services/apisix/ApisixService.test.ts`

**Step 1:**
- Write a failing test in `ApisixService.test.ts` verifying that `syncSslCertificate` makes a PUT request to `/apisix/admin/ssls/{domain}` to configure the ACME Let's Encrypt certificate.

**Step 2:**
- Implement the `syncSslCertificate` method in `ApisixService.ts` pointing to the `/apisix/admin/ssls` endpoint with the correct SNI and `key` / `cert` fields mapped for ACME. *(Note: APISIX 3.x uses a specific `ssl` object payload for SNI mapping. To use the built-in ACME plugin, we just need to make sure the route hostname is bound, but explicitly configuring an SSL object triggers the SNI handshake).*

### Task 2: Update Server Action

**Files:**
- Modify: `dashboard/src/app/app/endpoints/settingsActions.ts`

**Step 1:**
- In `updateEndpointDomainAction`, after successfully calling `ApisixService.createRoute` with the new vanity domain, detect if it's a custom vanity domain. If so, await `ApisixService.syncSslCertificate(customDomain)`.

### Task 3: Full Verification

**Step 1: Verify the build**
- Run: `npm run verify`
- Expected: PASS
