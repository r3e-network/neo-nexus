# UX Phase 2: Frontend Progressive Disclosure — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the React frontend consume the enhanced API error format (code, suggestion) and display actionable feedback with progressive disclosure.

**Architecture:** New `ApiRequestError` class in the API client preserves `code` and `suggestion` from responses. Enhanced `FeedbackBanner` shows suggestions and optional contextual action buttons. Enhanced `EmptyState` supports multiple actions. All form pages pass the new error fields through.

**Tech Stack:** React 19, TypeScript, TanStack Query, Tailwind CSS, Lucide icons

**Spec:** `docs/superpowers/specs/2026-03-30-user-friendliness-design.md` (Phase 2)

**No frontend test infrastructure exists.** Tasks are implementation + manual verification via `npm run build` in `web/`.

---

## File Structure

| File | Responsibility | Change |
|------|---------------|--------|
| `web/src/utils/api.ts` | API client, error parsing | Add `ApiRequestError` class, parse `code`/`suggestion` from responses |
| `web/src/components/FeedbackBanner.tsx` | Error/success display | Add `suggestion`, `code`, `actions` props; show suggestion text and contextual buttons |
| `web/src/components/EmptyState.tsx` | Empty state display | Support `actions` array instead of single `action` |
| `web/src/pages/CreateNode.tsx` | Create node form | Pass suggestion/code to FeedbackBanner |
| `web/src/pages/ImportNode.tsx` | Import node form | Pass suggestion/code to FeedbackBanner |
| `web/src/pages/Login.tsx` | Login form | Pass suggestion/code to FeedbackBanner |
| `web/src/pages/Setup.tsx` | First-time setup form | Pass suggestion/code to FeedbackBanner |
| `web/src/pages/settings/PasswordSection.tsx` | Change password form | Pass suggestion/code to FeedbackBanner |
| `web/src/pages/Dashboard.tsx` | Main dashboard | Dual-action empty state (Create + Import) |
| `web/src/pages/NodeDetail.tsx` | Node detail view | Progressive disclosure toggle for technical details |

---

### Task 1: ApiRequestError Class in API Client

**Files:**
- Modify: `web/src/utils/api.ts`

- [ ] **Step 1: Add ApiRequestError class**

At the top of `web/src/utils/api.ts`, after the existing imports, add:

```typescript
export class ApiRequestError extends Error {
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

- [ ] **Step 2: Update the error handling in the `request` function**

Find the existing error handling block (the `if (!response.ok)` section). Replace it:

```typescript
if (!response.ok) {
  let message = "API Error";
  let code: string | undefined;
  let suggestion: string | undefined;

  try {
    const body = (await response.json()) as {
      error?: string;
      code?: string;
      suggestion?: string;
    };
    message = body.error || message;
    code = body.code;
    suggestion = body.suggestion;
  } catch {
    // Response wasn't JSON
  }

  throw new ApiRequestError(message, code, suggestion, response.status);
}
```

- [ ] **Step 3: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds

- [ ] **Step 4: Commit**

```bash
git add web/src/utils/api.ts
git commit -m "feat(web): parse error code and suggestion from API responses"
```

---

### Task 2: Enhanced FeedbackBanner

**Files:**
- Modify: `web/src/components/FeedbackBanner.tsx`

- [ ] **Step 1: Rewrite FeedbackBanner with new props**

Replace the entire file content:

```tsx
import { AlertCircle, CheckCircle, ChevronDown, ChevronUp } from "lucide-react";
import { useState } from "react";

interface BannerAction {
  label: string;
  href?: string;
  onClick?: () => void;
}

interface FeedbackBannerProps {
  error?: string;
  suggestion?: string;
  code?: string;
  success?: string;
  actions?: BannerAction[];
}

