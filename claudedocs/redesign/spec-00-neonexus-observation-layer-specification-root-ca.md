### NeoNexus Observation Layer — specification (root cause R2)
Specifies a new `src/observe/` module that replaces the two-call reachability probe with a tiered sampler (per chain family, per cadence class), a set of derived quantities with exact formulas, a nine-state health machine whose central case is "answers RPC, height has not moved", a reference-head resolution ladder that degrades honestly to single-node and private networks, a `metric_samples`/rollup/health-transition/alarm schema with retention math (~190 MB steady state for 20 nodes), a closed-vocabulary alarm model with hysteresis, flap suppression and an explicit No-data state, and a Prometheus exposition that honours the three names docs/AGENT_API.md already promises. Everything is a pure function of stored samples, testable without a node, and lands in five stages that each leave the product working.

# The Observation Layer

**Scope:** root cause R2. Prerequisite for G1 (real alarms), G3 (real charts), G11 (stall
detection), G12 (peers/mempool on every surface), G13 (designation drift), G14 (scoped alert
rules), G15/G16 (Prometheus with a chain dimension), G17 (real `syncing_nodes`), G33 (health-aware
watchdog).

Today: `src/rpc_health/probe.rs:38-64` issues two calls and maps `ok_count` → `{Healthy, Degraded,
Unreachable}`; `src/supervision/probes.rs:42-48` probes **one node per tick** (so a 20-node fleet
gets each node every 20 s at best, a 100-node fleet every 100 s); `rpc_health_checks` keeps 24 rows
per node (`RPC_HEALTH_RETAIN_PER_NODE`, probes.rs:24) with no latency and no peers.

---

## 0. Design rules

These are the invariants the rest of the document implements. They exist because R1 and R2
compound: a vacuum in observation gets filled with green literals.

1. **NULL ≠ 0 ≠ false.** "Not sampled", "sampled and the method is unsupported", "sampled and the
   answer is zero" are three different values, stored differently and rendered differently.
2. **Evaluation is a pure function.** `health::evaluate(inputs) -> Verdict` takes a struct and
   returns a struct. No I/O, no clock, no database. This is the antidote to the tautological test
   noted at G11 (`tests/unit/supervision/tests.rs:167-206`): the fixtures become state vectors, not
   log lines copied out of the parser.
3. **Absence is a state, not a default.** `Unknown` is a first-class health state and `NoData` is a
   first-class alarm state. Neither is ever styled like `Healthy`/`OK`.
4. **Thresholds are derived, not compiled.** `msperblock` comes from the node
   (`getversion.protocol.msperblock`), peer expectations come from the fleet size on that network,
   mempool utilisation comes from `memorypoolmaxtransactions`. `classify_congestion`'s
   500/2000 constants (`src/chain_state/mempool.rs:174-182`) and `classify_connectivity`'s 0/1-2/3+
   (`peers.rs:171-177`) become functions of observed capacity.
5. **The observer never mutates the fleet.** It writes samples, derived state and events. Restarts
   stay in `src/supervision/restarts.rs`, gated by an explicit opt-in (see §9.6).
6. **One round trip, one stored latency.** Latency is measured on the class's *primary* method only,
   so `neonexus_node_rpc_latency_seconds` means one thing.

---

## 1. Module layout

```
src/observe.rs                    // pub use surface
src/observe/
  schedule.rs                     // due-queue, backoff, concurrency cap
  client.rs                       // timed JSON-RPC call, size cap, redaction
  capabilities.rs                 // per-node method support cache
  sample/
    mod.rs                        // SampleRound, NodeSample
    neo_n3.rs                     // N3 collectors, per class
    neox.rs                       // Neo X collectors, per class
  derive.rs                       // formulas of §3 (pure)
  health.rs                       // state machine of §4 (pure)
  reference.rs                    // reference-head ladder of §5
  governance.rs                   // per-network committee/validator/designation reader
  rollup.rs                       // downsampling + pruning
  alarm/
    model.rs   evaluate.rs   seed.rs

src/repository/schema/tables/observation.rs        // DDL of §6
src/repository/observation/{samples,health,rollups,alarms,governance,designations}.rs
src/supervision/observe.rs        // replaces probes.rs::probe_rpc_health
src/metrics/prometheus/families/{chain,health,alarms}.rs
```

`src/rpc_health/` stays, narrowed to what it is actually good at: a **one-shot liveness probe of a
bare endpoint** for the CLI (`--rpc-health <endpoint>`) and federation. `RpcHealthStatus` stops
being the node's health; `observe::HealthState` is.

---

## 2. What is sampled

### 2.1 Sample classes

Every sample is attributed to a class. A class has its own cadence, its own cost budget, and its own
failure semantics. A class that fails does not void the other classes in the same round.

| Class | Purpose | Failure means |
|---|---|---|
| `head` | height, header height, sync flag | node is not answering → drives `Unreachable` |
| `head_time` | head block's own timestamp + hash | chain-clock lag unavailable; falls back to local monotonic |
| `peers` | connection count | `Isolated` undeterminable → `peers = NULL`, not 0 |
| `peers_detail` | peer addresses, unconnected/bad counts | cosmetic; UI shows "not available" |
| `pool` | mempool depth + capacity | mempool alarms go `NoData` |
| `identity` | client version, chain magic/chain id, ms-per-block, validator count | thresholds fall back to configured defaults, flagged `assumed` |
| `governance` | committee, next validators (**per network, not per node**) | committee alarms go `NoData` |
| `governance_deep` | candidates and votes (**per network**) | as above |
| `designation` | `RoleManagement.getDesignatedByRole` (**per network, per role**) | duty state → `DutyUnknown`, never `DutyNotDesignated` |
| `anchor` | genesis / early-block hash, for wrong-chain detection | no chain-identity assertion |

