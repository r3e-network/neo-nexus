import { readFileSync } from 'node:fs';
import type {
  ConfigurationSnapshot,
  ConfigurationSnapshotNode,
  CreateNodeRequest,
  NodeInstance,
  PluginId,
} from '../types/index';
import { Errors } from '../api/errors';
import { stripReservedImportSettings } from './nodeSettings';

type MaybePromise<T> = T | Promise<T>;

export type ExportedConfigurationNode = Omit<NodeInstance, 'process' | 'metrics'>;

export interface ExportedConfigurationSnapshot {
  generatedAt: number;
  version: string;
  nodes: ExportedConfigurationNode[];
}

export interface RestoreConfigurationOptions {
  replaceExisting?: boolean;
}

export interface RestoreConfigurationResult {
  restoredCount: number;
  skippedCount: number;
  failedCount: number;
}

export interface NodeSnapshotServiceDependencies {
  getAllNodes(): NodeInstance[];
  createNode(request: CreateNodeRequest): Promise<NodeInstance>;
  installPlugin(nodeId: string, pluginId: PluginId, config?: Record<string, unknown>): Promise<void>;
  setPluginEnabled(nodeId: string, pluginId: PluginId, enabled: boolean): MaybePromise<void>;
  resetAllNodeData(): Promise<unknown>;
  onRollbackError?(error: unknown): void;
}

export class NodeSnapshotService {
  constructor(private readonly deps: NodeSnapshotServiceDependencies) {}

  exportConfiguration(): ExportedConfigurationSnapshot {
    const nodes = this.deps.getAllNodes().map(({ process: _process, metrics: _metrics, ...config }) => config);

    let version = '2.0.0';
    try {
      const pkg = JSON.parse(readFileSync(new URL('../../package.json', import.meta.url), 'utf-8')) as { version?: string };
      version = pkg.version || version;
    } catch {
      // Keep export available even if package metadata is unavailable.
    }

    return {
      generatedAt: Date.now(),
      version,
      nodes,
    };
  }

  async restoreConfiguration(
    snapshot: ConfigurationSnapshot,
    options: RestoreConfigurationOptions = {},
  ): Promise<RestoreConfigurationResult> {
    if (options.replaceExisting) {
      this.assertRestorableSnapshot(snapshot);
      const rollbackSnapshot = this.safeExportConfiguration();
      await this.deps.resetAllNodeData();

      try {
        return await this.restoreSnapshotNodes(snapshot, { rejectInvalid: true, failFast: true });
      } catch (error) {
        if (rollbackSnapshot) {
          await this.rollbackRestore(rollbackSnapshot);
        }
        throw error;
      }
    }

    return this.restoreSnapshotNodes(snapshot, { rejectInvalid: false, failFast: false });
  }

  private async restoreSnapshotNodes(
    snapshot: ConfigurationSnapshot,
    options: { rejectInvalid: boolean; failFast: boolean },
  ): Promise<RestoreConfigurationResult> {
    let restoredCount = 0;
    let skippedCount = 0;
    let failedCount = 0;

    for (const [index, node] of snapshot.nodes.entries()) {
      if (!this.isRestorableSnapshotNode(node)) {
        if (options.rejectInvalid) {
          throw Errors.snapshotInvalid(index);
        }
        skippedCount++;
        continue;
      }

      try {
        const restoredNode = await this.deps.createNode({
          name: node.name,
          type: node.type,
          network: node.network,
          syncMode: node.syncMode,
          version: node.version,
          customPorts: node.ports,
          settings: stripReservedImportSettings(node.settings),
        });

        for (const plugin of node.plugins ?? []) {
          await this.deps.installPlugin(restoredNode.id, plugin.id, plugin.config);
          if (plugin.enabled === false) {
            await this.deps.setPluginEnabled(restoredNode.id, plugin.id, false);
          }
        }

        restoredCount++;
      } catch (error) {
        if (options.failFast) {
          const message = error instanceof Error ? error.message : 'Unknown restore error';
          throw Errors.snapshotRestoreFailed(message);
        }
        failedCount++;
      }
    }

    return {
      restoredCount,
      skippedCount,
      failedCount,
    };
  }

  private assertRestorableSnapshot(snapshot: ConfigurationSnapshot): void {
    for (const [index, node] of snapshot.nodes.entries()) {
      if (!this.isRestorableSnapshotNode(node)) {
        throw Errors.snapshotInvalid(index);
      }
    }
  }

  private safeExportConfiguration(): ConfigurationSnapshot | null {
    try {
      return this.exportConfiguration();
    } catch {
      return null;
    }
  }

  private async rollbackRestore(snapshot: ConfigurationSnapshot): Promise<void> {
    try {
      await this.deps.resetAllNodeData();
      await this.restoreSnapshotNodes(snapshot, { rejectInvalid: false, failFast: false });
    } catch (rollbackError) {
      if (this.deps.onRollbackError) {
        this.deps.onRollbackError(rollbackError);
        return;
      }
      console.error('Failed to rollback configuration restore:', rollbackError);
    }
  }

  private isRestorableSnapshotNode(node: Partial<ConfigurationSnapshotNode>): node is ConfigurationSnapshotNode {
    return Boolean(node.name && node.type && node.network);
  }
}
