# Changelog

## [Unreleased]

## [2.5.3] - 2026-05-17

### Security
- Patched `systeminformation` to `^5.31.6` to clear the high-severity Linux command-injection advisory (GHSA-hvx9-hwr7-wjj9, CVE category CWE-78) affecting `networkInterfaces()` parsing of NetworkManager profile names.
- Login now runs `bcrypt.compare` against a constant dummy hash when the requested username does not exist, so successful and failed logins take the same wall-clock time and remote attackers cannot enumerate valid usernames via timing.
- The dev-mode JWT secret fallback now uses 32 bytes from `crypto.randomBytes` instead of `Math.random()`, eliminating a predictable secret if a developer forgets to set `JWT_SECRET` outside production.
- `/api/public/*` endpoints no longer return raw `error.message` strings to unauthenticated callers; failures are logged server-side and the response is a generic 500 so internal paths or library traces never leak.
- `validateNodePath` (used by `/api/nodes/detect`, `/scan`, and `/import`) now resolves symlinks with `realpathSync` and re-verifies the real path against the allow-list, closing a symlink-escape vector that could have leaked files outside `/home`, `/opt`, `/var/lib`, and the NeoNexus data directory.
- Fast-sync local snapshot sources are restricted to the same allow-list (plus `os.tmpdir()` for staging), preventing operators from probing arbitrary files via the verify endpoint.
- Telegram bot tokens and chat IDs are now validated against strict regexes before being interpolated into the request URL/body, preventing path-injection or unexpected payload shapes.
- Failed and successful login attempts are now written to the audit log with username and IP (never the password) so operators can detect brute-force or credential-stuffing patterns.

### Changed
- Grafana Cloud metrics now use Prometheus `remote_write` payloads with snappy compression instead of sending plain text to a remote-write endpoint.
- Better Stack Logs now supports source-specific ingest URLs while keeping the default Better Stack ingest endpoint.
- Better Stack Uptime and UptimeRobot now require a public health URL, preventing external monitors from being registered against local-only `localhost` addresses.
- Compatible dependency updates refreshed Sentry, React, React Router, TanStack Query, better-sqlite3, tar, PostCSS, and TypeScript ESLint within the existing major-version ranges.

### Fixed
- Integration URL validation errors now respond with `INTEGRATION_CONFIG_INVALID` (HTTP 400) and a usable suggestion instead of being re-coded as `INTERNAL_ERROR` (HTTP 500).
- `IntegrationCard` now surfaces save failures inline so a bad URL or DSN no longer silently fails — the user sees the exact API message.
- Every integration provider field now has a proper `<label htmlFor>` ↔ `<input id>` association, satisfying Chrome / assistive-tech form-label heuristics.
- README version badge and package manifests now point at `2.5.3`.

### Tested
- Added provider-level tests for Grafana remote_write, Better Stack log ingest routing, and uptime public health URL registration.
- Backend lint, test lint, typecheck, full backend test suite (668 tests across 82 files), and backend production build pass.
- Frontend lint, typecheck, full frontend test suite (34 tests across 8 files), and production build pass.
- `npm audit` reports zero vulnerabilities across all severities for both backend and frontend dependencies.

## [2.5.2] - 2026-05-06

### Added
- Dashboard empty-fleet onboarding now guides first-time operators toward creating a managed node, planning a private network, or importing an existing node in observe-only mode.
- Setup, password-change, and user-creation flows now show password-strength guidance before submission.
- Metrics responses now include block-height sync status so the UI can distinguish synced, catching-up, stale, and unknown data before operators rely on node height.

### Changed
- Plugin lifecycle management now reports richer version/action state and keeps installed, enabled, update, and uninstall flows aligned with node ownership and compatibility rules.
- Create-node role setup, integration credentials, and frontend setup UX now provide clearer defaults and safer validation for production operators.
- README version badge and package manifests now point at `2.5.2`.

### Fixed
- Hermes viewer tools now respect redaction boundaries for node details and plugin configuration instead of exposing admin-only fields through agent tools.
- Fast-sync downloads now use the shared outbound target guard so HTTPS snapshot sources receive the same SSRF and redirect protections as other user-supplied URLs.
- Browser-side source scanning now guards against accidental SQL/database logic in frontend code.
- First-run setup now trims usernames consistently across setup, login, and user creation.

### Tested
- Backend lint, test lint, typecheck, full backend test suite, and backend production build pass.
- Frontend lint, typecheck, full frontend test suite, and production build pass.

