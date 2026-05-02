# NodeManager Structure Optimization Design

## Goal

Reduce `NodeManager` complexity without changing public behavior, API contracts, database schema, or frontend behavior.

## Scope

This pass extracts the imported-node process attachment and managed-path classification logic from `src/core/NodeManager.ts` into a focused helper module. `NodeManager` remains the orchestration boundary for database writes, node lifecycle calls, plugin operations, and event emission.

## Architecture

- `src/core/nodeProcessAttachment.ts` owns process/PID helper logic:
  - parse and de-duplicate `pgrep` output
  - validate process IDs
  - score candidate processes against a node record
  - decide whether an attached PID is active or stale
  - classify generated NeoNexus node paths
- `NodeManager` delegates to this module for path and process decisions while keeping persistence and side effects in place.
- The module depends only on node path helpers and lifecycle inspection functions, so its behavior can be tested directly.

## Testing

Add focused unit tests for the extracted helper module and keep existing `NodeManager.system-actions` tests as regression coverage for the end-to-end import attach flow. Final verification must run `npm run verify` and `npm run build:backend`.
