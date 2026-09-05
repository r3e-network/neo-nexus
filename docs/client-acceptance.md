# Client acceptance matrix

NeoNexus's local suite proves the control paths with fixtures. Production
acceptance needs the five real clients against a real network. This document
defines the matrix and the exact commands that produce the evidence, so it can
run on any host that has the binaries without changing the product. The
checklist below is the pass/fail contract; a run is accepted only when every
row passes against a live node.

## Environment contract

The operator supplies:

- an installed `neo-nexus` binary (release build),
- paths + network params for the five clients: `neo-cli`, `neo-go`, `neo-rs`
  (`neo-node`), `neox-geth` (`neox-geth`), `neox-rs`,
- a reachable RPC endpoint per client (N3 clients speak Neo JSON-RPC; Neo X
  clients speak Ethereum JSON-RPC),
- a workspace directory for `neo-nexus` to supervise into.

No credential is supplied on the command line or stored in the harness: any
wallet/signer secrets come from the environment variables or files the client
and NeoNexus already read.

## Matrix rows

### 1. Identity smoke

For each client, confirm the binary reports the expected version and that
NeoNexus recognizes it:

```
neo-nexus --runtime-smoke <type> <binary>
neo-nexus --rpc-health <endpoint> [neo-n3|neo-x]
```

Pass: runtime smoke is `passed`; RPC health is `healthy` against the correct
family. This is the same evidence a release transaction uses as its acceptance
gate, so a failure here blocks any upgrade.

### 2. Managed lifecycle

Register and supervise each client through the shared pipeline. Register the
node from the Web workbench **Nodes → Add node** (or import it through the
repository API), then use the same lifecycle commands from the CLI:

```
# Web: Nodes → Add node (there is intentionally no --node-create CLI command)
neo-nexus --node-start <db> <name>
neo-nexus --node-status <db> <name>
neo-nexus --node-stop  <db> <name>
```

Pass: start reaches `Running` with a real PID; status shows the recorded PID
and a matching RPC observation; stop returns the node to `Stopped` and the
process is gone. This exercises the fenced operation ledger and the guardian's
PID identity check.

### 3. Long-run soak

Start the node and let it run for the soak window (default 1 hour). Poll RPC
health every 30 s and record block progress:

```
neo-nexus --rpc-health <endpoint> [family]        # polled
```

Pass: no `Unreachable` after the first successful probe; N3 `getblockcount` /
NeoX `eth_blockNumber` strictly increases at least once across the window for a
node that should be producing or syncing. For consensus nodes, committee and
height are stable and advancing.

### 4. Consensus / signing boundary

For a consensus-eligible node, confirm the chain agrees and the configured
signing boundary is exercised, without NeoNexus holding a private key:

```
neo-nexus --governance <endpoint>          # N3 committee / validators
neo-nexus --designation <endpoint> <role> [public-key]
```

Pass: governance reads return the committee and validators; the node's key (if
it signs) is designated for the expected role, and the signer page shows the
referenced custody service healthy. Signing is performed by the separately
deployed NeoOS signer / client-native wallet; NeoNexus reports references and
metadata only.

### 5. Cross-version upgrade

With the node stopped, install the new client version and publish it as one
release transaction:

```
neo-nexus --release-transaction <db> <name> <version>
neo-nexus --node-start <db> <name>
```

Pass: the transaction commits and the node record shows the new version; after
start the node runs and passes RPC health against the new client; the managed
config was regenerated. A version whose acceptance smoke fails must roll back
and leave the node on the old version (verify `--node-status` still shows the
old version and the release transaction is `rolled-back`).

### 6. Kill / recovery

Kill the supervised process out-of-band (SIGKILL on Unix, TerminateProcess on
Windows) and wait for the watchdog to attempt a bounded recovery:

```
# kill the node's PID recorded by --node-status
neo-nexus --node-status <db> <name>
```

Pass: the node transitions to `Crashed`; the watchdog schedules a bounded retry
(up to the configured attempts, 5/10/20 s backoff) and the node returns to
`Running` with a new PID within the budget; a recycled PID is never signalled.
An explicit `--node-stop` cancels pending recovery.

## Evidence report

Run `scripts/client-acceptance-matrix.sh` with a JSON environment file:

```json
{
  "db": "/path/to/acceptance.db",
  "soak_seconds": 3600,
  "clients": [
    { "type": "neo-cli", "binary": "/opt/neo-cli/neo-cli", "endpoint": "http://127.0.0.1:10332", "family": "neo-n3" },
    { "type": "neo-go",  "binary": "/opt/neo-go/neo-go",    "endpoint": "http://127.0.0.1:20332", "family": "neo-n3" },
    { "type": "neo-rs",  "binary": "/opt/neo-rs/neo-node",  "endpoint": "http://127.0.0.1:30332", "family": "neo-n3" },
    { "type": "neox-geth", "binary": "/opt/neox/geth",      "endpoint": "http://127.0.0.1:8545",  "family": "neo-x" },
    { "type": "neox-rs",  "binary": "/opt/neox/neox-rs",    "endpoint": "http://127.0.0.1:8546",  "family": "neo-x" }
  ]
}
```

The script writes `acceptance-report.json` with one row per matrix criterion,
its pass/fail verdict, and the captured evidence (version, block count, PID
transitions, release phase). CI or an operator can gate on `success: true`.

This document deliberately does not claim these rows ran here: NeoNexus cannot
bring up real consensus clients or a live network in its own test environment.
Running the harness requires the environment contract above.
