### 7.2 Alarm state — and the state that must not be green

```rust
pub enum AlarmState {
    NoData(NoDataReason),   // renders neutral/grey, with the reason. NEVER "OK".
    Ok,
    Pending,                // breaching, but not yet for `for_seconds`
    Alarm,
    Suppressed,             // maintenance window or flap hold
}

pub enum NoDataReason {
    NeverEvaluated,             // the rule was created and no tick has run yet
    NoSamples,                  // node has never answered
    SamplingDisabled,           // rpc_port == 0, or observation policy off
    MetricUnavailable(&'static str), // e.g. "no reference head", "client does not support txpool"
    NodeStopped,
}
```

```sql
CREATE TABLE IF NOT EXISTS alarm_states (
    rule_id                 TEXT NOT NULL,
    node_id                 TEXT NOT NULL,
    state                   TEXT NOT NULL,
    no_data_reason          TEXT,
    since_unix              INTEGER NOT NULL,
    evaluated_at_unix       INTEGER,           -- NULL = never evaluated
    observed_value          REAL,
    datapoints              INTEGER NOT NULL DEFAULT 0,
    breach_started_at_unix  INTEGER,
    clear_started_at_unix   INTEGER,
    flapping                INTEGER NOT NULL DEFAULT 0,
    transitions_in_window   INTEGER NOT NULL DEFAULT 0,
    reason                  TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (rule_id, node_id),
    FOREIGN KEY (rule_id) REFERENCES alarm_rules(id) ON DELETE CASCADE,
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
);
```

**The `NoData` rule is load-bearing.** `evaluated_at_unix IS NULL` or `datapoints = 0` **must** render
as "No data — never evaluated" / "No data — this node has never answered RPC", in the neutral badge
style, with the reason visible. Enforce it at the type level: the renderer takes `AlarmState`, not a
bool, and `AlarmState` has no `Default`. A rendering test asserts that a freshly seeded rule against
a freshly created node produces the string "No data" and not "OK" — that test is the direct
regression guard for G1, where four alarms read `● OK` because nothing ever evaluated them.

### 7.3 Hysteresis and flap suppression

- **Separate enter/exit thresholds.** `threshold` to enter, `recovery_threshold` to leave. Seeded
  defaults keep a visible gap (e.g. head lag enters at 50 and clears at 5), so a node oscillating at
  the boundary does not oscillate the alarm.
- **Separate durations.** `for_seconds` to fire, `recovery_for_seconds` (default 3×) to clear.
  Asymmetry is deliberate: alarms should be quick to fire and slow to clear.
- **Flap detection.** Count `Alarm ⇄ Ok` transitions in a rolling 900 s window. At ≥ 5, set
  `flapping = 1`, hold the alarm at its more severe state, emit **one** `AlarmFlapping` event, and
  stop routing further notifications for that `(rule, node)` until 900 s of quiet. The alarm still
  shows as flapping in the UI — suppression is about paging, not about hiding.
- **Maintenance suppression.** `node_health_state.suppressed_until_unix` (set by snapshot apply,
  runtime upgrade, operator-initiated stop) forces `Suppressed` and blocks routing. A workspace-wide
  `observation.maintenance_until_unix` does the same for the fleet.
- **Stopped nodes** do not alarm on chain metrics: a deliberately stopped node produces `NoData
  (NodeStopped)`, not `Alarm`. Stopping a node to work on it must not page the person doing it.

### 7.4 Routing

Alarm transitions are written to `runtime_events` with `node_id` set and new kinds
(`AlarmRaised`, `AlarmCleared`, `AlarmFlapping`, `AlarmNoData`), so the existing delivery path
(`src/supervision/alerts.rs`, `src/alerts/routing.rs`) carries them with no change to its transport.
`should_route_alert` gains a rule-aware branch: severity floor **and** rule scope, which is the
minimum needed to express "Critical to PagerDuty, Warning to Slack" once a second route row exists.
The existing PagerDuty/Opsgenie/Datadog dedup key (`src/alerts/payloads/common.rs:16`) is extended to
`(rule_id, node_id)` so a raise and its clear correlate into one incident.

### 7.5 Seeded built-in rules

Ship these as `builtin = 1` rows in a schema seed — the same four names the Alerts page currently
fakes (`src/web/pages/alerts.rs:137-162`), now backed by real evaluation:

| Name | Metric | Enter | For | Clear | Scope | Severity |
|---|---|---|---|---|---|---|
| `neonexus-block-height-stall` | `SecondsSinceHeightChange` ≥ | `20 × E` | 120 s | ≤ `2 × E` | Fleet | Critical |
| `neonexus-head-lag-high` | `HeadLagBlocks` ≥ | 50 | 300 s | ≤ 5 | Fleet | Warning |
| `neonexus-peer-count-low` | `PeersConnected` ≤ | `expected_peers − 1` | 120 s | ≥ `expected_peers` | Fleet | Warning |
| `neonexus-peers-isolated` | `HealthStateIs(Isolated)` | — | 60 s | — | Fleet | Critical |
| `neonexus-rpc-latency-high` | `RpcLatencyMs` ≥ | 1000 | 300 s | ≤ 400 | Fleet | Warning |
| `neonexus-validator-not-elected` | `DutyStateIs(NotElected)` | — | 0 s | — | Duty(Consensus) | Critical |
| `neonexus-designation-revoked` | `DutyStateIs(DesignationRevoked)` | — | 0 s | — | Duty(Oracle), Duty(StateValidator), Duty(Notary) | Critical |
| `neonexus-chain-identity-mismatch` | `ChainIdentityMismatch` | — | 0 s | — | Fleet | Critical |

`neonexus-signer-lease-expiring` from the current fake table is **not** seeded: signer leases have no
TTL (G44 — the TTL vocabulary is invented; exclusivity is enforced by a unique index at
`src/repository/schema/migrations.rs:112`). Do not build an evaluator for a quantity that does not
exist; delete the row instead.

---

## 8. Prometheus exposition

`docs/AGENT_API.md:266/271/276` already publishes three names. They are a contract; honour them
exactly, including the documented `node` label (the display name). `node_id` is added alongside as
the stable identity, so a rename does not silently split a series without warning.

New families in `src/metrics/prometheus/families/{chain,health,alarms}.rs`, appended by
`snapshot_to_text` (`src/metrics/prometheus.rs:24-31`). Existing workspace/system/process families
are unchanged.

**Absence rule:** a node with no sample emits **no chain series at all.** Prometheus semantics are
that absence and zero are different, and emitting `neonexus_node_block_height … 0` for an unprobed
node is the metrics-layer version of `● OK`. `neonexus_node_sample_age_seconds` is what a scraper
alerts on for staleness.

