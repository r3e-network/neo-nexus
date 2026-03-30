# User Friendliness: Smart Defaults + Progressive Disclosure

**Date:** 2026-03-30
**Approach:** Actionable error messages, progressive disclosure, environment-aware startup

## Overview

Improve user-friendliness across all surfaces (API, frontend, startup) so the system works for beginners without annoying experts. The core pattern: show simple, actionable information by default with details available on demand.

Three phases, each independently shippable:

1. **API error response format + error catalog** — backend foundation
2. **Frontend progressive disclosure** — UI consumes the new format
3. **Startup experience** — environment-aware, context-sensitive output

---

## Phase 1: API Error Response Format

### Enhanced Error Shape

Current:
```json
{ "error": "Cannot update configuration while node is running" }
```

New:
```json
{
  "error": "Cannot update configuration while node is running",
  "code": "NODE_RUNNING",
  "suggestion": "Stop the node first, then retry the update.",
  "status": 400
}
```

- `error` — unchanged, keeps backwards compatibility
- `code` — machine-readable, lets frontend show contextual actions
- `suggestion` — plain-language next step, always present on 4xx errors
- `status` — HTTP status mirrored in body

### Implementation: `src/api/errors.ts`

New file exporting:

```typescript
export class ApiError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly suggestion: string,
    public readonly status: number = 400,
  ) {
    super(message);
  }
}
```

Plus a catalog object mapping codes to `{ message, suggestion, status }` with factory functions:

```typescript
export const Errors = {
  nodeNotFound: (id?: string) =>
    new ApiError("NODE_NOT_FOUND", `Node${id ? ` ${id}` : ""} not found`,
      "Check the node ID — it may have been deleted. Use GET /api/nodes to list active nodes.", 404),
  nodeRunning: () =>
    new ApiError("NODE_RUNNING", "Cannot update configuration while node is running",
      "Stop the node first, then retry the update."),
  // ... etc for all codes
};
```

### Error Catalog

#### Node Operations
| Code | Message | Suggestion |
|------|---------|------------|
| `NODE_NOT_FOUND` | Node not found | Check the node ID — it may have been deleted. Use GET /api/nodes to list active nodes. |
| `NODE_RUNNING` | Cannot update configuration while node is running | Stop the node first, then retry the update. |
| `NODE_ALREADY_RUNNING` | Node is already running | This node is already started. Use restart if you want to cycle it. |
| `NODE_NOT_RUNNING` | Node is not running | The node is already stopped. Use start to launch it. |