### 2.2 Neo N3

| Method | What it gives | Cost on the node | Class | Default period |
|---|---|---|---|---|
| `getblockcount` | blocks held (= height + 1) | O(1), in-memory index | `head` | **15 s** |
| `getblockheadercount` | headers known | O(1) | `head` | 15 s, skipped if unsupported |
| `getblockheader <index-1> true` | `time` (**ms since epoch**), `hash`, `previousblockhash` | one header read, ~700 B | `head_time` | **60 s** |
| `getconnectioncount` | connected peer count | O(1) | `peers` | **15 s** |
| `getpeers` | connected/unconnected/bad arrays | O(peers), a few KB | `peers_detail` | **300 s** |
| `getrawmempool` | tx hashes | **O(pool)** — up to `memorypoolmaxtransactions` × 66 B; 50 000 entries ≈ 3.3 MB | `pool` | **120 s**, size-capped (§9.4) |
| `getversion` | `useragent`, `protocol.network` (magic), `protocol.msperblock`, `protocol.validatorscount`, `protocol.memorypoolmaxtransactions` | O(1), static | `identity` | **900 s** |
| `getcommittee` | 21 committee keys | contract storage read | `governance` | **300 s / network** |
| `getnextblockvalidators` | 7 validators for the next round | contract storage read | `governance` | **300 s / network** |
| `getcandidates` | all candidates + votes | iterates candidate storage, hundreds of entries | `governance_deep` | **3600 s / network** |
| `invokefunction` `RoleManagement.getDesignatedByRole` | designated keys for one role | a VM invoke | `designation` | **600 s / network / role** |
| `getblockhash 0` | genesis hash | O(1) | `anchor` | once, revalidated 24 h |

`getversion` is the highest-value slow call in the product and is currently thrown away
(`summarize_version`, `src/rpc_health/probe/summary.rs:21-35`, keeps only `useragent`). Its
`protocol` block supplies the block interval, the mempool capacity and — critically — the magic the
node *actually* joined, which is how G18's "private magic 1 230 000 dialling mainnet seeds" becomes
detectable at runtime instead of never.

### 2.3 Neo X

| Method | What it gives | Cost | Class | Default period |
|---|---|---|---|---|
| `eth_blockNumber` | latest block index (hex) | O(1) | `head` | **15 s** |
| `eth_syncing` | `false`, or `{startingBlock, currentBlock, highestBlock}` | O(1) | `head` | **15 s** |
| `net_peerCount` | peer count (hex) | O(1) | `peers` | **15 s** |
| `eth_getBlockByNumber("latest", false)` | `timestamp` (**seconds**), `hash`, `parentHash`, `gasUsed`, `gasLimit` | one header + tx-hash list (~13 KB on a full block) | `head_time` | **60 s** |
| `txpool_status` | `{pending, queued}` counters | O(1) | `pool` | **60 s** |
| `eth_gasPrice` | oracle over recent blocks, cached by the client | cheap, cached | `pool` | 60 s |
| `web3_clientVersion` | client string | O(1) | `identity` | **900 s** |
| `eth_chainId` | EIP-155 chain id | O(1) | `identity` | 900 s |
| `eth_getBlockByNumber("0x0", false)` | genesis hash, compared to `neox_genesis_hash()` (`src/config/format/neox.rs:47`) | O(1) | `anchor` | once, revalidated 24 h |

**Reverse the current mempool preference.** `src/chain_state/mempool.rs:136-146` tries
`eth_getBlockTransactionCountByNumber(["pending"])` first and falls back to `txpool_status`. On geth
the `pending` tag forces construction of a pending block — it is the expensive path. Order must be
`txpool_status` → (only if the `txpool` namespace is off) `eth_getBlockTransactionCountByNumber`, at
the slow cadence and behind a capability flag.

`eth_syncing` is exactly as cheap as `eth_blockNumber` and is the authoritative sync signal. It goes
in the fast class. This retires the log-scraping sync parsers, whose Neo X patterns match strings the
clients never emit (G11: geth emits `Imported new chain segment blocks=`, the parser looks for
`"Chain imported"` + `block=`).

### 2.4 Cadence is a policy, not a constant

```rust
// src/observe/schedule.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservationPolicy {
    pub enabled: bool,
    pub head_seconds: u64,        // 5..=300,    default 15
    pub head_time_seconds: u64,   // 15..=900,   default 60
    pub peers_seconds: u64,       // 5..=300,    default 15
    pub peers_detail_seconds: u64,// 60..=3600,  default 300
    pub pool_seconds: u64,        // 30..=3600,  default 120 (N3) / 60 (Neo X)
    pub identity_seconds: u64,    // 300..=86400,default 900
    pub governance_seconds: u64,  // 60..=3600,  default 300
    pub governance_deep_seconds: u64, // 600..=86400, default 3600
    pub designation_seconds: u64, // 120..=3600, default 600
    pub max_probes_per_tick: u8,  // 1..=16,     default 4
    pub probe_timeout_ms: u64,    // 500..=30000,default 3000
    pub allow_public_reference: bool, // default true; forced false for Network::Private
}
```

Persisted in `settings` with the existing key convention (`src/repository/settings_keys.rs`), prefix
`observation.`. Re-read every tick like the other policies (`LoopState::tick`,
`src/supervision/state.rs:101-110`), so a Settings change takes effect without a restart.

### 2.5 Capability cache

Not every client answers every method: `getblockheadercount` may be absent, `txpool` may be
disabled, `getpeers` may be missing on neo-rs.

