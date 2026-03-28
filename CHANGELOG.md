# Changelog

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
