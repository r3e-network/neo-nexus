# Node Roles, Fast Sync, and Private Network Orchestration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build role-based node configuration, isolated data contexts, verified fast-sync snapshots, storage-engine selection, and private-network plan generation for NeoNexus.

**Architecture:** Add focused backend managers for role profiles, data contexts, fast-sync manifests, and private-network plans. Wire them through admin-only API routes, extend node config generation to honor storage/data-context choices, then add frontend surfaces for role application, snapshot registration, and private-network building.

**Tech Stack:** TypeScript, Express, better-sqlite3, React, TanStack Query, Vite, Vitest, Supertest.

---

## Scope Check

The approved spec spans four related subsystems: roles, data contexts, fast sync, and private-network planning. They share the same persistence and configuration boundary, so this plan keeps them together but splits them into independently testable tasks. Do not begin frontend work until backend routes and types are passing.

## Working Tree Preflight

The current workspace already has frontend QA changes in these files:

- `web/src/components/Layout.tsx`
- `web/src/components/ToggleSwitch.tsx`
- `web/src/config/constants.ts`
- `web/src/pages/Dashboard.tsx`
- `web/src/pages/Nodes.tsx`
- `web/src/pages/PublicDashboard.tsx`
- `web/src/pages/Settings.tsx`
- `web/src/pages/plugins/PluginCard.tsx`
- `web/tests/frontend-utils.test.ts`

Before implementing this feature, preserve those changes in their own commit or move feature work to a fresh worktree from `HEAD`.

Recommended preflight:

```bash
git status --short
git add web/src/components/Layout.tsx web/src/components/ToggleSwitch.tsx web/src/config/constants.ts web/src/pages/Dashboard.tsx web/src/pages/Nodes.tsx web/src/pages/PublicDashboard.tsx web/src/pages/Settings.tsx web/src/pages/plugins/PluginCard.tsx web/tests/frontend-utils.test.ts
git commit -m "fix(ui): polish responsive frontend qa"
git worktree add ../neo-nexus-role-orchestration HEAD
cd ../neo-nexus-role-orchestration
```

Expected result: the feature implementation starts from a clean worktree that includes the design commit and the frontend QA commit.

## File Structure

### Backend Types and Data

- Modify `src/types/index.ts`: add `StorageEngine`, `RoleSyncStrategy`, role profile, data context, snapshot, and private-network plan types. Extend `NodeSettings` with optional `storageEngine`, `activeDataContextId`, and `role`.
- Modify `src/types/database.ts`: add row types for the new SQLite tables.
- Modify `src/database/schema.ts`: create the four new tables and required indexes.
- Modify `src/utils/paths.ts`: add helper paths for node data contexts and fast-sync staging.

### Backend Managers

- Create `src/core/NodeRoleManager.ts`: role CRUD, built-in role definitions, role plan generation, and role application.
- Create `src/core/NodeDataContextManager.ts`: create/list/activate data contexts and resolve active context paths.
- Create `src/core/FastSyncManager.ts`: register manifests, verify SHA256, validate compatibility, and stage snapshot imports.
- Create `src/core/PrivateNetworkManager.ts`: generate and persist 1/4/7 private-network plans.
- Modify `src/core/NodeManager.ts`: expose manager dependencies, apply storage engine defaults during create/update, and delegate role application operations.
- Modify `src/core/ConfigManager.ts`: generate configs using effective storage engine and data context paths.
- Modify `src/core/PluginManager.ts`: add upsert helpers for idempotent role application.

### API

- Create `src/api/routes/nodeRoles.ts`.
- Create `src/api/routes/fastSync.ts`.
- Create `src/api/routes/privateNetworks.ts`.
- Modify `src/api/routes/nodes.ts`: add data-context endpoints.
- Modify `src/server.ts`: instantiate managers and mount routes behind auth/admin middleware.

### Frontend

- Modify `web/src/hooks/useNodes.ts`: add storage engine, data context, and role fields.
- Create `web/src/hooks/useNodeRoles.ts`.
- Create `web/src/hooks/useFastSync.ts`.
- Create `web/src/hooks/usePrivateNetworks.ts`.
- Modify `web/src/utils/nodePayloads.ts`: include storage engine and sync strategy.
- Modify `web/src/pages/CreateNode.tsx`: storage engine and sync strategy controls.
- Modify `web/src/pages/NodeDetail.tsx`: add role/data panel.
- Create `web/src/pages/Roles.tsx`.
- Create `web/src/pages/PrivateNetworkBuilder.tsx`.
- Modify `web/src/components/Layout.tsx` and `web/src/App.tsx`: navigation and routes.

---

### Task 0: Preserve Existing Frontend QA Work

**Files:**
- Commit existing modified frontend files listed in Working Tree Preflight.

- [ ] **Step 1: Confirm the only pre-existing dirty files are frontend QA files**

Run:

```bash
git status --short
```

Expected: only the nine `web/` files listed in Working Tree Preflight appear as modified.

- [ ] **Step 2: Run frontend verification for the existing QA changes**

Run:

```bash
npm --prefix web run lint
npm --prefix web run typecheck
npm --prefix web run test
npm --prefix web run build
```

Expected: every command exits with code 0.

- [ ] **Step 3: Commit the existing QA changes**

Run:

```bash
git add web/src/components/Layout.tsx web/src/components/ToggleSwitch.tsx web/src/config/constants.ts web/src/pages/Dashboard.tsx web/src/pages/Nodes.tsx web/src/pages/PublicDashboard.tsx web/src/pages/Settings.tsx web/src/pages/plugins/PluginCard.tsx web/tests/frontend-utils.test.ts
git commit -m "fix(ui): polish responsive frontend qa"
```

Expected: one commit containing only the frontend QA files.

- [ ] **Step 4: Create a feature worktree**

Run:

```bash
git worktree add ../neo-nexus-role-orchestration HEAD
cd ../neo-nexus-role-orchestration
git status --short
```

Expected: clean worktree.

### Task 1: Type and Schema Foundation

**Files:**
- Modify: `src/types/index.ts`
- Modify: `src/types/database.ts`
- Modify: `src/database/schema.ts`
- Modify: `src/utils/paths.ts`
- Test: `tests/unit/database.init.test.ts`
- Test: `tests/unit/paths.test.ts`

- [ ] **Step 1: Write failing schema tests**

Add this test to `tests/unit/database.init.test.ts`:

```ts
it("creates role, data context, fast sync, and private network tables", async () => {
  const db = await initializeDatabase();

  const tables = db.prepare("SELECT name FROM sqlite_master WHERE type = 'table'").all() as Array<{ name: string }>;
  const tableNames = new Set(tables.map((table) => table.name));

  expect(tableNames.has("node_role_profiles")).toBe(true);
  expect(tableNames.has("node_role_applications")).toBe(true);
  expect(tableNames.has("node_data_contexts")).toBe(true);
  expect(tableNames.has("fast_sync_snapshots")).toBe(true);
  expect(tableNames.has("private_network_plans")).toBe(true);
});
```

Add this test to `tests/unit/paths.test.ts`:

```ts
it("builds deterministic data context and fast sync staging paths", () => {
  expect(getNodeDataContextPath("node-1", "ctx-mainnet")).toContain("node-1/data-contexts/ctx-mainnet");
  expect(getFastSyncStagingPath("snapshot-1")).toContain("fast-sync/staging/snapshot-1");
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
npm run test:backend -- tests/unit/database.init.test.ts tests/unit/paths.test.ts
```

Expected: FAIL because tables and path helpers do not exist.

- [ ] **Step 3: Add shared types**

In `src/types/index.ts`, add:

```ts
export type StorageEngine = "leveldb" | "rocksdb";
export type RoleSyncStrategy = "full" | "light" | "fast-sync";
export type NodeRoleKind = "builtin" | "custom";
export type RoleApplicationStatus = "planned" | "applied" | "failed";
export type FastSyncSourceType = "local" | "url" | "catalog";
export type PrivateNetworkTemplate = "single" | "four" | "seven";
export type PrivateNetworkPlanStatus = "draft" | "applied" | "failed";

export interface NodeRolePluginDesiredState {
  id: PluginId;
  enabled: boolean;
  config?: Record<string, unknown>;
}

export interface NodeRoleProfileBody {
  storageEngine?: StorageEngine;
  settings?: Partial<NodeSettings>;
  plugins?: NodeRolePluginDesiredState[];
  dataContext?: {
    mode: "reuse-or-create" | "always-create";
    labelTemplate: string;
  };
  sync?: {
    strategy: RoleSyncStrategy;
    allowCheckpoint?: boolean;
  };
  warnings?: string[];
  prerequisites?: string[];
}

export interface NodeRoleProfile {
  id: string;
  name: string;
  description?: string;
  kind: NodeRoleKind;
  nodeTypes: NodeType[];
  profile: NodeRoleProfileBody;
  createdBy?: string;
  createdAt: number;
  updatedAt: number;
}

export interface NodeRoleApplication {
  id: string;
  nodeId: string;
  roleId: string;
  roleName: string;
  applicationPlan: NodeRoleApplicationPlan;
  previousState?: Record<string, unknown>;
  appliedAt: number;
  appliedBy?: string;
  status: RoleApplicationStatus;
  errorMessage?: string;
}

export interface NodeRoleApplicationPlan {
  nodeId: string;
  roleId: string;
  roleName: string;
  requiresRestart: boolean;
  changes: Array<{
    type: "settings" | "plugin" | "storage" | "data-context" | "fast-sync";
    summary: string;
  }>;
  warnings: string[];
}

export interface NodeDataContext {
  id: string;
  nodeId: string;
  label: string;
  storageEngine: StorageEngine;
  syncStrategy: RoleSyncStrategy;
  checkpointHeight?: number;
  checkpointHash?: string;
  snapshotId?: string;
  active: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface FastSyncSnapshot {
  id: string;
  name: string;
  sourceType: FastSyncSourceType;
  source: string;
  chain: NodeChain;
  network: NodeNetwork;
  nodeType: NodeType;
  storageEngine: StorageEngine;
  height: number;
  blockHash?: string;
  sha256: string;
  sizeBytes?: number;
  signature?: string;
  trusted: boolean;
  createdAt: number;
  lastVerifiedAt?: number;
}

export interface PrivateNetworkPlan {
  id: string;
  name: string;
  template: PrivateNetworkTemplate;
  networkMagic: number;
  plan: {
    nodes: Array<{
      name: string;
      type: N3NodeType;
      roleIds: string[];
      storageEngine: StorageEngine;
      ports: Partial<PortConfig>;
      publicKey: string;
      address: string;
    }>;
    seedList: string[];
    validatorsCount: number;
    standbyCommittee: string[];
  };
  status: PrivateNetworkPlanStatus;
  createdAt: number;
  appliedAt?: number;
}
```

Extend `NodeSettings`:

```ts
storageEngine?: StorageEngine;
activeDataContextId?: string;
role?: {
  id: string;
  name: string;
  appliedAt: number;
};
```

- [ ] **Step 4: Add database row types**

In `src/types/database.ts`, add row interfaces matching the schema:

```ts
export interface NodeRoleProfileRow {
  id: string;
  name: string;
  description: string | null;
  kind: string;
  node_types: string;
  profile: string;
  created_by: string | null;
  created_at: number;
  updated_at: number;
}

export interface NodeRoleApplicationRow {
  id: string;
  node_id: string;
  role_id: string;
  role_name: string;
  application_plan: string;
  previous_state: string | null;
  applied_at: number;
  applied_by: string | null;
  status: string;
  error_message: string | null;
}

export interface NodeDataContextRow {
  id: string;
  node_id: string;
  label: string;
  storage_engine: string;
  sync_strategy: string;
  checkpoint_height: number | null;
  checkpoint_hash: string | null;
  snapshot_id: string | null;
  active: number;
  created_at: number;
  updated_at: number;
}

export interface FastSyncSnapshotRow {
  id: string;
  name: string;
  source_type: string;
  source: string;
  chain: string;
  network: string;
  node_type: string;
  storage_engine: string;
  height: number;
  block_hash: string | null;
  sha256: string;
  size_bytes: number | null;
  signature: string | null;
  trusted: number;
  created_at: number;
  last_verified_at: number | null;
}

export interface PrivateNetworkPlanRow {
  id: string;
  name: string;
  template: string;
  network_magic: number;
  plan: string;
  status: string;
  created_at: number;
  applied_at: number | null;
}
```