```sql
CREATE TABLE IF NOT EXISTS node_rpc_capabilities (
    node_id      TEXT NOT NULL,
    method       TEXT NOT NULL,
    support      TEXT NOT NULL,          -- 'supported' | 'unsupported' | 'unknown'
    checked_at_unix INTEGER NOT NULL,
    detail       TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (node_id, method)
);
```

Rules: a JSON-RPC `-32601 method not found` (or `-32600`/404) marks `unsupported`; a transport error
never does (that is the node being down, not the method being absent). `unsupported` is retried at
most every 6 h, because a runtime upgrade can add the method. A column fed by an unsupported method
is stored as `NULL` and the UI renders **"not supported by this client"** — which is different from
both zero and unknown.

### 2.6 Per-network collapse

`governance`, `governance_deep` and `designation` are **not per node**. A 20-node mainnet fleet must
issue one `getcommittee` per 5 minutes, not 20. The scheduler elects a *reader*: the node on that
(network, family) with the lowest recent latency that is `Healthy` or `Syncing`, with a deterministic
tiebreak on node id and a fallback to the next candidate on failure. The answer is stored once per
network; per-node facts (is my key on the committee, is my key designated) are computed **locally**
by comparing the node's bound public key (`node_signer_bindings` →
`src/core/node_signer.rs:209/:225/:270`) against the stored key set. This collapses N invokes into 1
and is what makes G13 affordable.

### 2.7 Wiring into the tick

`src/supervision/observe.rs` replaces `LoopState::probe_rpc_health`. Instead of "find the first due
node, probe it inline on the supervision thread", the loop drains a due-queue:

```rust
pub(super) fn observe(&mut self, state: &EngineState) {
    let policy = state.repository.load_observation_policy().unwrap_or_default();
    if !policy.enabled { return; }
    let now = Instant::now();
    let due: Vec<Job> = self.schedule.pop_due(now, policy.max_probes_per_tick);
    // Jobs run on a small bounded pool; never two in flight for the same node.
    for finished in self.pool.dispatch_and_collect(due, policy.probe_timeout_ms) {
        state.repository.record_sample_round(&finished.round)?;
        let verdict = health::evaluate(self.inputs_for(state, &finished)?);
        if let Some(transition) = state.repository.apply_health_verdict(&finished.node_id, &verdict)? {
            state.journal(&node, transition.event_kind(), transition.severity(), transition.message());
        }
    }
    self.rollup.run_if_due(state, now);
    self.alarms.evaluate_if_due(state, now);
}
```

Blocking `ureq` calls must not run on the supervision thread — a 3 s timeout × 4 nodes stalls
restarts and alert routing for 12 s. Bounded pool, `max_probes_per_tick` dispatched, results
collected next tick.

---

## 3. What is derived

All formulas operate on stored samples, in `src/observe/derive.rs`, as pure functions over a slice of
samples ordered newest-first. Every one returns `Option<T>`; `None` is propagated, never coerced.

Let `W` be the derivation window (default 300 s), `h(t)` the `block_count` at sample time `t`,
`E` the expected block interval in seconds.

**Expected block interval** `E`
```
E = observed_ms_per_block / 1000                        if identity sample present
  = 15                                                   else, ChainFamily::NeoN3   (neo_cli.rs:75)
  = 5                                                     else, ChainFamily::NeoX    (neox.rs:75)
```
When the fallback is used, the derivation carries `confidence: Assumed` and the UI says
"assuming 15 s blocks (node has not reported `msperblock` yet)".

**Head lag** — blocks behind the reference (§5)
```
head_lag_blocks = reference_block_count - node_block_count
  if reference is None                    -> None  (NOT 0)
  if head_lag_blocks in -2..=0            -> 0     (normal propagation jitter)
  if head_lag_blocks < -2                 -> 0, and flag AheadOfReference{by}
```

**Chain lag** — how old the node's own head block is. *This is the single-node staleness signal.*
```
chain_lag_seconds = now_unix - head_block_time_unix
  N3:    head_block_time_unix = getblockheader.time / 1000    // N3 timestamps are milliseconds
  NeoX:  head_block_time_unix = block.timestamp               // EVM timestamps are seconds
  if head_block_time_unix > now_unix + 30 -> None, flag ClockSuspect
```
The ms/s normalisation is a correctness trap worth a dedicated test: a missed division by 1000
makes every N3 node read as 54 000 years behind.

**Time since height changed** — carried forward, not rescanned
```
on each head sample:
  if block_count > state.last_height      -> last_height_change_at_unix = sampled_at_unix
  if block_count < state.last_height      -> emit HeightRegressed{from,to}; reset window derivations
seconds_since_height_change = now_unix - last_height_change_at_unix
  if last_height_change_at_unix is NULL   -> now_unix - first_successful_sample_at_unix
```
A regression is a real incident — a reorg deeper than expected, a restored snapshot, or a datadir
swap — and must invalidate `blocks_per_minute` rather than produce a negative rate.

**Blocks per minute**
```
pick (h0,t0) = oldest sample with head_ok=1 in [now-W, now]
               (h1,t1) = newest such sample
require  t1 - t0 >= W/2   else -> None
require  h1 >= h0         else -> None (regression)
bpm = (h1 - h0) * 60 / (t1 - t0)
expected_bpm = 60 / E
```

**Peer trend** (W = 900 s)
```
peer_trend      = peers(t1) - peers(t0)
peers_min_in_W  = min over window
expected_peers  = match network {
    Private => min(3, nodes_on_same_network_and_magic - 1),
    _       => 3,
}
```
`expected_peers` is why a 4-node private network stops reading "Healthy" from the same constant as
mainnet (G12).