## [2.5.1] - 2026-05-04

### Fixed
- First-run setup now establishes the authenticated session immediately after creating the initial admin account, so new operators land directly in the console instead of being bounced back to sign-in.
- Setup and login forms now bind visible labels to their inputs, improving accessibility and making browser-level QA more reliable.
- Plugin management no longer triggers a React maximum-update-depth loop when no `neo-cli` nodes exist.
- Private-network and node-orchestration views now use stable memo dependencies, removing React hook warnings in the role/data-context workflows.

### Tested
- Backend and frontend lint, typecheck, unit/integration tests, and production build pass.
- Browser QA covered setup, dashboard, nodes, create-node role presets, private-network planning, plugins, integrations, servers, Hermes Agent setup, settings, public status, and mobile dashboard layout.

## [2.5.0] - 2026-05-03

### Added
- **Node role orchestration** — Built-in and custom node identities for RPC/API, state, oracle, consensus, indexer, and secure-signer-client workflows, with one-click role application across plugins, config, storage, and data contexts.
- **Data context switching** — Nodes can record and switch isolated blockchain data contexts, making role changes and chain data separation explicit.
- **Private network planner** — One-click single-node, 4-node, and 7-node private N3 network plans with generated names, addresses, ports, committee setup, storage choice, and guarded apply flow.
- **Fast-sync manifests** — Snapshot registration, verification, download, checkpoint height/hash metadata, and data-context integration for faster node bootstrap paths.
- **Storage engine selection** — LevelDB/RocksDB selection is supported in node creation, role profiles, private networks, data contexts, and config orchestration.

### Changed
- README and operator documentation now describe role orchestration, private network planning, storage engines, fast sync, and data isolation workflows.

## [2.4.0] - 2026-05-02

### Added
- **Hermes Agent** — In-app AI agent that operates the node fleet on your behalf. Bring your own API key (Anthropic, OpenAI, or any OpenAI-compatible base URL). Streaming responses, 12 typed tools (read: list_nodes, get_node, get_node_logs, list_plugins, get_system_metrics, get_network_height, list_remote_servers, list_integrations; admin: start_node, stop_node, restart_node, set_plugin_enabled), per-user conversations persisted in SQLite, role-gated tool access, audit-logged control actions. Disabled by default behind `NEONEXUS_ENABLE_HERMES_AGENT=true`.
- **Neo X support (preview)** — Manage `neox-go` (geth fork from `bane-labs/go-ethereum`) alongside Neo N3. New `chain` axis above node type; mainnet (chain id 47763) and testnet (12227332); EVM JSON-RPC metrics (`eth_blockNumber`, `net_peerCount`, `eth_chainId`); separate port range (8551 RPC / 30303 P2P / 8571 WS / 8561 AuthRPC) so chains coexist on one host; Linux-only binaries downloaded from the bane-labs releases. Disabled by default behind `NEONEXUS_ENABLE_NEOX=true`.
- **Role-based access control** — New `requireAdmin` / `requireAdminForUnsafeMethods` middleware wired into all mutating routes (`/api/nodes`, `/api/nodes/:id/start|stop|restart`, `/api/nodes/:id/plugins`, `/api/secure-signers`, `/api/integrations`, `/api/system`, `/api/system/audit-log`, `/api/servers`). Viewers retain read access; only admins can mutate state.
- **WebSocket subprotocol auth** — Bearer tokens accepted via the `neonexus.auth` subprotocol (`new WebSocket(url, ['neonexus.auth', token])`) or the `Authorization` header. Query-string token (`?token=`) is now disabled by default.
- **Outbound SSRF / DNS-rebind protection** — New `outboundTargets`, `publicFetch`, and `safeIntegrationFetch` utilities. Literal and DNS-resolved targets are checked against IPv4 private ranges (10/8, 127/8, 169.254/16, 172.16/12, 192.168/16, 100.64/10 CGNAT, 0/8, 224/4), IPv6 link-local (fe80::/10), unique-local (fc00::/7), multicast (ff00::/8), loopback (`::1`), zero (`::`), and IPv4-mapped IPv6 in both dotted (`::ffff:127.0.0.1`) and hex (`::ffff:7f00:1`) forms. Pinned-IP HTTP/HTTPS client preserves original Host header and TLS SNI so cert validation still works.
- **GitHub Actions CI workflow** — `.github/workflows/ci.yml` runs lint + typecheck + test + coverage + build on every push to main and on pull requests.
- **Frontend chat panel** at `/agent` — Setup form (provider, model, BYO key, optional base URL) and chat UI with streaming text deltas, tool-call rendering, conversation list with delete, cancellation, mobile-responsive layout (conversation list collapses to a horizontal pill bar on small screens).
- **Sidebar entries** — "Hermes Agent" between Integrations and Settings; chain-aware violet "Neo X" badge on chain-X nodes in the fleet table.
- **Datadog site validation** — Strict DNS-label regex on the `site` field rejects credential-leak vectors like `@evil.com`, `evil.com#`, paths, ports, and CRLF.
- **DownloadManager redirect hardening** — HTTPS-only enforcement, 5-hop redirect cap with `Too many redirects` rejection, relative-redirect resolution against the original request URL.
- **Schema migrations** — `chain` column on `nodes` table (default `'n3'` for existing rows); new tables `agent_settings` (per-user), `agent_conversations`, `agent_messages`.
- **README screenshots** — 10 captures (public status, login, dashboard, nodes, plugins, integrations, servers, settings, agent setup, agent chat) embedded with relative paths so GitHub renders them.
- **Environment variables** — `NEONEXUS_ENABLE_HERMES_AGENT`, `NEONEXUS_ENABLE_NEOX`, `NEONEXUS_ALLOW_PRIVATE_REMOTE_SERVERS`, `NEONEXUS_ALLOW_PRIVATE_SIGNER_ENDPOINTS`, `NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS`, `NEONEXUS_ALLOW_WS_QUERY_TOKEN`, `NEONEXUS_SIGNER_WORKSPACE_ROOTS`.

