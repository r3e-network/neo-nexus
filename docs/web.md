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
- **One core pipeline.** Browser controls call the same `core::lifecycle`
  functions the CLI actions call; there is no web-only control path.

## Launch options

| Option | Default | Meaning |
|--------|---------|---------|
| `--web` | — | Explicit spelling of the default server mode |
| `--bind <addr>` | `127.0.0.1` | Listen address; `0.0.0.0` on cloud hosts |
| `--port <port>` | `8080` | Listen port |
| `--web-token <token>` | generated | Operator sign-in token |

The workspace root is `NEONEXUS_DATA_DIR` or the OS data directory, the same
convention the CLI uses. Everything beside `neonexus.db` (managed configs,
supervised logs) lives where the CLI puts it.

## Authentication

- One operator token per deployment. Resolution order: `--web-token` →
  `NEONEXUS_WEB_TOKEN` → generated once per launch and printed to stdout.
- The store keeps only the token's SHA-256 digest and compares digests, so
  token bytes never sit in memory and length leaks nothing.
- A successful login mints a UUID session id kept server-side with a 12-hour
  sliding expiry; the browser receives it as `HttpOnly; SameSite=Lax` cookie
  (`neonexus_session`). Logout deletes it.
- Pages redirect to `/login` without a session; API routes answer `401` so
  the polling script can send the browser back to the login page.

For internet-facing deployments: terminate TLS with a reverse proxy (nginx,
Caddy, or the cloud load balancer), set `--web-token` from your secret store,
and keep `--bind` on a private interface unless the proxy sits beside it.

## Surfaces

Every destination lives in one table — `src/web/nav.rs` — and the end-to-end
suite walks it, so a page in the sidebar is tested for its auth boundary whether
or not anyone remembers to write a test for it.

| Group | Route | What it shows |
|-------|-------|---------------|
| Fleet | `/` | Fleet counts, host CPU/memory, fleet table (live polling) |
| | `/nodes` | All nodes with status badges |
| | `/nodes/{id}` | Config facts, RPC health trend, Start/Stop/Restart |
| | `/monitor` | Managed process CPU/memory/uptime, missing-process first |
| | `/logs` | One node's supervised log tail with pattern diagnosis |
| Operations | `/operations` | Fleet readiness summary, event journal (latest 50) |
| | `/alerts` | Routing policy, delivery history (targets redacted) |
| | `/federation` | Peer NeoNexus servers with their last probe |
| | `/federation/{id}/probes` | Probe history for one server |
| | `/roles` | Duty × client support matrix, and a node's role plan |
| Assets | `/runtimes` | Installed binaries and catalog profiles, with verification state |
| | `/plugins` | Plugins applicable to one node's runtime, enable/disable |
| | `/snapshots` | Fast-sync archives and how far each has reached |
| | `/wallets` | Validated wallet metadata — never keys, passwords, or wallet bytes |
| | `/config` | Per-node managed config path and whether it was written |
| Insights | `/metrics` | Metrics snapshot text + Prometheus exposition |
| | `/settings` | Watchdog and monitor policies; runtime upgrade facts |
| Public | `/login` | Token sign-in |
| | `/healthz` | Liveness JSON for load balancers |

Controls that change state, all plain form posts: `POST /nodes/{id}/start`,
`/stop`, `/restart`; `POST /plugins/{id}/toggle`;
`POST /federation/{id}/toggle`; `POST /config/export`;
`POST /settings/watchdog`, `/settings/rpc-health`, `/settings/federation`;
`POST /alerts/routing`; `POST /logout`.

API: `GET /api/fleet`, `GET /api/readiness`, `GET /api/metrics-prometheus`.
All require the session cookie except `/healthz`.

## Cloud deployment sketch

```bash
# on the server
export NEONEXUS_WEB_TOKEN="$(openssl rand -hex 32)"
./neo-nexus --web --bind 0.0.0.0 --port 8080 --web-token "$NEONEXUS_WEB_TOKEN"
```

- Terminate TLS in a reverse proxy: it accepts public `:443` and forwards to
  the `--bind`/`--port` the workbench actually listens on.
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
