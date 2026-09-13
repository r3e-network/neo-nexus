## 3. Domain model

### 3.1 Entities

| Entity | Today | Why it becomes a row | Closes |
|---|---|---|---|
| **Network** | `enum Network{Mainnet,Testnet,Private}` + compile-time constant tables | `Private` carries no identity, so three modules re-derive magic/seeds/committee from constants with `profile: None`. Also the only correct home for every threshold P5 moves off the binary. | G18 G19 G22 G23 |
| **Host** | `format!("http://127.0.0.1:{}", rpc_port)`; local `Command::new` | A fleet is multi-host by definition. Every endpoint, port reservation, runtime installation and disk path is relative to a machine. | G34 G17 |
| **NodeSample** | `rpc_health_checks`: 2 calls, 24 rows, no latency, no peers | The whole of R2. One row per probe round carrying chain **and** process **and** disk, so the evaluator can put disk in the verdict. | G3 G4 G11 G12 G15 G16 G17 |
| **NodeHealthState / Transition** | nothing | Edge-trigger memory + the incident timeline the fake SVG path pretends to be. | G3 G11 |
| **AlarmRule / State / Transition** | four hardcoded `● OK` rows; no evaluator anywhere | `ChainHeadDelta`, `ConnectedPeers` become real metrics with real scope. `AlarmState::NoData` is the value whose absence let a never-probed node read green. | G1 G6 G14 |
| **AlertRoute** | five `alert_routing.*` settings keys, one provider | "Critical to PagerDuty, Warning to Slack" is two rows. | G14 |
| **ChainBlock** | nothing; `getblock` appears nowhere in `src/` | dBFT primary-index attribution — the only observable proxy for "am I producing my blocks". | G13 |
| **Designation / Transition** | `designation_status` CLI-only, persists nothing | *"Your Oracle designation was revoked at 03:14."* Append-on-change, so the table **is** the timeline. | G13 |
| **GovernanceSample / CandidateSample** | `governance_snapshot` CLI-only | Committee change detection; margin to rank 8 and rank 22. | G12 G13 |
| **NodeDuty** (set) | `node_roles`, one row per node | `plugin_states` is already many-to-many; the real single-duty limit is neo-go's exclusive `match role` over four signing services. Express that, don't flatten it. | G9 G39 |
| **SignerBackend / SignerKey** | `from_process_environment()` at startup, no insert, no reload | A backend you can only create by restarting the process is not a managed object. `key_id` gets a column, `curve` gets a home. | G28 G21 G44 |
| **NodeRevision / NetworkRevision** | `update_node` is a plain UPDATE | "What version was this on Tuesday", "what did we roll back from", "who changed it" — and the upgrader's missing rollback target. | G31 G35 |
| **NodeConfigRender** | `● In Sync` from `Path::is_file()` | Digest of what was written, for which node revision and network revision, for which purpose. Makes Start↔launch-pack byte-parity a testable assertion. | G5 G18 G32 |
| **RuntimeRelease / UpgradeRun / Attempt** | a parsed catalog entry alive only inside one call; seven `warn!`s | Breaks G25's circular dependency; replaces "batch completed: 0/3 successful" with a stage and a message per node. | G25 G31 |
| **ArchiveApplication** | unrecorded | Records `target_dir`, which is how "fast-sync ignored my custom datadir" becomes visible. | G40 G45 |
| **Environment / NodeTag / owner** | `Environment / Production` hardcoded on every node | Alarm scoping, filtering, bulk actions. | G7 G39 |
| **NodeSupervisionOverride** | one fleet-wide watchdog policy | Stopping one crash loop must not disable automatic restart fleet-wide. | G33 |
| **NodePeer / NetworkPeer** | `Vec::new()` literals; no field, no form | A private Neo X network is wireable only through the free-text args box. | G23 |
| **NodeNeox** (facet) | five `match node_type` sites | Namespaces, WS, authrpc/metrics ports, datadir, init state. | G21 G22 G23 |

**Deliberately not modelled**, because naming the decision matters as much as the entity list: instance type / vCPU / RAM / flavour; volume / IOPS / device / "Attached"; security group / CIDR / rule status; availability zone / VPC / region / account; `PrivateNetworkPlan` as a durable entity (the durable things are a Network row and its Node rows); a *declared* duty-support matrix (support is derived — §7.3); `ChainFamily` on `nodes` (derived from `node_type`; stored on `networks`, because a network must be authorable before any node exists on it).

### 3.2 SQL

SQLite floor **3.38** (generated columns 3.31, `STRICT` 3.37, JSON1 3.38); `rusqlite 0.37` with `bundled` ships 3.50.x. All new tables are `STRICT`. Timestamps are unix **seconds**; latency is **milliseconds** as `INTEGER`.

#### 3.2.1 Migration bookkeeping

```sql
CREATE TABLE IF NOT EXISTS schema_migrations (
    version         INTEGER NOT NULL PRIMARY KEY,
    name            TEXT    NOT NULL,
    applied_at_unix INTEGER NOT NULL,
    app_version     TEXT    NOT NULL DEFAULT ''
) STRICT;
```

Existing idempotent `add_column_if_missing` steps are recorded as version 1 (`baseline`) when `nodes` already exists, so no legacy workspace re-runs them.

#### 3.2.2 Observation (Stage 2 — additive, no rebuild)

One row per node per **sampling round**, where a round runs at the fast (`head`) cadence. Slower classes fill their columns only on the rounds they are due, leaving `NULL` elsewhere — this keeps the row count at the fast rate rather than the sum of all class rates, and SQLite stores a `NULL` in one byte.

