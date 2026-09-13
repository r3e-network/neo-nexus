# The Web Workbench

NeoNexus 4.0 runs the operator workbench as an HTTP service. Start the binary
and open the printed address in a browser — from a workstation, a bastion
host, or the same cloud server the node fleet lives on.

## Server model

- **One process, one binary.** `neo-nexus` (no options) or `neo-nexus --web`
  binds an axum/tokio server, opens (or creates) the workspace SQLite
  database, and serves until stopped.
- **Embedded assets.** The stylesheet and the polling script are string
  constants in `src/web/assets.rs`, compiled into the binary. There is no
  static directory to deploy and no Node toolchain anywhere.
- **Server-side rendering.** Pages are assembled from Rust functions in
  `src/web/pages/`. Every interpolated value passes through the HTML escaper
  in `src/web/html.rs`.
- **One core pipeline.** Node launch and stop live in `src/supervision.rs`;
  browser controls and the watchdog both call it, so a manual start and an
  automatic restart are the same code path against the same supervisor. There is
  no web-only control path.
- **Not only request-driven.** A background engine applies the policies the
  Settings page edits — see [Supervision](#supervision). Run exactly one
  instance per workspace: two engines would fight over the same nodes.

## Supervision

The engine thread wakes once a second and, for each policy that is enabled:

- reaps finished processes, journals the exit with the log's own diagnosis, and
  restarts a crashed node within the watchdog budget;
- probes RPC health for running nodes on the configured interval, recording a
  journal entry only when the status actually changes;
- probes enabled federation peers on their interval, likewise on change;
- offers new journal entries to the alert route and records what the webhook
  answered.

Long host work — a runtime download and install — runs on a job thread behind a
one-job-per-lane registry, so it cannot time out a browser, survives a page
reload, and cannot race a second install into the same directory. The Runtimes
page reports the running and recent jobs.

Before the first request is served, the engine reconciles any node still
recorded as Running against the host: a live orphan keeps its status, a dead one
is settled and journaled, and a recycled pid is settled but reported separately.
A blanket clear would be wrong here — a workbench killed with SIGKILL leaves its
nodes running, and dropping those rows loses the only handle on them.

Policies are re-read each tick, so saving in Settings applies without restarting
the server. Scheduled runtime upgrades are the one policy the Settings page
records but nothing yet enforces, and the page says so. The engine shares the server's single `ProcessSupervisor`: a node it
restarts is a node the browser can stop, and a node recorded as running without a
handle here is settled by pid rather than left claiming to run.

### Watchdog restart budget

The watchdog settings default to disabled, and explicit true/false take precedence
over any missing key; older databases without the new keys remain disabled.
Single-node clusters may still show per-iteration delays because each restart
is computed independently on its own tick; the final delay caps at the configured
maximum and jittered samples ±15% of the base (±30% max), so multiple nodes
scheduled in the same window will not synchronize perfectly. Production evaluation
of de-synchronization benefits remains unmeasured: no real multi-node cluster
experiment justifies changing the default.

## Launch options

| Option | Default | Meaning |
|--------|---------|---------|
| `--web` | — | Explicit spelling of the default server mode |
| `--bind <addr>` | `127.0.0.1` | Listen address; `0.0.0.0` on cloud hosts |
| `--port <port>` | `8080` | Listen port |
| `--web-token-file <path>` | interactive bootstrap only | Regular file containing the operator sign-in token |
| `--web-public-origin <origin>` | inferred only on loopback | Exact browser-facing origin; required and HTTPS for non-loopback listeners |

The workspace root is `NEONEXUS_DATA_DIR` or the OS data directory, the same
convention the CLI uses. Everything beside `neonexus.db` (managed configs,
supervised logs) lives where the CLI puts it.

## Authentication

- One operator token per deployment. Resolution order: `--web-token-file` →
  `NEONEXUS_WEB_TOKEN_FILE` → an interactive-only bootstrap token. The file
  must contain at least 32 bytes and no more than 4 KiB; Unix startup rejects
  group- or world-accessible files. Service or redirected startup without a
  token file fails closed instead of printing a credential into logs.
- Secret-bearing `--web-token` and `NEONEXUS_WEB_TOKEN` inputs are deliberately
  refused because command lines and process environments are observable.
- The store retains only the token's SHA-256 digest after startup/bootstrap
  output and compares fixed-size digests, so candidate length and prefix
  matches do not affect comparison timing.
- A successful login mints a UUID session id kept server-side with a 12-hour
  sliding expiry; the browser receives it as an `HttpOnly; SameSite=Strict`
  cookie (`neonexus_session`). HTTPS public origins also set `Secure`; logout
  deletes a cookie with the same attributes.
- Login failures are throttled per connecting IP with bounded, expiring server
  state. Forwarded-IP headers are not trusted; deployments behind a reverse
  proxy should also enforce per-client limits at that proxy. Protected POST
  requests are accepted only when an exact Origin, or a valid same-origin
  Referer when Origin is absent, matches the configured browser origin.
- Pages redirect to `/login` without a session; API routes answer `401` so
  the polling script can send the browser back to the login page.
- Every response is `no-store` and carries a strict browser policy: embedded
  CSS/JavaScript are pinned by SHA-256 in CSP, framing and MIME sniffing are
  denied, same-origin provenance is retained for CSRF validation without
  leaking Referer cross-site, cross-origin isolation headers are set, and HTTPS
  deployments emit HSTS. The workbench requires neither `unsafe-inline` nor
  third-party assets.

For internet-facing deployments: terminate TLS with a reverse proxy (nginx,
Caddy, or the cloud load balancer), mount the token as a private file, configure
the public HTTPS origin, and keep `--bind` on a private interface unless the
proxy sits beside it.

## Custody

NeoNexus supports a catalog containing three signer backend families. The
recommended `NEONEXUS_SIGNER_PROFILES_FILE` points at a bounded TOML registry
of named profiles. `console_backend` and `relay_backend` are independent
control-plane routes. Each managed node selects zero or one complete
`(backend_id, key_id)` binding; signing duties require one, and there is no
process-wide default. An unavailable or refusing target is never replaced by
another profile. Leave all signer settings absent to run non-signing nodes.

| Kind | Boundary | Capabilities |
|------|----------|--------------|
| `local-wallet` | One encrypted NEP-6 account, selected per node | Native node wallet signing plus NeoNexus's strictly parsed transaction lane. Raw is explicit opt-in; the application consensus API remains closed. No NeoX, remote administration, durable audit, or public relay |
| `local-signer` | A separately deployed `secure-sign-service-rs` endpoint on loopback/vsock | Official Neo SecureSign gRPC, consensus-only. No HTTP console, policy/caller/audit, transaction/raw signing, or relay |
| `neo-os-service` | NeoOS HTTPS custody plus an optional local node bridge | Full HTTP custody/control plane; native neo-cli consensus additionally requires the separately configured SecureSign-compatible gRPC bridge |

`local-wallet` reads an encrypted wallet and one-line passphrase from protected
regular files. Startup decrypts the selected account to verify its Neo N3
address, P-256 public key, and verification script, then pins the wallet SHA-256.
Every signing operation rereads the wallet and refuses a changed file. The
passphrase is read only at startup and then zeroized. One
`Arc<Zeroizing<[u8; 32]>>` private scalar remains in process memory until the
profile's final clone drops; it is not memory-locked and may still be exposed by
a privileged process dump or swap policy. Removing or rotating the password file
does not relock an already running profile. Neither secret can be entered or
returned through the web UI. Transaction requests are fully consumed as a
canonical Neo N3 unsigned envelope and must name the wallet among the signers;
request-id conflicts are rejected by a bounded process-local ledger. This is not
the service's policy engine, durable audit, or anti-equivocation system.

Only `neo-os-service` uses the HTTP contract in `docs/SIGNER_SERVICE.md` in the
`neo-os-services` repository. Sealed keys, boundaries, callers, and durable
audit stay in that signer process; NeoNexus adds the browser control plane and a
bounded, pre-body-admitted, timed relay for program callers. A service profile
may configure one admin identity and a different restricted signing identity;
NeoNexus never uses the admin identity on signing routes.

| Variable | Meaning |
|----------|---------|
| `NEONEXUS_SIGNER_PROFILES_FILE` | Recommended registry document; supports multiple named backends and explicit console/relay/internal routes. See `docs/signer-profiles.example.toml` |
| `NEONEXUS_SIGNER_BACKEND` | Single-profile compatibility selector: `local-wallet`, `local-signer`, or `neo-os-service` |
| `NEONEXUS_SIGNER_LOCAL_WALLET_PATH` | Encrypted NEP-6 wallet file; required by `local-wallet` |
| `NEONEXUS_SIGNER_LOCAL_WALLET_PASSWORD_FILE` | Protected, bounded one-line wallet passphrase file; required by `local-wallet` |
| `NEONEXUS_SIGNER_LOCAL_WALLET_ACCOUNT` | Optional address selecting one wallet account; selection fails when an omitted value would be ambiguous |
| `NEONEXUS_SIGNER_LOCAL_WALLET_NETWORK` | Required non-secret network label recorded with the local key identity |
| `NEONEXUS_SIGNER_LOCAL_WALLET_NETWORK_MAGIC` | Required Neo N3 `u32` transaction magic; mainnet/testnet must match the shared canonical network table |
| `NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_TRANSACTION` | Local-wallet transaction lane; default `true` |
| `NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_CONSENSUS` | Must remain `false`; `true` fails startup until typed consensus parsing and durable anti-equivocation exist |
| `NEONEXUS_SIGNER_LOCAL_WALLET_ALLOW_RAW` | Local-wallet unframed signing lane; default `false` |
| `NEONEXUS_LOCAL_SIGNER_ENDPOINT` | Official SecureSign gRPC endpoint; literal loopback IP with explicit port, or `vsock://cid:port` |
| `NEONEXUS_LOCAL_SIGNER_PUBLIC_KEY` | Exact compressed P-256 public key owned by the local signer |
| `NEONEXUS_LOCAL_SIGNER_NETWORK_MAGIC` | Non-zero Neo N3 network magic pinned by the local signer |
| `NEONEXUS_SIGNER_URL` | Mandatory HTTPS origin for a NeoOS HTTP service; no userinfo, path, query, or fragment |
| `NEONEXUS_SIGNER_SERVICE_URL` | Deprecated one-release alias for `NEONEXUS_SIGNER_URL`; setting both fails closed |
| `NEONEXUS_SIGNER_ADMIN_TOKEN_FILE` | Protected, bounded one-line bearer token file for an identity holding exactly `admin` and a whole-vault grant |
| `NEONEXUS_SIGNER_ADMIN_CALLER_ID` | Admin workload caller id; requires `NEONEXUS_SIGNER_ADMIN_WORKLOAD_KEY_FILE` instead of the token file |
| `NEONEXUS_SIGNER_ADMIN_WORKLOAD_KEY_FILE` | Protected file containing exactly one 64-lowercase-hex Ed25519 seed line |
| `NEONEXUS_SIGNER_ADMIN_WORKLOAD_SUBJECT` | Optional subject pinned to the configured workload caller; it is signed but not sent as a header, so the signer restores it from the caller record. Assertions use audience-bound `neoos-workload-v2`; the canonical `NEONEXUS_SIGNER_URL` origin must exactly equal the service's `SIGNER_SERVICE_WORKLOAD_AUDIENCE` |
| `NEONEXUS_SIGNER_SERVICE_ORIGIN` | Optional exact `Origin` for the console's admin caller, e.g. `https://nexus.internal.example`. It is not applied to relayed signing callers |
| `NEONEXUS_SIGNER_SERVICE_TIMEOUT_SECONDS` | Per-request timeout, default 10. The calls are blocking and run off the request thread |

The registry file cannot be combined with compatibility variables. It rejects
duplicate ids, unknown fields, cross-kind fields, invalid routes, a local wallet
as public relay, a service selected for internal signing without a separate sign
identity, non-loopback local signer endpoints, and any cleartext NeoOS HTTP service. Relative secret
paths resolve beside the registry. A single-service compatibility URL still
requires exactly one admin profile. `NEONEXUS_SIGNER_SERVICE_TOKEN` is refused
even when blank.

Secret readers use one cross-platform fail-closed implementation. Unix opens
with no-follow semantics and rejects group/other mode bits. Windows checks the
opened handle's DACL and owner, rejecting reparse points, null/complex DACLs, and
read/write/execute grants to identities other than the owner, LocalSystem,
Administrators, or Owner Rights. Default inherited Temp ACLs commonly fail this
check; explicitly protect production files instead of weakening validation.

For one migration release, HTTPS service settings without
`NEONEXUS_SIGNER_BACKEND` retain their previous NeoOS-service interpretation.
Untyped cleartext is refused; new deployments should use the registry.
This compatibility path does not introduce fallback: a configured backend that
fails remains failed. An unreachable service returns
`503 signer-service-unavailable`; NeoNexus does not consult the local wallet or
another endpoint.

With a service backend, the console can generate a key remotely, edit policy
(including `allow_raw`), manage callers, and read audit. It exposes no WIF,
raw-private-key, NEP-2, or passphrase input and no import route. Existing keys
are imported at the signer service's trusted operator boundary until v2
attestation gives a remote console enough evidence to authenticate the receiving
enclave and vault. With `local-wallet`, the page shows only its non-secret
identity, pinned wallet digest, network, and enabled capabilities; service-only
Keys, Policy, Callers, Audit, and API controls are absent.

The signing API at `/signer/api/v1/*` is enabled only for `neo-os-service`; it
is never connected to `local-wallet` or the consensus-only `local-signer`. It is outside the
session layer: a caller
presents its own bearer token or six-header audience-bound Ed25519 workload assertion and an
optional browser `Origin`/`Referer`. The workbench forwards only those signer
authentication inputs, the exact path/query, and the exact bounded body bytes;
it never injects its admin identity or reserializes JSON. Which credentials are
valid, which origins match, which keys a caller may reach and what a key's
boundary permits are all the service's decisions and audit rows.

## Surfaces

Every destination lives in one table — `src/web/nav.rs` — and the end-to-end
suite walks it, so a page in the sidebar is tested for its auth boundary whether
or not anyone remembers to write a test for it.

| Group | Route | What it shows |
|-------|-------|---------------|
| Overview | `/` | Fleet posture, attention queue, host pressure, and node table (live polling) |
| Fleet | `/nodes` | Search, status filter, and per-row View / Edit / Delete |
| | `/nodes/new` | Register a node |
| | `/nodes/{id}` | Config facts, launch command, plugins, RPC health, Start/Stop/Restart |
| | `/nodes/{id}/edit` | Change a node's client, ports, binary or arguments |
| | `/nodes/{id}/delete` | Confirmation step before an irreversible removal |
| | `/monitor` | Health: managed process CPU/memory/uptime, missing-process first |
| | `/logs` | One node's supervised log tail with pattern diagnosis |
| Operations | `/operations` | Fleet readiness summary and launch blockers |
| | `/events` | Searchable event journal with severity and row-limit filters |
| | `/alerts` | Routing policy, delivery history (targets redacted) |
| Network | `/federation` | Peer NeoNexus servers with their last probe |
| | `/federation/{id}/probes` | Probe history for one server |
| | `/roles` | Private-network duty × client support matrix and node role plan |
| Assets | `/runtimes` | Installed binaries and catalog profiles, with verification state |
| | `/snapshots` | Fast-sync archives and how far each has reached |
| | `/plugins` | Plugins applicable to one node's runtime, enable/disable |
| | `/config` | Per-node managed config path and whether it was written |
| | `/wallets` | Validated wallet metadata — never keys, passwords, or wallet bytes |
| Security | `/signer` | All loaded signer profiles, capabilities, and explicit route roles; service inventory/boundaries/callers/audit for the selected console profile, plus local-wallet public identity without secret or relay controls |
| Workspace | `/metrics` | Metrics snapshot text + Prometheus exposition |
| | `/settings` | Watchdog and monitor policies; runtime upgrade facts |
| Public | `/login` | Token sign-in |
| | `/healthz` | Liveness JSON for load balancers |
| | `/api/public/status` | Aggregate federation counts only; no node identity, version, host, process, or health inventory |

Controls that change state, all plain form posts: `POST /nodes/new` and
`POST /nodes/{id}/edit` register or update a node, `POST /nodes/{id}/delete`
removes it; `POST /nodes/{id}/start`, `/stop`, `/restart` drive the lifecycle;
`POST /plugins/{id}/toggle` and `POST /federation/{id}/toggle` flip a flag;
`POST /config/export` writes the workspace config set;
`POST /settings/watchdog`, `/settings/rpc-health`, `/settings/federation` and
`POST /alerts/routing` save policies; `POST /runtimes/install` queues a runtime
install as a background job; `POST /logout` ends the session.

Authenticated API: `GET /api/fleet`, `GET /api/readiness`, and
`GET /api/metrics-prometheus`. Public endpoints are only `/healthz` and the
inventory-minimized `GET /api/public/status`; all other API routes require the
session cookie or, for `/signer/api/v1/*`, their own signer caller proof.

## Cloud deployment sketch

```bash
# on the server
umask 077
openssl rand -hex 32 > /run/secrets/neonexus-web-token
./neo-nexus --web --bind 0.0.0.0 --port 8080 \
  --web-token-file /run/secrets/neonexus-web-token \
  --web-public-origin https://nexus.example.com
```

- Terminate TLS in a reverse proxy: it accepts public `:443` and forwards to
  the `--bind`/`--port` the workbench actually listens on.
- On Windows, grant the service identity exclusive read access to the token
  file with ACLs; Unix permission checks are enforced at startup.
- Point your Prometheus scraper at `/api/metrics-prometheus` behind the same
  auth or an internal route.
- `systemd`/`docker` restart policies are enough; SQLite is crash-safe and
  the supervisor reconciles transient status at startup.

## Testing posture

- `tests/web.rs` boots a real server on `127.0.0.1:0`, drives login with
  wrong/right tokens, verifies page + API auth, creates a node through the
  repository, and exercises the stop path end-to-end over HTTP.
- `make web-smoke` runs the binary against a throwaway workspace in CI and
  asserts `/healthz` is public while `/api/fleet` is not.
- `make signer-compat` delegates to the service-owned harness, auto-detecting
  either a sibling checkout or `../neo-os/neo-os-services` (with
  `NEO_OS_SERVICES_DIR` available as an explicit override). It
  builds and starts a real Rust signer on an ephemeral loopback port, provisions
  a file-backed admin credential, exercises every supported v1 route, and
  removes all generated Neo N3/NeoX keys, callers, credentials, process state,
  and vault data on every exit. Both repositories run it in Linux CI.
