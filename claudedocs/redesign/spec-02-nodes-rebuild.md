### 005 — `nodes` rebuild

Rebuilt rather than altered, because ALTER TABLE ADD COLUMN cannot add a `REFERENCES` column with a non-NULL default while foreign keys are on, and we need `host_id NOT NULL DEFAULT 'local'`, `network_id NOT NULL`, a `UNIQUE` name, and the removal of `network`. Follows SQLite's documented 12-step procedure with `PRAGMA foreign_keys = OFF` for the duration, so the `REFERENCES nodes(id)` clauses in the child tables are not rewritten by the rename, plus `PRAGMA foreign_key_check` before commit.

```sql
CREATE TABLE nodes_new (
    id                   TEXT    NOT NULL PRIMARY KEY,
    name                 TEXT    NOT NULL,
    host_id              TEXT    NOT NULL DEFAULT 'local'
                                 REFERENCES hosts(id)    ON DELETE RESTRICT,
    network_id           TEXT    NOT NULL
                                 REFERENCES networks(id) ON DELETE RESTRICT,
    node_type            TEXT    NOT NULL,
    origin               TEXT    NOT NULL DEFAULT 'managed',

    binary_path          TEXT    NOT NULL,
    args                 TEXT    NOT NULL DEFAULT '',
    runtime_version      TEXT    NOT NULL DEFAULT 'latest',
    runtime_package_id   TEXT,
    storage_engine       TEXT    NOT NULL DEFAULT 'leveldb',

    data_dir             TEXT,
    config_mode          TEXT    NOT NULL DEFAULT 'managed',
    external_config_path TEXT,

    rpc_port             INTEGER NOT NULL DEFAULT 10332,
    p2p_port             INTEGER NOT NULL DEFAULT 10333,
    ws_port              INTEGER,
    metrics_port         INTEGER,

    environment_id       TEXT    REFERENCES environments(id) ON DELETE SET NULL,
    owner                TEXT    NOT NULL DEFAULT '',

    status               TEXT    NOT NULL,
    pid                  INTEGER,

    revision             INTEGER NOT NULL DEFAULT 1,
    created_at_unix      INTEGER NOT NULL,
    created_at_estimated INTEGER NOT NULL DEFAULT 0,
    updated_at_unix      INTEGER NOT NULL,

    CHECK (origin IN ('managed','mirrored')),
    CHECK (config_mode IN ('managed','external')),
    CHECK (config_mode = 'managed' OR external_config_path IS NOT NULL),
    CHECK (created_at_estimated IN (0,1)),
    CHECK (length(name) BETWEEN 1 AND 120),
    CHECK (rpc_port BETWEEN 1 AND 65535),
    CHECK (p2p_port BETWEEN 1 AND 65535),
    CHECK (ws_port      IS NULL OR ws_port      BETWEEN 1 AND 65535),
    CHECK (metrics_port IS NULL OR metrics_port BETWEEN 1 AND 65535),
    CHECK (rpc_port <> p2p_port),
    CHECK (ws_port IS NULL OR (ws_port <> rpc_port AND ws_port <> p2p_port)),
    CHECK (metrics_port IS NULL OR (metrics_port <> rpc_port
                                AND metrics_port <> p2p_port
                                AND (ws_port IS NULL OR metrics_port <> ws_port))),
    -- A node NeoNexus does not supervise cannot own a local pid.
    CHECK (pid IS NULL OR origin = 'managed')
) STRICT;

INSERT INTO nodes_new (id,name,host_id,network_id,node_type,origin,binary_path,args,
                       runtime_version,runtime_package_id,storage_engine,data_dir,
                       config_mode,external_config_path,rpc_port,p2p_port,ws_port,
                       metrics_port,environment_id,owner,status,pid,revision,
                       created_at_unix,created_at_estimated,updated_at_unix)
SELECT n.id, n.name, 'local',
       :resolved_network_id,          -- see migration table, step 5
       n.node_type, 'managed', n.binary_path, n.args, n.runtime_version,
       (SELECT ri.package_id FROM runtime_installations ri
         WHERE ri.binary_path = n.binary_path LIMIT 1),
       n.storage_engine, NULL, 'managed', NULL,
       n.rpc_port, n.p2p_port, n.ws_port, NULL, NULL, '',
       n.status, n.pid, 1,
       :created_at, :created_at_estimated, :now
FROM nodes n;

DROP TABLE nodes;
ALTER TABLE nodes_new RENAME TO nodes;

-- Name is a join key: the launch-pack exporter keys members by name while
-- uniqueness was checked only at plan time, so duplicates wrote one member's
-- config twice.
CREATE UNIQUE INDEX idx_nodes_name_unique ON nodes (name COLLATE NOCASE);
CREATE INDEX idx_nodes_network     ON nodes (network_id, name COLLATE NOCASE);
CREATE INDEX idx_nodes_host        ON nodes (host_id,    name COLLATE NOCASE);
CREATE INDEX idx_nodes_status      ON nodes (status,     name COLLATE NOCASE);
CREATE INDEX idx_nodes_type        ON nodes (node_type,  name COLLATE NOCASE);
CREATE INDEX idx_nodes_environment ON nodes (environment_id) WHERE environment_id IS NOT NULL;
CREATE INDEX idx_nodes_owner       ON nodes (owner COLLATE NOCASE) WHERE owner <> '';
CREATE INDEX idx_nodes_package     ON nodes (runtime_package_id) WHERE runtime_package_id IS NOT NULL;
```

Port reservation triggers — the collision guard:

