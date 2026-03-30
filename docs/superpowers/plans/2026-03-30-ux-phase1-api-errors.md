# UX Phase 1: API Error Response Format — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all terse API error messages with structured responses containing machine-readable codes and actionable suggestions.

**Architecture:** New `ApiError` class with code/suggestion fields, a shared `respondWithApiError` helper used by all routers, and factory functions for every known error. Core classes (`NodeManager`, `PortManager`, `NodeDetector`) throw `ApiError` directly. Routers throw `ApiError` for inline validation. The shared helper serializes to the enhanced JSON format.

**Tech Stack:** TypeScript, Express, Vitest

**Spec:** `docs/superpowers/specs/2026-03-30-user-friendliness-design.md` (Phase 1)

---

### Task 1: ApiError Class

**Files:**
- Create: `src/api/errors.ts`
- Create: `tests/unit/api-errors.test.ts`

- [ ] **Step 1: Write failing test for ApiError class**

```typescript
// tests/unit/api-errors.test.ts
import { describe, expect, it } from "vitest";
import { ApiError } from "../../src/api/errors";

describe("ApiError", () => {
  it("carries code, message, suggestion, and status", () => {
    const err = new ApiError("NODE_NOT_FOUND", "Node not found",
      "Check the node ID.", 404);

    expect(err).toBeInstanceOf(Error);
    expect(err.code).toBe("NODE_NOT_FOUND");
    expect(err.message).toBe("Node not found");
    expect(err.suggestion).toBe("Check the node ID.");
    expect(err.status).toBe(404);
  });

  it("defaults status to 400", () => {
    const err = new ApiError("MISSING_FIELDS", "Missing fields", "Provide all fields.");
    expect(err.status).toBe(400);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npx vitest run tests/unit/api-errors.test.ts`
Expected: FAIL — cannot resolve `../../src/api/errors`

- [ ] **Step 3: Implement ApiError class**

```typescript
// src/api/errors.ts
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

- [ ] **Step 4: Run test to verify it passes**

Run: `npx vitest run tests/unit/api-errors.test.ts`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add src/api/errors.ts tests/unit/api-errors.test.ts
git commit -m "feat(api): add ApiError class with code and suggestion fields"
```

---

### Task 2: Error Catalog

**Files:**
- Modify: `src/api/errors.ts`
- Modify: `tests/unit/api-errors.test.ts`

- [ ] **Step 1: Write failing tests for error factories**

Append to `tests/unit/api-errors.test.ts`:

```typescript
import { ApiError, Errors } from "../../src/api/errors";

describe("Errors catalog", () => {
  it("nodeNotFound returns 404 with suggestion", () => {
    const err = Errors.nodeNotFound("node-abc");
    expect(err.code).toBe("NODE_NOT_FOUND");
    expect(err.message).toBe("Node node-abc not found");
    expect(err.status).toBe(404);
    expect(err.suggestion).toContain("GET /api/nodes");
  });

  it("nodeNotFound works without an id", () => {
    const err = Errors.nodeNotFound();
    expect(err.message).toBe("Node not found");
  });

  it("missingFields lists the required fields", () => {
    const err = Errors.missingFields("name", "type", "network");
    expect(err.code).toBe("MISSING_FIELDS");
    expect(err.message).toContain("name, type, network");
    expect(err.status).toBe(400);
  });

  it("every factory returns an ApiError", () => {
    const factories = [
      () => Errors.nodeNotFound(),
      () => Errors.nodeRunning(),
      () => Errors.nodeAlreadyRunning(),
      () => Errors.nodeNotRunning(),
      () => Errors.missingFields("a"),
      () => Errors.nameExists("x"),
      () => Errors.invalidCredentials(),
      () => Errors.noToken(),
      () => Errors.tokenInvalid(),
      () => Errors.sessionInvalid(),
      () => Errors.pathBlocked("/etc"),
      () => Errors.pathNotAllowed(),
      () => Errors.pathNotFound("/x"),
      () => Errors.detectionFailed("/x"),
      () => Errors.detectionNotFound(),
      () => Errors.importInvalid(["bad port"]),
      () => Errors.signerRequiresProfile(),
      () => Errors.signerNeoCliOnly(),
      () => Errors.signerNotAvailable("s1"),
      () => Errors.portConflictNode(10332, "RPC"),
      () => Errors.portConflictSystem(10332, "RPC"),
      () => Errors.noPortRange(),
      () => Errors.pluginsCliOnly(),
      () => Errors.setupCompleted(),
      () => Errors.credentialsRequired(),
      () => Errors.adminRequired(),
      () => Errors.passwordRequired(),
      () => Errors.cannotDeleteSelf(),
      () => Errors.notAuthenticated(),
      () => Errors.missingField("path"),
      () => Errors.signerProfileNotFound(),
      () => Errors.signerFieldsRequired(),
      () => Errors.serverFieldsRequired(),
      () => Errors.snapshotRequired(),
    ];

    for (const factory of factories) {
      const err = factory();
      expect(err).toBeInstanceOf(ApiError);
      expect(err.code).toBeTruthy();
      expect(err.suggestion).toBeTruthy();
    }
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npx vitest run tests/unit/api-errors.test.ts`
Expected: FAIL — `Errors` is not exported