### Changed
- **Integration providers route through `safeIntegrationFetch`** — Webhook, Slack, Discord, Sentry (testConnection), Grafana Loki, Datadog, and Grafana Cloud now use the pinned outbound client. User-supplied URLs/sites can no longer reach private/local targets without an explicit override.
- **`RemoteServerManager` and `SecureSignerManager`** — Resolve hostnames once and pin TCP/HTTP probes to the resolved address; resolver is injectable for tests.
- **Type definitions** — `NodeChain` introduced; `NodeType` widened to include `neox-go`; `NodeNetwork` widened to include `neox-mainnet` / `neox-testnet`. N3-specific data structures (committee, hardforks, seed lists, network magic) narrowed to `N3NodeNetwork` so they cannot accidentally process X networks.
- **PortManager** — Now chain-aware; `allocatePorts(index, chain)` and `findNextIndex(max, chain)` route X nodes to a separate port range.
- **NetworkHeightTracker** — Returns null for Neo X networks (per-node height is fetched directly via `eth_blockNumber` inside `NeoXNode`).
- **Frontend Node form** — Chain selector at the top; switching to Neo X auto-selects `neox-mainnet` and clears N3-only signer state. The Neo X option only shows when `NEONEXUS_ENABLE_NEOX=true`.

### Fixed
- **IPv6 fe80::/10 link-local check incomplete** — Previously `startsWith("fe80:")` only blocked the canonical `fe80::` range. Now `fe80–febf` (the full /10) is matched, blocking `fe93::`, `fea0::`, `febf::`.
- **Version display showed `vv0.118.0`** — Stored version already starts with `v` and the UI was prepending another. New `formatVersion()` helper handles both forms; applied to Nodes, NodeDetail, Servers, and PublicDashboard.
- **Dashboard fleet-health badge contradicting itself** — When all nodes were stopped the page showed a red `0%` next to a green "Healthy" badge. Badge logic now distinguishes No nodes / Needs attention / All stopped / Partial / Healthy.
- **Mobile `/agent` page broken** — Fixed-width conversation sidebar consumed the entire 375 px viewport, hiding the chat. Switched to `flex-col` on small screens with the conversation list collapsed to a horizontal-scrolling pill bar.
- **Empty assistant bubble after early provider error** — When the LLM returned 401 before any text streamed, the persisted assistant message rendered as a blank cloud next to the error banner. `isVisibleMessage` filter now drops empty assistant/tool messages.
- **`publicFetch` missing response error handler** — Added `res.on("error")` listener with `cleanupAbortListener` + `reject` so a truncated response stream rejects the promise instead of leaking an unhandled error.
- **Misleading test assertion** — `tests/unit/integration-outbound.test.ts` asserted `fetchMock` was not called, but the implementation switched to raw `http.request`/`https.request` so the assertion always passed. Replaced with `vi.spyOn(http, "request")` and `vi.spyOn(https, "request")` checks.