```sql
CREATE TRIGGER trg_nodes_ports_ai AFTER INSERT ON nodes BEGIN
    INSERT INTO host_port_reservations (host_id,port,node_id,purpose)
        SELECT NEW.host_id, NEW.rpc_port, NEW.id, 'rpc'     WHERE NEW.origin = 'managed';
    INSERT INTO host_port_reservations (host_id,port,node_id,purpose)
        SELECT NEW.host_id, NEW.p2p_port, NEW.id, 'p2p'     WHERE NEW.origin = 'managed';
    INSERT INTO host_port_reservations (host_id,port,node_id,purpose)
        SELECT NEW.host_id, NEW.ws_port, NEW.id, 'ws'       WHERE NEW.origin = 'managed' AND NEW.ws_port IS NOT NULL;
    INSERT INTO host_port_reservations (host_id,port,node_id,purpose)
        SELECT NEW.host_id, NEW.metrics_port, NEW.id, 'metrics' WHERE NEW.origin = 'managed' AND NEW.metrics_port IS NOT NULL;
END;

CREATE TRIGGER trg_nodes_ports_au
AFTER UPDATE OF host_id, rpc_port, p2p_port, ws_port, metrics_port, origin ON nodes BEGIN
    DELETE FROM host_port_reservations WHERE node_id = OLD.id;
    INSERT INTO host_port_reservations (host_id,port,node_id,purpose)
        SELECT NEW.host_id, NEW.rpc_port, NEW.id, 'rpc'     WHERE NEW.origin = 'managed';
    INSERT INTO host_port_reservations (host_id,port,node_id,purpose)
        SELECT NEW.host_id, NEW.p2p_port, NEW.id, 'p2p'     WHERE NEW.origin = 'managed';
    INSERT INTO host_port_reservations (host_id,port,node_id,purpose)
        SELECT NEW.host_id, NEW.ws_port, NEW.id, 'ws'       WHERE NEW.origin = 'managed' AND NEW.ws_port IS NOT NULL;
    INSERT INTO host_port_reservations (host_id,port,node_id,purpose)
        SELECT NEW.host_id, NEW.metrics_port, NEW.id, 'metrics' WHERE NEW.origin = 'managed' AND NEW.metrics_port IS NOT NULL;
END;
```

Node revision history and its guards:

```sql
CREATE TABLE IF NOT EXISTS node_revisions (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id         TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    revision        INTEGER NOT NULL,
    changed_at_unix INTEGER NOT NULL,
    actor_kind      TEXT    NOT NULL DEFAULT 'unknown',
    actor_id        TEXT,
    reason          TEXT    NOT NULL DEFAULT '',
    spec            TEXT    NOT NULL,     -- full node row as of this revision
    changed_fields  TEXT    NOT NULL DEFAULT '[]',
    event_id        INTEGER REFERENCES runtime_events(id) ON DELETE SET NULL,
    UNIQUE (node_id, revision),
    CHECK (json_valid(spec) AND json_valid(changed_fields)),
    CHECK (actor_kind IN ('operator','cli','api-token','agent','watchdog','supervisor','system','unknown'))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_node_revisions_recent
    ON node_revisions (node_id, changed_at_unix DESC);
CREATE INDEX IF NOT EXISTS idx_node_revisions_at
    ON node_revisions (changed_at_unix DESC);

-- Runtime state (status, pid) is not spec and creates no history.
CREATE TRIGGER trg_nodes_revision_required BEFORE UPDATE ON nodes
WHEN NEW.revision = OLD.revision
 AND (NEW.name                 IS NOT OLD.name
   OR NEW.host_id              IS NOT OLD.host_id
   OR NEW.network_id           IS NOT OLD.network_id
   OR NEW.node_type            IS NOT OLD.node_type
   OR NEW.binary_path          IS NOT OLD.binary_path
   OR NEW.args                 IS NOT OLD.args
   OR NEW.runtime_version      IS NOT OLD.runtime_version
   OR NEW.runtime_package_id   IS NOT OLD.runtime_package_id
   OR NEW.storage_engine       IS NOT OLD.storage_engine
   OR NEW.data_dir             IS NOT OLD.data_dir
   OR NEW.config_mode          IS NOT OLD.config_mode
   OR NEW.external_config_path IS NOT OLD.external_config_path
   OR NEW.rpc_port             IS NOT OLD.rpc_port
   OR NEW.p2p_port             IS NOT OLD.p2p_port
   OR NEW.ws_port              IS NOT OLD.ws_port
   OR NEW.metrics_port         IS NOT OLD.metrics_port
   OR NEW.environment_id       IS NOT OLD.environment_id
   OR NEW.owner                IS NOT OLD.owner)
BEGIN
    SELECT RAISE(ABORT,
      'node spec changed without advancing nodes.revision; write node_revisions first');
END;

CREATE TRIGGER trg_nodes_revision_paired AFTER UPDATE OF revision ON nodes
WHEN (SELECT count(*) FROM node_revisions
       WHERE node_id = NEW.id AND revision = NEW.revision) = 0
BEGIN
    SELECT RAISE(ABORT,
      'nodes.revision advanced with no matching node_revisions row');
END;
```

Duties and static peers:

```sql
CREATE TABLE IF NOT EXISTS node_duties (
    node_id          TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    duty             TEXT    NOT NULL,
    is_primary       INTEGER NOT NULL DEFAULT 0,
    assigned_at_unix INTEGER NOT NULL,
    assigned_by      TEXT    NOT NULL DEFAULT 'system',
    PRIMARY KEY (node_id, duty),
    CHECK (duty IN ('rpc-api','state','indexer','validator','oracle',
                    'state-validator','notary','observer')),
    CHECK (is_primary IN (0,1))
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_node_duties_primary
    ON node_duties (node_id) WHERE is_primary = 1;
-- neo-go's exclusive `match role` over its four signing services is the real
-- single-duty limit; this states it instead of flattening every node to one duty.
CREATE UNIQUE INDEX IF NOT EXISTS idx_node_duties_single_signing
    ON node_duties (node_id)
    WHERE duty IN ('validator','oracle','state-validator','notary');
CREATE INDEX IF NOT EXISTS idx_node_duties_by_duty ON node_duties (duty, node_id);

CREATE TABLE IF NOT EXISTS node_static_peers (
    node_id TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    peer    TEXT NOT NULL,
    kind    TEXT NOT NULL DEFAULT 'static',
    PRIMARY KEY (node_id, peer),
    CHECK (kind IN ('static','trusted','seed')),
    CHECK (length(peer) BETWEEN 3 AND 512)
) STRICT;

CREATE TABLE IF NOT EXISTS node_supervision_overrides (
    node_id              TEXT    NOT NULL PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    restart_enabled      INTEGER,      -- NULL: inherit the workspace policy
    max_restart_attempts INTEGER,
    base_delay_seconds   INTEGER,
    max_delay_seconds    INTEGER,
    jitter_enabled       INTEGER,
    paused_until_unix    INTEGER,      -- "stop relaunching the node I am editing"
    paused_reason        TEXT    NOT NULL DEFAULT '',
    updated_at_unix      INTEGER NOT NULL,
    updated_by           TEXT    NOT NULL DEFAULT 'system',
    CHECK (restart_enabled IS NULL OR restart_enabled IN (0,1)),
    CHECK (jitter_enabled  IS NULL OR jitter_enabled  IN (0,1)),
    CHECK (max_restart_attempts IS NULL OR max_restart_attempts BETWEEN 0 AND 100),
    CHECK (base_delay_seconds IS NULL OR base_delay_seconds BETWEEN 1 AND 3600),
    CHECK (max_delay_seconds  IS NULL OR max_delay_seconds  BETWEEN 1 AND 86400)
) STRICT;
```

