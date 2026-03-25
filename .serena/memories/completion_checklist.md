# Completion Checklist

Before finishing work in this repo:

- Run `npm run verify` from the repo root.
- If the task only touches the dashboard workspace, `npm run verify --workspace=dashboard` is the same underlying check.
- Confirm auth-sensitive pages still build as dynamic routes when they use request/session state.
- For data mutations, confirm queries remain scoped to the current organization.
- If scripts or build assumptions changed, update the relevant docs (`README.md` or `docs/*`) and root/workspace package scripts as needed.
- Be explicit in final notes about any remaining product-level gaps that are mocked or simulated rather than fully production-backed (for example simulated analytics or mocked crypto payment verification).