```prometheus
# HELP neonexus_node_running Process running state (1 = running, 0 = stopped).
# TYPE neonexus_node_running gauge
neonexus_node_running{node="neo-01",node_id="node-a1b2",type="neo-cli",chain="neo-n3",network="mainnet"} 1

# HELP neonexus_node_observable Whether chain state can be sampled from this node (1 = RPC sampling enabled and the port is set).
# TYPE neonexus_node_observable gauge
neonexus_node_observable{node="neo-01",node_id="node-a1b2"} 1

# HELP neonexus_node_block_height Index of the highest block the node holds (block count minus one).
# TYPE neonexus_node_block_height gauge
neonexus_node_block_height{node="neo-01",node_id="node-a1b2",chain="neo-n3",network="mainnet"} 6245099

# HELP neonexus_node_header_height Index of the highest block header the node knows; Neo N3 only.
# TYPE neonexus_node_header_height gauge

# HELP neonexus_node_rpc_latency_seconds Round trip of the most recent head probe.
# TYPE neonexus_node_rpc_latency_seconds gauge
neonexus_node_rpc_latency_seconds{node="neo-01",node_id="node-a1b2"} 0.0034

# HELP neonexus_node_rpc_probes_total Head probes attempted since the workbench started.
# TYPE neonexus_node_rpc_probes_total counter
# HELP neonexus_node_rpc_probe_failures_total Head probes that did not answer.
# TYPE neonexus_node_rpc_probe_failures_total counter

# HELP neonexus_node_head_lag_blocks Blocks behind the reference head; absent when no reference exists.
# TYPE neonexus_node_head_lag_blocks gauge
# HELP neonexus_reference_head_block_height Reference head used for lag, by source.
# TYPE neonexus_reference_head_block_height gauge
neonexus_reference_head_block_height{network="mainnet",chain="neo-n3",source="fleet-median",contributors="4"} 6245101

# HELP neonexus_node_chain_lag_seconds Wall clock minus the timestamp of the node's own head block.
# TYPE neonexus_node_chain_lag_seconds gauge
# HELP neonexus_node_seconds_since_height_change Seconds since this node's block height last increased.
# TYPE neonexus_node_seconds_since_height_change gauge
# HELP neonexus_node_blocks_per_minute Observed block rate over the derivation window.
# TYPE neonexus_node_blocks_per_minute gauge
# HELP neonexus_node_expected_blocks_per_minute Block rate implied by the chain's configured block interval.
# TYPE neonexus_node_expected_blocks_per_minute gauge
# HELP neonexus_node_sync_eta_seconds Estimated seconds to reach the reference head; absent when not catching up.
# TYPE neonexus_node_sync_eta_seconds gauge

# HELP neonexus_node_peers_connected Peers currently connected.
# TYPE neonexus_node_peers_connected gauge
# HELP neonexus_node_peers_expected Peers this node should have on its network.
# TYPE neonexus_node_peers_expected gauge
# HELP neonexus_node_peers_unconnected Known but unconnected peers; Neo N3 only.
# TYPE neonexus_node_peers_unconnected gauge
# HELP neonexus_node_peers_bad Peers the node has marked bad; Neo N3 only.
# TYPE neonexus_node_peers_bad gauge

# HELP neonexus_node_mempool_transactions Transactions in the node's pool.
# TYPE neonexus_node_mempool_transactions gauge
neonexus_node_mempool_transactions{node="neo-01",node_id="node-a1b2",state="verified"} 18
neonexus_node_mempool_transactions{node="neo-01",node_id="node-a1b2",state="unverified"} 0
# HELP neonexus_node_mempool_capacity Maximum transactions the node's pool will hold.
# TYPE neonexus_node_mempool_capacity gauge
# HELP neonexus_node_gas_price_wei Gas price the node's oracle reports; Neo X only.
# TYPE neonexus_node_gas_price_wei gauge

# HELP neonexus_node_sample_age_seconds Age of the most recent successful sample.
# TYPE neonexus_node_sample_age_seconds gauge

# HELP neonexus_node_health_state Current health state; exactly one series per node is 1.
# TYPE neonexus_node_health_state gauge
neonexus_node_health_state{node="neo-01",node_id="node-a1b2",state="healthy"} 1
neonexus_node_health_state{node="neo-01",node_id="node-a1b2",state="stalled"} 0
# ... one series per HealthState variant, kube-state-metrics style

# HELP neonexus_node_duty_state Duty state per node and duty; exactly one series per pair is 1.
# TYPE neonexus_node_duty_state gauge
neonexus_node_duty_state{node="neo-01",node_id="node-a1b2",duty="oracle",state="designated"} 1

# HELP neonexus_node_committee_member Whether this node's key is on the Neo N3 committee (1/0); absent when no key is bound.
# TYPE neonexus_node_committee_member gauge
# HELP neonexus_node_next_validator Whether this node's key is in the next round's validator set (1/0).
# TYPE neonexus_node_next_validator gauge
# HELP neonexus_chain_committee_size Committee members observed on this network.
# TYPE neonexus_chain_committee_size gauge

# HELP neonexus_alarm_state Alarm state per rule and node; exactly one series per pair is 1.
# TYPE neonexus_alarm_state gauge
neonexus_alarm_state{rule="neonexus-block-height-stall",node="neo-01",severity="critical",state="ok"} 1
neonexus_alarm_state{rule="neonexus-head-lag-high",node="neo-02",severity="warning",state="no_data"} 1

# HELP neonexus_observer_scheduler_queue_depth Sampling jobs waiting to run.
# TYPE neonexus_observer_scheduler_queue_depth gauge
# HELP neonexus_observer_samples_total Sample rounds completed, by outcome.
# TYPE neonexus_observer_samples_total counter
```

`neonexus_node_block_height` deliberately publishes **height**, not count: the documented HELP text
says "Latest observed block height", and `block_count - 1` is the index of the highest block held. A
unit test asserts `metric == sample.block_count - 1` and a comment states the relationship, so the
one-off between `getblockcount` and `eth_blockNumber` (already normalised in
`src/rpc_health/probe/methods.rs:48-55`) is not re-introduced downstream.

Label cardinality: bounded by node count × a fixed set of states. A 100-node fleet produces roughly
100 × (9 health + 8 duty + ~20 numeric + rules × alarm states). With 8 seeded rules and 5 alarm states
that is ~7 700 series — fine for a single scrape target, and there is no unbounded label anywhere
(no `pid` on chain series; `pid` stays on the process family where it belongs).

---

## 9. Cost control and degradation

### 9.1 Scheduler, not a spin loop

A binary heap keyed by `next_due_at` over `(node_id, SampleClass)` jobs, plus per-network jobs for
`governance`/`designation`. Invariants:
- never two in-flight requests to the same node;
- at most `max_probes_per_tick` (default 4) dispatched per second;
- a job that overruns its timeout is abandoned, not awaited — its round is stored with
  `head_ok = 0` and `error_detail = "timeout after 3000ms"`.

Steady-state RPC load per node at defaults: 4 head + 4 peers per minute, 1 head_time per minute,
0.5 pool per minute, 0.07 identity per minute ≈ **9.6 requests/minute/node**, of which all but ~1.5
are O(1) in-memory reads. A 20-node fleet is ~3.2 requests/second across the whole fleet.

### 9.2 Failure backoff

On consecutive head failures, multiply the node's interval by 2 each time, capped at 8× (15 s → 120 s),
and skip all non-`head` classes entirely. Reset to base on the first success. A node that has been
down for an hour costs 30 requests/hour, not 240, and the operator still sees a fresh "last checked"
timestamp.

### 9.3 Slowness backoff

If `latency_max_ms` over the last 5 minutes exceeds 1 000 ms, double the interval (capped at 4×) and
record `SamplingThrottled` once per transition, with the reason shown on the node page. A node under
load must not be pushed further by its own manager. Restore the base interval after 10 minutes below
500 ms.

### 9.4 Response-size budget

`client.rs` reads at most 2 MiB per response. Exceeding it aborts the read, stores
`pool_ok = 0, error_detail = "mempool response exceeded 2 MiB"`, and triples that class's interval
for that node. `getrawmempool` on a congested N3 mainnet node is the only realistic offender, and it
degrades to "depth unknown" rather than dragging the workbench's memory with it.

### 9.5 `rpc_port == 0`

The node's `observability` is `process-only`. Consequences, all explicit:
- no sampling jobs are scheduled (zero RPC cost);
- health state is `Unknown` with reason "RPC is disabled on this node (`rpc_port = 0`); chain state
  cannot be observed" and `next_action` linking to `/nodes/{id}/edit`;
- Prometheus emits `neonexus_node_running` and `neonexus_node_observable 0`, and **no chain series**;
- every chain alarm for that node is `NoData(SamplingDisabled)`;
- the fleet list shows a distinct "not observable" badge, not a green one.

This replaces `src/web/pages/nodes/detail_tabs.rs:273`, where `node.rpc_port == 0` currently grants
`🟢 2/2 System & Instance Checks Passed` (G6).

### 9.6 Degrade, never fail

- A failed class never voids the round. A round with `head_ok = 1, pool_ok = 0` is a good round.
- An unsupported method is `NULL` + a capability row, rendered "not supported by this client".
- A missing reference is `head_lag = NULL`, rendered "no reference head", and the state machine
  falls back to the two local staleness witnesses.
