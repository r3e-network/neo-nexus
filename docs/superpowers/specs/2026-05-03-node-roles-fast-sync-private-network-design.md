# Node Roles, Fast Sync, and Private Network Orchestration Design

Status: approved direction, implementation pending
Date: 2026-05-03

## Context

NeoNexus already has strong primitives for managed nodes, imported-node ownership, plugin install/update/remove flows, generated native configs, secure signer profiles, storage inspection, and configuration snapshots. The missing layer is orchestration: users currently need to know which plugins, settings, storage engine, chain data directory, and network parameters correspond to a node's intended operational role.

This feature adds a first-class role/profile system that can apply a complete node identity, preserve and switch user-defined identities, isolate blockchain data contexts, bootstrap private networks, and accelerate setup through snapshot/checkpoint-based fast sync.

## Goals

- Let users pick a node role such as RPC/API, consensus, state, oracle, indexer, or secure signer client and have NeoNexus configure the needed plugins, node settings, storage engine, and config files.
- Let users save custom roles from current node state and reapply them later.
- Record role applications so users can see what is active, what changed, and when it was applied.
- Allow one-click switching between isolated blockchain data contexts without deleting old chain data.
- Support fast sync packages through local paths or custom URLs with required SHA256 verification.
- Support checkpoint metadata: height, block hash, network, node type, storage engine, and snapshot source.
- Let users choose LevelDB or RocksDB at node creation, role application, and private-network template time.
- Provide private-network builders for 1-node, 4-node, and 7-node N3 networks with generated node plans, ports, seed lists, committee public keys, and addresses.
- Preserve the security model: do not store plaintext private keys, WIF, or wallet passphrases by default.

## Non-Goals

- Do not silently trust third-party fast-sync archives.
- Do not ship a curated official/community snapshot source in the first implementation. The manifest format will allow it later.
- Do not default to generating or storing local plaintext wallet private keys.
- Do not apply role changes to running nodes unless the user explicitly chooses a stop-apply-restart operation.
- Do not support Neo X private-network bootstrapping in this first pass; roles may still cover Neo X where they do not require N3 plugins.

## Architecture

### NodeRoleManager

New backend manager responsible for role definitions and role application.

Responsibilities:

- List built-in and custom role profiles.
- Create, update, delete, and clone custom roles.
- Build an application plan for a node before mutating anything.
- Apply a role by coordinating `NodeManager`, `PluginManager`, and `ConfigManager`.
- Record role application history.
- Validate node type, chain, network, imported-node ownership, secure-signer requirements, and running-state constraints.

Built-in roles are defined in code and exposed as read-only profiles. Custom roles are persisted in SQLite.

### NodeDataContextManager

New backend manager for isolated chain data contexts.

Responsibilities:

- List contexts for a node.
- Create a context with label, storage engine, sync strategy, and optional checkpoint.
- Mark one context active.
- Resolve effective data paths for config generation and storage reporting.
- Prevent context switches while the node is running unless the user chooses stop-apply-restart.

Data isolation will be implemented by generated config paths first, not by rewriting the node base path. For example:

- neo-cli storage path: `Data/<contextId>` or `Data_RocksDB/<contextId>`
- neo-cli state service path: `Data_MPT/<contextId>`
- neo-go `DBConfiguration` data path: `./data/<contextId>`

The existing node base directory remains stable, while each chain-data context has its own storage directory.

### FastSyncManager

New backend manager for snapshots and checkpoints.

Responsibilities:

- Validate snapshot manifests from local JSON, local archive path, or custom URL.
- Require SHA256 for all archives.
- Download archives into a staging directory.
- Verify checksum before extraction.
- Check compatibility against chain, network, node type, storage engine, checkpoint height, and block hash.
- Extract into a temporary context directory, then atomically activate it.
- Record downloads and imports.

First implementation source policy:

- Local file path or custom URL only.
- SHA256 required.
- Manifest signature field supported but optional.
- Official/community manifest source reserved for later.

### PrivateNetworkManager

New backend manager for private-network templates.

Responsibilities:

- Generate 1-node, 4-node, and 7-node private-network plans.
- Allocate ports for every node.
- Generate network magic and seed list.
- Generate public key/address inventory for validator slots.
- Assign roles per node.
- Create managed node records from the plan.
- Write private-network configs with matching committee, validators count, seed list, and checkpoint metadata.

Default templates:

- 1 node: consensus + RPC + state + optional oracle, useful for local development.
- 4 nodes: 4 consensus nodes, optional RPC observer, useful for small lab networks.
- 7 nodes: 7 consensus nodes, optional RPC/state/oracle observers, closer to production-style topology.

Address generation creates public keys and addresses by default. Private key export or local dev wallet generation must be an explicit unsafe development option with separate warning and tests.

## Data Model

### `node_role_profiles`

