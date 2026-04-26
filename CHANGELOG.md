# Changelog

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
