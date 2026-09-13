### Gates the model makes enforceable

1. **Render parity.** For any `(node_id, node_revision, network_revision)` with both a `launch` and a `launch-pack` render, `primary_sha256` and `sidecar_sha256` must be equal.
2. **Schema derivation.** `workspace_integrity`'s required-table list is generated from the same `&[TableDef]` that `create_tables` executes — a test asserts they are the same value, ending the two hand-maintained declarations that have already drifted.
3. **Event kind coverage.** A test asserts every `EventKind` variant has at least one non-test construction site, and every `EventKind` a page can filter by is constructible.
4. **Route/nav/form parity.** Every registered GET route has a nav entry or an inbound link; every `control_form` action string resolves to a registered route.
5. **No fabricated status.** A lint over `src/web/pages/` rejects literal `● OK`, `passed`, `Healthy`, `In Sync` and `Armed` strings that are not inside a `match` on a state value.
6. **CLI/JSON parity.** Every new entity has `neonexus <entity> list --json` and a `--json` read path, so the headless automation surface tracks the console by construction rather than by discipline.

---

## 9. Queries the model must answer

```sql
-- "Which nodes are running but not syncing?" — undetectable today.
SELECT n.id, n.name, h.label AS host, s.block_height, s.height_unchanged_secs
FROM nodes n
JOIN hosts h ON h.id = n.host_id
JOIN node_samples s ON s.id = (SELECT id FROM node_samples
                                WHERE node_id = n.id ORDER BY observed_at_unix DESC LIMIT 1)
WHERE n.status = 'running'
  AND s.outcome = 'ok'
  AND s.height_unchanged_secs >= 90;

-- "What version was node X on last Tuesday, and who put it there?"
SELECT json_extract(spec,'$.runtime_version'), json_extract(spec,'$.binary_path'),
       actor_kind, actor_id, changed_at_unix, reason
FROM node_revisions
WHERE node_id = :id AND changed_at_unix <= :tuesday
ORDER BY revision DESC LIMIT 1;

-- "When was our Oracle designation revoked?"
SELECT observed_at_unix, observed_height, designated
FROM node_designations
WHERE node_id = :id AND chain_role = 'oracle'
ORDER BY observed_at_unix DESC LIMIT 5;

-- "Which of my nodes are on a network that cannot produce a bootable config?"
SELECT n.name, w.label, w.kind
FROM nodes n JOIN networks w ON w.id = n.network_id
WHERE w.complete = 0;

-- "What is in alarm right now, and for how long?" — replaces four `● OK` literals.
SELECT r.name, r.severity, n.name AS node, a.state, a.since_unix, a.last_value, a.reason
FROM alarm_states a
JOIN alarm_rules r ON r.id = a.rule_id
JOIN nodes n       ON n.id = a.node_id
WHERE a.state = 'alarm'
ORDER BY r.severity DESC, a.since_unix ASC;

-- "Why did last night's upgrade batch report 0 of 3?"
SELECT n.name, a.from_version, a.to_version, a.stage, a.outcome, a.message
FROM runtime_upgrade_attempts a JOIN nodes n ON n.id = a.node_id
WHERE a.run_id = :run ORDER BY a.attempted_at_unix;

-- "Are two nodes contending for a port on this host?"
SELECT port, group_concat(node_id) FROM host_port_reservations
WHERE host_id = :host GROUP BY port HAVING count(*) > 1;   -- must always be empty
```

---

## 10. Staging

Each stage compiles, passes, and ships on its own.

| Stage | Migrations | What becomes true |
|---|---|---|
| **A. Foundations** | 000–003 | Hosts and grouping exist and are inert. No behaviour change; the local host and environments are seeded and unused. Ships with the Hosts page (read-only) and the `local` row. |
| **B. Network identity** | 004–005 | Chain identity is persisted, config generation takes a mandatory profile, Start refuses on an incomplete network, and every `effective_*` fallback is deleted. Unblocks G18, G19, G22, G35, G38. Landing 004 and 005 together is required: the `nodes` rebuild needs `networks` to exist for its `NOT NULL` FK. |
| **C. Observation** | 008 | `node_samples` writes latency, peers, mempool, head-lag and height-delta on every tick; `chain_state` gets its second surface; the Health chart and the CPU filter read real buckets. Prerequisite for D. |
| **D. Alarms** | 009 | The four fake alarm rows become four real, scoped, disabled-by-default rules with an `insufficient-data` state and per-route delivery. |
| **E. Custody and duties** | 006–007 | Signer backends and keys are rows with a create form; the key id is a picker; curve mismatch is a data-level refusal; duties are a set with a DB-enforced signing limit. |
| **F. History and provenance** | 010, 012, 013 | Designations, governance, config renders and event actors. The render-parity gate becomes assertable. |
| **G. Runtimes and hosts merged** | 011, 014, 015, 016 | First-run works (seeded catalog), upgrades record reasons and have a rollback target, federation is a host transport with create/edit/delete, per-node supervision override lands. |
| **H. Cleanup** | 017 + view drops | `rpc_health_checks` and `node_roles` become views and then disappear; mirrored nodes turn on once a peer exposes per-node state. |


---

### The Neo N3 Chain Surface: Design Specification
Specifies how NeoNexus should observe and report Neo N3 chain state — a per-node chain sampler feeding duty-specific node panels, a network-scoped governance page, and a designation change journal — replacing the fabricated "Monitoring" chrome the gap register documents. Every verdict is carried by one `ChainFinding` type that cannot be constructed without evidence from an actual RPC response, so a page has nothing to render green with when nothing was sampled. It also specifies the honest duty×client truth table derived by probing the real config generator and the real signer gates rather than a hand-maintained list, states plainly which validator questions public JSON-RPC cannot answer (live view number, consensus peer identity), and stages the work so each landing leaves the product working.

# The Neo N3 Chain Surface

**Scope.** What the web console shows and does with `src/chain_state/` — governance, designation, peers, mempool — plus the observation layer those reads need to become useful. Neo X appears only where the two families must share a surface; the Neo X operational surface is a separate specification.

**Status of inputs.** Written against `claudedocs/NEONEXUS_GAP_REGISTER.md` (treated as established) and a read of `src/chain_state/{model,governance,designation,peers,mempool,rpc,render}.rs`, `src/roles/role/{model,availability}.rs`, `src/core/node_signer.rs`, `src/supervision/{state,probes}.rs`, `src/rpc_health/probe*`, `src/repository/schema/tables/{inventory,observability}.rs`, `src/web/{router,nav}.rs`.

---

## 0. Three rules this surface is built on

These are design constraints, not aspirations; §14 specifies the gates that enforce them.

**Rule 1 — A verdict must carry its evidence.** `ChainFinding` cannot be constructed without at least one `Evidence`, and `Evidence` can only be minted by the sampler from an actual JSON-RPC response it received. There is no way to write `● OK` on a page: the function that renders a status word takes a `ChainFinding`, and the only way to get one is to have read something. This is the type-level answer to R1. `fn active_alarms_table() -> String` (`src/web/pages/alerts.rs:122`) is structurally impossible in this module.

**Rule 2 — Unknown is a first-class state and is never green.** Four outcomes everywhere: **Known-good**, **Known-bad**, **Unknown (not sampled)**, **Unanswerable (this client/chain does not expose it)**. The last two render grey with the reason, never a number and never a colour. A method that answers `-32601 method not found` produces *Unanswerable*, not zero and not a failure — this is the exact bug class that made a healthy Neo X node read **Unreachable** (`src/rpc_health/probe/methods.rs` doc comment).

**Rule 3 — Thresholds come from the chain, not from the binary.** Every comparison reads its bound from what the network declares: block interval from `getversion.protocol.msperblock`, mempool capacity from `memorypoolmaxtransactions`, validator count from `validatorscount`, committee size from `getcommittee().len()`. Compile-time constants are replaced: `classify_connectivity`'s `1..=2 => Sparse` (`peers.rs:171-177`) and `classify_congestion`'s `500`/`2000` (`mempool.rs:174-182`) become functions of the network's own parameters plus an operator override. A 4-node private network stops reading "Healthy" at 3 peers; 500 tx stops reading "Elevated" on a chain configured for 5,000-tx blocks (G12).

