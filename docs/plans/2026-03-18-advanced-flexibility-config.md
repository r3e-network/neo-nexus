# Advanced Node Flexibility & Configuration Injection Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Provide ultimate flexibility for advanced users who need to customize the node beyond the standard form fields. This means supporting custom Environment Variables injection and custom Docker run flags for the node engine directly from the dashboard UI.

**Architecture:**
1. **Extend `NodeSettings`:** Add two new fields to the `NodeSettings` type: `envVars: Record<string, string>` and `customDockerFlags: string`.
2. **Runtime Integration (`RenderedNodeRuntime.ts`):** Modify the `buildNeoGoRunCommand`, `buildNeoCliRunCommand`, and `buildNeoXRunCommand` functions to loop over the `envVars` and inject `-e KEY=VALUE`, and append the `customDockerFlags` string to the `docker run` command string.
3. **UI Implementation:** In the Settings tab of `EndpointDetailsClient.tsx`, add an "Advanced Configuration" expandable section. Within it, provide a key-value builder for Environment Variables and a text input for Custom Docker Flags.
4. **Validation:** Ensure the input is correctly formatted and safely injected into the Bash script generation.

**Tech Stack:** Next.js App Router, TypeScript, React, Prisma JSON columns.

### Task 1: Extend NodeSettings Engine

**Files:**
- Modify: `dashboard/src/services/settings/NodeSettings.ts`
- Modify: `dashboard/src/services/settings/NodeSettings.test.ts`
- Modify: `dashboard/src/services/settings/RenderedNodeRuntime.ts`
- Modify: `dashboard/src/services/settings/RenderedNodeRuntime.test.ts`

**Step 1: Write failing tests**
- In `NodeSettings.test.ts`, ensure `mergeNodeSettings` handles `envVars` and `customDockerFlags`.
- In `RenderedNodeRuntime.test.ts`, assert that passing `envVars: { 'NEO_DEBUG': '1' }` results in `-e NEO_DEBUG=1` in the run command, and `customDockerFlags: '--memory=4g'` is appended.
- Run: `npm run test --workspace=dashboard -- src/services/settings/NodeSettings.test.ts src/services/settings/RenderedNodeRuntime.test.ts`
- Expected: FAIL

**Step 2: Implement Data Structures and Generators**
- Update `NodeSettings` type.
- Modify the three `buildXRunCommand` functions in `RenderedNodeRuntime.ts`. Add `-e` string formatting and append the custom flags. Note: Sanitize or escape env var values to prevent bash injection.

**Step 3: Verify tests pass**
- Run: `npm run test --workspace=dashboard -- src/services/settings/NodeSettings.test.ts src/services/settings/RenderedNodeRuntime.test.ts`
- Expected: PASS

### Task 2: Implement UI for Advanced Settings

**Files:**
- Modify: `dashboard/src/app/app/endpoints/[id]/EndpointDetailsClient.tsx`

**Step 1: Expand Settings State**
- Support `envVars` object and `customDockerFlags` string in the component state.

**Step 2: Render Advanced Section**
- Below the existing sliders/toggles, add a collapsible `<details>` or toggleable UI section labeled "Advanced (Danger Zone)".
- Implement a key/value input list for Environment Variables.
- Implement a single text input for Extra Docker Flags.

### Task 3: Full Verification

**Step 1: Verify the build**
- Run: `npm run verify`
- Expected: PASS