- A database write failure logs and drops the sample; it never blocks the tick or the state machine's
  in-memory counters (which are re-derivable from the next successful write).
- **Downstream consumers must handle `None`.** The watchdog integration (G33) reads
  `HealthState::Stalled` / `Unreachable` sustained for a configurable duration as a restart trigger,
  **per node**, **opt-in**, defaulting off — `Unknown` and `NoData` never trigger a restart. An
  observation layer that can restart nodes on the strength of data it does not have is worse than no
  observation layer.

---

## 10. Headless surface

`src/cli/` gains, all with `--json`:

- `--observe <node-id>` — the current sample, derivations, state, duty states, reference quality.
- `--observe-fleet` — one row per node; the JSON is the same shape the web fleet page renders, so
  drift between the two is a diff, not a discovery.
- `--health-history <node-id> [--since <duration>]` — transitions plus rollups.
- `--alarms [--state alarm|pending|no-data]` — with a non-zero exit code when any alarm is firing, so
  it composes into CI and cron.
- `--chain-state <node-id>` — replaces `--peer-health <rpc-endpoint>` and friends
  (`src/cli/actions/chain.rs:102`): keyed by **node id**, resolving the endpoint through
  `node_rpc_endpoint` and the family through `node_type.family()`, both of which already exist
  (G12). The endpoint form stays as `--chain-state-endpoint <url> <family>` for ad-hoc use.

---

## 11. Landing sequence

Each stage compiles, passes, and leaves the product working.

**Stage 1 — sample and store.** `observe/` with `head`, `peers`, `identity` classes; scheduler;
`metric_samples`; `node_health_state` with only `{Stopped, Starting, Unreachable, Unknown, Healthy}`
(no lag-dependent states yet); `--observe`. `rpc_health_checks` keeps being written in parallel for
one release so nothing regresses. Node detail replaces the invented `1.2% / 64.5 MB / 3.2 ms` block
(G2) with real latency, peers, height and last-change — and "not measured" where nothing is measured.

**Stage 2 — derive and classify.** `head_time` class, `derive.rs`, `reference.rs`, the full
nine-state machine, `node_health_transitions`, rollups and retention. The Health page's fake 60-minute
SVG path (G3) is replaced by a render of `metric_rollups(60)`, and the time-range pills become real
query parameters. `syncing_nodes` (G17) starts counting `HealthState::Syncing`. Delete
`rpc_health_checks` writes and the log-scraping sync parsers.

**Stage 3 — alarms.** `alarm_rules`/`alarm_states`/`alarm_transitions`, seeded built-ins, the
`NoData` renderer, scope-aware routing. The Alerts page binds to rows instead of returning a literal
`vec![]` (G1).

**Stage 4 — governance and duty.** `governance`, `governance_deep`, `designation` classes; duty state
axis; the revocation and validator-set-exit transitions. `chain_state/` gets its second caller and
the CLI commands key by node id (G12, G13).

**Stage 5 — exposition and consumers.** Prometheus families; the health-aware watchdog trigger
behind a per-node opt-in (G33); `neonexus_node_*` reconciled with `docs/AGENT_API.md`.

---

## 12. Test obligations

The register's most useful warning is that the existing sync parsers are covered by a test whose
fixtures were written to match the parser (G11, `tests/unit/supervision/tests.rs:167-206`), which is
why CI is blind to their being wrong. Counter-measures:

1. **State-machine tests are vectors, not recordings.** `health::evaluate` takes a `HealthInputs`
   struct; tests construct inputs directly and assert the verdict. Required cases: answers-but-frozen
   (the `Stalled` case), frozen-chain vs frozen-node, zero peers with a moving height, a node ahead of
   the reference, clock skew, height regression, fresh node inside `starting_grace`, `rpc_port == 0`,
   sample older than `stale_sample_seconds`, and a snapshot-suppressed node.
2. **Parser fixtures come from the clients, not from us.** JSON-RPC response fixtures are captured
   verbatim from neo-cli, neo-go, neo-rs, geth and reth (checked in under `tests/fixtures/rpc/`),
   with the source client and version recorded in the filename. A parser test that cannot point at a
   real response does not count.
3. **Unit normalisation.** An explicit test that an N3 `getblockheader.time` of `1_700_000_000_000`
   and a Neo X `timestamp` of `0x6553f100` both land in the same `head_block_time_unix` seconds
   domain.
4. **No-data rendering.** A seeded rule against a node with zero samples renders "No data" — asserted
   on the HTML, not on the model.
5. **Retention arithmetic.** Insert 48 h of synthetic samples, run the rollup and prune jobs, assert
   the exact surviving row counts per tier and that no bucket is double-counted.
6. **Prometheus absence.** A node with no samples emits `neonexus_node_running` and
   `neonexus_node_observable`, and zero chain series — asserted by absence, not by value.

---

### NeoNexus Target Domain Model: Hosts, Networks, Observations, and History
Replaces the single `nodes` table plus node-id side tables with a fleet model: Host and Network become first-class entities (Network carries magic/chain-id/seeds/committee/genesis and is the single source every config render reads, killing the `profile: None` fallback that ships unbootable configs); observation becomes a `node_samples` time series with explicit NULL-means-not-measured semantics feeding real alarm rules with an `insufficient-data` state; and every spec change, designation, governance shift and upgrade attempt gets an append-only temporal record with an actor. Includes the full SQL for 28 new tables and 7 changed ones with constraints, triggers and indexes, and a 17-step migration where every value is marked either backfilled from evidence or defaulted and flagged — never invented.

# NeoNexus Target Domain Model

## 0. The rules this model obeys

Three rules, derived from the register's root causes, constrain every decision below. They are stated first because they explain choices that would otherwise look like over-engineering.

**Rule 1 — No stored value that is not a measurement or an operator's statement.** R1 put green literals where state belonged. The storage layer answer is that *absence is a value*: `NULL` in an observation column means "not measured", never `0`. `node_samples.peer_count IS NULL` and `node_samples.peer_count = 0` are different facts and must render differently ("not measured" vs "isolated"). Where a status can be derived, it is a SQLite **generated column**, so it cannot disagree with the data it claims to summarise (`networks.complete`).

**Rule 2 — Identity is persisted once and read from one place.** R1/R3 let chain identity be recomputed at three call sites from a constant with `profile: None`. The model makes chain identity a row, and config generation takes a non-optional reference to it. Divergence between Start and the launch-pack exporter becomes impossible by construction, and `node_config_renders` records the SHA-256 so a test can assert it.

**Rule 3 — Safety invariants live in the database.** The codebase already set this precedent at `src/repository/schema/migrations.rs:66` (`enforce_signer_lease_exclusivity`), with the correct reasoning: a workspace can be edited by hand, restored from an older release, or written by a future caller that forgets to ask. Port collision, double signing, unnamed history, and a node on an unidentified network are all in that class. They get indexes, `CHECK` constraints, and triggers — not just Rust checks.

Target SQLite floor is **3.38** (generated columns 3.31, `STRICT` 3.37, JSON1 built in from 3.38). `rusqlite = "0.37"` with `bundled` ships 3.50.x, so this is satisfied without a runtime probe. All new tables are `STRICT`; existing tables become `STRICT` only where they are already being rebuilt.

---

## 1. Entities

### 1.1 What becomes first-class

