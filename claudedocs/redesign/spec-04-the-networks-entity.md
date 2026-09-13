### 1.1 The `networks` entity

`Network` is an enum with three variants (`src/types/network.rs`) that today means two different things at once: *which published chain* and *whether a chain is published at all*. Split it.

```sql
CREATE TABLE IF NOT EXISTS networks (
    id                  TEXT PRIMARY KEY,       -- 'neox-mainnet' | 'neox-testnet' | 'net-<uuid>'
    name                TEXT NOT NULL,
    family              TEXT NOT NULL,          -- ChainFamily::slug(): 'neo-n3' | 'neo-x'
    kind                TEXT NOT NULL,          -- 'public' | 'private'
    builtin             INTEGER NOT NULL DEFAULT 0,   -- seeded, read-only
    parent_network_id   TEXT REFERENCES networks(id), -- neox-mainnet -> neo-n3-mainnet

    -- Neo N3 identity
    network_magic       INTEGER,
    validators_count    INTEGER,
    seed_nodes          TEXT,                   -- JSON array of host:port
    committee_keys      TEXT,                   -- JSON array of compressed secp256r1 pubkeys

    -- Neo X identity
    chain_id            INTEGER,                -- EIP-155
    genesis_hash        TEXT,                   -- 0x + 64 hex, the runtime anchor
    genesis_path        TEXT,                   -- managed artefact on disk
    genesis_sha256      TEXT,                   -- of the file bytes, the fleet-wide identity
    reth_chain_spec     TEXT,                   -- '--chain' value: preset name or path
    block_period_secs   INTEGER NOT NULL DEFAULT 15,

    created_at_unix     INTEGER NOT NULL,
    updated_at_unix     INTEGER NOT NULL
);
ALTER TABLE nodes ADD COLUMN network_id TEXT REFERENCES networks(id);
```

**Seed data replaces the constants.** `MAINNET_BOOTNODES`, `TESTNET_BOOTNODES`, `neox_genesis_hash`, `neox_chain_id`, `neox_reth_chain`, `neox_block_period_secs`, `neox_validator_count` (`src/config/format/neox.rs`) become the *seeder's* input and nothing else's. After migration there is exactly one reader of chain identity — the loaded row — which makes G22's three-places drift structurally impossible rather than a thing to remember. The functions stay (they are correct, sourced, and tested) but become `pub(in crate::repository::seed)`.

**Migration.** Idempotent, no data loss, runs on open:

1. Create `networks`; insert four builtin rows: `neo-n3-mainnet`, `neo-n3-testnet`, `neox-mainnet` (chain_id 47763, genesis `0x2ee574…dbd7`, spec `neox-mainnet`, parent `neo-n3-mainnet`), `neox-testnet` (12227332, `0x221f7d…eb71`, `neox-testnet`, parent `neo-n3-testnet`), block period 5s for the Neo X rows, 15s for N3.
2. For each existing node: `network in (mainnet,testnet)` → `network_id = '<family>-<network>'`.
3. For each existing node with `network = 'private'`: create `net-<node_id>` with `kind='private'`, `family` from `node_type.family()`, **all identity columns NULL**, name `"<node name> private network"`. This is deliberately per-node and deliberately incomplete: it makes the existing silent breakage (G18, G22) visible as an unfinished network record the operator can complete and merge, rather than converting it into a fabricated default.
4. `nodes.network` is retained and kept in sync (`networks.kind='private' → 'private'`, else the public name) until every reader is migrated, then dropped in a later release. Nothing in stage 1 reads it.

**Editing.** `/networks` (list) and `/networks/{id}` (detail + edit form). Builtin rows are read-only except for an operator-supplied `genesis_path` (public Neo X networks need a genesis *file* the operator gets from their distribution; NeoNexus ships the expected hash, never the bytes). Private rows are fully editable while no member node is running; editing chain id or genesis while members exist requires a typed confirmation and marks every member `NeedsReinit` (§2.4).

### 1.2 Flag-value extraction (the missing primitive)

`src/launch/argv_read.rs`. Used by chain identity, datadir, namespaces, WS, peers, ports, and the display of every one of them.

```rust
pub enum ArgError { MissingValue(String), Malformed { flag: String, value: String } }

/// Last occurrence wins — both geth (urfave/cli) and reth (clap) resolve
/// repeats that way, and an operator who pasted a flag twice means the second.
pub fn flag_value<'a>(args: &'a [String], flag: &str) -> Result<Option<&'a str>, ArgError>;
pub fn flag_present(args: &[String], flag: &str) -> bool;
pub fn flag_values_csv<'a>(args: &'a [String], flag: &str) -> Result<Option<Vec<&'a str>>, ArgError>;
pub fn flag_u64(args: &[String], flag: &str) -> Result<Option<u64>, ArgError>;
```

Rules, all of which have a test:

- `--flag value` and `--flag=value` both parse. `--flag=` is `Malformed`, not empty-string-accepted.
- A value beginning with `--` is `MissingValue`, not a value. (`--networkid --http` is an operator error that must surface, not a chain id of `--http`.)
- Non-numeric for a numeric flag is `Malformed` → **Critical** with the offending text quoted. Today it is indistinguishable from absent.
- The existing `has_flag` in `src/launch/neox.rs:121` and `has_chain_argument` in `src/diagnostics/checks/chain.rs:85` both collapse into this module. `--datadir.chain` is removed from the chain-detection list entirely: it selects a *subdirectory name*, not a chain, and its presence currently clears a safety gate while selecting nothing.

**Parity gate (test, not review):** for every value-taking flag the launch planner can emit, `argv_read` must have an extractor, asserted by a table test over a single `NEOX_FLAGS` declaration that both the emitter and the reader consume. This is what stops "we emit it but cannot see it" from regrowing.

### 1.3 Resolution, validation, reporting

```rust
pub struct NeoXChainIdentity {
    pub network: NetworkRecord,
    pub chain_id: Attested<u64>,
    pub genesis_hash: Attested<String>,
    pub chain_spec: Attested<ChainSpecRef>,   // Preset(&str) | File{path, sha256, chain_id}
    pub agreement: Agreement,
}
pub fn resolve_neox_identity(node, network, args, last_observation) -> NeoXChainIdentity;
```

- `argv` for geth = `flag_u64(args, "--networkid")`. For neox-rs = `flag_value(args, "--chain")`, then: if it names a preset in `{neox-mainnet, neox-testnet}` → that row's chain id; if it is a path → read the file as JSON and take `config.chainId` (cheap serde, no crypto) plus the file's sha256; if the file is absent or unparseable → `Malformed`, Critical.
- **NeoNexus does not compute a genesis hash offline.** That needs header RLP + keccak and a dependency, and it would still not prove what the client did with the file. The file's sha256 is the artefact identity (and is exactly what an operator can verify on their other hosts: *"every member must use the same genesis, byte for byte"*). The genesis *hash* arrives from the running node (§5) and is compared to `networks.genesis_hash`; for a private network with a NULL hash the first observation records it with provenance (`recorded from <node> at <time>`), never silently.
- **Reporting.** `chain_identity_checks` (`src/diagnostics/checks/chain.rs:26`) renders the resolved struct only. One number appears in the report, from one field. The current shape — a `Pass` line printing `neox_chain_id(node.network, None)` alongside a blocker that has already read the operator's flag — is deleted.

### 1.4 The private-network safety property

> **A node the operator called private must never dial a public network.**

Three enforcement points, because readiness alone is advisory and the current one (`private_chain_spec_check`) is satisfiable by a flag that selects nothing:

**INV-X1 — identity completeness is a launch precondition.** A node on a `kind='private'` network cannot Start until:
- `networks.chain_id IS NOT NULL`, and
- geth: `genesis_path` exists and hashes to `genesis_sha256`, and the node's datadir carries a matching init fingerprint (§2.3);
- neox-rs: `reth_chain_spec` resolves to a file that exists, parses, and whose `config.chainId` equals `networks.chain_id`.

Enforced in `node_lifecycle` before `LaunchPlanner::plan`, returning the same `Result` the signer preflight already uses — so it fails the Start action with a message, not a badge.

**INV-X2 — no public reachability.** For `kind='private'`, the rendered config and plan must satisfy:
- zero bootnodes drawn from the seeded public sets (this is the check worth keeping; the current blanket "a private network must not carry any bootnodes" at `validation/runtimes/neox/geth.rs:60-67` is a **false critical** that makes a correctly-wired private network unlaunchable, and is removed);
- `chain_id ∉ PUBLIC_CHAIN_IDS` where `PUBLIC_CHAIN_IDS = {47763, 12227332, 1, 11155111, 17000}` — a private chain id colliding with a public one means replayable transactions, and it is Critical;
- discovery is off **only when there are no bootnodes** (see §3.2 — today it is unconditionally off for private, which is why a private network cannot self-discover).

**INV-X3 — argv reconciliation.** Because `push_missing` (`src/launch/neox.rs:109`) lets operator flags win (correctly), the flags must be *read*: `Agreement::Overridden` on chain id is a Warning on a public network and a **Critical** on a private one, because on a private network an override is the exact route back onto Neo X MainNet with keys the operator treats as throwaway — the hazard `src/launch/neox.rs:13-24` already documents in prose and cannot currently detect.

CLI parity: `neonexus network show <id> --json`, `neonexus node identity <node-id> --json` emitting the full `Attested` triple. Exit code 1 on `Mismatch`.

---

## 2. Bootstrapping

### 2.1 Datadir as data

Add `nodes.data_dir TEXT` (nullable; NULL = managed default `<workspace>/nodes/<id>/data`, matching `DATA_DIR` at `src/launch/neox.rs:40`). At plan time the resolved value is `argv --datadir` if present, else the column, else the default — and the **resolved** value is written into the `LaunchPlan` and displayed. Today the plan pushes `--datadir` with `push_missing` and the UI never learns which one won. This also removes the free-text-only limitation noted in G40 for the Neo X clients.

Expected layout, per client, used by the readiness checks:

| Client | Chain data | Node key | NeoNexus marker |
|---|---|---|---|
| neox-geth | `<datadir>/geth/chaindata/` | `<datadir>/geth/nodekey` | `<datadir>/.neonexus-init.json` |
| neox-rs | `<datadir>/<chain>/db/mdbx.dat`, `static_files/` | `<datadir>/<chain>/discovery-secret` | — (no init step) |

### 2.2 The `geth init` action

Currently `geth init` is mentioned in a config comment (`generator/neox/geth.rs:79`) and verified nowhere. Make it an action:

`POST /nodes/{id}/neox/init` — supervised one-shot `<binary> init --datadir <resolved> <genesis_path>`, output captured through the existing process-log path so failures land in `/logs`; on success writes `.neonexus-init.json`:

```json
{ "chain_id": 12345, "genesis_sha256": "…", "genesis_path": "…",
  "client": "neox-geth", "client_version": "…", "initialised_at_unix": 1757… }
```

and emits `EventKind::NeoXDataDirInitialised { node_id, network_id, chain_id, genesis_sha256 }`.

Refuses when: the node is running; the network has no genesis artefact; `chaindata` already exists **unless** `reinitialise=1` plus a typed confirmation of the node name — re-init discards the chain, and that must be as loud as a delete.

CLI: `neonexus neox init --node <id> [--reinitialise] [--json]`. This is the first Neo X action that exists on both surfaces by construction, and it closes the dead end where readiness instructed the operator to run a command the console could not run (the G27/G30 pattern).

Be explicit in the UI copy: **the marker file is NeoNexus's own record, not a client guarantee.** The authoritative check is the runtime block-0 hash (§5). The marker exists to catch the wrong-genesis case *before* launching, which is otherwise diagnosed as a network problem (`src/diagnostics/checks/chain.rs:8-12` already describes exactly this failure).

### 2.3 Init state

`node_neox.init_state TEXT` ∈ `{uninitialised, initialised, mismatched, not-applicable}` (neox-rs is always `not-applicable`), plus `init_genesis_sha256`, `initialised_at_unix`. Recomputed on every readiness run from the marker + `chaindata` presence + the network's current `genesis_sha256`.

`mismatched` is reached when the network's genesis artefact changed after init. It is Critical with one remediation: re-initialise (destructive) or correct the network record.

### 2.4 The Neo X readiness contract

What the console must verify before it calls a Neo X node ready. Each is a `DiagnosticCheck` with a `DiagnosticResolution` that actually routes there (`src/diagnostics/model/check.rs:95`); two new resolutions are needed: `Networks → /networks/{id}` and `Peers → /nodes/{id}/peers`.

| # | Check | Severity when failing |
|---|---|---|
| 1 | Network record resolves; identity complete for its kind | Critical |
| 2 | Genesis artefact present and hashes to `genesis_sha256` (geth, and neox-rs when `--chain` is a file) | Critical |
| 3 | geth: datadir initialised, marker matches network genesis | Critical |
| 3b | neox-rs: `--chain` resolves (preset in `SUPPORTED_CHAINS`, or file parses with matching `chainId`) | Critical |
| 4 | Chain-id agreement (declared / argv / observed) | Critical on `Mismatch`; Critical on `Overridden` for private, Warning for public |
| 5 | `http_api` contains `eth` and `net` | Critical (the probe cannot work without them) |
| 6 | Bind addresses are loopback, or `admin`/`personal` not enabled | Critical if a dangerous namespace is exposed off-loopback |
| 7 | Ports planned and free: rpc, p2p (TCP+UDP), ws (if enabled), authrpc, metrics (if enabled) | Critical on collision |
| 8 | Peer set non-empty **or** discovery on with bootnodes | Warning (Critical for private with neither — the node cannot ever find a peer) |
| 9 | Duty is launchable on this client (§6) | Critical |

The current `Pass` that asserts block period and validator count for a private network (`format/neox.rs:75-87`, surfaced at `checks/chain.rs:36-41`) is **deleted**: those are facts about a genesis NeoNexus explicitly refuses to generate, so they are R1 literals. Block period for a private network is either operator-entered on the network record (and then it is declared, and labelled so) or absent.

---

## 3. Peering

### 3.1 Peers as rows

```sql
CREATE TABLE IF NOT EXISTS network_peers (
    network_id TEXT NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    enode      TEXT NOT NULL,
    kind       TEXT NOT NULL,       -- 'bootnode' | 'static' | 'trusted'
    note       TEXT,
    added_at_unix INTEGER NOT NULL,
    PRIMARY KEY (network_id, enode, kind)
);
CREATE TABLE IF NOT EXISTS node_peers (
    node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    enode   TEXT NOT NULL,
    kind    TEXT NOT NULL,
    note    TEXT,
    PRIMARY KEY (node_id, enode, kind)
);
```

Two scopes, because they answer two different questions. In a private network **every member needs every other member's enode** — that is a property of the network, authored once, and it is what `PrivateNetworkPlanner` should emit (G19). Node scope is for the single node behind NAT that needs one extra static peer. Effective set = `network_peers ∪ node_peers`, deduplicated by `(enode, kind)` with node scope winning on `note`.