**Mempool trend and utilisation**
```
pool_total       = mempool_verified + mempool_unverified      (NULLs excluded; both NULL -> None)
pool_utilisation = pool_total / mempool_capacity              // capacity from getversion
pool_trend_per_min = (pool_total(t1) - pool_total(t0)) * 60 / (t1 - t0)
```
Congestion becomes `pool_utilisation >= 0.5 -> Elevated`, `>= 0.9 -> Congested`, with `None` when
capacity is unknown — replacing the compile-time 500/2000.

**Sync ETA**
```
catchup_bpm = bpm_node(W) - bpm_reference(W)      // reference must have >= W/2 of samples
if head_lag_blocks <= sync_exit_lag  -> Some(0)
if catchup_bpm <= 0.05               -> None, reason = "not catching up"
eta_seconds = head_lag_blocks / catchup_bpm * 60
```
Never render an ETA from `bpm_node` alone: a node syncing at 60 bpm against a chain producing 4 bpm
has a real ETA; a node syncing at 4 bpm against a chain producing 4 bpm has none, and the naive
formula would confidently predict the wrong one.

**RPC latency** — measured on the class's primary method (`getblockcount` / `eth_blockNumber`),
stored as `rpc_latency_ms INTEGER`. Window aggregate is `max` at the 1-minute rollup (with 4 samples
per minute there is no honest p95; the column is named `latency_max_ms` and nothing calls it a
percentile).

```rust
// src/observe/derive.rs
#[derive(Debug, Clone, PartialEq)]
pub struct Derived {
    pub expected_block_seconds: f64,
    pub block_interval_confidence: Confidence,     // Observed | Assumed
    pub head_lag_blocks: Option<i64>,
    pub reference: ReferenceQuality,
    pub chain_lag_seconds: Option<i64>,
    pub seconds_since_height_change: Option<u64>,
    pub blocks_per_minute: Option<f64>,
    pub expected_blocks_per_minute: f64,
    pub peers_connected: Option<u32>,
    pub expected_peers: u32,
    pub peer_trend: Option<i32>,
    pub pool_total: Option<u64>,
    pub pool_utilisation: Option<f64>,
    pub sync_eta_seconds: Option<u64>,
    pub rpc_latency_ms: Option<u32>,
    pub flags: Vec<DerivationFlag>, // AheadOfReference, ClockSuspect, HeightRegressed, ...
}
```

---

## 4. The health state machine

### 4.1 States

Nine node-level states. Evaluation is an **ordered guard chain, first match wins**, so precedence is
part of the definition.

```rust
// src/observe/health.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthState {
    Stopped, Starting, Unreachable, Unknown,
    Isolated, Stalled, Syncing, Degraded, Healthy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StallScope { Node, Chain }   // qualifier on Stalled only

#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub state: HealthState,
    pub scope: Option<StallScope>,
    pub reason: String,            // one sentence, operator-facing, always populated
    pub evidence: Vec<(&'static str, String)>, // ("head_lag","1284"), ("since_height_change","412s")
    pub next_action: Option<&'static str>,     // what to do, per §4.4
}
```

Constants (derived where possible, all overridable per fleet):

```
confirm_n              = 2 evaluations            // soft-state entry
unreachable_after      = 3 consecutive head failures
reachable_after        = 2 consecutive head successes
starting_grace         = neo-cli 180s, neo-go 60s, neo-rs 60s, neox-geth 120s, neox-reth 120s
stall_seconds          = clamp(20 * E, 60, 900)   // N3 -> 300s, Neo X -> 100s
sync_enter_lag         = max(10, ceil(120 / E))   // N3 -> 10,   Neo X -> 24
sync_exit_lag          = max(2,  ceil(30  / E))   // N3 -> 2,    Neo X -> 6
stale_sample_seconds   = 3 * head_seconds
latency_degraded_ms    = 1000
```

| # | State | Entry | Exit |
|---|---|---|---|
| 1 | **Stopped** | supervisor reports no process **and** the operator has not asked it to run | a launch is issued → `Starting` |
| 2 | **Starting** | process spawned, `now - started_at < starting_grace`, and no successful `head` sample since the spawn | first successful `head` sample → re-evaluate from #4; grace expiry with no success → `Unreachable` |
| 3 | **Unreachable** | process expected to be running and `consecutive_rpc_failures >= 3` | `consecutive_rpc_successes >= 2` → re-evaluate from #4 |
| 4 | **Unknown** | any of: `rpc_port == 0`; observation disabled; newest sample older than `stale_sample_seconds`; fewer than 2 successful samples so nothing is derivable | the condition clears |
| 5 | **Isolated** | `peers_connected == Some(0)` for `confirm_n` consecutive evaluations (only when peers is observable) | `peers_connected >= 1` |
| 6 | **Stalled** | see §4.2 | any `block_count` increase, immediately (no confirm delay on recovery) |
| 7 | **Syncing** | `eth_syncing != false`, **or** `head_lag_blocks > sync_enter_lag`, **or** `header_count - block_count > sync_enter_lag`; **and** height is advancing (`bpm > 0` or a height change within `stall_seconds`) | `head_lag_blocks <= sync_exit_lag` for `confirm_n` consecutive evaluations and `eth_syncing == false` |
| 8 | **Degraded** | reachable and in sync, but: `rpc_latency_ms > latency_degraded_ms` for `confirm_n`; **or** `0 < peers < expected_peers`; **or** a non-`head` class is erroring (not merely unsupported); **or** `bpm < expected_bpm / 2` while lag is within bounds | all listed conditions clear for `confirm_n` |
| 9 | **Healthy** | none of the above | — |

Asymmetric `sync_enter_lag`/`sync_exit_lag` is the hysteresis: a node does not flip to `Syncing`
because one block arrived late, and does not declare itself caught up until it is genuinely at the
head.