### 007 — signer custody

```sql
CREATE TABLE IF NOT EXISTS signer_backends (
    id                TEXT    NOT NULL PRIMARY KEY,
    label             TEXT    NOT NULL,
    kind              TEXT    NOT NULL,
    endpoint          TEXT,
    credential_ref    TEXT,
    wallet_profile_id TEXT    REFERENCES neo_wallet_profiles(id) ON DELETE SET NULL,
    network_id        TEXT    REFERENCES networks(id) ON DELETE RESTRICT,
    source            TEXT    NOT NULL DEFAULT 'workspace',
    enabled           INTEGER NOT NULL DEFAULT 1,
    created_at_unix   INTEGER NOT NULL,
    updated_at_unix   INTEGER NOT NULL,
    CHECK (kind IN ('local-wallet','local-signer','neo-os-service')),
    CHECK (source IN ('workspace','process-environment')),
    CHECK (enabled IN (0,1)),
    CHECK (length(id) BETWEEN 1 AND 128),
    CHECK (kind <> 'local-wallet' OR wallet_profile_id IS NOT NULL),
    CHECK (kind =  'local-wallet' OR endpoint IS NOT NULL)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_signer_backends_label
    ON signer_backends (label COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS signer_keys (
    backend_id         TEXT    NOT NULL REFERENCES signer_backends(id) ON DELETE CASCADE,
    key_id             TEXT    NOT NULL,
    label              TEXT    NOT NULL DEFAULT '',
    curve              TEXT    NOT NULL DEFAULT 'secp256r1',
    public_key         TEXT,
    address            TEXT,
    state              TEXT    NOT NULL DEFAULT 'unknown',
    discovered         INTEGER NOT NULL DEFAULT 0,
    first_seen_at_unix INTEGER NOT NULL,
    last_seen_at_unix  INTEGER,
    PRIMARY KEY (backend_id, key_id),
    -- Neo N3 signs on secp256r1; Neo X on secp256k1. A binding that crosses
    -- them is the "cannot consume a Neo N3 NEP-6 signer profile" dead end,
    -- expressible here instead of at spawn time.
    CHECK (curve IN ('secp256r1','secp256k1')),
    CHECK (state IN ('unknown','active','disabled','missing')),
    CHECK (discovered IN (0,1)),
    CHECK (length(key_id) BETWEEN 1 AND 128)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_signer_keys_public
    ON signer_keys (public_key) WHERE public_key IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_signer_keys_backend
    ON signer_keys (backend_id, label COLLATE NOCASE);
```

`node_signer_bindings` rebuilt (same 12-step procedure), keeping the exclusivity index name so `enforce_signer_lease_exclusivity` continues to apply:

```sql
CREATE TABLE node_signer_bindings_new (
    node_id       TEXT    NOT NULL PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
    backend_id    TEXT    NOT NULL,
    key_id        TEXT    NOT NULL,
    bound_at_unix INTEGER NOT NULL,
    bound_by      TEXT    NOT NULL DEFAULT 'system',
    FOREIGN KEY (backend_id, key_id)
        REFERENCES signer_keys(backend_id, key_id) ON DELETE RESTRICT
) STRICT;

INSERT INTO node_signer_bindings_new (node_id,backend_id,key_id,bound_at_unix,bound_by)
SELECT node_id, backend_id, key_id, :now, 'system' FROM node_signer_bindings;

DROP TABLE node_signer_bindings;
ALTER TABLE node_signer_bindings_new RENAME TO node_signer_bindings;

CREATE UNIQUE INDEX idx_node_signer_bindings_exclusive_lease
    ON node_signer_bindings (backend_id, key_id);
```

### 008 — observation