- [ ] **Step 5: Add schema tables and indexes**

In `src/database/schema.ts`, add these `CREATE TABLE IF NOT EXISTS` statements inside the main `db.exec` block:

```sql
CREATE TABLE IF NOT EXISTS node_role_profiles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT,
  kind TEXT NOT NULL CHECK (kind IN ('builtin', 'custom')),
  node_types TEXT NOT NULL,
  profile TEXT NOT NULL,
  created_by TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS node_role_applications (
  id TEXT PRIMARY KEY,
  node_id TEXT NOT NULL,
  role_id TEXT NOT NULL,
  role_name TEXT NOT NULL,
  application_plan TEXT NOT NULL,
  previous_state TEXT,
  applied_at INTEGER NOT NULL,
  applied_by TEXT,
  status TEXT NOT NULL CHECK (status IN ('planned', 'applied', 'failed')),
  error_message TEXT,
  FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_node_role_applications_node ON node_role_applications(node_id, applied_at);

CREATE TABLE IF NOT EXISTS node_data_contexts (
  id TEXT PRIMARY KEY,
  node_id TEXT NOT NULL,
  label TEXT NOT NULL,
  storage_engine TEXT NOT NULL CHECK (storage_engine IN ('leveldb', 'rocksdb')),
  sync_strategy TEXT NOT NULL CHECK (sync_strategy IN ('full', 'light', 'fast-sync')),
  checkpoint_height INTEGER,
  checkpoint_hash TEXT,
  snapshot_id TEXT,
  active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0, 1)),
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL,
  FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_node_data_contexts_node ON node_data_contexts(node_id, active);

CREATE TABLE IF NOT EXISTS fast_sync_snapshots (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  source_type TEXT NOT NULL CHECK (source_type IN ('local', 'url', 'catalog')),
  source TEXT NOT NULL,
  chain TEXT NOT NULL,
  network TEXT NOT NULL,
  node_type TEXT NOT NULL,
  storage_engine TEXT NOT NULL CHECK (storage_engine IN ('leveldb', 'rocksdb')),
  height INTEGER NOT NULL,
  block_hash TEXT,
  sha256 TEXT NOT NULL,
  size_bytes INTEGER,
  signature TEXT,
  trusted INTEGER NOT NULL DEFAULT 0 CHECK (trusted IN (0, 1)),
  created_at INTEGER NOT NULL,
  last_verified_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_fast_sync_snapshots_network ON fast_sync_snapshots(chain, network, node_type, storage_engine);

CREATE TABLE IF NOT EXISTS private_network_plans (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  template TEXT NOT NULL CHECK (template IN ('single', 'four', 'seven')),
  network_magic INTEGER NOT NULL,
  plan TEXT NOT NULL,
  status TEXT NOT NULL CHECK (status IN ('draft', 'applied', 'failed')),
  created_at INTEGER NOT NULL,
  applied_at INTEGER
);
```

- [ ] **Step 6: Add path helpers**

In `src/utils/paths.ts`, add:

```ts
export function getNodeDataContextsRoot(nodeId: string): string {
  return join(getNodePath(nodeId), "data-contexts");
}

export function getNodeDataContextPath(nodeId: string, contextId: string): string {
  return join(getNodeDataContextsRoot(nodeId), contextId);
}

export function getFastSyncRoot(): string {
  return join(paths.base, "fast-sync");
}

export function getFastSyncStagingPath(snapshotId: string): string {
  return join(getFastSyncRoot(), "staging", snapshotId);
}
```

- [ ] **Step 7: Run tests to verify pass**

Run:

```bash
npm run test:backend -- tests/unit/database.init.test.ts tests/unit/paths.test.ts
```

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add src/types/index.ts src/types/database.ts src/database/schema.ts src/utils/paths.ts tests/unit/database.init.test.ts tests/unit/paths.test.ts
git commit -m "feat: add role orchestration schema"
```

### Task 2: Built-In Roles and Planning

**Files:**
- Create: `src/core/NodeRoleManager.ts`
- Modify: `src/core/index.ts`
- Test: `tests/unit/NodeRoleManager.test.ts`

- [ ] **Step 1: Write failing built-in role tests**

Create `tests/unit/NodeRoleManager.test.ts`:

```ts
import Database from "better-sqlite3";
import { describe, expect, it } from "vitest";
import { NodeRoleManager } from "../../src/core/NodeRoleManager";

