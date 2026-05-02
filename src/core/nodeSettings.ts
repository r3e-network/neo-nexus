import type { ImportedNodeOwnershipMode, NodeSettings } from '../types/index';

export function stripReservedImportSettings(settings?: NodeSettings): NodeSettings | undefined {
  if (!settings) {
    return undefined;
  }

  const { import: _reservedImport, ...mutableSettings } = settings;
  return mutableSettings;
}

export function normalizeImportedOwnershipMode(mode: unknown): ImportedNodeOwnershipMode {
  if (mode === 'managed-config' || mode === 'managed-process') {
    return mode;
  }
  return 'observe-only';
}