### Security
- **SSRF / DNS-rebind protection** centralized in one helper used by every user-URL provider, remote-server manager, and secure-signer probe. Closes a class of issues where an attacker-controlled webhook URL could resolve to AWS metadata (`169.254.169.254`) or other private targets.
- **RBAC enforced** on every mutating route. Viewer accounts now cannot start/stop nodes, install plugins, edit servers, configure integrations, or read the audit log.
- **WebSocket auth tokens out of URL query strings** — Default behavior; the legacy `?token=` path is gated behind `NEONEXUS_ALLOW_WS_QUERY_TOKEN=true`.
- **Datadog site input** is now strict DNS-label syntax to prevent confused-deputy hostname injection that would leak the API key to an attacker-controlled host.

### Tested
- Backend: 59 test files / 413 tests pass (was 55 / 389 in 2.3.0).
- Frontend: 2 test files / 12 tests pass.
- Multi-viewport QA: 12 pages × 4 viewports (desktop/laptop/tablet/mobile) screenshot pass with 0 console errors, 0 failed requests, 0 horizontal-overflow layouts.
- Functional smoke: 8/8 sidebar links navigate without ErrorBoundary; Neo X chain selector swaps networks; agent send hits real Anthropic with stub key and surfaces error event; public status page renders all sections.

### Migration notes
- **No schema changes that affect existing data.** The `chain` column on `nodes` is auto-backfilled to `'n3'` for pre-existing rows. New agent tables are unused unless `NEONEXUS_ENABLE_HERMES_AGENT=true`.
- **WebSocket clients using `?token=`** must either switch to the `neonexus.auth` subprotocol (recommended), use the `Authorization` Bearer header, or set `NEONEXUS_ALLOW_WS_QUERY_TOKEN=true` to retain legacy behavior.
- **Viewers** that previously had write access via direct API calls will receive 403 from mutating endpoints. Promote them to `admin` if they need write access.
- **Integration credentials** that target private/local hosts (rare) now require `NEONEXUS_ALLOW_PRIVATE_INTEGRATION_TARGETS=true`. Same gate exists for remote server profiles (`_REMOTE_SERVERS`) and secure-signer endpoints (`_SIGNER_ENDPOINTS`).

## [2.3.0] - 2026-04-26

### Added
- **Native node import ownership model** — Imported neo-cli/neo-go nodes now default to observe-only, with explicit managed-config and managed-process adoption paths.
- **Secure Signer policy hardening** — Signer profiles and node bindings now preserve policy/account/unlock metadata and fail closed when requested hardware protection is unavailable.
- **Professional operator console UI** — Login, Dashboard, Nodes, Node Detail, Plugins, Settings, and Secure Signer areas now share a polished cloud-console design system.
- **Tailwind v4 Vite integration** — Frontend builds now use `@tailwindcss/vite` so utility classes are generated reliably in preview and production builds.

### Changed
- **Imported-node lifecycle safety** — Process lifecycle actions require explicit ownership mode plus PID, command, argv/cwd, and path-boundary validation before any stop/start behavior.
- **Plugin mutation routing** — Plugin install/update/uninstall/enable/disable paths are aligned with node ownership guards instead of relying on route-level assumptions.
- **Frontend ownership UX** — Observe-only and managed-config states now surface clearer disabled actions, risk copy, and upgrade paths before backend rejection.
- **Config persistence semantics** — Imported node config paths and key-protection settings are preserved more carefully during updates.

### Fixed
- **PID/path prefix collisions** — Prevented trusted path checks such as `/opt/neo` from matching sibling paths like `/opt/neo-other`.
- **Imported neo-cli config audit** — Audits now inspect adopted config files such as `config.mainnet.json` / `config.testnet.json` instead of assuming `base/config.json`.
- **WebSocket login-page noise** — The frontend only opens authenticated WebSocket connections after both token and user state are present.
- **Node detail layout balance** — Collapsed technical details no longer leave an empty right column; the toggle is integrated into the page control bar.
- **Plugin catalog density** — Plugin cards avoid cramped two-column layouts at ordinary desktop widths.

## [2.2.0] - 2026-03-30