export function FeedbackBanner({ error, suggestion, code, success, actions }: FeedbackBannerProps) {
  const [collapsed, setCollapsed] = useState(false);

  if (!error && !success) return null;

  const isError = !!error;
  const Icon = isError ? AlertCircle : CheckCircle;
  const borderColor = isError ? "border-red-500/20" : "border-emerald-500/20";
  const bgColor = isError ? "bg-red-500/10" : "bg-emerald-500/10";
  const textColor = isError ? "text-red-300" : "text-emerald-300";
  const subtextColor = isError ? "text-red-400/70" : "text-emerald-400/70";

  return (
    <div className={`${borderColor} ${bgColor} border rounded-lg p-4 mb-4 animate-fade-in`}>
      <div className="flex items-start gap-3">
        <Icon className={`w-5 h-5 ${textColor} shrink-0 mt-0.5`} />
        <div className="flex-1 min-w-0">
          <p className={`text-sm ${textColor}`}>{error || success}</p>

          {suggestion && !collapsed && (
            <p className={`text-sm ${subtextColor} mt-1`}>{suggestion}</p>
          )}

          {(suggestion || (actions && actions.length > 0)) && (
            <div className="flex items-center gap-3 mt-2 flex-wrap">
              {actions?.map((action) =>
                action.href ? (
                  <a
                    key={action.label}
                    href={action.href}
                    className="text-xs font-medium text-blue-400 hover:text-blue-300 underline"
                  >
                    {action.label}
                  </a>
                ) : (
                  <button
                    key={action.label}
                    type="button"
                    onClick={action.onClick}
                    className="text-xs font-medium text-blue-400 hover:text-blue-300 underline"
                  >
                    {action.label}
                  </button>
                ),
              )}

              {suggestion && (
                <button
                  type="button"
                  onClick={() => setCollapsed(!collapsed)}
                  className={`text-xs ${subtextColor} hover:${textColor} flex items-center gap-1 ml-auto`}
                >
                  {collapsed ? (
                    <>Show hint <ChevronDown className="w-3 h-3" /></>
                  ) : (
                    <>Hide hint <ChevronUp className="w-3 h-3" /></>
                  )}
                </button>
              )}
            </div>
          )}
        </div>
      </div>
      {code && (
        <p className="text-[10px] text-slate-600 mt-2 ml-8 font-mono select-all">{code}</p>
      )}
    </div>
  );
}
```

Key design decisions:
- Suggestion shown by default, collapsible for experts
- Actions render as small blue links inside the banner
- Error code shown in tiny monospace at bottom (grep-able for experts, invisible to beginners)
- Fully backwards compatible — existing `<FeedbackBanner error={error} />` still works

- [ ] **Step 2: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add web/src/components/FeedbackBanner.tsx
git commit -m "feat(web): enhanced FeedbackBanner with suggestion, code, and actions"
```

---

### Task 3: Enhanced EmptyState

**Files:**
- Modify: `web/src/components/EmptyState.tsx`

- [ ] **Step 1: Update EmptyState to support multiple actions**

Replace the entire file:

```tsx
import { Link } from "react-router-dom";

interface EmptyStateAction {
  label: string;
  href?: string;
  onClick?: () => void;
  variant?: "primary" | "secondary";
}

interface EmptyStateProps {
  icon: React.ElementType;
  title: string;
  description?: string;
  action?: EmptyStateAction;
  actions?: EmptyStateAction[];
}

export function EmptyState({ icon: Icon, title, description, action, actions }: EmptyStateProps) {
  const allActions = actions ?? (action ? [action] : []);

  return (
    <div className="text-center py-12">
      <Icon className="w-12 h-12 text-slate-600 mx-auto mb-4" />
      <h3 className="text-white font-medium mb-1">{title}</h3>
      {description && <p className="text-sm text-slate-400 mb-4">{description}</p>}
      {allActions.length > 0 && (
        <div className="flex items-center justify-center gap-3">
          {allActions.map((a) => {
            const cls = a.variant === "secondary"
              ? "btn btn-secondary inline-flex"
              : "btn btn-primary inline-flex";
            return a.href ? (
              <Link key={a.label} to={a.href} className={cls}>{a.label}</Link>
            ) : (
              <button key={a.label} type="button" className={cls} onClick={a.onClick}>{a.label}</button>
            );
          })}
        </div>
      )}
    </div>
  );
}
```

Backwards compatible: single `action` prop still works. New `actions` array takes precedence when both provided.

