import { describe, expect, it, vi } from 'vitest';

vi.unmock('better-sqlite3');

import Database from 'better-sqlite3';
import { BUILTIN_ROLES, NodeRoleManager } from '../../src/core/NodeRoleManager';

function createManager() {
  const db = new Database(':memory:');
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

describe('NodeRoleManager', () => {
  it('lists built-in roles for common Neo node identities', () => {
    const manager = createManager();
    const roles = manager.listRoles();
    const ids = roles.map((role) => role.id);

    expect(ids).toContain('builtin-rpc-api');
    expect(ids).toContain('builtin-state');
    expect(ids).toContain('builtin-oracle');
    expect(ids).toContain('builtin-consensus');
    expect(ids).toContain('builtin-indexer');
    expect(ids).toContain('builtin-secure-signer-client');
  });

  it('plans plugin, storage, and data context changes for a state node role', () => {
    const manager = createManager();
    const plan = manager.planRoleApplication({
      roleId: 'builtin-state',
      node: {
        id: 'node-1',
        name: 'State Node',
        chain: 'n3',
        type: 'neo-cli',
        network: 'mainnet',
        syncMode: 'full',
        version: '3.9.2',
        ports: { rpc: 10332, p2p: 10333 },
        paths: { base: '/tmp/node-1', data: '/tmp/node-1/data', logs: '/tmp/node-1/logs', config: '/tmp/node-1/config' },
        settings: {},
        createdAt: 1,
        updatedAt: 1,
        process: { status: 'stopped' },
        plugins: [],
      },
    });

    expect(plan.requiresRestart).toBe(true);
    expect(plan.changes.map((change) => change.type)).toEqual(expect.arrayContaining(['plugin', 'storage', 'data-context']));
    expect(plan.changes.map((change) => change.summary).join(' ')).toContain('StateService');
    expect(plan.changes.map((change) => change.summary).join(' ')).toContain('rocksdb');
  });

  it('rejects applying neo-cli plugin roles to neo-go nodes', () => {
    const manager = createManager();

    expect(() => manager.planRoleApplication({
      roleId: 'builtin-consensus',
      node: {
        id: 'node-1',
        name: 'Go Node',
        chain: 'n3',
        type: 'neo-go',
        network: 'mainnet',
        syncMode: 'full',
        version: '0.118.0',
        ports: { rpc: 20332, p2p: 20333 },
        paths: { base: '/tmp/node-1', data: '/tmp/node-1/data', logs: '/tmp/node-1/logs', config: '/tmp/node-1/config' },
        settings: {},
        createdAt: 1,
        updatedAt: 1,
        process: { status: 'stopped' },
        plugins: [],
      },
    })).toThrow(/not compatible/i);
  });

  it('protects built-in role templates from exported value mutation', () => {
    const exportedStateRole = BUILTIN_ROLES.find((role) => role.id === 'builtin-state');
    expect(exportedStateRole).toBeDefined();

    try {
      exportedStateRole!.profile.storageEngine = 'leveldb';
      exportedStateRole!.profile.plugins = [];
    } catch {
      // Frozen exports are acceptable; the manager should still use the original template.
    }

    const manager = createManager();
    const role = manager.getRole('builtin-state');
    const plan = manager.planRoleApplication({
      roleId: 'builtin-state',
      node: {
        id: 'node-1',
        name: 'State Node',
        chain: 'n3',
        type: 'neo-cli',
        network: 'mainnet',
        syncMode: 'full',
        version: '3.9.2',
        ports: { rpc: 10332, p2p: 10333 },
        paths: { base: '/tmp/node-1', data: '/tmp/node-1/data', logs: '/tmp/node-1/logs', config: '/tmp/node-1/config' },
        settings: {},
        createdAt: 1,
        updatedAt: 1,
        process: { status: 'stopped' },
        plugins: [],
      },
    });
    const summaries = plan.changes.map((change) => change.summary).join(' ');

    expect(role?.profile.storageEngine).toBe('rocksdb');
    expect(summaries).toContain('StateService');
    expect(summaries).toContain('rocksdb');
  });

  it('does not plan redundant storage, plugin, or settings changes for an already matching role', () => {
    const manager = createManager();

    const plan = manager.planRoleApplication({
      roleId: 'builtin-secure-signer-client',
      activeDataContext: {
        id: 'ctx-1',
        label: 'signer-client-mainnet-leveldb',
        storageEngine: 'leveldb',
        syncStrategy: 'full',
        active: true,
      },
      node: {
        id: 'node-1',
        name: 'Signer Client',
        chain: 'n3',
        type: 'neo-cli',
        network: 'mainnet',
        syncMode: 'full',
        version: '3.9.2',
        ports: { rpc: 10332, p2p: 10333 },
        paths: { base: '/tmp/node-1', data: '/tmp/node-1/data', logs: '/tmp/node-1/logs', config: '/tmp/node-1/config' },
        settings: {
          activeDataContextId: 'ctx-1',
          keyProtection: { mode: 'secure-signer' },
        },
        createdAt: 1,
        updatedAt: 1,
        process: { status: 'stopped' },
        plugins: [
          {
            id: 'SignClient',
            version: '3.9.2',
            config: {},
            installedAt: 1,
            enabled: true,
          },
        ],
      },
    });

    expect(plan.requiresRestart).toBe(false);
    expect(plan.changes).toEqual([]);
  });

  it('does not plan advisory fast-sync work when the desired strategy already matches', () => {
    const manager = createManager();

    const plan = manager.planRoleApplication({
      roleId: 'builtin-state',
      activeDataContext: {
        id: 'ctx-1',
        label: 'state-mainnet-rocksdb',
        storageEngine: 'rocksdb',
        syncStrategy: 'fast-sync',
        active: true,
      },
      node: {
        id: 'node-1',
        name: 'State Node',
        chain: 'n3',
        type: 'neo-cli',
        network: 'mainnet',
        syncMode: 'full',
        version: '3.9.2',
        ports: { rpc: 10332, p2p: 10333 },
        paths: { base: '/tmp/node-1', data: '/tmp/node-1/data', logs: '/tmp/node-1/logs', config: '/tmp/node-1/config' },
        settings: {
          activeDataContextId: 'ctx-1',
          storageEngine: 'rocksdb',
          relay: true,
          maxConnections: 80,
          minPeers: 10,
          maxPeers: 60,
        },
        createdAt: 1,
        updatedAt: 1,
        process: { status: 'stopped' },
        plugins: [
          {
            id: 'StateService',
            version: '3.9.2',
            config: { AutoStart: true, FullState: false },
            installedAt: 1,
            enabled: true,
          },
          {
            id: 'RpcServer',
            version: '3.9.2',
            config: {},
            installedAt: 1,
            enabled: true,
          },
        ],
      },
    });

    expect(plan.requiresRestart).toBe(false);
    expect(plan.changes).toEqual([]);
  });

  it('uses active data context storage when node settings storage is stale', () => {
    const manager = createManager();

    const plan = manager.planRoleApplication({
      roleId: 'builtin-state',
      activeDataContext: {
        id: 'ctx-1',
        label: 'state-mainnet-rocksdb',
        storageEngine: 'rocksdb',
        syncStrategy: 'fast-sync',
        active: true,
      },
      node: {
        id: 'node-1',
        name: 'State Node',
        chain: 'n3',
        type: 'neo-cli',
        network: 'mainnet',
        syncMode: 'full',
        version: '3.9.2',
        ports: { rpc: 10332, p2p: 10333 },
        paths: { base: '/tmp/node-1', data: '/tmp/node-1/data', logs: '/tmp/node-1/logs', config: '/tmp/node-1/config' },
        settings: {
          activeDataContextId: 'ctx-1',
          relay: true,
          maxConnections: 80,
          minPeers: 10,
          maxPeers: 60,
        },
        createdAt: 1,
        updatedAt: 1,
        process: { status: 'stopped' },
        plugins: [
          {
            id: 'StateService',
            version: '3.9.2',
            config: { AutoStart: true, FullState: false },
            installedAt: 1,
            enabled: true,
          },
          {
            id: 'RpcServer',
            version: '3.9.2',
            config: {},
            installedAt: 1,
            enabled: true,
          },
        ],
      },
    });

    expect(plan.requiresRestart).toBe(false);
    expect(plan.changes).toEqual([]);
  });

  it('plans data context work for always-create roles even when active context matches', () => {
    const manager = createManager();
    const role = manager.createCustomRole({
      name: 'Archive Context',
      nodeTypes: ['neo-cli'],
      profile: {
        storageEngine: 'leveldb',
        dataContext: { mode: 'always-create', labelTemplate: 'archive-{network}-{storageEngine}' },
        sync: { strategy: 'full', allowCheckpoint: false },
      },
    });

    const plan = manager.planRoleApplication({
      roleId: role.id,
      activeDataContext: {
        id: 'ctx-1',
        label: 'archive-mainnet-leveldb',
        storageEngine: 'leveldb',
        syncStrategy: 'full',
        active: true,
      },
      node: {
        id: 'node-1',
        name: 'Archive Node',
        chain: 'n3',
        type: 'neo-cli',
        network: 'mainnet',
        syncMode: 'full',
        version: '3.9.2',
        ports: { rpc: 10332, p2p: 10333 },
        paths: { base: '/tmp/node-1', data: '/tmp/node-1/data', logs: '/tmp/node-1/logs', config: '/tmp/node-1/config' },
        settings: { activeDataContextId: 'ctx-1' },
        createdAt: 1,
        updatedAt: 1,
        process: { status: 'stopped' },
        plugins: [],
      },
    });

    expect(plan.requiresRestart).toBe(true);
    expect(plan.changes).toEqual([
      { type: 'data-context', summary: 'Create data context archive-mainnet-leveldb' },
    ]);
  });

  it('keeps returned and stored custom roles isolated from caller input mutation', () => {
    const manager = createManager();
    const profile = {
      storageEngine: 'leveldb' as const,
      plugins: [{ id: 'RpcServer' as const, enabled: true, config: { BindAddress: '127.0.0.1' } }],
    };
    const nodeTypes = ['neo-cli' as const];

    const created = manager.createCustomRole({ name: 'Mutable Input Role', nodeTypes, profile });
    nodeTypes.push('neo-go');
    profile.storageEngine = 'rocksdb';
    profile.plugins[0].config.BindAddress = '0.0.0.0';

    const stored = manager.getRole(created.id);

    expect(created.nodeTypes).toEqual(['neo-cli']);
    expect(created.profile.storageEngine).toBe('leveldb');
    expect(created.profile.plugins?.[0]?.config?.BindAddress).toBe('127.0.0.1');
    expect(stored?.nodeTypes).toEqual(['neo-cli']);
    expect(stored?.profile.storageEngine).toBe('leveldb');
    expect(stored?.profile.plugins?.[0]?.config?.BindAddress).toBe('127.0.0.1');
  });
});