```sql
CREATE TABLE IF NOT EXISTS node_samples (
    id                      INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    sampled_at_unix         INTEGER NOT NULL,
    node_id                 TEXT,            -- exactly one of node_id / reference_id
    reference_id            TEXT,            -- reference_endpoints.id
    family                  TEXT NOT NULL,   -- 'neo-n3' | 'neo-x'
    endpoint                TEXT NOT NULL,
    outcome                 TEXT NOT NULL,   -- ok | partial | unreachable | rejected
    error_kind              TEXT,            -- transport|timeout|tls|http-status|rpc-error|decode|unsupported-method
    error_detail            TEXT NOT NULL DEFAULT '',

    -- per-class attempt flag: NULL = not attempted this round
    head_ok                 INTEGER,
    head_time_ok            INTEGER,
    peers_ok                INTEGER,
    pool_ok                 INTEGER,
    identity_ok             INTEGER,

    -- head
    rpc_latency_ms          INTEGER,         -- primary method round trip; the only latency in the product
    block_height            INTEGER,         -- normalised: index of the highest block held
    header_height           INTEGER,         -- N3 getblockheadercount; NULL on Neo X
    best_block_hash         TEXT,
    syncing                 INTEGER,         -- Neo X eth_syncing: 1|0; NULL elsewhere
    sync_highest_block      INTEGER,

    -- head_time (chain clock, not ours)
    head_block_time_unix    INTEGER,         -- normalised to SECONDS (N3 divides getblockheader.time by 1000)

    -- derived-at-write, so the evaluator is a pure function of one row + state
    reference_height        INTEGER,
    head_lag_blocks         INTEGER,
    height_unchanged_secs   INTEGER,

    -- peers
    peers_connected         INTEGER,
    peers_unconnected       INTEGER,
    peers_bad               INTEGER,

    -- pool
    mempool_verified        INTEGER,
    mempool_unverified      INTEGER,
    mempool_capacity        INTEGER,
    gas_price_wei           INTEGER,         -- Neo X only

    -- identity (cheap, slow cadence, cached for a process lifetime)
    observed_magic          INTEGER,         -- N3 protocol.network | Neo X eth_chainId
    observed_genesis_hash   TEXT,
    observed_ms_per_block   INTEGER,
    observed_validators     INTEGER,
    observed_pool_capacity  INTEGER,
    client_version          TEXT,

    -- state service
    state_local_root_index      INTEGER,
    state_validated_root_index  INTEGER,

    -- host/process, so the verdict can name disk instead of advising a look
    process_cpu_percent     REAL,
    process_memory_bytes    INTEGER,
    disk_free_bytes         INTEGER,
    disk_total_bytes        INTEGER,

    CHECK (outcome IN ('ok','partial','unreachable','rejected')),
    CHECK ((node_id IS NULL) <> (reference_id IS NULL)),
    CHECK (syncing IS NULL OR syncing IN (0,1)),
    CHECK (rpc_latency_ms IS NULL OR rpc_latency_ms >= 0),
    CHECK (peers_connected IS NULL OR peers_connected >= 0),
    FOREIGN KEY (node_id) REFERENCES nodes(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX IF NOT EXISTS idx_node_samples_node_recent
    ON node_samples (node_id, sampled_at_unix DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_node_samples_reference_recent
    ON node_samples (reference_id, sampled_at_unix DESC) WHERE reference_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_node_samples_chain_key
    ON node_samples (family, observed_magic, observed_genesis_hash, sampled_at_unix DESC);
CREATE INDEX IF NOT EXISTS idx_node_samples_prune
    ON node_samples (sampled_at_unix);
```

> **Every nullable column means "not measured."** Never write `0` to say "we did not look". A neox-rs node with no metrics port gets `NULL` CPU, not `0.0`. This is the storage-layer statement of P2, and the `ON DELETE CASCADE` fixes half of G37 by construction.

```sql
-- One row per node: the state machine's memory. Findings are derived on read.
CREATE TABLE IF NOT EXISTS node_health_state (
    node_id                     TEXT NOT NULL PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    state                       TEXT NOT NULL,
    stall_scope                 TEXT,              -- 'node' | 'chain' | NULL
    since_unix                  INTEGER NOT NULL,
    evaluated_at_unix           INTEGER NOT NULL,
    reason                      TEXT NOT NULL,
    next_action                 TEXT NOT NULL,
    suspected_cause             TEXT,

    last_block_height           INTEGER,
    last_height_change_at_unix  INTEGER,
    first_sample_at_unix        INTEGER,
    consecutive_rpc_failures    INTEGER NOT NULL DEFAULT 0,
    consecutive_rpc_successes   INTEGER NOT NULL DEFAULT 0,
    candidate_state             TEXT,
    candidate_count             INTEGER NOT NULL DEFAULT 0,

    head_lag_blocks             INTEGER,
    reference_quality           TEXT NOT NULL,     -- ReferenceQuality discriminant
    chain_key                   TEXT,              -- '<family>:<magic>:<genesis>' — the observed chain
    observability               TEXT NOT NULL,     -- 'rpc' | 'process-only'
    sampling_interval_seconds   INTEGER NOT NULL,
    suppressed_until_unix       INTEGER
) STRICT;

CREATE TABLE IF NOT EXISTS node_health_transitions (
    id                  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id             TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    changed_at_unix     INTEGER NOT NULL,
    from_state          TEXT,                      -- NULL for the first observation ever
    to_state            TEXT NOT NULL,
    stall_scope         TEXT,
    reason              TEXT NOT NULL,
    head_lag_blocks     INTEGER,
    peers_connected     INTEGER,
    height_unchanged_secs INTEGER,
    disk_free_bytes     INTEGER
) STRICT;
CREATE INDEX IF NOT EXISTS idx_node_health_transitions_recent
    ON node_health_transitions (node_id, changed_at_unix DESC, id DESC);

-- A -32601 marks unsupported; a transport error never does. Retried every 6h,
-- because a runtime upgrade can add a method.
CREATE TABLE IF NOT EXISTS node_rpc_capabilities (
    node_id         TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    method          TEXT NOT NULL,
    support         TEXT NOT NULL,     -- supported | unsupported | unknown
    checked_at_unix INTEGER NOT NULL,
    detail          TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (node_id, method),
    CHECK (support IN ('supported','unsupported','unknown'))
) STRICT;

CREATE TABLE IF NOT EXISTS node_sample_rollups (
    node_id             TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    bucket_seconds      INTEGER NOT NULL,          -- 60 | 300 | 3600
    bucket_start_unix   INTEGER NOT NULL,
    samples             INTEGER NOT NULL,
    head_ok_samples     INTEGER NOT NULL,
    block_height_first  INTEGER,
    block_height_last   INTEGER,
    head_lag_min        INTEGER, head_lag_max INTEGER, head_lag_last INTEGER,
    chain_lag_max       INTEGER,
    latency_min_ms      INTEGER, latency_avg_ms INTEGER, latency_max_ms INTEGER,
    peers_min           INTEGER, peers_avg INTEGER, peers_max INTEGER,
    pool_last           INTEGER, pool_max INTEGER,
    cpu_percent_avg     REAL,    memory_bytes_max INTEGER, disk_free_min_bytes INTEGER,
    worst_state         TEXT NOT NULL,
    seconds_not_healthy INTEGER NOT NULL,
    PRIMARY KEY (node_id, bucket_seconds, bucket_start_unix),
    CHECK (bucket_seconds IN (60,300,3600)),
    CHECK (bucket_start_unix % bucket_seconds = 0),
    CHECK (head_ok_samples BETWEEN 0 AND samples)
) STRICT WITHOUT ROWID;

-- Provenance is operator-critical: "fleet median of 4" and "a public RPC" are
-- different claims about how much to trust the lag numbers above.
CREATE TABLE IF NOT EXISTS network_heads (
    chain_key        TEXT NOT NULL,     -- '<family>:<magic>:<genesis>'
    observed_at_unix INTEGER NOT NULL,
    height           INTEGER NOT NULL,
    best_block_hash  TEXT,
    source           TEXT NOT NULL,     -- configured | fleet-median | fleet-pair | public-seed | self-only
    contributors     INTEGER NOT NULL DEFAULT 0,
    source_detail    TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (chain_key, observed_at_unix),
    CHECK (source IN ('configured','fleet-median','fleet-pair','public-seed','self-only'))
) STRICT;

CREATE TABLE IF NOT EXISTS reference_endpoints (
    id              TEXT NOT NULL PRIMARY KEY,
    label           TEXT NOT NULL,
    endpoint        TEXT NOT NULL UNIQUE,
    family          TEXT NOT NULL,
    chain_key       TEXT,
    network_id      TEXT,              -- set from Stage 5
    enabled         INTEGER NOT NULL DEFAULT 1,
    source          TEXT NOT NULL,     -- operator | seed
    created_at_unix INTEGER NOT NULL,
    updated_at_unix INTEGER NOT NULL
) STRICT;
```