- `id TEXT PRIMARY KEY`
- `name TEXT NOT NULL`
- `description TEXT`
- `kind TEXT NOT NULL` - `builtin` or `custom`
- `node_types TEXT NOT NULL` - JSON array
- `profile TEXT NOT NULL` - JSON role body
- `created_by TEXT`
- `created_at INTEGER NOT NULL`
- `updated_at INTEGER NOT NULL`

Built-in roles may be exposed from code without table rows. The table stores custom roles and cloned built-ins.

### `node_role_applications`

- `id TEXT PRIMARY KEY`
- `node_id TEXT NOT NULL`
- `role_id TEXT NOT NULL`
- `role_name TEXT NOT NULL`
- `application_plan TEXT NOT NULL` - JSON
- `previous_state TEXT` - JSON rollback snapshot
- `applied_at INTEGER NOT NULL`
- `applied_by TEXT`
- `status TEXT NOT NULL` - `planned`, `applied`, `failed`
- `error_message TEXT`

### `node_data_contexts`

- `id TEXT PRIMARY KEY`
- `node_id TEXT NOT NULL`
- `label TEXT NOT NULL`
- `storage_engine TEXT NOT NULL` - `leveldb` or `rocksdb`
- `sync_strategy TEXT NOT NULL` - `full`, `light`, or `fast-sync`
- `checkpoint_height INTEGER`
- `checkpoint_hash TEXT`
- `snapshot_id TEXT`
- `active INTEGER NOT NULL DEFAULT 0`
- `created_at INTEGER NOT NULL`
- `updated_at INTEGER NOT NULL`

### `fast_sync_snapshots`

- `id TEXT PRIMARY KEY`
- `name TEXT NOT NULL`
- `source_type TEXT NOT NULL` - `local`, `url`, future `catalog`
- `source TEXT NOT NULL`
- `chain TEXT NOT NULL`
- `network TEXT NOT NULL`
- `node_type TEXT NOT NULL`
- `storage_engine TEXT NOT NULL`
- `height INTEGER NOT NULL`
- `block_hash TEXT`
- `sha256 TEXT NOT NULL`
- `size_bytes INTEGER`
- `signature TEXT`
- `trusted INTEGER NOT NULL DEFAULT 0`
- `created_at INTEGER NOT NULL`
- `last_verified_at INTEGER`

### `private_network_plans`

- `id TEXT PRIMARY KEY`
- `name TEXT NOT NULL`
- `template TEXT NOT NULL` - `single`, `four`, or `seven`
- `network_magic INTEGER NOT NULL`
- `plan TEXT NOT NULL` - JSON generated plan
- `status TEXT NOT NULL` - `draft`, `applied`, `failed`
- `created_at INTEGER NOT NULL`
- `applied_at INTEGER`

## Role Profile Shape

Each role profile contains:

- display metadata
- supported node types
- storage engine preference
- node settings patch
- plugin desired state
- plugin configs
- data context policy
- sync policy
- warnings and prerequisites

Example:

```json
{
  "storageEngine": "rocksdb",
  "settings": {
    "relay": true,
    "minPeers": 10,
    "maxPeers": 60
  },
  "plugins": [
    { "id": "RpcServer", "enabled": true, "config": { "BindAddress": "0.0.0.0" } },
    { "id": "StateService", "enabled": true, "config": { "AutoStart": true, "FullState": false } }
  ],
  "dataContext": {
    "mode": "reuse-or-create",
    "labelTemplate": "{role}-{network}-{storageEngine}"
  },
  "sync": {
    "strategy": "fast-sync",
    "allowCheckpoint": true
  }
}
```

## Built-In Roles

- RPC/API node: `RpcServer`, optional `RestServer`, standard LevelDB unless user chooses RocksDB.
- State node: `StateService`, optional `ApplicationLogs`, recommends RocksDB and fast sync.
- Oracle node: `OracleService`, RPC optional, checkpoint supported.
- Consensus node: `DBFTPlugin`, conservative sync policy, explicit key/signer requirement warnings.
- Indexer node: `ApplicationLogs` and `TokensTracker`, recommends RocksDB for larger datasets.
- Secure signer client: `SignClient`, requires existing secure signer profile.
- Storage-optimized node: RocksDB, no extra network-role plugins by default.

## Apply Flow

1. User selects a node and role.
2. UI requests an application plan.
3. Backend validates compatibility and returns planned changes:
   - plugins to install, update, enable, disable, or remove
   - node settings patch
   - storage engine change
   - data context selection or creation
   - checkpoint/snapshot action
   - warnings and restart requirement
4. User confirms.
5. Backend requires stopped node or executes explicit stop-apply-restart.
6. Backend records previous state.
7. Backend applies storage, data context, settings, plugins, and configs.
8. Backend writes final node config.
9. Backend records role application result.

Failure handling:

- If plugin/config application fails before final config write, restore recorded plugin/settings state where possible.
- Snapshot extraction always happens in staging and only activates after checksum validation.
- Failed role applications stay in history with an error message.