```sql
CREATE TABLE IF NOT EXISTS node_samples (
    id                     INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id                TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    observed_at_unix       INTEGER NOT NULL,
    endpoint               TEXT    NOT NULL,
    outcome                TEXT    NOT NULL,

    rpc_latency_ms         INTEGER,
    client_version         TEXT,

    block_height           INTEGER,
    header_height          INTEGER,
    best_block_hash        TEXT,
    reference_height       INTEGER,
    head_lag_blocks        INTEGER,
    height_delta           INTEGER,
    height_unchanged_secs  INTEGER,
    syncing                INTEGER,

    peer_count             INTEGER,
    peer_unconnected_count INTEGER,
    peer_bad_count         INTEGER,

    mempool_count          INTEGER,
    mempool_verified_count INTEGER,

    process_cpu_percent    REAL,
    process_memory_bytes   INTEGER,
    disk_free_bytes        INTEGER,

    error_kind             TEXT,
    error_message          TEXT    NOT NULL DEFAULT '',

    -- Every nullable column above means "not measured". Never write 0 to say
    -- "we did not look".
    CHECK (outcome IN ('ok','partial','unreachable','rejected')),
    CHECK (syncing IS NULL OR syncing IN (0,1)),
    CHECK (rpc_latency_ms IS NULL OR rpc_latency_ms >= 0),
    CHECK (block_height   IS NULL OR block_height   >= 0),
    CHECK (peer_count     IS NULL OR peer_count     >= 0),
    CHECK (process_cpu_percent IS NULL OR process_cpu_percent >= 0.0),
    CHECK (error_kind IS NULL OR error_kind IN
           ('transport','timeout','tls','http-status','rpc-error','decode','unsupported-method'))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_node_samples_node_recent
    ON node_samples (node_id, observed_at_unix DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_node_samples_recent
    ON node_samples (observed_at_unix DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_node_samples_stalled
    ON node_samples (node_id, observed_at_unix DESC)
    WHERE height_unchanged_secs IS NOT NULL;

CREATE TABLE IF NOT EXISTS node_sample_rollups (
    node_id            TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    bucket_secs        INTEGER NOT NULL,
    bucket_start_unix  INTEGER NOT NULL,
    samples            INTEGER NOT NULL,
    ok_samples         INTEGER NOT NULL,
    latency_p50_ms     INTEGER,
    latency_p95_ms     INTEGER,
    latency_max_ms     INTEGER,
    block_height_first INTEGER,
    block_height_last  INTEGER,
    head_lag_max       INTEGER,
    peer_count_min     INTEGER,
    peer_count_max     INTEGER,
    cpu_percent_avg    REAL,
    memory_bytes_max   INTEGER,
    PRIMARY KEY (node_id, bucket_secs, bucket_start_unix),
    CHECK (bucket_secs IN (60,300,3600)),
    CHECK (samples >= 1 AND ok_samples BETWEEN 0 AND samples),
    CHECK (bucket_start_unix % bucket_secs = 0)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_node_sample_rollups_window
    ON node_sample_rollups (bucket_secs, bucket_start_unix DESC, node_id);

CREATE TABLE IF NOT EXISTS network_heads (
    network_id       TEXT    NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    observed_at_unix INTEGER NOT NULL,
    height           INTEGER NOT NULL,
    best_block_hash  TEXT,
    -- Provenance is operator-critical: if every node in your fleet is stalled,
    -- fleet-max lag reads zero. The UI must name which reference it used.
    source           TEXT    NOT NULL,
    source_detail    TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY (network_id, observed_at_unix),
    CHECK (source IN ('fleet-max','reference-endpoint')),
    CHECK (height >= 0)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_network_heads_recent
    ON network_heads (network_id, observed_at_unix DESC);
```

### 009 — alarms and routes

```sql
CREATE TABLE IF NOT EXISTS alert_routes (
    id                    TEXT    NOT NULL PRIMARY KEY,
    label                 TEXT    NOT NULL,
    provider              TEXT    NOT NULL,
    target                TEXT    NOT NULL,
    min_severity          TEXT    NOT NULL DEFAULT 'warning',
    timeout_seconds       INTEGER NOT NULL DEFAULT 10,
    enabled               INTEGER NOT NULL DEFAULT 1,
    created_at_unix       INTEGER NOT NULL,
    updated_at_unix       INTEGER NOT NULL,
    last_delivery_at_unix INTEGER,
    last_delivery_status  TEXT,
    CHECK (min_severity IN ('info','warning','critical')),
    CHECK (enabled IN (0,1)),
    CHECK (timeout_seconds BETWEEN 1 AND 120),
    CHECK (length(target) BETWEEN 1 AND 2048)
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_alert_routes_label
    ON alert_routes (label COLLATE NOCASE);

CREATE TABLE IF NOT EXISTS alarm_rules (
    id                     TEXT    NOT NULL PRIMARY KEY,
    name                   TEXT    NOT NULL,
    enabled                INTEGER NOT NULL DEFAULT 1,
    metric                 TEXT    NOT NULL,
    comparator             TEXT    NOT NULL,
    threshold              REAL    NOT NULL,
    evaluation_period_secs INTEGER NOT NULL DEFAULT 60,
    datapoints_evaluated   INTEGER NOT NULL DEFAULT 2,
    datapoints_to_alarm    INTEGER NOT NULL DEFAULT 2,
    missing_data           TEXT    NOT NULL DEFAULT 'insufficient-data',
    selector_kind          TEXT    NOT NULL DEFAULT 'all',
    selector_value         TEXT    NOT NULL DEFAULT '',
    severity               TEXT    NOT NULL DEFAULT 'warning',
    route_id               TEXT    REFERENCES alert_routes(id) ON DELETE SET NULL,
    runbook_url            TEXT,
    description            TEXT    NOT NULL DEFAULT '',
    origin                 TEXT    NOT NULL DEFAULT 'authored',
    created_at_unix        INTEGER NOT NULL,
    updated_at_unix        INTEGER NOT NULL,
    updated_by             TEXT    NOT NULL DEFAULT 'system',

    CHECK (metric IN ('block-height-stall-secs','head-lag-blocks','peer-count',
                      'rpc-latency-ms','rpc-unreachable-secs','mempool-count',
                      'process-cpu-percent','process-memory-bytes','disk-free-bytes',
                      'restart-count','designation-missing','committee-membership-lost')),
    CHECK (comparator IN ('gt','gte','lt','lte','eq','neq')),
    CHECK (missing_data IN ('insufficient-data','breaching','not-breaching','ignore')),
    CHECK (selector_kind IN ('all','node','host','network','environment','duty','tag',
                             'node-type','chain-family')),
    CHECK (severity IN ('info','warning','critical')),
    CHECK (origin IN ('seeded','authored')),
    CHECK (enabled IN (0,1)),
    CHECK (evaluation_period_secs BETWEEN 10 AND 86400),
    CHECK (datapoints_evaluated BETWEEN 1 AND 60),
    CHECK (datapoints_to_alarm BETWEEN 1 AND datapoints_evaluated),
    CHECK (selector_kind = 'all' OR selector_value <> '')
) STRICT;

CREATE UNIQUE INDEX IF NOT EXISTS idx_alarm_rules_name ON alarm_rules (name COLLATE NOCASE);
CREATE INDEX IF NOT EXISTS idx_alarm_rules_enabled ON alarm_rules (enabled DESC, severity, name);

CREATE TABLE IF NOT EXISTS alarm_states (
    rule_id                TEXT    NOT NULL REFERENCES alarm_rules(id) ON DELETE CASCADE,
    node_id                TEXT    NOT NULL REFERENCES nodes(id)       ON DELETE CASCADE,
    state                  TEXT    NOT NULL,
    since_unix             INTEGER NOT NULL,
    last_evaluated_at_unix INTEGER NOT NULL,
    last_value             REAL,
    breaching_datapoints   INTEGER NOT NULL DEFAULT 0,
    reason                 TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY (rule_id, node_id),
    -- 'insufficient-data' is the value whose absence made a never-probed node
    -- indistinguishable from a passing one.
    CHECK (state IN ('ok','alarm','insufficient-data'))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_alarm_states_alarming
    ON alarm_states (since_unix DESC) WHERE state = 'alarm';
CREATE INDEX IF NOT EXISTS idx_alarm_states_node ON alarm_states (node_id, state);

CREATE TABLE IF NOT EXISTS alarm_transitions (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    rule_id         TEXT    NOT NULL REFERENCES alarm_rules(id) ON DELETE CASCADE,
    node_id         TEXT    NOT NULL REFERENCES nodes(id)       ON DELETE CASCADE,
    changed_at_unix INTEGER NOT NULL,
    from_state      TEXT    NOT NULL,
    to_state        TEXT    NOT NULL,
    value           REAL,
    reason          TEXT    NOT NULL DEFAULT '',
    event_id        INTEGER REFERENCES runtime_events(id) ON DELETE SET NULL,
    CHECK (from_state IN ('ok','alarm','insufficient-data')),
    CHECK (to_state   IN ('ok','alarm','insufficient-data')),
    CHECK (from_state <> to_state)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_alarm_transitions_recent
    ON alarm_transitions (changed_at_unix DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_alarm_transitions_node
    ON alarm_transitions (node_id, changed_at_unix DESC);
```