### 3.2 Client mapping

**neox-geth** — all three kinds are config keys already modelled in `GethP2p` (`generator/neox/geth/model.rs:66-83`):

| Kind | Key | Meaning |
|---|---|---|
| bootnode | `Node.P2P.BootstrapNodes` | discovery seeds |
| static | `Node.P2P.StaticNodes` | always redial, ignore slot pressure |
| trusted | `Node.P2P.TrustedNodes` | always accepted inbound, bypass peer limits |

`NoDiscovery` becomes `effective_bootnodes.is_empty()`, replacing `private` (`generator/neox/geth.rs:26,49`). **This is a behaviour change and it is the point**: a private network *with* bootnodes should discover, which is how members find each other without listing every pair; a network with no bootnodes should not spray UDP. The launch plan's matching `--disable-discovery` for neox-rs (`src/launch/neox.rs:89-91`) changes the same way.

**neox-rs** — split between file and flags, which the generator already understands (`generator/neox/reth.rs:1-12`):

| Kind | Carrier |
|---|---|
| bootnode | `--bootnodes <enode,enode,…>` (launch flag) |
| static / trusted | `[peers].trusted_nodes` in the config file, **and** `--trusted-peers` when the operator supplied them via argv |
| trusted-only | `[peers].trusted_nodes_only` — now a node setting, default off |

`trusted_nodes_only` with an empty trusted list isolates the node permanently. The generator comment at `reth.rs:49-52` already knows this; make it a hard validation error rather than a silent `false`.

### 3.3 One enode parser

Promote `is_enode` (`validation/runtimes/neox/geth.rs:96-104`) to `src/types/enode.rs`:

```rust
pub struct Enode { pub public_key: [u8; 64], pub host: HostRef, pub tcp_port: u16, pub udp_port: Option<u16> }
impl FromStr for Enode { … }   // enode://<128 hex>@host:port[?discport=N]
pub enum HostRef { Ip(IpAddr), Dns(String) }
```

Used by: the form (reject on submit with the reason), both generators, both validators, the peer tables, and the launch flag renderer. A DNS host is accepted for static/trusted and **Warned** for bootnode (discovery needs an address; geth resolves at startup only). `?discport=` is preserved verbatim — dropping it silently changes which UDP port the peer is dialled on.

### 3.4 The node's own enode

The thing an operator actually needs, and the reason private Neo X networking is currently a shell task. `admin_nodeInfo` would give it, but `admin` is off by design and should stay off.

**`GET /nodes/{id}/neox/enode`** derives it locally: read the node key from disk (`<datadir>/geth/nodekey` — 64 hex chars; `<datadir>/<chain>/discovery-secret` for neox-rs), derive the secp256k1 public key, and compose `enode://<pubkey>@<advertised host>:<p2p port>`. Pure public-key derivation, no signing, and it works while the node is **stopped** — which is when the operator is wiring the network. The advertised host is `nodes.host` (the multi-host column from G34) or an operator-entered value, never `127.0.0.1` silently: if the host is unknown, render `enode://<pubkey>@<your host>:30303` with the placeholder visibly a placeholder.

CLI: `neonexus neox enode --node <id> [--json]`. A "Copy all member enodes" action on the private network page emits the full set as a CSV line and as a `network_peers` bulk insert for the other members — the actual workflow, done once.

---

## 4. The RPC surface

### 4.1 Namespaces as data

```sql
-- node_neox facet
http_enabled     INTEGER NOT NULL DEFAULT 1,
http_api         TEXT    NOT NULL DEFAULT 'eth,net,web3,txpool',
http_corsdomain  TEXT,
ws_enabled       INTEGER NOT NULL DEFAULT 0,
ws_api           TEXT,
ws_origins       TEXT,
authrpc_port     INTEGER,
metrics_enabled  INTEGER NOT NULL DEFAULT 0,
metrics_port     INTEGER,
dangerous_ack    INTEGER NOT NULL DEFAULT 0
```

Catalogue as a typed enum with per-client availability and a risk level:

| Namespace | neox-geth | neox-rs | Risk | Why it is on |
|---|---|---|---|---|
| `eth` | ✓ | ✓ | **required** | height, syncing, chain id, genesis, gas price |
| `net` | ✓ | ✓ | **required** | `net_peerCount` |
| `web3` | ✓ | ✓ | low | client version |
| `txpool` | ✓ | ✓ | low | mempool depth |
| `rpc` | ✓ | ✓ | low | `rpc_modules` — makes the namespace set *observable* |
| `debug` | ✓ | ✓ | high | tracing; expensive, can pin CPU |
| `trace` | ✗ | ✓ | high | Parity-style tracing, neox-rs only |
| `admin` | ✓ | ✓ | **dangerous** | add/remove peers |
| `personal` | ✓ (deprecated) | ✗ | **dangerous** | unlocks accounts |

Rules:

1. `eth` and `net` cannot be deselected — the observation layer depends on them, and a node whose probe cannot work is a node that reads Unreachable forever. Validation: Critical if absent from the effective set.
2. A namespace the chosen client does not have is not offered. Changing the client re-validates and names what was dropped. This is the direct fix for G23's *"the two clients expose different RPC surfaces for reasons the operator cannot see"* — the reason becomes a column in the picker.
3. **One default for both clients**: `DEFAULT_HTTP_API = [eth, net, web3, txpool]`, the current geth constant. neox-rs now emits `--http.api eth,net,web3,txpool`. Today it inherits Reth's `STANDARD_MODULES` and `txpool_status` (`chain_state/mempool.rs:149`) silently cannot work on one of the two supported clients.
4. `admin` / `personal` require `dangerous_ack = 1` (a per-node checkbox with the consequence spelled out) **and** a loopback bind. Either off-loopback bind or missing ack is Critical. The existing reasoning at `generator/neox/geth.rs:13-16` is correct and becomes enforced rather than commented.
5. **Provenance, not assertion.** An operator `--http.api` *replaces* the list; it is not additive. The node page shows the effective set as an `Attested<Vec<Namespace>>`: declared from the row, argv when overridden (labelled "replaced by launch flags"), observed from `rpc_modules` when `rpc` is enabled, else "not observable — enable the `rpc` namespace to verify".

### 4.2 WebSocket

The current state is the cleanest single example of R1 in the Neo X surface: a port is validated, reserved, rendered as a URL, and badged **Open**, and for neox-rs nothing ever listens on it.

- `ws_enabled` is the authority. `ws_port.is_some()` means only *reserved*. A reserved-but-disabled port renders **"reserved, not listening"**.
- neox-geth: `WSHost`/`WSPort`/`WSModules`/`WSOrigins` in the config, exactly as `GethNode` already models (`generator/neox/geth/model.rs:50-59`) — the host and port must travel together or geth opens its default 8546, which the model comment already documents.
- neox-rs launch plan gains: `--ws --ws.addr 127.0.0.1 --ws.port <p> --ws.api <ws_api or http_api> [--ws.origins <list>]`, emitted exactly when `ws_enabled`, each value flag through `push_missing`, `--ws` through `flag_present`.
- The **Open** badge is replaced by observed reachability: a TCP connect on the probe tick (invert `is_localhost_tcp_port_available`, which already exists in `src/port_planner/probe.rs`), stored per port. Values: `listening` / `configured, not reachable` / `not configured`. Two lines of real state replacing a literal.

### 4.3 Ports nobody is planning

The port planner reserves rpc, p2p and ws (`src/port_planner/planner.rs`). Neo X clients open more:

- **authrpc / engine API** — geth opens 8551 by default. On a single host with two managed Neo X nodes this is a guaranteed collision that the planner cannot see. Emit `--authrpc.addr 127.0.0.1 --authrpc.port <planned>` (or the client's disable where available) and reserve the port.
- **metrics** — if `metrics_enabled`: geth `--metrics --metrics.addr 127.0.0.1 --metrics.port <planned>`; neox-rs `--metrics 127.0.0.1:<planned>`. Reserve it. This also fixes G16 at the root: `NeoXGethMetricsAdapter::metrics_url` returns a hardcoded `http://localhost:8546/metrics` (`src/supervisor/model/metrics.rs:110`) — geth's **WebSocket** default — and the reth one returns `:9091` (`:132-134`), both discarding `_rpc_port`, so the value is a per-host constant that collides across nodes. With a planned, emitted metrics port the URL becomes real, and scraping it supplies block height and peer count for free.
- **P2P UDP** — geth uses one number for TCP and UDP discovery; reth takes `--port` (TCP) and `--discovery.port` (UDP, defaulting to `--port`). Reserve one number, emit both where the client needs them, and say so in the ports card.

`PortAssignment` grows from `{rpc, p2p, ws?}` to a named map so a client can declare which ports it needs; the planner allocates a contiguous block as it does now.

---

## 5. Observation

Neo X maps onto the observation layer (the sibling spec's `node_observations` table + derived states) with **more** signal than N3, because `eth_syncing` answers directly the question N3 forces you to infer.

### 5.1 Method map

| Observation | Neo N3 | Neo X | Stored |
|---|---|---|---|
| liveness / version | `getversion` | `web3_clientVersion` | `client_version`, `latency_ms` |
| height | `getblockcount` | `eth_blockNumber` | `block_height` (normalised — keep `ProbeMethods::block_count`'s +1, `rpc_health/probe/methods.rs:48-55`) |
| **sync state** | — | **`eth_syncing`** | `sync_state`, `sync_current`, `sync_highest`, `sync_stage` |
| peers | `getpeers` / `getconnectioncount` | `net_peerCount` | `peer_count` |
| mempool | `getrawmempool` | `txpool_status` | `mempool_pending`, `mempool_queued` |
| **chain identity** | magic in `getversion` | **`eth_chainId`** | `observed_chain_id` |
| **genesis** | `getblockhash 0` | **`eth_getBlockByNumber("0x0", false).hash`** | `observed_genesis_hash` |
| fee floor | — | `eth_gasPrice` | `gas_price_wei` |

Cadence: `eth_blockNumber`, `eth_syncing`, `net_peerCount` every tick. `txpool_status` and `eth_gasPrice` every 4th tick (cheap but not free). `eth_chainId` and the genesis hash **once per process lifetime** — they cannot change while a node runs, and re-reading them every tick is the kind of cost that gets an observation layer switched off. Re-read on every start and on any network-record edit.

### 5.2 State mapping

With `P = networks.block_period_secs` (5 for Neo X):

| Condition | State | Notes |
|---|---|---|
| `eth_chainId ≠ networks.chain_id` | **WrongChain** (Critical) | Outranks everything. The node is healthy and useless. |
| `observed_genesis_hash ≠ networks.genesis_hash` | **WrongGenesis** (Critical) | The "sits at block 0 with no peers" failure, named correctly instead of diagnosed as a firewall. |
| `eth_syncing == false` and height advanced within `2P` | **Synced** | |
| `eth_syncing == false` and height unchanged for `12P` (≈60s, configurable per network) | **Stalled** | The classic failure. Unambiguous on Neo X: the client claims it is done. |
| `eth_syncing == {currentBlock, highestBlock, …}` | **Syncing** | Progress = `current/highest`; ETA from the observed rate across the last N samples. |
| `Syncing` and `currentBlock` unchanged across 3 ticks | **SyncStalled** | Distinct from Stalled: the node knows it is behind and is not catching up — remediation names **peers**, not the client. |
| `net_peerCount == 0` | **Isolated** | Reuse `classify_connectivity` (`chain_state/peers.rs:171-177`) but make thresholds network-scoped, not compile-time — a 4-member private network must not read "Healthy" at 3 peers (G12). Default for private = `min(3, members-1)`. |
| probe failed | **Unreachable** | As today. |

Parsing `eth_syncing` must be **lenient**: geth in snap sync adds `syncedAccounts`, `healedBytecodes`, …; reth reports its own stage. Take `currentBlock`/`highestBlock` when present; otherwise record `Syncing(indeterminate)` with the raw stage string preserved for display. Never discard the whole observation because an unknown field appeared — that is how a client upgrade silently blinds the monitor.

**Head lag without an external dependency.** `network_head = max(block_height)` over healthy nodes on the same `network_id`; `head_lag = network_head − node.block_height`. Free, works for private networks, and correct for the multi-node case that motivated it. With a single node on a network, render **"no reference"** — not `0`. Optionally, a per-network `reference_endpoint` for operators who want to compare against a public RPC; if set, it is the reference and its provenance is shown.

`eth_gasPrice` is context, not health: store it, display it, never colour it. It supports exactly one alert condition worth having — "gas price above X for N minutes" for an operator whose relayer pays.

### 5.3 Log parsers: delete the sync path

The register (G11) establishes that **both Neo X parsers gate on strings the clients never emit** — geth on `"Chain imported"` + `block=` against a real `Imported new chain segment blocks=`, reth on `"Block #"` + `"state root"` + `targetheight=` — and that the covering test's fixtures were written to match the parser (`tests/unit/supervision/tests.rs:167-206`), so CI is blind.

Decision: **remove sync-progress extraction from both Neo X parsers.** `eth_syncing` + `eth_blockNumber` are authoritative, structured, and available whenever the node is up. The log path can only be wrong in ways CI cannot see, and it competes with a better source.

Keep the parsers for what logs are uniquely good at — level, timestamp, message, and **fatal-error detection for a process that will not start**, where RPC is unavailable by definition. Two supporting changes:

- Emit `--log.format json` (both clients) from the launch plan, so the parser has a contract instead of a guess. Note that the current geth JSON branch reads `time`/`level`/`msg`/`logger` (`src/supervisor/model/log_parsers.rs`), which is not geth's JSON shape (`t`/`lvl`/`msg`) — fix the field names as part of this, since they are the parser's only remaining gate.
- **Fixture provenance rule:** every log-parser test fixture must be captured from a real client binary and carry a header naming the client and version it came from. Hand-written fixtures are rejected in review. This is the specific discipline that stops G11's tautology from regrowing.

---

## 6. Keys

### 6.1 Two schemes, named

Neo N3 and Neo X key material differ at every layer, and the product currently has one model:

| | Neo N3 | Neo X |
|---|---|---|
| Curve | secp256r1 (NIST P-256) | secp256k1 |
| Container | NEP-6 JSON, scrypt | Web3 Secret Storage V3 keystore |
| Address | Base58Check, version 0x35 | 0x + 20 bytes (keccak of pubkey) |
| Transaction | Neo tx with witnesses, network magic in the signed hash | EIP-155 / typed EVM transactions, chain id in the signature |

```rust
pub enum KeyScheme { NeoN3Secp256r1Nep6, NeoXSecp256k1Keystore }
impl ChainFamily { pub fn key_scheme(self) -> KeyScheme { … } }
```

`signer_keys.scheme` column; `SignerBackendProfile` declares which schemes it can hold — `LocalWallet` → NEP-6 (today's behaviour), new `LocalKeystore` → V3 keystore, `NeoOsService` → declared by its own `GET /keys` response, `LocalSigner` (SecureSign gRPC) → N3 only. `SignerBackendKind` (`src/signing/profile.rs:9-13`) gains the variant rather than overloading `LocalWallet`, because "encrypted wallet file" meaning two incompatible formats is precisely the terminology collapse the register calls out in G44.

### 6.2 The binding rule, enforced once, early

```
node.node_type.family().key_scheme() == key.scheme
```

Checked **at bind time** in `POST /nodes/{id}/signer`, with a message naming both schemes; and the key picker only lists compatible keys. Today the equivalent check lives at the bottom of `ensure_local_wallet_runtime` (`src/core/node_signer.rs:299-302`) and fires at **Start** — correct, but hours after the mistake, and only for one backend kind. Keep the launch-time check as a belt-and-braces invariant; move the teaching moment to the binding.

This is the requirement "without letting one chain's key be bound to the other's node", satisfied structurally: the mismatch is unrepresentable in the UI and rejected at the write.

### 6.3 What Neo X duties are actually launchable

Derive the duty picker from what the launch path can produce, and **consult `role_availability` on the launch path** — the register notes it is consulted by nothing despite `src/roles.rs:2-4` claiming otherwise, which is why an unsupported duty is accepted and silently does nothing.

| Duty | Neo X | Basis |
|---|---|---|
| RpcApi | **Launchable** | `--http` + namespaces; no key |
| Observer | **Launchable** | peering only; no key |
| State | **Launchable** — but must *do* something | Bind it to concrete settings: `eth` namespace + no state pruning. A duty that changes no setting is a label. |
| Indexer | **Launchable** — same condition | `eth_getLogs` requires retained receipts/logs; bind to the client's archive/retention flags |
| **Consensus** | **Unsupported** | Change `roles/role/availability.rs:127` from `Supported`. Neither launch path emits any validator flag anywhere in `src/`; Neo X dBFT block production needs a keystore-resident validator key inside the client plus on-chain membership, and NeoNexus provisions neither. Today the duty is offered by the matrix and then fails four different ways in `node_signer.rs:131-137, :299-302, :307-313` — an operator can configure a validator that cannot exist. New text: *"NeoNexus cannot launch a Neo X validator: block production requires the client's own keystore-resident dBFT key and on-chain membership, and NeoNexus provisions neither."* |
| Oracle / StateValidator / Notary | Unsupported | Existing reasons in `availability.rs` are correct and well-argued — **show** them in the picker rather than hiding the options; they teach the N3↔X difference at the moment it matters. |

**Direction rule:** capability arrives on the launch path first, and the availability matrix follows. Never the reverse. A test asserts that every `Supported` pair has a launch path that emits at least one duty-specific flag or config key.

**A Neo X node with a launchable duty needs no key at all.** The signer section on a Neo X node must say that ("no signer required for this duty"), not render an empty custody panel. The double-signing lease (`node_signer_bindings` unique index, `migrations.rs:112`) remains meaningful for N3 and has no subject on Neo X today; say so rather than showing a green "Lease Valid" tile (G2).

**Open question, flagged not answered:** *Archive* (full history retention — geth `--gcmode archive`, reth archive) is a genuinely distinct Neo X operational mode with real config consequences and no representation today. It is a candidate duty or a node setting. It is listed here as a decision to take, not asserted as designed — inventing a duty is the same failure mode as inventing a metric.

---

## 7. Neo N3 ↔ Neo X

### 7.1 Position: model the relationship, not the bridge

Neo X is Neo N3's EVM sidechain and there is an official bridge between them. The question is what a **node manager** should do with that.

**Not the bridge.** Bridge state is contract state on two chains: deposits, withdrawals, relayer liveness, locked balances. Rendering it requires indexing events on both chains with reorg handling — a block-explorer capability, not a node-manager one. And it is the largest available surface on which to fabricate numbers: a "Bridge Health: ● OK" tile is R1 at a new scale, and it would be the most expensive possible lie, because it is about money. **No bridge balances, no transfer tables, no bridge health widget.** If an operator runs bridge infrastructure, its nodes are nodes and get managed as nodes.

**Yes the relationship**, in four concrete ways:

1. **Chain family becomes an IA axis, not a node-type detail.** Every fleet surface is network-scoped and filterable by family and network. Today neither type nor network is a filterable axis at all (`NodeInventoryFilter` is `{status, query}`, `types/node_inventory.rs:33-45`), so an operator with both chains cannot ask "show me my Neo X fleet". One IA, two families — that is what first-class means structurally.

2. **A Networks section.** `/networks` and `/networks/{id}` for both families, with a family-appropriate identity card (N3: magic, seeds, committee, validators count; Neo X: chain id, genesis, bootnodes, block period) and a shared members/head/observed-drift panel. This is where G18, G19 and G22 all land, and it is where a Neo X operator finally does what they cannot do today: attach a genesis, set a chain spec, paste bootnodes.

3. **Parentage as a labelling and consistency fact.** `networks.parent_network_id` links `neox-mainnet → neo-n3-mainnet` and `neox-testnet → neo-n3-testnet`. Used for exactly two things: correct labelling ("Neo X TestNet — sidechain of Neo N3 TestNet"), and a **consistency warning** when one topology mixes maturity levels (a Neo X MainNet node alongside an N3 TestNet node in the same staging environment is almost always a mistake). Nothing more; it must not imply a data path that NeoNexus does not observe.

4. **One incident vocabulary.** At 03:00 the operator does not care which family a stalled node belongs to. `NodeObservation` and its derived states (`Synced`/`Syncing`/`Stalled`/`SyncStalled`/`Isolated`/`WrongChain`/`Unreachable`) are **family-neutral**; the family changes only the *evidence* rendered in one slot — peers-with-enodes vs peers-with-addresses, chain id vs network magic, genesis hash vs committee roster. Alert rules, routing scopes and the CLI JSON shape are identical across families. Design rule: any new field that exists on one family and not the other goes in the evidence enum, never in the top-level observation.

### 7.2 Terminology

The reth-based client is `NeoXReth` in Rust, `neox-rs` in serde and as a binary name, and "neox-reth" in log-parser metadata. The register (G43/G44) documents what multiple names per thing costs. Pick **neox-rs** for everything operator-facing and rename the enum variant to `NeoXRs` to match — the existing `#[serde(rename = "neox-rs")]` on `NodeType` (`src/types/node_type.rs:85`) already exists because the mismatch was a hazard once.

---

## 8. Staging

Each stage leaves the product working and shippable on its own.

**Stage 0 — Honest reporting (no schema change).** Land `argv_read`; rewrite `chain_identity_checks` to print the resolved identity triple; drop `--datadir.chain` from chain detection; delete the private block-period/validator-count `Pass`; replace the WS **Open** badge with configured/listening/reserved. Fixes G22's self-contradicting report and one G23 literal with no migration.

**Stage 1 — `networks`.** Table, seed, migration, `nodes.network_id`. Generators, validators, readiness and display switch to reading the row. `/networks` list + detail, read-only for builtin. Three-places derivation is gone.

**Stage 2 — Peering.** `Enode` type; `network_peers` / `node_peers`; forms; both clients' emission; corrected `NoDiscovery`; removal of the false bootnode critical; `GET /nodes/{id}/neox/enode` + CLI. A private Neo X network becomes wireable from the console.

**Stage 3 — RPC surface.** `node_neox` facet; namespace catalogue and picker; neox-rs `--http.api`; `--ws*` emission; authrpc and metrics ports into the planner; observed reachability replaces the badge. WS stops lying; the two clients stop diverging invisibly.

**Stage 4 — Bootstrapping.** Genesis artefacts; `POST /nodes/{id}/neox/init` + `neonexus neox init`; init state machine; the full readiness contract; INV-X1 as a launch precondition. A private Neo X node becomes safely launchable.

**Stage 5 — Observation.** `eth_syncing` / `eth_chainId` / genesis / `net_peerCount` / `txpool_status` / `eth_gasPrice` onto the shared observation layer; network-scoped thresholds; head lag from the network's own max; delete Neo X log sync-parsing; JSON log format; real fixtures.

**Stage 6 — Keys and IA.** `KeyScheme` column and bind-time rule; duty picker derived from `role_availability`; Consensus → Unsupported; family/network filters; parentage labelling; the `neox-rs` naming pass.

---

## 9. Gates this spec adds

Mechanical, not review-dependent — root cause R3 regrows without them:

1. **Flag round-trip.** Every value-taking flag the Neo X planner can emit has an `argv_read` extractor, asserted by a table test over one shared `NEOX_FLAGS` declaration.
2. **Namespace availability.** Every namespace offered for a client is in that client's supported set; `eth` and `net` are unremovable.
3. **Golden configs.** One committed golden file per `(client, network kind)` — geth/public, geth/private, neox-rs/public, neox-rs/private — asserting the exact rendered config **and** the exact argv. This is what catches "the config changed and the flags did not".
4. **Fixture provenance.** Log-parser fixtures are captured from real binaries and carry a client+version header.
5. **No literal status words in Neo X surfaces.** Lint on `● OK` / `Open` / `passed` / `Healthy` string literals in `src/web/pages/` — the general rule from the register's sequencing step 1, applied here because §4.2 is its clearest instance.
6. **Every Neo X action exists on both surfaces.** `neox init` and `neox enode` ship with route **and** CLI in the same change; a declared-capability table test enforces it.

## 10. Explicit non-goals

- NeoNexus does not generate a Neo X genesis. An invented allocation is a healthy-looking chain of one.
- NeoNexus does not compute a genesis hash offline. The file's sha256 is the artefact identity; the chain's block-0 hash comes from the running node.
- NeoNexus does not sign anything, on either chain. Neo X changes nothing about that.
- NeoNexus does not launch a Neo X validator, and the UI stops offering to.
- NeoNexus does not model the bridge.

---