- [ ] **Step 3: Implement all error factories**

Append to `src/api/errors.ts`:

```typescript
export const Errors = {
  // Node operations
  nodeNotFound: (id?: string) =>
    new ApiError("NODE_NOT_FOUND",
      id ? `Node ${id} not found` : "Node not found",
      "Check the node ID — it may have been deleted. Use GET /api/nodes to list active nodes.", 404),
  nodeRunning: () =>
    new ApiError("NODE_RUNNING",
      "Cannot update configuration while node is running",
      "Stop the node first, then retry the update."),
  nodeAlreadyRunning: () =>
    new ApiError("NODE_ALREADY_RUNNING",
      "Node is already running",
      "This node is already started. Use restart if you want to cycle it."),
  nodeNotRunning: () =>
    new ApiError("NODE_NOT_RUNNING",
      "Node is not running",
      "The node is already stopped. Use start to launch it."),

  // Validation
  missingFields: (...fields: string[]) =>
    new ApiError("MISSING_FIELDS",
      `Missing required fields: ${fields.join(", ")}`,
      `Provide all required fields. Type must be "neo-cli" or "neo-go", network must be "mainnet" or "testnet".`),
  missingField: (field: string) =>
    new ApiError("MISSING_FIELDS",
      `Missing required field: ${field}`,
      `The "${field}" field is required.`),
  nameExists: (name: string) =>
    new ApiError("NAME_EXISTS",
      `Node name "${name}" already exists`,
      "Choose a different name — each node must have a unique display name."),
  pathBlocked: (path: string) =>
    new ApiError("PATH_BLOCKED",
      `Access to path ${path} is not permitted`,
      "Paths must be under /home, /opt, or /var/lib. System directories are blocked for safety."),
  pathNotAllowed: () =>
    new ApiError("PATH_NOT_ALLOWED",
      "Path must be under an allowed directory",
      "Allowed directories: /home, /opt, /var/lib, and the NeoNexus data directory."),
  pathNotFound: (path: string) =>
    new ApiError("PATH_NOT_FOUND",
      `Path does not exist: ${path}`,
      "The directory does not exist on this machine. Double-check the path for typos.", 404),

  // Detection & Import
  detectionFailed: (path: string) =>
    new ApiError("DETECTION_FAILED",
      `Could not detect valid node installation at ${path}. Make sure the path contains a valid neo-cli or neo-go installation.`,
      "The path must contain a neo-cli (config.json + binary) or neo-go (config.yaml/protocol.yml + binary) installation."),
  detectionNotFound: () =>
    new ApiError("DETECTION_NOT_FOUND",
      "No valid node installation detected at the specified path",
      "No recognizable node files found. Verify the path points to the directory containing the node binary and config files.", 404),
  importInvalid: (errors: string[]) =>
    new ApiError("IMPORT_INVALID",
      `Invalid node configuration: ${errors.join(", ")}`,
      "The installation was detected but has issues. Check that data and config paths exist and ports are in the valid range (1-65535)."),

  // Secure signer
  signerRequiresProfile: () =>
    new ApiError("SIGNER_REQUIRES_PROFILE",
      "Secure signer protection requires a signer profile",
      "Create a signer profile in Settings > Secure Signers first, then reference its ID here."),
  signerNeoCliOnly: () =>
    new ApiError("SIGNER_NEO_CLI_ONLY",
      "Secure signer protection currently requires a neo-cli node with SignClient support",
      "Only neo-cli nodes support the SignClient plugin. Switch to neo-cli or use standard wallet mode."),
  signerNotAvailable: (id: string) =>
    new ApiError("SIGNER_NOT_AVAILABLE",
      `Secure signer profile ${id} is not available`,
      "The profile may be disabled or deleted. Check Settings > Secure Signers to verify it is active."),
  signerProfileNotFound: () =>
    new ApiError("SIGNER_NOT_FOUND",
      "Secure signer profile not found",
      "The requested signer profile does not exist. Check the profile ID.", 404),
  signerFieldsRequired: () =>
    new ApiError("MISSING_FIELDS",
      "Missing required fields: name, mode, endpoint",
      "Provide a name, the signing mode (e.g. nitro), and the endpoint URL."),

  // Auth
  noToken: () =>
    new ApiError("NO_TOKEN",
      "No token provided",
      "Include a Bearer token in the Authorization header. Log in via POST /api/auth/login to get one.", 401),
  tokenInvalid: () =>
    new ApiError("TOKEN_INVALID",
      "Invalid or expired token",
      "Your session has expired. Log in again to get a fresh token.", 401),
  sessionInvalid: () =>
    new ApiError("SESSION_INVALID",
      "Session expired or invalid",
      "Your session was invalidated (password change or admin action). Please log in again.", 401),
  invalidCredentials: () =>
    new ApiError("INVALID_CREDENTIALS",
      "Invalid credentials",
      "Username or password is incorrect. Default credentials are admin/admin if this is a fresh install.", 401),
  notAuthenticated: () =>
    new ApiError("NOT_AUTHENTICATED",
      "Not authenticated",
      "You must be logged in to access this resource.", 401),
  credentialsRequired: () =>
    new ApiError("CREDENTIALS_REQUIRED",
      "Username and password are required",
      "Both username and password must be provided."),
  passwordRequired: () =>
    new ApiError("PASSWORD_REQUIRED",
      "Current and new password are required",
      "Provide both your current password and the new password."),
  adminRequired: () =>
    new ApiError("ADMIN_REQUIRED",
      "Admin access required",
      "This action requires administrator privileges.", 403),
  setupCompleted: () =>
    new ApiError("SETUP_COMPLETED",
      "Setup already completed. Use /register to create new users.",
      "The initial admin account has already been created. Ask an admin to register additional users.", 403),
  cannotDeleteSelf: () =>
    new ApiError("CANNOT_DELETE_SELF",
      "Cannot delete your own account",
      "Ask another admin to delete your account, or deactivate it instead."),

  // Ports
  portConflictNode: (port: number, name: string) =>
    new ApiError("PORT_CONFLICT_NODE",
      `Port ${port} (${name}) is already in use by another node`,
      "Another managed node is using this port. Let NeoNexus auto-assign ports, or pick a different range."),
  portConflictSystem: (port: number, name: string) =>
    new ApiError("PORT_CONFLICT_SYSTEM",
      `Port ${port} (${name}) is already in use by another process`,
      `A process outside NeoNexus is binding this port. Run \`lsof -i :${port}\` to identify it.`),
  noPortRange: () =>
    new ApiError("NO_PORT_RANGE",
      "No available port range found",
      "All port slots are taken (max 100 nodes). Delete unused nodes to free up port ranges."),

  // Plugins
  pluginsCliOnly: () =>
    new ApiError("PLUGINS_CLI_ONLY",
      "Plugins are only supported for neo-cli nodes",
      "neo-go has built-in equivalents for most plugins. Check the neo-go documentation for the feature you need."),

  // Servers
  serverFieldsRequired: () =>
    new ApiError("MISSING_FIELDS",
      "Missing required fields: name, baseUrl",
      "Provide a display name and the base URL of the remote NeoNexus instance."),

  // System
  snapshotRequired: () =>
    new ApiError("SNAPSHOT_REQUIRED",
      "A valid snapshot payload is required",
      "POST a JSON body containing the configuration snapshot from a previous export."),
};
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npx vitest run tests/unit/api-errors.test.ts`
Expected: PASS (all tests)

- [ ] **Step 5: Commit**

```bash
git add src/api/errors.ts tests/unit/api-errors.test.ts
git commit -m "feat(api): add error catalog with factory functions for all known errors"
```

---

### Task 3: Shared respondWithApiError Helper

**Files:**
- Create: `src/api/respond.ts`
- Create: `tests/unit/api-respond.test.ts`

- [ ] **Step 1: Write failing tests for the helper**

```typescript
// tests/unit/api-respond.test.ts
import { describe, expect, it, vi } from "vitest";
import { respondWithApiError } from "../../src/api/respond";
import { ApiError } from "../../src/api/errors";

