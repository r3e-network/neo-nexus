# Node Settings Runtime Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make endpoint settings updates actually affect the managed node runtime, then align the settings UI so it only exposes fields the runtime really supports.

**Architecture:** Introduce a pure renderer that turns persisted `NodeSettings` plus `clientEngine` into concrete runtime artifacts: Neo N3 config YAML for `neo-go`, docker run flags for `neo-cli`, and docker run flags for `neo-x-geth`. Bootstrap the VM with a `neonexus-node-sync` script that reads `/etc/neonexus/node-settings.json`, rewrites the engine config/run command, and recreates the node container. Reuse that same script when settings are updated remotely so the control plane and VM stay consistent.

**Tech Stack:** Next.js, TypeScript, Vitest, cloud-init bootstrap scripts, Docker-on-VM runtime

### Task 1: Render Runtime Settings From Persisted NodeSettings

**Files:**
- Create: `dashboard/src/services/settings/RenderedNodeRuntime.ts`
- Create: `dashboard/src/services/settings/RenderedNodeRuntime.test.ts`
- Modify: `dashboard/src/services/settings/NodeSettings.ts`

**Step 1: Write the failing test**

```ts
it('renders neo-go settings into rpc, websocket, and peer config', () => {
  const rendered = renderNodeRuntimeArtifacts({
    clientEngine: 'neo-go',
    network: 'mainnet',
    settings: {
      maxPeers: 42,
      rpcEnabled: true,
      websocketEnabled: true,
      graphqlEnabled: false,
      cacheMb: null,
    },
  });

  expect(rendered.neoGoConfig).toContain('MaxPeers: 42');
  expect(rendered.neoGoConfig).toContain('Enabled: true');
  expect(rendered.runCommand).toContain('nspccdev/neo-go:0.106.0');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/settings/RenderedNodeRuntime.test.ts`
Expected: FAIL because the renderer file does not exist yet.

**Step 3: Write minimal implementation**

Implement a pure renderer that returns:
- `neoGoConfig` when `clientEngine === 'neo-go'`
- `runCommand` for all supported engines
- only settings that actually map to the current engine

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/settings/RenderedNodeRuntime.test.ts`
Expected: PASS

### Task 2: Bootstrap The VM With A Real Node Sync Script

**Files:**
- Modify: `dashboard/src/services/provisioning/NodeBootstrap.ts`
- Test: `dashboard/src/services/provisioning/NodeBootstrap.test.ts`

**Step 1: Write the failing test**

```ts
it('includes a node sync script that reads node-settings.json before starting the container', () => {
  const script = buildNodeBootstrapScript(...);
  expect(script).toContain('neonexus-node-sync');
  expect(script).toContain('/etc/neonexus/node-settings.json');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/provisioning/NodeBootstrap.test.ts`
Expected: FAIL because bootstrap currently has no node sync script.

**Step 3: Write minimal implementation**

Bootstrap should:
- write `/usr/local/bin/neonexus-node-sync`
- write default `/etc/neonexus/node-settings.json`
- use the renderer output to recreate `neonexus-node`
- call `neonexus-node-sync` from cloud-init instead of a hardcoded docker run command

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/provisioning/NodeBootstrap.test.ts`
Expected: PASS

### Task 3: Apply Settings Updates Through The Node Sync Script

**Files:**
- Modify: `dashboard/src/services/settings/RemoteNodeSettings.ts`
- Modify: `dashboard/src/app/app/endpoints/settingsActions.ts`
- Test: `dashboard/src/services/settings/RemoteNodeSettings.test.ts`

**Step 1: Write the failing test**

```ts
it('syncs settings and runs neonexus-node-sync on the remote host', () => {
  const command = buildRemoteNodeSettingsSyncCommand(...);
  expect(command).toContain('node-settings.json');
  expect(command).toContain('neonexus-node-sync');
});
```

**Step 2: Run test to verify it fails**

Run: `npm run test --workspace=dashboard -- src/services/settings/RemoteNodeSettings.test.ts`
Expected: FAIL because the remote sync command only restarts Docker.

**Step 3: Write minimal implementation**

Change the remote sync command to:
- upload the rendered settings JSON
- execute `/usr/local/bin/neonexus-node-sync`
- let that script recreate the runtime safely

**Step 4: Run test to verify it passes**

Run: `npm run test --workspace=dashboard -- src/services/settings/RemoteNodeSettings.test.ts`
Expected: PASS

### Task 4: Align The Settings UI With Supported Runtime Fields

**Files:**
- Modify: `dashboard/src/app/app/endpoints/[id]/EndpointDetailsClient.tsx`

**Step 1: Write the failing test**

No dedicated component test exists today; rely on the pure runtime renderer tests from Task 1 and manual code review of this file.

**Step 2: Implement minimal UI alignment**

For `neo-go` / `neo-cli`:
- remove the fake `RPC.MaxGasInvoke` binding to `cacheMb`
- replace unrelated labels like “P2P Notary Request Payload” and “DBFT Consensus Logging”
- expose only the generic fields that actually map to stored settings

For `neo-x-geth`:
- keep cache and websocket controls that map to the actual runtime renderer

**Step 3: Verify**

Run: `npm run lint --workspace=dashboard -- src/app/app/endpoints/[id]/EndpointDetailsClient.tsx`
Expected: PASS

### Task 5: Full Verification

**Files:**
- Modify docs only if the supported settings matrix needs explanation

**Step 1: Run focused tests**

Run: `npm run test --workspace=dashboard -- src/services/settings/RenderedNodeRuntime.test.ts src/services/settings/RemoteNodeSettings.test.ts src/services/provisioning/NodeBootstrap.test.ts`
Expected: PASS

**Step 2: Run the full verification suite**

Run: `npm run verify`
Expected: PASS

**Step 3: Review the diff**

Run: `git diff -- dashboard/src/services/settings dashboard/src/services/provisioning/NodeBootstrap.ts dashboard/src/app/app/endpoints/[id]/EndpointDetailsClient.tsx docs/plans/2026-03-17-node-settings-runtime.md`
Expected: Only settings-runtime and aligned UI changes.
