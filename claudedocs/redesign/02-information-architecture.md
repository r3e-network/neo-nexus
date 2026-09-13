## 4. Information architecture

### 4.1 Navigation

Rendered from the database. `nav::render(active: NavKey, &[NetworkSummary])`.

```
  ⌂  Now                        /
  ▤  Nodes                      /nodes

NETWORKS                                     ← generated: one row per networks row
  ●  Neo N3 MainNet             /networks/neo-n3-mainnet
  ●  Neo X TestNet              /networks/neox-testnet
  ⚠  shibuya-dev                /networks/net-7f2a
     All networks               /networks

OBSERVE
  ◉  Health                     /health
  ⚠  Alerts                     /alerts
  ⏱  Timeline                   /timeline
  ⌸  Logs                       /logs

OPERATE
  ✓  Readiness                  /readiness
  ⚑  Duties                     /duties

SUPPLY
  ▢  Hosts                      /hosts
  ⬡  Runtimes                   /runtimes
  ⤓  Fast-sync archives         /archives
  ⚿  Signers                    /signers

WORKSPACE
  ⚙  Workspace                  /workspace
```

**14 top-level destinations + one generated row per network**, from today's 17 + 2 orphans.

The active network pins open its five sub-destinations:

```
  Neo N3 MainNet                /networks/neo-n3-mainnet
      Overview                  /networks/neo-n3-mainnet
      Governance                /networks/neo-n3-mainnet/governance
      Designations              /networks/neo-n3-mainnet/designations
      Topology                  /networks/neo-n3-mainnet/topology
      Identity                  /networks/neo-n3-mainnet/identity
```

Rules:

- **The dot is the only colour in the chrome**, and it is a function of `node_health_state` for that network's members: grey = not checked, green = all healthy, amber = warning, red = a node needs you, ⚠ = `complete = 0` so Start is blocked.
- **The `Services ▾` menu is deleted**, not regenerated. A second navigation with different labels and coverage — omitting nine destinations, uniquely containing one — is a naming defect with a UI around it (G42).
- **The global `Health 2/2` pill is deleted.** `layout_with_density` has no fleet parameter and cannot honestly render a fleet verdict; the attention queue on `/` is the right widget and a header pill is the wrong one.
- **Section ordering is fixed.** SUPPLY does not float to the top when a workspace is empty; a fresh workspace shows `/`'s empty state with a link. An operator who learns the layout on day one must find it unchanged at 03:00 on day two hundred.
- **Neo X networks render all five sub-destinations.** Governance and Designations render one sentence — *"Committee, candidates and RoleManagement designations are Neo N3 native-contract concepts; Neo X is an EVM sidechain and has none"* — rather than an empty table. The absence is worth teaching once.
- **Breadcrumbs are derived from the path** (`breadcrumb_for(path)` is the only public constructor), express containment, never link to the current page, and are **absent at depth 1**. `CloudWatch / Metrics / All metrics` with both crumbs href-ing `/monitor` (G43) becomes unrepresentable.

### 4.2 Every page

Format: **route · the ONE question · shows · does · does NOT own.**

---

#### `/` — **Now**
> **Does anything need me right now, and which node is it?**

The most important screen in the product. One job: rank.

1. **The sentence.** `3 nodes need attention.` / `Nothing needs attention. 12 nodes healthy, 2 not checked.` / `Observation is off. NeoNexus is not watching any node.` Bound to counts, never a pill.

2. **Attention queue** — `attention_queue(&[NodeVerdict])`. One row per non-healthy node, ordered by the fixed ladder (§4.3). Grouped by network when more than one network has a row.

   ```
   ● seed-03   Stalled (node)    Height frozen at 6,245,100 for 11m 04s while
     neo-cli · Consensus · Neo N3 MainNet · host local      mainnet advanced to 6,245,144.
     Disk is 99.2% full (412 MB free of 50 GB). Peers 8, RPC 3 ms.
     → Free disk space on host local, then restart      [Open seed-03]  [Silence 1h]
   ```

   Four facts in reading order: **which node · what state · what is wrong with numbers · what to do.** Line 3 is `Verdict.suspected_cause` rendered with its measurement — *the verdict tells you disk is at 99.2%, it does not send you to look.* Line 4 is `Verdict.next_action`, which the type guarantees exists (P3).

3. **Chain-stall collapse.** When ≥2 nodes on one `chain_key` are `Stalled` with `StallScope::Chain`, they do **not** produce N rows. They produce one banner:

   ```
   ▲ Neo N3 MainNet has not advanced for 11m. 6 of 6 nodes agree.
     This is the network, not your fleet. Do not restart.        [Open network ▸]
   ```

   This is the most opinionated widget in the design and it is worth the whole reference-head ladder. Six red rows at 03:00 produce six restarts; one banner produces zero, and restarting a validator during a view change is exactly the wrong move.

4. **Not-checked band.** Grey, separate, never mixed with healthy: `rpc_port == 0`, observation disabled, newest sample older than `3 × head_seconds`, host unreachable, fewer than two samples. Each row carries its reason and a link that fixes it. On this page absence has its own real estate — P2 made structural.

5. **Networks strip.** One line per network: head, reference provenance (`fleet median, 4 contributors` / `reference endpoint coz.io` / `no reference — single node`), interval observed vs declared, member count, sample age.

6. **Recent transitions.** Last 10 rows of `node_health_transitions ∪ alarm_transitions ∪ node_designation_transitions`, each linking to `/nodes/{id}?tab=timeline&at={changed_at}`.

**Does:** silence a node or rule for 1h/4h/24h (writes `suppressed_until_unix`, emits an event with an actor, and therefore also pauses the watchdog — one control, one meaning); force an immediate sample (`POST /observe-now`); start/stop/restart from a row's overflow menu with a confirm.
**Does NOT own:** inventory, rule configuration, charts, creation. **There is no "Add node" button and no chart on this page.** Both deliberate: the landing page is a pager.
**Refresh:** 15 s poll via ~40 lines of progressive-enhancement JS with a visible `updated 3s ago` and a Pause control; `<noscript>` falls back to `<meta http-equiv="refresh" content="30">`. Nothing else in the product polls.

---

#### `/nodes` — **Nodes**
> **What do I have, and which of it is not well?**

The inventory, not the incident board. `NodeInventoryFilter` grows from `{status, query}` to `{health, status, duty, client, network_id, host_id, environment_id, tag, owner, q}`, every select rendered from the enum's **full** variant set — which fixes the role filter offering 5 of 8 duties, so State/StateValidator/Notary nodes stop vanishing from every filtered view (G39, G45).

Columns: **Process · Chain · Duty · Node · Network · Host · Client/version · Height · Head lag · Peers · Sampled.** Three badge columns, never fused (P4). Grouped by network by default; `?group=none` flattens. Row click navigates.

**Does:** batch start/stop/restart on the current filter, with a flash that **names every failure** (`Start: 2 of 3 nodes started. seed-03 did not start — no key is bound to it. → Open seed-03`); add node; export the filtered set as configs or IaC; save a filter as a URL.
**Does NOT own:** detail. **The instance drawer is deleted** — its compact-density bug (pinned to `visible.first()` while the operator believes they clicked row B, G47) is a class of defect that exists only because a list page tried to be a detail page.

---

#### `/networks` — **Networks**
> **What chains does this workspace know, and can each produce a bootable config?**

Rows: label, family, kind, identity summary, **`complete`** (the generated column), members, revision, head + provenance. Quarantined rows are grouped with a banner: *"3 private networks were created one-per-node during migration because NeoNexus could not prove they were the same chain."*

**Merge proposals** appear as cards when quarantined networks share observed magic **and** observed genesis **and** mutual peer visibility: *"net-a1b2 and net-c3d4 report the same chain id and genesis, and each appears in the other's peer list. Merge into one network?"* The operator confirms; NeoNexus merges in one transaction and records it in `network_revisions`.

