# Suggested Commands

From repo root `/home/neo/git/neo-nexus`:

- `npm run dev` — run the dashboard dev server through the workspace root.
- `npm run build` — production build of the dashboard workspace.
- `npm run lint` — run ESLint for the dashboard workspace.
- `npm run typecheck` — run TypeScript type checking for the dashboard workspace.
- `npm run verify` — full verification: lint + typecheck + build.
- `npm test` — currently aliases `npm run verify` at the repo root.

From `dashboard/` directly:

- `npm run dev`
- `npm run build`
- `npm run lint`
- `npm run typecheck`
- `npm run verify`

Useful shell commands in this Linux environment:

- `rg --files` for fast file listing
- `rg -n "pattern" path` for text search
- `git status --short` for working tree summary
- `sed -n 'start,endp' file` for targeted reads
