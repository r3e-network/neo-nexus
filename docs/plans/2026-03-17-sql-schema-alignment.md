# SQL Schema Alignment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the stale `infrastructure/database/schema.sql` Supabase-era schema with SQL that matches the live Prisma data model used by the dashboard.

**Architecture:** Use Prisma as the source of truth. Generate a fresh SQL script from `dashboard/prisma/schema.prisma` using Prisma’s schema-diff tooling, then replace the old hand-written SQL file and update any surrounding comments/doc wording that still imply Supabase/auth-based ownership.

**Tech Stack:** Prisma CLI, PostgreSQL DDL, TypeScript workspace verification

### Task 1: Generate Authoritative SQL

**Files:**
- Modify: `infrastructure/database/schema.sql`

**Step 1: Generate the SQL**

Run:
```bash
cd dashboard
npx prisma migrate diff --from-empty --to-schema-datamodel prisma/schema.prisma --script
```

**Step 2: Replace the old schema file**

Update `infrastructure/database/schema.sql` with the generated SQL and remove stale Supabase/auth-specific comments.

**Step 3: Verify**

Run:
```bash
npm run typecheck --workspace=dashboard
```

Expected: PASS

### Task 2: Review Drift

**Files:**
- Modify docs only if the schema file comment/header needs clarification

**Step 1: Review**

Run:
```bash
git diff -- infrastructure/database/schema.sql docs/DEPLOY_FRONTEND.md docs/DEPLOY_INFRASTRUCTURE.md
```

Expected: The SQL file now mirrors Prisma models rather than the old Supabase schema.

### Task 3: Full Verification

**Step 1: Run the full verification suite**

Run:
```bash
npm run verify
```

Expected: PASS