## Storage Engine Behavior

For neo-cli:

- LevelDB uses default storage path and does not require RocksDBStore.
- RocksDB installs/enables `RocksDBStore` and writes `ApplicationConfiguration.Storage.Engine = "RocksDBStore"`.
- Only one storage plugin can be active.

For neo-go:

- Write `ApplicationConfiguration.DBConfiguration.Type` based on user selection.
- No neo-cli storage plugin is installed.

Changing storage engine creates or switches to a data context by default so LevelDB and RocksDB data are isolated.

## Fast Sync and Checkpoint Behavior

Fast sync has three modes:

- Full sync: no snapshot.
- Light sync: existing lightweight behavior.
- Fast sync: verified snapshot import into a data context.

Fast sync validation:

- Chain and network must match the node.
- Node type must match the snapshot.
- Storage engine must match the active context.
- SHA256 must match before extraction.
- Checkpoint height must be positive.
- Checkpoint hash must be recorded when available.

The UI must clearly mark snapshots as user-provided unless they come from a future trusted catalog.

## API Surface

Planned endpoints:

- `GET /api/node-roles`
- `POST /api/node-roles`
- `PUT /api/node-roles/:id`
- `DELETE /api/node-roles/:id`
- `POST /api/node-roles/from-node/:nodeId`
- `POST /api/node-roles/:roleId/plan`
- `POST /api/node-roles/:roleId/apply`
- `GET /api/nodes/:id/data-contexts`
- `POST /api/nodes/:id/data-contexts`
- `POST /api/nodes/:id/data-contexts/:contextId/activate`
- `GET /api/fast-sync/snapshots`
- `POST /api/fast-sync/snapshots`
- `POST /api/fast-sync/snapshots/:id/verify`
- `POST /api/private-networks/plan`
- `POST /api/private-networks/:planId/apply`

All mutating endpoints require admin role.

## Frontend

New and changed surfaces:

- Add `Roles` workspace in the main navigation or as a tab under Nodes.
- Node detail gets a `Role and data` panel showing active role, role history, active data context, storage engine, and checkpoint.
- Create Node gets storage engine and sync strategy controls.
- Plugins page remains available for expert editing but role-managed changes show a role ownership hint.
- Private Network Builder creates 1/4/7 node plans with editable node names, ports, roles, storage engine, and snapshot/checkpoint defaults.

The UI should favor operational clarity over marketing layout: dense forms, explicit warnings, preview before apply, and clear rollback/history information.

## Security

- No plaintext WIF, private keys, or passphrases are stored by default.
- Fast-sync archives require SHA256 before extraction.
- Snapshot extraction uses staging directories and avoids path traversal.
- Custom URLs must reuse existing outbound/private-target protections where applicable.
- Viewer role cannot see sensitive paths, plugin configs, snapshot source secrets, or signer details.
- Consensus/private-network key material defaults to public key/address inventory only.

## Testing

Backend unit tests:

- built-in role definitions produce expected plugin/settings/storage plans
- applying a role blocks running nodes unless stop-apply-restart is requested
- role application records history and rollback snapshot
- storage engine switch creates separate data context
- fast-sync snapshot rejects missing SHA256, wrong network, wrong storage engine, and bad hash
- private-network 1/4/7 templates produce expected validator counts, seeds, ports, and role assignments

Integration tests:

- role listing and custom role CRUD
- plan/apply endpoints and admin enforcement
- data-context activation
- snapshot registration/verification
- private-network plan/apply

Frontend tests:

- create-node payload includes storage engine and sync strategy
- role helper renders/serializes plugin configs correctly
- private-network builder produces expected request body

Manual/browser QA:

- desktop and mobile coverage for Roles, Node Detail, Create Node, and Private Network Builder
- no horizontal overflow
- all dangerous operations require explicit confirmation

## Implementation Phases

1. Data model and backend managers for role profiles, data contexts, and fast-sync manifest validation.
2. Role planning and apply API with built-in roles.
3. Storage engine selection in create/update flows.
4. Fast-sync staging, checksum verification, and data-context activation.
5. Private-network plan generation and apply.
6. Frontend Roles workspace, Node Detail role/data panel, Create Node storage/sync controls, and Private Network Builder.
7. Browser QA and documentation updates.

## Acceptance Criteria

- A user can create a node with LevelDB or RocksDB selected.
- A user can apply a built-in RPC/state/oracle/consensus role to a stopped neo-cli node and see plugins/config updated.
- A user can save a custom role from an existing node and reapply it.
- A user can create and switch isolated data contexts.
- A user can register a custom fast-sync snapshot with SHA256 and checkpoint metadata.
- A bad snapshot hash or incompatible snapshot is rejected before extraction.
- A user can generate a 1-node, 4-node, or 7-node private-network plan.
- Applying a private-network plan creates managed nodes with matching private network config and role assignments.
- No plaintext private key or passphrase is persisted by default.