function mockResponse() {
  const json = vi.fn();
  const status = vi.fn(() => ({ json }));
  return { status, json, raw: { status, json } as unknown };
}

describe("respondWithApiError", () => {
  it("serializes an ApiError with all fields", () => {
    const res = mockResponse();
    const err = new ApiError("NODE_NOT_FOUND", "Node not found", "Check the ID.", 404);

    respondWithApiError(res.raw as never, err);

    expect(res.status).toHaveBeenCalledWith(404);
    expect(res.json).toHaveBeenCalledWith({
      error: "Node not found",
      code: "NODE_NOT_FOUND",
      suggestion: "Check the ID.",
      status: 404,
    });
  });

  it("falls back to 500 for plain Error", () => {
    const res = mockResponse();
    respondWithApiError(res.raw as never, new Error("something broke"));

    expect(res.status).toHaveBeenCalledWith(500);
    expect(res.json).toHaveBeenCalledWith(
      expect.objectContaining({ error: "something broke", status: 500 }),
    );
  });

  it("falls back to 500 for non-Error values", () => {
    const res = mockResponse();
    respondWithApiError(res.raw as never, "string error");

    expect(res.status).toHaveBeenCalledWith(500);
    expect(res.json).toHaveBeenCalledWith(
      expect.objectContaining({ error: "string error", status: 500 }),
    );
  });

  it("maps plain Error 'not found' messages to 404", () => {
    const res = mockResponse();
    respondWithApiError(res.raw as never, new Error("Node xyz not found"));

    expect(res.status).toHaveBeenCalledWith(404);
  });

  it("maps plain Error validation messages to 400", () => {
    const res = mockResponse();
    respondWithApiError(res.raw as never, new Error("Cannot update while running"));

    expect(res.status).toHaveBeenCalledWith(400);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npx vitest run tests/unit/api-respond.test.ts`
Expected: FAIL — cannot resolve `../../src/api/respond`

- [ ] **Step 3: Implement the helper**

```typescript
// src/api/respond.ts
import type { Response } from "express";
import { ApiError } from "./errors";

export function respondWithApiError(res: Response, error: unknown): void {
  if (error instanceof ApiError) {
    res.status(error.status).json({
      error: error.message,
      code: error.code,
      suggestion: error.suggestion,
      status: error.status,
    });
    return;
  }

  const message = error instanceof Error ? error.message : String(error);
  let status = 500;

  if (/not found/i.test(message)) {
    status = 404;
  } else if (
    /missing required fields|invalid|already exists|requires|cannot|not available|unsupported/i.test(message)
  ) {
    status = 400;
  }

  res.status(status).json({
    error: message,
    code: "INTERNAL_ERROR",
    suggestion: "An unexpected error occurred. If this persists, check the server logs.",
    status,
  });
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `npx vitest run tests/unit/api-respond.test.ts`
Expected: PASS (5 tests)

- [ ] **Step 5: Commit**

```bash
git add src/api/respond.ts tests/unit/api-respond.test.ts
git commit -m "feat(api): add shared respondWithApiError helper"
```

---

### Task 4: Migrate Auth Middleware

**Files:**
- Modify: `src/api/middleware/auth.ts`
- Modify: `tests/unit/auth.middleware.test.ts`

- [ ] **Step 1: Update auth middleware to throw ApiError**

In `src/api/middleware/auth.ts`, add import and replace the three error returns:

```typescript
import { Errors } from '../api/errors';
import { respondWithApiError } from '../api/respond';
```

Wait — the middleware returns `res.status().json()` directly, not throwing. Since it's middleware (not inside a try/catch), we keep returning but use the `respondWithApiError` helper or construct responses inline using `Errors`:

Replace:
```typescript
return res.status(401).json({ error: "No token provided" });
```
With:
```typescript
const err = Errors.noToken();
return res.status(err.status).json({
  error: err.message, code: err.code, suggestion: err.suggestion, status: err.status,
});
```

Apply the same pattern to all three returns in the middleware:
- `"No token provided"` → `Errors.noToken()`
- `"Session expired or invalid"` → `Errors.sessionInvalid()`
- `"Invalid or expired token"` → `Errors.tokenInvalid()`

- [ ] **Step 2: Update auth middleware test**

In `tests/unit/auth.middleware.test.ts`, update expectations to check for new fields. Read the current test file first, then update assertions to also verify `code` and `suggestion` in the response body.

- [ ] **Step 3: Run tests**

Run: `npx vitest run tests/unit/auth.middleware.test.ts`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/api/middleware/auth.ts tests/unit/auth.middleware.test.ts
git commit -m "feat(auth): return structured error responses from auth middleware"
```

---

### Task 5: Migrate Nodes Router

This is the largest router. It has its own `respondWithNodeError` which gets replaced by the shared helper, and ~18 inline error returns.

**Files:**
- Modify: `src/api/routes/nodes.ts`
- Modify: `tests/integration/nodes.router.actual.test.ts`

- [ ] **Step 1: Replace respondWithNodeError with shared helper and update validateNodePath**

In `src/api/routes/nodes.ts`:

Add imports:
```typescript
import { ApiError, Errors } from '../api/errors';  // add to existing imports
import { respondWithApiError } from '../api/respond';
```

Replace `validateNodePath` throws:
```typescript
// Line 14: throw new Error(`Access to path ${resolved} is not permitted`)
throw Errors.pathBlocked(resolved);

// Line 18: throw new Error(`Path must be under an allowed directory`)
throw Errors.pathNotAllowed();
```

Delete the entire `respondWithNodeError` function (lines 31-47) and replace all `respondWithNodeError(res, error)` calls with `respondWithApiError(res, error)`.

- [ ] **Step 2: Replace inline error returns with ApiError throws**

Replace each inline `res.status().json({ error })` with a throw. Since each route handler is wrapped in try/catch → `respondWithApiError`, throwing works:

```typescript
// POST / — missing fields (line 91-95)
if (!request.name || !request.type || !request.network) {
  throw Errors.missingFields("name", "type", "network");
}

// POST / — signer validation (line 97-100)
const secureSignerValidationError = validateSecureSignerRequest(request);
if (secureSignerValidationError) {
  throw secureSignerValidationError;  // already an ApiError (see step 3)
}

// POST /import — missing fields (line 115-119)
if (!request.name || !request.existingPath) {
  throw Errors.missingFields("name", "existingPath");
}

// POST /detect — missing path (line 135-137)
if (!path) {
  throw Errors.missingField("path");
}

// POST /detect — nothing detected (line 143-148)
if (!detected) {
  throw Errors.detectionNotFound();
}

// POST /scan — missing path (line 167-169)
if (!path) {
  throw Errors.missingField("path");
}

// GET /:id — not found (line 189-191)
if (!node) {
  throw Errors.nodeNotFound(req.params.id);
}

// PUT /:id — not found (line 203-205)
if (!existingNode) {
  throw Errors.nodeNotFound(req.params.id);
}

// GET /:id/logs — not found (line 273-275)
if (!node) {
  throw Errors.nodeNotFound(req.params.id);
}

// GET /:id/signer-health — not found (line 298-300)
if (!node) {
  throw Errors.nodeNotFound(req.params.id);
}

// POST /:id/storage/clean — not found (line 315-317)
if (!node) {
  throw Errors.nodeNotFound(req.params.id);
}

// GET /:id/config-audit — not found (line 332-334)
if (!node) {
  throw Errors.nodeNotFound(req.params.id);
}
```

- [ ] **Step 3: Update validateSecureSignerRequest to return ApiError**

Change return type from `string | null` to `ApiError | null`:

```typescript
const validateSecureSignerRequest = (
  request: Pick<CreateNodeRequest, "type" | "settings"> | Pick<UpdateNodeRequest, "settings">,
  existingNodeType?: CreateNodeRequest["type"],
): ApiError | null => {
  const keyProtection = request.settings?.keyProtection;
  if (keyProtection?.mode !== "secure-signer") {
    return null;
  }

  const nodeType = "type" in request && request.type ? request.type : existingNodeType;
  if (nodeType !== "neo-cli") {
    return Errors.signerNeoCliOnly();
  }

  if (!keyProtection.signerProfileId) {
    return Errors.signerRequiresProfile();
  }

  const profile = nodeManager.getSecureSignerManager().getProfile(keyProtection.signerProfileId);
  if (!profile || !profile.enabled) {
    return Errors.signerNotAvailable(keyProtection.signerProfileId);
  }

  return null;
};
```

- [ ] **Step 4: Update router integration tests**

In `tests/integration/nodes.router.actual.test.ts`, update assertions to check for `code` and `suggestion`:

```typescript
// Example — "rejects creation requests with missing required fields"
expect(response.body.code).toBe("MISSING_FIELDS");
expect(response.body.suggestion).toBeTruthy();

// Example — "returns 404 when a node does not exist"
expect(response.body.code).toBe("NODE_NOT_FOUND");
expect(response.body.suggestion).toBeTruthy();

// Example — "rejects secure signer creation without a signer profile"
expect(response.body.code).toBe("SIGNER_REQUIRES_PROFILE");

// Example — "rejects secure signer creation for neo-go nodes"
expect(response.body.code).toBe("SIGNER_NEO_CLI_ONLY");

// Example — "rejects secure signer creation when profile is missing"
expect(response.body.code).toBe("SIGNER_NOT_AVAILABLE");
```

Add `code` and `suggestion` assertions to every existing error-path test. The `error` message assertions stay — they verify backwards compatibility.

- [ ] **Step 5: Run all affected tests**

Run: `npx vitest run tests/integration/nodes.router.actual.test.ts tests/unit/api-errors.test.ts tests/unit/api-respond.test.ts`
Expected: PASS (all)

- [ ] **Step 6: Run full test suite**

Run: `npx vitest run`
Expected: PASS — no regressions. Some tests in other files may reference error messages that are unchanged.

- [ ] **Step 7: Commit**

```bash
git add src/api/routes/nodes.ts tests/integration/nodes.router.actual.test.ts
git commit -m "feat(nodes): structured error responses with codes and suggestions"
```

---

### Task 6: Migrate Auth Router

**Files:**
- Modify: `src/api/routes/auth.ts`
- Modify: `tests/integration/auth.routes.test.ts`
- Modify: `tests/integration/auth.router.actual.test.ts`

- [ ] **Step 1: Read the current auth router and its tests**

Read `src/api/routes/auth.ts`, `tests/integration/auth.routes.test.ts`, and `tests/integration/auth.router.actual.test.ts` to understand the current error patterns.

- [ ] **Step 2: Add imports and replace error returns**

Add imports to `src/api/routes/auth.ts`:
```typescript
import { Errors } from '../api/errors';
import { respondWithApiError } from '../api/respond';
```

Replace all `res.status(4xx).json({ error: "..." })` calls. In each try/catch, replace the catch with `respondWithApiError(res, error)`. For inline validation, throw the appropriate `Errors` factory:

Key replacements:
- `"Setup already completed..."` → `throw Errors.setupCompleted()`
- `"Username and password are required"` → `throw Errors.credentialsRequired()`
- `"Invalid credentials"` → `throw Errors.invalidCredentials()`
- `"Admin access required"` → `throw Errors.adminRequired()`
- `"Current and new password are required"` → `throw Errors.passwordRequired()`
- `"Not authenticated"` → `throw Errors.notAuthenticated()`
- `"Cannot delete your own account"` → `throw Errors.cannotDeleteSelf()`
- All `catch (error)` blocks → `respondWithApiError(res, error)`

- [ ] **Step 3: Update auth tests to verify new fields**

Add `code` assertions to error-path tests in both auth test files. Keep existing `error` message assertions for backwards compatibility.

- [ ] **Step 4: Run tests**

Run: `npx vitest run tests/integration/auth.routes.test.ts tests/integration/auth.router.actual.test.ts`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/api/routes/auth.ts tests/integration/auth.routes.test.ts tests/integration/auth.router.actual.test.ts
git commit -m "feat(auth): structured error responses with codes and suggestions"
```

---

### Task 7: Migrate Remaining Routers (Secure Signers, Servers, System)

**Files:**
- Modify: `src/api/routes/secureSigners.ts`
- Modify: `src/api/routes/servers.ts`
- Modify: `src/api/routes/system.ts`
- Modify: relevant test files for each

- [ ] **Step 1: Read all three routers and their test files**

Read `src/api/routes/secureSigners.ts`, `src/api/routes/servers.ts`, `src/api/routes/system.ts` and their corresponding test files.

- [ ] **Step 2: Migrate secureSigners.ts**

Add imports and replace error patterns:
- `"Secure signer profile not found"` → `throw Errors.signerProfileNotFound()`
- `"Missing required fields: name, mode, endpoint"` → `throw Errors.signerFieldsRequired()`
- `"ciphertextBase64 is required"` → `throw Errors.missingField("ciphertextBase64")`
- All regex-based status detection (not found → 404, else 400) → `respondWithApiError(res, error)`
- All `catch` blocks → `respondWithApiError(res, error)`

- [ ] **Step 3: Migrate servers.ts**

- `"Missing required fields: name, baseUrl"` → `throw Errors.serverFieldsRequired()`
- All `catch` blocks → `respondWithApiError(res, error)`

- [ ] **Step 4: Migrate system.ts**

- `"A valid snapshot payload is required"` → `throw Errors.snapshotRequired()`
- All `catch` blocks → `respondWithApiError(res, error)`

- [ ] **Step 5: Update test files for all three routers**

Add `code` assertions to error-path tests. Keep existing `error` message assertions.

- [ ] **Step 6: Run all affected tests**

Run: `npx vitest run tests/integration/secure-signers.router.actual.test.ts tests/integration/servers.router.actual.test.ts tests/integration/system.router.actual.test.ts`
Expected: PASS

- [ ] **Step 7: Run full suite for regressions**

Run: `npx vitest run`
Expected: PASS (all 261+ tests)

- [ ] **Step 8: Commit**

```bash
git add src/api/routes/secureSigners.ts src/api/routes/servers.ts src/api/routes/system.ts tests/
git commit -m "feat(api): structured error responses in signers, servers, and system routers"
```

---

### Task 8: Migrate Core Classes

**Files:**
- Modify: `src/core/NodeManager.ts`
- Modify: `src/core/PortManager.ts`
- Modify: `src/core/NodeDetector.ts`
- Modify: relevant test files

- [ ] **Step 1: Migrate NodeManager throws**

Add import to `src/core/NodeManager.ts`:
```typescript
import { Errors } from '../api/errors';
```

Replace each `throw new Error(...)` in public methods:

```typescript
// importExistingNode
throw Errors.detectionFailed(request.existingPath);     // was: "Could not detect valid..."
throw Errors.importInvalid(validation.errors);           // was: "Invalid node configuration..."

// createNode
// Keep "Could not determine latest version" as plain Error (internal, not user-facing)

// updateNode
throw Errors.nodeNotFound(nodeId);                       // was: "Node ${nodeId} not found"
throw Errors.nodeRunning();                              // was: "Cannot update configuration..."

// deleteNode
throw Errors.nodeNotFound(nodeId);                       // was: "Node ${nodeId} not found"

// startNode
throw Errors.nodeNotFound(nodeId);                       // was: "Node ${nodeId} not found"
throw Errors.nodeAlreadyRunning();                       // was: "Node is already running"

// stopNode
throw Errors.nodeNotRunning();                           // was: "Node is not running"

// getStorageInfo
throw Errors.nodeNotFound(nodeId);                       // was: "Node ${nodeId} not found"

// installPlugin
throw Errors.nodeNotFound(nodeId);                       // was: "Node ${nodeId} not found"
throw Errors.pluginsCliOnly();                           // was: "Plugins are only supported..."
throw Errors.nodeRunning();                              // was: "Cannot install plugins..."
// Note: reuse nodeRunning — message says "Cannot update..." but suggestion says "Stop first"

// syncNodeSecureSigner
throw Errors.nodeNotFound(nodeId);                       // was: "Node ${nodeId} not found"
throw Errors.signerNotAvailable(signerProfileId!);       // was: "Secure signer profile..."

// assertSecureSignerCompatibility
throw Errors.signerRequiresProfile();                    // was: "Secure signer protection requires..."
throw Errors.signerNeoCliOnly();                         // was: "Secure signer protection currently..."
throw Errors.signerNotAvailable(signerProfileId);        // was: "Secure signer profile..."
```

- [ ] **Step 2: Migrate PortManager throws**

Add import to `src/core/PortManager.ts`:
```typescript
import { Errors } from '../api/errors';
```

Replace:
```typescript
// allocatePorts
throw Errors.portConflictNode(port, name);               // was: "Port ... already in use by another node"
throw Errors.portConflictSystem(port, name);             // was: "Port ... already in use by another process"

// findNextIndex
throw Errors.noPortRange();                              // was: "No available port range found..."
```

- [ ] **Step 3: Migrate NodeDetector throws**

Add import to `src/core/NodeDetector.ts`:
```typescript
import { Errors } from '../api/errors';
```

Replace in `detect()`:
```typescript
throw Errors.pathNotFound(basePath);                     // was: "Path does not exist: ${basePath}"
```

- [ ] **Step 4: Update tests that check error messages from core classes**

Update tests that assert on error messages from `NodeManager`, `PortManager`, and `NodeDetector`. The error messages are largely unchanged, but tests should also verify the error is an `ApiError`:

```typescript
// Example in NodeDetector test:
expect(() => NodeDetector.detect("/nonexistent")).toThrow("Path does not exist");
// Still passes because ApiError extends Error and message is the same
```

Most existing tests should pass without changes since `ApiError` extends `Error` and the messages are preserved. Run and fix any that break.

- [ ] **Step 5: Run full test suite**

Run: `npx vitest run`
Expected: PASS (all tests)

- [ ] **Step 6: Run TypeScript compiler**

Run: `npx tsc --noEmit`
Expected: clean (exit 0)

- [ ] **Step 7: Commit**

```bash
git add src/core/NodeManager.ts src/core/PortManager.ts src/core/NodeDetector.ts tests/
git commit -m "feat(core): throw ApiError with codes from NodeManager, PortManager, NodeDetector"
```

---

### Task 9: Final Verification

- [ ] **Step 1: Run full test suite**

Run: `npx vitest run`
Expected: ALL tests pass

- [ ] **Step 2: Run TypeScript compiler**

Run: `npx tsc --noEmit`
Expected: clean

- [ ] **Step 3: Manually verify enhanced response format**

Start the server and hit an error endpoint to verify the response shape:

```bash
curl -s http://localhost:8080/api/nodes/nonexistent | jq .
```

Expected:
```json
{
  "error": "Node not found",
  "code": "NODE_NOT_FOUND",
  "suggestion": "Check the node ID — it may have been deleted. Use GET /api/nodes to list active nodes.",
  "status": 404
}
```

- [ ] **Step 4: Verify backwards compatibility**

Confirm that `response.body.error` still contains the same message string as before for all error cases. Existing clients that only read `error` are unaffected.