Seeded rules — the four names the fake table advertised, now real, and **disabled** until an operator reviews the thresholds:

```sql
INSERT OR IGNORE INTO alarm_rules
    (id,name,enabled,metric,comparator,threshold,evaluation_period_secs,
     datapoints_evaluated,datapoints_to_alarm,missing_data,selector_kind,
     severity,origin,description,created_at_unix,updated_at_unix) VALUES
 ('block-height-stall','Block height stall',0,'block-height-stall-secs','gte',90,30,2,2,
  'insufficient-data','all','critical','seeded',
  'Chain head has not advanced for the configured interval while the node answers RPC.',:now,:now),
 ('peer-count-low','Peer count low',0,'peer-count','lt',3,60,2,2,
  'insufficient-data','all','warning','seeded',
  'Connected peers below the minimum for two consecutive evaluations.',:now,:now),
 ('head-lag','Chain head lag',0,'head-lag-blocks','gte',5,60,2,2,
  'insufficient-data','all','warning','seeded',
  'Node height trails the reference head. Requires a network reference endpoint.',:now,:now),
 ('rpc-unreachable','RPC unreachable',0,'rpc-unreachable-secs','gte',120,30,2,2,
  'breaching','all','critical','seeded',
  'RPC endpoint has not answered for the configured interval.',:now,:now);
```

`alert_deliveries` gains a route: `ALTER TABLE alert_deliveries ADD COLUMN route_id TEXT;` plus `CREATE INDEX idx_alert_deliveries_route ON alert_deliveries (route_id, attempted_at_unix DESC);`

### 010 — chain state

```sql
CREATE TABLE IF NOT EXISTS node_designations (
    id                  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id             TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    chain_role          TEXT    NOT NULL,
    observed_at_unix    INTEGER NOT NULL,
    observed_height     INTEGER,
    -- NULL means "no key to compare against", which is a different operator
    -- response from "not designated". The CLI collapses both into exit code 1.
    designated          INTEGER,
    designated_keys     TEXT    NOT NULL DEFAULT '[]',
    compared_public_key TEXT,
    query_error         TEXT,
    CHECK (chain_role IN ('state-validator','oracle','neofs-alphabet','p2p-notary')),
    CHECK (designated IS NULL OR designated IN (0,1)),
    CHECK (json_valid(designated_keys) AND json_type(designated_keys) = 'array')
) STRICT;

CREATE INDEX IF NOT EXISTS idx_node_designations_recent
    ON node_designations (node_id, chain_role, observed_at_unix DESC, id DESC);

CREATE TABLE IF NOT EXISTS network_governance_samples (
    id                  INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    network_id          TEXT    NOT NULL REFERENCES networks(id) ON DELETE CASCADE,
    observed_at_unix    INTEGER NOT NULL,
    observed_height     INTEGER,
    committee           TEXT    NOT NULL,
    next_validators     TEXT    NOT NULL,
    candidates          TEXT    NOT NULL,
    committee_hash      TEXT    NOT NULL,
    observed_via_node_id TEXT   REFERENCES nodes(id) ON DELETE SET NULL,
    CHECK (json_valid(committee) AND json_valid(next_validators) AND json_valid(candidates)),
    CHECK (length(committee_hash) = 64)
) STRICT;

-- Append-on-change: one row per distinct committee, so the table is the timeline.
CREATE UNIQUE INDEX IF NOT EXISTS idx_network_governance_change
    ON network_governance_samples (network_id, committee_hash, observed_at_unix);
CREATE INDEX IF NOT EXISTS idx_network_governance_recent
    ON network_governance_samples (network_id, observed_at_unix DESC);
```

### 011 — runtimes

```sql
CREATE TABLE IF NOT EXISTS runtime_releases (
    id                 TEXT    NOT NULL PRIMARY KEY,
    catalog_profile_id TEXT    NOT NULL REFERENCES runtime_catalog_profiles(id) ON DELETE CASCADE,
    package_id         TEXT    NOT NULL,
    node_type          TEXT    NOT NULL,
    version            TEXT    NOT NULL,
    os                 TEXT    NOT NULL,
    arch               TEXT    NOT NULL,
    url                TEXT    NOT NULL,
    sha256             TEXT    NOT NULL,
    bytes              INTEGER NOT NULL,
    signature          TEXT,
    signed_by          TEXT,
    published_at_unix  INTEGER,
    seen_at_unix       INTEGER NOT NULL,
    yanked             INTEGER NOT NULL DEFAULT 0,
    UNIQUE (catalog_profile_id, package_id),
    CHECK (length(sha256) = 64),
    CHECK (yanked IN (0,1)),
    CHECK (bytes > 0)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_runtime_releases_lookup
    ON runtime_releases (node_type, os, arch, version) WHERE yanked = 0;
```