- [ ] **Step 2: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add web/src/components/EmptyState.tsx
git commit -m "feat(web): EmptyState supports multiple actions with variants"
```

---

### Task 4: Update Form Pages to Pass Suggestion/Code

**Files:**
- Modify: `web/src/pages/CreateNode.tsx`
- Modify: `web/src/pages/ImportNode.tsx`
- Modify: `web/src/pages/Login.tsx`
- Modify: `web/src/pages/Setup.tsx`
- Modify: `web/src/pages/settings/PasswordSection.tsx`

All five pages follow the same pattern. For each:

1. Add state variables for suggestion and code
2. Update catch blocks to extract from `ApiRequestError`
3. Pass new props to `FeedbackBanner`

- [ ] **Step 1: Update CreateNode.tsx**

Read `web/src/pages/CreateNode.tsx`. Add import and state:

```typescript
import { ApiRequestError } from '../utils/api';
```

Add state alongside existing `error`:
```typescript
const [error, setError] = useState('');
const [suggestion, setSuggestion] = useState('');
const [code, setCode] = useState('');
```

Update the catch block in `handleSubmit`:
```typescript
} catch (err) {
  if (err instanceof ApiRequestError) {
    setError(err.message);
    setSuggestion(err.suggestion ?? '');
    setCode(err.code ?? '');
  } else {
    setError(err instanceof Error ? err.message : 'Failed to create node');
    setSuggestion('');
    setCode('');
  }
}
```

Also clear suggestion/code where error is cleared:
```typescript
setError('');
setSuggestion('');
setCode('');
```

Update FeedbackBanner usage:
```tsx
<FeedbackBanner error={error} suggestion={suggestion} code={code} />
```

- [ ] **Step 2: Update ImportNode.tsx**

Read `web/src/pages/ImportNode.tsx`. Same pattern but with three `onError` handlers in mutations:

```typescript
import { ApiRequestError } from '../utils/api';

const [suggestion, setSuggestion] = useState('');
const [code, setCode] = useState('');
```

Update each `onError`:
```typescript
onError: (err: unknown) => {
  if (err instanceof ApiRequestError) {
    setError(err.message);
    setSuggestion(err.suggestion ?? '');
    setCode(err.code ?? '');
  } else {
    setError(err instanceof Error ? err.message : "Failed to detect node configuration");
    setSuggestion('');
    setCode('');
  }
  setDetected(null);
},
```

Apply the same pattern to all three mutation `onError` handlers (detect, scan, import), each with its own fallback message.

Update each `onSuccess` that clears error to also clear suggestion/code.

Update FeedbackBanner:
```tsx
<FeedbackBanner error={error} suggestion={suggestion} code={code} />
```

- [ ] **Step 3: Update Login.tsx**

Read `web/src/pages/Login.tsx`. Same pattern:

```typescript
import { ApiRequestError } from '../utils/api';

const [suggestion, setSuggestion] = useState('');
const [code, setCode] = useState('');
```

Update catch in `handleSubmit`:
```typescript
} catch (err) {
  if (err instanceof ApiRequestError) {
    setError(err.message);
    setSuggestion(err.suggestion ?? '');
    setCode(err.code ?? '');
  } else {
    setError(err instanceof Error ? err.message : 'Authentication failed');
    setSuggestion('');
    setCode('');
  }
}
```

Update FeedbackBanner:
```tsx
<FeedbackBanner error={error} suggestion={suggestion} code={code} />
```

- [ ] **Step 4: Update Setup.tsx**

Read `web/src/pages/Setup.tsx`. Same pattern:

```typescript
import { ApiRequestError } from '../utils/api';

const [suggestion, setSuggestion] = useState('');
const [code, setCode] = useState('');
```

Update catch:
```typescript
} catch (err) {
  if (err instanceof ApiRequestError) {
    setError(err.message);
    setSuggestion(err.suggestion ?? '');
    setCode(err.code ?? '');
  } else {
    setError(err instanceof Error ? err.message : 'Setup failed');
    setSuggestion('');
    setCode('');
  }
}
```

Update FeedbackBanner:
```tsx
<FeedbackBanner error={error} suggestion={suggestion} code={code} />
```

- [ ] **Step 5: Update PasswordSection.tsx**

Read `web/src/pages/settings/PasswordSection.tsx`. This one uses `passwordError` and `passwordSuccess`:

```typescript
import { ApiRequestError } from '../../utils/api';