`Isolated` outranks `Stalled` deliberately: zero peers *causes* the stall, and the operator needs the
cause. `Stalled` outranks `Syncing` because a node that is behind and not moving is not syncing.

### 4.2 `Stalled` — the case that matters

A node that answers RPC promptly while its height has not moved is the failure this whole document
exists for. Its definition must be tight enough not to fire on a slow block and loose enough not to
need a reference node.

```rust
fn is_stalled(i: &HealthInputs) -> Option<StallScope> {
    // 1. The node is answering. This is what distinguishes Stalled from Unreachable.
    if !i.latest.head_ok { return None; }
    if i.derived.rpc_latency_ms? > i.policy.probe_timeout_ms as u32 { return None; }

    // 2. It has been running long enough for "no progress" to mean something.
    if i.state == HealthState::Starting { return None; }
    if i.node_suppressed_until_unix.is_some_and(|u| u > i.now_unix) { return None; } // snapshot/upgrade

    // 3. Two independent staleness witnesses; either one is sufficient.
    let local_stale = i.derived.seconds_since_height_change? >= i.stall_seconds;
    let chain_stale = i.derived.chain_lag_seconds
        .is_some_and(|lag| lag >= i.stall_seconds as i64);   // None when ClockSuspect
    if !(local_stale || chain_stale) { return None; }

    // 4. Confirm over consecutive evaluations, so one long block does not page anyone.
    if i.candidate_streak(HealthState::Stalled) + 1 < i.policy.confirm_n { return None; }

    // 5. Scope: is it this node, or the whole chain?
    Some(match i.reference {
        ReferenceHead::Known { block_count, seconds_since_reference_advanced, .. }
            if seconds_since_reference_advanced >= i.stall_seconds
               && block_count <= i.latest.block_count => StallScope::Chain,
        _ => StallScope::Node,
    })
}
```

Why two witnesses:

- **`local_stale`** (`seconds_since_height_change`) works with no reference and no clock trust, but
  cannot tell a stuck node from a halted chain, and is blind for the first `stall_seconds` after a
  fresh start.
- **`chain_stale`** (`now - head_block_time`) is immediate — a node that starts up and syncs to a
  three-day-old head is stale on its very first sample — but depends on the local clock and on the
  chain having honest timestamps. Guarded by the 30 s skew check; disabled with a visible
  `ClockSuspect` flag when the head block is in the future.

Scope changes what the operator does:

- `Stalled(Node)` — "neo-01 has held block 6 245 100 for 11 minutes while the network is at
  6 245 144. Check peers and disk; consider a restart."
- `Stalled(Chain)` — "no node on mainnet has advanced in 11 minutes. This is the network, not your
  fleet. Do not restart." That distinction is worth the whole reference-head apparatus: restarting a
  validator during a view change is exactly the wrong move.

The suppression check (step 2) matters because `SnapshotApplied` fast-sync
(`src/snapshots/control/maintenance.rs:47-49`) and the runtime upgrader
(`src/supervision/upgrade.rs`) both legitimately freeze a node. Both set
`node_health_state.suppressed_until_unix`.

### 4.3 Per-node vs per-duty

The nine states above are **per node**. Duty health is a **second, independent axis** per
`(node, duty)`. Collapsing them loses exactly the case G13 describes — node fine, duty silently
dead — so they are never rolled into one badge.

```rust
pub enum DutyState {
    Unknown,              // never evaluated, or the node has no bound key to compare
    NotApplicable,        // duty needs no chain designation (RpcApi, Indexer, Observer, State)
    Designated,           // key is in getDesignatedByRole for this role
    NotDesignated,        // key is known and is NOT in the set
    DesignationRevoked,   // was Designated, now is not (transition, held for 24h)
    Elected,              // Consensus: key in getnextblockvalidators
    CommitteeOnly,        // Consensus: key on committee but outside the top 7
    NotElected,           // Consensus: key registered as candidate, not in committee
    NotACandidate,        // Consensus: key not in getcandidates at all
    Misconfigured,        // duty claimed, but the launch path cannot perform it (G20, G21)
}
```

| Axis | Source | Cadence |
|---|---|---|
| node state | `head`/`peers` samples | 15 s |
| `Oracle`, `StateValidator`, `Notary` duty | `designation` class, compared locally | 600 s |
| `Consensus` duty | `governance` + `governance_deep`, compared locally | 300 s / 3600 s |
| `RpcApi`, `State`, `Indexer`, `Observer` | `NotApplicable` — nothing on chain to check | n/a |

`NodeRole::designation()` (`src/roles/role/model.rs:94-103`, currently zero non-test callers) is the
mapping from duty to `ChainRole`. This is its caller.

`DesignationRevoked` and the committee equivalent (`Elected` → `CommitteeOnly` → `NotElected`) are
**transitions**, and they are the headline the register asks for: *"your Oracle designation was
revoked at 03:14"*, *"your validator key left the top 7 at 09:02 — you are now committee-only and are
no longer producing blocks."* They are recorded in `node_designation_transitions` and emitted as
events with `Critical` severity for a duty the node is configured to perform.

### 4.4 Every state carries a next action

`Verdict.next_action` is a required field for non-`Healthy` states, because the 03:00 operator needs
*which node, what is wrong, what to do* in that order:

| State | `next_action` |
|---|---|
| `Unreachable` | "Check the process is alive and the RPC port is bound; open the last 200 log lines." |
| `Unknown(no RPC)` | "This node has `rpc_port = 0`. Chain state cannot be observed. Set an RPC port in the node editor." |
| `Isolated` | "0 peers. Check the seed list and P2P port reachability." |
| `Stalled(Node)` | "Height frozen while the network advances. Check disk space and peers before restarting." |
| `Stalled(Chain)` | "The network has stopped advancing. Do not restart — check the validator set." |
| `Syncing` | "Behind by N blocks, catching up at M blocks/min, ETA …" (or "not catching up") |
| `Degraded` | the specific sub-reason |