`runtime_installations` rebuilt with a host and a release link:

```sql
CREATE TABLE runtime_installations_new (
    host_id           TEXT    NOT NULL DEFAULT 'local' REFERENCES hosts(id) ON DELETE CASCADE,
    package_id        TEXT    NOT NULL,
    release_id        TEXT    REFERENCES runtime_releases(id) ON DELETE SET NULL,
    label             TEXT    NOT NULL,
    node_type         TEXT    NOT NULL,
    version           TEXT    NOT NULL,
    os                TEXT    NOT NULL,
    arch              TEXT    NOT NULL,
    install_root      TEXT    NOT NULL DEFAULT '',
    binary_path       TEXT    NOT NULL,
    sha256            TEXT    NOT NULL,
    signature_verified INTEGER NOT NULL DEFAULT 0,
    signer_public_key TEXT,
    bytes             INTEGER NOT NULL,
    installed_at_unix INTEGER NOT NULL,
    PRIMARY KEY (host_id, package_id),
    CHECK (signature_verified IN (0,1))
) STRICT;

INSERT INTO runtime_installations_new
    (host_id,package_id,release_id,label,node_type,version,os,arch,install_root,
     binary_path,sha256,signature_verified,signer_public_key,bytes,installed_at_unix)
SELECT 'local',package_id,NULL,label,node_type,version,os,arch,'',
       binary_path,sha256,signature_verified,signer_public_key,bytes,installed_at_unix
FROM runtime_installations;

DROP TABLE runtime_installations;
ALTER TABLE runtime_installations_new RENAME TO runtime_installations;

CREATE INDEX idx_runtime_installations_type
    ON runtime_installations (node_type, version, host_id);
CREATE INDEX idx_runtime_installations_binary
    ON runtime_installations (binary_path);
```

```sql
CREATE TABLE IF NOT EXISTS runtime_upgrade_runs (
    id              TEXT    NOT NULL PRIMARY KEY,
    started_at_unix INTEGER NOT NULL,
    finished_at_unix INTEGER,
    trigger         TEXT    NOT NULL,
    policy_snapshot TEXT    NOT NULL,
    nodes_selected  INTEGER NOT NULL DEFAULT 0,
    nodes_succeeded INTEGER NOT NULL DEFAULT 0,
    nodes_failed    INTEGER NOT NULL DEFAULT 0,
    CHECK (trigger IN ('policy','manual')),
    CHECK (json_valid(policy_snapshot))
) STRICT;

CREATE TABLE IF NOT EXISTS runtime_upgrade_attempts (
    run_id             TEXT    NOT NULL REFERENCES runtime_upgrade_runs(id) ON DELETE CASCADE,
    node_id            TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    attempted_at_unix  INTEGER NOT NULL,
    from_version       TEXT    NOT NULL,
    to_version         TEXT    NOT NULL,
    from_binary_path   TEXT    NOT NULL,
    to_binary_path     TEXT,
    -- The rollback handle update_node never retained.
    from_node_revision INTEGER NOT NULL,
    outcome            TEXT    NOT NULL,
    stage              TEXT    NOT NULL,
    message            TEXT    NOT NULL DEFAULT '',
    PRIMARY KEY (run_id, node_id),
    CHECK (outcome IN ('succeeded','failed','skipped')),
    CHECK (stage IN ('select','download','install','stop','rebind','start','verify','complete'))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_runtime_upgrade_attempts_node
    ON runtime_upgrade_attempts (node_id, attempted_at_unix DESC);
```

### 012 — renders and snapshots

```sql
CREATE TABLE IF NOT EXISTS node_config_renders (
    id               INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id          TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    rendered_at_unix INTEGER NOT NULL,
    purpose          TEXT    NOT NULL,
    target_path      TEXT    NOT NULL,
    node_revision    INTEGER NOT NULL,
    network_revision INTEGER NOT NULL,
    primary_sha256   TEXT    NOT NULL,
    sidecar_sha256   TEXT    NOT NULL DEFAULT '{}',
    bytes            INTEGER NOT NULL,
    CHECK (purpose IN ('launch','export','launch-pack','drift-check')),
    CHECK (length(primary_sha256) = 64),
    CHECK (json_valid(sidecar_sha256) AND json_type(sidecar_sha256) = 'object')
) STRICT;

CREATE INDEX IF NOT EXISTS idx_node_config_renders_recent
    ON node_config_renders (node_id, rendered_at_unix DESC, id DESC);
-- The parity assertion: for one (node_revision, network_revision), 'launch'
-- and 'launch-pack' must agree on primary_sha256.
CREATE INDEX IF NOT EXISTS idx_node_config_renders_parity
    ON node_config_renders (node_id, node_revision, network_revision, purpose);

CREATE TABLE IF NOT EXISTS snapshot_applications (
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    node_id         TEXT    NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    snapshot_id     TEXT    REFERENCES fast_sync_snapshots(id) ON DELETE SET NULL,
    applied_at_unix INTEGER NOT NULL,
    target_dir      TEXT    NOT NULL,
    bytes           INTEGER,
    height_before   INTEGER,
    height_after    INTEGER,
    outcome         TEXT    NOT NULL,
    message         TEXT    NOT NULL DEFAULT '',
    CHECK (outcome IN ('applied','failed','rolled-back'))
) STRICT;

CREATE INDEX IF NOT EXISTS idx_snapshot_applications_node
    ON snapshot_applications (node_id, applied_at_unix DESC);

ALTER TABLE fast_sync_snapshots ADD COLUMN network_id TEXT REFERENCES networks(id);
CREATE INDEX IF NOT EXISTS idx_fast_sync_snapshots_match
    ON fast_sync_snapshots (network_id, node_type);
```

### 013 — events