**Retention**, all settings-backed under `observation.retention.*`:

| Tier | Resolution | Kept | Bytes/node |
|---|---|---|---|
| `node_samples` | 15 s | 6 h | 0.25 MB |
| `node_sample_rollups(60)` | 1 min | 7 d | 2.1 MB |
| `node_sample_rollups(300)` | 5 min | 90 d | 5.2 MB |
| `node_sample_rollups(3600)` | 1 h | 400 d | 0.4 MB |
| `node_health_transitions` | event | 365 d | 0.4 MB |
| `chain_blocks` | per block | 20 000 rows (≈3.5 d at 15 s) | per network |
| governance / designation change logs | change + heartbeat | 365 d | negligible |

≈ **8.4 MB per node steady state**; 20 nodes ≈ 170 MB data, ≈ 200 MB on disk with indexes. Growth is **bounded**: the 90-day tier fills over the first quarter and then stops. Monthly growth after day 90 is zero. At 100 nodes, drop raw to 2 h and the 5-minute tier to 30 days via the setting.

Pruning is `DELETE … WHERE sampled_at_unix < ?` in batches of 5 000 inside one transaction, replacing `prune_rpc_health_keep_recent_per_node`, which runs a full per-node `DELETE … NOT IN (SELECT … LIMIT 24)` **on every probe**. Set `PRAGMA auto_vacuum = INCREMENTAL` at creation and run `PRAGMA incremental_vacuum(1000)` after each prune; existing workspaces get a one-time `VACUUM` in the migration, behind a size check, announced in the changelog.

#### 3.2.3 Alarms (Stage 3)

```sql
CREATE TABLE IF NOT EXISTS alert_routes (
    id                    TEXT NOT NULL PRIMARY KEY,
    label                 TEXT NOT NULL,
    provider              TEXT NOT NULL,
    target                TEXT NOT NULL,
    min_severity          TEXT NOT NULL DEFAULT 'warning',
    heartbeat_seconds     INTEGER,             -- see §6.8: dead-man
    timeout_seconds       INTEGER NOT NULL DEFAULT 10,
    enabled               INTEGER NOT NULL DEFAULT 1,
    created_at_unix       INTEGER NOT NULL,
    updated_at_unix       INTEGER NOT NULL,
    last_delivery_at_unix INTEGER,
    last_delivery_status  TEXT,
    CHECK (min_severity IN ('info','warning','critical')),
    CHECK (timeout_seconds BETWEEN 1 AND 120)
) STRICT;

CREATE TABLE IF NOT EXISTS alarm_rules (
    id                     TEXT NOT NULL PRIMARY KEY,
    name                   TEXT NOT NULL,
    enabled                INTEGER NOT NULL DEFAULT 0,   -- seeded rules ship OFF
    metric                 TEXT NOT NULL,
    comparator             TEXT NOT NULL,                -- gt|gte|lt|lte|eq|neq
    threshold              REAL NOT NULL,
    recovery_threshold     REAL,                         -- defaults to threshold
    for_seconds            INTEGER NOT NULL DEFAULT 120,
    recovery_for_seconds   INTEGER NOT NULL DEFAULT 360, -- 3x: quick to fire, slow to clear
    missing_data           TEXT NOT NULL DEFAULT 'no-data',
    selector_kind          TEXT NOT NULL DEFAULT 'all',
    selector_value         TEXT NOT NULL DEFAULT '',
    severity               TEXT NOT NULL DEFAULT 'warning',
    route_id               TEXT REFERENCES alert_routes(id) ON DELETE SET NULL,
    runbook_url            TEXT,
    description            TEXT NOT NULL DEFAULT '',
    origin                 TEXT NOT NULL DEFAULT 'authored',
    created_at_unix        INTEGER NOT NULL,
    updated_at_unix        INTEGER NOT NULL,
    updated_by             TEXT NOT NULL DEFAULT 'system',
    CHECK (metric IN ('head-lag-blocks','height-unchanged-secs','chain-lag-secs',
                      'peers-connected','rpc-latency-ms','mempool-utilisation-percent',
                      'blocks-per-minute','sample-age-secs','restarts-in-last-hour',
                      'disk-free-percent','process-memory-bytes',
                      'health-state-is','duty-state-is','chain-identity-mismatch',
                      'consensus-slots-skipped','consensus-interval-overrun-secs')),
    CHECK (comparator IN ('gt','gte','lt','lte','eq','neq')),
    CHECK (missing_data IN ('no-data','breaching','not-breaching','ignore')),
    CHECK (selector_kind IN ('all','node','host','network','chain-family',
                             'duty','environment','tag','node-type')),
    CHECK (severity IN ('info','warning','critical')),
    CHECK (selector_kind = 'all' OR selector_value <> '')
) STRICT;

CREATE TABLE IF NOT EXISTS alarm_states (
    rule_id                TEXT NOT NULL REFERENCES alarm_rules(id) ON DELETE CASCADE,
    node_id                TEXT NOT NULL REFERENCES nodes(id)       ON DELETE CASCADE,
    state                  TEXT NOT NULL,     -- no-data | ok | pending | alarm | suppressed
    no_data_reason         TEXT,              -- never-evaluated | no-samples | sampling-disabled
                                              -- | metric-unavailable | node-stopped
    since_unix             INTEGER NOT NULL,
    evaluated_at_unix      INTEGER,           -- NULL = never evaluated
    observed_value         REAL,
    datapoints             INTEGER NOT NULL DEFAULT 0,
    breach_started_at_unix INTEGER,
    clear_started_at_unix  INTEGER,
    flapping               INTEGER NOT NULL DEFAULT 0,
    transitions_in_window  INTEGER NOT NULL DEFAULT 0,
    reason                 TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (rule_id, node_id),
    CHECK (state IN ('no-data','ok','pending','alarm','suppressed'))
) STRICT;

CREATE TABLE IF NOT EXISTS alarm_transitions (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    rule_id         TEXT NOT NULL REFERENCES alarm_rules(id) ON DELETE CASCADE,
    node_id         TEXT NOT NULL REFERENCES nodes(id)       ON DELETE CASCADE,
    changed_at_unix INTEGER NOT NULL,
    from_state      TEXT NOT NULL,
    to_state        TEXT NOT NULL,
    value           REAL,
    reason          TEXT NOT NULL DEFAULT '',
    event_id        INTEGER REFERENCES runtime_events(id) ON DELETE SET NULL,
    CHECK (from_state <> to_state)
) STRICT;
```