---

## 5. Where the reference head comes from

A four-rung ladder, resolved per `(network, family, observed_magic)`. The observed magic/chain-id is
part of the key: a node that joined magic 1 230 000 must never be compared against a node on
860 833 102, whatever the `Network` enum says. That guard is what turns G18 from invisible into a
loud `WrongChain` finding.

```rust
pub enum ReferenceQuality {
    Configured { endpoint_label: String },   // rung 1 — operator's own trusted RPC
    FleetMedian { contributors: u8 },        // rung 2 — >= 3 healthy fleet nodes, median
    FleetPair,                               // rung 2b — exactly 2 nodes, max, low confidence
    PublicSeed { host: String },             // rung 3 — opt-in, mainnet/testnet only
    SelfOnly,                                // rung 4 — no reference exists
}
```

**Rung 1 — operator-configured reference RPC.** A new table (§6.5). Sampled on the `head` cadence
with `node_id IS NULL, reference_id` set, so reference samples live in the same table and the same
derivations apply. Chosen when present and its newest sample is fresher than `stale_sample_seconds`.

**Rung 2 — fleet consensus.** The heads of all nodes on the same `(network, family, magic)` that are
`Healthy` or `Degraded` (not `Syncing`, not `Stalled` — a syncing node is not a reference) within the
last two intervals. **Median, not max.** Max is one buggy or forked node away from telling the whole
fleet it is behind; the median needs a majority to be wrong. Requires ≥ 3 contributors. With exactly
2, use the max and mark `FleetPair` so the UI can say "compared against one other node".

**Rung 3 — public seeds, opt-in.** For `Mainnet`/`Testnet` only, and only when
`allow_public_reference` is set. The RPC endpoints alongside the seeds already in
`src/config/format/network.rs:59-78` (`seedN.neo.org:10332` for mainnet, `seedNt5.neo.org:20332` for
testnet; the Neo X public RPCs for `NeoX`). Rate-limited hard (once per 60 s per network, never per
node), timeout 5 s, failure is silent and demotes to rung 2. **Forced off for `Network::Private`** —
there is no honest public reference for a network the operator invented, and quietly dialling
seed1.neo.org from a private-network workspace is both wrong and a data-egress surprise.

**Rung 4 — self-only.** `head_lag_blocks` is `None`. The UI renders "no reference head — lag cannot
be computed (single node on this network)" with a link to add a reference endpoint. Alarms scoped to
`HeadLagBlocks` render `NoData`, never `Ok`. The state machine still works: `Stalled` fires from
`chain_lag_seconds` and `seconds_since_height_change`, and `Syncing` from `eth_syncing` /
`header_count - block_count`. This is the honest answer for a single-node private network, and it is
usable — a developer's one-node private chain that stops producing blocks still shows `Stalled`,
because its head block timestamp ages.

**Private-network specifics.** For `Network::Private`, `E` comes from the generated config
(`MillisecondsPerBlock` for N3, `neox_block_period_secs` for Neo X) when `getversion` has not
answered yet, and `expected_peers = nodes_on_network - 1`. A one-validator private chain that only
produces blocks when there are transactions is a real and legitimate configuration: mark the network
`on_demand_blocks` (a per-network setting) and `stall_seconds` stops applying — `Stalled` is
suppressed and the node shows `Healthy (idle chain)` instead. Without that flag, a dev private
network would page its owner every five minutes.

---

## 6. Storage

All DDL in `src/repository/schema/tables/observation.rs`, called from `create_tables`. New indexes in
`src/repository/schema/indexes.rs`. Timestamps are unix **seconds** to match the existing schema;
latency is **milliseconds** as `INTEGER`.

### 6.1 `metric_samples`

One row per node per **sampling round**, where a round runs at the fast (`head`) cadence. Slower
classes fill their columns only on the rounds they are due, leaving `NULL` elsewhere. This keeps the
row count at the fast rate rather than the sum of all class rates, and SQLite stores a `NULL` in one
byte.

```sql
CREATE TABLE IF NOT EXISTS metric_samples (
    id                      INTEGER PRIMARY KEY AUTOINCREMENT,
    sampled_at_unix         INTEGER NOT NULL,
    node_id                 TEXT,          -- exactly one of node_id / reference_id
    reference_id            TEXT,
    family                  TEXT NOT NULL, -- 'neo-n3' | 'neo-x'

    -- per-class outcome: NULL = not attempted this round
    head_ok                 INTEGER,       -- 1 ok, 0 failed
    head_time_ok            INTEGER,
    peers_ok                INTEGER,
    pool_ok                 INTEGER,
    identity_ok             INTEGER,
    error_detail            TEXT,          -- first failing class, redacted, <=200 chars

    -- head
    rpc_latency_ms          INTEGER,       -- primary method round trip
    block_count             INTEGER,       -- normalised: number of blocks held
    header_count            INTEGER,       -- N3 getblockheadercount; NULL on Neo X
    syncing                 INTEGER,       -- Neo X eth_syncing: 1/0; NULL elsewhere
    sync_highest_block      INTEGER,       -- Neo X eth_syncing.highestBlock

    -- head_time
    head_block_time_unix    INTEGER,       -- normalised to SECONDS (N3 divides by 1000)
    head_block_hash         TEXT,

    -- peers
    peers_connected         INTEGER,
    peers_unconnected       INTEGER,
    peers_bad               INTEGER,

    -- pool
    mempool_verified        INTEGER,
    mempool_unverified      INTEGER,
    mempool_capacity        INTEGER,       -- memorypoolmaxtransactions / txpool limit
    gas_price_wei           INTEGER,       -- Neo X only

    -- identity
    observed_magic          INTEGER,       -- N3 protocol.network / Neo X eth_chainId
    observed_ms_per_block   INTEGER,
    observed_validators     INTEGER,
    client_version          TEXT,

    CHECK ((node_id IS NULL) <> (reference_id IS NULL)),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_metric_samples_node_recent
    ON metric_samples (node_id, sampled_at_unix DESC);
CREATE INDEX IF NOT EXISTS idx_metric_samples_reference_recent
    ON metric_samples (reference_id, sampled_at_unix DESC) WHERE reference_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_metric_samples_prune
    ON metric_samples (sampled_at_unix);
```

