# Integration suite (`cargo test --test integration`)

Mounted by `tests/integration.rs`. Every test drives the `neo_nexus` library
through its public API; nothing in this directory stands in for product code.

| File | Tests | What they check |
|---|---|---|
| `lifecycle.rs` | 9 | The lifecycle core the web workbench and `--node-start` / `--node-stop` / `--node-restart` share: `execute_node_launch` and `stop_node_runtime` over a real SQLite workspace, with the real `LaunchPlanner`, `ConfigExporter` and `ProcessSupervisor` launching a real child process. |
| `node_types.rs` | 8 | Public node contracts: node-id and port validation, node-type names, chain family, default storage engine, plugin support, and that each `NodeTypeTraits` config path carries its declared format's extension. |
| `common.rs` | — | A throwaway workspace (temp directory + SQLite repository) and the launch inputs `--node-start` derives for a node. |

For each of neo-cli, neo-go, neo-rs, neox-geth and neox-rs, a start renders the
managed config in the generator's format at the path the launch command points
the client to, persists `Running` with the pid of a live process, and logs the
launch; a stop ends that process, persists `Stopped` with no pid, and logs the
stop. Further tests cover restart (new pid, old process gone); all five types
running under one supervisor while a sixth node with a missing runtime fails
alone in `Error`, and stopping one of the five without touching the others; a
node that failed on a missing runtime starting once a real one is rebound; and
a dropped supervisor ending every process it still managed.

Not covered here:

- Real Neo clients. The runtime is `tests/support/stub_runtime.rs`, a compiled
  stand-in that ignores its arguments and sleeps; client behaviour, sync and
  RPC are not exercised.
- Readiness evaluation, signer-backed duties and plugins: these launches pass no
  signer registry and no plugins.
- A runtime that traps or ignores SIGTERM (graceful versus forced stop): see
  `tests/domain/config_launch_supervisor/supervisor_logs/processes/termination.rs`
  (Unix only).
- Log parsers and the adapter registry: unit tests in
  `tests/unit/supervisor/log_parsers.rs` and `tests/unit/supervisor/adapters.rs`.
- HTTP routes: `tests/web.rs`.

The stand-in is built once per checkout with `rustc` into cargo's
`CARGO_TARGET_TMPDIR` (`target/tmp/`); each test's workspace is a `TempDir`
removed when the test ends, so a run leaves nothing in the system temp
directory.