| Entity | Today | Why it must be a row | Register |
|---|---|---|---|
| **Host** | `http://127.0.0.1:{rpc_port}` interpolated at `src/rpc_health/probe/endpoint.rs:4`; `Command::new` at `supervisor/process/spawn.rs:28` | A fleet is by definition multi-host. Every endpoint, every port reservation, every runtime installation and every disk path is *relative to a machine*; with no Host they are all relative to an assumption. | G34, G17 |
| **Network** | `enum Network { Mainnet, Testnet, Private }` (`src/types/network.rs`) + compile-time constant tables | A `Private` variant carries no identity. Three modules independently re-derive magic/chain-id from constants with `profile: None`, so a private node renders with no seeds and no committee and readiness says "ready". See §2. | G18, G19, G22, G23 |
| **Node** | `nodes` (13 columns) | Gains host, network, timestamps, data dir, config mode, metrics port, owner, environment, revision. | G34, G35, G38, G39, G40 |
| **Duty** | `node_roles` — exactly one row per node | `plugin_states` is already many-to-many and is what drives neo-cli's emitted sidecars; the real single-duty limit is neo-go's exclusive `match role` over four signing services. A set with one *primary* and a DB-enforced "at most one signing duty" expresses the actual constraint instead of flattening it. | G9, G20, G21, G39 |
| **SignerBackend** | `SignerRegistry::from_process_environment()` at `web/state.rs:217`; no insert, no reload | A backend you can only create by restarting the process with different environment variables is not a managed object. `binding.rs:182` tells the operator to "configure the signer registry" with no link and no form because there is nowhere to write to. | G28 |
| **SignerKey** | A free-text `key_id` string on the binding | The key id is demanded as free text and appears nowhere in the keys table; a typo is caught on first dispatch, i.e. when the validator fails to start. Making it a row turns the field into a picker and gives `curve` a home — which is exactly what makes "a Neo N3 NEP-6 profile cannot serve a Neo X node" a data fact rather than a runtime string comparison. | G28, G21, G44 |
| **Binding** | `node_signer_bindings` | Keeps its name and its exclusivity index; gains a composite FK to `signer_keys`, `bound_at_unix` and `bound_by`. | G44 |
| **RuntimeRelease** | A parsed catalog entry that exists only inside one function call | The install chain is circular today: the upgrade policy needs a `catalog_profile_id` obtainable only from a backup of a workspace that could never have created one. Persisting releases (and seeding a default catalog profile) breaks it and gives the upgrader something to compare against. | G25, G31 |
| **RuntimeInstallation** | `runtime_installations`, PK `package_id` | Installations are per machine. PK becomes `(host_id, package_id)`. | G25, G34 |
| **Snapshot / SnapshotApplication** | `fast_sync_snapshots`; applications unrecorded | `network` as an enum string means every private network shares one snapshot namespace. Application history is what makes "we fast-synced this node on Tuesday and it has been wrong since" answerable. | G40, G45 |
| **AlertRoute** | Five `alert_routing.*` settings keys — one provider, one URL | "Critical to PagerDuty, Warning to Slack" is unexpressible with a singleton. | G14 |
| **AlarmRule / AlarmState / AlarmTransition** | Four hardcoded table rows that permanently read `● OK`; `fn active_alarms_table() -> String` takes no arguments | The names in the fake table (`ChainHeadDelta`, `ConnectedPeers`, `SignerLeaseTTL`) become real rules with real thresholds and a real scope. `AlarmState` has an explicit `insufficient-data` third value — the direct answer to "a node that has never been probed is indistinguishable from a passing one". | G1, G2, G6, G14 |
| **NodeSample** | `rpc_health_checks`: two RPC calls, status = how many answered, 24 rows kept, no latency, no peers | The whole of R2. See §5. | G11, G12, G15, G16, G17, G3 |
| **NodeSampleRollup** | Nothing; the 60-minute chart is a literal SVG path | A 1h/1d/1w selector needs buckets, not a `MetricsCollector::new(Duration::ZERO)` per page. | G3, G4 |
| **NetworkHead** | Nothing | Head-lag is meaningless without a reference height, and the *provenance* of that reference is itself operator-critical: "highest node in your own fleet" and "a public RPC" are different claims. | G11 |
| **NodeDesignation** | `designation_status` is CLI-only, asks only at current height, persists nothing | "Your Oracle designation was revoked at 03:14" is the case where a node keeps running and stops doing its job. It is only answerable from an append-on-change history. | G13 |
| **NetworkGovernanceSample** | `governance_snapshot` is CLI-only | A committee member learns they were voted out by watching the chain. Append-on-change keyed by a committee hash. | G12, G13 |
| **Environment / NodeTag** | `Environment / Production` hardcoded for every node including testnet | Alarm scoping, filtering and bulk actions need a real axis. `NodeInventoryFilter` is `{status, query}`; type and network are not even filterable. | G7, G39 |
| **NodeRevision / NetworkRevision** | Nothing; `update_node` is a plain UPDATE | "What version was this on last Tuesday", "what did we roll back from", "who changed this". Also the rollback handle the upgrader lacks. | G35, G31 |
| **NodeConfigRender** | Nothing; "● In Sync" comes from `Path::is_file()` | Records the SHA-256 of what was written, for which node revision and which network revision, for which purpose. Makes drift real and makes Start↔launch-pack parity a testable assertion. | G5, G18, G32 |
| **RuntimeUpgradeRun / Attempt** | Two fleet-level events with `node_id: None` and seven `warn!` calls | "batch completed: 0/3 successful" with no reason anywhere in the browser. | G31 |
| **HostProbe** | `remote_server_probe_records` | Federation becomes one host transport rather than a parallel universe. `syncing_nodes` is deliberately **not** carried across. | G17, G29, G34 |
| **NodeSupervisionOverride** | One workspace watchdog policy read for every node | Stopping one crash loop must not mean disabling automatic restart fleet-wide. `paused_until_unix` is "stop relaunching the node I am editing". | G33 |
| **NodeStaticPeer** | Hardcoded empty `static_nodes`/`trusted_nodes`; reachable only through the free-text "Extra arguments" box | | G23 |
| **Event** (changed) | `runtime_events`; actor inferred by `event.message.contains("Hermes")` | Gains `actor_kind`/`actor_id`, `host_id`, `network_id`, `correlation_id`, `details`. Existing rows backfill to `actor_kind = 'unknown'` — never to a guess. | G8, G36, G46 |

### 1.2 What is deliberately not modelled

Naming these matters as much as the entity list, because the absence of a decision is what let the costume grow.

- **Instance type / flavor / vCPU / RAM** (`t3.{node_type}-{role_slug}`, "⚡ Flavor: 8 vCPU · 32 GB RAM"). Nothing in NeoNexus allocates CPU, RAM or IO. Deleted, not modelled. The real per-process CPU/RSS already exist in `MetricsSnapshot::node_process` and become `node_samples.process_cpu_percent` / `process_memory_bytes`.
- **Volume / IOPS / device / "Attached"** (`vol-…`, `/dev/xvda (Root)`, `3000 IOPS (gp3)`). Deleted. What is real is a **path** and its **free space**: `nodes.data_dir` and `node_samples.disk_free_bytes`.
- **Security group / firewall rules / CIDR / "Rule Status: Open"**. NeoNexus has no firewall capability (`iptables|pfctl|ufw|nftables` appears only in UI captions). The real facts are the bind addresses NeoNexus itself generated, which are a *function of the rendered config* — so they are read from `node_config_renders` and the network profile, not stored as assertions.
- **Availability zone / VPC / region / account** (`nexus-az-1a`, `vpc-{network}`, `Account: 0123-4567-8901`). Deleted. `hosts.label` and `hosts.address` are the truth.
- **Duty *support* matrix.** `role_availability` is a hand-maintained table consulted by nothing on the launch path, which is why Neo X Consensus is offered and structurally impossible to launch. Support is a pure function of `(node_type, duty, signer_backend_kind, key.curve)` computed from the launch path, not a stored table. The picker is generated from it.
- **A `PrivateNetworkPlan` entity.** A plan is not a durable thing; the durable thing is a **Network** row plus the **Node** rows planned onto it. `PrivateNetworkPlanner::plan` materialises both in one transaction. The on-disk `DeploymentManifest` becomes an *export artefact* of that pair, recorded in `node_config_renders` with `purpose = 'launch-pack'`.
- **`ChainFamily` on `nodes`.** It is derived from `node_type` and the existing doc comment is right. It *is* stored on `networks`, because a network belongs to exactly one family and must be authorable before any node exists on it.

---

## 2. The Network entity

