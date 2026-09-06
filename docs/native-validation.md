# Native Rust App Validation

Validation date: 2026-08-28

This document describes the verification evidence expected before NeoNexus is
released or pushed as a pure Rust application.

NeoNexus is validated as a pure Rust web-served operations workbench, not as a
frontend project, Node toolchain, WebView/Tauri shell, or browser-framework
app. The gates cover Rust correctness, cross-platform CI, source purity,
source quality, web behavior, node-management behavior, and release handoff
evidence.

## Core Gates

```bash
cargo fmt --all -- --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test --lib
cargo test --test ci_policy
cargo test --test domain
cargo test --test repository
cargo test --test web
cargo run -- --self-check
cargo run -- --source-purity .
cargo run -- --source-quality .
cargo run -- --ci-policy .github/workflows/ci.yml
make web-smoke
```

Expected result:

- Every gate exits `0`.
- `--self-check` reports `mode: web workbench` with a green workspace database.
- `--source-purity` reports no findings: no Node/Web manifests, no frontend
  source files, no WebView/Tauri dependencies. Browser assets live in
  `src/web/assets.rs` as Rust string constants.
- `--source-quality` reports no findings: no panic-oriented production
  markers, no hardcoded platform shortcut labels, no oversized maintenance
  files.
- `--ci-policy` confirms the cross-platform gate set on Ubuntu, macOS, and
  Windows with no frontend toolchain.
- `cargo test --test web` proves the auth boundary and the lifecycle control
  path over real HTTP.

## Web Smoke Evidence

`make web-smoke` starts the workbench against a throwaway workspace and
requires:

- `/healthz` reports `status` `ok` in its compact JSON body, without authentication.
- `/api/fleet` refuses unauthenticated access.

## Release Handoff Evidence

```bash
cargo build --release
./target/release/neo-nexus --self-check
./target/release/neo-nexus --package-release dist
./target/release/neo-nexus --verify-release-package-integrity dist
./target/release/neo-nexus --verify-release-package-integrity-json dist
```

These explicitly integrity-only checks validate bounded ZIP layout, manifests,
checksums, and the binary hash. They do not authenticate a publisher. Before a
production handoff, an external release signer must sign the exact canonical
sidecar manifest bytes and consumers must run `--verify-release-package` (or
its JSON form) with the independently pinned base64 Ed25519 public key and the
base64 detached signature file. Authenticated verification fails when either
the trust anchor or signature is absent. The repository intentionally contains
no production signing key or trust anchor.

Pushes to the main branch additionally attest every file in `dist/` with
GitHub's build-provenance service (`actions/attest@v4`). Verify a downloaded CI
artifact with `gh attestation verify <artifact> -R <owner>/<repo>`. This proves
which repository workflow built the bytes; the explicit Ed25519 command above
remains the offline/operator-controlled publisher trust path.
