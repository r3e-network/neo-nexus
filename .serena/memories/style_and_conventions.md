# Style And Conventions

- Language/stack: TypeScript + React 19 + Next.js App Router.
- Imports use the `@/` alias for `dashboard/src/*`.
- Server-side auth/org access should go through shared helpers in `dashboard/src/server/organization.ts` instead of ad hoc `(session.user as any)` casts.
- Shared unknown-error formatting should use helpers in `dashboard/src/server/errors.ts` rather than `any` catch blocks.
- Dashboard routes under `dashboard/src/app/app` that depend on auth/session are intentionally dynamic; use `export const dynamic = 'force-dynamic';` when a page depends on request headers/session state.
- Prefer tenant-scoped Prisma queries for authenticated resources; do not fall back to global or first-record organization access.
- UI code uses Tailwind classes inline and prefers explicit local types for SWR payloads/client state over `any`.
- Production safety convention: avoid silent credential fallbacks for external admin APIs (for example APISIX admin credentials must be explicitly configured).