---

## 1. The read inventory

Every method below must be exercised against a real client and a response fixture checked in under `tests/fixtures/chain/<client>-<version>/<method>.json` **before the panel that consumes it ships**. Fixtures hand-written to match a parser are what made CI blind to G11 (`tests/unit/supervision/tests.rs:167-206`); the gate in §14.3 rejects a fixture with no recorded provenance header.

### 1.1 Neo N3 — every node, every duty

| Method | Gives | Confidence | Notes |
|---|---|---|---|
| `getversion` | `protocol.network`, `msperblock`, `validatorscount`, `memorypoolmaxtransactions`, `maxtransactionsperblock`, `maxtraceableblocks`, `hardforks`; `rpc.sessionenabled`, `maxiteratorresultitems`; useragent | High | The single most valuable call in the product and currently used only for a version string (`probe.rs:49`). `protocol.network` is the **running node's real magic** — comparing it to the configured magic detects the G18 catastrophe (config says private 1230000, node dialled mainnet seeds) from the outside. |
| `getblockcount` | height | Certain | already used |
| `getblockheadercount` | header height | High | `headers − blocks` is the node's own statement that it knows it is behind. The N3 analogue of `eth_syncing`; named as missing in G11. |
| `getbestblockhash` | head hash | High | Fork detection across fleet nodes on one network. |
| `getblock(height, 1)` | header incl. `primary`, `time`, `nextconsensus`, tx count | Medium-high — **fixture required before §6.4 ships** | Block timestamps give a real block-interval series, independent of the workbench clock. |
| `getpeers` | `connected[]`, `unconnected[]`, `bad[]` | Certain | already parsed (`peers.rs:95-130`) |
| `getrawmempool(true)` | verified/unverified | Certain | already parsed (`mempool.rs:82-114`) |
| `getcommittee` | 21 keys | Certain | already parsed |
| `getnextblockvalidators` | next-round validators | Certain | already parsed |
| `getcandidates` | `{publickey, votes}` | Certain | already parsed |
| `invokefunction` RoleManagement `getDesignatedByRole` | designated keys at a height | Certain | already implemented (`designation.rs:27-55`) |

### 1.2 Neo N3 — duty-specific

| Method | Duty | Confidence |
|---|---|---|
| `getstateheight` → `{localrootindex, validatedrootindex}` | State, StateValidator | Medium-high — fixture required |
| `getstateroot(index)` → `{index, roothash, witnesses}` | StateValidator | Medium — fixture required |
| `listplugins` | RpcApi, Indexer, State (neo-cli only) | Medium-high — neo-go has no plugins; renders *Unanswerable* there |
| `getapplicationlog`, `getnep17transfers` (capability probe only) | Indexer | High |
| `getrawnotarypool` | Notary (neo-go only) | Medium — **verify before shipping**; if absent, the Notary panel shows designation only |
| `invokefunction` NeoToken `getAccountState`, `unclaimedGas`, `getRegisterPrice` | Consensus, account panel | Medium-high — fixture required |
| `invokefunction` NEO/GAS `balanceOf` | account panel | High — native hashes are protocol-fixed, same pattern as `ROLE_MANAGEMENT_HASH` |
| `invokefunction` OracleContract `getPrice` | Oracle | Medium |

`getnep17balances` is **not** used for the account panel: it requires the TokensTracker plugin on neo-cli and answers `-32601` without it. `balanceOf` on the native contracts works on any RPC node.

### 1.3 Neo X