#### 3.2.4 Chain state (Stage 4)

Keyed on `chain_key` through Stage 4, re-keyed to `network_id` in Stage 5.

```sql
-- Append only when the digest changes; otherwise bump last_seen_at_unix in place.
-- The table is therefore a change log by construction, and a 5-minute poll does
-- not write 288 identical rows per network per day.
CREATE TABLE IF NOT EXISTS chain_governance_samples (
    id                   INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    chain_key            TEXT NOT NULL,
    network_id           TEXT,
    first_seen_at_unix   INTEGER NOT NULL,
    last_seen_at_unix    INTEGER NOT NULL,
    height               INTEGER NOT NULL,
    committee            TEXT NOT NULL,   -- JSON array, in returned order
    next_validators      TEXT NOT NULL,
    committee_digest     TEXT NOT NULL,
    validators_digest    TEXT NOT NULL,
    source_node_id       TEXT REFERENCES nodes(id) ON DELETE SET NULL,
    CHECK (json_valid(committee) AND json_valid(next_validators))
) STRICT;

-- Only the rows a fleet cares about: its own keys, plus the two rank boundaries.
CREATE TABLE IF NOT EXISTS chain_candidate_samples (
    id               INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    sampled_at_unix  INTEGER NOT NULL,
    chain_key        TEXT NOT NULL,
    public_key       TEXT NOT NULL,
    rank             INTEGER NOT NULL,
    votes            INTEGER NOT NULL,
    candidate_count  INTEGER NOT NULL,
    role             TEXT NOT NULL     -- fleet-key | boundary-validator | boundary-committee
) STRICT;

-- Populated only for chains where the fleet holds a Consensus duty.
CREATE TABLE IF NOT EXISTS chain_blocks (
    chain_key         TEXT NOT NULL,
    height            INTEGER NOT NULL,
    block_time_unix   INTEGER NOT NULL,
    primary_index     INTEGER NOT NULL,
    tx_count          INTEGER NOT NULL,
    next_consensus    TEXT NOT NULL,
    validators_digest TEXT NOT NULL,   -- joins to the governance row in force at this height
    PRIMARY KEY (chain_key, height)
) STRICT WITHOUT ROWID;

CREATE TABLE IF NOT EXISTS node_designations (
    id                  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id             TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    chain_role          INTEGER NOT NULL,   -- 4|8|16|32, consensus-visible, never renumbered
    first_seen_at_unix  INTEGER NOT NULL,
    last_seen_at_unix   INTEGER NOT NULL,
    height              INTEGER NOT NULL,
    compared_key        TEXT,               -- NULL = we had no key to compare
    key_source          TEXT NOT NULL,      -- signer-binding | wallet-profile | none
    includes_node_key   INTEGER,            -- 0|1|NULL — mirrors Option<bool>, do not collapse
    designated_keys     TEXT NOT NULL DEFAULT '[]',
    designated_digest   TEXT NOT NULL,
    outcome             TEXT NOT NULL,      -- designated|not-designated|no-key|query-failed
    detail              TEXT NOT NULL DEFAULT ''
) STRICT;

CREATE TABLE IF NOT EXISTS node_designation_transitions (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id         TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    chain_role      INTEGER NOT NULL,
    changed_at_unix INTEGER NOT NULL,
    from_status     TEXT,
    to_status       TEXT NOT NULL,
    height          INTEGER
) STRICT;
```

> `includes_node_key = NULL` means **"no key to compare"**, which is a different operator response from "not designated". The CLI currently collapses both into exit 1 (`cli/actions/chain.rs:23-26`); that collapse is removed — key-unknown exits **2**.

#### 3.2.5 Networks (Stage 5)