The `ON DELETE CASCADE` fixes half of G37 by construction. (The `api_tokens` half is separate.)

### 6.2 `node_health_state` — one row per node, the machine's memory

```sql
CREATE TABLE IF NOT EXISTS node_health_state (
    node_id                     TEXT PRIMARY KEY,
    state                       TEXT NOT NULL,     -- HealthState
    stall_scope                 TEXT,              -- 'node' | 'chain' | NULL
    since_unix                  INTEGER NOT NULL,  -- when this state was entered
    evaluated_at_unix           INTEGER NOT NULL,
    reason                      TEXT NOT NULL,
    next_action                 TEXT,

    last_block_count            INTEGER,
    last_height_change_at_unix  INTEGER,
    first_sample_at_unix        INTEGER,
    consecutive_rpc_failures    INTEGER NOT NULL DEFAULT 0,
    consecutive_rpc_successes   INTEGER NOT NULL DEFAULT 0,
    candidate_state             TEXT,              -- state under consideration
    candidate_count             INTEGER NOT NULL DEFAULT 0,

    head_lag_blocks             INTEGER,
    reference_quality           TEXT NOT NULL,     -- ReferenceQuality discriminant
    observed_magic              INTEGER,
    observability               TEXT NOT NULL,     -- 'rpc' | 'process-only'
    sampling_interval_seconds   INTEGER NOT NULL,  -- current, after backoff
    suppressed_until_unix       INTEGER,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);
```

### 6.3 `node_health_transitions` — the incident timeline

```sql
CREATE TABLE IF NOT EXISTS node_health_transitions (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id             TEXT NOT NULL,
    changed_at_unix     INTEGER NOT NULL,
    from_state          TEXT,                -- NULL for the first observation ever
    to_state            TEXT NOT NULL,
    stall_scope         TEXT,
    reason              TEXT NOT NULL,
    head_lag_blocks     INTEGER,
    peers_connected     INTEGER,
    seconds_since_height_change INTEGER,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_node_health_transitions_recent
    ON node_health_transitions (node_id, changed_at_unix DESC, id DESC);
```

This table — not a hardcoded SVG path — is what the node page's history strip renders.

### 6.4 `metric_rollups` — downsampling

```sql
CREATE TABLE IF NOT EXISTS metric_rollups (
    bucket_seconds      INTEGER NOT NULL,   -- 60 or 300
    bucket_start_unix   INTEGER NOT NULL,
    node_id             TEXT NOT NULL,
    samples             INTEGER NOT NULL,
    head_ok_samples     INTEGER NOT NULL,
    block_count_last    INTEGER,
    block_count_delta   INTEGER,
    head_lag_min        INTEGER,
    head_lag_max        INTEGER,
    head_lag_last       INTEGER,
    chain_lag_max       INTEGER,
    latency_min_ms      INTEGER,
    latency_avg_ms      INTEGER,
    latency_max_ms      INTEGER,
    peers_min           INTEGER,
    peers_avg           INTEGER,
    peers_max           INTEGER,
    pool_last           INTEGER,
    pool_max            INTEGER,
    worst_state         TEXT NOT NULL,      -- worst HealthState seen in the bucket
    seconds_not_healthy INTEGER NOT NULL,
    PRIMARY KEY (bucket_seconds, node_id, bucket_start_unix),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) WITHOUT ROWID;
```

Rollup job, in `src/observe/rollup.rs`, runs on the tick:
- every 60 s: build the 60-second bucket for the **previous completed** minute from `metric_samples`;
- every 300 s: build the 300-second bucket from the five completed 60-second buckets (not from raw —
  half the I/O and identical results for min/max/last; `avg` is sample-count-weighted).

Retention, run after each rollup pass:

| Tier | Resolution | Kept |
|---|---|---|
| `metric_samples` | 15 s | **6 hours** |
| `metric_rollups(60)` | 1 min | **7 days** |
| `metric_rollups(300)` | 5 min | **90 days** |
| `node_health_transitions` | event | **365 days** |
| `chain_observations` | on change + 6 h heartbeat | 365 days |
| `alarm_transitions` | event | 365 days |

Raw pruning is `DELETE FROM metric_samples WHERE sampled_at_unix < ?` in batches of 5 000 inside one
transaction, so a long-idle workspace catching up does not hold a write lock for seconds. This
replaces `prune_rpc_health_keep_recent_per_node` (`src/repository/events_health/rpc_health.rs:73`),
which runs a full per-node `DELETE … NOT IN (SELECT … LIMIT 24)` **on every single probe**.

### 6.5 Reference endpoints, governance, designations