const [passwordSuggestion, setPasswordSuggestion] = useState('');
const [passwordCode, setPasswordCode] = useState('');
```

Update the catch block:
```typescript
} catch (error) {
  if (error instanceof ApiRequestError) {
    setPasswordError(error.message);
    setPasswordSuggestion(error.suggestion ?? '');
    setPasswordCode(error.code ?? '');
  } else {
    setPasswordError(error instanceof Error ? error.message : 'Failed to change password');
    setPasswordSuggestion('');
    setPasswordCode('');
  }
}
```

Clear on success:
```typescript
setPasswordSuggestion('');
setPasswordCode('');
```

Update FeedbackBanner:
```tsx
<FeedbackBanner error={passwordError} suggestion={passwordSuggestion} code={passwordCode} success={passwordSuccess} />
```

- [ ] **Step 6: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds

- [ ] **Step 7: Commit**

```bash
git add web/src/pages/CreateNode.tsx web/src/pages/ImportNode.tsx web/src/pages/Login.tsx web/src/pages/Setup.tsx web/src/pages/settings/PasswordSection.tsx
git commit -m "feat(web): pass error code and suggestion to FeedbackBanner on all form pages"
```

---

### Task 5: Enhanced Dashboard Empty State

**Files:**
- Modify: `web/src/pages/Dashboard.tsx`

- [ ] **Step 1: Update empty state to dual actions**

Read `web/src/pages/Dashboard.tsx`. Find the `<EmptyState>` usage and replace it:

```tsx
<EmptyState
  icon={Server}
  title="No nodes yet"
  description="Deploy a new Neo node or import an existing installation"
  actions={[
    { label: "Create Node", href: "/nodes/create", variant: "primary" },
    { label: "Import Existing", href: "/nodes/import", variant: "secondary" },
  ]}
/>
```

Also add `import { FolderInput } from "lucide-react"` if not already imported (not needed if only using actions without icons).

- [ ] **Step 2: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add web/src/pages/Dashboard.tsx
git commit -m "feat(web): dual-action empty state on dashboard (Create + Import)"
```

---

### Task 6: Node Detail Progressive Disclosure

**Files:**
- Modify: `web/src/pages/NodeDetail.tsx`

- [ ] **Step 1: Read NodeDetail.tsx and identify sections**

Read the full `web/src/pages/NodeDetail.tsx`. The Overview tab currently shows:
- Metrics card (block height, peers, CPU, memory)
- NodeConfigEditor component
- Sidebar with: Ports card, Process Info card, Secure Signer card

The progressive disclosure change: wrap the sidebar technical details (Ports, Process Info) in a collapsible section. The Secure Signer card stays visible since it's security-relevant.

- [ ] **Step 2: Add details toggle**

Add state at the top of the component:
```typescript
const [showDetails, setShowDetails] = useState(() => {
  return localStorage.getItem("neo-nexus-show-node-details") === "true";
});

const toggleDetails = () => {
  setShowDetails((prev) => {
    localStorage.setItem("neo-nexus-show-node-details", String(!prev));
    return !prev;
  });
};
```

Add the toggle button above the sidebar's Ports card:

```tsx
<button
  type="button"
  onClick={toggleDetails}
  className="flex items-center gap-2 text-sm text-slate-400 hover:text-slate-200 mb-3 w-full"
>
  {showDetails ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
  {showDetails ? "Hide technical details" : "Show technical details"}
</button>
```

Wrap the Ports card and Process Info card in a conditional:
```tsx
{showDetails && (
  <>
    {/* Ports card */}
    {/* Process Info card */}
  </>
)}
```

Add `ChevronDown` and `ChevronUp` to the lucide-react imports.

- [ ] **Step 3: Verify build**

Run: `cd web && npm run build`
Expected: Build succeeds

- [ ] **Step 4: Commit**

```bash
git add web/src/pages/NodeDetail.tsx
git commit -m "feat(web): progressive disclosure toggle for technical details on node page"
```

---

### Task 7: Final Build Verification

- [ ] **Step 1: Clean build**

```bash
cd web && rm -rf dist && npm run build
```
Expected: Build succeeds with no errors

- [ ] **Step 2: Run backend tests (ensure no regressions)**

```bash
npx vitest run
```
Expected: All tests pass

- [ ] **Step 3: TypeScript check on backend**

```bash
npx tsc --noEmit
```
Expected: Clean