```sql
CREATE TABLE IF NOT EXISTS networks (
    id                         TEXT NOT NULL PRIMARY KEY,
    label                      TEXT NOT NULL,
    family                     TEXT NOT NULL,           -- neo-n3 | neo-x
    kind                       TEXT NOT NULL,           -- mainnet | testnet | private
    origin                     TEXT NOT NULL DEFAULT 'authored',  -- seeded|authored|planned|quarantined|imported
    locked                     INTEGER NOT NULL DEFAULT 0,
    seed_hash                  TEXT,
    parent_network_id          TEXT REFERENCES networks(id),      -- neox-mainnet -> neo-n3-mainnet

    -- Neo N3 identity
    network_magic              INTEGER,
    validators_count           INTEGER,
    committee_public_keys      TEXT NOT NULL DEFAULT '[]',
    seed_nodes                 TEXT NOT NULL DEFAULT '[]',
    milliseconds_per_block     INTEGER,
    max_transactions_per_block INTEGER,
    memory_pool_max_tx         INTEGER,

    -- Neo X identity
    chain_id                   INTEGER,
    bootnodes                  TEXT NOT NULL DEFAULT '[]',
    genesis_hash               TEXT,
    genesis_path               TEXT,
    genesis_sha256             TEXT,
    reth_chain_spec            TEXT,
    block_period_secs          INTEGER,

    -- observation policy for THIS chain (P5)
    chain_key                  TEXT,                    -- observed identity, set by the sampler
    stall_multiple             INTEGER NOT NULL DEFAULT 20,
    expected_peers_floor       INTEGER NOT NULL DEFAULT 3,
    on_demand_blocks           INTEGER NOT NULL DEFAULT 0,  -- private chains that only produce on tx

    notes                      TEXT NOT NULL DEFAULT '',
    revision                   INTEGER NOT NULL DEFAULT 1,
    created_at_unix            INTEGER NOT NULL,
    updated_at_unix            INTEGER NOT NULL,
    created_by                 TEXT NOT NULL DEFAULT 'system',

    -- Derived. No code path can mark a network bootable that is not.
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
                  OR (genesis_path IS NOT NULL AND genesis_sha256 IS NOT NULL
                      AND json_array_length(bootnodes) >= 1))
            THEN 1 ELSE 0 END
          ELSE 0 END
    ) VIRTUAL,

    CHECK (family IN ('neo-n3','neo-x')),
    CHECK (kind   IN ('mainnet','testnet','private')),
    CHECK (origin IN ('seeded','authored','planned','quarantined','imported')),
    CHECK (json_valid(committee_public_keys) AND json_type(committee_public_keys) = 'array'),
    CHECK (json_valid(seed_nodes)            AND json_type(seed_nodes)            = 'array'),
    CHECK (json_valid(bootnodes)             AND json_type(bootnodes)             = 'array'),
    CHECK (kind = 'private' OR origin = 'seeded'),
    CHECK (network_magic IS NULL OR network_magic BETWEEN 0 AND 4294967295),
    CHECK (chain_id IS NULL OR chain_id >= 1),
    CHECK (milliseconds_per_block IS NULL OR milliseconds_per_block BETWEEN 1000 AND 600000),
    CHECK (stall_multiple BETWEEN 3 AND 500)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_networks_public_singleton
    ON networks (family, kind) WHERE kind <> 'private';
-- Two private networks sharing 1_230_000 is today's universal default. Not after this.
-- Quarantined rows carry NULL magic and are exempt until identity is authored.
CREATE UNIQUE INDEX IF NOT EXISTS idx_networks_n3_magic
    ON networks (network_magic) WHERE family = 'neo-n3' AND network_magic IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_networks_neox_chain_id
    ON networks (chain_id) WHERE family = 'neo-x' AND chain_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_networks_label ON networks (label COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_networks_incomplete ON networks (family, kind) WHERE complete = 0;

CREATE TABLE IF NOT EXISTS network_revisions (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    network_id      TEXT NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    revision        INTEGER NOT NULL,
    changed_at_unix INTEGER NOT NULL,
    actor_kind      TEXT NOT NULL DEFAULT 'unknown',
    actor_id        TEXT,
    reason          TEXT NOT NULL DEFAULT '',
    identity        TEXT NOT NULL,
    changed_fields  TEXT NOT NULL DEFAULT '[]',
    UNIQUE (network_id, revision)
) STRICT;

CREATE TABLE IF NOT EXISTS network_peers (
    network_id    TEXT NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    peer          TEXT NOT NULL,          -- enode:// (Neo X) or host:port (N3 seed)
    kind          TEXT NOT NULL,          -- bootnode | static | trusted | seed
    note          TEXT NOT NULL DEFAULT '',
    added_at_unix INTEGER NOT NULL,
    PRIMARY KEY (network_id, peer, kind),
    CHECK (kind IN ('bootnode','static','trusted','seed'))
) STRICT;
```

**Seeded rows** (`origin='seeded'`, `locked=1`) for `neo-n3-mainnet`, `neo-n3-testnet`, `neox-mainnet` (chain_id 47763, genesis `0x2ee5…dbd7`, spec `neox-mainnet`, 5 s), `neox-testnet` (12227332, `0x221f…eb71`). The constants in `src/config/format/{network,committee,neox}.rs` move to `src/config/format/seeds.rs` and become `pub(in crate::repository::seed)` — reachable **only** from the seeder. `seed_nodes()`, `standby_committee()`, `network_magic()`, `validators_count()`, `neox_chain_id()`, `neox_bootnodes()`, `neox_genesis_hash()`, `neox_reth_chain()`, `neox_block_period_secs()` and every `effective_*` helper are deleted from the generation path.

On each open the seeder upserts a locked row whose `seed_hash` differs from the shipped constant (so a transcription fix in a release reaches existing workspaces) and never touches an unlocked row.

> **Seeding the four public rows is a first-run requirement, not a nicety.** Without it, removing the compiled-in fallback makes standing up a first N3 RPC node begin with the operator sourcing a seed list and a standby committee from Neo's documentation — a regression on today's behaviour. This is the one place the newcomer judge's criticism of three separate proposals bites hardest, and it is answered by a schema seed.

#### 3.2.6 Hosts (Stage 5)

```sql
CREATE TABLE IF NOT EXISTS hosts (
    id                   TEXT NOT NULL PRIMARY KEY,
    label                TEXT NOT NULL,
    transport            TEXT NOT NULL,        -- local-process | ssh | neonexus-peer
    address              TEXT NOT NULL DEFAULT '127.0.0.1',
    service_scheme       TEXT NOT NULL DEFAULT 'http',
    control_endpoint     TEXT,
    credential_ref       TEXT,                 -- names a secret; never holds one
    workspace_root       TEXT,
    supervises_processes INTEGER NOT NULL DEFAULT 1,
    enabled              INTEGER NOT NULL DEFAULT 1,
    description          TEXT NOT NULL DEFAULT '',
    os TEXT, arch TEXT, agent_version TEXT,
    last_seen_at_unix    INTEGER,
    created_at_unix      INTEGER NOT NULL,
    updated_at_unix      INTEGER NOT NULL,
    CHECK (transport IN ('local-process','ssh','neonexus-peer')),
    CHECK (service_scheme IN ('http','https')),
    -- One process supervises one machine. A second local host asserts a place
    -- where Command::new does not run.
    CHECK (transport <> 'local-process' OR (id = 'local' AND supervises_processes = 1)),
    -- A peer is observed, never supervised: there is no way to spawn there.
    CHECK (transport <> 'neonexus-peer' OR (control_endpoint IS NOT NULL AND supervises_processes = 0)),
    CHECK (transport <> 'ssh' OR control_endpoint IS NOT NULL)
) STRICT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_hosts_control_endpoint
    ON hosts (control_endpoint) WHERE control_endpoint IS NOT NULL;

-- Every collision becomes a constraint violation with a readable message,
-- instead of a surprise at Start. Maintained by triggers on `nodes`.
CREATE TABLE IF NOT EXISTS host_port_reservations (
    host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    port    INTEGER NOT NULL,
    node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    purpose TEXT NOT NULL,     -- rpc | p2p | ws | metrics | authrpc | sidecar
    PRIMARY KEY (host_id, port),
    CHECK (port BETWEEN 1 AND 65535),
    CHECK (purpose IN ('rpc','p2p','ws','metrics','authrpc','sidecar'))
) STRICT;

CREATE TABLE IF NOT EXISTS host_probes (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    host_id TEXT NOT NULL REFERENCES hosts(id) ON DELETE CASCADE,
    checked_at_unix INTEGER NOT NULL,
    status TEXT NOT NULL,      -- reachable | degraded | unreachable | rejected
    latency_ms INTEGER,
    total_nodes INTEGER, running_nodes INTEGER, error_nodes INTEGER,
    reported_agent_version TEXT,
    message TEXT NOT NULL DEFAULT ''
) STRICT;
```