```sql
CREATE TABLE IF NOT EXISTS reference_endpoints (
    id              TEXT PRIMARY KEY,
    label           TEXT NOT NULL,
    endpoint        TEXT NOT NULL UNIQUE,
    family          TEXT NOT NULL,
    network         TEXT NOT NULL,
    enabled         INTEGER NOT NULL DEFAULT 1,
    source          TEXT NOT NULL,        -- 'operator' | 'seed'
    created_at_unix INTEGER NOT NULL,
    updated_at_unix INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chain_observations (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    observed_at_unix    INTEGER NOT NULL,
    network             TEXT NOT NULL,
    family              TEXT NOT NULL,
    observed_magic      INTEGER,
    source_node_id      TEXT,
    block_count         INTEGER,
    committee_json      TEXT,             -- ["02...", ...]
    validators_json     TEXT,
    candidates_json     TEXT,             -- [{publickey,votes,active}]
    digest              TEXT NOT NULL,    -- sha256(committee || validators)
    reason              TEXT NOT NULL     -- 'changed' | 'heartbeat'
);
CREATE INDEX IF NOT EXISTS idx_chain_observations_recent
    ON chain_observations (network, family, observed_at_unix DESC);

CREATE TABLE IF NOT EXISTS node_designations (
    node_id             TEXT NOT NULL,
    chain_role          INTEGER NOT NULL,  -- ChainRole discriminant: 4,8,16,32
    observed_at_unix    INTEGER NOT NULL,
    status              TEXT NOT NULL,     -- 'designated'|'not-designated'|'no-key'|'query-failed'
    node_public_key     TEXT,
    designated_count    INTEGER,
    height              INTEGER,
    detail              TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (node_id, chain_role),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS node_designation_transitions (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id         TEXT NOT NULL,
    chain_role      INTEGER NOT NULL,
    changed_at_unix INTEGER NOT NULL,
    from_status     TEXT,
    to_status       TEXT NOT NULL,
    height          INTEGER,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);
```

`chain_observations` inserts **only when the digest changes**, plus one heartbeat row every 6 h so
"last checked" is an honest timestamp rather than an inference from the newest change. That is what
keeps a 5-minute governance poll from writing 288 identical rows per network per day.

`status = 'no-key'` is the `includes_node_key: None` case the existing model already gets right
(`src/chain_state/model.rs:34-36`) and must survive into storage: a node with no signer binding is
**not** "not designated", it is "no key to compare".

### 6.6 Disk cost for a 20-node fleet

Row sizes measured against SQLite's varint encoding: small integers cost 1–3 bytes, `node_id`
(`node-<uuid>`, 41 bytes) dominates.

| Table | Row | Rows/node | Bytes/node |
|---|---|---|---|
| `metric_samples` @15 s, 6 h | ~110 B + ~50 B index | 1 440 | 0.23 MB |
| `metric_rollups(60)` @7 d | ~200 B | 10 080 | 2.0 MB |
| `metric_rollups(300)` @90 d | ~200 B | 25 920 | 5.2 MB |
| `node_health_transitions` @365 d | ~120 B | ~3 600 (10/day) | 0.4 MB |
| **Steady state per node** | | | **≈ 7.9 MB** |

- 20 nodes → **≈ 158 MB of data, ≈ 190 MB on disk with indexes.**
- Growth is **bounded**: the 90-day tier fills over the first quarter (≈ 60 MB/month for the first
  three months, 20 nodes) and then stops. Monthly growth after day 90 is zero.
- **+7.9 MB per additional node.** A 100-node fleet costs ≈ 950 MB; at that size drop the raw tier to
  2 h and the 5-minute tier to 30 days (≈ 320 MB) via the retention policy, which is a setting.
- Halving `head_seconds` to 7.5 s doubles only the raw tier (+0.23 MB/node). The 90-day tier is the
  cost driver, and it is resolution-independent.

Set `PRAGMA auto_vacuum = INCREMENTAL` at creation and run `PRAGMA incremental_vacuum(1000)` after
each prune; otherwise deleted pages are retained and the file only ever grows to its high-water mark.
Existing workspaces cannot change `auto_vacuum` without a full `VACUUM` — do it once, in the
migration, behind a size check, and say so in the changelog rather than silently blocking startup on
a multi-hundred-megabyte rewrite.

---

## 7. The alarm model

### 7.1 A closed vocabulary, not a query language

```rust
// src/observe/alarm/model.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlarmMetric {
    // numeric, from derivations
    HeadLagBlocks,
    SecondsSinceHeightChange,
    ChainLagSeconds,
    PeersConnected,
    RpcLatencyMs,
    MempoolUtilisationPercent,
    BlocksPerMinute,
    SampleAgeSeconds,
    RestartsInLastHour,
    // categorical, from the state machines
    HealthStateIs(HealthState),
    DutyStateIs(DutyState),
    ChainIdentityMismatch,       // observed_magic != configured magic
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison { AtLeast, AtMost, Equals }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmScope {
    Fleet,
    Network(Network),
    Family(ChainFamily),
    Duty(NodeRole),
    Node(String),
}

pub struct AlarmRule {
    pub id: String,
    pub name: String,               // "NeoNode-BlockHeight-Stall" — now a real row
    pub enabled: bool,
    pub metric: AlarmMetric,
    pub comparison: Comparison,
    pub threshold: f64,
    pub recovery_threshold: Option<f64>,   // defaults to `threshold`
    pub for_seconds: u32,                  // breach must persist this long
    pub recovery_for_seconds: u32,         // default 3 * for_seconds
    pub scope: AlarmScope,
    pub severity: EventSeverity,
    pub description: String,
    pub builtin: bool,
}
```

Every rule resolves its scope to a node set at evaluation time and produces one alarm instance per
`(rule_id, node_id)`. `AlarmScope::Duty(Consensus)` is how "page on the validator, warn on observers"
(G14) becomes expressible — two rules, different scopes, different severities.