#### Validation
| Code | Message | Suggestion |
|------|---------|------------|
| `MISSING_FIELDS` | Missing required fields: name, type, network | Provide all three fields. Type must be "neo-cli" or "neo-go", network must be "mainnet" or "testnet". |
| `MISSING_IMPORT_FIELDS` | Missing required fields: name, existingPath | Provide a display name and the filesystem path to the existing node installation. |
| `NAME_EXISTS` | Node name already exists | Choose a different name — each node must have a unique display name. |
| `INVALID_TYPE` | Invalid node type | Supported types are "neo-cli" (C# reference node) and "neo-go" (Go alternative). |
| `PATH_BLOCKED` | Access to path {path} is not permitted | Paths must be under /home, /opt, or /var/lib. System directories are blocked for safety. |
| `PATH_NOT_ALLOWED` | Path must be under an allowed directory | Allowed directories: /home, /opt, /var/lib, and the NeoNexus data directory. |

#### Secure Signer
| Code | Message | Suggestion |
|------|---------|------------|
| `SIGNER_REQUIRES_PROFILE` | Secure signer protection requires a signer profile | Create a signer profile in Settings > Secure Signers first, then reference its ID here. |
| `SIGNER_NEO_CLI_ONLY` | Secure signer protection currently requires a neo-cli node | Only neo-cli nodes support the SignClient plugin. Switch to neo-cli or use standard wallet mode. |
| `SIGNER_NOT_AVAILABLE` | Secure signer profile {id} is not available | The profile may be disabled or deleted. Check Settings > Secure Signers to verify it is active. |

#### Authentication
| Code | Message | Suggestion |
|------|---------|------------|
| `NO_TOKEN` | No token provided | Include a Bearer token in the Authorization header. Log in via POST /api/auth/login to get one. |
| `TOKEN_INVALID` | Invalid or expired token | Your session has expired. Log in again to get a fresh token. |
| `SESSION_INVALID` | Session expired or invalid | Your session was invalidated (password change or admin action). Please log in again. |
| `INVALID_CREDENTIALS` | Invalid credentials | Username or password is incorrect. Default credentials are admin/admin if this is a fresh install. |

#### Detection & Import
| Code | Message | Suggestion |
|------|---------|------------|
| `DETECTION_FAILED` | Could not detect valid node installation at {path} | The path must contain a neo-cli (config.json + binary) or neo-go (config.yaml/protocol.yml + binary) installation. |
| `DETECTION_NOT_FOUND` | No valid node installation detected at the specified path | No recognizable node files found. Verify the path points to the directory containing the node binary and config files. |
| `IMPORT_INVALID` | Invalid node configuration: {errors} | The installation was detected but has issues. Check that data and config paths exist and ports are in the valid range (1-65535). |
| `PATH_NOT_FOUND` | Path does not exist: {path} | The directory does not exist on this machine. Double-check the path for typos. |

#### Ports & Resources
| Code | Message | Suggestion |
|------|---------|------------|
| `PORT_CONFLICT_NODE` | Port {port} ({name}) is already in use by another node | Another managed node is using this port. Let NeoNexus auto-assign ports, or pick a different range. |
| `PORT_CONFLICT_SYSTEM` | Port {port} ({name}) is already in use by another process | A process outside NeoNexus is binding this port. Run `lsof -i :{port}` to identify it. |
| `NO_PORT_RANGE` | No available port range found | All port slots are taken (max 100 nodes). Delete unused nodes to free up port ranges. |
| `PLUGINS_CLI_ONLY` | Plugins are only supported for neo-cli nodes | neo-go has built-in equivalents for most plugins. Check the neo-go documentation for the feature you need. |

### Router Changes

Update `respondWithNodeError` in `src/api/routes/nodes.ts`:

```typescript
const respondWithNodeError = (res: Response, error: unknown) => {
  if (error instanceof ApiError) {
    return res.status(error.status).json({
      error: error.message,
      code: error.code,
      suggestion: error.suggestion,
      status: error.status,
    });
  }
  // Fallback for non-ApiError exceptions (unchanged behavior)
  const message = error instanceof Error ? error.message : String(error);
  // ... existing regex matching for status codes
};
```

Route handlers throw `ApiError` instances instead of plain `Error`:

```typescript
// Before:
if (!request.name || !request.type || !request.network) {
  return res.status(400).json({ error: 'Missing required fields: name, type, network' });
}

// After:
if (!request.name || !request.type || !request.network) {
  throw Errors.missingFields();
}
```

Apply `ApiError` to: `nodes.ts`, `auth.ts` routes, `secure-signers.ts` routes, `servers.ts` routes, `system.ts` routes. The auth middleware (`auth.ts`) also gets updated to throw `ApiError`.

### Shared `respondWithApiError` Helper

Extract from each router into a shared utility at `src/api/respond.ts` so all routers use the same error response format. Each router's `respondWithNodeError` / `respondWithError` pattern is replaced by importing the shared helper.

---

## Phase 2: Frontend Progressive Disclosure

### Enhanced FeedbackBanner

Update `web/src/components/FeedbackBanner.tsx` to consume the new API error shape:

```typescript
interface FeedbackBannerProps {
  error?: string;
  suggestion?: string;
  errorCode?: string;
  success?: string;
  actions?: Array<{ label: string; onClick: () => void }>;
}
```

Behavior:
- Shows `error` message prominently (unchanged)
- Shows `suggestion` below the error in lighter text — this is the actionable hint
- If `errorCode` matches a known contextual action, renders an inline button:
  - `NODE_RUNNING` → "Stop Node" button
  - `SIGNER_REQUIRES_PROFILE` → "Go to Secure Signers" link
  - `TOKEN_INVALID` / `SESSION_INVALID` → "Log In" button
- `actions` prop allows page-specific overrides
- Suggestion is collapsible for experts who don't need it (persisted in localStorage as a preference)

### API Client Update

Update `web/src/utils/api.ts` to parse the enhanced response:

```typescript
interface ApiErrorResponse {
  error: string;
  code?: string;
  suggestion?: string;
  status?: number;
}

class ApiRequestError extends Error {
  constructor(
    message: string,
    public readonly code?: string,
    public readonly suggestion?: string,
    public readonly status?: number,
  ) {
    super(message);
  }
}
```

The `request()` function throws `ApiRequestError` instead of plain `Error`, preserving `code` and `suggestion` from the response body.

### Form Pages

Each form page passes the new fields to `FeedbackBanner`:

```typescript
catch (err) {
  if (err instanceof ApiRequestError) {
    setError(err.message);
    setSuggestion(err.suggestion);
    setErrorCode(err.code);
  } else {
    setError(err instanceof Error ? err.message : 'An unexpected error occurred');
  }
}
```

Affected pages: `CreateNode.tsx`, `ImportNode.tsx`, `Login.tsx`, `Setup.tsx`, `PasswordSection.tsx`, and any settings page with mutations.

### Enhanced Empty States

Update `EmptyState` component to support multiple actions:

```typescript
interface EmptyStateProps {
  icon: React.ElementType;
  title: string;
  description?: string;
  actions?: Array<{ label: string; href?: string; onClick?: () => void; variant?: "primary" | "secondary" }>;
}
```

Updated empty states:

| Page | Title | Actions |
|------|-------|---------|
| Dashboard (no nodes) | No nodes yet | **Create Node** (primary), **Import Existing** (secondary) |
| Nodes list (empty) | No nodes managed | **Create Node** (primary), **Import Existing** (secondary) |
| Integrations (none) | No integrations configured | **Browse Integrations** (primary) |
| Logs (no entries) | No logs yet | Start a node to begin collecting logs |
| Servers (none) | No remote servers | **Add Server** (primary) |

### Node Detail: Progressive Disclosure

Add a "Details" toggle on the node detail page. Default view shows:
- Status, block height, peers, sync progress
- Start/Stop/Restart buttons

Expanded view (toggled) adds:
- Ports (RPC, P2P, WebSocket, Metrics)
- File paths (base, data, logs, config)
- Raw configuration JSON
- Version and creation date

The toggle state persists in localStorage.

---

## Phase 3: Startup Experience

### Environment Detection

Add `src/utils/environment.ts`:

```typescript
interface RuntimeEnvironment {
  isDocker: boolean;
  isSystemd: boolean;
  isFirstRun: boolean;
  nodeCount: number;
  runningCount: number;
  hasDefaultPassword: boolean;
}
```

Detection methods:
- Docker: check `/.dockerenv` existence or `/proc/1/cgroup` contains "docker"
- Systemd: check `INVOCATION_ID` env var
- First run: check if users table is empty (before default admin creation)
- Node/running counts: query database after initialization

### Startup Output Redesign

Replace emoji-heavy scattered `console.log` calls with a structured `printStartupBanner()` function in `src/utils/startup.ts`.

**First run:**
```
NeoNexus v2.2.0

  Getting started:
    1. Open http://localhost:8080
    2. Create your admin account
    3. Deploy your first Neo node

  Data: ~/.neonexus
```

**Returning user:**
```
NeoNexus v2.2.0

  URL:   http://localhost:8080
  Nodes: 3 managed (2 running)
  Data:  ~/.neonexus
```

**With security warnings (only when relevant):**
```
NeoNexus v2.2.0

  URL:   http://localhost:8080
  Nodes: 3 managed (2 running)
  Data:  ~/.neonexus

  Warnings:
    - Default admin password is still in use. Change it in Settings.
    - Server is bound to all interfaces (0.0.0.0). Ensure firewall is configured.
```

Design principles:
- No emojis in terminal output (they render inconsistently across terminals)
- Indented key-value pairs for scannability
- Warnings section only appears when there are warnings
- Docker environments suppress firewall hints (container networking)
- Systemd environments suppress "Ctrl+C to stop" hints

### Graceful Shutdown

Replace current shutdown messages with:
```
Shutting down... stopped 2 nodes. Goodbye.
```

Single line, includes the count of nodes that were stopped.

---

## Files Changed (estimated)

### Phase 1 (API errors)
| File | Change |
|------|--------|
| `src/api/errors.ts` | **New** — ApiError class + error catalog |
| `src/api/respond.ts` | **New** — shared respondWithApiError helper |
| `src/api/routes/nodes.ts` | Throw ApiError, use shared helper |
| `src/api/routes/auth.ts` | Throw ApiError, use shared helper |
| `src/api/routes/secure-signers.ts` | Throw ApiError, use shared helper |
| `src/api/routes/servers.ts` | Throw ApiError, use shared helper |
| `src/api/routes/system.ts` | Throw ApiError, use shared helper |
| `src/api/middleware/auth.ts` | Throw ApiError for auth failures |
| `src/core/NodeManager.ts` | Throw ApiError in public methods |
| `src/core/PortManager.ts` | Throw ApiError for port conflicts |
| `src/core/NodeDetector.ts` | Throw ApiError for detection failures |
| Tests | Update error expectations |

### Phase 2 (Frontend)
| File | Change |
|------|--------|
| `web/src/utils/api.ts` | Parse enhanced error response, ApiRequestError class |
| `web/src/components/FeedbackBanner.tsx` | Add suggestion, code, actions props |
| `web/src/components/EmptyState.tsx` | Support multiple actions |
| `web/src/pages/CreateNode.tsx` | Pass suggestion/code to banner |
| `web/src/pages/ImportNode.tsx` | Pass suggestion/code to banner |
| `web/src/pages/Login.tsx` | Pass suggestion/code to banner |
| `web/src/pages/Setup.tsx` | Pass suggestion/code to banner |
| `web/src/pages/settings/PasswordSection.tsx` | Pass suggestion/code to banner |
| `web/src/pages/Dashboard.tsx` | Enhanced empty state, dual actions |
| `web/src/pages/NodeDetail.tsx` | Progressive disclosure toggle |

### Phase 3 (Startup)
| File | Change |
|------|--------|
| `src/utils/environment.ts` | **New** — runtime environment detection |
| `src/utils/startup.ts` | **New** — structured banner printer |
| `src/index.ts` | Replace console.log calls with printStartupBanner |
| `src/server.ts` | Remove startup logging (moved to startup.ts) |
| `src/database/schema.ts` | Remove inline credential logging |

---

## Non-Goals

- No onboarding wizard or step-by-step tutorial (Approach 3 scope)
- No tooltips or contextual help popovers
- No changes to WebSocket message format
- No changes to the public dashboard
- No i18n/localization
- No terminal coloring library (keep zero dependencies for output)