```sql
ALTER TABLE runtime_events ADD COLUMN actor_kind     TEXT NOT NULL DEFAULT 'unknown';
ALTER TABLE runtime_events ADD COLUMN actor_id       TEXT;
ALTER TABLE runtime_events ADD COLUMN host_id        TEXT;
ALTER TABLE runtime_events ADD COLUMN network_id     TEXT;
ALTER TABLE runtime_events ADD COLUMN correlation_id TEXT;
ALTER TABLE runtime_events ADD COLUMN details        TEXT NOT NULL DEFAULT '{}';

CREATE INDEX IF NOT EXISTS idx_runtime_events_node_recent
    ON runtime_events (node_id, occurred_at_unix DESC, id DESC) WHERE node_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_runtime_events_kind_recent
    ON runtime_events (kind, occurred_at_unix DESC, id DESC);
CREATE INDEX IF NOT EXISTS idx_runtime_events_correlation
    ON runtime_events (correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_runtime_events_actor
    ON runtime_events (actor_kind, occurred_at_unix DESC);
```

`ALTER TABLE` cannot add a `CHECK`, so `actor_kind`'s domain is enforced in Rust (a single `EventActor` type with no free-text constructor) and asserted by the workspace integrity checker: `SELECT count(*) FROM runtime_events WHERE actor_kind NOT IN (…)` must be 0.

### 015 — token scoping

```sql
ALTER TABLE api_tokens ADD COLUMN scope_node_id   TEXT REFERENCES nodes(id) ON DELETE CASCADE;
ALTER TABLE api_tokens ADD COLUMN revoked_at_unix INTEGER;
ALTER TABLE api_tokens ADD COLUMN last_used_at_unix INTEGER;

CREATE INDEX IF NOT EXISTS idx_api_tokens_scope
    ON api_tokens (scope_node_id) WHERE scope_node_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_api_tokens_active
    ON api_tokens (created_at_unix DESC) WHERE revoked_at_unix IS NULL;
```

Deleting a node now removes its scoped token rather than leaving it listed as Active forever.

### 017 — retiring `rpc_health_checks`

One release after 008's dual-write, and only once no writer remains:

```sql
DROP TABLE rpc_health_checks;
CREATE VIEW rpc_health_checks AS
SELECT s.id,
       s.observed_at_unix AS checked_at_unix,
       s.node_id,
       n.name            AS node_name,
       s.endpoint,
       CASE s.outcome WHEN 'ok' THEN 'healthy'
                      WHEN 'partial' THEN 'degraded'
                      ELSE 'unreachable' END AS status,
       s.client_version  AS version,
       s.block_height    AS block_count,
       s.error_message   AS message
FROM node_samples s JOIN nodes n ON n.id = s.node_id;
```

The view keeps every reader compiling and returns richer truth. `delete_node`'s explicit `DELETE FROM rpc_health_checks` is removed — `node_samples` cascades.

### Legacy table dispositions

- `node_roles` — replaced by `node_duties`. Kept for one release as `CREATE VIEW node_roles AS SELECT node_id, duty AS role FROM node_duties WHERE is_primary = 1;`
- `remote_servers`, `remote_server_probe_records` — migrated into `hosts` / `host_probes` and dropped in the same migration that lands the merged Hosts page.
- The five orphaned `signer_*` tables from the custody split stay untouched, as the existing comment in `tables.rs` requires.

---

## 7. Migration

17 steps, each recorded in `schema_migrations`, each idempotent, each leaving the workspace openable by the release that introduced it.

| # | Step | Backfilled from evidence | Defaulted (and flagged) |
|---|---|---|---|
| 000 | `schema_migrations`; record `baseline` = 1 if `nodes` exists | — | — |
| 001 | Pre-flight conflict scan (see below); emit findings, abort on unresolvable | — | — |
| 002 | `hosts`, `host_port_reservations`, `host_probes`; seed `local` | `hosts.workspace_root` from the open workspace path | Every node → `local`. Sound, not assumed: the only spawn is local `Command::new`. |
| 003 | `environments` (seeded), `node_tags` | — | **Nothing.** The hardcoded `Environment / Production` is deleted, not migrated. |
| 004 | `networks`, `network_revisions`; seed 4 public; create legacy private per family in use | Public identity from the shipped constants. Private: whatever the old code actually produced — magic `1230000`, `validators_count 1`, **empty** seeds and committee | Legacy private rows land with `complete = 0` (generated), which blocks Start and raises a Critical readiness finding. Empty is preserved rather than invented. |
| 005 | `nodes` rebuild; `node_revisions`; triggers; `node_duties`; `node_static_peers`; `node_supervision_overrides` | `network_id` from `(nodes.network, node_type.family())`. `runtime_package_id` by matching `binary_path` against `runtime_installations`. `created_at_unix` from the earliest `runtime_events.occurred_at_unix` for that `node_id` (preferring `node-created`) | No event ⇒ `created_at_unix = :now` **and `created_at_estimated = 1`**, so the UI never claims a creation time it invented. `data_dir`, `environment_id`, `owner`, `metrics_port` = NULL / `''`. `config_mode = 'managed'` for everyone. |
| 006 | `node_duties` from `node_roles` | One row per existing `node_roles` row, `is_primary = 1` | A node with no `node_roles` row gets **no duty row** — `None` stays `None`. It is never rendered as "Observer", "Node", "standard" or pre-selected as "rpc-api". |
| 007 | `signer_backends`, `signer_keys`; `node_signer_bindings` rebuild | Backends from `SignerRegistry::from_process_environment()` at first open, `source = 'process-environment'`. Keys from each distinct `(backend_id, key_id)` in existing bindings | Discovered keys get `discovered = 1`, `public_key = NULL`, `state = 'unknown'`, `curve` from the backend's family. A discovered key is rendered as "seen in a binding, never confirmed by the backend". |
| 008 | `node_samples`, `node_sample_rollups`, `network_heads`; dual-write begins | Every `rpc_health_checks` row → a sample: `checked_at_unix`→`observed_at_unix`, `status`→`outcome`, `block_count`→`block_height`, `version`→`client_version`, `message`→`error_message` | `rpc_latency_ms`, `peer_count`, `head_lag_blocks`, CPU, memory = **NULL**, never 0. They were never measured. |
| 009 | `alert_routes`, `alarm_rules` (4 seeded, disabled), `alarm_states`, `alarm_transitions`; `alert_deliveries.route_id` | One route from the five `alert_routing.*` settings keys, labelled "Default route"; existing deliveries point at it | Seeded rules are `enabled = 0`: an alarm the operator has not reviewed must not page at 03:00 on the strength of a migration. |
| 010 | `node_designations`, `network_governance_samples` | — | Empty. Nothing on disk records a past designation, and inventing one is exactly the failure mode being fixed. |
| 011 | `runtime_releases`; `runtime_installations` rebuild; upgrade runs/attempts; **seed the default Neo runtime catalog profile** | Installations → `host_id = 'local'`, `install_root` = parent of `binary_path` | `release_id = NULL` for pre-existing installs. The seeded catalog profile is what breaks the circular first-run dependency. |
| 012 | `node_config_renders`, `snapshot_applications`; `fast_sync_snapshots.network_id` | Snapshot `network` string + `node_type.family()` → `network_id` | No render history exists; the table starts empty and fills on the next Start/export. |
| 013 | `runtime_events` actor columns + indexes | — | `actor_kind = 'unknown'` for every existing row. The `contains("Hermes")` heuristic is deleted, not ported. |
| 014 | `remote_servers` → `hosts`; `remote_server_probe_records` → `host_probes`; drop both | `id`, `label`←`name`, `description`, `enabled`, timestamps, `control_endpoint`←`base_url`, `address` = host component of `base_url`, `service_scheme` from its scheme | `transport = 'neonexus-peer'`, `supervises_processes = 0`. `syncing_nodes` / `total_blocks` / `total_peers` / `public_node_count` are **dropped**, not carried. |
| 015 | `api_tokens` scoping | `scope_node_id` parsed from `permissions` where it already encodes a node namespace; otherwise NULL | NULL = workspace-scoped. |
| 016 | `node_supervision_overrides` | — | Empty; NULL means inherit, so behaviour is unchanged. |
| 017 | Drop `rpc_health_checks`, create the compatibility view | — | — (one release after 008) |