> `syncing_nodes`, `total_blocks`, `total_peers` and `public_node_count` from `remote_server_probe_records` are **not** carried across. The first counted processes inside a ~600 ms launch window under a column headed "Syncing" (G17); the others were always `NULL` for the NeoNexus↔NeoNexus topology. They return, per node, through mirrored `node_samples`.

#### 3.2.7 `nodes` rebuild (Stage 5)

Rebuilt, not altered: we need `host_id NOT NULL DEFAULT 'local'` with a `REFERENCES`, `network_id NOT NULL`, a `UNIQUE` name (G38), and the removal of `network`. SQLite's documented 12-step procedure, `PRAGMA foreign_keys = OFF` for the duration (so child `REFERENCES nodes(id)` clauses are not rewritten by the rename), `PRAGMA foreign_key_check` before commit.

```sql
CREATE TABLE nodes_new (
    id                   TEXT NOT NULL PRIMARY KEY,
    name                 TEXT NOT NULL,
    host_id              TEXT NOT NULL DEFAULT 'local' REFERENCES hosts(id)    ON DELETE RESTRICT,
    network_id           TEXT NOT NULL                 REFERENCES networks(id) ON DELETE RESTRICT,
    node_type            TEXT NOT NULL,
    origin               TEXT NOT NULL DEFAULT 'managed',   -- managed | mirrored

    binary_path          TEXT NOT NULL,
    args                 TEXT NOT NULL DEFAULT '',
    runtime_version      TEXT NOT NULL DEFAULT 'latest',
    runtime_package_id   TEXT,
    runtime_layout       TEXT NOT NULL DEFAULT 'shared',    -- shared | per-node  (G24)
    storage_engine       TEXT NOT NULL DEFAULT 'leveldb',

    data_dir             TEXT,
    config_mode          TEXT NOT NULL DEFAULT 'managed',   -- managed | external
    external_config_path TEXT,

    rpc_port             INTEGER NOT NULL DEFAULT 10332,
    p2p_port             INTEGER NOT NULL DEFAULT 10333,
    ws_port              INTEGER,
    metrics_port         INTEGER,
    authrpc_port         INTEGER,

    environment_id       TEXT REFERENCES environments(id) ON DELETE SET NULL,
    owner                TEXT NOT NULL DEFAULT '',

    status               TEXT NOT NULL,
    pid                  INTEGER,

    revision             INTEGER NOT NULL DEFAULT 1,
    created_at_unix      INTEGER NOT NULL,
    created_at_estimated INTEGER NOT NULL DEFAULT 0,
    updated_at_unix      INTEGER NOT NULL,

    CHECK (origin IN ('managed','mirrored')),
    CHECK (runtime_layout IN ('shared','per-node')),
    CHECK (config_mode IN ('managed','external')),
    CHECK (config_mode = 'managed' OR external_config_path IS NOT NULL),
    CHECK (rpc_port <> p2p_port),
    CHECK (ws_port      IS NULL OR (ws_port <> rpc_port AND ws_port <> p2p_port)),
    CHECK (metrics_port IS NULL OR (metrics_port NOT IN (rpc_port, p2p_port)
                                AND (ws_port IS NULL OR metrics_port <> ws_port))),
    CHECK (authrpc_port IS NULL OR (authrpc_port NOT IN (rpc_port, p2p_port)
                                AND (ws_port      IS NULL OR authrpc_port <> ws_port)
                                AND (metrics_port IS NULL OR authrpc_port <> metrics_port))),
    -- A node NeoNexus does not supervise cannot own a local pid.
    CHECK (pid IS NULL OR origin = 'managed')
) STRICT;

-- Name is a join key: the launch-pack exporter keys members by name while
-- uniqueness was checked only at plan time, so duplicates wrote one member's
-- config twice (G38).
CREATE UNIQUE INDEX idx_nodes_name_unique ON nodes (name COLLATE NOCASE);
CREATE INDEX idx_nodes_network ON nodes (network_id, name COLLATE NOCASE);
CREATE INDEX idx_nodes_host    ON nodes (host_id,    name COLLATE NOCASE);
CREATE INDEX idx_nodes_status  ON nodes (status,     name COLLATE NOCASE);
CREATE INDEX idx_nodes_type    ON nodes (node_type,  name COLLATE NOCASE);
```

Port reservation triggers (`trg_nodes_ports_ai` / `_au`) insert one `host_port_reservations` row per non-NULL port for `origin='managed'` nodes, and rewrite them on any port/host/origin update.

History, with the invariant in the database so a future caller cannot skip it:

```sql
CREATE TABLE IF NOT EXISTS node_revisions (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id         TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    revision        INTEGER NOT NULL,
    changed_at_unix INTEGER NOT NULL,
    actor_kind      TEXT NOT NULL DEFAULT 'unknown',
    actor_id        TEXT,
    reason          TEXT NOT NULL DEFAULT '',
    spec            TEXT NOT NULL,                 -- full node row at this revision
    changed_fields  TEXT NOT NULL DEFAULT '[]',
    event_id        INTEGER REFERENCES runtime_events(id) ON DELETE SET NULL,
    UNIQUE (node_id, revision),
    CHECK (actor_kind IN ('operator','cli','api-token','agent','watchdog',
                          'supervisor','upgrader','system','unknown'))
) STRICT;

-- Runtime state (status, pid) is NOT spec and creates no history; transition_node_status
-- is unaffected.
CREATE TRIGGER trg_nodes_revision_required BEFORE UPDATE ON nodes
WHEN NEW.revision = OLD.revision
 AND (NEW.name IS NOT OLD.name OR NEW.host_id IS NOT OLD.host_id
   OR NEW.network_id IS NOT OLD.network_id OR NEW.node_type IS NOT OLD.node_type
   OR NEW.binary_path IS NOT OLD.binary_path OR NEW.args IS NOT OLD.args
   OR NEW.runtime_version IS NOT OLD.runtime_version
   OR NEW.runtime_package_id IS NOT OLD.runtime_package_id
   OR NEW.runtime_layout IS NOT OLD.runtime_layout
   OR NEW.storage_engine IS NOT OLD.storage_engine OR NEW.data_dir IS NOT OLD.data_dir
   OR NEW.config_mode IS NOT OLD.config_mode
   OR NEW.external_config_path IS NOT OLD.external_config_path
   OR NEW.rpc_port IS NOT OLD.rpc_port OR NEW.p2p_port IS NOT OLD.p2p_port
   OR NEW.ws_port IS NOT OLD.ws_port OR NEW.metrics_port IS NOT OLD.metrics_port
   OR NEW.authrpc_port IS NOT OLD.authrpc_port
   OR NEW.environment_id IS NOT OLD.environment_id OR NEW.owner IS NOT OLD.owner)
BEGIN
    SELECT RAISE(ABORT,
      'node spec changed without advancing nodes.revision; write node_revisions first');
END;

CREATE TRIGGER trg_nodes_revision_paired AFTER UPDATE OF revision ON nodes
WHEN (SELECT count(*) FROM node_revisions
       WHERE node_id = NEW.id AND revision = NEW.revision) = 0
BEGIN
    SELECT RAISE(ABORT, 'nodes.revision advanced with no matching node_revisions row');
END;
```

Duties, peers, supervision:

```sql
CREATE TABLE IF NOT EXISTS node_duties (
    node_id          TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    duty             TEXT NOT NULL,
    is_primary       INTEGER NOT NULL DEFAULT 0,
    assigned_at_unix INTEGER NOT NULL,
    assigned_by      TEXT NOT NULL DEFAULT 'system',
    PRIMARY KEY (node_id, duty),
    CHECK (duty IN ('rpc-api','state','indexer','validator','oracle',
                    'state-validator','notary','observer'))
) STRICT;
CREATE UNIQUE INDEX idx_node_duties_primary ON node_duties (node_id) WHERE is_primary = 1;
-- neo-go's exclusive `match role` over its four signing services is the real
-- single-duty limit. State this instead of flattening every node to one duty.
CREATE UNIQUE INDEX idx_node_duties_single_signing ON node_duties (node_id)
    WHERE duty IN ('validator','oracle','state-validator','notary');

CREATE TABLE IF NOT EXISTS node_peers (
    node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    peer    TEXT NOT NULL,
    kind    TEXT NOT NULL,   -- bootnode | static | trusted | seed | oracle
    note    TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (node_id, peer, kind)
) STRICT;

CREATE TABLE IF NOT EXISTS node_supervision_overrides (
    node_id              TEXT NOT NULL PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    restart_enabled      INTEGER,      -- NULL = inherit the workspace policy
    max_restart_attempts INTEGER, base_delay_seconds INTEGER, max_delay_seconds INTEGER,
    jitter_enabled       INTEGER,
    paused_until_unix    INTEGER,      -- "stop relaunching the node I am editing"
    paused_reason        TEXT NOT NULL DEFAULT '',
    health_restart_state TEXT,         -- opt-in: 'stalled' | 'unreachable' | NULL
    health_restart_after_secs INTEGER,
    updated_at_unix      INTEGER NOT NULL,
    updated_by           TEXT NOT NULL DEFAULT 'system'
) STRICT;
```

#### 3.2.8 The remaining tables

Mechanical; full DDL follows the same shape. Listed with their load-bearing constraint.

| Table | Key columns | The constraint that matters |
|---|---|---|
| `environments` | `id, label, rank, default_alarm_severity` | Seeded 4 rows. **Migration assigns no node an environment** — the hardcoded `Environment/Production` is deleted, not migrated. |
| `node_tags` | `(node_id, key, value)` | `idx_node_tags_lookup(key, value, node_id)` so `tag:team=core` is an indexed alarm selector. |
| `signer_backends` | `id, label, kind, endpoint, wallet_profile_id, source` | `CHECK (kind <> 'local-wallet' OR wallet_profile_id IS NOT NULL)`. `source ∈ {workspace, process-environment}`. |
| `signer_keys` | `(backend_id, key_id), label, curve, public_key, state, discovered` | `CHECK (curve IN ('secp256r1','secp256k1'))` — the curve column is what makes N3↔Neo X key mismatch a data fact rather than a runtime string comparison. |
| `node_signer_bindings` | rebuilt: `+ bound_at_unix, bound_by`, composite FK to `signer_keys` | keeps `idx_node_signer_bindings_exclusive_lease` so `enforce_signer_lease_exclusivity` continues to apply. |
| `node_config_renders` | `node_id, purpose, target_path, node_revision, network_revision, primary_sha256, sidecar_sha256` | `purpose ∈ {launch, export, launch-pack, drift-check}`; the parity index `(node_id, node_revision, network_revision, purpose)`. |
| `runtime_releases` | `catalog_profile_id, package_id, node_type, version, os, arch, url, sha256, signature` | `UNIQUE(catalog_profile_id, package_id)`, `length(sha256)=64`. |
| `runtime_installations` | rebuilt: PK `(host_id, package_id)`, `+ release_id, install_root` | installations are per machine. |
| `runtime_upgrade_runs` / `_attempts` | `from_version, to_version, from_node_revision, outcome, stage, message` | `stage ∈ {select,download,install,stop,rebind,start,verify,complete}` — the rollback target `update_node` never retained. |
| `archive_applications` | `node_id, archive_id, target_dir, height_before/after, outcome` | records `target_dir` (G40). |
| `fast_sync_archives` | renamed from `fast_sync_snapshots`, `+ network_id` | `compatible_entries` finally has a real compatibility key. |
| `node_neox` | `http_enabled, http_api, ws_enabled, ws_api, ws_origins, authrpc_port, metrics_enabled, dangerous_ack, init_state, init_genesis_sha256` | `CHECK (init_state IN ('uninitialised','initialised','mismatched','not-applicable'))`. |
| `runtime_events` | `+ actor_kind, actor_id, host_id, network_id, correlation_id, details` | `actor_kind` domain enforced in Rust (`ALTER TABLE` cannot add `CHECK`) and asserted by the integrity checker. |
| `api_tokens` | `+ scope_node_id REFERENCES nodes(id) ON DELETE CASCADE, revoked_at_unix, last_used_at_unix` | closes the other half of G37. |