**Does:** create a private network (§4.5); merge quarantined networks; re-seed a locked public row.
**Does NOT own:** governance (that is the network's own page).

---

#### `/networks/{id}` — **network overview**
> **Is this chain healthy, and is my fleet on it healthy?**

`head_strip(&ReferenceHead)` (height, hash, provenance, seconds since the reference advanced, observed interval vs declared `msperblock` with an `(assumed)` marker before identity is sampled) · `fleet_table` worst-first · `standing_strip` (committee seats, validator seats, designations held) · open alarms on this network including grey No-data rows · identity summary with the incomplete-blocks-Start banner · `transition_strip` of the last five chain changes.

**Does:** start/stop/restart scoped to this network; add a node here; refresh governance.
**Does NOT own:** the committee table, the identity editor, per-node diagnosis — one line of each, then a link.

#### `/networks/{id}/governance`
> **Who controls this chain right now, and where does my key stand?**

- **Committee (21)** — rank · public key · validator flag · votes · **This fleet** marker linking to the node; below, *"last changed 2026-09-11 04:20 — 1 in, 1 out"* with an expandable diff.
- **Next-block validators (7)** — a **separate** table. "Committee member" and "produces blocks" are different facts that one 21-row table conflates.
- **Candidates** — top 25 with horizontal rules at the rank 7/8 and rank 21/22 boundaries, fleet keys pinned into view wherever they rank, omitted count stated.
- **Margin** — the operator's actual question as distance to the two boundaries: `+2.1M NEO over rank 8 → you keep producing` · `+18.4M NEO over rank 22 → you keep your seat`.
- **Vote trend** — from `chain_candidate_samples`, always captioned with sample count and start date (*"from 4,032 samples since 2026-08-30"*). Fewer than two samples reads "not enough history". No RPC returns historical vote totals; the chart is honest about being our own record.
- **Block production** — interval median/p95/max vs `msperblock`, throughput vs `maxtransactionsperblock`, primary distribution across the validator set, and — for fleet validators — assigned vs proposed with the unattributed count (§7.1.3).
- **Cadence line** — *"sampled 2 min ago · every 5 min 15 s (21 committee × 15 s blocks) · [Refresh now]"*. Derivation visible, override in `/workspace`.

**Does NOT own any transaction.** No vote button, no `registerCandidate`, no `designateAsRole`. Where an action is needed the page names the transaction, its on-chain price read from `getRegisterPrice`, and who can send it. NeoNexus holds no committee key and must not ask for one.

Two things this page states as **unanswerable**, in one sentence each rather than as a fabricated tile: the current dBFT view number, and consensus peer identity. What it shows instead is §7.1.4.

#### `/networks/{id}/designations`
> **Who holds each on-chain role here, and is one of my keys still among them?**

One row per `ChainRole` — StateValidator, Oracle, **NeoFSAlphabet**, P2PNotary — with holder count, holders, whether a fleet key is among them, last change height and time. **NeoFSAlphabet appears only here**: no `NodeRole` maps to it, so it never reaches a node page or the duty picker.

Below, the journal from `node_designation_transitions`, newest first, revocations in red:

> **seed-02 is no longer designated for Oracle as of height 6,421,905 (observed 03:14:22 UTC).** The process is still running and will stop answering oracle requests. Designation is a committee transaction; NeoNexus cannot restore it.
> [Stop seed-02] [Reassign its duty] [Current holders]

Four states, four renderings, none shared: **designated** · **not designated** · **key unknown** · **could not read the chain**.

#### `/networks/{id}/topology`
> **Are my nodes actually connected to this network, and to each other?**

Per-node peers against a network-derived expectation (private: `members − 1`; public: `expected_peers_floor`, labelled as a policy not a fact), unconnected and bad counts on N3, mempool vs `memorypoolmaxtransactions`. A **fleet cross-reference** matches peer `address:port` against this workspace's own nodes and names which of your nodes can see each other — which on a private network answers the connectivity question completely, and which is also the evidence source for the merge proposal on `/networks`. Declared peers are editable rows (`network_peers`), with one `Enode` parser shared by the form, both generators, both validators and the launch-flag renderer. "Copy all member enodes" emits the set and bulk-inserts each member into the others' rows.

**Does NOT own firewalls.** NeoNexus manages none. The `Inbound Security Group Rules (Firewall Ruleset)` table with `Rule Status: Open` and `0.0.0.0/0` — which contradicted the endpoints card one tab away — is deleted with no replacement (G7).

#### `/networks/{id}/identity`
> **What chain is this, exactly, and will a config render bootably?**

One `attested_row` per identity fact, three columns wide:

| Fact | Declared | From argv | Observed | |
|---|---|---|---|---|
| Network magic | 12345 | `--networkid 12345` | 12345 | Confirmed |
| Genesis hash | 0x2ee5…dbd7 | — | 0x2ee5…dbd7 | Confirmed |
| Seed list | *(empty)* | — | — | **Incomplete** |
| Standby committee | 1 key | — | — | **Incomplete** |
| ms per block | 15000 | — | 15000 | Confirmed |

Agreement values: `Confirmed · DeclaredOnly · Overridden · Mismatch · Unknown`. **`Mismatch` on magic or genesis outranks every other state in the product** — a node that is healthy and on the wrong chain is the most expensive silent failure available. `complete` comes from the generated column and names the exact missing field.

**Does:** edit (unlocked networks), attach a genesis artefact with its sha256, bump the revision with a reason, view `network_revisions`.
**Does NOT own** a node's raw `args` — it reads them for the argv column, via the real flag-**value** extractor (§7.2.2) rather than the presence test that lets one report acknowledge `--networkid 12345` and print "chain id 1230000" in the line above (G22).

---

#### `/health` — **Health**
> **How has each node behaved over time?**

Small multiples over `node_sample_rollups`, one per metric, series per node, `?range=1h|6h|24h|7d|90d` mapped onto the 60 s/300 s/3600 s buckets — **real query parameters with handlers**, replacing bare `<span>`s over a fixed SVG path (G3). Metrics: block height, head lag, height-unchanged seconds, blocks/min vs expected, RPC latency max, peers vs expected, mempool utilisation, process CPU/RSS, disk free. Plus a **health strip** per node: 60 one-minute cells coloured by `node_sample_rollups.worst_state`.

**Gaps are drawn as gaps.** A window with no samples is a visible break with a hover reason, not a straight line between two points. This is where R1 would return if you let it.

CPU/RSS come from a **long-lived collector** held in shared state and refreshed on the supervision tick — which is what makes the ≥50% High-CPU filter match on a saturated host instead of returning empty always (G4). Every consumer's throwaway `MetricsCollector::new(Duration::ZERO)` is deleted; `refresh_if_due` gets its first caller.

**Does NOT own verdicts.** A chart never colours a node; `Healthy`/`Stalled` come from the state machine and appear as the strip.

---

#### `/alerts` — **Alerts**
> **What is firing, what will page me, and where does it go?**

Four tabs: **State · Rules · Routes · Deliveries.**

- **State** — one row per `(rule, node)`: rule, node, network, state, observed value vs threshold, `since`, flapping flag, routes delivered to. `?state=no-data` is a first-class filter and the `NoData` reason renders in full in the neutral badge style. A resolved-in-7-days section below with duration and peak value.
- **Rules** — real rows over `alarm_rules` with metric, comparator, threshold, recovery threshold, `for_seconds`, **scope**, severity, route, enabled, and a roll-up (`4 ok · 1 alarm · 2 no-data`). Create/edit is a form over the enum, not a query language. `Duty(Consensus)` scope is the whole reason the scope column exists: *page on the validator, warn on observers* is two rows.
- **Routes** — many rows with a min severity each, so `Critical → PagerDuty, Warning → Slack` is two rows. **Send test** per route, wiring `preview_alert_route` (implemented, CLI-only).
- **Deliveries** — the send log with response codes.

**Deleted:** `fn active_alarms_table()` and its four permanently-`● OK` rows, the `0 In alarm` / `4 OK` literals, and their mirror on the landing page. `neonexus-signer-lease-expiring` is **not** reseeded: signer bindings have no TTL, the TTL vocabulary is invented (G44), and building an evaluator for a quantity that does not exist is how you get back here.

---

#### `/timeline` — **Timeline**
> **What happened, in order, and who did it?**

Not just `runtime_events`: the merge of `runtime_events`, `node_health_transitions`, `alarm_transitions`, `node_designation_transitions`, `runtime_upgrade_attempts`, `node_config_renders`, `archive_applications` and `node_revisions` into one time-ordered stream. That merge is the point — reconstructing an incident needs *"restart at 03:11, config rendered at 03:11, stalled at 03:14"* as one list.

Filters: **kind** (from `EventKind::ALL`, which today is referenced only from tests — G36), severity, node, host, network, actor, `since`/`until`, free text. Columns: time · severity · **Actor** · node · kind · message · details. Actor renders **—** with help text *"NeoNexus does not record who triggered an event yet"* until `actor_kind` is populated in Stage 5; it is never a grep of the message for `"Hermes"` (G8).

**Does:** permalink a window; export the filtered range, wiring `EventJournalReporter::write`, which already accepts exactly the `RuntimeEventFilter` the page applied.
**Does NOT** claim immutability. The word is banned until a hash chain exists.

---

#### `/logs` — **Logs**
> **What did the process actually say?**

Stdout/stderr for the scoped node, redacted, level/timestamp/message from `--log.format json` where supported, with a time cursor shared with Timeline and Health: arriving with `?at=…&window=15m` from an alarm lands you on the right 15 minutes, not on `tail -n 200`.

**Does NOT own sync progress.** The log-derived `SyncProgress` parsers are deleted from the sync path: neo-rs has no parser, and both Neo X parsers gate on strings the clients never emit (geth emits `Imported new chain segment blocks=`; the parser looks for `"Chain imported"` + `block=`), under a test whose fixtures were written to match the parser (G11). `eth_syncing` and `getblockheadercount` are authoritative and structured. Logs keep what logs are uniquely good at: level, message, and fatal errors from a process that never came up to answer RPC.

---

#### `/readiness` — **Readiness**
> **Which nodes are configured in a way that cannot work — before I start them?**

Per-node × per-check, grouped by network, with the five-value outcome vocabulary: **Pass · Warn · Fail · Not applicable · Not evaluated.** One `checks()` function, one denominator, and a tally that **lists outcomes** (`3 pass · 1 warn · 2 not evaluated`) and never a fraction — `2/2 System & Instance Checks Passed` came from absence counting as a pass (G6).

Lenses: `?check=config-drift` is where `/config` lands — every node whose `node_config_renders` digest no longer matches disk, from the real SHA-256 + semantic `ConfigDriftDetector::check` rather than `Path::is_file()` (G5).

New checks: network identity complete; genesis present and hashing; datadir initialised (Neo X); chain-id agreement across declared/argv/observed; `eth`+`net` present; duty launchable; ports planned and free including authrpc and metrics; **the neo-cli runtime-layout gate**, which today surfaces only at Start (G24).

Every finding carries a `DiagnosticResolution` that routes somewhere that can fix it. Two new resolutions (`Networks → /networks/{id}`, `Peers → /networks/{id}/topology`) and one correction: `RuntimeResolution::RuntimeManager` routes to `/nodes/{id}/edit`, which its own text already names, instead of `/runtimes` (G27).

**Does:** re-run; reconcile a drifted config; **export the readiness report**; **export the support bundle** — CLI-only today, and the operator filing the ticket is the one least likely to have shell access (G30).
**Does NOT own runtime health.** Readiness judges configuration; Health and the node's Health tab judge behaviour. Keeping them apart is what stops "ready" from being read as "working" — the G18 catastrophe, where a node with no seed list and no committee reported "ready, 10 pass, 0 critical".

---

#### `/duties` — **Duties**
> **Which duties can each client actually perform, and which are assigned?**

The client × duty matrix with cells `Full | Caveat{level, reason, remedy} | Unsupported{reason} | Unverified`, **derived** (§7.3), never hand-maintained. Plus per-node assignment with the six-rung ladder and the blocking rung named.

**Does:** assign a duty; unsupported options render `<option disabled>` with the reason as the label, so an operator *learns* why neo-rs cannot be an oracle instead of wondering where the option went. An explicit **"No duty (unassigned)"** exists and is the default; `None` never renders as "Observer".
**Does NOT own** on-chain designation (that is the network page and the node's Duty tab) or the binding itself (that is `/signers`).

---

#### `/hosts` — **Hosts**
> **Where do my nodes run, and can I still reach them?**

Absorbs Federation entirely: a peer workspace is `transport='neonexus-peer'`, and `/hosts?transport=neonexus-peer` is where `/federation` lands. Rows: label, transport, address, reachability from `host_probes` with latency, node count by health, agent version, OS/arch, workspace root. `/hosts/{id}` adds probe history, the **port reservation map** grouped by port, and the runtime installations bound to it.

**Does:** create/edit/delete/enable a host — including peer workspaces, whose `create_remote_server`/`update_`/`delete_` functions exist today and are called only from tests while the page tells the operator to write Rust (G29). Test connection.

---

#### `/runtimes` — **Runtimes**
> **Which client versions can I install, what is installed where, and what did the upgrader do?**

Tabs: **Catalogs** (profiles, with a **default Neo catalog seeded at schema creation** — the seed that breaks G25's circular dependency) · **Releases** (sha256, signature state, install per host) · **Installed** (per host, with the nodes bound to each, and the `runtime_layout` each node uses) · **Plugin packages** (a plugin package is an artifact exactly like a runtime release) · **Upgrades** (`runtime_upgrade_runs`/`_attempts`: per node `from_version → to_version`, the **stage** it failed at, the message, and `from_node_revision` as the rollback target — replacing "batch completed: 0/3 successful" with no reason anywhere in the browser, G31).

**Does:** run an upgrade now; roll a node back to revision *n−1*; edit the policy including `require_signed_catalog`, which is stored, offered in settings, and has no reader in the engine today.

---

#### `/archives` — **Fast-sync archives**
> **Which chain-data archive can I apply to skip initial sync?**

The catalogue with `compatible_entries` filtering actually applied (implemented, zero callers), plus **application history including `target_dir`** — which is how "fast-sync ignored my custom datadir" becomes visible (G40).

States plainly, in one sentence: *"This does not back up a node's chain data. NeoNexus cannot do that."* Neither this page nor `/workspace` implies otherwise. "Snapshot" is retired from the product because its everyday meaning is the inverse of this object's direction — which is how `📸 Create Snapshot Backup` came to point at an import page (G45).

---

#### `/signers` — **Signers**
> **What can sign, and which node is each key bound to?**

Merges `/signer` and `/wallets`: a NEP-6 wallet is *a kind of* signer backend, not a fourth peer concept (G44). Tabs: **Backends · Keys · Bindings.**

- **Backends** gains a create form. Today they exist only as process environment read once at startup, with no insert and no reload (G28).
- **Keys** gains a **`key_id` column** — the string the binding form demands as free text and that appears nowhere readable today — plus curve, scheme, public key, state, and the bound node with its network.
- **Bindings** shows node ↔ key with `bound_at` / `bound_by` and one sentence: *"A key can be bound to one node at a time."* No TTL, no renewal, no "Lease Valid" tile.

**Does:** create a backend; import a NEP-6 wallet **with its validation report** (today's web import calls `profile_from_path` only, discards WARNs, and says "imported successfully"); bind via a **curve-filtered picker**, so a Neo N3 secp256r1 key cannot be offered to a Neo X node — rejected at the write, not at Start hours later; rotate.

---

#### `/workspace` — **Workspace**
> **How is this installation configured, and can I get it out and back in?**

Tabs: **Settings** (observation cadences and retention, reference endpoints, supervision defaults, alert defaults, density, vocabulary of the maintenance window) · **Backup** (export **and** import, both wired — today the page is in no nav section, no page links to it, its button posts to a route that does not exist, and there is no import route at all, G26) · **Integrity** (`WorkspaceIntegrityChecker` live, with its required-table list **generated from the same `&[TableDef]` that `create_tables` executes**, ending the two hand-maintained declarations that have already drifted, G41) · **Support bundle** · **API tokens** (`/workspace/api-tokens`).

Integrity surfaces orphans as actionable rows: API tokens scoped to deleted nodes, port reservations held by nothing, signer bindings with no node, archives for networks with no members, **quarantined runtime specs the operator was never told about** — today `quarantined_runtime_spec` is `pub(crate)`, not on `WorkspaceQueries`, and its only caller is the backup exporter, so the operator retypes argv the database is holding (G27).

### 4.3 The attention ladder (fixed, not configurable)

| # | Condition | Severity | Why here |
|---|---|---|---|
| 1 | `ChainIdentityMismatch` (magic or genesis) | Critical | The node is perfectly healthy and completely useless. Nothing else you fix matters. |
| 2 | `DutyState::DesignationRevoked`, or `Elected → CommitteeOnly/NotElected` | Critical | The process is fine and the job is silently dead. Invisible everywhere else. |
| 3 | `DutyState::NotPerforming` (Consensus, slots skipped) | Critical | You are elected, keyed, and not producing. §7.1.3. |
| 4 | `Stalled(Node)` | Critical | The classic failure this redesign exists for. |
| 5 | Process `Running` + `Unreachable` | Critical | Up and not answering — usually a crash loop inside the client. |
| 6 | Process `Failed` | Critical | Down, and NeoNexus knows why. |
| 7 | `Isolated` | Critical | Zero peers. Causes #4 if left. |
| 8 | `Stalled(Chain)` | Warning | **Collapsed to one network banner.** Not your fleet. |
| 9 | `Degraded` | Warning | Latency, partial peers, a failing sample class. |
| 10 | `Syncing`, not catching up | Warning | Behind with no ETA is a real problem wearing a normal-looking word. |
| 11 | `Syncing` with an ETA | Info | Collapsed into `2 nodes syncing (ETA 40m, 3h)`. |
| 12 | Readiness Critical on a stopped node | Warning | It will fail when you start it; better now than at 03:00. |

`Starting` inside its grace window and `Stopped` by operator intent produce **no row**. Stopping a node to work on it must not page the person doing it.

The ladder is not configurable, on purpose: the layout must be the same every time. This is a judgement, and if it is wrong for a fleet, the wrong thing is at the top at 03:00 — accepted, because a scannable unranked list fails worse.

### 4.4 Fate of today's 17 destinations (+2 orphans)

| Today | Fate | Why |
|---|---|---|
| `/` Fleet overview | **Keep, rebuilt → Now** | Five literal tiles (`0 In alarm`, `4 OK`, `Health passed`, `nexus-az-1a`, `Account: 0123-4567-8901`) deleted; tile grid → ranked attention queue. |
| `/nodes` Nodes | **Keep, rebuilt** | Real filter axes (G39); drawer deleted rather than patched (G47). |
| `/monitor` "Health" | **Rename → `/health`, absorbs `/metrics`** | Verdicts move to `/` and the node page; the fabricated chart, selectors and watchdog tile are deleted (G3). Nav label was already "Health"; breadcrumb "CloudWatch / Metrics / All metrics" and title "CloudWatch Metrics & Telemetry" go (G43). |
| `/metrics` Metrics | **Delete, 301 → `/health`** | Same `collect_snapshot` rendered twice, one a raw `<pre>` with no header and strictly less information (G47). |
| `/logs` Logs | **Keep, scoped + time-anchored** | Sync parsing removed (G11). |
| `/operations` Readiness | **Rename → `/readiness`** | Absorbs the config-drift lens. Nav label unchanged; "Systems Manager OpsCenter" chrome deleted. |
| `/events` Events | **Rename + merge → `/timeline`** | Events alone cannot reconstruct an incident; health/alarm/designation transitions must be in the same stream. Gains kind/node/actor/time filters and export (G36, G30). |
| `/alerts` Alerts | **Keep, rebuilt** | Fake alarm table and tiles deleted (G1); real rules with scope and routes (G14). |
| `/federation` Federation | **Merge → `/hosts?transport=neonexus-peer`** | A peer workspace is one host transport, not a parallel universe (G34); gains create/edit/delete (G29). |
| `/roles` **"Private network"** | **Rename → `/duties`** | The nav label named a different feature entirely (G45). The name "Private network" now belongs to the real authoring flow at `/networks/new?kind=private` (G19). Matrix becomes derived (G21). |
| `/runtimes` Runtimes | **Keep, extended** | Absorbs plugin packages and upgrade history; gains catalog create + schema seed (G25, G31). "AMIs & Node Runtime Catalogs" deleted. |
| `/snapshots` "EBS Snapshots" | **Rename → `/archives` "Fast-sync archives"** | The word named the inverse of the thing (G45); gains compatibility filtering and application history (G40). |
| `/plugins` Plugins | **Merge → node Config tab + `/runtimes`** | Per-node plugin state is per-node config; a fleet grid of toggles is an authoring surface pretending to be an overview. |
| `/config` Configuration | **Merge → node Config tab + `/readiness?check=config-drift`** | `● In Sync` from `Path::is_file()` (G5) and the KMS assertions over files holding **plaintext wallet unlock passwords** are deleted; the config bytes belong next to the node that is crash-looping (G32). |
| `/wallets` Wallets | **Merge → `/signers`** | A NEP-6 wallet is a kind of signer backend (G44). Also fixes the light-themed import form inside the dark shell (G43). |
| `/signer` Signer | **Rename → `/signers`, absorbs wallets** | Gains backend create, `key_id` column, curve-filtered binding (G28). Breadcrumb "KMS / Customer managed keys" deleted. |
| `/settings` Settings | **Rename → `/workspace`** | Absorbs `/backup`, integrity, support bundle, API tokens. Today it contains no `<a href>` at all (G42). |
| `/backup` *(orphan)* | **Merge → `/workspace?tab=backup`** | In no nav section, no inbound link, button 404s, no import route (G26). |
| `/settings/api-tokens` *(orphan)* | **Keep as `/workspace/api-tokens`** | Marked Settings active while having no nav entry (G42). |

**New:** `/networks`, `/networks/{id}` (+4 sub-destinations), `/hosts`, `/duties`.
**Net:** 17 visible + 2 orphans → **14 top-level + 5 network sub-destinations**. Four new, five merges, one deletion, six renames.

**Redirects kept (301), permanently:** `/metrics → /health`, `/monitor → /health`, `/events → /timeline`, `/operations → /readiness`, `/snapshots → /archives`, `/federation → /hosts?transport=neonexus-peer`, `/roles → /duties`, `/config → /readiness?check=config-drift`, `/plugins → /nodes`, `/wallets → /signers?tab=wallets`, `/signer → /signers`, `/backup → /workspace?tab=backup`, `/settings → /workspace`, `/settings/api-tokens → /workspace/api-tokens`.

### 4.5 The two flows that must be designed, not fall out of the structure

Three judges, on three different proposals, called the absence of these fatal. They are destinations with a designed sequence, not a side effect of having forms.

#### First run — `/` empty state → add your first node

A fresh workspace has four seeded public networks, one seeded runtime catalog profile, one `local` host, and zero nodes. `/` renders:

> **No nodes yet.** NeoNexus is watching nothing.
> **[Add your first node]** · or **[Create a private network]** if you are standing up a chain of your own.

`/nodes/new` is five steps, each of which can only be wrong in a way the next step catches:

1. **Network** — the four seeded rows plus any private ones. Picking one fixes the family, which fixes step 2. *"A network is a specific Neo chain. MainNet and TestNet are public; a private network is one you run yourself."*
2. **Client** — filtered to the family. Each option shows what it can do here.
3. **Duty** — generated from `duty_support_for_node` (§7.3); unsupported options are `disabled` with the reason as the label. Default is **No duty (unassigned)**.
4. **Runtime** — installed runtimes for this client, or **Install now** from the seeded catalog (download → sha256 verify → unpack → bind), inline. If the duty requires the neo-cli per-node layout, this step installs `runtime_layout = 'per-node'` (§7.3.1). This is the step that today has no supported path at all (G25).
5. **Host, ports, data directory** — `local`, ports planned against `host_port_reservations`, data dir defaulted and shown. A signer step appears only when `requires_signer()`.

Then: readiness runs, and Start is offered only if no Critical finding remains.

#### Private network authoring — `/networks/new?kind=private`

This is `PrivateNetworkPlanner`'s first caller (G19) and it is a planner, not a form. Six steps, with teaching copy at each — the newcomer's actual difficulty is not clicking, it is not knowing what a standby committee is.

1. **Name and family.**
2. **Chain identity.** For N3: network magic (offered as a random unused value, with *"the magic number keeps your chain's messages from being accepted by any other Neo network — it must be unique and it must match on every member"*), `msperblock`, `maxtransactionsperblock`, `memorypoolmaxtransactions`. For Neo X: chain id (checked against `PUBLIC_CHAIN_IDS = {47763, 12227332, 1, 11155111, 17000}`, because a private chain id colliding with a public one means replayable transactions), block period, and a **genesis file upload** — NeoNexus refuses to invent a Neo X genesis allocation and says so.
3. **Members.** How many nodes, how many of them are validators. The page states the dBFT arithmetic in one line: *"dBFT tolerates f faults with n = 3f + 1 nodes and needs 2f + 1 agreement. 4 validators tolerate 1 fault and need 3 to agree. 7 tolerate 2 and need 5."*
4. **Keys.** For each validator, **generate** a key in a chosen signer backend (this is `--signer-key-generate`'s first web caller) or select an existing one, curve-filtered. The page then states the invariant it is about to enforce: *"StandbyCommittee is exactly the set of validator public keys, in order. NeoNexus derives it from the keys above; you do not type it."*
5. **Hosts and ports.** Member → host assignment, ports allocated across members through the planner against `host_port_reservations`. Seed list is derived as `{host.address}:{p2p_port}` per validator — today it is hardcoded `127.0.0.1:{p2p_port}`, which is wrong the moment a second host exists.
6. **Review and create.** One transaction: the `networks` row (`origin='planned'`, `complete=1` by construction), N `nodes` rows, N `node_duties`, N signer bindings, `network_peers` cross-wired. Then: **Export launch pack** (`PrivateNetworkDeploymentExporter::write`, also first-callered), and `PrivateNetworkMaterialized` / `PrivateNetworkLaunchPackExported` get their first emission sites.

**`complete = 0` cannot be reached from this flow.** The generated column is a safety net for hand-authored and quarantined rows, not the primary teaching mechanism — a red card an hour after you started is a failure report, not a guide.

---

## 5. The node page

`/nodes/{id}?tab=…`. This is where an operator lands from every alarm, every attention row, every chart point.

**Today: eight tabs** (`detail_tabs.rs`) — Details, Status checks, Monitoring, Networking, Security, Storage, Tags, IaC. Four are fabricated in whole or part; one is arithmetic over absence.

**Tomorrow: a permanent header and six tabs.**

### 5.1 The header — not a tab, never scrolls away

```
seed-01                    [Start] [Restart] [Stop] [Sample now] [Silence ▾] [Export ▾]
neo-cli 3.7.4 · Consensus · Neo N3 MainNet · host local · sampled 11s ago

  PROCESS             CHAIN                    DUTY
  ● Running           ● Stalled (node)         ● Elected — rank 6 of 21
  since 4d 02h        for 11m 04s              0 of 14 assigned slots in 500 blocks

  ▸ Height frozen at 6,245,100 for 11m while Neo N3 MainNet is at 6,245,144.
    Disk is 99.2% full (412 MB free of 50 GB).
    → Free disk space on host local, then restart.
```

Three verdicts from three state machines, **always all three, never fused** (P4). The reason sentence and next action come from the same `Verdict` that produced the attention row on `/` — one type, two mounts.

For Consensus-duty nodes, the duty badge carries **rung 6** (production attribution) rather than stopping at "Designated", because "elected, keyed, and producing nothing" is the incident and it must be on the first screen, not one tab down.

**Restart guard.** When the node holds the Consensus duty **and** its network is `Stalled(Chain)` or the current inter-block gap exceeds `3 × msperblock`, Restart requires a typed confirmation of the node name and shows: *"Neo N3 MainNet has not produced a block for 71 s on a 15 s chain. This is consistent with a view change. Restarting a validator during a view change makes recovery slower."* A duty-aware console that knows you are an elected validator should be the thing that stops you making it worse at 03:00.

### 5.2 Tab 1 — **Health** *(default)*

> **What is wrong, and what is the evidence?**

- **Verdict + evidence.** `Verdict.evidence` as a definition list: `head_lag 44`, `height_unchanged_secs 664`, `chain_lag 671`, `peers 8 (expected 3)`, `latency 3.4 ms`, `disk_free 412 MB (0.8%)`. Every row names the method and field it came from. Nothing here renders without a sample, because `Evidence` can only be minted by the sampler.
- **Health strip** — 60 one-minute cells from `node_sample_rollups(60)` with 1h/6h/24h/7d as real query parameters. This replaces the fixed SVG path claiming to be a 60-minute chart.
- **Observation panel** — every field an `Observation<T>` with three renderings (a value, *not sampled*, *not supported by this client*) and never a zero: height, header height / `eth_syncing`, head lag **with reference provenance named**, chain lag, seconds since height change, blocks/min vs expected, sync ETA (or *"not catching up"*), latency, peers connected/unconnected/bad vs expected, mempool vs capacity, sample age, sampling interval and any backoff.
- **Chain identity** — reported vs configured magic/chain id and genesis, as `Attested<T>`. `Mismatch` is this page's top-ranked Critical. One field of one call, and it catches the private-node-dialling-mainnet-seeds catastrophe from outside the config generator entirely.
- **This node's alarms**, with `NoData` reasons visible.

### 5.3 Tab 2 — **Duty**

> **Is the thing this node exists for actually happening?**

The six-rung ladder. Each rung is **Pass / Blocked / Unknown**; each blocked rung carries its concrete reason and exactly one link that can unblock it.

```
1 Assigned      ✓  Oracle
2 Producible    ✓  neo-cli emits an OracleService config block for this duty
3 Enabled       ✗  OracleService AutoStart = false, Nodes = []          → Edit config
                   this node will not start the service even if designated
4 Keyed         ✓  03a1…7f2c  via signer binding kms-local / oracle-key-1 (secp256r1)
5 Designated    ✓  designated since 2026-07-12 09:14 (1 of 7 holders)
6 Performing    ?  not observable over JSON-RPC for this duty — levels 1-5 reported
```

**This pairing is the point.** Without it the console reports a correctly designated neo-cli oracle that has never answered a request, which is the current state (G20). Rung 6 is honest about being unreachable for Oracle and Notary; it is defined and bound for Consensus and partially for StateValidator (§7.1.3).

For Consensus, the panel expands into standing (committee rank, validator membership, margin to rank 8 and rank 22, vote trend with sample count), production (assigned vs proposed with unattributed count, seconds since last proposal, interval on your turn), and the derived view number labelled as an inference.

Custody lives here, because custody exists *for* a duty: backend, **key id**, public key, curve, scheme, bound at, bound by, and one sentence — *"A key can be bound to one node at a time."* For duties with `requires_signer() == false`: *"No signer required for this duty"*, not an empty custody panel with a green Lease Valid tile.

Secondary duties render as collapsed sections of identical shape.

### 5.4 Tab 3 — **Config**

> **What exactly would this node be started with, and does disk agree?**

- **The rendered managed config inline**, redacted through the same `redact_sensitive_text` helper `/logs` already uses, with a live SHA-256 diff against disk and against the previous render. This is real drift, replacing `● In Sync` derived from `Path::is_file()` — and it is the thing an operator needs most in a crash loop and cannot currently read at all (G32).
- **Every sidecar** with its own digest.
- **Resolved launch argv** exactly as it would execute, each flag tagged *managed* / *operator-supplied* / *overrides a managed value*, with the flag-**value** extractor showing which `--datadir` won.
- **Data directory** (resolved: argv → column → default), storage engine, free space. No volume ids, no IOPS, no `/dev/xvda`.
- **Ports** table: rpc · p2p · ws · metrics · authrpc, each **reserved** vs **listening** (a TCP connect on the probe tick). A ws port that is reserved and never opened reads *"reserved, not listening"* instead of a green `Open` badge (G23).
- **Plugins** for this node with drift against `listplugins` — the `/plugins` destination folds in here.
- **Static and trusted peers** — editable rows.
- **Node-scoped readiness findings**, inline, each with a resolution that routes.
- **Config render history** with purpose and digest, which is what makes the Start↔launch-pack byte-parity assertion testable.
- **Permissions line**, replacing the deleted KMS assertions: *"Written 0600. Not encrypted at rest. These files contain the wallet unlock password in plaintext — exclude the workspace folder from unencrypted backups."*

**One write surface.** A tab-level invariant, testable: no `<form>` element may appear in the Health, Duty, Process, Timeline or Logs renderers other than the shared action bar.

### 5.5 Tab 4 — **Process**

> **What does this machine say?**

Supervisor truth only: status, pid, started at, uptime, last exit code, exit history, restart attempts against the **effective** watchdog policy, and the **per-node supervision override** including `paused_until` — "stop relaunching the node I am editing", which today requires disabling automatic restart fleet-wide (G33). Real per-process CPU and RSS from the long-lived collector, disk free and total, a log tail, smoke test.

### 5.6 Tab 5 — **Timeline**

> **What happened to this node, in order?**

`/timeline?node={id}` rendered inline — same function, same filters, pre-scoped. Health transitions, alarm transitions, designation changes, starts/stops/restarts with actor, config renders with digest, upgrade attempts with stage and message, archive applications with target dir, node revisions with what changed and who changed it. "Open full timeline →" leaves with the scope preserved.

There is one renderer and two mounts, so there is no per-node reimplementation to drift and no second query-parameter name to get wrong.

### 5.7 Tab 6 — **Logs**

> **What did the process say?**

`/logs?node={id}` inline, with a quick-range control anchored on transitions: `Since last transition · Last 15m · Around {at}`. Arriving from an alarm at 03:14 opens the log window around 03:14.

### 5.8 Deleted tabs, named

| Today | Fate |
|---|---|
| **Details** | → header + Config. It was a read-only echo of the edit form. |
| **Status checks** | → header verdicts + Health evidence; the fleet view is `/readiness`. The `2/2 passed` arithmetic dies with it, and with it the `rpc_port == 0` green (G6). |
| **Monitoring** | **Deleted entirely.** `1.2% (Active)` / `64.5 MB` / `3.2 ms` selected only by `is_running()`; a literal sparkline path; `Watchdog Armed (5 retries/60m)` contradicting the real editable policy; four unconditional `● OK` cards including `● OK (Lease Valid)` on a function that receives no signer argument, so it reads green on a Consensus node the same page flags `requires Signer Lease` seventy lines earlier (G2). Its honest counterparts are Health and Process; RPC latency becomes a measured field. |
| **Networking** | Security-group table **deleted with no replacement** — NeoNexus manages no firewall. Listening addresses → Config. |
| **Security** | → Duty (custody) + Config. |
| **Storage** | `vol-…` / `/dev/xvda (Root)` / `3000 IOPS (gp3)` / `Attached` / the WAL integrity guarantee → **deleted**. Path + engine → Config; free space → Process. |
| **Tags** | **Deleted, not emptied with a promise.** `Environment / Production` hardcoded on every node including testnet, read-only, under a "cost allocation and access control" claim, goes with it. Tags return as a Config field in Stage 5 when `node_tags` exists. |
| **IaC** | → an **action** (`Export ▾` with a format picker), not a tab. A format is a button. |

**The retire path** is a confirm dialog on Delete, not a tab, and it enumerates consequences **from data**: ports released (named), API tokens revoked (named — today left listed as Active forever, G37), binding released, archives kept, config files left at a named path, data dir left at a named path and its size, events retained.

---

## 6. The observation layer

New module `src/observe/`. This is R2, and it is the largest single body of work in the plan. `src/rpc_health/` stays, narrowed to what it is good at: a one-shot liveness probe of a bare endpoint for the CLI and federation. `RpcHealthStatus` stops being the node's health; `observe::HealthState` is.

```
src/observe.rs
src/observe/
  schedule.rs      // due-queue, backoff, concurrency cap
  client.rs        // timed JSON-RPC call, 2 MiB size cap, redaction
  capabilities.rs  // node_rpc_capabilities cache
  sample/{mod,neo_n3,neox}.rs
  derive.rs        // §6.3 formulas — pure
  health.rs        // §6.4 state machine — pure
  reference.rs     // §6.5 reference ladder
  governance.rs    // per-chain committee/validator/designation reader
  consensus.rs     // §7.1.3 block walk
  rollup.rs
  alarm/{model,evaluate,seed}.rs
```

### 6.1 Sample classes

Every sample is attributed to a class with its own cadence, cost budget and failure semantics. **A class that fails does not void the other classes in the same round** — a round with `head_ok = 1, pool_ok = 0` is a good round.

| Class | N3 methods | Neo X methods | Default period |
|---|---|---|---|
| `head` | `getblockcount`, `getblockheadercount` | `eth_blockNumber`, `eth_syncing` | **15 s** |
| `head_time` | `getblockheader(h-1, true)` → `time` (**ms**), `hash` | `eth_getBlockByNumber("latest", false)` → `timestamp` (**s**) | 60 s |
| `peers` | `getconnectioncount` | `net_peerCount` | 15 s |
| `peers_detail` | `getpeers` | — | 300 s |
| `pool` | `getrawmempool(true)` | `txpool_status`, `eth_gasPrice` | 120 s / 60 s |
| `identity` | `getversion` → `protocol.{network,msperblock,validatorscount,memorypoolmaxtransactions}` | `web3_clientVersion`, `eth_chainId` | 900 s |
| `anchor` | `getblockhash 0` | `eth_getBlockByNumber("0x0", false)` | once per process, revalidated 24 h |
| `state` | `getstateheight` | — | 120 s (State/StateValidator duties) |
| `governance` | `getcommittee`, `getnextblockvalidators` | — | 300 s **per chain** |
| `governance_deep` | `getcandidates` | — | 3600 s **per chain** |
| `designation` | `invokefunction` RoleManagement `getDesignatedByRole` | — | 600 s **per chain per role** |
| `consensus` | `getblock(h, 1)` → `primary`, `time`, `nextconsensus` | — | per tick, ≤20 heights (Consensus fleets only) |
| `process` | — (local) | — (local) | 15 s, from the long-lived collector |

**`getversion` is the highest-value slow call in the product and is currently thrown away** — `summarize_version` keeps only `useragent`. Its `protocol` block supplies the block interval, the mempool capacity, the validator count and, critically, the magic the node *actually* joined.

**Reverse the current Neo X mempool preference.** `chain_state/mempool.rs:136-146` tries `eth_getBlockTransactionCountByNumber(["pending"])` first; on geth the `pending` tag forces construction of a pending block — it is the expensive path. Order is `txpool_status` → (only when the `txpool` namespace is off) the block-count form, at the slow cadence, behind a capability flag.

**Per-chain collapse.** `governance`, `governance_deep`, `designation` and `consensus` are **not per node**. A 20-node mainnet fleet issues one `getcommittee` per 5 minutes, not 20. The scheduler elects a **reader**: the healthiest node on that `chain_key` by last sample, preferring one holding the RpcApi duty, with deterministic tiebreak on node id and fallback on failure, and `source_node_id` recorded so the page can say where the answer came from. Per-node facts (is my key on the committee, is my key designated) are computed **locally** by comparing the node's bound public key against the stored key set. This collapses N invokes into 1 and is what makes G13 affordable.

**Capability cache.** A JSON-RPC `-32601` marks `unsupported`; a transport error never does (that is the node being down, not the method being absent). A column fed by an unsupported method is stored `NULL` and renders **"not supported by this client"** — different from both zero and unknown. This is the bug class that made a healthy Neo X node read **Unreachable**.

### 6.2 Scheduling and cost

`ObservationPolicy` is persisted in `settings` under `observation.*` and re-read every tick like the other policies, so a change takes effect without a restart. Fields: per-class seconds (each clamped), `max_probes_per_tick` (1–16, default 4), `probe_timeout_ms` (500–30000, default 3000), `allow_public_reference` (default true, **forced false for private networks**), `enabled`.

A binary heap keyed by `next_due_at` over `(node_id, SampleClass)` jobs plus per-chain jobs. Invariants: never two in-flight requests to the same node; at most `max_probes_per_tick` dispatched per tick; an overrun job is abandoned, not awaited, and its round stored with `head_ok = 0, error_detail = "timeout after 3000ms"`.

**Blocking `ureq` calls must not run on the supervision thread** — a 3 s timeout × 4 nodes stalls restarts and alert routing for 12 s. Bounded pool, dispatch, collect next tick. This replaces `LoopState::probe_rpc_health`, which probes **one node per tick**, so a 20-node fleet gets each node every 20 s at best and a 100-node fleet every 100 s.

Steady state per node at defaults: 4 head + 4 peers + 1 head_time + 0.5 pool + 0.07 identity ≈ **9.6 requests/minute/node**, nearly all O(1) in-memory reads. A 20-node fleet is ~3.2 req/s.

**Backoff.** On consecutive head failures multiply the node's interval ×2 to a cap of ×8 (15 s → 120 s) and skip all non-`head` classes; reset on the first success. A node down for an hour costs 30 requests, not 240, and the operator still sees a fresh "last checked". On `latency_max_ms > 1000` over 5 minutes, double the interval (cap ×4), record `SamplingThrottled` once, show the reason on the node page, restore after 10 minutes below 500 ms. A node under load must not be pushed further by its own manager.

**`rpc_port == 0`** ⇒ `observability = 'process-only'`: no jobs scheduled (zero RPC cost); health `Unknown` with the reason and a `next_action` linking to the editor; Prometheus emits `neonexus_node_running` and `neonexus_node_observable 0` and **no chain series**; every chain alarm for that node is `NoData(SamplingDisabled)`.

### 6.3 Derivations

Pure functions in `src/observe/derive.rs` over a newest-first slice. Every one returns `Option<T>`; `None` propagates and is never coerced.

**Expected block interval** `E` = `observed_ms_per_block / 1000` when identity has been sampled; else 15 (N3) / 5 (Neo X) with `confidence: Assumed`, and the UI says *"assuming 15 s blocks (node has not reported msperblock yet)"*.

**Head lag** = `reference_height − block_height`; `None` when there is no reference (**not `0`**); clamped to 0 within `-2..=0` (propagation jitter); `< -2` sets `AheadOfReference{by}`.

**Chain lag** = `now − head_block_time_unix`, where N3 divides `getblockheader.time` by 1000 and Neo X does not. *This ms/s normalisation gets a dedicated test: a missed division makes every N3 node read as 54 000 years behind.* Head block in the future by >30 s ⇒ `None` + `ClockSuspect`.

**Height-unchanged seconds** — carried forward, not rescanned. On each head sample: height increased ⇒ set `last_height_change_at_unix`; height **decreased** ⇒ emit `HeightRegressed{from,to}` and invalidate window derivations. A regression is a real incident (a deep reorg, a restored archive, a datadir swap) and must not produce a negative rate.

**Blocks per minute** over window W (default 300 s), requiring `t1 - t0 >= W/2` and `h1 >= h0`, else `None`.

**Sync ETA** = `head_lag / (bpm_node − bpm_reference) × 60`, `None` when that difference ≤ 0.05 with reason *"not catching up"*. Never compute from `bpm_node` alone: a node syncing at 60 bpm against a chain producing 4 bpm has a real ETA; one syncing at 4 bpm against 4 bpm has none, and the naive formula confidently predicts the wrong one.

**Expected peers** = `members − 1` on private networks, `expected_peers_floor` on public (default 3, labelled as a policy).

**Mempool utilisation** = `(verified + unverified) / capacity`, `None` when capacity is unknown; `>= 0.5` Elevated, `>= 0.9` Congested — replacing the compile-time 500/2000.

**RPC latency** measured on the class's primary method only, so `neonexus_node_rpc_latency_seconds` means one thing. Window aggregate at the 1-minute rollup is `max`, named `latency_max_ms`, and nothing calls it a percentile — with 4 samples per minute there is no honest p95.

### 6.4 The health state machine

`health::evaluate(HealthInputs) -> Verdict` — pure, no I/O, no clock, no database. An **ordered guard chain, first match wins**, so precedence is part of the definition. This is the antidote to the tautological test at `tests/unit/supervision/tests.rs:167-206`: fixtures become state vectors, not log lines copied out of the parser.

Constants, derived where possible, all fleet-overridable:

```
confirm_n            = 2 evaluations
unreachable_after    = 3 consecutive head failures
reachable_after      = 2 consecutive head successes
starting_grace       = neo-cli 180s, neo-go 60s, neo-rs 60s, neox-geth 120s, neox-rs 120s
stall_seconds        = clamp(networks.stall_multiple * E, 60, 900)   // N3 -> 300s, Neo X -> 100s
sync_enter_lag       = max(10, ceil(120 / E))
sync_exit_lag        = max(2,  ceil(30  / E))
stale_sample_seconds = 3 * head_seconds
latency_degraded_ms  = 1000
disk_critical_pct    = 2.0
```

| # | State | Entry | Exit |
|---|---|---|---|
| 1 | **Stopped** | no process and the operator has not asked it to run | a launch is issued → Starting |
| 2 | **Starting** | spawned, within `starting_grace`, no successful head sample since | first success → re-evaluate from #4; grace expiry → Unreachable |
| 3 | **Unreachable** | expected running and `consecutive_rpc_failures >= 3` | `consecutive_rpc_successes >= 2` |
| 4 | **Unknown** | `rpc_port == 0`; observation disabled; newest sample older than `stale_sample_seconds`; fewer than 2 successful samples | the condition clears |
| 5 | **Isolated** | `peers_connected == Some(0)` for `confirm_n` (only when peers is observable) | `peers >= 1` |
| 6 | **Stalled** | §6.4.1 | any height increase, **immediately** — no confirm delay on recovery |
| 7 | **Syncing** | `eth_syncing != false`, or `head_lag > sync_enter_lag`, or `header_height − block_height > sync_enter_lag`; **and** height is advancing | `head_lag <= sync_exit_lag` for `confirm_n` and `eth_syncing == false` |
| 8 | **Degraded** | reachable and in sync, but latency > 1000 ms for `confirm_n`; or `0 < peers < expected`; or a non-head class erroring (not merely unsupported); or `bpm < expected_bpm / 2`; or `disk_free_percent < disk_critical_pct` | all clear for `confirm_n` |
| 9 | **Healthy** | none of the above | — |

Asymmetric enter/exit lag is the hysteresis: a node does not flip to Syncing because one block arrived late, and does not declare itself caught up until it is genuinely at the head. `Isolated` outranks `Stalled` deliberately — zero peers *causes* the stall and the operator needs the cause. `Stalled` outranks `Syncing` because a node that is behind and not moving is not syncing.

#### 6.4.1 `Stalled` — the case that matters

```rust
fn is_stalled(i: &HealthInputs) -> Option<StallScope> {
    // 1. The node is answering. This is what distinguishes Stalled from Unreachable.
    if !i.latest.head_ok { return None; }
    if i.derived.rpc_latency_ms? > i.policy.probe_timeout_ms as u32 { return None; }

    // 2. Running long enough for "no progress" to mean something.
    if i.state == HealthState::Starting { return None; }
    if i.suppressed_until_unix.is_some_and(|u| u > i.now_unix) { return None; }
    if i.network.on_demand_blocks { return None; }   // a private chain that only blocks on tx

    // 3. Two independent staleness witnesses; either is sufficient.
    let local_stale = i.derived.height_unchanged_secs? >= i.stall_seconds;
    let chain_stale = i.derived.chain_lag_seconds        // None when ClockSuspect
        .is_some_and(|lag| lag >= i.stall_seconds as i64);
    if !(local_stale || chain_stale) { return None; }

    // 4. Confirm, so one long block does not page anyone.
    if i.candidate_streak(HealthState::Stalled) + 1 < i.policy.confirm_n { return None; }

    // 5. Scope: is it this node, or the whole chain?
    Some(match i.reference {
        ReferenceHead::Known { height, seconds_since_reference_advanced, .. }
            if seconds_since_reference_advanced >= i.stall_seconds
               && height <= i.latest.block_height => StallScope::Chain,
        _ => StallScope::Node,
    })
}
```

**Why two witnesses.** `height_unchanged_secs` works with no reference and no clock trust, but cannot tell a stuck node from a halted chain and is blind for the first `stall_seconds` after a fresh start. `chain_lag_seconds` is immediate — a node that starts up and syncs to a three-day-old head is stale on its very first sample — but depends on the local clock, so it is guarded by the 30 s skew check and disabled with a visible `ClockSuspect` flag.

**Cause attribution.** When `Stalled(Node)` fires, the evaluator sets `suspected_cause` from the same round's evidence, in this order: `DiskNearFull` if `disk_free_percent < 5`; `NoPeers` if `peers == 0`; `PeersBelowExpected`; else `Unknown`. The verdict then reads *"Disk is 99.2% full (412 MB free of 50 GB). → Free disk space on host local, then restart"* instead of *"check disk space and peers before restarting"*. The measurement is one column away in the same row; sending the operator to look for it is a design failure, not a limitation.

**Suppression.** `SnapshotApplied` fast-sync and the runtime upgrader both legitimately freeze a node; both set `node_health_state.suppressed_until_unix`, as does an operator silence.

### 6.5 The reference head ladder

Resolved per `chain_key = (family, observed_magic, observed_genesis_hash)`. **The observed magic is part of the key**: a node that joined magic 1 230 000 must never be compared against one on 860 833 102, whatever the configured value says. That guard is what turns G18 from invisible into a loud `WrongChain` finding, and — per §2.2 — it is what lets observation ship before `networks`.

| Rung | Source | Chosen when |
|---|---|---|
| 1 | `Configured { endpoint_label }` | a `reference_endpoints` row for this chain has a sample fresher than `stale_sample_seconds` |
| 2 | `FleetMedian { contributors }` | ≥3 fleet nodes on this `chain_key` that are Healthy or Degraded (not Syncing, not Stalled) within 2 intervals. **Median, not max** — max is one buggy or forked node away from telling the whole fleet it is behind; the median needs a majority to be wrong |
| 2b | `FleetPair` | exactly 2 such nodes; use max, mark low confidence, and the UI says "compared against one other node" |
| 3 | `PublicSeed { host }` | mainnet/testnet only, `allow_public_reference` set, rate-limited to once per 60 s **per chain, never per node**, 5 s timeout, silent failure demotes to rung 2. **Forced off for private networks** — there is no honest public reference for a chain the operator invented, and quietly dialling seed1.neo.org from a private workspace is both wrong and a data-egress surprise |
| 4 | `SelfOnly` | otherwise. `head_lag = None`, rendered *"no reference head — lag cannot be computed (single node on this network)"* with a link to add a reference endpoint |

Rung 4 is usable: `Stalled` still fires from `chain_lag_seconds` and `height_unchanged_secs`, and `Syncing` from `eth_syncing` / the header-block gap. A developer's one-node private chain that stops producing still shows `Stalled`, because its head block timestamp ages.

### 6.6 Duty state — the second axis

Per `(node, duty)`, independent of node health. Collapsing them loses exactly the case G13 describes.

```rust
pub enum DutyState {
    Unknown, NotApplicable,
    Designated, NotDesignated, DesignationRevoked,          // RoleManagement duties
    Elected, CommitteeOnly, NotElected, NotACandidate,      // Consensus
    NotPerforming { slots_assigned: u32, slots_proposed: u32, window: u32 },
    Misconfigured,                                          // duty claimed, launch path cannot perform it
}
```

`NodeRole::designation()` — zero non-test callers today — gets its caller in the designation sampler. `DesignationRevoked` and `Elected → CommitteeOnly → NotElected` are **transitions**, recorded in `node_designation_transitions` / `chain_governance_samples`, and emitted as Critical events for a duty the node is configured to perform.

### 6.7 Alarms

`AlarmState ∈ {NoData(reason), Ok, Pending, Alarm, Suppressed}`. **The `NoData` rule is load-bearing:** `evaluated_at_unix IS NULL` or `datapoints = 0` **must** render as "No data — never evaluated" / "No data — this node has never answered RPC", in the neutral badge style, with the reason visible. Enforced at the type level (`AlarmState` has no `Default`), and by a rendering test asserting that a freshly seeded rule against a freshly created node produces the string "No data" and not "OK" — the direct regression guard for G1.

**Hysteresis and flapping.** Separate enter/exit thresholds (defaults keep a visible gap: head lag enters at 50, clears at 5). Separate durations (`recovery_for_seconds` defaults to 3× `for_seconds` — quick to fire, slow to clear). Flap detection counts `Alarm ⇄ Ok` transitions in a rolling 900 s window; at ≥5, set `flapping`, hold the more severe state, emit **one** `AlarmFlapping`, and stop routing until 900 s of quiet — the alarm still shows as flapping, because suppression is about paging, not hiding. **Stopped nodes** produce `NoData(NodeStopped)`, never `Alarm`.

**Seeded rules**, all `enabled = 0` until reviewed — the four names the fake table advertised, now real, plus five:

| Name | Metric | Enter | For | Clear | Scope | Severity |
|---|---|---|---|---|---|---|
| `neonexus-block-height-stall` | `height-unchanged-secs` ≥ | `20 × E` | 120 s | ≤ `2 × E` | Fleet | Critical |
| `neonexus-head-lag-high` | `head-lag-blocks` ≥ | 50 | 300 s | ≤ 5 | Fleet | Warning |
| `neonexus-peer-count-low` | `peers-connected` ≤ | expected − 1 | 120 s | ≥ expected | Fleet | Warning |
| `neonexus-peers-isolated` | `health-state-is(Isolated)` | — | 60 s | — | Fleet | Critical |
| `neonexus-rpc-latency-high` | `rpc-latency-ms` ≥ | 1000 | 300 s | ≤ 400 | Fleet | Warning |
| `neonexus-disk-low` | `disk-free-percent` ≤ | 5 | 300 s | ≥ 10 | Fleet | Critical |
| `neonexus-validator-slots-skipped` | `consensus-slots-skipped` ≥ | 3 | 0 s | ≤ 0 | Duty(Consensus) | **Critical** |
| `neonexus-validator-not-elected` | `duty-state-is(NotElected)` | — | 0 s | — | Duty(Consensus) | Critical |
| `neonexus-designation-revoked` | `duty-state-is(DesignationRevoked)` | — | 0 s | — | Duty(Oracle/StateValidator/Notary) | Critical |
| `neonexus-chain-identity-mismatch` | `chain-identity-mismatch` | — | 0 s | — | Fleet | Critical |

`neonexus-validator-slots-skipped` is the rule the operator judge said had nothing to fire on. It now does, because §7.1.3 gives it a series.

**Routing.** Alarm transitions are written to `runtime_events` with `node_id` set and new kinds (`AlarmRaised`, `AlarmCleared`, `AlarmFlapping`, `AlarmNoData`), so the existing delivery path carries them with no transport change. `should_route_alert` gains a rule-aware branch: severity floor **and** rule scope. The existing PagerDuty/Opsgenie/Datadog dedup key is extended to `(rule_id, node_id)` so a raise and its clear correlate into one incident.

### 6.8 Self-monitoring, honestly

**NeoNexus cannot page you when NeoNexus is down.** Say it on the page rather than implying otherwise. What it can do:

- `neonexus_observer_last_sample_age_seconds` and `neonexus_observer_scheduler_queue_depth` in the exposition, so an external Prometheus alerts on staleness.
- A built-in `observer-not-sampling` rule (`sample-age-secs ≥ 5 × head_seconds`, Fleet, Critical) that catches a wedged sampler while the process lives.
- **`alert_routes.heartbeat_seconds`**: when set, NeoNexus posts a liveness ping to that route on an interval. The route's other end (PagerDuty heartbeats, Dead Man's Snitch, a Prometheus `absent()`) alarms on its absence. `/alerts` says exactly this in one sentence: *"A heartbeat is how you find out NeoNexus stopped. Nothing inside NeoNexus can tell you that."*

### 6.9 Prometheus

`docs/AGENT_API.md:266/271/276` already publishes `neonexus_node_running`, `neonexus_node_block_height`, `neonexus_node_rpc_latency_seconds` with a `node` label and **zero hits in `src/`** (G15). They are a contract; honour them exactly, and add `node_id` alongside as the stable identity so a rename does not silently split a series.

New families in `src/metrics/prometheus/families/{chain,health,alarms,observer}.rs`, appended by `snapshot_to_text`. Existing workspace/system/process families unchanged.

**Absence rule:** a node with no sample emits **no chain series at all.** Prometheus semantics are that absence and zero differ, and emitting `neonexus_node_block_height … 0` for an unprobed node is the metrics-layer version of `● OK`.

```
neonexus_node_running{node,node_id,type,chain,network}                 0|1
neonexus_node_observable{node,node_id}                                 0|1
neonexus_node_block_height{node,node_id,chain,network}                 # = block_count - 1, asserted by test
neonexus_node_header_height{...}                                       # N3 only
neonexus_node_rpc_latency_seconds{node,node_id}
neonexus_node_rpc_probes_total / _probe_failures_total                 counters
neonexus_node_head_lag_blocks{...}                                     # absent with no reference
neonexus_reference_head_block_height{chain,network,source,contributors}
neonexus_node_chain_lag_seconds / _seconds_since_height_change
neonexus_node_blocks_per_minute / _expected_blocks_per_minute / _sync_eta_seconds
neonexus_node_peers_connected / _peers_expected / _peers_unconnected / _peers_bad
neonexus_node_mempool_transactions{state="verified|unverified"} / _mempool_capacity
neonexus_node_disk_free_bytes / _disk_total_bytes
neonexus_node_sample_age_seconds
neonexus_node_health_state{node,node_id,state}                         # one series per variant, one is 1
neonexus_node_duty_state{node,node_id,duty,state}
neonexus_node_committee_member / _next_validator                       # absent when no key bound
neonexus_node_consensus_slots_assigned_total / _slots_proposed_total   counters
neonexus_chain_committee_size{chain,network}
neonexus_alarm_state{rule,node,severity,state}
neonexus_observer_scheduler_queue_depth / _samples_total{outcome} / _last_sample_age_seconds
```

Cardinality is bounded: ~100 nodes × (9 health + 8 duty + ~22 numeric) + rules × 5 alarm states ≈ 7 700 series. No unbounded label anywhere; `pid` stays on the process family where it belongs.

The `add_chain_labels` helper that exists at `neox_geth_adapter.rs:51` and is never called gets its caller. The two same-named metrics-adapter hierarchies (G16) collapse: the substantive one (1,853 lines, can scrape a node's own `/metrics`) is constructed with a **planned, emitted, reserved** metrics port (§7.2.4) instead of the hardcoded `http://localhost:8546/metrics` — geth's *WebSocket* default, which the repo itself flags — and `:9091`, both discarding `_rpc_port`.

### 6.10 Test obligations

The register's most useful warning is that the sync parsers are covered by a test whose fixtures were written to match the parser, which is why CI is blind to their being wrong. Counter-measures, all mandatory:

1. **State-machine tests are vectors, not recordings.** Required cases: answers-but-frozen; frozen-chain vs frozen-node; zero peers with a moving height; ahead of reference; clock skew; height regression; fresh node inside grace; `rpc_port == 0`; stale sample; suppressed node; disk-critical attribution.
2. **Fixtures come from the clients.** JSON-RPC and log fixtures are captured verbatim from real neo-cli, neo-go, neo-rs, geth and reth binaries, checked in under `tests/fixtures/{rpc,logs}/`, each carrying a `_provenance` header naming client, version, network and capture date. **A fixture without one fails the test that loads it.**
3. **Unit normalisation.** An explicit test that an N3 `getblockheader.time` of `1_700_000_000_000` and a Neo X `timestamp` of `0x6553f100` land in the same seconds domain.
4. **No-data rendering.** Asserted on the HTML, not the model.
5. **Retention arithmetic.** 48 h of synthetic samples → rollup + prune → exact surviving row counts per tier, no double-counted bucket.
6. **Prometheus absence.** A node with no samples emits `_running` and `_observable` and **zero** chain series — asserted by absence, not by value.

---
