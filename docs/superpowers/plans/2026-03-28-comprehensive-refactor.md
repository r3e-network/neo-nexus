# NeoNexus Comprehensive Refactoring Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor NeoNexus into a professional, production-ready codebase — eliminating duplication, decomposing oversized modules, fixing broken tooling, tightening type safety, and splitting bloated frontend components.

**Architecture:** Incremental refactor with tests verified between each task. Backend: extract shared RPC logic to BaseNode, decompose NodeManager into focused classes (repository + lifecycle), extract hardcoded constants to data files, fix ESLint. Frontend: split 3 oversized page components, extract shared UI components, centralize configuration constants.

**Tech Stack:** TypeScript 5.8, Node.js 20+, Express 4, React 19, Vite 6, Tailwind CSS 4, Vitest, ESLint 9

---

## File Structure

### New Files (Backend)
- `eslint.config.mjs` — ESLint 9 flat config for backend
- `web/eslint.config.mjs` — ESLint 9 flat config for frontend
- `src/core/NodeRepository.ts` — Database CRUD operations extracted from NodeManager
- `src/data/neo-committee.ts` — Hardcoded committee keys and hardfork heights
- `src/data/plugin-versions.ts` — Plugin version mapping extracted from PluginManager
- `src/types/database.ts` — Shared database row type definitions

### New Files (Frontend)
- `web/src/components/SignerStatus.tsx` — Shared signer status badge (deduplicated)
- `web/src/components/NodeProtectionLabel.tsx` — Shared protection label
- `web/src/components/ToggleSwitch.tsx` — Reusable toggle switch
- `web/src/pages/settings/PasswordSection.tsx` — Extracted from Settings.tsx
- `web/src/pages/settings/StorageSection.tsx` — Extracted from Settings.tsx
- `web/src/pages/settings/SecureSignerSection.tsx` — Extracted from Settings.tsx
- `web/src/pages/settings/DangerZoneSection.tsx` — Extracted from Settings.tsx
- `web/src/pages/plugins/PluginCard.tsx` — Extracted from Plugins.tsx
- `web/src/pages/plugins/ConfigField.tsx` — Extracted from Plugins.tsx
- `web/src/pages/node-detail/NodeConfigEditor.tsx` — Extracted from NodeDetail.tsx
- `web/src/pages/node-detail/NodeLogsView.tsx` — Extracted from NodeDetail.tsx
- `web/src/config/constants.ts` — Centralized refetch intervals, timeouts, limits

### Modified Files
- `src/nodes/BaseNode.ts` — Add `executeRpc()` method
- `src/nodes/NeoCliNode.ts` — Remove `executeCommand()`, use inherited `executeRpc()`
- `src/nodes/NeoGoNode.ts` — Remove `executeCommand()`, use inherited `executeRpc()`
- `src/core/NodeManager.ts` — Extract DB ops to NodeRepository, shrink from 1019 to ~500 LOC
- `src/core/ConfigManager.ts` — Import constants from data files instead of inline
- `src/core/PluginManager.ts` — Import version mapping, deduplicate `getNodeConfig()`
- `src/server.ts` — Extract magic numbers to named constants
- `src/types/index.ts` — Use `Omit`/`Pick` to deduplicate signer request types
- `web/src/pages/Settings.tsx` — Slim down from 973 to ~100 LOC (imports sections)
- `web/src/pages/Plugins.tsx` — Slim down from 545 to ~200 LOC
- `web/src/pages/NodeDetail.tsx` — Slim down from 664 to ~300 LOC
- `web/src/pages/Dashboard.tsx` — Use shared SignerStatus component
- `web/src/pages/Nodes.tsx` — Use shared SignerStatus/ProtectionLabel components

---

## Task 1: Fix ESLint Configuration

**Files:**
- Create: `eslint.config.mjs`
- Create: `web/eslint.config.mjs`
- Modify: `package.json` (add `typescript-eslint` dev dependency)

- [ ] **Step 1: Install typescript-eslint**

```bash
cd /home/neo/git/neo-nexus && npm install --save-dev typescript-eslint @eslint/js
```

- [ ] **Step 2: Create backend ESLint config**

Create `eslint.config.mjs`:

```javascript
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-require-imports': 'off',
    },
  },
  {
    ignores: ['dist/', 'web/', 'archive/', 'node_modules/', 'tests/'],
  },
);
```

- [ ] **Step 3: Create frontend ESLint config**

Create `web/eslint.config.mjs`:

```javascript
import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import reactHooks from 'eslint-plugin-react-hooks';
import reactRefresh from 'eslint-plugin-react-refresh';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    plugins: {
      'react-hooks': reactHooks,
      'react-refresh': reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      'react-refresh/only-export-components': ['warn', { allowConstantExport: true }],
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      '@typescript-eslint/no-explicit-any': 'warn',
    },
  },
  {
    ignores: ['dist/', 'node_modules/'],
  },
);
```