### The pre-flight conflict scan (step 001)

Three invariants are about to become constraints and a legacy workspace may already violate them. Each follows the precedent set by `enforce_signer_lease_exclusivity`: detect, resolve in the direction that fails safe, and write a Critical event naming what changed.

**Duplicate names.** Deterministic dedupe by rowid order: the first keeps its name, later ones get ` #2`, ` #3`. The original is recorded in the node's revision-1 `spec` and in a `NodeUpdated` event with `actor_kind = 'system'` and a message naming both names. Leaving duplicates is not an option, because the launch-pack exporter keys members by name and silently writes one member's config twice.

**Port collisions on `local`.** All node ports are inserted into `host_port_reservations` in rowid order; a losing insert leaves that node **with its port intact but unreserved**, and the migration emits a Critical event plus an integrity finding naming both nodes and the port. The unique index still builds. Nothing is renumbered: choosing a new port for a running node on the operator's behalf is worse than telling them two nodes are fighting for a socket.

**Nodes whose `args` already contain `--datadir` or `--config`.** Not parsed, not backfilled. The migration emits one Warning event per node naming the flag, and the node page shows "this node's data directory / config is set through raw arguments; move it into the field so snapshots and drift checks can see it". Inferring a path from argv during a migration would be exactly the kind of invention the model is being rebuilt to eliminate.

### Backup round-trip

`WorkspaceBackup` gains `hosts: Vec<HostBackup>` and `networks: Vec<NetworkBackup>` (both `#[serde(default)]`), and `NodeBackup` gains `host_id`, `network_id`, `duties: Vec<String>`, `tags`, `environment_id`, `owner`, `data_dir`, `config_mode`, `metrics_port`. `NodeBackup.network` (the legacy string) is still written, derived from `networks.kind`, so an older release can still read a newer backup's nodes.

Import order becomes hosts → networks → runtime/wallet/signer profiles → nodes → per-node children. A backup with `network_id` referencing a network the target workspace lacks **creates it from the embedded `networks[]`** rather than failing or falling back. An old backup with only `network` resolves to the seeded public network, or to the legacy private network for that family — creating it if absent, `complete = 0`.

`WorkspaceBackupImporter` also stops being the sole production writer of half the schema, which was the symptom that made G25 and G29 possible.

---

## 8. Rust-side consequences

- `NodeConfig` drops `network: Network` and gains `host_id`, `network_id`, plus a small resolved `NetworkRef { id, label, kind, family }`. The full `NetworkProfile` is loaded only by the config path.
- `Network` survives as `NetworkKind { Mainnet, Testnet, Private }` on the network row — preserving display strings, backup strings, snapshot matching and the per-kind defaults.
- `RuntimeConfigProfile`, `effective_network_magic`, `effective_seed_nodes`, `effective_validators_count`, `effective_committee_public_keys`, `seed_nodes`, `standby_committee`, `network_magic`, `validators_count`, `neox_chain_id`, `neox_bootnodes`, `neox_genesis_hash`, `neox_reth_chain`, `neox_block_period_secs`, `neox_validator_count` are deleted from the generation path. Their bodies move to `src/config/format/seeds.rs`, reachable only by the network seeder.
- `NodeRole` gains no variants; `NodeRole::designation()` acquires its first non-test caller in the designation poller. `role_availability` is deleted and replaced by `fn duty_is_launchable(node_type, duty, backend_kind, curve) -> DutySupport` derived from the actual launch path, which is what the picker renders.
- `RpcHealthRecord` is replaced by `NodeSample`. `RpcHealthStatus` becomes a derivation over the latest sample plus alarm states, with `Unknown` as a real variant.
- `NodeInventoryFilter` grows to `{ status, node_type, network_id, host_id, environment_id, duty, tag, owner, query }`.
- `RuntimeEventFilter` gains `kind: Option<EventKind>`, `node_id`, `host_id`, `since_unix`, `until_unix` — so `EventKind::ALL`, referenced only from tests today, gets a production caller.
- `chain_state::{peer_telemetry, mempool_telemetry, designation_status, governance_snapshot}` are called by the observation tick with a node id, and their thresholds take values from the `networks` row instead of compile-time constants.