`eth_chainId` (real chain id vs configured — the G22 detector), `eth_syncing` (object form gives the node's own target height), `eth_blockNumber`, `eth_getBlockByNumber("latest")` (timestamp for block interval), `net_peerCount`, `txpool_status`. All high confidence. **No designation surface exists on Neo X** — the Neo X duty panel says so in one sentence rather than rendering an empty or green one.

---

## 2. Core types

```rust
// src/chain_state/finding.rs
pub struct Evidence {
    pub method: &'static str,   // "getversion"
    pub field: &'static str,    // "protocol.network"
    pub value: String,          // as read, not as interpreted
    pub endpoint: String,
    pub sampled_at_unix: u64,
}

pub struct ChainFinding {
    pub code: ChainFindingCode,      // stable kebab-case; the alert-rule key
    pub severity: FindingSeverity,   // Info | Notice | Warning | Critical
    pub subject: FindingSubject,     // Node(id) | Network(network) | Duty(node_id, NodeRole)
    pub statement: String,           // one sentence containing the numbers
    pub evidence: Vec<Evidence>,     // non-empty by construction
    pub next_step: Option<NextStep>, // { text, Href | Command | Contact }
}

impl ChainFinding {
    /// The only constructor. A finding with no evidence cannot exist.
    pub fn new(code: ChainFindingCode, severity: FindingSeverity,
               subject: FindingSubject, statement: String,
               evidence: Vec<Evidence>) -> Option<Self> { /* None when evidence is empty */ }
}
```

```rust
pub enum Observation<T> { Known(T, Evidence), Unknown(NotSampled), Unanswerable(&'static str) }
```

`Observation<T>` is what every panel field holds. `Unanswerable` carries the reason as a static string authored next to the probe, e.g. `"neo-go exposes no plugin list"`. The HTML helper for `Observation` has three renderings and none of them is a badge class that can read green for `Unknown`.

**One type, three surfaces.** `Vec<ChainFinding>` is what the node page renders, what `--chain-node <id> --json` serialises, and what the event journal receives. A test (§14.2) asserts the HTML page and the CLI JSON are produced from the same vector, which is the structural fix for R3 in this module.

### 2.1 The duty readiness ladder

Six levels, evaluated per node. Each panel in §5 states which level the node has reached and what is blocking the next. This frame exists because G20's failure — duty accepted, provisioned, never performed — is invisible unless configuration truth and chain truth are shown on one axis.

1. **Assigned** — a duty row exists in `node_roles`.
2. **Producible** — the launch path can emit a configuration for this (client, duty). §9.
3. **Enabled** — the rendered configuration actually switches the service on. This is where neo-cli StateValidator (`"AutoVerify": false`, `generator/neo_cli/sidecar.rs:192`) and Oracle (`"AutoStart": false`, `"Nodes": []`, `:170,:178`) currently stop.
4. **Keyed** — a signer binding or wallet profile yields a usable public key, and the signer gate for this (client, duty, backend) accepts it.
5. **Designated / Elected** — chain says this key holds the role (RoleManagement) or is in the committee/validator set (Consensus).
6. **Performing** — observed evidence the duty is being done.

Level 6 is honestly reachable only for Consensus (partially, §6.4) and, at fleet level, StateValidator (`validatedrootindex` advancing). For Oracle and Notary the panel states: *"whether this node answered a request is not observable over JSON-RPC; NeoNexus reports levels 1–5."* It does not substitute a green tile.

---

## 3. Storage

New file `src/repository/schema/tables/chain.rs`, added to the `workspace_integrity` required-table list in the same commit (G41 is a live trap for any new table).

```sql
CREATE TABLE IF NOT EXISTS chain_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sampled_at_unix INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    family TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    outcome TEXT NOT NULL,                 -- ok | unreachable | unexpected
    detail TEXT NOT NULL DEFAULT '',
    round_trip_ms INTEGER,                 -- first successful call; the only latency in the product
    height INTEGER,
    header_height INTEGER,
    best_block_hash TEXT,
    best_block_time_unix INTEGER,          -- from the header, not our clock
    reported_network INTEGER,              -- getversion.protocol.network | eth_chainId
    ms_per_block INTEGER,
    validators_count INTEGER,
    peers_connected INTEGER,
    peers_unconnected INTEGER,
    peers_bad INTEGER,
    mempool_verified INTEGER,
    mempool_unverified INTEGER,
    mempool_capacity INTEGER,
    syncing INTEGER,                       -- Neo X only: 0 | 1 | NULL
    sync_target_height INTEGER,
    state_local_root_index INTEGER,
    state_validated_root_index INTEGER,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_chain_samples_node_time
    ON chain_samples (node_id, sampled_at_unix DESC);
```

Every column except `outcome` is nullable: *not measured* must be representable, or the schema forces a lie.

```sql
-- Edge-trigger memory for events. Scalars only; findings are derived on read.
CREATE TABLE IF NOT EXISTS chain_node_state (
    node_id TEXT PRIMARY KEY,
    updated_at_unix INTEGER NOT NULL,
    liveness TEXT NOT NULL,                -- unknown|synced|lagging|stalled|unreachable
    head_lag INTEGER,
    seconds_since_height_change INTEGER,
    last_height_change_unix INTEGER,
    connectivity TEXT NOT NULL,            -- unknown|isolated|sparse|healthy
    congestion TEXT NOT NULL,              -- unknown|normal|elevated|congested
    network_matches INTEGER,               -- 0|1|NULL
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- Append only when a digest changes; otherwise bump last_seen_at_unix in place.
-- The table is therefore a change log by construction.
CREATE TABLE IF NOT EXISTS chain_governance_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_seen_at_unix INTEGER NOT NULL,
    last_seen_at_unix INTEGER NOT NULL,
    network TEXT NOT NULL,
    source_node_id TEXT,
    height INTEGER NOT NULL,
    committee TEXT NOT NULL,               -- JSON array, in returned order
    next_validators TEXT NOT NULL,
    committee_digest TEXT NOT NULL,
    validators_digest TEXT NOT NULL
);

-- Only the rows a fleet cares about: its own keys, plus the rank boundaries
-- that decide committee (21/22) and validator (7/8) membership.
CREATE TABLE IF NOT EXISTS chain_candidate_samples (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    sampled_at_unix INTEGER NOT NULL,
    network TEXT NOT NULL,
    public_key TEXT NOT NULL,
    rank INTEGER NOT NULL,
    votes INTEGER NOT NULL,
    candidate_count INTEGER NOT NULL,
    role TEXT NOT NULL                     -- fleet-key | boundary-validator | boundary-committee
);

-- Also append-on-change. "When was my Oracle designation revoked" becomes a
-- lookup, not a scan.
CREATE TABLE IF NOT EXISTS chain_designations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_seen_at_unix INTEGER NOT NULL,
    last_seen_at_unix INTEGER NOT NULL,
    node_id TEXT NOT NULL,
    chain_role TEXT NOT NULL,
    height INTEGER NOT NULL,
    compared_key TEXT,                     -- NULL = we had no key to compare
    key_source TEXT NOT NULL,              -- signer-binding | wallet-profile | none
    includes_node_key INTEGER,             -- 0 | 1 | NULL, mirrors Option<bool>
    designated_keys TEXT NOT NULL,
    designated_digest TEXT NOT NULL,
    outcome TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT '',
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

-- Populated only for networks where the fleet holds a Consensus duty.
CREATE TABLE IF NOT EXISTS chain_blocks (
    network TEXT NOT NULL,
    height INTEGER NOT NULL,
    block_time_unix INTEGER NOT NULL,
    primary_index INTEGER NOT NULL,
    tx_count INTEGER NOT NULL,
    next_consensus TEXT NOT NULL,
    validators_digest TEXT NOT NULL,       -- joins to the governance row in force
    PRIMARY KEY (network, height)
);
```

**Retention**, all settings-backed with defaults: `chain_samples` by age, 24 h; `chain_blocks` by row count, 20 000 (≈3.5 days at 15 s); `chain_governance_samples` and `chain_designations` by age, 365 days (they are change logs and are tiny); `chain_candidate_samples` by age, 90 days. Pruning runs on the supervision tick alongside `prune_rpc_health_keep_recent_per_node`.

**`rpc_health_checks` is not extended.** It stays what it is — method reachability — and `chain_samples` supersedes it for anything an operator reads. `RpcHealthRecord` keeps no latency field; `round_trip_ms` lives on the chain sample where it has a defined meaning.

---

## 4. Sampling

### 4.1 Per-node sample — on the existing RPC-health tick

Extend `LoopState::probe_rpc_health` (`src/supervision/probes.rs:33`) rather than adding a second scheduler. It already: reads an operator-configurable interval (`RpcHealthMonitorPolicy`, default 30 s, min 10 s), selects **one due node per tick**, and edge-triggers journal entries. All three behaviours are correct and are kept.

Order of calls per sample, with the first success timed for `round_trip_ms`:

1. `getversion` — version, protocol parameters, network magic. Failure here ⇒ `outcome = unreachable`, stop.
2. `getblockcount`, `getblockheadercount`, `getbestblockhash`.
3. `getpeers` (fallback `getconnectioncount`, already implemented).
4. `getrawmempool(true)`.
5. Duty-conditional: `getstateheight` for State/StateValidator; `getrawnotarypool` for neo-go Notary.

Budget: 3 s per call (matching `CHAIN_TIMEOUT` and `RPC_HEALTH_TIMEOUT`), 8 s for the whole sample; a partial sample is stored with the fields it got and `outcome = unexpected` plus the detail. **A partial sample is normal and must render as a mix of Known and Unknown**, never as a failure of the node.

Neo X substitutes §1.3's method list; `eth_syncing`'s object form supplies `sync_target_height` directly and takes precedence over the fleet reference for that node.

### 4.2 Network reference head

```
reference(network) = max(height) over fleet nodes on `network` sampled within 2×interval
                     ∪ eth_syncing.highestBlock where reported
                     ∪ operator-configured reference endpoint (optional, per network)
head_lag(node) = reference − node.height
```

A network with one node has `head_lag = 0` by definition. The page says so: *"the reference is this node itself — add a second node on this network or a reference endpoint to detect a fleet-wide stall."* Stall detection is deliberately independent of the reference:

```
stalled  ⟺  now − last_height_change_unix  >  stall_multiple × ms_per_block
```

`stall_multiple` defaults to 6 (90 s on a 15 s chain), operator-settable per network, with a 60 s floor. `ms_per_block` comes from `getversion.protocol`; where it is unavailable (Neo X) it is estimated from the median interval of the last 20 observed block timestamps and labelled *estimated*. This is the check that catches the fleet-wide stall the fleet-max reference cannot see, and it is the direct answer to "running but not syncing" (G11).

### 4.3 Governance sample — per network, not per node

Governance is a property of the network. Asking twelve mainnet nodes for `getcommittee` produces twelve near-identical answers and twelve times the load. One sample per network per cadence, taken through a **reference node**: the healthiest node on that network by last sample, preferring one with the RpcApi duty, with deterministic fallback down the list and the chosen node recorded in `source_node_id` so the page can say where the answer came from.

**Cadence is derived**, per Rule 3: `committee_size × ms_per_block` — 21 × 15 s = 315 s on mainnet, which is one committee-refresh epoch. Clamped to [60 s, 3600 s], operator-overridable per network. Plus:

- a **Refresh now** control (POST, redirect — no JS);
- a forced re-sample whenever a fleet key's committee or validator membership changes, since designation changes are committee acts and often land in the same window.

### 4.4 Designation sample

Two calls (`getblockcount`, one `invokefunction`) per (node, chain role). Only sampled for nodes whose duty has `NodeRole::designation() == Some(_)` — `src/roles/role/model.rs:94-103`, currently a function with zero non-test callers (G13). Sampled on the **governance** cadence, not the 30 s one, plus immediately on any committee change, plus on demand. A per-node override allows faster polling for a node an operator is actively watching.

### 4.5 Block walk — only where it earns its cost

`chain_blocks` is populated only for networks where the fleet holds a Consensus duty. Each tick fetches `getblock(h, 1)` for heights new since the last sample, capped at 20 per tick. At a 30 s interval on a 15 s chain that is two calls. On first enable it backfills at most 200 blocks and says how far back the data goes.

---

## 5. The per-node chain panel

**Placement.** `/nodes/{id}`, a new **Chain** tab placed second, after Details. The **Monitoring** tab (`src/web/pages/nodes/detail_tabs.rs:407-540`) is deleted in the same change — its CPU, memory, latency, sparkline, watchdog badge and four unconditional `● OK` cards are G2, and real per-process CPU/RSS already render honestly on `/monitor`.

### 5.1 Common header — every node, every duty, both families

Rendered from the latest `chain_samples` row plus `chain_node_state`.

| Field | Source | Failure rendering |
|---|---|---|
| Endpoint, family, sampled at | node + sample | — |
| Round-trip | `round_trip_ms` | *Unknown (not sampled)* |
| Client version | `getversion.useragent` / `web3_clientVersion` | *Unknown* |
| **Network / chain id: reported vs configured** | `protocol.network` / `eth_chainId` vs node config | **mismatch ⇒ Critical finding `chain-network-mismatch`** |
| Height | `getblockcount` | *Unknown* |
| Head lag | reference − height | *"reference is this node itself"* when alone |
| Headers ahead of blocks | `getblockheadercount − getblockcount` | *Unanswerable* on Neo X (use `eth_syncing`) |
| Time since height last changed | `chain_node_state` | *Unknown* until two samples exist |
| Liveness verdict | §4.2 | **Unknown** until two samples exist — explicitly, not Synced |
| Peers | `getpeers` | expectation derived per Rule 3 |
| Mempool | `getrawmempool(true)` | capacity from `memorypoolmaxtransactions` |

The network-mismatch row is the most valuable thing on the page and costs one field of one call. It catches the reproduced G18 failure — a node whose config carries magic 1 230 000 while the binary falls back to compiled-in public defaults and dials mainnet seeds — from outside the config generator entirely.

**Peer expectation** replaces `classify_connectivity`'s constants:

- `Network::Private` ⇒ expected = (other fleet nodes on this network) − 1; **isolated ⇔ 0**, **sparse ⇔ below expected**.
- Public networks ⇒ operator floor per network, default 3, with the statement naming it as a policy not a fact.
- Unknown until the first `getpeers` answers.

### 5.2 Duty-specific sections

#### RpcApi — *"can my consumers use this node?"*

- **Method surface**: a capability probe run at bind time and on version change (not per tick) over the methods dApps need — `getblock`, `getcontractstate`, `invokefunction`, `getapplicationlog`, `getnep17transfers`, `getstateroot`. Each renders Available / `-32601` not exposed / Unknown.
- **Plugins loaded vs configured**: `listplugins` on neo-cli against the plugin set NeoNexus wrote. Drift is a Warning finding with a link to `/plugins?node={id}`. *Unanswerable* on neo-go.
- **RPC settings that break clients**: `protocol.rpc.sessionenabled`, `maxiteratorresultitems` — read from the running node, not from the config we wrote.
- **Round-trip** over the retained window: p50 and max, labelled *"loopback probe from this workbench"* so it is never read as a user-facing SLO.
- Mempool depth (a public endpoint's queue is its consumers' queue).
- **No designation section.** RpcApi is a local capability; the page says nothing about RoleManagement rather than showing an empty panel.

#### State — *"are my proofs answerable and current?"*

- Three numbers with three meanings, side by side: `getblockcount` (chain height), `getstateheight.localrootindex` (how far my MPT has been computed), `getstateheight.validatedrootindex` (how far the network's designated validators have witnessed roots). Each labelled; the gap between the first two is *this node's* lag, the gap between the last two is *the network's*.
- `FullState` from the rendered config: whether historical proofs are answerable at all.
- *Unanswerable* when `getstateheight` returns `-32601` — which is itself the finding: the StateService is not loaded.
- No designation section: **State is not StateValidator.** The page says this in one line, because the two names are one letter apart in the duty picker.

#### Indexer

- Capability probe: `getapplicationlog`, `getnep17transfers`. `-32601` on either ⇒ Critical finding `indexer-not-serving` — the duty is assigned and the index is not being served.
- Plugin drift as for RpcApi.
- The index's own height is **not exposed** by any client; stated, not invented.

#### Consensus — §6.

#### Oracle

- **Designation** (§7) for `ChainRole::Oracle`.
- **Local enablement truth**, side by side with it: for neo-cli, `"AutoStart": false` and `"Nodes": []` are read out of the *rendered config*, with the statement *"this node's OracleService is configured not to start; designation alone will not make it answer requests"* (G20). For neo-go, `Oracle.Enabled` derived from the wallet (`generator/neo_go/services.rs:35,:45`).
- **Oracle peer list** — the `Nodes` array must become a real, persisted, editable field (today there is no field anywhere to fill it in and every Start rewrites the sidecar, `src/config/export/node.rs:187-208`). The panel shows it and flags empty.
- `OracleContract.getPrice()` — the GAS per response, read from chain.
- **Not observable, stated plainly**: pending request depth, and this node's response count. No standard JSON-RPC method exposes either.

#### StateValidator

- **Designation** for `ChainRole::StateValidator`.
- `validatedrootindex` vs chain height, with its trend: roots advancing ⇒ the designated set is working; a widening gap ⇒ the state-root quorum is broken, which is a **network** finding, surfaced here and on the network page.
- **Local enablement truth**: neo-cli's hardcoded `"AutoVerify": false` renders as *"provisioned but disabled — this node computes state roots and publishes no witness"*, level 3 of the ladder, blocking. neo-go derives the same switch from the wallet, so the panel shows a genuine difference between two clients holding the same duty.
- **Not observable**: whether *this key's* witness is in any given root. Attribution would require verifying each signature in the witness against each designated key; NeoNexus does not do this and says so.

#### Notary

- **Designation** for `ChainRole::P2PNotary`.
- neo-go only: notary pool depth, **if** `getrawnotarypool` verifies against a real neo-go build; otherwise the section is designation-only.
- neo-cli and neo-rs never reach this panel — the duty is not offered (§9).

#### Observer

- Common header only, plus one sentence: *"this node performs no chain duty; no designation or committee record refers to it."* No tiles, no badges. This is the correct rendering of a duty that is genuinely read-only — and distinct from a node with **no duty at all**, which renders *"No duty assigned"* and never the word Observer (G9).

#### Neo X, any duty

Common header from §1.3, `eth_syncing` in place of the header/block gap, txpool in place of mempool, and one sentence where the designation section would be: *"Oracle, StateValidator and P2PNotary are designations on the Neo N3 RoleManagement native contract. Neo X is an EVM sidechain and has no such contract."* Consensus on Neo X is not offered (§9).

### 5.3 What the panel does, not just shows

Every finding at Warning or above carries a `next_step`. The permitted forms are: a link to a page in this console that can actually change the thing; a copyable CLI command; or a named human ("the committee"). A finding whose only remedy is outside the product says so — that is the honest case for designation, and it is better than a button that cannot be honoured.

---

## 6. The consensus / validator surface

### 6.1 What public JSON-RPC can and cannot answer

| Operator question | Answerable? | How |
|---|---|---|
| Am I in the committee? | **Yes** | `getcommittee` contains my key |
| Am I in the top 7 — producing next round? | **Yes** | `getnextblockvalidators` contains my key |
| How many votes do I have? | **Yes** | `getcandidates` → my key's `votes` |
| Is that trending down? | **Only from our own samples** | No RPC returns historical vote totals. The chart shows what NeoNexus sampled and states its start date. Before that date: Unknown. |
| How close am I to falling out? | **Yes, derived** | sorted candidates: my votes − votes at rank 8 (validator boundary) and rank 22 (committee boundary), as an absolute and a percentage |
| Did the committee or validator set just change? | **Yes** | digest diff of successive samples; the diff names who entered and who left |
| Am I producing the blocks assigned to me? | **Partially, by derivation** | §6.4 — with three stated caveats |
| Are we in a view change **right now**? | **No** | §6.5 |
| What is my consensus peer set? | **No** | §6.6 |
| Is the chain producing blocks at all? | **Yes** | height progress + block timestamps vs `msperblock` |
| Was I slashed or penalised? | **N/A** | Neo N3 dBFT has no slashing. The page does not offer a slashing surface, because inventing one would teach a wrong model of the protocol. |

### 6.2 Standing panel

Rendered from the latest governance sample for the node's network, with the node's own key highlighted:

```
Committee        member — rank 6 of 21          (getcommittee, 2 min ago)
Next validators  producing — 1 of 7             (getnextblockvalidators)
Votes            41,203,118 NEO                 (getcandidates)
Margin           +2.1M NEO over rank 8          → you keep producing
                 +18.4M NEO over rank 22        → you keep your committee seat
Trend            −310k NEO over 14 days         (from 4,032 samples since 2026-08-30)
```

The margin lines are the operator's actual question — "am I about to be voted out" — expressed as the distance to the two boundaries that matter. The trend line always names its sample count and start date; with fewer than two samples it reads *"not enough history"*.

### 6.3 Block production panel

From `chain_blocks`:

- **Block interval**: median, p95 and max over the retained window, against `msperblock`. Computed from block timestamps, not from the workbench clock — so a slow workbench cannot manufacture a chain problem.
- **Throughput**: transactions per block over the window, against `maxtransactionsperblock`.
- **Primary distribution**: how often each validator index was the primary over the window. A healthy set is roughly uniform; a validator that never appears as primary is the shape of a real outage.

### 6.4 "Am I producing my assigned blocks?" — the derivation and its caveats

In dBFT 2.0 the primary for height *h* at view *v* is index `(h − v) mod n` over the ordered validator list. The block header carries the primary index (`primary` in verbose `getblock`). Therefore:

```
derived_view(h) = (h − block.primary) mod n
missed_at_view_0(h, me) ⟺ (h mod n) == my_index  ∧  block.primary != my_index
```

This yields a real "blocks assigned vs blocks proposed" counter and a real per-height view number — the only view information obtainable from the outside.

**Three caveats, all of which must be on the page, not only in this document:**

1. **Validator set epochs.** The validator list is recomputed periodically, so an index only means something relative to the set in force at that height. `chain_blocks.validators_digest` records which set each block is attributed under; blocks straddling a set change are excluded from the counter and counted as *unattributed*. The panel shows the unattributed count.
2. **Coverage.** Attribution exists only for heights NeoNexus sampled. The panel states the height range and the number of gaps.
3. **Confidence.** This derivation is the one place in the surface that *accuses* an operator's node of failing. It ships behind a fixture captured from a real validator on a network that has actually experienced a view change, and `ChainValidatorMissedProposal` is emitted only when the block's validator set matches the set in force and there is no gap in the preceding 10 heights. If either condition fails, the counter renders and the event does not.

### 6.5 View change — stated as unanswerable

No Neo N3 JSON-RPC method exposes the local dBFT state machine's current view, its `PrepareRequest`/`PrepareResponse`/`ChangeView` traffic, or its timer. The panel says exactly that, and then shows the two things that *are* observable:

- **derived view of recent blocks** (§6.4), labelled *inference from block headers*;
- **current block-interval gap**: seconds since the last block against `msperblock`. A gap of several multiples with no new block is what a view change looks like from outside, and is reported as *"block production has not advanced for 71 s on a 15 s chain; this is consistent with a view change in progress, but the current view is not observable"*.

The node's own log contains consensus messages and is a legitimate **future** source. It is named as a future source on the page, not silently synthesised into a number — precisely the mistake the log-derived `SyncProgress` parsers made when they gated on strings the clients never emit (G11).

### 6.6 Consensus peer set — stated as unanswerable

`getpeers` returns addresses and ports. It does not return public keys, so there is no way to learn from RPC whether your eight peers include the other six validators. The panel therefore reports:

- connected peer addresses (already parsed, `peers.rs:104-118`);
- **cross-reference against this fleet**: peers whose address:port matches another managed node are named, which on a private network answers the question completely;
- **cross-reference against operator-declared validator endpoints**, a new optional per-network list;
- and, for everything else: *"peer identity is not exposed by getpeers; NeoNexus cannot confirm the validator mesh."*

### 6.7 What the consensus panel does *not* do

It offers no vote, no candidate registration, no designation, no key export. `src/chain_state.rs:8-12` states the policy and it holds: NeoNexus reads. Where an action is needed, the panel names the transaction (`NeoToken.registerCandidate`, the committee's `RoleManagement.designateAsRole`), its cost where readable from chain (`getRegisterPrice`), and who can perform it.

---

## 7. The designation surface

This is where the register's sharpest sentence lives: *NeoNexus cannot say "your Oracle designation was revoked at 03:14" — the case where a node keeps running and stops doing its job* (G13).

### 7.1 Resolving the node's key

Priority order, with the source always shown:

1. `node_signer_bindings` → `SignerRegistry::key_info()` → `KeyPublic.public_key` (`src/core/node_signer.rs:537`).
2. `node_wallets` → `NeoWalletProfile.contract_public_keys` (`src/wallet/model.rs:32`). When the profile carries more than one key, all are compared and the matching one is named.
3. None ⇒ `includes_node_key = NULL`.

Case 3 is **not** "not designated". It maps to `RoleDesignation.includes_node_key: None`, which `summary()` already renders correctly (`model.rs:56-60`), and it emits `ChainDesignationUnknown` at Warning: the node has a duty that requires designation and NeoNexus has no key to check. Today the CLI collapses this into exit 1, identical to "not designated" (`cli/actions/chain.rs:23-26`); that collapse is removed — an unknown key exits 2.

### 7.2 What the panel shows

```
Oracle designation                                    checked at height 6,421,880, 3 min ago
  This node's key      03a1…7f2c   (signer binding kms-local / oracle-key-1)
  Designated           yes — 1 of 7 designated keys
  Unchanged since      2026-07-12 09:14 UTC   (1,042 checks)
  Other holders        02b4…, 03c9…, …        [show all]
```

Four states, four renderings, none of them shared: **designated**, **not designated**, **key unknown**, **could not read the chain**. `ChainQueryError`'s existing split between `Unreachable` and `Unexpected` (`model.rs:10-15`) is preserved into the UI: a node that cannot be reached and a node that answered something we did not understand are different problems.

### 7.3 Detecting change — revocation is the dangerous one

`chain_designations` is append-on-change, so every transition is a row. Transitions and their events:

| Transition | Event | Severity |
|---|---|---|
| not designated → designated | `ChainDesignationGranted` | Notice |
| **designated → not designated** | **`ChainDesignationRevoked`** | **Critical** |
| designated set changed, our membership unchanged | `ChainDesignationSetChanged` | Info |
| key resolvable → not resolvable | `ChainDesignationUnknown` | Warning |
| read failed for N consecutive samples | `ChainDesignationUnreadable` | Warning |

The revocation event's message is the whole point and is specified verbatim in shape:

> *"{node} is no longer designated for Oracle as of height 6,421,905 (observed 03:14:22 UTC). The process is still running and will stop answering oracle requests. Designation is a committee transaction; NeoNexus cannot restore it."*

Next steps offered: stop the node; reassign its duty (link to `/nodes/{id}/edit`); view the current holders; raise with the committee. No button claims to re-designate.

### 7.4 The panel that tells the truth twice

Designation is necessary and not sufficient. The panel pairs each chain answer with the corresponding local answer from the ladder (§2.1), so an operator cannot read a green designation and conclude the duty is being performed:

```
Level 3  Enabled       ✗  OracleService AutoStart = false, Nodes = []   → this node will not start the service
Level 5  Designated    ✓  designated since 2026-07-12
```

Without this pairing the console would report a correctly designated neo-cli oracle that has never answered a single request — which is exactly the state G20 describes.

### 7.5 NeoFSAlphabet

`ChainRole::NeoFSAlphabet` is designatable and no `NodeRole` maps to it (`role/model.rs:94-103`). It appears on the **network** page's designation table — "who holds each role on this network" — and never on a node page, because no node in this fleet can hold it. It is not added to the duty picker.

---

## 8. The governance page

### 8.1 Routes

| Route | Contents |
|---|---|
| `GET /chain` | One card per distinct `(network, family)` in the fleet: head, block interval, node count, and the fleet's standing in one line. Nav entry under **Network**, replacing the mislabelled "Private network" entry that opens the duty matrix (G45). |
| `GET /chain/{network}` | The network page, below. |
| `GET /chain/{network}?candidates=all` | Full candidate tail, paginated. |
| `POST /chain/{network}/refresh` | Force a governance sample; redirect back with a flash. |
| `GET /nodes/{id}?tab=chain` | The node panel (§5), reachable from every chain table row. |

Every one of these gets a nav entry or an inbound link, per the parity gate the register asks for (sequencing item 4).

### 8.2 The network page

**Header.** Network name, family, magic *as reported by the reference node* with the source named, reference head, block interval against `msperblock`, sample age, and the reference node with a link.

**Committee (21).** Rank, public key, validator flag, votes, and a **This fleet** column marking rows whose key belongs to a managed node, linked to it. Rows for fleet keys are visually distinguished. Below: *"last changed 2026-09-11 04:20 — 1 in, 1 out"* with the diff expandable.

**Next-block validators (7).** The same table filtered, kept separate because "committee member" and "produces blocks" are different facts that the same 21-row table conflates. Fleet membership marked.

**Candidates.** Top 25 by votes with the two boundaries (rank 7/8, rank 21/22) marked by a horizontal rule, fleet keys pinned into view wherever they rank, and the count of omitted candidates. This is `render.rs`'s `CANDIDATE_ROWS` idea with the fleet's own keys always visible.

**Designations.** One row per `ChainRole` — StateValidator, Oracle, NeoFSAlphabet, P2PNotary — with holder count, the fleet's keys among them, and last-change time. This is the fleet-wide view of §7 and the only place NeoFSAlphabet appears.

**Fleet on this network.** Every managed node: height, head lag, peers, liveness, duty, and its chain standing where it has one.

**State roots.** `validatedrootindex` against head, from whichever node answers `getstateheight` — a network-level health signal for the StateValidator set.

**Neo X networks** render the header, the fleet table, and one sentence explaining that committee, candidates and designations are Neo N3 native-contract concepts. They do not render empty tables.

### 8.3 Refresh cadence, stated on the page

The page always shows *"sampled 2 min ago · every 5 min 15 s (21 blocks × 15 s) · Refresh now"*. The cadence is derived (§4.3), the derivation is visible, and the override is in Settings. A sample older than 3× the cadence renders the whole page's values in the Unknown style with a banner naming the reason (reference node unreachable, monitor disabled, workbench restarted). **Stale never renders as current.**

---

## 9. The duty → capability truth table

### 9.1 The honest matrix today

Derived by composing the four gates in §9.2 against the code as it stands. **Bold** marks cells where the derived truth differs from what `role_availability` (`src/roles/role/availability.rs`) currently declares and the picker currently offers.

| Duty | neo-cli | neo-go | neo-rs | neox-geth | neox-reth |
|---|---|---|---|---|---|
| RpcApi | Full | Full | Full | Full | Full |
| State | Full | Full | Full | Full | Full |
| Indexer | Full | Full | Full | Full | Full |
| Observer | Full | Full | Full | Full | Full |
| Consensus | **Caveat — runtime layout** | Full | **Not supported** | **Not supported** | **Not supported** |
| Oracle | **Caveat — provisioned, disabled** | Full | Not supported | Not supported | Not supported |
| StateValidator | **Caveat — provisioned, disabled** | Full | Not supported | Not supported | Not supported |
| Notary | Not supported | Full | Not supported | Not supported | Not supported |

Reasons, each traceable to code:

- **neo-cli Consensus / Oracle / StateValidator — runtime layout.** `ensure_neo_cli_runtime_root` (`src/core/node_signer.rs:405-424`) requires the binary's parent directory to equal the node working directory. Managed installs land under `<workspace>/runtimes/…` and copy only the executable; plugins install under `<workspace>/nodes/<id>/Plugins`. No chip the editor offers can satisfy this (G24). These cells are **Caveat**, not Full, and the caveat is evaluated per node so a hand-placed runtime tree resolves it.
- **neo-cli Oracle — provisioned, disabled.** `"AutoStart": false`, `"Nodes": []` (`generator/neo_cli/sidecar.rs:170,:178`), with `stdin(Stdio::null())` (`supervisor/process/spawn.rs:32`) denying the `start oracle` console command that would compensate.
- **neo-cli StateValidator — provisioned, disabled.** `"AutoVerify": false` hardcoded in a zero-argument function (`sidecar.rs:192`), pinned by a test. neo-go flips the same switch from the wallet.
- **neo-rs Consensus — not supported.** `ensure_local_wallet_runtime` rejects neo-rs outright (`node_signer.rs:296-298`: "no safe NEP-6 wallet integration"), and `ensure_sign_client_runtime` accepts only neo-cli Consensus (`:307-313`). There is no path.
- **neox Consensus — not supported.** Same two rejections (`:299-302`, `:307-313`), plus no `--validator`/`--miner` flag anywhere (G21).

### 9.2 Deriving the matrix instead of declaring it

`role_availability` is kept — its four doc-commented claims are genuine facts about the client software, researched and cited, and belong in code. What changes is that it becomes **one of four inputs**, not the answer:

```rust
pub enum DutySupport {
    Full,
    Caveat { level: LadderLevel, reason: String, remedy: Option<NextStep> },
    Unsupported { reason: String },
    Unverified { reason: String },
}

pub fn duty_support(node_type: NodeType, role: NodeRole) -> DutySupport   // matrix cell
pub fn duty_support_for_node(node: &NodeConfig, role: NodeRole, ctx: &NodeContext) -> DutySupport
```

Four gates, composed worst-first:

1. **`role_availability(node_type, role)`** — protocol facts about the client. Unchanged.
2. **`config_generator_emits(node_type, role)`** — **probed, not declared.** Renders the real config for a synthetic node with that (type, role) through the real `ConfigGenerator` and inspects the duty's enable switch (`AutoStart`, `AutoVerify`, `Enabled`, the neo-go service block). Returns Emitted-and-enabled / Emitted-but-disabled / Not-emitted. This is what makes G20 impossible to reintroduce: a generator change that disables a switch changes the matrix, the picker and the CI assertion in the same build.
3. **`signer_route_support(node_type, role, backend_kind)`** — a new pure function extracted from the bodies of `ensure_local_wallet_runtime` and `ensure_sign_client_runtime`, called by **both** the launch path and the UI. Same function, two callers, no second declaration to drift. This is what turns neo-rs and neox Consensus from Supported-on-paper into Unsupported-in-fact.
4. **`runtime_layout_satisfied(node)`** — the neo-cli runtime-root gate (`node_signer.rs:405-424`), evaluated against an actual node, so it is a per-node blocker rather than a matrix cell, shown in the editor before Save and in Readiness rather than only at Start.

### 9.3 How the picker is built

- The node editor's duty control is generated by mapping `NodeRole::ALL` through `duty_support_for_node`. **Full** renders a normal option; **Caveat** renders a selectable option whose helper text is the caveat and its remedy; **Unsupported** and **Unverified** render `<option disabled>` with the reason as the label — visible, so an operator learns why neo-rs cannot be an oracle rather than wondering where the option went.
- An explicit **"No duty (unassigned)"** option exists and is the default. The presets that today resolve to no duty — `"relay"` and `"custom"` (`src/web/node_form.rs:150`) — are either mapped to a real duty or removed; a preset may not silently produce a duty-less node (G9).
- `apply_role` (`src/roles.rs:200-209`) gates on `duty_support_for_node`, not only on the local `role_availability` table, and rejects a POST for an unsupported duty even if the form is forged.
- `/roles` renders the same `duty_support` matrix, so the page an operator consults and the control they use can never disagree. The nav entry is renamed from "Private network" to **Duty support** (G45).

### 9.4 The gate that keeps it honest

A test iterating `NodeType::ALL × NodeRole::ALL` asserts that `duty_support`'s declared level equals the level obtained by actually rendering the config and calling the real signer-route function. A generator or gate change that makes a duty inert fails CI with the cell named. This is the parity mechanism the register asks for at sequencing item 4, applied to the one place it has already failed.

---

## 10. GAS and account state

Shown only where a duty gives an account chain-visible meaning: Consensus (the node's key may be a registered candidate and a voter) and the designated duties (the key is on chain). Never on RpcApi, Indexer or Observer nodes.

**Reads**, all `invokefunction` on native contracts, all available on any RPC node:

- `NeoToken.balanceOf(script_hash)`, `GasToken.balanceOf(script_hash)` — NEO and GAS held.
- `NeoToken.getAccountState(script_hash)` → `{balance, balanceHeight, voteTo}` — **who this account votes for**, which for a self-voting validator is the number behind its own standing.
- `NeoToken.unclaimedGas(script_hash, height)` — accrued, unclaimed.
- `NeoToken.getRegisterPrice()` — the current candidate registration price, read from chain rather than hardcoded, shown next to the "you are not a registered candidate" state.
- `OracleContract.getPrice()` — GAS per oracle response, on the Oracle panel.

**What the panel does not assert.** It does not claim a burn rate, a runway, or that a duty will fail at a given balance. Neo N3's designated services are not uniformly funded from the node's own account — oracle response fees are escrowed by the requester, state-root witnesses are extensible P2P payloads and not transactions at all — and the product has no verified model of this. The panel therefore **reports balances and their movement over the sampled window** and offers an optional, default-off, per-node low-balance threshold that the operator sets, with copy that says *"you asked to be told when this falls below X"* rather than *"this node will stop working"*.

Balances sample on the governance cadence (they move slowly and each read is an `invokefunction`), stored on `chain_candidate_samples`' schedule. All four fields are `Observation<T>`: a node whose key cannot be resolved shows *Unknown*, not zero.

---

## 11. Node page vs fleet page vs network page

The allocation rule: **a fact goes where its subject lives.** A per-node fact on the node page; a per-network fact once, on the network page; the fleet list carries only what an operator scans across rows.

| | Node page `/nodes/{id}?tab=chain` | Fleet `/nodes` | Network `/chain/{network}` |
|---|---|---|---|
| Height, head lag | ✓ full | ✓ column | ✓ per-node table |
| Liveness verdict | ✓ with findings | ✓ badge | ✓ per-node |
| Seconds since height change | ✓ | — | ✓ per-node |
| Reported vs configured network | ✓ **Critical when mismatched** | ✓ badge only when mismatched | ✓ reference node's value |
| Peers | ✓ count + sample + fleet cross-reference | ✓ count | — |
| Mempool | ✓ | — | ✓ reference node's |
| Round-trip | ✓ | — | — |
| Duty panel, ladder | ✓ | duty column only | — |
| Designation for this node | ✓ | ✓ badge when revoked/unknown | — |
| Designation holders per role | — | — | ✓ (incl. NeoFSAlphabet) |
| Committee / validators / candidates | ✓ **this node's standing only** | — | ✓ full tables |
| Block interval, throughput, primary distribution | ✓ when Consensus | — | ✓ network-wide |
| State-root validated index | ✓ when State/StateValidator | — | ✓ network health |
| Reference head, sample cadence | — | — | ✓ |

Two consequences worth stating:

- **Governance is never duplicated per node.** A node page shows *this node's standing* — rank, margin, producing or not — and links to the network page for the tables. Twelve nodes on one network produce one committee table, not twelve.
- **Per-node context is carried on every hop.** `/chain/{network}` rows link to `/nodes/{id}?tab=chain`; node findings link back with the network in the URL; the node activity card's `/events?q={name}` is fixed to `/events?query={name}&node={id}` (G46). Any new link added by this work is covered by the route/nav parity test.

---

## 12. Events

Every kind below is constructed by the sampler in the same commit that adds it — the register counts 51 of 93 existing kinds with no construction site (G36), and this surface does not add to that number.

| Kind | Severity | Trigger |
|---|---|---|
| `ChainLivenessChanged` | Info / Warning / Critical by target state | Edge-triggered on `chain_node_state.liveness`; Stalled ⇒ Critical, Lagging/Unreachable ⇒ Warning |
| `ChainNetworkMismatch` | **Critical** | Reported network ≠ configured, once per change |
| `ChainPeerConnectivityChanged` | Critical on Isolated, Warning on Sparse | Edge-triggered |
| `ChainMempoolCongested` | Warning / Critical | Entering Elevated / Congested, thresholds from `memorypoolmaxtransactions` |
| `ChainCommitteeChanged` | Info; **Warning** if a fleet key left; **Critical** if a fleet key holding the Consensus duty left | Committee digest change |
| `ChainValidatorSetChanged` | same escalation | Validator digest change |
| `ChainDesignationGranted` | Notice | §7.3 |
| `ChainDesignationRevoked` | **Critical** | §7.3 |
| `ChainDesignationSetChanged` | Info | §7.3 |
| `ChainDesignationUnknown` | Warning | Duty requires designation, no key resolvable |
| `ChainDesignationUnreadable` | Warning | N consecutive read failures |
| `ChainValidatorMissedProposal` | Warning | §6.4, only under the confidence conditions stated there |
| `ChainBlockProductionStalled` | **Critical** | Network-scoped: no new block across all nodes on a network beyond the stall multiple |

All are edge-triggered through `chain_node_state` / the append-on-change tables, following the existing `should_record_rpc_health_event` pattern (`src/health_events.rs:66-71`), so a healthy fleet writes nothing.

These events are also what finally give alert routing something numeric to scope on. This specification does not design the routing rules (G14 is its own work), but it deliberately gives every event a `node_id` and a stable `kind`, which is the minimum a per-node or per-severity rule needs — *"page on the validator, warn on observers"* becomes expressible without further changes to this module.

---

## 13. CLI and API parity

The CLI keeps working headless and gains node-keyed forms, since `node_rpc_endpoint(node)` and `node_type.family()` both already exist and the current signature makes an operator retype an endpoint the database holds (G12):

| Command | Notes |
|---|---|
| `--chain-node <node-id> [--json]` | The whole node panel. Exit 0 synced, 1 degraded, 2 unknown/unreadable. |
| `--governance <network> [--json]` | Network page data. `--governance-endpoint <url>` retained for a bare endpoint. |
| `--designation --node <node-id> [--role <role>] [--json]` | Exit 0 designated, 1 not designated, **2 key unknown or chain unreadable** — the collapse at `cli/actions/chain.rs:23-26` is removed. |
| `--peer-health --node <id>`, `--mempool-status --node <id>` | Existing endpoint forms retained. |
| `--duty-support [--json]` | Prints the §9.1 matrix, so CI can assert it. |

`GET /api/chain/nodes/{id}` and `GET /api/chain/networks/{network}` serialise the same `ChainFinding` vectors under the existing API-token permission model. The Prometheus exposition gains `neonexus_node_block_height`, `neonexus_node_head_lag_blocks`, `neonexus_node_peers_connected`, `neonexus_node_seconds_since_height_change` and `neonexus_chain_designated` with node and network labels — note that the first of these is already documented and does not exist (`docs/AGENT_API.md:266`), so this closes a documented-but-absent metric rather than inventing one.

---

## 14. Gates

**14.1 No literal status.** A lint over `src/web/pages/` rejecting literal `● OK`, `passed`, `Healthy`, `In Sync` outside the helper that renders a `ChainFinding` or an `Observation`. This is the register's own suggested rule (sequencing item 1), and this module is the first to be built under it.

**14.2 One source, three surfaces.** A test asserting that for a fixture workspace, the `ChainFinding` codes present in the rendered HTML, in `--chain-node --json`, and in the events emitted for the same sample are the same set. This is the R3 antidote applied at the point where R3 has already bitten.

**14.3 Fixtures have provenance.** Every file under `tests/fixtures/chain/` must carry a `_provenance` object naming client, version, network and capture date. A fixture without one fails the test that loads it. Hand-written fixtures that match the parser are why CI was blind to G11.

**14.4 Unknown is exercised.** For every panel, a test rendering it with zero samples asserting the output contains no status colour other than the Unknown style. A panel that reads green on an empty database fails.

**14.5 Duty matrix parity.** §9.4.

**14.6 Routes are reachable.** Every route added here has a nav entry or an inbound link from a rendered page, asserted by the parity test the register asks for.

---

## 15. Staging

Each stage leaves the product working and shippable.

**Stage 0 — subtract (≈1 day).** Delete the Monitoring tab's fabrications (G2) and the fake alarm tables that would otherwise sit next to real ones (G1). Purely subtractive; makes room for the Chain tab and stops the console lying during incidents immediately.

**Stage 1 — the sampler (largest single value).** `chain_samples`, `chain_node_state`, `ChainFinding`/`Observation`, the extended probe on the existing tick, the common header panel, the `/nodes` chain column, `--chain-node`. Ships head-lag, stall detection, peer and mempool verdicts with derived thresholds, round-trip, and the reported-vs-configured network check. "Running but not syncing" becomes detectable; `getversion.protocol` alone retires several invented constants.

**Stage 2 — the network page.** Governance sampler, `chain_governance_samples`, `chain_candidate_samples`, `/chain` and `/chain/{network}`, committee/validator change events, the nav rename. Read-only and independent of Stage 3.

**Stage 3 — designation.** Key resolution, `chain_designations`, the per-duty node sections, the revocation alarm, the ladder pairing that shows chain truth and local truth together. This is the stage that answers *"your Oracle designation was revoked at 03:14"*.

**Stage 4 — duty truth.** `duty_support`, the generator probe, the extracted `signer_route_support`, the derived picker, `/roles` rewritten, the CI parity gate. Stops G20/G21 regrowing and makes the editor stop offering duties that cannot run.

**Stage 5 — validator attribution.** `chain_blocks`, the block walk, primary/view derivation, block-interval and throughput panels, the missed-proposal counter. Gated on a fixture captured from a real validator that has seen a view change; ships the counter before the event.

**Stage 6 — account and GAS.** Native-contract balance reads, `voteTo`, unclaimed GAS, register price, the optional operator threshold.

---

## 16. Deliberate omissions

Stated so they are choices rather than gaps:

- **No signing, ever.** No designate button, no vote button, no candidate registration. `src/chain_state.rs:8-12` is the policy and this design does not touch it.
- **No live view number, no consensus peer identity.** §6.5, §6.6. Both are unanswerable over public JSON-RPC and are labelled as such on the page.
- **No signature attribution.** Which designated keys witnessed a given state root, and which validators signed a given block, would require verifying each signature against each key. Not done, and said.
- **No oracle request depth, no per-node response count.** No standard method exposes either.
- **No slashing surface.** Neo N3 dBFT has none; offering one would teach a wrong protocol model.
- **No consensus-log parsing in this work.** Named as a future source on the view-change panel rather than synthesised into a number, because log-string parsing against strings the clients never emit is precisely how the existing sync-progress path became inert.


---

### The Neo X Surface: making Neo X a first-class chain in NeoNexus
Neo X today is a Neo N3 node with a different binary: chain identity is a pure function of an enum recomputed in three places, peers and RPC namespaces are unrepresentable, WebSocket is displayed but never opened, and observation is two RPC calls. This spec introduces a persisted `networks` entity that owns chain identity, a per-node Neo X facet that owns the EVM surface (namespaces, WS, peers, datadir, metrics/authrpc ports), an argv flag-value extractor so declared/argv/observed identity can be reconciled instead of asserted, a `geth init` bootstrap action with a verifiable readiness contract, and an `eth_syncing`-driven observation mapping that makes "running but not syncing" detectable. It also settles the key model (secp256k1 keystore vs NEP-6/secp256r1, enforced at bind time), declares Neo X Consensus unlaunchable, and decides that the product models the N3↔X *relationship* (networks, parentage, one incident vocabulary) but not the bridge.

# The Neo X Surface

**Goal.** Neo X stops being "a Neo N3 node with a different binary" and becomes a chain with its own identity record, its own peering surface, its own RPC surface, and its own observation vocabulary — inside one IA, one alert model, and one CLI.

**Scope of the problem, from source.** Everything Neo X-specific in the product today is a `match node_type` in five files. There is no Neo X entity, no Neo X setting, and no Neo X form field:

- Chain identity is the pure function `neox_chain_id(Network, Option<&RuntimeConfigProfile>)` (`src/config/format/neox.rs:17`), recomputed independently in the generator (`src/config/generator/neox/geth.rs:30`), the validator (`src/config/validation/runtimes/neox/geth.rs:15`) and the readiness readout (`src/diagnostics/checks/chain.rs:38`) — all three with `profile: None` at launch, so all three produce `1_230_000` for every private network in the workspace.
- `has_chain_argument` (`src/diagnostics/checks/chain.rs:85-91`) tests for a flag's *presence*. There is no flag-value extraction anywhere in `src/`. That is why one readiness report can acknowledge `--networkid 12345` and print "chain id 1230000" in the line above.
- `static_nodes` / `trusted_nodes` are `Vec::new()` literals (`generator/neox/geth.rs:51-52`, `reth.rs:48`); private bootnodes are `Vec::new()` (`format/neox.rs:38`); there is no peer field on `NodeConfig` (`src/types/node.rs:22-36`) and no form control.
- `HTTP_MODULES` is a 4-element const for geth (`generator/neox/geth.rs:17`); `reth_args` (`src/launch/neox.rs:58-98`) emits no `--http.api` at all, so neox-rs comes up on Reth's `STANDARD_MODULES` = `[eth, net, web3]` — which is why `txpool_status` in `src/chain_state/mempool.rs:149` can only ever fail on one of the two clients.
- A ws port is validated (`src/types/ports.rs:8`), reserved (`src/port_planner/planner.rs`), rendered `ws://127.0.0.1:{p}` and badged **Open** (`src/web/pages/nodes/detail_tabs.rs:596-602`). geth opens it (the config carries `WSHost`/`WSPort`); neox-rs never does, because no `--ws` flag exists in `src/`. The badge is a literal either way.
- Observation is `web3_clientVersion` + `eth_blockNumber`, status = how many answered (`src/rpc_health/probe.rs:59-64`). `eth_syncing`, `eth_chainId`, `eth_gasPrice` appear nowhere. `net_peerCount` and `txpool_status` exist only in `src/chain_state/`, reachable only from a CLI that takes a hand-typed endpoint.

The register's G21/G22/G23 are three symptoms of one absence: **Neo X has no persisted state of its own, so every Neo X fact is either a constant or a string in `args`.**

---

## 0. The shape of the fix

Three new things, and one missing primitive.

| # | Thing | Owns |
|---|---|---|
| A | `networks` table (shared with the N3 chain-identity work, G18/G19) | Chain identity: id/magic, genesis, bootnodes, block period, kind (public/private), parentage |
| B | `node_neox` facet table + `node_peers` | The EVM surface of one node: namespaces, WS, metrics/authrpc ports, datadir, init state, peer overrides |
| C | `src/launch/argv_read.rs` — **flag-value extraction** | Reading what the operator actually typed, so declared/argv/observed can be reconciled |

**The pattern that replaces every fabricated Neo X value: the identity triple.** For each fact that can be stated three ways, store and render all three, and never print a constant as if it were observed:

```rust
pub struct Attested<T> {
    pub declared: Option<T>,   // the workspace's networks/node row
    pub argv:     Option<T>,   // extracted from node.args at plan time
    pub observed: Option<T>,   // read from the running node over RPC
    pub observed_at_unix: Option<u64>,
}
pub enum Agreement { Confirmed, DeclaredOnly, Overridden, Mismatch, Unknown }
```

`Agreement::Mismatch` on chain id or genesis hash is a Critical that outranks every other node state. `Unknown` renders the word "unknown", never a green pill. This one type is the Neo X answer to root cause R1.

---

## 1. Chain identity