### 3.3 Migration

17 idempotent steps, each recorded in `schema_migrations`, each leaving the workspace openable by the release that introduced it. Two columns because the distinction is the whole discipline: **backfilled from evidence** vs **defaulted and flagged**. Nothing is invented.

| # | Step | Backfilled from evidence | Defaulted and flagged |
|---|---|---|---|
| 000 | `schema_migrations`; record `baseline` = 1 | — | — |
| 001 | **Pre-flight conflict scan** (§3.4) | — | — |
| 002 | `hosts`, `host_port_reservations`, `host_probes`; seed `local` | `workspace_root` from the open workspace path | Every node → `local`. Not an assumption: the only spawn is local `Command::new`. |
| 003 | `environments` (seeded), `node_tags` | — | **Nothing.** The hardcoded Production tag is deleted. |
| 004 | `networks`, `network_revisions`, `network_peers`; seed 4 public; **quarantine one row per private node** | Public identity from shipped constants. Quarantined rows take `chain_key` from Stage-2 observation where a sample exists. | Quarantined rows: identity `NULL`, `complete = 0` → blocks Start and raises a Critical readiness finding. Empty is preserved, never invented. |
| 005 | `nodes` rebuild; `node_revisions` + triggers; `node_duties`; `node_peers`; `node_supervision_overrides` | `network_id` per 004's rule. `runtime_package_id` by matching `binary_path` against `runtime_installations`. `created_at_unix` from the earliest `runtime_events.occurred_at_unix` for that node (preferring `node-created`). | No event ⇒ `created_at_unix = :now` **and `created_at_estimated = 1`**, so the UI never claims a creation time it invented. `data_dir`, `environment_id`, `owner`, `metrics_port`, `authrpc_port` = NULL. |
| 006 | `node_duties` from `node_roles` | one row per existing row, `is_primary = 1` | A node with no `node_roles` row gets **no duty row**. `None` stays `None` and never renders as Observer / Node / standard / pre-selected rpc-api (G9). |
| 007 | `signer_backends`, `signer_keys`; `node_signer_bindings` rebuild | backends from `from_process_environment()` at first open (`source='process-environment'`); keys from each distinct `(backend_id, key_id)` in bindings | discovered keys: `discovered=1`, `public_key=NULL`, `state='unknown'`, curve from the backend's family. Rendered as *"seen in a binding, never confirmed by the backend"*. |
| 008 | *(Stage 2, lands first chronologically)* observation tables; dual-write with `rpc_health_checks` for one release | every `rpc_health_checks` row → a sample | `rpc_latency_ms`, `peers_connected`, `head_lag_blocks`, CPU, memory = **NULL**, never 0. |
| 009 | alarms + routes | one route from the five `alert_routing.*` keys, labelled "Default route"; existing deliveries point at it | Seeded rules are `enabled = 0`. An alarm nobody has reviewed must not page at 03:00 on the strength of a migration. |
| 010 | chain-state tables | — | Empty. Nothing on disk records a past designation; inventing one is the failure being fixed. |
| 011 | `runtime_releases`; `runtime_installations` rebuild; upgrade tables; **seed the default Neo catalog profile** | installs → `host_id='local'`, `install_root` = parent of `binary_path` | `release_id = NULL` for pre-existing installs. The seed breaks G25's circular dependency. |
| 012 | `node_config_renders`, `archive_applications`; `fast_sync_archives.network_id` | archive `network` string + family → `network_id` | render history starts empty and fills on the next Start/export. |
| 013 | `runtime_events` actor columns + indexes | — | `actor_kind='unknown'` for every existing row. The `contains("Hermes")` heuristic is deleted, not ported. |
| 014 | `remote_servers` → `hosts`; probes → `host_probes`; drop both | id, label←name, description, enabled, timestamps, `control_endpoint`←`base_url`, address = its host component | `transport='neonexus-peer'`, `supervises_processes=0`. `syncing_nodes` et al. dropped. |
| 015 | `api_tokens` scoping | `scope_node_id` parsed from `permissions` where it already encodes a node namespace | NULL = workspace-scoped. |
| 016 | `node_neox` facet | `http_api` from the geth constant; ports from `args` via `argv_read` | `ws_enabled = (ws_port IS NOT NULL AND node_type = 'neox-geth')` — neox-rs never opened one. |
| 017 | drop `rpc_health_checks`, create a compatibility view over `node_samples` | — | one release after 008 |

Legacy dispositions: `node_roles` → view over `node_duties WHERE is_primary=1` for one release; the five orphaned `signer_*` tables from the custody split stay untouched per the existing comment in `tables.rs`.

### 3.4 The pre-flight conflict scan (step 001)

Three invariants become constraints, and a legacy workspace may already violate them. Each follows the precedent already set by `enforce_signer_lease_exclusivity` (`migrations.rs:66`): detect, resolve in the direction that fails safe, write a Critical event naming what changed.

**Duplicate names** — deterministic dedupe by rowid: the first keeps its name, later ones get ` #2`, ` #3`. The original is recorded in the node's revision-1 `spec` and in a `NodeUpdated` event naming both. Leaving duplicates is not an option, because the launch-pack exporter keys by name and silently writes one member's config twice.

**Port collisions on `local`** — all ports inserted in rowid order; a losing insert leaves that node **with its port intact but unreserved**, plus a Critical event and an integrity finding naming both nodes and the port. The unique index still builds. **Nothing is renumbered:** choosing a new port for a running node on the operator's behalf is worse than telling them two nodes are fighting for a socket.

**Nodes whose `args` already contain `--datadir` or `--config`** — not parsed, not backfilled. One Warning event per node naming the flag, and the node page shows *"this node's data directory / config is set through raw arguments; move it into the field so archives and drift checks can see it."* Inferring a path from argv during a migration is exactly the invention this rebuild exists to eliminate.

---