- [ ] **Step 4: Install frontend eslint deps**

```bash
cd /home/neo/git/neo-nexus/web && npm install --save-dev typescript-eslint @eslint/js
```

- [ ] **Step 5: Verify lint runs**

```bash
cd /home/neo/git/neo-nexus && npm run lint
cd /home/neo/git/neo-nexus/web && npm run lint
```

Expected: Both lint commands execute (warnings are OK, no config errors).

- [ ] **Step 6: Run tests to verify nothing broke**

```bash
cd /home/neo/git/neo-nexus && npm test
```

Expected: 272 tests passing.

- [ ] **Step 7: Commit**

```bash
git add eslint.config.mjs web/eslint.config.mjs package.json package-lock.json web/package.json web/package-lock.json
git commit -m "build: add ESLint 9 flat config for backend and frontend"
```

---

## Task 2: Extract RPC Command to BaseNode

**Files:**
- Modify: `src/nodes/BaseNode.ts` (add `executeRpc()`)
- Modify: `src/nodes/NeoCliNode.ts` (remove `executeCommand()`, delegate)
- Modify: `src/nodes/NeoGoNode.ts` (remove `executeCommand()`, delegate)

The `executeCommand()` method is identical (lines 59-91 in NeoCliNode, lines 45-75 in NeoGoNode). Move it to BaseNode.

- [ ] **Step 1: Add `executeRpc()` to BaseNode**

Add the following method to `src/nodes/BaseNode.ts` after the `getResourceUsage()` method (after line 202):

```typescript
  /**
   * Execute an RPC command on the running node via JSON-RPC.
   */
  async executeRpc(method: string, ...params: string[]): Promise<string> {
    if (!this.isRunning()) {
      throw new Error('Node is not running');
    }

    const rpcUrl = `http://127.0.0.1:${this.config.ports.rpc}`;

    const response = await fetch(rpcUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method,
        params,
        id: 1,
      }),
    });

    if (!response.ok) {
      throw new Error(`RPC call failed: ${response.statusText}`);
    }

    const data = await response.json() as { error?: { message: string }; result: unknown };
    if (data.error) {
      throw new Error(`RPC error: ${data.error.message}`);
    }

    return JSON.stringify(data.result);
  }
