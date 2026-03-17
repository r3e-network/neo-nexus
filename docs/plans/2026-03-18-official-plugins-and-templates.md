# Official Neo Plugins & Node Templates Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Expand the platform to officially support the complete suite of standard Neo plugins (e.g. ApplicationLogs, RpcServer, OracleService, DBFTPlugin, TokensTracker, StateService, etc.). Introduce a "Node Template" system (e.g., Consensus Node, RPC Node, Oracle Node) to pre-configure nodes and plugins optimally based on their intended role. Enable advanced configurations (like enabling/disabling specific RPC methods) directly from the Web UI.

**Architecture:**
1. **Plugin Catalog Expansion:** Update the `PluginCatalog` to define schemas and expected configuration formats for all official Neo-CLI plugins.
2. **Node Templates (Presets):** Introduce a new concept of `NodeTemplate` during the Provisioning flow. A template defines the base engine, default `NodeSettings`, and a default set of `NodePlugins`.
    * *Consensus Node:* Includes DBFTPlugin.
    * *RPC Node:* Includes RpcServer, ApplicationLogs, TokensTracker, StateService.
    * *Oracle Node:* Includes OracleService.
3. **Advanced Settings UI:** Enhance the `Settings` and `Plugins` UI to support dynamic form generation based on the plugin's configuration schema (e.g., toggling specific RPC methods, setting Oracle keys).
4. **Configuration Sync Automation:** Ensure the `RenderedNodeRuntime.ts` and `RemotePluginSync.ts` are robust enough to handle these complex, multi-file configuration updates across different node types.

**Tech Stack:** Next.js App Router, Prisma, TypeScript, Vitest, React Hook Form (or native controlled components).

### Task 1: Expand the Plugin Catalog

**Files:**
- Modify: `dashboard/src/services/plugins/PluginCatalog.ts`
- Modify: `dashboard/src/services/plugins/PluginCatalog.test.ts`
- Modify: `dashboard/src/services/plugins/PluginConfigRenderer.ts`
- Modify: `dashboard/src/services/plugins/PluginConfigRenderer.test.ts`

**Step 1: Write the failing tests**
- Add tests in `PluginCatalog.test.ts` verifying the presence of all official plugins (`ApplicationLogs`, `DBFTPlugin`, `OracleService`, `RestServer`, `RpcServer`, `StateService`, `TokensTracker`, etc.).
- Add tests in `PluginConfigRenderer.test.ts` to ensure default configs can be rendered for these new plugins based on expected Neo-CLI `config.json` structures.

**Step 2: Run tests to verify they fail**
Run: `npm run test --workspace=dashboard -- src/services/plugins/PluginCatalog.test.ts src/services/plugins/PluginConfigRenderer.test.ts`
Expected: FAIL

**Step 3: Implement the expanded catalog**
- Update `SUPPORTED_PLUGINS` in `PluginCatalog.ts` with the new definitions. Include metadata about configurable parameters (e.g., `MaxLogSize`, `DisabledMethods` for RpcServer).
- Update `renderPluginConfig` to handle formatting these configurations correctly for Neo-CLI.

**Step 4: Run tests to verify they pass**
Run: `npm run test --workspace=dashboard -- src/services/plugins/PluginCatalog.test.ts src/services/plugins/PluginConfigRenderer.test.ts`
Expected: PASS

### Task 2: Implement Node Templates System

**Files:**
- Create: `dashboard/src/services/provisioning/NodeTemplates.ts`
- Create: `dashboard/src/services/provisioning/NodeTemplates.test.ts`
- Modify: `dashboard/src/app/app/endpoints/new/page.tsx`
- Modify: `dashboard/src/app/api/endpoints/route.ts` (or relevant actions for provisioning)

**Step 1: Define Templates**
- Create `NodeTemplates.ts` defining presets: `RPC_NODE`, `CONSENSUS_NODE`, `ORACLE_NODE`, `CUSTOM_NODE`.
- Each template should specify: `clientEngine` (default), `settings` (NodeSettings defaults), `plugins` (list of plugin IDs to install automatically).

**Step 2: Write tests for NodeTemplates**
- Verify templates return the correct base configuration.

**Step 3: Update Provisioning UI**
- In `app/endpoints/new/page.tsx`, add a step/selector for "Node Template" before selecting the network/engine.
- When a template is selected, pre-fill the recommended engine and schedule the corresponding plugins for installation during the provisioning flow.

### Task 3: Advanced UI Configuration for Plugins

**Files:**
- Modify: `dashboard/src/app/app/endpoints/[id]/EndpointDetailsClient.tsx`
- Modify: `dashboard/src/app/app/endpoints/pluginActions.ts`

**Step 1: Dynamic Plugin Config UI**
- In `EndpointDetailsClient.tsx`, instead of a generic "Configuration (JSON/Text)" textarea, render specific input fields based on the selected plugin's schema (defined in `PluginCatalog`).
- E.g., for `RpcServer`, show a multiselect/tag input for `DisabledMethods`.
- E.g., for `ApplicationLogs`, show a number input for `MaxLogSize`.
- For secrets (like DBFT private key or Oracle key), keep using the secure Vault flow.

**Step 2: Update Actions**
- Ensure `addNodePluginAction` and `updatePluginConfigAction` (new action needed) can process these structured configuration objects and pass them to the `PluginConfigRenderer`.

### Task 4: Full Verification

**Step 1: Run the full verification suite**
Run: `npm run verify`
Expected: PASS