### 2.1 The defect, precisely

`node_lifecycle.rs:170-175` renders every launch with `profile: None`. The fallbacks are `Network::Private => Vec::new()` for seeds (`config/format/network.rs:79`) and committee (`committee.rs:81`), magic `1_230_000` (`network.rs:13`). Validation cannot catch it: `neo_cli.rs:39` gates committee checks on `profile.is_some()` and never checks `SeedList`; `neo_go.rs:46` and `neo_rs/p2p.rs:56` check `len >= 0`. neo-cli is worst because it *omits* the keys, so it falls back to compiled-in public defaults and dials mainnet seeds while carrying magic 1230000. Readiness reports "ready, 10 pass, 0 critical".

Three things cause this and all three are model defects:

1. Chain identity has nowhere to live, so `RuntimeConfigProfile` is constructed at two unreachable call sites and is `Option` everywhere else.
2. `Option<&RuntimeConfigProfile>` makes "no identity" representable, and every `effective_*` helper exists solely to paper over it.
3. "Complete enough to boot" is a judgement made by validators that receive the already-degraded value, instead of a property of the identity itself.

### 2.2 What the Network holds

One row per chain instance. Family-partitioned columns, because Neo N3 and Neo X share almost nothing: N3 has a 4-byte magic, a `host:port` seed list and a standby committee of secp256r1 keys; Neo X has an EIP-155 chain id, `enode://` bootnodes and a genesis whose hash is the only proof a node joined the right chain.

- **Identity (N3):** `network_magic`, `validators_count`, `committee_public_keys` (JSON array), `seed_nodes` (JSON array), `milliseconds_per_block`, `max_transactions_per_block`.
- **Identity (Neo X):** `chain_id`, `bootnodes` (JSON array of enode URIs), `genesis_hash`, `genesis_path`, `reth_chain_preset`, `block_period_secs`.
- **Provenance:** `origin ∈ {seeded, authored, planned, imported}`, `locked`, `revision`, `created_by`.
- **Observation anchor:** `reference_endpoints` (JSON array) — public RPC endpoints used to establish the authoritative head for head-lag. Empty is legal; then `head_lag_blocks` is `NULL` and the UI says "no reference head configured", which is honest.
- **Derived:** `complete` — a **generated column**, not a stored flag, so no code path can mark a network complete that is not.

`complete` for N3 requires a magic, a validators count, at least one seed, and `committee_public_keys` at least as long as `validators_count` — which is exactly neo-go's own startup requirement ("configuration should include StandbyCommittee", and it refuses again if the committee is shorter than `ValidatorsCount`). For Neo X, public networks are complete by their compiled-in identity; a private one needs a genesis path and at least one bootnode, because NeoNexus explicitly refuses to invent a Neo X genesis allocation.

### 2.3 How networks are created

**Public networks are seeded, not fallen back to.** The four rows (`neo-n3-mainnet`, `neo-n3-testnet`, `neox-mainnet`, `neox-testnet`) are inserted at schema initialisation from the constants that exist today (`config/format/committee.rs`, `network.rs`, `neox.rs`). Those constants move into `src/config/format/seeds.rs` and become reachable **only from the seeder**. `seed_nodes(network)`, `standby_committee(network)`, `network_magic(network)`, `validators_count(network)` and every `effective_*` helper are deleted.

Seeded rows carry `locked = 1` and a `seed_hash` over their identity columns. On each open, the seeder upserts a locked row whose `seed_hash` differs from the shipped constant — so a transcription fix in a release reaches existing workspaces — and never touches an unlocked row. An operator who needs a variant clones a locked row into an authored one.

**Private networks are authored.** Two paths, one destination:

- `POST /networks` — a form with magic/chain-id, validators count, committee keys, seeds, block period. It is the first screen a private network goes through and it cannot be saved incomplete except as an explicit draft (`complete = 0`, which blocks Start).
- `PrivateNetworkPlanner::plan` → in one transaction: insert the `networks` row (`origin = 'planned'`), insert the node rows, derive `seed_nodes` as `{host.address}:{p2p_port}` for each consensus member (today it is hardcoded `127.0.0.1:{p2p_port}` at `private_network/manifest/network.rs:29`, which is wrong the moment a second host exists), and write `committee_public_keys` from the generated roster. This gives `PrivateNetworkMaterialized` and `PrivateNetworkLaunchPackExported` their first construction sites.

Magic and chain id are **unique within a family** (`idx_networks_n3_magic`, `idx_networks_neox_chain_id`). Two private networks sharing magic `1230000` — the current universal default — become impossible.

### 2.4 How config generation reads it

`RuntimeConfigProfile` and the magic-override token machinery collapse into one type loaded from the row:

```rust
pub struct NetworkProfile {
    pub id: String,
    pub label: String,
    pub family: ChainFamily,
    pub kind: NetworkKind,          // Mainnet | Testnet | Private
    pub revision: u32,
    pub complete: bool,
    pub n3: Option<NeoN3Identity>,   // magic, seeds, committee, validators_count, ms_per_block, max_tx
    pub neox: Option<NeoXIdentity>,  // chain_id, bootnodes, genesis_hash, genesis_path, reth_preset, period
}
```

Signature changes, all removing an `Option`:

```rust
// before: ConfigGenerator::render_for_node(node, plugins, profile: Option<&RuntimeConfigProfile>, ctx)
// after:
ConfigGenerator::render_for_node(node: &NodeConfig,
                                 network: &NetworkProfile,
                                 plugins: &[PluginState],
                                 ctx: &GenerationContext) -> RenderedConfig
```

There is exactly one loader, `fn network_profile(repo: &Repository, node: &NodeConfig) -> Result<NetworkProfile>`, and exactly these callers: the launch path (`node_lifecycle.rs:170`), `ConfigExporter`, the launch-pack exporter (`private_network/exporter/writer/nodes.rs`), `ConfigDriftDetector`, `ConfigValidator`, the diagnostics readiness readout, and the Prometheus chain labels. Same node revision + same network revision ⇒ same bytes, everywhere.

`neox_chain_id(network, profile)` is deleted; `profile.neox.chain_id` is the only source, which removes the three-way re-derivation. `has_chain_argument`, which today tests only for a flag's *presence*, becomes flag-**value** extraction compared against `profile.neox.chain_id`, so a report can no longer simultaneously acknowledge `--networkid 12345` and print "chain id 1230000".

Two gates follow directly:

- **Start refuses on `complete = 0`**, with the reason and a link to the network editor. This is a behaviour change and it is the correct one: today a private neo-cli node starts, omits `SeedList`, and dials mainnet seeds while carrying a private magic. Neither the old behaviour nor "render `SeedList: []`" is acceptable, so the node does not start until the operator supplies an identity.
- **`node_config_renders` records the SHA-256** of the primary config and each sidecar, tagged with `purpose`. A test asserts that for a given `(node_revision, network_revision)`, the `launch` digest equals the `launch-pack` digest. G18's "byte-identical" requirement stops being a hope.

---

## 3. Multi-host

### 3.1 The Host abstraction

`hosts` is "a machine NeoNexus can reach", with a `transport` that says *how*:

| transport | supervises processes | control | today |
|---|---|---|---|
| `local-process` | yes | `Command::new` | the only one implemented |
| `ssh` | yes | `control_endpoint = user@host:port`, `credential_ref` names a secret, never holds one | designed for, not built |
| `neonexus-peer` | **no** | `control_endpoint = https://peer/`; read-only aggregate + (later) per-node mirror | this is today's `remote_servers` |

Exactly one `local-process` host may exist, and its id is `local` — enforced by `CHECK (transport <> 'local-process' OR (id = 'local' AND supervises_processes = 1))`. One NeoNexus process supervises one machine; a second local host would be a lie about where `Command::new` runs.

`hosts.workspace_root` is the path on that machine, so `node_workspace_path` stops implicitly meaning "this machine's workspace".

### 3.2 The seeded `local` host

Inserted by migration 002 before `nodes` is rebuilt:

```sql
INSERT OR IGNORE INTO hosts (id, label, transport, address, service_scheme,
                             supervises_processes, enabled, workspace_root,
                             created_at_unix, updated_at_unix)
VALUES ('local', 'This machine', 'local-process', '127.0.0.1', 'http',
        1, 1, :workspace_root, :now, :now);
```

Every existing node gets `host_id = 'local'`. This is a **default, not a backfill**, and it is sound: the only spawn in the codebase is local `Command::new` and the only endpoint is `127.0.0.1`, so "local" is not an assumption about the data, it is a restatement of what the code could physically have produced.

### 3.3 Endpoint derivation

One function replaces `format!("http://127.0.0.1:{}", node.rpc_port)`:

```rust
pub fn node_service_endpoint(host: &Host, node: &NodeConfig, port: ServicePort) -> String {
    let p = match port {
        ServicePort::Rpc => node.rpc_port,
        ServicePort::Ws => node.ws_port?,            // None ⇒ no endpoint, not a fabricated one
        ServicePort::Metrics => node.metrics_port?,
        ServicePort::P2p => node.p2p_port,
    };
    format!("{}://{}:{}", host.service_scheme, host.address, p)
}
```

Consequences:

- `rpc_health` probes remote nodes with no change beyond taking a host.
- The `chain_state` CLI stops taking a hand-typed endpoint. `--peer-health <rpc-endpoint> [neo-n3|neo-x]` becomes `--peer-health <node-id>`; family comes from `node_type.family()` and the endpoint from the host. Peer and mempool telemetry then poll on the RPC-health schedule and persist into `node_samples`, which is what makes them surfaceable at all.
- The Neo X metrics adapter stops hardcoding `http://localhost:8546/metrics` (geth's WebSocket default, which the repo itself flags) and `:9091`, and stops discarding `_rpc_port`. `nodes.metrics_port` is the field it needed; the launch path emits `--metrics`/`--metrics.port` from it. That in turn supplies the block height and peer count Theme 2 is missing from a second, independent source.

### 3.4 Ports become per-host, and collision becomes impossible

Today `reserved_node_ports` scans every node workspace-wide, which is both too strict (two hosts cannot reuse 10332) and too loose (it never notices node A's RPC port equalling node B's P2P port, and nothing enforces it at rest).

`host_port_reservations(host_id, port)` with a composite primary key, maintained by triggers on `nodes`, makes every collision a constraint violation with a readable message. The port planner reads this table instead of scanning nodes, and `ws_port` finally means something: a WS port that is reserved and surfaced but never opened (no `--ws*` flag exists anywhere in `src/`) is a reservation for a service that does not run — with the table in place, the launch path can assert that a reserved `ws` port implies an emitted WS flag, or refuse the reservation.

### 3.5 Relationship to `federation`

Federation stops being a parallel universe and becomes a host transport.

- `remote_servers` rows migrate into `hosts` with `transport = 'neonexus-peer'`, `supervises_processes = 0`, `control_endpoint = base_url`, `address` = the host component of that URL. The `UNIQUE(base_url)` constraint survives as `UNIQUE(control_endpoint) WHERE control_endpoint IS NOT NULL`.
- `remote_server_probe_records` migrates into `host_probes`. **`syncing_nodes` is not carried across**: it counted processes inside a ~600 ms launch window under a column literally headed "Syncing". The column returns only when it is fed by a peer's real `node_samples.syncing`.
- The Federation page becomes the Hosts page filtered to `transport = 'neonexus-peer'`. `create_remote_server` / `update_remote_server` / `delete_remote_server` — today called only from tests, with the page telling the operator to write Rust — get routes because they are now the generic host forms.
- **Mirrored nodes** are the join federation lacks. `nodes.origin = 'mirrored'` marks a node discovered from a peer: it belongs to that peer's host, has no local process, refuses Start, takes no port reservations, and receives `node_samples` rows written from the peer's public API. Federation's Blocks/Peers columns, permanently `—` today, then carry real numbers. This is the last stage and depends on the peer exposing per-node state; the model supports it now so the schema does not have to change again.

---

## 4. Grouping and scoping

Four axes, each chosen because an alarm, a filter or a bulk action needs it.

1. **Environment** — `environments` is a seeded lookup (`production`/`staging`/`development`/`lab`) with a `rank` and a `default_alarm_severity`, and `nodes.environment_id` is a nullable FK. It is a table rather than a free-text column because alarm severity defaults and sort order are properties of the environment, not of each node. Crucially, **migration assigns no node an environment**: the hardcoded `Environment / Production` (applied today to every node including testnet) is deleted, not migrated.
2. **Tags** — `node_tags(node_id, key, value)`, arbitrary, with `idx_node_tags_lookup(key, value, node_id)` so `tag:team=core` is an indexed selector.
3. **Owner** — `nodes.owner`, free text, indexed. Deliberately not an `operators` table: the console has no authentication, and a foreign key to a user entity that cannot be authenticated would imply an access control that does not exist.
4. **Structural axes already present but unfilterable** — `node_type`, `network_id`, `host_id`, `duty`, `chain_family`. `NodeInventoryFilter` grows from `{status, query}` to a struct over all of them, and the role filter stops offering 5 of 8 duties (which today makes State/StateValidator/Notary nodes vanish from every filtered view).

**Alarm scoping** is `(selector_kind, selector_value)` over exactly this set: `all | node | host | network | environment | duty | tag | node-type | chain-family`. "Page on the validator, warn on observers" becomes two rules with `selector_kind = 'duty'`. Bulk actions take the same selector, so the filter the operator applied is the selection they act on.

---

## 5. History and observation

### 5.1 `node_samples` — the observation layer

One row per probe tick per node. The column set is chosen so that every fabricated value in Theme 1 has a real counterpart, and every question in Theme 2 is a query:

| Column | Replaces | Enables |
|---|---|---|
| `rpc_latency_ms` | `"3.2 ms"` literal under the caption "Loopback probe latency" | The first real latency measurement in the product |
| `block_height`, `header_height`, `best_block_hash` | `block_count` only | `getblockheadercount` / `eth_syncing.highestBlock` give in-node sync lag without any reference |
| `reference_height`, `head_lag_blocks` | nothing | Lag against `network_heads` |
| `height_delta`, `height_unchanged_secs` | nothing | **"Running but not syncing"** — the classic failure, currently undetectable |
| `syncing` | log-scraping with parsers that gate on strings the clients never emit | `eth_syncing` directly, instead of matching `"Chain imported"` against geth's actual `Imported new chain segment blocks=` |
| `peer_count`, `peer_unconnected_count`, `peer_bad_count` | `classify_connectivity` computed and discarded in a CLI | Peer alarms, the `Isolated` state on the node page |
| `mempool_count`, `mempool_verified_count` | `classify_congestion` likewise | Congestion, with the threshold read from `networks.max_transactions_per_block` instead of a compile-time 500 — a 500-tx mempool is not "Elevated" on a chain configured for 5,000-tx blocks |
| `process_cpu_percent`, `process_memory_bytes` | `"1.2% (Active)"`, `"64.5 MB"` | The real values already exist in `MetricsSnapshot::node_process`; persisting them makes the High-CPU filter and the CPU sort work |
| `disk_free_bytes` | `"3000 IOPS (gp3)"`, `"Attached"` | The only storage fact that matters to an on-call operator |
| `outcome`, `error_kind`, `error_message` | `status` = how many of two calls answered | `ok / partial / unreachable / rejected` distinguishes a transport failure from a node that answered with an error |

The rule that makes this worth building: **`NULL` is "not measured".** A neox-reth node with no metrics port gets `NULL` CPU, not `0.0`. `RpcHealthStatus` stops being "how many of two calls answered" and is derived from the sample plus the alarm states that reference it.

**Retention and rollups.** Raw samples 24 h; `node_sample_rollups` at 60 s for 7 days, 300 s for 30 days, 3600 s for 400 days. Retention windows live in `workspace_settings` under `observation.retention.*`. The 60-minute chart reads the 60 s bucket; the 1h/3h/1d/1w pills — today bare `<span>`s with a hover restyle and no handler — select a bucket and a window.

**`network_heads` provenance is a first-class column.** `source ∈ {fleet-max, reference-endpoint}`. `fleet-max` is the max height across healthy nodes on that network and is a *weak* reference: if all your nodes are stalled together, lag reads zero. `reference-endpoint` polls `networks.reference_endpoints` and is authoritative. The UI must name which one it used, and `head_lag_blocks` stays `NULL` when neither is available.

### 5.2 Temporal records and the questions they answer

| Table | Question |
|---|---|
| `node_revisions` | *"What version was this on last Tuesday?"* — latest revision with `changed_at_unix <= t`. *"What did we roll back from?"* — revision `n-1`. *"Who changed this?"* — `actor_kind`/`actor_id`. Also the upgrader's missing rollback target. |
| `network_revisions` | *"Did someone change the committee, and is every rendered config now stale?"* A private network's identity changing invalidates every node's config; `node_config_renders.network_revision` says exactly which. |
| `node_samples` + rollups | *"When did it stop syncing?"* — first sample where `height_unchanged_secs` crossed the threshold. *"Was it slow before it died?"* |
| `node_designations` | *"When was the Oracle designation revoked?"* Append-on-change: a row is written only when the answer differs from the last one, so the table *is* the timeline. `designated` is nullable — `NULL` means "no key to compare against", which is a different fact from "not designated", and the CLI currently collapses them into the same exit code 1. |
| `network_governance_samples` | *"When were we voted off the committee?"* Append-on-change keyed by `committee_hash`. |
| `alarm_transitions` | *"How long was it in alarm, and what was the value when it flipped?"* The alarm history CloudWatch chrome pretends to have. |
| `runtime_upgrade_runs` / `_attempts` | *"Why did 0 of 3 succeed?"* — `outcome`, `stage`, `message` per node, with `from_version`→`to_version` as columns instead of a `tracing` macro. |
| `snapshot_applications` | *"Did the fast-sync that ran on Tuesday put data in the right directory?"* — `target_dir` recorded, which is how the "snapshot apply ignores a custom datadir" bug becomes visible. |
| `node_config_renders` | *"What exactly was written to disk, when, and does it still match?"* Turns drift from `Path::is_file()` into a digest comparison, and makes the config text renderable inline. |
| `host_probes` | *"When did this peer last answer?"* |
| `runtime_events` (+ actor) | *"Who did this?"* — really, not by grepping the message for `"Hermes"`. |

### 5.3 The actor

`actor_kind ∈ { operator, cli, api-token, agent, watchdog, supervisor, system, unknown }`, with `actor_id` scoped to the kind: the API token id for `api-token`, the agent id for `agent`, the OS user for `cli`, the configured operator name (a workspace setting) for `operator`. `unknown` is a real value and it is what every pre-migration row gets. The current heuristic — `if event.message.contains("Hermes") || event.message.contains("probe")` → `arn:neo:agent::hermes-ai`, else `arn:neo:iam::nexus:operator`, under a column headed "User Identity" — is deleted rather than approximated.

Attribution is enforced structurally rather than by convention. The repository inserts the `node_revisions` row **before** the `nodes` UPDATE, in the same transaction, supplying the actor from Rust; a `BEFORE UPDATE` trigger aborts any spec-field update that does not bump `revision`, and an `AFTER UPDATE OF revision` trigger aborts if no matching revision row exists. History therefore cannot be skipped by a future caller that forgets. `status` and `pid` are explicitly **not** spec fields: runtime state does not create history (it creates events and samples), so `transition_node_status` is unaffected.

The "CloudTrail-grade immutable audit journal" claim over a plain table with no hash chain and a live arbitrary-insert path via backup import is deleted. If immutability is later wanted, `runtime_events` gains a `prev_hash`/`entry_hash` chain and restore writes to a separate `imported_events` table — noted as a follow-on, not claimed now.

---

## 6. Full SQL

Ordered as migrations. `:now` is the migration timestamp. All new tables are `STRICT`.

### 000 — migration bookkeeping

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version         INTEGER NOT NULL PRIMARY KEY,
    name            TEXT    NOT NULL,
    applied_at_unix INTEGER NOT NULL,
    app_version     TEXT    NOT NULL DEFAULT ''
) STRICT;
```

The existing idempotent `add_column_if_missing` steps are recorded as version 1 (`baseline`) when `nodes` already exists, so no legacy workspace re-runs them.

### 002 — hosts

```sql
CREATE TABLE IF NOT EXISTS hosts (
    id                   TEXT    NOT NULL PRIMARY KEY,
    label                TEXT    NOT NULL,
    transport            TEXT    NOT NULL,
    address              TEXT    NOT NULL DEFAULT '127.0.0.1',
    service_scheme       TEXT    NOT NULL DEFAULT 'http',
    control_endpoint     TEXT,
    credential_ref       TEXT,
    workspace_root       TEXT,
    supervises_processes INTEGER NOT NULL DEFAULT 1,
    enabled              INTEGER NOT NULL DEFAULT 1,
    description          TEXT    NOT NULL DEFAULT '',
    os                   TEXT,
    arch                 TEXT,
    agent_version        TEXT,
    last_seen_at_unix    INTEGER,
    created_at_unix      INTEGER NOT NULL,
    updated_at_unix      INTEGER NOT NULL,

    CHECK (transport IN ('local-process','ssh','neonexus-peer')),
    CHECK (service_scheme IN ('http','https')),
    CHECK (supervises_processes IN (0,1)),
    CHECK (enabled IN (0,1)),
    CHECK (length(address) BETWEEN 1 AND 255),
    -- One process supervises one machine. A second local host would assert a
    -- place where Command::new does not run.
    CHECK (transport <> 'local-process' OR (id = 'local' AND supervises_processes = 1)),
    -- A peer is observed, never supervised: NeoNexus has no way to spawn there.
    CHECK (transport <> 'neonexus-peer' OR (control_endpoint IS NOT NULL AND supervises_processes = 0)),
    CHECK (transport <> 'ssh' OR control_endpoint IS NOT NULL)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_hosts_control_endpoint
    ON hosts (control_endpoint) WHERE control_endpoint IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_hosts_label
    ON hosts (label COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_hosts_enabled
    ON hosts (enabled DESC, label COLLATE NOCASE ASC);

INSERT OR IGNORE INTO hosts
    (id, label, transport, address, service_scheme, supervises_processes,
     enabled, workspace_root, created_at_unix, updated_at_unix)
VALUES ('local','This machine','local-process','127.0.0.1','http',1,1,
        :workspace_root, :now, :now);
```

```sql
CREATE TABLE IF NOT EXISTS host_port_reservations (
    host_id TEXT    NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    port    INTEGER NOT NULL,
    node_id TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    purpose TEXT    NOT NULL,
    PRIMARY KEY (host_id, port),
    CHECK (port BETWEEN 1 AND 65535),
    CHECK (purpose IN ('rpc','p2p','ws','metrics','sidecar'))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_host_port_reservations_node
    ON host_port_reservations (node_id);
```

```sql
CREATE TABLE IF NOT EXISTS host_probes (
    id                    INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    host_id               TEXT    NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    checked_at_unix       INTEGER NOT NULL,
    status                TEXT    NOT NULL,
    latency_ms            INTEGER,
    total_nodes           INTEGER,
    running_nodes         INTEGER,
    error_nodes           INTEGER,
    reported_agent_version TEXT,
    message               TEXT    NOT NULL DEFAULT '',
    CHECK (status IN ('reachable','degraded','unreachable','rejected')),
    CHECK (latency_ms IS NULL OR latency_ms >= 0)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_host_probes_recent
    ON host_probes (host_id, checked_at_unix DESC, id DESC);
```

> `syncing_nodes`, `total_blocks`, `total_peers` and `public_node_count` from `remote_server_probe_records` are **not** reproduced. The first was a process counter under a chain word; the others were always `NULL` for the NeoNexus↔NeoNexus topology. They return, per node, through mirrored `node_samples`.

### 003 — grouping

```sql
CREATE TABLE IF NOT EXISTS environments (
    id                     TEXT    NOT NULL PRIMARY KEY,
    label                  TEXT    NOT NULL,
    rank                   INTEGER NOT NULL DEFAULT 100,
    default_alarm_severity TEXT    NOT NULL DEFAULT 'warning',
    created_at_unix        INTEGER NOT NULL,
    CHECK (default_alarm_severity IN ('info','warning','critical')),
    CHECK (rank BETWEEN 0 AND 1000)
) STRICT;

INSERT OR IGNORE INTO environments (id,label,rank,default_alarm_severity,created_at_unix) VALUES
    ('production','Production',10,'critical',:now),
    ('staging','Staging',20,'warning',:now),
    ('development','Development',30,'info',:now),
    ('lab','Lab',40,'info',:now);

CREATE TABLE IF NOT EXISTS node_tags (
    node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    key     TEXT NOT NULL,
    value   TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (node_id, key),
    CHECK (length(key) BETWEEN 1 AND 64),
    CHECK (length(value) <= 256),
    CHECK (key = lower(key))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_node_tags_lookup ON node_tags (key, value, node_id);
```

### 004 — networks

```sql
CREATE TABLE IF NOT EXISTS networks (
    id                         TEXT    NOT NULL PRIMARY KEY,
    label                      TEXT    NOT NULL,
    family                     TEXT    NOT NULL,
    kind                       TEXT    NOT NULL,
    origin                     TEXT    NOT NULL DEFAULT 'authored',
    locked                     INTEGER NOT NULL DEFAULT 0,
    seed_hash                  TEXT,

    -- Neo N3 identity
    network_magic              INTEGER,
    validators_count           INTEGER,
    committee_public_keys      TEXT    NOT NULL DEFAULT '[]',
    seed_nodes                 TEXT    NOT NULL DEFAULT '[]',
    milliseconds_per_block     INTEGER,
    max_transactions_per_block INTEGER,

    -- Neo X identity
    chain_id                   INTEGER,
    bootnodes                  TEXT    NOT NULL DEFAULT '[]',
    genesis_hash               TEXT,
    genesis_path               TEXT,
    reth_chain_preset          TEXT,
    block_period_secs          INTEGER,

    -- observation anchor
    reference_endpoints        TEXT    NOT NULL DEFAULT '[]',

    notes                      TEXT    NOT NULL DEFAULT '',
    revision                   INTEGER NOT NULL DEFAULT 1,
    created_at_unix            INTEGER NOT NULL,
    updated_at_unix            INTEGER NOT NULL,
    created_by                 TEXT    NOT NULL DEFAULT 'system',

    -- Derived, never stored: a network cannot be marked bootable by a code path.
    complete INTEGER GENERATED ALWAYS AS (
        CASE family
          WHEN 'neo-n3' THEN CASE
            WHEN network_magic IS NOT NULL
             AND validators_count IS NOT NULL
             AND json_array_length(seed_nodes) >= 1
             AND json_array_length(committee_public_keys) >= validators_count
            THEN 1 ELSE 0 END
          WHEN 'neo-x' THEN CASE
            WHEN chain_id IS NOT NULL
             AND (kind <> 'private'
                  OR (genesis_path IS NOT NULL AND json_array_length(bootnodes) >= 1))
            THEN 1 ELSE 0 END
          ELSE 0
        END
    ) VIRTUAL,

    CHECK (family IN ('neo-n3','neo-x')),
    CHECK (kind   IN ('mainnet','testnet','private')),
    CHECK (origin IN ('seeded','authored','planned','imported')),
    CHECK (locked IN (0,1)),
    CHECK (json_valid(committee_public_keys) AND json_type(committee_public_keys) = 'array'),
    CHECK (json_valid(seed_nodes)            AND json_type(seed_nodes)            = 'array'),
    CHECK (json_valid(bootnodes)             AND json_type(bootnodes)             = 'array'),
    CHECK (json_valid(reference_endpoints)   AND json_type(reference_endpoints)   = 'array'),

    -- Identity that the family cannot function without, regardless of completeness.
    CHECK (family <> 'neo-n3' OR (network_magic IS NOT NULL
                                  AND network_magic BETWEEN 0 AND 4294967295
                                  AND validators_count IS NOT NULL
                                  AND validators_count >= 1)),
    CHECK (family <> 'neo-x'  OR (chain_id IS NOT NULL AND chain_id >= 1)),
    -- A public network is defined by the chain, not by this workspace.
    CHECK (kind = 'private' OR origin = 'seeded'),
    CHECK (milliseconds_per_block IS NULL OR milliseconds_per_block BETWEEN 1000 AND 600000),
    CHECK (block_period_secs IS NULL OR block_period_secs BETWEEN 1 AND 600)
) STRICT;

-- Exactly one mainnet and one testnet per family.
CREATE UNIQUE INDEX IF NOT EXISTS idx_networks_public_singleton
    ON networks (family, kind) WHERE kind <> 'private';
-- Two networks sharing a magic would cross-talk; today every private network
-- shares 1_230_000 by default.
CREATE UNIQUE INDEX IF NOT EXISTS idx_networks_n3_magic
    ON networks (network_magic) WHERE family = 'neo-n3';
CREATE UNIQUE INDEX IF NOT EXISTS idx_networks_neox_chain_id
    ON networks (chain_id) WHERE family = 'neo-x';
CREATE UNIQUE INDEX IF NOT EXISTS idx_networks_label
    ON networks (label COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_networks_incomplete
    ON networks (family, kind) WHERE complete = 0;

CREATE TABLE IF NOT EXISTS network_revisions (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    network_id      TEXT    NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    revision        INTEGER NOT NULL,
    changed_at_unix INTEGER NOT NULL,
    actor_kind      TEXT    NOT NULL DEFAULT 'unknown',
    actor_id        TEXT,
    reason          TEXT    NOT NULL DEFAULT '',
    identity        TEXT    NOT NULL,
    changed_fields  TEXT    NOT NULL DEFAULT '[]',
    UNIQUE (network_id, revision),
    CHECK (json_valid(identity) AND json_valid(changed_fields))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_network_revisions_recent
    ON network_revisions (network_id, changed_at_unix DESC);
```

Seeded rows (identity values transcribed from the constants that exist today):

```sql
INSERT INTO networks (id,label,family,kind,origin,locked,seed_hash,
                      network_magic,validators_count,committee_public_keys,seed_nodes,
                      milliseconds_per_block,max_transactions_per_block,
                      reference_endpoints,created_at_unix,updated_at_unix,created_by)
VALUES ('neo-n3-mainnet','Neo N3 MainNet','neo-n3','mainnet','seeded',1,:hash,
        860833102, 7, :mainnet_committee_json,
        '["seed1.neo.org:10333","seed2.neo.org:10333","seed3.neo.org:10333","seed4.neo.org:10333","seed5.neo.org:10333"]',
        15000, 200, '["https://mainnet1.neo.coz.io:443"]', :now,:now,'system')
ON CONFLICT(id) DO UPDATE SET
    network_magic=excluded.network_magic, validators_count=excluded.validators_count,
    committee_public_keys=excluded.committee_public_keys, seed_nodes=excluded.seed_nodes,
    milliseconds_per_block=excluded.milliseconds_per_block,
    max_transactions_per_block=excluded.max_transactions_per_block,
    seed_hash=excluded.seed_hash, updated_at_unix=excluded.updated_at_unix
WHERE networks.locked = 1 AND networks.seed_hash IS NOT excluded.seed_hash;
-- …and the same shape for neo-n3-testnet, neox-mainnet (chain_id 47763,
-- genesis 0x2ee5…dbd7, reth preset neox-mainnet, period 5, 2 bootnodes) and
-- neox-testnet (chain_id 12227332, genesis 0x221f…eb71).
```
