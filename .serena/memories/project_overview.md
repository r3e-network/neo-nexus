# Neo Nexus Project Overview

- Purpose: NeoNexus is a Neo ecosystem infrastructure platform combining a marketing site, authenticated dashboard, and deployment/infrastructure assets for Neo N3 and Neo X node services.
- Primary app: `dashboard/` is a Next.js 16 App Router application that contains both the marketing site and the authenticated control console.
- Backend/data: Prisma with PostgreSQL schema in `dashboard/prisma/schema.prisma`; NextAuth is used for authentication; Stripe is used for billing flows.
- Infra assets: `infrastructure/` contains Docker, Helm charts, and SQL schema for the control-plane/infrastructure side.
- Repo layout:
  - `dashboard/src/app/(marketing)` public pages
  - `dashboard/src/app/app` authenticated app pages
  - `dashboard/src/app/api` route handlers
  - `dashboard/src/services` integration/services layer (Kubernetes, APISIX, Stripe, Neo node checks, vault)
  - `docs/` architecture and deployment docs
  - root `package.json` proxies workspace scripts into `dashboard/`
- Current notable state: the dashboard code now has shared typed organization/session helpers in `dashboard/src/server/organization.ts`, shared error helpers in `dashboard/src/server/errors.ts`, and production build/typecheck/lint verification wired through workspace scripts.