### Added
- **SaaS Integrations page** — Dedicated page to connect NeoNexus to external services, with category tabs (Metrics, Logging, Uptime, Alerting, Errors)
- **11 integration providers** — Grafana Cloud (metrics), Datadog, Better Stack (logging + uptime), Grafana Loki, UptimeRobot, Sentry, Webhook, Slack, Discord, Telegram
- **Token-gated activation** — Each provider is optional and only activates when credentials are configured via the Integrations page
- **Save & Test flow** — One-click credential validation with real connectivity checks against each SaaS API
- **IntegrationManager** — Backend singleton that routes metrics, logs, and notifications to enabled providers with fire-and-forget error isolation
- **Metrics push** — System and per-node metrics (CPU, memory, disk, block height, peers, sync progress) pushed to Grafana Cloud and Datadog on the existing 5-second interval
- **Log shipping** — Node logs batched and shipped to Better Stack (Logtail) and Grafana Loki every 5 seconds
- **Event notifications** — Node start/stop/crash, watchdog restart/exhaustion, and disk alerts sent to Slack, Discord, Telegram, and generic webhooks
- **Sentry error tracking** — Uncaught exceptions forwarded to Sentry when configured
- **Uptime monitoring** — Auto-register/deregister health endpoint monitors with Better Stack Uptime and UptimeRobot
- **Alert debouncing** — 5-minute cooldown on repeated notification events to prevent alert flooding
- **Credential security** — Sensitive fields (API keys, webhook URLs) redacted in API responses; deterministic redaction prefix prevents credential corruption on save
- **Frontend form validation** — Required fields validated before Save & Test; password/sensitive URL fields have reveal toggle with aria-labels
- **ARIA accessibility** — Category tabs use proper `role="tablist"` / `role="tab"` / `aria-selected` semantics

### Changed
- **Provider architecture** — Plugin-based design with `Map<IntegrationId, Provider>` for O(1) lookup and safe removal
- **@sentry/node dependency** — Added for Sentry error tracking integration

## [2.1.0] - 2026-03-29

### Added
- **Initial setup flow** — First-time users are guided through admin account creation instead of using default credentials
- **User management** — Admins can create, view, and delete users from the Settings page
- **Audit log viewer** — Admins can view all system actions (node starts/stops, config changes) with pagination
- **Network height display** — Node detail shows network height and sync progress percentage for mainnet/testnet nodes
- **Loading skeletons** — All pages now show shimmer skeleton placeholders while data loads
- **Empty states** — Attractive empty states with icons, descriptions, and action buttons across all pages
- **SpinnerButton component** — Async action buttons show loading spinners during operations
- **Shared UI components** — Extracted reusable SignerStatus, NodeProtectionLabel, and ToggleSwitch components
- **ESLint 9 configuration** — Flat config for both backend (TypeScript) and frontend (React + TypeScript)
- **Frontend constants** — Centralized refetch intervals and UI limits in `web/src/config/constants.ts`
- **404 catch-all route** — Unknown URLs now redirect to the dashboard

### Changed
- **NodeManager decomposed** — Extracted database operations to `NodeRepository` class (1019 LOC -> 821 LOC)
- **RPC duplication eliminated** — Moved identical `executeCommand()` from NeoCliNode/NeoGoNode to `BaseNode.executeRpc()`
- **Database types extracted** — Shared row types in `src/types/database.ts` replace 3 inline definitions
- **Hardcoded constants extracted** — Committee keys, hardfork heights, and plugin versions moved to `src/data/` modules
- **Server magic numbers named** — 15 inline magic numbers replaced with documented constants
- **Type deduplication** — `UpdateSecureSignerRequest` now uses `Partial<CreateSecureSignerRequest>`
- **Settings.tsx split** — 973 LOC decomposed into PasswordSection, StorageSection, SecureSignerSection, DangerZoneSection (~89 LOC parent)
- **Plugins.tsx split** — 545 LOC decomposed into PluginCard and ConfigField components (~239 LOC parent)
- **NodeDetail.tsx split** — 664 LOC decomposed into NodeConfigEditor and NodeLogsView (~275 LOC parent)
- **`any` types eliminated** — Replaced remaining `as any` casts in ConfigManager with proper Record types

### Fixed
- **Restart button** — Now properly handles stop failure (won't attempt start if stop fails)
- **Form validation** — CreateNode validates numeric fields and peer constraints (minPeers <= maxPeers)
- **FeedbackBanner animation** — Added fade-in entrance animation
- **Button loading states** — All form submit buttons now show spinners during async operations
- **Accessibility** — Added `aria-label` attributes to icon-only action buttons, `type="button"` to non-submit buttons
- **PublicDashboard loading** — Public status page now shows skeletons instead of blank content while loading

## [2.0.0] - 2026-03-27

Initial release of NeoNexus v2 — self-hosted Neo N3 node management platform.