```

- [ ] **Step 2: Update NeoCliNode to use inherited method**

In `src/nodes/NeoCliNode.ts`, remove the entire `executeCommand()` method (lines 59-91). Replace `this.executeCommand(` calls with `this.executeRpc(`:

- Line 138: `const result = await this.executeRpc('getblockcount');`
- Line 151: `const result = await this.executeRpc('getpeers');`

- [ ] **Step 3: Update NeoGoNode to use inherited method**

In `src/nodes/NeoGoNode.ts`, remove the entire `executeCommand()` method (lines 45-75). Replace `this.executeCommand(` calls with `this.executeRpc(`:

- Line 117: `const result = await this.executeRpc('getblockcount');`
- Line 130: `const result = await this.executeRpc('getpeers');`

- [ ] **Step 4: Run tests**

```bash
cd /home/neo/git/neo-nexus && npm test
```

Expected: All 272 tests pass.

- [ ] **Step 5: Run typecheck**

```bash
cd /home/neo/git/neo-nexus && npm run typecheck
```

Expected: No errors.

- [ ] **Step 6: Commit**

```bash
git add src/nodes/BaseNode.ts src/nodes/NeoCliNode.ts src/nodes/NeoGoNode.ts
git commit -m "refactor: extract duplicated RPC command to BaseNode.executeRpc()"
```

---

## Task 3: Extract Database Row Types

**Files:**
- Create: `src/types/database.ts`
- Modify: `src/core/NodeManager.ts` (import shared types)
- Modify: `src/core/PluginManager.ts` (import shared types)

The same database row shapes are defined inline in 3 places (NodeManager lines 852-871, PluginManager lines 269-288, and similar).

- [ ] **Step 1: Create shared database row types**

Create `src/types/database.ts`:

```typescript
import type { NodeType, NodeNetwork, SyncMode, NodeStatus } from './index';

export interface NodeRow {
  id: string;
  name: string;
  type: NodeType;
  network: NodeNetwork;
  sync_mode: SyncMode;
  version: string;
  rpc_port: number;
  p2p_port: number;
  websocket_port: number | null;
  metrics_port: number | null;
  base_path: string;
  data_path: string;
  logs_path: string;
  config_path: string;
  wallet_path: string | null;
  settings: string;
  created_at: number;
  updated_at: number;
}

export interface ProcessRow {
  status: NodeStatus;
  pid: number | null;
  error_message: string | null;
}

export interface MetricsRow {
  block_height: number;
  header_height: number;
  connected_peers: number;
  unconnected_peers: number;
  sync_progress: number;
  memory_usage: number;
  cpu_usage: number;
  last_update: number;
}

export interface PluginRow {
  id: string;
  name: string;
  description: string;
  category: string;
  requires_config: number;
  dependencies: string | null;
  default_config: string | null;
}

export interface InstalledPluginRow {
  plugin_id: string;
  version: string;
  config: string | null;
  installed_at: number;
  enabled: number;
}

export function nodeRowToConfig(row: NodeRow) {
  return {
    id: row.id,
    name: row.name,
    type: row.type,
    network: row.network,
    syncMode: row.sync_mode,
    version: row.version,
    ports: {
      rpc: row.rpc_port,
      p2p: row.p2p_port,
      websocket: row.websocket_port ?? undefined,
      metrics: row.metrics_port ?? undefined,
    },
    paths: {
      base: row.base_path,
      data: row.data_path,
      logs: row.logs_path,
      config: row.config_path,
      wallet: row.wallet_path ?? undefined,
    },
    settings: row.settings ? JSON.parse(row.settings) : {},
    createdAt: row.created_at,
    updatedAt: row.updated_at,
  };
}
```

- [ ] **Step 2: Update NodeManager to use shared types**

In `src/core/NodeManager.ts`:
- Add import: `import { NodeRow, ProcessRow, MetricsRow, nodeRowToConfig } from '../types/database';`
- Replace inline type in `getNodeConfigFromDb()` (lines 852-871) with `as NodeRow | undefined`
- Replace inline mapping (lines 875-898) with `return nodeRowToConfig(row);`
- Replace inline type in `getProcessFromDb()` (line 903) with `as ProcessRow | undefined`
- Replace inline type in `getMetricsFromDb()` (line 918) with `as MetricsRow | undefined`

- [ ] **Step 3: Update PluginManager to use shared types**

In `src/core/PluginManager.ts`:
- Add import: `import { NodeRow, PluginRow, InstalledPluginRow, nodeRowToConfig } from '../types/database';`
- Replace inline type in `getAvailablePlugins()` (lines 40-48) with `as PluginRow[]`
- Replace inline type in `getPlugin()` (lines 66-74) with `as PluginRow | undefined`
- Replace inline type in `getInstalledPlugins()` (lines 99-105) with `as InstalledPluginRow[]`
- Replace inline type in `getNodeConfig()` (lines 269-288) with `as NodeRow | undefined`
- Replace the manual mapping (lines 294-318) with `return nodeRowToConfig(row);`

- [ ] **Step 4: Run tests and typecheck**

```bash
cd /home/neo/git/neo-nexus && npm test && npm run typecheck
```

Expected: All passing.

- [ ] **Step 5: Commit**

```bash
git add src/types/database.ts src/core/NodeManager.ts src/core/PluginManager.ts
git commit -m "refactor: extract shared database row types to src/types/database.ts"
```

---

## Task 4: Extract NodeRepository from NodeManager

**Files:**
- Create: `src/core/NodeRepository.ts`
- Modify: `src/core/NodeManager.ts`
- Create: `tests/unit/NodeRepository.test.ts`

Extract the 7 private database methods from NodeManager (lines 806-996) into a dedicated NodeRepository class.

- [ ] **Step 1: Create NodeRepository**

Create `src/core/NodeRepository.ts`:

```typescript
import type Database from 'better-sqlite3';
import type { NodeConfig, NodeStatus, NodeMetrics, LogEntry } from '../types/index';
import { NodeRow, ProcessRow, MetricsRow, nodeRowToConfig } from '../types/database';

export class NodeRepository {
  constructor(private db: Database.Database) {}

  saveNode(config: NodeConfig): void {
    const stmt = this.db.prepare(`
      INSERT INTO nodes (
        id, name, type, network, sync_mode, version,
        rpc_port, p2p_port, websocket_port, metrics_port,
        base_path, data_path, logs_path, config_path, wallet_path,
        settings, created_at, updated_at
      ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `);

    stmt.run(
      config.id, config.name, config.type, config.network,
      config.syncMode, config.version,
      config.ports.rpc, config.ports.p2p,
      config.ports.websocket ?? null, config.ports.metrics ?? null,
      config.paths.base, config.paths.data, config.paths.logs,
      config.paths.config, config.paths.wallet ?? null,
      JSON.stringify(config.settings), config.createdAt, config.updatedAt,
    );

    this.db.prepare(`INSERT INTO node_processes (node_id, status) VALUES (?, 'stopped')`).run(config.id);
    this.db.prepare(`INSERT INTO node_metrics (node_id) VALUES (?)`).run(config.id);
  }

  getNodeConfig(nodeId: string): NodeConfig | null {
    const row = this.db.prepare('SELECT * FROM nodes WHERE id = ?').get(nodeId) as NodeRow | undefined;
    if (!row) return null;
    return nodeRowToConfig(row);
  }

  getProcess(nodeId: string): { status: NodeStatus; pid?: number; errorMessage?: string } {
    const row = this.db.prepare('SELECT * FROM node_processes WHERE node_id = ?').get(nodeId) as ProcessRow | undefined;
    return {
      status: row?.status ?? 'stopped',
      pid: row?.pid ?? undefined,
      errorMessage: row?.error_message ?? undefined,
    };
  }

  getMetrics(nodeId: string): NodeMetrics | undefined {
    const row = this.db.prepare('SELECT * FROM node_metrics WHERE node_id = ?').get(nodeId) as MetricsRow | undefined;
    if (!row) return undefined;
    return {
      blockHeight: row.block_height,
      headerHeight: row.header_height,
      connectedPeers: row.connected_peers,
      unconnectedPeers: row.unconnected_peers,
      syncProgress: row.sync_progress,
      memoryUsage: row.memory_usage,
      cpuUsage: row.cpu_usage,
      lastUpdate: row.last_update,
    };
  }

  getAllNodeIds(): string[] {
    const rows = this.db.prepare('SELECT id FROM nodes').all() as Array<{ id: string }>;
    return rows.map(r => r.id);
  }

  updateStatus(nodeId: string, status: NodeStatus, pid?: number): void {
    const now = Date.now();
    if (status === 'running') {
      this.db.prepare(`UPDATE node_processes SET status = ?, pid = ?, last_started = ? WHERE node_id = ?`)
        .run(status, pid ?? null, now, nodeId);
    } else if (status === 'stopped') {
      this.db.prepare(`UPDATE node_processes SET status = ?, pid = NULL, last_stopped = ? WHERE node_id = ?`)
        .run(status, now, nodeId);
    } else {
      this.db.prepare(`UPDATE node_processes SET status = ?, pid = ? WHERE node_id = ?`)
        .run(status, pid ?? null, nodeId);
    }
  }

  saveLogEntry(nodeId: string, entry: LogEntry): void {
    this.db.prepare(`INSERT INTO logs (node_id, timestamp, level, source, message) VALUES (?, ?, ?, ?, ?)`)
      .run(nodeId, entry.timestamp, entry.level, entry.source, entry.message);
  }

  saveMetrics(nodeId: string, metrics: NodeMetrics): void {
    this.db.prepare(`
      UPDATE node_metrics
      SET block_height = ?, header_height = ?, connected_peers = ?,
          unconnected_peers = ?, sync_progress = ?, memory_usage = ?,
          cpu_usage = ?, last_update = ?
      WHERE node_id = ?
    `).run(
      metrics.blockHeight, metrics.headerHeight, metrics.connectedPeers,
      metrics.unconnectedPeers, metrics.syncProgress, metrics.memoryUsage,
      metrics.cpuUsage, metrics.lastUpdate, nodeId,
    );
  }

  updateSyncProgress(nodeId: string, syncProgress: number): void {
    this.db.prepare('UPDATE node_metrics SET sync_progress = ? WHERE node_id = ?')
      .run(syncProgress, nodeId);
  }

  deleteNode(nodeId: string): void {
    this.db.prepare('DELETE FROM nodes WHERE id = ?').run(nodeId);
  }

  updateNode(nodeId: string, updates: string[], values: (string | number)[]): void {
    this.db.prepare(`UPDATE nodes SET ${updates.join(', ')} WHERE id = ?`).run(...values, nodeId);
  }

  transaction<T>(fn: () => T): T {
    return this.db.transaction(fn)();
  }
}
```

- [ ] **Step 2: Update NodeManager to use NodeRepository**

In `src/core/NodeManager.ts`:
- Add import: `import { NodeRepository } from './NodeRepository';`
- Add field: `private repo: NodeRepository;`
- In constructor, add: `this.repo = new NodeRepository(db);`
- Replace all calls to private DB methods with `this.repo.*`:
  - `this.saveNodeToDb(config)` → `this.repo.saveNode(config)`
  - `this.getNodeConfigFromDb(nodeId)` → `this.repo.getNodeConfig(nodeId)`
  - `this.getProcessFromDb(nodeId)` → `this.repo.getProcess(nodeId)`
  - `this.getMetricsFromDb(nodeId)` → `this.repo.getMetrics(nodeId)`
  - `this.updateNodeStatus(nodeId, ...)` → `this.repo.updateStatus(nodeId, ...)`
  - `this.saveLogEntry(nodeId, entry)` → `this.repo.saveLogEntry(nodeId, entry)`
  - `this.saveMetrics(nodeId, metrics)` → `this.repo.saveMetrics(nodeId, metrics)`
- Remove all 7 private DB methods (lines 806-996)
- Replace `getAllNodes()` to use `this.repo.getAllNodeIds()`
- Replace `updateSyncProgress()` to delegate to `this.repo.updateSyncProgress()`
- Expose `repo` getter for tests: `getRepository(): NodeRepository { return this.repo; }`

- [ ] **Step 3: Run tests**

```bash
cd /home/neo/git/neo-nexus && npm test
```

Expected: All 272 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/core/NodeRepository.ts src/core/NodeManager.ts
git commit -m "refactor: extract database operations from NodeManager to NodeRepository"
```

---

## Task 5: Extract Hardcoded Constants to Data Files

**Files:**
- Create: `src/data/neo-committee.ts`
- Create: `src/data/plugin-versions.ts`
- Modify: `src/core/ConfigManager.ts`
- Modify: `src/core/PluginManager.ts`

- [ ] **Step 1: Create neo-committee.ts**

Create `src/data/neo-committee.ts` — move the `NEO_GO_STANDBY_COMMITTEE` and `NEO_GO_HARDFORKS` constants from `src/core/ConfigManager.ts` (lines 7-73) into this file:

```typescript
import type { NodeNetwork } from '../types/index';

/**
 * Standby committee public keys for neo-go configuration.
 * Source: https://github.com/neo-project/neo/blob/master/src/Neo/ProtocolSettings.cs
 */
export const NEO_GO_STANDBY_COMMITTEE: Record<Exclude<NodeNetwork, 'private'>, string[]> = {
  // ... exact same data from ConfigManager.ts lines 8-53
};

/**
 * Hardfork block heights for neo-go configuration.
 * Source: https://github.com/nspcc-dev/neo-go/tree/master/config
 */
export const NEO_GO_HARDFORKS: Record<Exclude<NodeNetwork, 'private'>, Record<string, number>> = {
  // ... exact same data from ConfigManager.ts lines 57-73
};
```

- [ ] **Step 2: Create plugin-versions.ts**

Create `src/data/plugin-versions.ts` — move the `PLUGIN_VERSIONS` constant from `src/core/PluginManager.ts` (lines 9-25):

```typescript
/**
 * Maps neo-cli versions to compatible plugin release versions.
 * When a node version is not found, falls back to the latest available.
 *
 * Source: https://github.com/neo-project/neo-modules/releases
 */
export const PLUGIN_VERSIONS: Record<string, string> = {
  // ... exact same data from PluginManager.ts lines 9-25
};
```

- [ ] **Step 3: Update ConfigManager imports**

In `src/core/ConfigManager.ts`:
- Remove the inline `NEO_GO_STANDBY_COMMITTEE` and `NEO_GO_HARDFORKS` constants (lines 7-73)
- Add: `import { NEO_GO_STANDBY_COMMITTEE, NEO_GO_HARDFORKS } from '../data/neo-committee';`

- [ ] **Step 4: Update PluginManager imports**

In `src/core/PluginManager.ts`:
- Remove the inline `PLUGIN_VERSIONS` constant (lines 9-25)
- Add: `import { PLUGIN_VERSIONS } from '../data/plugin-versions';`

- [ ] **Step 5: Run tests and typecheck**

```bash
cd /home/neo/git/neo-nexus && npm test && npm run typecheck
```

Expected: All passing.

- [ ] **Step 6: Commit**

```bash
git add src/data/neo-committee.ts src/data/plugin-versions.ts src/core/ConfigManager.ts src/core/PluginManager.ts
git commit -m "refactor: extract hardcoded constants to src/data/ modules"
```

---

## Task 6: Extract Server Constants and Clean Up Magic Numbers

**Files:**
- Modify: `src/server.ts`

- [ ] **Step 1: Extract magic numbers to named constants**

At the top of `src/server.ts` (after imports, before `createAppServer`), add:

```typescript
/** Rate limiting configuration */
const RATE_LIMIT_WINDOW_MS = 15 * 60 * 1000;     // 15 minutes
const RATE_LIMIT_MAX_REQUESTS = 1000;
const AUTH_RATE_LIMIT_MAX = 5;                      // 5 login attempts per window
const CONTROL_RATE_LIMIT_WINDOW_MS = 60 * 1000;    // 1 minute
const CONTROL_RATE_LIMIT_MAX = 10;

/** Periodic task intervals */
const SESSION_CLEANUP_INTERVAL_MS = 60 * 60 * 1000;  // 1 hour
const METRICS_BROADCAST_INTERVAL_MS = 5_000;          // 5 seconds
const LOG_RETENTION_INTERVAL_MS = 10 * 60 * 1000;     // 10 minutes
const NETWORK_HEIGHT_INTERVAL_MS = 60 * 1000;          // 1 minute

/** Shutdown configuration */
const GRACEFUL_SHUTDOWN_TIMEOUT_MS = 5_000;
const FORCE_EXIT_TIMEOUT_MS = 30_000;

/** Log retention defaults */
const DEFAULT_LOG_RETENTION_MAX_ROWS = 50_000;
const DEFAULT_AUDIT_LOG_MAX_ROWS = 100_000;
const DEFAULT_LOG_PRUNE_BATCH_SIZE = 500_000;
```

- [ ] **Step 2: Replace inline magic numbers with constants**

Replace all corresponding magic numbers throughout `server.ts`:
- `15 * 60 * 1000` → `RATE_LIMIT_WINDOW_MS`
- `1000` (rate limit) → `RATE_LIMIT_MAX_REQUESTS`
- `5` (auth limit) → `AUTH_RATE_LIMIT_MAX`
- `60 * 1000` (control) → `CONTROL_RATE_LIMIT_WINDOW_MS`
- `10` (control max) → `CONTROL_RATE_LIMIT_MAX`
- `60 * 60 * 1000` (session cleanup) → `SESSION_CLEANUP_INTERVAL_MS`
- `5000` (metrics) → `METRICS_BROADCAST_INTERVAL_MS`
- `10 * 60 * 1000` (log retention) → `LOG_RETENTION_INTERVAL_MS`
- `60 * 1000` (network) → `NETWORK_HEIGHT_INTERVAL_MS`
- `5000` (force shutdown) → `GRACEFUL_SHUTDOWN_TIMEOUT_MS`
- `30_000` (force exit) → `FORCE_EXIT_TIMEOUT_MS`
- `"50000"` (max per node) → `DEFAULT_LOG_RETENTION_MAX_ROWS`
- `100000` (audit prune) → `DEFAULT_AUDIT_LOG_MAX_ROWS`
- `500000` (log prune batch) → `DEFAULT_LOG_PRUNE_BATCH_SIZE`

- [ ] **Step 3: Run tests**

```bash
cd /home/neo/git/neo-nexus && npm test
```

Expected: All 272 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/server.ts
git commit -m "refactor: extract magic numbers to named constants in server.ts"
```

---

## Task 7: Deduplicate Type Definitions

**Files:**
- Modify: `src/types/index.ts`

- [ ] **Step 1: Deduplicate SecureSigner request types**

In `src/types/index.ts`, replace `UpdateSecureSignerRequest` (lines 310-325) with:

```typescript
export type UpdateSecureSignerRequest = Partial<CreateSecureSignerRequest>;
```

This eliminates 15 lines of duplicate field definitions.

- [ ] **Step 2: Run typecheck**

```bash
cd /home/neo/git/neo-nexus && npm run typecheck
```

Expected: No errors.

- [ ] **Step 3: Run tests**

```bash
cd /home/neo/git/neo-nexus && npm test
```

Expected: All passing.

- [ ] **Step 4: Commit**

```bash
git add src/types/index.ts
git commit -m "refactor: deduplicate SecureSigner request types using Partial<>"
```

---

## Task 8: Centralize Frontend Constants

**Files:**
- Create: `web/src/config/constants.ts`
- Modify: `web/src/hooks/useNodes.ts`
- Modify: `web/src/hooks/usePlugins.ts`
- Modify: `web/src/hooks/useSecureSigners.ts`
- Modify: `web/src/hooks/useServers.ts`
- Modify: `web/src/hooks/usePublic.ts`

- [ ] **Step 1: Create constants file**

Create `web/src/config/constants.ts`:

```typescript
/** React Query refetch intervals (ms) */
export const REFETCH_INTERVALS = {
  dashboard: 5_000,
  nodeDetail: 5_000,
  plugins: 5_000,
  signerHealth: 10_000,
  servers: 15_000,
  publicDashboard: 5_000,
} as const;

/** UI limits */
export const UI_LIMITS = {
  maxNotificationToasts: 3,
  notificationDismissMs: 8_000,
  maxLogEntries: 50,
  maxUnreadBadge: 9,
} as const;
```

- [ ] **Step 2: Update hooks to use centralized constants**

In each hook file, replace hardcoded `refetchInterval` values with imports from `../config/constants`:

For example, in `web/src/hooks/useNodes.ts`, change:
```typescript
refetchInterval: 5000,
```
to:
```typescript
import { REFETCH_INTERVALS } from '../config/constants';
// ...
refetchInterval: REFETCH_INTERVALS.dashboard,
```

Apply the same pattern across all hook files.

- [ ] **Step 3: Run typecheck**

```bash
cd /home/neo/git/neo-nexus/web && npx tsc --noEmit
```

Expected: No errors.

- [ ] **Step 4: Commit**

```bash
git add web/src/config/constants.ts web/src/hooks/
git commit -m "refactor(web): centralize refetch intervals and UI limits"
```

---

## Task 9: Extract Shared Frontend Components

**Files:**
- Create: `web/src/components/SignerStatus.tsx`
- Create: `web/src/components/NodeProtectionLabel.tsx`
- Create: `web/src/components/ToggleSwitch.tsx`
- Modify: `web/src/pages/Dashboard.tsx`
- Modify: `web/src/pages/Nodes.tsx`
- Modify: `web/src/pages/Plugins.tsx`

- [ ] **Step 1: Read current duplicated components**

Read the `DashboardSignerStatus` component from `Dashboard.tsx` and `NodeSignerStatus` from `Nodes.tsx` to understand the shared interface. Read the `ToggleSwitch` component from `Plugins.tsx`.

- [ ] **Step 2: Create SignerStatus component**

Create `web/src/components/SignerStatus.tsx` — extract the shared signer status badge that appears in both Dashboard.tsx and Nodes.tsx. The component should accept a `signerHealth` prop and render the status indicator.

- [ ] **Step 3: Create NodeProtectionLabel component**

Create `web/src/components/NodeProtectionLabel.tsx` — extract the protection label rendering shared between Dashboard.tsx and Nodes.tsx.

- [ ] **Step 4: Create ToggleSwitch component**

Create `web/src/components/ToggleSwitch.tsx` — extract the toggle switch from Plugins.tsx (lines 79-111) into a reusable component.

- [ ] **Step 5: Update Dashboard.tsx and Nodes.tsx**

Replace inline `DashboardSignerStatus`/`NodeSignerStatus` with `<SignerStatus />` import. Replace inline protection label rendering with `<NodeProtectionLabel />` import.

- [ ] **Step 6: Update Plugins.tsx**

Replace inline `ToggleSwitch` with the shared component import.

- [ ] **Step 7: Run typecheck**

```bash
cd /home/neo/git/neo-nexus/web && npx tsc --noEmit
```

Expected: No errors.

- [ ] **Step 8: Commit**

```bash
git add web/src/components/SignerStatus.tsx web/src/components/NodeProtectionLabel.tsx web/src/components/ToggleSwitch.tsx web/src/pages/Dashboard.tsx web/src/pages/Nodes.tsx web/src/pages/Plugins.tsx
git commit -m "refactor(web): extract shared SignerStatus, NodeProtectionLabel, ToggleSwitch components"
```

---

## Task 10: Split Settings.tsx (973 LOC)

**Files:**
- Create: `web/src/pages/settings/PasswordSection.tsx`
- Create: `web/src/pages/settings/StorageSection.tsx`
- Create: `web/src/pages/settings/SecureSignerSection.tsx`
- Create: `web/src/pages/settings/DangerZoneSection.tsx`
- Modify: `web/src/pages/Settings.tsx`

- [ ] **Step 1: Read Settings.tsx to identify section boundaries**

Read the full file to map each independent section (password management, storage operations, secure signer profiles, danger zone).

- [ ] **Step 2: Create PasswordSection**

Extract the password management section into `web/src/pages/settings/PasswordSection.tsx`. This should include the password change form with all its state and handlers.

- [ ] **Step 3: Create StorageSection**

Extract the storage operations section (backup/restore, log cleanup, config export) into `web/src/pages/settings/StorageSection.tsx`.

- [ ] **Step 4: Create SecureSignerSection**

Extract the secure signer profile management (list, create, edit, test) into `web/src/pages/settings/SecureSignerSection.tsx`.

- [ ] **Step 5: Create DangerZoneSection**

Extract the danger zone (stop all nodes, reset all data) into `web/src/pages/settings/DangerZoneSection.tsx`.

- [ ] **Step 6: Slim down Settings.tsx**

Replace the body of `Settings.tsx` with imports of the 4 section components. The page should just be layout + section composition (~100 LOC).

- [ ] **Step 7: Run typecheck**

```bash
cd /home/neo/git/neo-nexus/web && npx tsc --noEmit
```

Expected: No errors.

- [ ] **Step 8: Commit**

```bash
git add web/src/pages/settings/ web/src/pages/Settings.tsx
git commit -m "refactor(web): split Settings.tsx into focused section components"
```

---

## Task 11: Split Plugins.tsx (545 LOC)

**Files:**
- Create: `web/src/pages/plugins/PluginCard.tsx`
- Create: `web/src/pages/plugins/ConfigField.tsx`
- Modify: `web/src/pages/Plugins.tsx`

- [ ] **Step 1: Read Plugins.tsx to identify component boundaries**

Read the full file. The key components to extract:
- `PluginCard` (~138 lines) — the card for each plugin with install/uninstall/config
- `ConfigField` (~50 lines) — individual config field renderer

- [ ] **Step 2: Create ConfigField component**

Extract `ConfigField` into `web/src/pages/plugins/ConfigField.tsx`.

- [ ] **Step 3: Create PluginCard component**

Extract `PluginCard` into `web/src/pages/plugins/PluginCard.tsx`. This needs to accept props for the plugin data, installed status, config draft, and mutation handlers.

- [ ] **Step 4: Slim down Plugins.tsx**

Replace inline components with imports. The page should handle node selection and list rendering (~200 LOC).

- [ ] **Step 5: Run typecheck**

```bash
cd /home/neo/git/neo-nexus/web && npx tsc --noEmit
```

Expected: No errors.

- [ ] **Step 6: Commit**

```bash
git add web/src/pages/plugins/ web/src/pages/Plugins.tsx
git commit -m "refactor(web): split Plugins.tsx into PluginCard and ConfigField components"
```

---

## Task 12: Split NodeDetail.tsx (664 LOC)

**Files:**
- Create: `web/src/pages/node-detail/NodeConfigEditor.tsx`
- Create: `web/src/pages/node-detail/NodeLogsView.tsx`
- Modify: `web/src/pages/NodeDetail.tsx`

- [ ] **Step 1: Read NodeDetail.tsx to identify component boundaries**

Read the full file. The key components to extract:
- `NodeConfigEditor` — the configuration editing form (~200 lines)
- `NodeLogsView` — the log viewer panel (~100 lines)

- [ ] **Step 2: Create NodeConfigEditor component**

Extract the configuration editing section into `web/src/pages/node-detail/NodeConfigEditor.tsx`. This includes form fields for name, settings, ports, and the save handler.

- [ ] **Step 3: Create NodeLogsView component**

Extract the logs viewer into `web/src/pages/node-detail/NodeLogsView.tsx`. This includes the scrollable log list, auto-scroll behavior, and log entry rendering.

- [ ] **Step 4: Slim down NodeDetail.tsx**

Replace inline sections with imports. The page should handle node data fetching, tab navigation, and action buttons (~300 LOC).

- [ ] **Step 5: Run typecheck**

```bash
cd /home/neo/git/neo-nexus/web && npx tsc --noEmit
```

Expected: No errors.

- [ ] **Step 6: Commit**

```bash
git add web/src/pages/node-detail/ web/src/pages/NodeDetail.tsx
git commit -m "refactor(web): split NodeDetail.tsx into NodeConfigEditor and NodeLogsView"
```

---

## Task 13: Fix Remaining `any` Types

**Files:**
- Modify: `src/core/ConfigManager.ts` (lines 105, 138, 507)
- Modify: `src/server.ts` (line 369)

- [ ] **Step 1: Fix ConfigManager `any` casts**

In `src/core/ConfigManager.ts`:

Replace line 105:
```typescript
const baseApp = (baseConfig as any)?.ApplicationConfiguration || {};
```
with:
```typescript
const baseApp = (baseConfig as Record<string, unknown>)?.ApplicationConfiguration as Record<string, unknown> || {};
```

Replace line 138:
```typescript
ProtocolConfiguration: (baseConfig as any)?.ProtocolConfiguration || {
```
with:
```typescript
ProtocolConfiguration: (baseConfig as Record<string, unknown>)?.ProtocolConfiguration || {
```

Replace line 507:
```typescript
const onDiskForks = (onDisk as any)?.ProtocolConfiguration?.Hardforks;
```
with:
```typescript
const onDiskForks = (onDisk as Record<string, Record<string, unknown>>)?.ProtocolConfiguration?.Hardforks as Record<string, number> | undefined;
```

- [ ] **Step 2: Fix server.ts type**

In `src/server.ts` line 369, replace:
```typescript
const adminUsers = userManager.getAllUsers().filter((u: { role: string }) => u.role === "admin");
```
with proper typing (remove inline type annotation if the return type of `getAllUsers()` already includes `role`).

- [ ] **Step 3: Run typecheck and tests**

```bash
cd /home/neo/git/neo-nexus && npm run typecheck && npm test
```

Expected: All passing with no `any` warnings in modified files.

- [ ] **Step 4: Commit**

```bash
git add src/core/ConfigManager.ts src/server.ts
git commit -m "refactor: eliminate remaining 'any' type casts"
```

---

## Task 14: Final Verification

- [ ] **Step 1: Run full test suite**

```bash
cd /home/neo/git/neo-nexus && npm test
```

Expected: All 272+ tests pass.

- [ ] **Step 2: Run typecheck**

```bash
cd /home/neo/git/neo-nexus && npm run typecheck
```

Expected: No errors.

- [ ] **Step 3: Run lint**

```bash
cd /home/neo/git/neo-nexus && npm run lint
cd /home/neo/git/neo-nexus/web && npm run lint
```

Expected: Both pass (warnings acceptable).

- [ ] **Step 4: Run build**

```bash
cd /home/neo/git/neo-nexus && npm run build
```

Expected: Build succeeds.

- [ ] **Step 5: Verify LOC reduction**

```bash
wc -l src/core/NodeManager.ts src/server.ts web/src/pages/Settings.tsx web/src/pages/Plugins.tsx web/src/pages/NodeDetail.tsx
```

Expected targets:
- `NodeManager.ts`: ~500 LOC (from 1019)
- `Settings.tsx`: ~100 LOC (from 973)
- `Plugins.tsx`: ~200 LOC (from 545)
- `NodeDetail.tsx`: ~300 LOC (from 664)