function createManager() {
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE node_role_profiles (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT,
      kind TEXT NOT NULL,
      node_types TEXT NOT NULL,
      profile TEXT NOT NULL,
      created_by TEXT,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );
    CREATE TABLE node_role_applications (
      id TEXT PRIMARY KEY,
      node_id TEXT NOT NULL,
      role_id TEXT NOT NULL,
      role_name TEXT NOT NULL,
      application_plan TEXT NOT NULL,
      previous_state TEXT,
      applied_at INTEGER NOT NULL,
      applied_by TEXT,
      status TEXT NOT NULL,
      error_message TEXT
    );
  `);
  return new NodeRoleManager(db);
}

describe("NodeRoleManager", () => {
  it("lists built-in roles for common Neo node identities", () => {
    const manager = createManager();
    const roles = manager.listRoles();
    const ids = roles.map((role) => role.id);

    expect(ids).toContain("builtin-rpc-api");
    expect(ids).toContain("builtin-state");
    expect(ids).toContain("builtin-oracle");
    expect(ids).toContain("builtin-consensus");
    expect(ids).toContain("builtin-indexer");
    expect(ids).toContain("builtin-secure-signer-client");
  });

  it("plans plugin, storage, and data context changes for a state node role", () => {
    const manager = createManager();
    const plan = manager.planRoleApplication({
      roleId: "builtin-state",
      node: {
        id: "node-1",
        name: "State Node",
        chain: "n3",
        type: "neo-cli",
        network: "mainnet",
        syncMode: "full",
        version: "3.9.2",
        ports: { rpc: 10332, p2p: 10333 },
        paths: { base: "/tmp/node-1", data: "/tmp/node-1/data", logs: "/tmp/node-1/logs", config: "/tmp/node-1/config" },
        settings: {},
        createdAt: 1,
        updatedAt: 1,
        process: { status: "stopped" },
        plugins: [],
      },
    });

    expect(plan.requiresRestart).toBe(true);
    expect(plan.changes.map((change) => change.type)).toEqual(expect.arrayContaining(["plugin", "storage", "data-context"]));
    expect(plan.changes.map((change) => change.summary).join(" ")).toContain("StateService");
    expect(plan.changes.map((change) => change.summary).join(" ")).toContain("rocksdb");
  });

  it("rejects applying neo-cli plugin roles to neo-go nodes", () => {
    const manager = createManager();

    expect(() => manager.planRoleApplication({
      roleId: "builtin-consensus",
      node: {
        id: "node-1",
        name: "Go Node",
        chain: "n3",
        type: "neo-go",
        network: "mainnet",
        syncMode: "full",
        version: "0.118.0",
        ports: { rpc: 20332, p2p: 20333 },
        paths: { base: "/tmp/node-1", data: "/tmp/node-1/data", logs: "/tmp/node-1/logs", config: "/tmp/node-1/config" },
        settings: {},
        createdAt: 1,
        updatedAt: 1,
        process: { status: "stopped" },
        plugins: [],
      },
    })).toThrow(/not compatible/i);
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
npm run test:backend -- tests/unit/NodeRoleManager.test.ts
```

Expected: FAIL because `NodeRoleManager` does not exist.

- [ ] **Step 3: Implement manager with built-in roles**

Create `src/core/NodeRoleManager.ts`:

```ts
import crypto from "node:crypto";
import type Database from "better-sqlite3";
import type {
  NodeInstance,
  NodeRoleApplication,
  NodeRoleApplicationPlan,
  NodeRoleProfile,
  NodeRoleProfileBody,
  NodeRolePluginDesiredState,
  NodeType,
  StorageEngine,
} from "../types";
import type { NodeRoleApplicationRow, NodeRoleProfileRow } from "../types/database";

export interface RolePlanInput {
  roleId: string;
  node: NodeInstance;
  storageEngineOverride?: StorageEngine;
}

export interface CreateCustomRoleInput {
  name: string;
  description?: string;
  nodeTypes: NodeType[];
  profile: NodeRoleProfileBody;
  createdBy?: string;
}

const now = () => Date.now();

const BUILTIN_ROLES: NodeRoleProfile[] = [
  {
    id: "builtin-rpc-api",
    name: "RPC / API Node",
    description: "Expose JSON-RPC and optional REST APIs for wallets, dApps, and monitoring.",
    kind: "builtin",
    nodeTypes: ["neo-cli"],
    profile: {
      storageEngine: "leveldb",
      settings: { relay: true, maxConnections: 80, minPeers: 10, maxPeers: 60 },
      plugins: [{ id: "RpcServer", enabled: true, config: {} }],
      dataContext: { mode: "reuse-or-create", labelTemplate: "rpc-{network}-{storageEngine}" },
      sync: { strategy: "fast-sync", allowCheckpoint: true },
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: "builtin-state",
    name: "State Node",
    description: "Track state roots and serve state proof workflows.",
    kind: "builtin",
    nodeTypes: ["neo-cli"],
    profile: {
      storageEngine: "rocksdb",
      settings: { relay: true, maxConnections: 80, minPeers: 10, maxPeers: 60 },
      plugins: [
        { id: "StateService", enabled: true, config: { AutoStart: true, FullState: false } },
        { id: "RpcServer", enabled: true, config: {} },
      ],
      dataContext: { mode: "reuse-or-create", labelTemplate: "state-{network}-{storageEngine}" },
      sync: { strategy: "fast-sync", allowCheckpoint: true },
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: "builtin-oracle",
    name: "Oracle Node",
    description: "Run OracleService for off-chain data requests.",
    kind: "builtin",
    nodeTypes: ["neo-cli"],
    profile: {
      storageEngine: "leveldb",
      settings: { relay: true, minPeers: 10, maxPeers: 60 },
      plugins: [
        { id: "OracleService", enabled: true, config: { AutoStart: true } },
        { id: "RpcServer", enabled: true, config: {} },
      ],
      dataContext: { mode: "reuse-or-create", labelTemplate: "oracle-{network}-{storageEngine}" },
      sync: { strategy: "fast-sync", allowCheckpoint: true },
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: "builtin-consensus",
    name: "Consensus Node",
    description: "Configure dBFT consensus participation for validator nodes.",
    kind: "builtin",
    nodeTypes: ["neo-cli"],
    profile: {
      storageEngine: "leveldb",
      settings: { relay: true, minPeers: 10, maxPeers: 80 },
      plugins: [{ id: "DBFTPlugin", enabled: true, config: { AutoStart: false } }],
      dataContext: { mode: "reuse-or-create", labelTemplate: "consensus-{network}-{storageEngine}" },
      sync: { strategy: "full", allowCheckpoint: false },
      warnings: ["Consensus nodes require validator key material or a secure signing workflow."],
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: "builtin-indexer",
    name: "Indexer Node",
    description: "Index application logs and token transfers.",
    kind: "builtin",
    nodeTypes: ["neo-cli"],
    profile: {
      storageEngine: "rocksdb",
      settings: { relay: true, maxConnections: 100, minPeers: 10, maxPeers: 80 },
      plugins: [
        { id: "ApplicationLogs", enabled: true, config: {} },
        { id: "TokensTracker", enabled: true, config: {} },
        { id: "RpcServer", enabled: true, config: {} },
      ],
      dataContext: { mode: "reuse-or-create", labelTemplate: "indexer-{network}-{storageEngine}" },
      sync: { strategy: "fast-sync", allowCheckpoint: true },
    },
    createdAt: 0,
    updatedAt: 0,
  },
  {
    id: "builtin-secure-signer-client",
    name: "Secure Signer Client",
    description: "Use SignClient for external signing.",
    kind: "builtin",
    nodeTypes: ["neo-cli"],
    profile: {
      storageEngine: "leveldb",
      settings: { keyProtection: { mode: "secure-signer" } },
      plugins: [{ id: "SignClient", enabled: true, config: {} }],
      dataContext: { mode: "reuse-or-create", labelTemplate: "signer-client-{network}-{storageEngine}" },
      sync: { strategy: "full", allowCheckpoint: false },
      prerequisites: ["Select a secure signer profile before applying this role."],
    },
    createdAt: 0,
    updatedAt: 0,
  },
];

export class NodeRoleManager {
  constructor(private readonly db: Database.Database) {}

  listRoles(): NodeRoleProfile[] {
    const customRows = this.db.prepare("SELECT * FROM node_role_profiles ORDER BY name").all() as NodeRoleProfileRow[];
    return [...BUILTIN_ROLES, ...customRows.map((row) => this.mapRoleRow(row))];
  }

  getRole(roleId: string): NodeRoleProfile | null {
    const builtin = BUILTIN_ROLES.find((role) => role.id === roleId);
    if (builtin) return builtin;
    const row = this.db.prepare("SELECT * FROM node_role_profiles WHERE id = ?").get(roleId) as NodeRoleProfileRow | undefined;
    return row ? this.mapRoleRow(row) : null;
  }

  createCustomRole(input: CreateCustomRoleInput): NodeRoleProfile {
    const role: NodeRoleProfile = {
      id: `role-${crypto.randomUUID()}`,
      name: input.name,
      description: input.description,
      kind: "custom",
      nodeTypes: input.nodeTypes,
      profile: input.profile,
      createdBy: input.createdBy,
      createdAt: now(),
      updatedAt: now(),
    };
    this.db.prepare(`
      INSERT INTO node_role_profiles (id, name, description, kind, node_types, profile, created_by, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(role.id, role.name, role.description ?? null, role.kind, JSON.stringify(role.nodeTypes), JSON.stringify(role.profile), role.createdBy ?? null, role.createdAt, role.updatedAt);
    return role;
  }

  planRoleApplication(input: RolePlanInput): NodeRoleApplicationPlan {
    const role = this.getRole(input.roleId);
    if (!role) throw new Error(`Role ${input.roleId} not found`);
    if (!role.nodeTypes.includes(input.node.type)) {
      throw new Error(`Role ${role.name} is not compatible with ${input.node.type}`);
    }
    const storageEngine = input.storageEngineOverride ?? role.profile.storageEngine ?? input.node.settings.storageEngine ?? "leveldb";
    const plugins = role.profile.plugins ?? [];
    const warnings = [...(role.profile.warnings ?? []), ...(role.profile.prerequisites ?? [])];
    const changes: NodeRoleApplicationPlan["changes"] = [];

    if (input.node.settings.storageEngine !== storageEngine) {
      changes.push({ type: "storage", summary: `Switch storage engine to ${storageEngine}` });
    }
    if (role.profile.dataContext) {
      changes.push({ type: "data-context", summary: `Use data context ${this.renderLabel(role.profile.dataContext.labelTemplate, input.node, storageEngine)}` });
    }
    for (const plugin of plugins) {
      changes.push({ type: "plugin", summary: `${plugin.enabled ? "Enable" : "Disable"} ${plugin.id}` });
    }
    if (role.profile.settings && Object.keys(role.profile.settings).length > 0) {
      changes.push({ type: "settings", summary: "Apply node settings from role" });
    }
    if (role.profile.sync?.strategy === "fast-sync") {
      changes.push({ type: "fast-sync", summary: "Allow verified fast-sync snapshot for this role" });
    }

    return {
      nodeId: input.node.id,
      roleId: role.id,
      roleName: role.name,
      requiresRestart: true,
      changes,
      warnings,
    };
  }

  recordApplication(application: Omit<NodeRoleApplication, "id" | "appliedAt">): NodeRoleApplication {
    const record: NodeRoleApplication = {
      ...application,
      id: `role-app-${crypto.randomUUID()}`,
      appliedAt: now(),
    };
    this.db.prepare(`
      INSERT INTO node_role_applications (id, node_id, role_id, role_name, application_plan, previous_state, applied_at, applied_by, status, error_message)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(
      record.id,
      record.nodeId,
      record.roleId,
      record.roleName,
      JSON.stringify(record.applicationPlan),
      record.previousState ? JSON.stringify(record.previousState) : null,
      record.appliedAt,
      record.appliedBy ?? null,
      record.status,
      record.errorMessage ?? null,
    );
    return record;
  }

  listApplications(nodeId: string): NodeRoleApplication[] {
    const rows = this.db.prepare("SELECT * FROM node_role_applications WHERE node_id = ? ORDER BY applied_at DESC").all(nodeId) as NodeRoleApplicationRow[];
    return rows.map((row) => ({
      id: row.id,
      nodeId: row.node_id,
      roleId: row.role_id,
      roleName: row.role_name,
      applicationPlan: JSON.parse(row.application_plan) as NodeRoleApplicationPlan,
      previousState: row.previous_state ? JSON.parse(row.previous_state) as Record<string, unknown> : undefined,
      appliedAt: row.applied_at,
      appliedBy: row.applied_by ?? undefined,
      status: row.status as NodeRoleApplication["status"],
      errorMessage: row.error_message ?? undefined,
    }));
  }

  private mapRoleRow(row: NodeRoleProfileRow): NodeRoleProfile {
    return {
      id: row.id,
      name: row.name,
      description: row.description ?? undefined,
      kind: row.kind as NodeRoleProfile["kind"],
      nodeTypes: JSON.parse(row.node_types) as NodeType[],
      profile: JSON.parse(row.profile) as NodeRoleProfileBody,
      createdBy: row.created_by ?? undefined,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }

  private renderLabel(template: string, node: NodeInstance, storageEngine: StorageEngine): string {
    return template
      .replaceAll("{network}", node.network)
      .replaceAll("{storageEngine}", storageEngine)
      .replaceAll("{role}", "role");
  }
}

export { BUILTIN_ROLES };
```

- [ ] **Step 4: Export manager**

In `src/core/index.ts`, add:

```ts
export { NodeRoleManager } from "./NodeRoleManager";
```

- [ ] **Step 5: Run tests**

Run:

```bash
npm run test:backend -- tests/unit/NodeRoleManager.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add src/core/NodeRoleManager.ts src/core/index.ts tests/unit/NodeRoleManager.test.ts
git commit -m "feat: add node role planning"
```

### Task 3: Data Context Manager and Config Path Resolution

**Files:**
- Create: `src/core/NodeDataContextManager.ts`
- Modify: `src/core/ConfigManager.ts`
- Modify: `src/core/StorageManager.ts`
- Test: `tests/unit/NodeDataContextManager.test.ts`
- Test: `tests/unit/neo-go-config.test.ts`
- Test: `tests/unit/secure-signer-config.test.ts`

- [ ] **Step 1: Write failing data context tests**

Create `tests/unit/NodeDataContextManager.test.ts`:

```ts
import Database from "better-sqlite3";
import { describe, expect, it } from "vitest";
import { NodeDataContextManager } from "../../src/core/NodeDataContextManager";

function createManager() {
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE node_data_contexts (
      id TEXT PRIMARY KEY,
      node_id TEXT NOT NULL,
      label TEXT NOT NULL,
      storage_engine TEXT NOT NULL,
      sync_strategy TEXT NOT NULL,
      checkpoint_height INTEGER,
      checkpoint_hash TEXT,
      snapshot_id TEXT,
      active INTEGER NOT NULL DEFAULT 0,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL
    );
  `);
  return new NodeDataContextManager(db);
}

describe("NodeDataContextManager", () => {
  it("creates a first context as active", () => {
    const manager = createManager();
    const context = manager.createContext("node-1", {
      label: "state-mainnet-rocksdb",
      storageEngine: "rocksdb",
      syncStrategy: "fast-sync",
      checkpointHeight: 5800000,
      checkpointHash: "0xabc",
    });

    expect(context.active).toBe(true);
    expect(manager.listContexts("node-1")).toHaveLength(1);
  });

  it("activates one context and deactivates the previous active context", () => {
    const manager = createManager();
    const first = manager.createContext("node-1", { label: "default", storageEngine: "leveldb", syncStrategy: "full" });
    const second = manager.createContext("node-1", { label: "state", storageEngine: "rocksdb", syncStrategy: "fast-sync" });

    manager.activateContext("node-1", second.id);

    expect(manager.getActiveContext("node-1")?.id).toBe(second.id);
    expect(manager.listContexts("node-1").find((ctx) => ctx.id === first.id)?.active).toBe(false);
  });
});
```

Add a config test to `tests/unit/neo-go-config.test.ts`:

```ts
it("writes neo-go database path for the active data context", () => {
  const config = ConfigManager.generateNeoGoConfig({
    id: "node-1",
    name: "Neo Go",
    chain: "n3",
    type: "neo-go",
    network: "private",
    syncMode: "full",
    version: "0.118.0",
    ports: { rpc: 20332, p2p: 20333 },
    paths: { base: "/tmp/node-1", data: "/tmp/node-1/data", logs: "/tmp/node-1/logs", config: "/tmp/node-1/config" },
    settings: { activeDataContextId: "ctx-state", storageEngine: "rocksdb" },
    createdAt: 1,
    updatedAt: 1,
  });

  expect((config.ApplicationConfiguration as any).DBConfiguration.Type).toBe("rocksdb");
  expect((config.ApplicationConfiguration as any).DBConfiguration.LevelDBOptions.DataDirectoryPath).toBe("./data-contexts/ctx-state");
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
npm run test:backend -- tests/unit/NodeDataContextManager.test.ts tests/unit/neo-go-config.test.ts
```

Expected: FAIL because manager and config behavior do not exist.

- [ ] **Step 3: Implement NodeDataContextManager**

Create `src/core/NodeDataContextManager.ts`:

```ts
import crypto from "node:crypto";
import type Database from "better-sqlite3";
import type { NodeDataContext, RoleSyncStrategy, StorageEngine } from "../types";
import type { NodeDataContextRow } from "../types/database";

export interface CreateNodeDataContextInput {
  label: string;
  storageEngine: StorageEngine;
  syncStrategy: RoleSyncStrategy;
  checkpointHeight?: number;
  checkpointHash?: string;
  snapshotId?: string;
}

export class NodeDataContextManager {
  constructor(private readonly db: Database.Database) {}

  listContexts(nodeId: string): NodeDataContext[] {
    const rows = this.db.prepare("SELECT * FROM node_data_contexts WHERE node_id = ? ORDER BY created_at").all(nodeId) as NodeDataContextRow[];
    return rows.map((row) => this.mapRow(row));
  }

  getActiveContext(nodeId: string): NodeDataContext | null {
    const row = this.db.prepare("SELECT * FROM node_data_contexts WHERE node_id = ? AND active = 1").get(nodeId) as NodeDataContextRow | undefined;
    return row ? this.mapRow(row) : null;
  }

  createContext(nodeId: string, input: CreateNodeDataContextInput): NodeDataContext {
    const hasExisting = this.listContexts(nodeId).length > 0;
    const context: NodeDataContext = {
      id: `ctx-${crypto.randomUUID()}`,
      nodeId,
      label: input.label,
      storageEngine: input.storageEngine,
      syncStrategy: input.syncStrategy,
      checkpointHeight: input.checkpointHeight,
      checkpointHash: input.checkpointHash,
      snapshotId: input.snapshotId,
      active: !hasExisting,
      createdAt: Date.now(),
      updatedAt: Date.now(),
    };
    this.db.prepare(`
      INSERT INTO node_data_contexts (id, node_id, label, storage_engine, sync_strategy, checkpoint_height, checkpoint_hash, snapshot_id, active, created_at, updated_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(context.id, nodeId, context.label, context.storageEngine, context.syncStrategy, context.checkpointHeight ?? null, context.checkpointHash ?? null, context.snapshotId ?? null, context.active ? 1 : 0, context.createdAt, context.updatedAt);
    return context;
  }

  activateContext(nodeId: string, contextId: string): NodeDataContext {
    const context = this.listContexts(nodeId).find((candidate) => candidate.id === contextId);
    if (!context) throw new Error(`Data context ${contextId} not found for node ${nodeId}`);
    const tx = this.db.transaction(() => {
      this.db.prepare("UPDATE node_data_contexts SET active = 0, updated_at = ? WHERE node_id = ?").run(Date.now(), nodeId);
      this.db.prepare("UPDATE node_data_contexts SET active = 1, updated_at = ? WHERE node_id = ? AND id = ?").run(Date.now(), nodeId, contextId);
    });
    tx();
    return this.getActiveContext(nodeId)!;
  }

  private mapRow(row: NodeDataContextRow): NodeDataContext {
    return {
      id: row.id,
      nodeId: row.node_id,
      label: row.label,
      storageEngine: row.storage_engine as StorageEngine,
      syncStrategy: row.sync_strategy as RoleSyncStrategy,
      checkpointHeight: row.checkpoint_height ?? undefined,
      checkpointHash: row.checkpoint_hash ?? undefined,
      snapshotId: row.snapshot_id ?? undefined,
      active: row.active === 1,
      createdAt: row.created_at,
      updatedAt: row.updated_at,
    };
  }
}
```

- [ ] **Step 4: Update ConfigManager for storage engine/context**

In `src/core/ConfigManager.ts`, add helper:

```ts
private static getDataContextRelativePath(node: NodeConfig): string {
  return node.settings.activeDataContextId
    ? `data-contexts/${node.settings.activeDataContextId}`
    : "data";
}

private static getStorageEngine(node: NodeConfig): StorageEngine {
  return node.settings.storageEngine ?? "leveldb";
}
```

Update neo-cli storage generation:

```ts
Storage: {
  Engine: this.getStorageEngine(node) === "rocksdb" || installedPlugins.includes("RocksDBStore") ? "RocksDBStore" : "LevelDBStore",
  Path: this.getDataContextRelativePath(node),
},
```

Update neo-go `DBConfiguration`:

```ts
DBConfiguration: {
  Type: this.getStorageEngine(node),
  LevelDBOptions: {
    DataDirectoryPath: `./${this.getDataContextRelativePath(node)}`,
  },
},
```

- [ ] **Step 5: Run tests**

Run:

```bash
npm run test:backend -- tests/unit/NodeDataContextManager.test.ts tests/unit/neo-go-config.test.ts tests/unit/secure-signer-config.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add src/core/NodeDataContextManager.ts src/core/ConfigManager.ts src/core/StorageManager.ts tests/unit/NodeDataContextManager.test.ts tests/unit/neo-go-config.test.ts tests/unit/secure-signer-config.test.ts
git commit -m "feat: add node data contexts"
```

### Task 4: Storage Engine Selection in Node Create and Plugin Flow

**Files:**
- Modify: `src/core/NodeManager.ts`
- Modify: `src/core/PluginManager.ts`
- Modify: `src/api/routes/nodes.ts`
- Modify: `web/src/utils/nodePayloads.ts`
- Test: `tests/unit/NodeManager.plugin-mutations.test.ts`
- Test: `tests/integration/nodes.router.actual.test.ts`
- Test: `tests/unit/web.nodePayloads.test.ts`

- [ ] **Step 1: Write failing storage engine tests**

Add to `tests/unit/NodeManager.plugin-mutations.test.ts`:

```ts
it("upserts RocksDBStore when neo-cli storage engine is rocksdb", async () => {
  const node = {
    id: "node-1",
    type: "neo-cli",
    process: { status: "stopped" },
    settings: { storageEngine: "rocksdb" },
  };
  const manager = Object.create(NodeManager.prototype) as NodeManager & {
    getNode: ReturnType<typeof vi.fn>;
    pluginManager: {
      upsertPlugin: ReturnType<typeof vi.fn>;
      getInstalledPlugins: ReturnType<typeof vi.fn>;
    };
  };

  manager.getNode = vi.fn(() => node);
  manager.pluginManager = {
    upsertPlugin: vi.fn(),
    getInstalledPlugins: vi.fn(() => [{ id: "RocksDBStore", enabled: true }]),
  };
  const writeNodeConfig = vi.spyOn(ConfigManager, "writeNodeConfig").mockResolvedValue(undefined);

  await manager.ensureStorageEngine("node-1", "rocksdb");

  expect(manager.pluginManager.upsertPlugin).toHaveBeenCalledWith("node-1", "RocksDBStore", expect.any(String), {});
  expect(writeNodeConfig).toHaveBeenCalledWith(node, ["RocksDBStore"]);
});
```

Add to `tests/unit/web.nodePayloads.test.ts`:

```ts
it("includes storage engine and sync strategy in create payload", () => {
  expect(normalizeNodeUpsertPayload({
    name: "State node",
    type: "neo-cli",
    network: "mainnet",
    syncMode: "full",
    storageEngine: "rocksdb",
    syncStrategy: "fast-sync",
    maxConnections: "",
    minPeers: "",
    maxPeers: "",
    relay: true,
    debugMode: false,
    customConfig: "",
    keyProtectionMode: "standard",
    secureSignerProfileId: "",
  })).toMatchObject({
    settings: {
      storageEngine: "rocksdb",
      syncStrategy: "fast-sync",
    },
  });
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
npm run test:backend -- tests/unit/NodeManager.plugin-mutations.test.ts tests/unit/web.nodePayloads.test.ts
```

Expected: FAIL because `ensureStorageEngine`, `upsertPlugin`, and payload fields do not exist.

- [ ] **Step 3: Add PluginManager upsert helper**

In `src/core/PluginManager.ts`, add:

```ts
async upsertPlugin(nodeId: string, pluginId: PluginId, nodeVersion: string, config: Record<string, unknown> = {}): Promise<void> {
  const existing = this.getInstalledPlugins(nodeId).find((plugin) => plugin.id === pluginId);
  if (!existing) {
    await this.installPlugin(nodeId, pluginId, nodeVersion, config);
    return;
  }
  this.updatePluginConfig(nodeId, pluginId, { ...existing.config, ...config });
  this.setPluginEnabled(nodeId, pluginId, true);
}
```

- [ ] **Step 4: Add NodeManager storage helper**

In `src/core/NodeManager.ts`, add public method:

```ts
async ensureStorageEngine(nodeId: string, storageEngine: StorageEngine): Promise<NodeInstance> {
  const node = this.getNode(nodeId);
  if (!node) throw Errors.nodeNotFound(nodeId);
  if (node.process.status === "running") throw Errors.nodeRunning();
  this.assertCanWriteImportedNode(node, "storage engine updates");

  const nextSettings: NodeSettings = { ...(node.settings || {}), storageEngine };
  this.repo.updateNode(nodeId, ["settings = ?", "updated_at = ?"], [JSON.stringify(nextSettings), Date.now(), nodeId]);

  if (node.type === "neo-cli" && storageEngine === "rocksdb") {
    await this.pluginManager.upsertPlugin(nodeId, "RocksDBStore", node.version, {});
  }
  if (node.type === "neo-cli" && storageEngine === "leveldb") {
    const installed = this.pluginManager.getInstalledPlugins(nodeId);
    if (installed.some((plugin) => plugin.id === "RocksDBStore")) {
      this.pluginManager.setPluginEnabled(nodeId, "RocksDBStore", false);
    }
  }

  const updatedNode = this.getNode(nodeId)!;
  await ConfigManager.writeNodeConfig(updatedNode, this.getEnabledPluginIds(nodeId));
  return updatedNode;
}
```

Import `StorageEngine` from `../types/index`.

- [ ] **Step 5: Include storage in create/update request handling**

In `src/types/index.ts`, extend `NodeSettings` with:

```ts
syncStrategy?: RoleSyncStrategy;
```

In `NodeManager.createNode`, after `settings: stripReservedImportSettings(request.settings) || {}`, ensure default:

```ts
settings: {
  storageEngine: request.settings?.storageEngine ?? "leveldb",
  syncStrategy: request.settings?.syncStrategy ?? request.syncMode ?? "full",
  ...(stripReservedImportSettings(request.settings) || {}),
},
```

- [ ] **Step 6: Update frontend payload types**

In `web/src/utils/nodePayloads.ts`, add form fields:

```ts
storageEngine: "leveldb" | "rocksdb";
syncStrategy: "full" | "light" | "fast-sync";
```

Ensure `normalizeNodeUpsertPayload` writes:

```ts
settings: {
  ...settings,
  storageEngine: values.storageEngine,
  syncStrategy: values.syncStrategy,
}
```

- [ ] **Step 7: Run tests**

Run:

```bash
npm run test:backend -- tests/unit/NodeManager.plugin-mutations.test.ts tests/integration/nodes.router.actual.test.ts tests/unit/web.nodePayloads.test.ts
npm --prefix web run typecheck
```

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add src/core/NodeManager.ts src/core/PluginManager.ts src/api/routes/nodes.ts src/types/index.ts web/src/utils/nodePayloads.ts tests/unit/NodeManager.plugin-mutations.test.ts tests/integration/nodes.router.actual.test.ts tests/unit/web.nodePayloads.test.ts
git commit -m "feat: support storage engine selection"
```

### Task 5: Fast Sync Snapshot Manifest Validation

**Files:**
- Create: `src/core/FastSyncManager.ts`
- Create: `src/api/routes/fastSync.ts`
- Modify: `src/server.ts`
- Test: `tests/unit/FastSyncManager.test.ts`
- Test: `tests/integration/fast-sync.router.actual.test.ts`

- [ ] **Step 1: Write failing FastSyncManager tests**

Create `tests/unit/FastSyncManager.test.ts`:

```ts
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import crypto from "node:crypto";
import Database from "better-sqlite3";
import { describe, expect, it } from "vitest";
import { FastSyncManager } from "../../src/core/FastSyncManager";

function createManager() {
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE fast_sync_snapshots (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      source_type TEXT NOT NULL,
      source TEXT NOT NULL,
      chain TEXT NOT NULL,
      network TEXT NOT NULL,
      node_type TEXT NOT NULL,
      storage_engine TEXT NOT NULL,
      height INTEGER NOT NULL,
      block_hash TEXT,
      sha256 TEXT NOT NULL,
      size_bytes INTEGER,
      signature TEXT,
      trusted INTEGER NOT NULL DEFAULT 0,
      created_at INTEGER NOT NULL,
      last_verified_at INTEGER
    );
  `);
  return new FastSyncManager(db);
}

describe("FastSyncManager", () => {
  it("requires sha256 when registering snapshots", () => {
    const manager = createManager();

    expect(() => manager.registerSnapshot({
      name: "mainnet snapshot",
      sourceType: "local",
      source: "/tmp/snapshot.tar.gz",
      chain: "n3",
      network: "mainnet",
      nodeType: "neo-cli",
      storageEngine: "leveldb",
      height: 100,
      sha256: "",
    })).toThrow(/sha256/i);
  });

  it("verifies local snapshot sha256", async () => {
    const dir = mkdtempSync(join(tmpdir(), "neonexus-fast-sync-"));
    const archive = join(dir, "snapshot.bin");
    writeFileSync(archive, "snapshot-data");
    const sha256 = crypto.createHash("sha256").update("snapshot-data").digest("hex");
    const manager = createManager();
    const snapshot = manager.registerSnapshot({
      name: "mainnet snapshot",
      sourceType: "local",
      source: archive,
      chain: "n3",
      network: "mainnet",
      nodeType: "neo-cli",
      storageEngine: "leveldb",
      height: 100,
      sha256,
    });

    await expect(manager.verifySnapshot(snapshot.id)).resolves.toMatchObject({ ok: true, sha256 });
  });

  it("rejects incompatible snapshot metadata", () => {
    const manager = createManager();
    const snapshot = manager.registerSnapshot({
      name: "testnet snapshot",
      sourceType: "local",
      source: "/tmp/snapshot.tar.gz",
      chain: "n3",
      network: "testnet",
      nodeType: "neo-cli",
      storageEngine: "rocksdb",
      height: 100,
      sha256: "a".repeat(64),
    });

    expect(() => manager.assertCompatible(snapshot, {
      chain: "n3",
      network: "mainnet",
      type: "neo-cli",
      storageEngine: "rocksdb",
    })).toThrow(/network/i);
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
npm run test:backend -- tests/unit/FastSyncManager.test.ts
```

Expected: FAIL because manager does not exist.

- [ ] **Step 3: Implement FastSyncManager**

Create `src/core/FastSyncManager.ts` with:

```ts
import crypto from "node:crypto";
import { createReadStream, existsSync, statSync } from "node:fs";
import type Database from "better-sqlite3";
import type { FastSyncSnapshot, FastSyncSourceType, NodeChain, NodeNetwork, NodeType, StorageEngine } from "../types";
import type { FastSyncSnapshotRow } from "../types/database";

export interface RegisterFastSyncSnapshotInput {
  name: string;
  sourceType: FastSyncSourceType;
  source: string;
  chain: NodeChain;
  network: NodeNetwork;
  nodeType: NodeType;
  storageEngine: StorageEngine;
  height: number;
  blockHash?: string;
  sha256: string;
  sizeBytes?: number;
  signature?: string;
  trusted?: boolean;
}

export interface SnapshotCompatibilityInput {
  chain: NodeChain;
  network: NodeNetwork;
  type: NodeType;
  storageEngine: StorageEngine;
}

export class FastSyncManager {
  constructor(private readonly db: Database.Database) {}

  listSnapshots(): FastSyncSnapshot[] {
    const rows = this.db.prepare("SELECT * FROM fast_sync_snapshots ORDER BY created_at DESC").all() as FastSyncSnapshotRow[];
    return rows.map((row) => this.mapRow(row));
  }

  getSnapshot(id: string): FastSyncSnapshot | null {
    const row = this.db.prepare("SELECT * FROM fast_sync_snapshots WHERE id = ?").get(id) as FastSyncSnapshotRow | undefined;
    return row ? this.mapRow(row) : null;
  }

  registerSnapshot(input: RegisterFastSyncSnapshotInput): FastSyncSnapshot {
    if (!/^[a-fA-F0-9]{64}$/.test(input.sha256)) {
      throw new Error("Fast sync snapshot sha256 must be a 64-character hex digest");
    }
    if (input.height <= 0) {
      throw new Error("Fast sync snapshot height must be positive");
    }
    const snapshot: FastSyncSnapshot = {
      id: `snapshot-${crypto.randomUUID()}`,
      name: input.name,
      sourceType: input.sourceType,
      source: input.source,
      chain: input.chain,
      network: input.network,
      nodeType: input.nodeType,
      storageEngine: input.storageEngine,
      height: input.height,
      blockHash: input.blockHash,
      sha256: input.sha256.toLowerCase(),
      sizeBytes: input.sizeBytes,
      signature: input.signature,
      trusted: input.trusted === true,
      createdAt: Date.now(),
    };
    this.db.prepare(`
      INSERT INTO fast_sync_snapshots (id, name, source_type, source, chain, network, node_type, storage_engine, height, block_hash, sha256, size_bytes, signature, trusted, created_at, last_verified_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    `).run(snapshot.id, snapshot.name, snapshot.sourceType, snapshot.source, snapshot.chain, snapshot.network, snapshot.nodeType, snapshot.storageEngine, snapshot.height, snapshot.blockHash ?? null, snapshot.sha256, snapshot.sizeBytes ?? null, snapshot.signature ?? null, snapshot.trusted ? 1 : 0, snapshot.createdAt, null);
    return snapshot;
  }

  async verifySnapshot(id: string): Promise<{ ok: boolean; sha256: string; sizeBytes: number; verifiedAt: number }> {
    const snapshot = this.getSnapshot(id);
    if (!snapshot) throw new Error(`Snapshot ${id} not found`);
    if (snapshot.sourceType !== "local") throw new Error("Only local snapshot verification is available in this operation");
    if (!existsSync(snapshot.source)) throw new Error(`Snapshot file does not exist: ${snapshot.source}`);
    const sha256 = await this.sha256File(snapshot.source);
    if (sha256 !== snapshot.sha256) throw new Error("Snapshot sha256 does not match manifest");
    const verifiedAt = Date.now();
    const sizeBytes = statSync(snapshot.source).size;
    this.db.prepare("UPDATE fast_sync_snapshots SET size_bytes = ?, last_verified_at = ? WHERE id = ?").run(sizeBytes, verifiedAt, id);
    return { ok: true, sha256, sizeBytes, verifiedAt };
  }

  assertCompatible(snapshot: FastSyncSnapshot, node: SnapshotCompatibilityInput): void {
    if (snapshot.chain !== node.chain) throw new Error("Snapshot chain does not match node chain");
    if (snapshot.network !== node.network) throw new Error("Snapshot network does not match node network");
    if (snapshot.nodeType !== node.type) throw new Error("Snapshot node type does not match node type");
    if (snapshot.storageEngine !== node.storageEngine) throw new Error("Snapshot storage engine does not match active data context");
  }

  private sha256File(filePath: string): Promise<string> {
    return new Promise((resolve, reject) => {
      const hash = crypto.createHash("sha256");
      const stream = createReadStream(filePath);
      stream.on("error", reject);
      stream.on("data", (chunk) => hash.update(chunk));
      stream.on("end", () => resolve(hash.digest("hex")));
    });
  }

  private mapRow(row: FastSyncSnapshotRow): FastSyncSnapshot {
    return {
      id: row.id,
      name: row.name,
      sourceType: row.source_type as FastSyncSourceType,
      source: row.source,
      chain: row.chain as NodeChain,
      network: row.network as NodeNetwork,
      nodeType: row.node_type as NodeType,
      storageEngine: row.storage_engine as StorageEngine,
      height: row.height,
      blockHash: row.block_hash ?? undefined,
      sha256: row.sha256,
      sizeBytes: row.size_bytes ?? undefined,
      signature: row.signature ?? undefined,
      trusted: row.trusted === 1,
      createdAt: row.created_at,
      lastVerifiedAt: row.last_verified_at ?? undefined,
    };
  }
}
```

- [ ] **Step 4: Add fast sync route tests**

Create `tests/integration/fast-sync.router.actual.test.ts`:

```ts
import express from "express";
import request from "supertest";
import { describe, expect, it, vi } from "vitest";
import { createFastSyncRouter } from "../../src/api/routes/fastSync";

describe("Fast sync router", () => {
  it("lists snapshots", async () => {
    const app = express();
    const manager = { listSnapshots: vi.fn(() => [{ id: "snapshot-1", name: "Mainnet", sha256: "a".repeat(64) }]) };
    app.use("/api/fast-sync", createFastSyncRouter(manager as never));

    const response = await request(app).get("/api/fast-sync/snapshots");

    expect(response.status).toBe(200);
    expect(response.body.snapshots[0].id).toBe("snapshot-1");
  });

  it("registers a custom snapshot", async () => {
    const app = express();
    app.use(express.json());
    const manager = {
      registerSnapshot: vi.fn((input) => ({ id: "snapshot-1", ...input })),
    };
    app.use("/api/fast-sync", createFastSyncRouter(manager as never));

    const response = await request(app).post("/api/fast-sync/snapshots").send({
      name: "Mainnet",
      sourceType: "local",
      source: "/tmp/snapshot.tar.gz",
      chain: "n3",
      network: "mainnet",
      nodeType: "neo-cli",
      storageEngine: "leveldb",
      height: 100,
      sha256: "a".repeat(64),
    });

    expect(response.status).toBe(201);
    expect(manager.registerSnapshot).toHaveBeenCalledWith(expect.objectContaining({ name: "Mainnet" }));
  });
});
```

- [ ] **Step 5: Implement fast sync routes**

Create `src/api/routes/fastSync.ts`:

```ts
import { Router, type Request, type Response } from "express";
import type { FastSyncManager } from "../../core/FastSyncManager";
import { respondWithApiError } from "../respond";

export function createFastSyncRouter(manager: FastSyncManager): Router {
  const router = Router();

  router.get("/snapshots", (_req: Request, res: Response) => {
    try {
      res.json({ snapshots: manager.listSnapshots() });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/snapshots", (req: Request, res: Response) => {
    try {
      const snapshot = manager.registerSnapshot(req.body);
      res.status(201).json({ snapshot });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/snapshots/:id/verify", async (req: Request<{ id: string }>, res: Response) => {
    try {
      const result = await manager.verifySnapshot(req.params.id);
      res.json({ result });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
```

Mount in `src/server.ts` after manager instantiation:

```ts
const fastSyncManager = new FastSyncManager(config.db);
app.use("/api/fast-sync", requireAuth, requireAdmin, createFastSyncRouter(fastSyncManager));
```

- [ ] **Step 6: Run tests**

Run:

```bash
npm run test:backend -- tests/unit/FastSyncManager.test.ts tests/integration/fast-sync.router.actual.test.ts
```

Expected: PASS.

- [ ] **Step 7: Commit**

Run:

```bash
git add src/core/FastSyncManager.ts src/api/routes/fastSync.ts src/server.ts tests/unit/FastSyncManager.test.ts tests/integration/fast-sync.router.actual.test.ts
git commit -m "feat: add fast sync manifests"
```

### Task 6: Role and Data Context API Routes

**Files:**
- Create: `src/api/routes/nodeRoles.ts`
- Modify: `src/api/routes/nodes.ts`
- Modify: `src/server.ts`
- Test: `tests/integration/node-roles.router.actual.test.ts`
- Test: `tests/integration/nodes.router.actual.test.ts`

- [ ] **Step 1: Write failing node role route tests**

Create `tests/integration/node-roles.router.actual.test.ts`:

```ts
import express from "express";
import request from "supertest";
import { describe, expect, it, vi } from "vitest";
import { createNodeRolesRouter } from "../../src/api/routes/nodeRoles";

describe("Node roles router", () => {
  it("lists roles", async () => {
    const app = express();
    const roleManager = { listRoles: vi.fn(() => [{ id: "builtin-state", name: "State Node" }]) };
    const nodeManager = { getNode: vi.fn() };
    app.use("/api/node-roles", createNodeRolesRouter(roleManager as never, nodeManager as never));

    const response = await request(app).get("/api/node-roles");

    expect(response.status).toBe(200);
    expect(response.body.roles[0].id).toBe("builtin-state");
  });

  it("plans a role for a node", async () => {
    const app = express();
    app.use(express.json());
    const node = { id: "node-1", type: "neo-cli", process: { status: "stopped" } };
    const roleManager = { planRoleApplication: vi.fn(() => ({ nodeId: "node-1", roleId: "builtin-state", changes: [], warnings: [], requiresRestart: true })) };
    const nodeManager = { getNode: vi.fn(() => node) };
    app.use("/api/node-roles", createNodeRolesRouter(roleManager as never, nodeManager as never));

    const response = await request(app).post("/api/node-roles/builtin-state/plan").send({ nodeId: "node-1" });

    expect(response.status).toBe(200);
    expect(roleManager.planRoleApplication).toHaveBeenCalledWith(expect.objectContaining({ roleId: "builtin-state", node }));
  });
});
```

Add to `tests/integration/nodes.router.actual.test.ts`:

```ts
it("lists data contexts for a node", async () => {
  const getDataContextManager = vi.fn(() => ({
    listContexts: vi.fn(() => [{ id: "ctx-1", label: "default", active: true }]),
  }));
  (mockNodeManager as any).getDataContextManager = getDataContextManager;

  const response = await request(app).get("/api/nodes/node-1/data-contexts");

  expect(response.status).toBe(200);
  expect(response.body.contexts[0].id).toBe("ctx-1");
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
npm run test:backend -- tests/integration/node-roles.router.actual.test.ts tests/integration/nodes.router.actual.test.ts
```

Expected: FAIL because routes and node manager accessors do not exist.

- [ ] **Step 3: Implement node role routes**

Create `src/api/routes/nodeRoles.ts`:

```ts
import { Router, type Request, type Response } from "express";
import type { NodeRoleManager } from "../../core/NodeRoleManager";
import type { NodeManager } from "../../core/NodeManager";
import { Errors } from "../errors";
import { respondWithApiError } from "../respond";

export function createNodeRolesRouter(roleManager: NodeRoleManager, nodeManager: NodeManager): Router {
  const router = Router();

  router.get("/", (_req: Request, res: Response) => {
    try {
      res.json({ roles: roleManager.listRoles() });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/", (req: Request, res: Response) => {
    try {
      const role = roleManager.createCustomRole(req.body);
      res.status(201).json({ role });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/:roleId/plan", (req: Request<{ roleId: string }>, res: Response) => {
    try {
      const nodeId = req.body?.nodeId;
      if (!nodeId) throw Errors.missingField("nodeId");
      const node = nodeManager.getNode(nodeId);
      if (!node) throw Errors.nodeNotFound(nodeId);
      const plan = roleManager.planRoleApplication({
        roleId: req.params.roleId,
        node,
        storageEngineOverride: req.body?.storageEngine,
      });
      res.json({ plan });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
```

- [ ] **Step 4: Add data context accessors and routes**

In `NodeManager`, add:

```ts
private dataContextManager = new NodeDataContextManager(this.db);

getDataContextManager(): NodeDataContextManager {
  return this.dataContextManager;
}
```

In `src/api/routes/nodes.ts`, add before `/:id` detail route:

```ts
router.get("/:id/data-contexts", (req: Request<NodeParams>, res: Response) => {
  try {
    const node = nodeManager.getNode(req.params.id);
    if (!node) throw Errors.nodeNotFound(req.params.id);
    res.json({ contexts: nodeManager.getDataContextManager().listContexts(req.params.id) });
  } catch (error) {
    respondWithApiError(res, error);
  }
});

router.post("/:id/data-contexts", (req: Request<NodeParams>, res: Response) => {
  try {
    const node = nodeManager.getNode(req.params.id);
    if (!node) throw Errors.nodeNotFound(req.params.id);
    if (node.process.status === "running") throw Errors.nodeRunning();
    const context = nodeManager.getDataContextManager().createContext(req.params.id, req.body);
    res.status(201).json({ context });
  } catch (error) {
    respondWithApiError(res, error);
  }
});

router.post("/:id/data-contexts/:contextId/activate", async (req: Request<NodeParams & { contextId: string }>, res: Response) => {
  try {
    const node = nodeManager.getNode(req.params.id);
    if (!node) throw Errors.nodeNotFound(req.params.id);
    if (node.process.status === "running") throw Errors.nodeRunning();
    const context = nodeManager.getDataContextManager().activateContext(req.params.id, req.params.contextId);
    await nodeManager.updateNode(req.params.id, {
      settings: {
        activeDataContextId: context.id,
        storageEngine: context.storageEngine,
        syncStrategy: context.syncStrategy,
      },
    });
    res.json({ context, node: nodeManager.getNode(req.params.id) });
  } catch (error) {
    respondWithApiError(res, error);
  }
});
```

- [ ] **Step 5: Mount routes**

In `src/server.ts`, add:

```ts
const nodeRoleManager = new NodeRoleManager(config.db);
app.use("/api/node-roles", requireAuth, requireAdminForUnsafeMethods, createNodeRolesRouter(nodeRoleManager, nodeManager));
```

- [ ] **Step 6: Run tests**

Run:

```bash
npm run test:backend -- tests/integration/node-roles.router.actual.test.ts tests/integration/nodes.router.actual.test.ts
```

Expected: PASS.

- [ ] **Step 7: Commit**

Run:

```bash
git add src/api/routes/nodeRoles.ts src/api/routes/nodes.ts src/server.ts src/core/NodeManager.ts tests/integration/node-roles.router.actual.test.ts tests/integration/nodes.router.actual.test.ts
git commit -m "feat: expose role and data context APIs"
```

### Task 7: Private Network Plan Generation

**Files:**
- Create: `src/core/PrivateNetworkManager.ts`
- Create: `src/api/routes/privateNetworks.ts`
- Modify: `src/server.ts`
- Test: `tests/unit/PrivateNetworkManager.test.ts`
- Test: `tests/integration/private-networks.router.actual.test.ts`

- [ ] **Step 1: Write failing private network tests**

Create `tests/unit/PrivateNetworkManager.test.ts`:

```ts
import Database from "better-sqlite3";
import { describe, expect, it } from "vitest";
import { PrivateNetworkManager } from "../../src/core/PrivateNetworkManager";

function createManager() {
  const db = new Database(":memory:");
  db.exec(`
    CREATE TABLE private_network_plans (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      template TEXT NOT NULL,
      network_magic INTEGER NOT NULL,
      plan TEXT NOT NULL,
      status TEXT NOT NULL,
      created_at INTEGER NOT NULL,
      applied_at INTEGER
    );
  `);
  return new PrivateNetworkManager(db);
}

describe("PrivateNetworkManager", () => {
  it.each([
    ["single", 1, 1],
    ["four", 4, 4],
    ["seven", 7, 7],
  ] as const)("generates a %s private network plan", (template, nodeCount, validatorsCount) => {
    const manager = createManager();
    const plan = manager.createPlan({ name: `${template} lab`, template, storageEngine: "leveldb" });

    expect(plan.plan.nodes).toHaveLength(nodeCount);
    expect(plan.plan.validatorsCount).toBe(validatorsCount);
    expect(plan.plan.standbyCommittee).toHaveLength(validatorsCount);
    expect(plan.plan.seedList).toHaveLength(nodeCount);
    expect(new Set(plan.plan.nodes.map((node) => node.ports.p2p))).toHaveLength(nodeCount);
  });

  it("does not include private key material in generated plans", () => {
    const manager = createManager();
    const plan = manager.createPlan({ name: "seven lab", template: "seven", storageEngine: "rocksdb" });

    expect(JSON.stringify(plan)).not.toMatch(/privateKey|wif|password/i);
  });
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
npm run test:backend -- tests/unit/PrivateNetworkManager.test.ts
```

Expected: FAIL because manager does not exist.

- [ ] **Step 3: Implement PrivateNetworkManager**

Create `src/core/PrivateNetworkManager.ts`:

```ts
import crypto from "node:crypto";
import type Database from "better-sqlite3";
import type { PrivateNetworkPlan, PrivateNetworkTemplate, StorageEngine } from "../types";
import type { PrivateNetworkPlanRow } from "../types/database";

export interface CreatePrivateNetworkPlanInput {
  name: string;
  template: PrivateNetworkTemplate;
  storageEngine: StorageEngine;
}

const TEMPLATE_COUNTS: Record<PrivateNetworkTemplate, number> = {
  single: 1,
  four: 4,
  seven: 7,
};

export class PrivateNetworkManager {
  constructor(private readonly db: Database.Database) {}

  createPlan(input: CreatePrivateNetworkPlanInput): PrivateNetworkPlan {
    const nodeCount = TEMPLATE_COUNTS[input.template];
    const networkMagic = this.generateNetworkMagic();
    const nodes = Array.from({ length: nodeCount }, (_, index) => {
      const publicKey = this.generatePublicKey(index);
      return {
        name: `${input.name} ${index + 1}`,
        type: "neo-cli" as const,
        roleIds: ["builtin-consensus", "builtin-rpc-api"],
        storageEngine: input.storageEngine,
        ports: {
          rpc: 10332 + index * 10,
          p2p: 10333 + index * 10,
          websocket: 10334 + index * 10,
        },
        publicKey,
        address: this.publicKeyToDisplayAddress(publicKey, index),
      };
    });
    const plan: PrivateNetworkPlan = {
      id: `privnet-${crypto.randomUUID()}`,
      name: input.name,
      template: input.template,
      networkMagic,
      plan: {
        nodes,
        seedList: nodes.map((node) => `127.0.0.1:${node.ports.p2p}`),
        validatorsCount: nodeCount,
        standbyCommittee: nodes.map((node) => node.publicKey),
      },
      status: "draft",
      createdAt: Date.now(),
    };
    this.db.prepare(`
      INSERT INTO private_network_plans (id, name, template, network_magic, plan, status, created_at, applied_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    `).run(plan.id, plan.name, plan.template, plan.networkMagic, JSON.stringify(plan.plan), plan.status, plan.createdAt, null);
    return plan;
  }

  listPlans(): PrivateNetworkPlan[] {
    const rows = this.db.prepare("SELECT * FROM private_network_plans ORDER BY created_at DESC").all() as PrivateNetworkPlanRow[];
    return rows.map((row) => this.mapRow(row));
  }

  private generateNetworkMagic(): number {
    return 700000000 + crypto.randomInt(1, 99999999);
  }

  private generatePublicKey(index: number): string {
    const seed = crypto.createHash("sha256").update(`neonexus-private-network-${Date.now()}-${index}`).digest("hex");
    return `03${seed.slice(0, 64)}`;
  }

  private publicKeyToDisplayAddress(publicKey: string, index: number): string {
    const digest = crypto.createHash("sha256").update(publicKey).digest("hex").slice(0, 33);
    return `N${digest}${index}`;
  }

  private mapRow(row: PrivateNetworkPlanRow): PrivateNetworkPlan {
    return {
      id: row.id,
      name: row.name,
      template: row.template as PrivateNetworkTemplate,
      networkMagic: row.network_magic,
      plan: JSON.parse(row.plan) as PrivateNetworkPlan["plan"],
      status: row.status as PrivateNetworkPlan["status"],
      createdAt: row.created_at,
      appliedAt: row.applied_at ?? undefined,
    };
  }
}
```

- [ ] **Step 4: Add routes and tests**

Create `tests/integration/private-networks.router.actual.test.ts`:

```ts
import express from "express";
import request from "supertest";
import { describe, expect, it, vi } from "vitest";
import { createPrivateNetworksRouter } from "../../src/api/routes/privateNetworks";

describe("Private networks router", () => {
  it("creates a private network plan", async () => {
    const app = express();
    app.use(express.json());
    const manager = { createPlan: vi.fn((input) => ({ id: "privnet-1", ...input, plan: { nodes: [] } })) };
    app.use("/api/private-networks", createPrivateNetworksRouter(manager as never));

    const response = await request(app).post("/api/private-networks/plan").send({
      name: "local lab",
      template: "four",
      storageEngine: "rocksdb",
    });

    expect(response.status).toBe(201);
    expect(manager.createPlan).toHaveBeenCalledWith({ name: "local lab", template: "four", storageEngine: "rocksdb" });
  });
});
```

Create `src/api/routes/privateNetworks.ts`:

```ts
import { Router, type Request, type Response } from "express";
import type { PrivateNetworkManager } from "../../core/PrivateNetworkManager";
import { respondWithApiError } from "../respond";

export function createPrivateNetworksRouter(manager: PrivateNetworkManager): Router {
  const router = Router();

  router.get("/plans", (_req: Request, res: Response) => {
    try {
      res.json({ plans: manager.listPlans() });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  router.post("/plan", (req: Request, res: Response) => {
    try {
      const plan = manager.createPlan(req.body);
      res.status(201).json({ plan });
    } catch (error) {
      respondWithApiError(res, error);
    }
  });

  return router;
}
```

Mount in `src/server.ts`:

```ts
const privateNetworkManager = new PrivateNetworkManager(config.db);
app.use("/api/private-networks", requireAuth, requireAdmin, createPrivateNetworksRouter(privateNetworkManager));
```

- [ ] **Step 5: Run tests**

Run:

```bash
npm run test:backend -- tests/unit/PrivateNetworkManager.test.ts tests/integration/private-networks.router.actual.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add src/core/PrivateNetworkManager.ts src/api/routes/privateNetworks.ts src/server.ts tests/unit/PrivateNetworkManager.test.ts tests/integration/private-networks.router.actual.test.ts
git commit -m "feat: add private network planning"
```

### Task 8: Role Application Orchestration

**Files:**
- Modify: `src/core/NodeRoleManager.ts`
- Modify: `src/core/NodeManager.ts`
- Modify: `src/api/routes/nodeRoles.ts`
- Test: `tests/unit/NodeRoleManager.apply.test.ts`
- Test: `tests/integration/node-roles.router.actual.test.ts`

- [ ] **Step 1: Write failing apply tests**

Create `tests/unit/NodeRoleManager.apply.test.ts`:

```ts
import Database from "better-sqlite3";
import { describe, expect, it, vi } from "vitest";
import { NodeRoleManager } from "../../src/core/NodeRoleManager";

describe("NodeRoleManager apply", () => {
  it("blocks applying roles to running nodes without stopApplyRestart", async () => {
    const db = new Database(":memory:");
    db.exec(`
      CREATE TABLE node_role_profiles (id TEXT PRIMARY KEY, name TEXT, description TEXT, kind TEXT, node_types TEXT, profile TEXT, created_by TEXT, created_at INTEGER, updated_at INTEGER);
      CREATE TABLE node_role_applications (id TEXT PRIMARY KEY, node_id TEXT, role_id TEXT, role_name TEXT, application_plan TEXT, previous_state TEXT, applied_at INTEGER, applied_by TEXT, status TEXT, error_message TEXT);
    `);
    const manager = new NodeRoleManager(db);
    const nodeManager = {
      getNode: vi.fn(() => ({
        id: "node-1",
        name: "State",
        chain: "n3",
        type: "neo-cli",
        network: "mainnet",
        syncMode: "full",
        version: "3.9.2",
        ports: { rpc: 10332, p2p: 10333 },
        paths: { base: "/tmp/node", data: "/tmp/node/data", logs: "/tmp/node/logs", config: "/tmp/node/config" },
        settings: {},
        process: { status: "running" },
        createdAt: 1,
        updatedAt: 1,
      })),
    };

    await expect(manager.applyRole({
      roleId: "builtin-state",
      nodeId: "node-1",
      nodeManager: nodeManager as never,
      stopApplyRestart: false,
    })).rejects.toThrow(/running/i);
  });
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
npm run test:backend -- tests/unit/NodeRoleManager.apply.test.ts
```

Expected: FAIL because `applyRole` does not exist.

- [ ] **Step 3: Implement applyRole**

In `src/core/NodeRoleManager.ts`, add:

```ts
export interface ApplyRoleInput {
  roleId: string;
  nodeId: string;
  nodeManager: NodeManager;
  appliedBy?: string;
  stopApplyRestart?: boolean;
  storageEngineOverride?: StorageEngine;
}

async applyRole(input: ApplyRoleInput): Promise<NodeRoleApplication> {
  const node = input.nodeManager.getNode(input.nodeId);
  if (!node) throw new Error(`Node ${input.nodeId} not found`);
  if (node.process.status === "running" && !input.stopApplyRestart) {
    throw new Error("Node must be stopped before applying a role");
  }

  const plan = this.planRoleApplication({
    roleId: input.roleId,
    node,
    storageEngineOverride: input.storageEngineOverride,
  });
  const previousState = {
    settings: node.settings,
    plugins: node.plugins ?? [],
  };

  try {
    if (node.process.status === "running" && input.stopApplyRestart) {
      await input.nodeManager.stopNode(node.id);
    }

    const role = this.getRole(input.roleId)!;
    const storageEngine = input.storageEngineOverride ?? role.profile.storageEngine ?? node.settings.storageEngine ?? "leveldb";
    await input.nodeManager.ensureStorageEngine(node.id, storageEngine);

    for (const plugin of role.profile.plugins ?? []) {
      if (plugin.enabled) {
        await input.nodeManager.installOrUpdatePlugin(node.id, plugin.id, plugin.config ?? {});
      } else {
        await input.nodeManager.setPluginEnabled(node.id, plugin.id, false);
      }
    }

    await input.nodeManager.updateNode(node.id, {
      settings: {
        ...(role.profile.settings ?? {}),
        role: { id: role.id, name: role.name, appliedAt: Date.now() },
      },
    });

    if (input.stopApplyRestart) {
      await input.nodeManager.startNode(node.id);
    }

    return this.recordApplication({
      nodeId: node.id,
      roleId: role.id,
      roleName: role.name,
      applicationPlan: plan,
      previousState,
      appliedBy: input.appliedBy,
      status: "applied",
    });
  } catch (error) {
    return this.recordApplication({
      nodeId: node.id,
      roleId: input.roleId,
      roleName: plan.roleName,
      applicationPlan: plan,
      previousState,
      appliedBy: input.appliedBy,
      status: "failed",
      errorMessage: error instanceof Error ? error.message : "Role application failed",
    });
  }
}
```

Add imports for `NodeManager`.

In `NodeManager`, add:

```ts
async installOrUpdatePlugin(nodeId: string, pluginId: PluginId, config?: Record<string, unknown>): Promise<void> {
  const node = this.getNode(nodeId);
  if (!node) throw Errors.nodeNotFound(nodeId);
  if (node.type !== "neo-cli") throw Errors.pluginsCliOnly();
  if (node.process.status === "running") throw Errors.nodeRunning();
  this.assertCanWriteImportedNode(node, "plugin role application");
  await this.pluginManager.upsertPlugin(nodeId, pluginId, node.version, config ?? {});
  const updatedNode = this.getNode(nodeId)!;
  await ConfigManager.writeNodeConfig(updatedNode, this.getEnabledPluginIds(nodeId));
}
```

- [ ] **Step 4: Add apply route**

In `src/api/routes/nodeRoles.ts`, add:

```ts
router.post("/:roleId/apply", async (req: Request<{ roleId: string }>, res: Response) => {
  try {
    const nodeId = req.body?.nodeId;
    if (!nodeId) throw Errors.missingField("nodeId");
    const application = await roleManager.applyRole({
      roleId: req.params.roleId,
      nodeId,
      nodeManager,
      stopApplyRestart: req.body?.stopApplyRestart === true,
      storageEngineOverride: req.body?.storageEngine,
    });
    res.json({ application, node: nodeManager.getNode(nodeId) });
  } catch (error) {
    respondWithApiError(res, error);
  }
});
```

- [ ] **Step 5: Run tests**

Run:

```bash
npm run test:backend -- tests/unit/NodeRoleManager.apply.test.ts tests/integration/node-roles.router.actual.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add src/core/NodeRoleManager.ts src/core/NodeManager.ts src/api/routes/nodeRoles.ts tests/unit/NodeRoleManager.apply.test.ts tests/integration/node-roles.router.actual.test.ts
git commit -m "feat: apply node roles"
```

### Task 9: Frontend API Hooks and Create Node Controls

**Files:**
- Create: `web/src/hooks/useNodeRoles.ts`
- Create: `web/src/hooks/useFastSync.ts`
- Create: `web/src/hooks/usePrivateNetworks.ts`
- Modify: `web/src/hooks/useNodes.ts`
- Modify: `web/src/utils/nodePayloads.ts`
- Modify: `web/src/pages/CreateNode.tsx`
- Test: `tests/unit/web.nodePayloads.test.ts`
- Test: `web/tests/frontend-utils.test.ts`

- [ ] **Step 1: Write failing frontend payload tests**

Add to `web/tests/frontend-utils.test.ts`:

```ts
import { buildRoleApplyPayload } from "../src/hooks/useNodeRoles";

it("builds role apply payload with storage engine and restart preference", () => {
  expect(buildRoleApplyPayload({
    nodeId: "node-1",
    storageEngine: "rocksdb",
    stopApplyRestart: true,
  })).toEqual({
    nodeId: "node-1",
    storageEngine: "rocksdb",
    stopApplyRestart: true,
  });
});
```

- [ ] **Step 2: Run tests to verify failure**

Run:

```bash
npm --prefix web run test
npm run test:backend -- tests/unit/web.nodePayloads.test.ts
```

Expected: FAIL because hook helper and payload fields do not exist.

- [ ] **Step 3: Add hooks**

Create `web/src/hooks/useNodeRoles.ts`:

```ts
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "../utils/api";

export type StorageEngine = "leveldb" | "rocksdb";

export interface NodeRole {
  id: string;
  name: string;
  description?: string;
  kind: "builtin" | "custom";
  nodeTypes: string[];
}

export interface RoleApplyInput {
  nodeId: string;
  storageEngine?: StorageEngine;
  stopApplyRestart?: boolean;
}

export function buildRoleApplyPayload(input: RoleApplyInput) {
  return {
    nodeId: input.nodeId,
    ...(input.storageEngine ? { storageEngine: input.storageEngine } : {}),
    stopApplyRestart: input.stopApplyRestart === true,
  };
}

export function useNodeRoles() {
  return useQuery({
    queryKey: ["node-roles"],
    queryFn: async () => {
      const response = await api.get<{ roles: NodeRole[] }>("/node-roles");
      return response.roles;
    },
  });
}

export function usePlanNodeRole(roleId: string) {
  return useMutation({
    mutationFn: async (input: RoleApplyInput) => {
      const response = await api.post<{ plan: unknown }>(`/node-roles/${roleId}/plan`, buildRoleApplyPayload(input));
      return response.plan;
    },
  });
}

export function useApplyNodeRole(roleId: string) {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (input: RoleApplyInput) => {
      const response = await api.post<{ application: unknown }>(`/node-roles/${roleId}/apply`, buildRoleApplyPayload(input));
      return response.application;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["nodes"] });
      queryClient.invalidateQueries({ queryKey: ["node-roles"] });
    },
  });
}
```

Create `web/src/hooks/useFastSync.ts` and `web/src/hooks/usePrivateNetworks.ts` using the same API wrapper pattern.

- [ ] **Step 4: Extend node types and CreateNode**

In `web/src/hooks/useNodes.ts`, add:

```ts
storageEngine?: "leveldb" | "rocksdb";
syncStrategy?: "full" | "light" | "fast-sync";
activeDataContextId?: string;
role?: { id: string; name: string; appliedAt: number };
```

In `web/src/pages/CreateNode.tsx`, add segmented controls near sync mode:

```tsx
const STORAGE_ENGINES = [
  { value: "leveldb", label: "LevelDB", description: "Default, stable storage engine" },
  { value: "rocksdb", label: "RocksDB", description: "Higher throughput for state and index workloads" },
] as const;

const SYNC_STRATEGIES = [
  { value: "full", label: "Full sync", description: "Sync from genesis" },
  { value: "light", label: "Light sync", description: "Use light sync where supported" },
  { value: "fast-sync", label: "Fast sync", description: "Use a verified snapshot/checkpoint" },
] as const;
```

Render radio card groups using existing Node Type/Network visual pattern.

- [ ] **Step 5: Run frontend checks**

Run:

```bash
npm --prefix web run lint
npm --prefix web run typecheck
npm --prefix web run test
```

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```bash
git add web/src/hooks/useNodeRoles.ts web/src/hooks/useFastSync.ts web/src/hooks/usePrivateNetworks.ts web/src/hooks/useNodes.ts web/src/utils/nodePayloads.ts web/src/pages/CreateNode.tsx web/tests/frontend-utils.test.ts tests/unit/web.nodePayloads.test.ts
git commit -m "feat(ui): add storage and role hooks"
```

### Task 10: Roles Workspace, Node Detail Panel, and Private Network Builder

**Files:**
- Create: `web/src/pages/Roles.tsx`
- Create: `web/src/pages/PrivateNetworkBuilder.tsx`
- Modify: `web/src/pages/NodeDetail.tsx`
- Modify: `web/src/components/Layout.tsx`
- Modify: `web/src/App.tsx`
- Test: `web/tests/frontend-utils.test.ts`

- [ ] **Step 1: Write frontend helper tests**

Add to `web/tests/frontend-utils.test.ts`:

```ts
import { summarizePrivateNetworkTemplate } from "../src/pages/PrivateNetworkBuilder";

it("summarizes private network templates", () => {
  expect(summarizePrivateNetworkTemplate("single")).toMatchObject({ nodeCount: 1, validatorsCount: 1 });
  expect(summarizePrivateNetworkTemplate("four")).toMatchObject({ nodeCount: 4, validatorsCount: 4 });
  expect(summarizePrivateNetworkTemplate("seven")).toMatchObject({ nodeCount: 7, validatorsCount: 7 });
});
```

- [ ] **Step 2: Run test to verify failure**

Run:

```bash
npm --prefix web run test
```

Expected: FAIL because `PrivateNetworkBuilder` does not exist.

- [ ] **Step 3: Create PrivateNetworkBuilder**

Create `web/src/pages/PrivateNetworkBuilder.tsx` with exported helper:

```tsx
import { Network, Plus } from "lucide-react";
import { useState } from "react";
import { useCreatePrivateNetworkPlan } from "../hooks/usePrivateNetworks";

export type PrivateNetworkTemplate = "single" | "four" | "seven";

export function summarizePrivateNetworkTemplate(template: PrivateNetworkTemplate) {
  const counts = { single: 1, four: 4, seven: 7 } as const;
  return {
    nodeCount: counts[template],
    validatorsCount: counts[template],
  };
}

export default function PrivateNetworkBuilder() {
  const createPlan = useCreatePrivateNetworkPlan();
  const [name, setName] = useState("Local private network");
  const [template, setTemplate] = useState<PrivateNetworkTemplate>("single");
  const [storageEngine, setStorageEngine] = useState<"leveldb" | "rocksdb">("leveldb");
  const summary = summarizePrivateNetworkTemplate(template);

  return (
    <div className="space-y-7 animate-fade-in">
      <section className="page-hero pb-5">
        <p className="console-kicker">Private network</p>
        <h1 className="mt-2 text-3xl font-semibold text-slate-950">Network Builder</h1>
        <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
          Generate a private Neo N3 network plan with roles, ports, seed list, committee public keys, and isolated data defaults.
        </p>
      </section>
      <div className="card space-y-5">
        <div>
          <label className="block text-sm font-medium text-slate-700">Network name</label>
          <input className="input mt-2" value={name} onChange={(event) => setName(event.target.value)} />
        </div>
        <div className="grid gap-3 md:grid-cols-3">
          {(["single", "four", "seven"] as const).map((option) => {
            const optionSummary = summarizePrivateNetworkTemplate(option);
            return (
              <button key={option} type="button" className={`rounded-lg border p-4 text-left ${template === option ? "border-teal-400 bg-teal-50" : "border-slate-200 bg-white"}`} onClick={() => setTemplate(option)}>
                <Network className="h-5 w-5 text-teal-700" />
                <p className="mt-3 font-semibold text-slate-950">{optionSummary.nodeCount} node{optionSummary.nodeCount === 1 ? "" : "s"}</p>
                <p className="mt-1 text-sm text-slate-600">{optionSummary.validatorsCount} validator slot{optionSummary.validatorsCount === 1 ? "" : "s"}</p>
              </button>
            );
          })}
        </div>
        <div className="flex flex-wrap gap-2">
          {(["leveldb", "rocksdb"] as const).map((engine) => (
            <button key={engine} type="button" className={`filter-chip ${storageEngine === engine ? "filter-chip-active" : ""}`} onClick={() => setStorageEngine(engine)}>
              {engine === "leveldb" ? "LevelDB" : "RocksDB"}
            </button>
          ))}
        </div>
        <button className="btn btn-primary" type="button" onClick={() => createPlan.mutate({ name, template, storageEngine })}>
          <Plus className="h-4 w-4" /> Generate plan
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Create Roles page**

Create `web/src/pages/Roles.tsx`:

```tsx
import { ShieldCheck, Wand2 } from "lucide-react";
import { useNodeRoles } from "../hooks/useNodeRoles";

export default function Roles() {
  const { data: roles = [], isLoading } = useNodeRoles();
  return (
    <div className="space-y-7 animate-fade-in">
      <section className="page-hero pb-5">
        <p className="console-kicker">Node roles</p>
        <h1 className="mt-2 text-3xl font-semibold text-slate-950">Roles</h1>
        <p className="mt-2 max-w-3xl text-sm leading-6 text-slate-600">
          Save and apply complete node identities for plugins, storage, sync strategy, and isolated data contexts.
        </p>
      </section>
      <div className="grid gap-4 xl:grid-cols-2">
        {isLoading ? (
          <div className="card">Loading roles...</div>
        ) : roles.map((role) => (
          <article key={role.id} className="card">
            <div className="flex items-start gap-3">
              <div className="rounded-lg bg-teal-50 p-2 text-teal-700"><Wand2 className="h-5 w-5" /></div>
              <div>
                <h2 className="text-lg font-semibold text-slate-950">{role.name}</h2>
                <p className="mt-1 text-sm text-slate-600">{role.description}</p>
                <p className="mt-3 inline-flex items-center gap-2 text-xs text-slate-500"><ShieldCheck className="h-3.5 w-3.5" /> {role.kind}</p>
              </div>
            </div>
          </article>
        ))}
      </div>
    </div>
  );
}
```

- [ ] **Step 5: Wire navigation and routes**

In `web/src/components/Layout.tsx`, add nav items:

```ts
{ path: "/roles", icon: Wand2, label: "Roles" },
{ path: "/private-networks", icon: Network, label: "Private Networks" },
```

In `web/src/App.tsx`, add protected routes:

```tsx
<Route path="/roles" element={<ProtectedPage><Roles /></ProtectedPage>} />
<Route path="/private-networks" element={<ProtectedPage><PrivateNetworkBuilder /></ProtectedPage>} />
```

Import `Roles` and `PrivateNetworkBuilder`.

- [ ] **Step 6: Add NodeDetail role/data panel**

In `web/src/pages/NodeDetail.tsx`, add a card in the overview grid:

```tsx
<div className="card">
  <h2 className="text-lg font-semibold text-slate-950">Role and data</h2>
  <dl className="mt-4 space-y-3 text-sm">
    <div>
      <dt className="text-slate-500">Role</dt>
      <dd className="text-slate-950">{node.settings?.role?.name ?? "No role applied"}</dd>
    </div>
    <div>
      <dt className="text-slate-500">Storage</dt>
      <dd className="capitalize text-slate-950">{node.settings?.storageEngine ?? "leveldb"}</dd>
    </div>
    <div>
      <dt className="text-slate-500">Data context</dt>
      <dd className="text-slate-950">{node.settings?.activeDataContextId ?? "default"}</dd>
    </div>
  </dl>
</div>
```

- [ ] **Step 7: Run frontend verification**

Run:

```bash
npm --prefix web run lint
npm --prefix web run typecheck
npm --prefix web run test
npm --prefix web run build
```

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```bash
git add web/src/pages/Roles.tsx web/src/pages/PrivateNetworkBuilder.tsx web/src/pages/NodeDetail.tsx web/src/components/Layout.tsx web/src/App.tsx web/tests/frontend-utils.test.ts
git commit -m "feat(ui): add role orchestration screens"
```

### Task 11: End-to-End Verification and Browser QA

**Files:**
- Modify: `README.md`
- Modify: `docs/superpowers/specs/2026-05-03-node-roles-fast-sync-private-network-design.md`

- [ ] **Step 1: Update README feature documentation**

Add a short section to `README.md`:

```md
### Node Roles, Fast Sync, and Private Networks

NeoNexus can apply role profiles to managed nodes. Built-in profiles cover RPC/API, state, oracle, indexer, consensus, and secure signer client nodes. Roles configure plugins, node settings, storage engine, and data context metadata together.

Fast sync snapshots are user-provided in this release. Register a local path or custom URL with a SHA256 digest and checkpoint metadata before importing chain data into an isolated context.

Private network planning supports 1-node, 4-node, and 7-node Neo N3 templates. Plans include ports, seed lists, committee public keys, and addresses, but do not store plaintext private keys or wallet passphrases by default.
```

- [ ] **Step 2: Run full verification**

Run:

```bash
npm run verify
```

Expected: PASS for backend lint, backend tests, frontend lint, frontend tests, typechecks, and frontend build.

- [ ] **Step 3: Run browser QA**

Start services:

```bash
NEONEXUS_DATA_DIR=/tmp/neonexus-role-qa HOST=127.0.0.1 PORT=8080 npm run dev:server
npm --prefix web run dev -- --host 127.0.0.1
```

Use a browser QA script or the in-app browser to cover:

- `/roles`
- `/private-networks`
- `/nodes/create`
- `/nodes/:id`
- `/plugins`
- `/settings`

Expected:

- no page-level horizontal overflow at 390px and 1440px
- no console errors
- role cards render
- private-network builder can generate a plan
- create-node form shows LevelDB/RocksDB and sync strategy controls

- [ ] **Step 4: Commit docs**

Run:

```bash
git add README.md docs/superpowers/specs/2026-05-03-node-roles-fast-sync-private-network-design.md
git commit -m "docs: document role orchestration"
```

## Final Verification Checklist

- [ ] `npm run verify` exits with code 0.
- [ ] Browser QA covers desktop and mobile.
- [ ] `git status --short` is clean.
- [ ] No snapshot extraction path accepts unchecked archive contents.
- [ ] No generated private-network plan contains `privateKey`, `wif`, or `password`.
- [ ] Viewer responses do not expose sensitive role, plugin, snapshot, signer, or path data.

## Spec Coverage Review

- Roles and custom role persistence: Tasks 2, 6, 8, 10.
- Data context isolation: Tasks 1, 3, 6, 10.
- Fast sync and checkpoint metadata: Tasks 1, 5, 9, 11.
- LevelDB/RocksDB selection: Tasks 3, 4, 9, 10.
- Private network 1/4/7 planning: Tasks 7, 10, 11.
- Security defaults for key material: Tasks 7 and 11.
- Frontend usability: Tasks 9, 10, 11.
- Tests and verification: each implementation task includes red/green checks; Task 11 runs full verification.
