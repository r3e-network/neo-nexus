# Node operations audit — 2026-09-05

The priority of this audit is reliable process ownership and recovery. Hermes
uses the same guarded operations as the workbench; the local supervisor remains
responsible for nodes even when Hermes or its language-model provider is down.
Hermes owns conversation channels. NeoNexus does not duplicate Telegram setup.

## Coverage

| Capability | neo-cli | neo-go | neo-rs | neox-rs | geth-neox (`neox-geth`) |
|---|---|---|---|---|---|
| Shared start/stop/restart and bounded crash recovery | Yes | Yes | Yes | Yes | Yes |
| Configuration injection and local-change conflict review | JSON | YAML | TOML | TOML | TOML / arguments |
| Runtime selection and hash/host/type validation | Yes | Yes | Yes | Yes | Yes |
| RPC health and recent observations | N3 | N3 | N3 | EVM | EVM |
| Committee/candidate chain-state page | N3 RPC | N3 RPC | N3 RPC | Family guard | Family guard |
| Hermes scoped observation and lifecycle tools | Yes | Yes | Yes | Yes | Yes |
| Native neo-cli plugin packages and activation | Yes | Different client API | Different client API | Different client API | Different client API |
| NeoOS custody management client | Shared service | Shared service | Shared service | Shared service | Shared service |

“Shared service” does not mean that a node's consensus engine automatically uses
the NeoOS signer. Native consensus signing contracts remain client-specific.

## Guardian and recovery decisions

- Dirty exits become `Crashed`; failures to launch become `Error`. Finite,
  backoff-based recovery continues after failed automatic launch attempts and
  stops when its budget is exhausted. An explicit Stop cancels queued recovery.
- Workbench startup reconciles recorded PIDs. A live matching process is kept;
  a missing process becomes Crashed and enters recovery; a recycled PID becomes
  Error and is never signalled or automatically replaced.
- Old exit notifications and delayed RPC responses cannot overwrite a replacement
  process. Launch and stop recheck the node under the shared process lock.
- Windows nodes are placed in separate console process groups. A targeted helper
  sends CTRL_BREAK and waits; an unresponsive process can be force-stopped.
  Hermes uses its own planned-stop command before fallback process termination.
- Managed companions have separate desired-running state, retry budgets and
  binary/configuration fingerprints. Hermes source changes also require review.
  Backup restore carries profiles, not live PIDs or desired-running state.

The workbench server must stay running for continuous supervision. A one-shot
CLI launch by itself does not create a background guardian. Operating-system
service management of the workbench remains a deployment responsibility.

## Observability and alerts

RPC replies must have a matching JSON-RPC envelope and valid method-specific
result shape. Invalid/null block counts and versions do not produce Healthy.
EVM `eth_syncing` distinguishes false from a valid syncing object; unavailable or
malformed synchronization data is Degraded. N3 uses its own methods.

Due RPC checks are ordered by the oldest observation and run in bounded parallel
batches, so a fleet larger than one batch is not permanently starved. A checked
timestamp is retained; a historical observation is not a claim of current health.
Signer service health also runs in the background, independent of page views.

Alert consumption follows insertion IDs through the entire retained journal.
Cursor and failed-attempt counts survive server restarts. Failed webhooks have a
three-attempt bound, and do not follow redirects or retain response-body secrets.
An unreadable cursor restarts from the oldest retained event with a diagnostic:
delivery can repeat, but the engine does not silently skip the backlog. This is
at-least-once best effort, not exactly-once delivery.

Hermes can query node events and logs through MCP. Proactive conversations or
scheduled polling are configured in Hermes; configuring this integration does
not send a Telegram message or create an autonomous Hermes schedule.

## Configuration, versions and plugins

See [configuration and versions](configuration-versions.md). Generated, accepted
and current hashes detect local drift. Candidate files, backups and review tokens
allow Keep local or Adopt generated while rejecting stale reviews. Runtime
changes require renewed review even when the generated text has not changed.
Conflict preflight runs before stopping a node or changing its recorded version.

neo-cli package updates retain operator configuration and key files. Disabling
a plugin moves it out of the Plugins load tree; it is not only a database flag.
Declared plugin versions and compatible runtime versions are checked before
installation or runtime selection. These declarations do not prove arbitrary
assembly ABI compatibility. Full multi-file client releases must be deployed
with their dependencies; the runtime installer manages an executable asset.

Unknown future configuration schemas cannot be inferred from a version string.
External `--config` paths remain operator-owned and require explicit handling.

## Signing boundaries

The [NeoOS signer client](signer-service.md) uses the actual audience-bound
workload-v2 protocol. NeoX requests use `neox`, an explicit numeric chain ID and
the full signed transaction response. Health checks carry no custody credential;
signing requests have no automatic retry or local-key fallback. The workbench
manages references and public metadata, not imported custody private keys.

neo-cli's official SignClient configuration is injected as
`PluginConfiguration.Name` and `Endpoint`; the endpoint targets the local NeoOS
N3 bridge. The bridge, not the node configuration, holds caller/key selection.
The upstream DBFT plugin's AutoStart path uses a wallet. Remote signing still
requires `start consensus SignClient`; the current managed child has no console
command channel, so unattended remote-signer consensus startup is not complete.
See the [upstream DBFT implementation](https://github.com/neo-project/neo-node/blob/v3.10.1/plugins/DBFTPlugin/DBFTPlugin.cs).

neo-go's native wallet unlock configuration, neo-rs wallet/HSM integration,
geth-neox wallet/Clef path and neox-rs validator key interface are different
contracts. A working REST custody client is not a replacement for those native
consensus adapters. Missing adapters remain explicit; no fabricated endpoint is
injected into an unsupported node.

## Verification and remaining deployment evidence

Final Windows verification: `cargo test --all-targets --quiet` passed **714**
tests, with three intentionally ignored cases (one child-process fixture and two
external signer contracts). The two signer contracts were separately run against
the adjacent Rust signer service and passed. Fifteen supervision regressions also
passed on Ubuntu under WSL. Formatting, all-target Clippy with warnings denied,
source purity/quality, CI policy, self-check, Cargo Audit and redacted Gitleaks
scans of `src` and `tests` passed. No live channel credentials were used.

Regression coverage includes Windows and Linux process termination/recovery,
startup reconciliation, malformed RPC responses, alert backlog/retry recovery,
configuration conflicts, version selection, plugin layout and signer contracts.
HTTP tests exercise a real local server, scoped MCP permissions, revocation,
Hermes configuration preservation, and real supervised process start/restart/stop.
Hermes configuration tests use local fixture files, not live channel credentials.

Five real blockchain networks have not been started by this audit, and no live
Hermes/LLM/Telegram conversation has been performed. Native client acceptance
must still verify chain synchronization, consensus signing, upgrade behavior and
recovery with the operator's actual versions and credentials. macOS process
behavior is not covered by the Windows/Linux runs.
