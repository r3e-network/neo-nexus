# NeoNexus — The Redesign

> **Status:** definitive. This document replaces the four competing proposals and the five subsystem specifications as the thing you build from. Where they disagreed, the call is made here and the reason is given.
>
> **Grounded in:** `claudedocs/NEONEXUS_GAP_REGISTER.md` (47 defects, 3 root causes — treated as established fact), `src/web/nav.rs`, `src/web/router.rs`, `src/web/pages/nodes/detail_tabs.rs`, `src/repository/schema/tables/inventory.rs`, `src/roles/role/model.rs`.
>
> **Audience:** the engineer who will build it. Every file path is repo-relative.

---

## 1. What NeoNexus is

NeoNexus is a node manager for Neo — Neo N3 (neo-cli, neo-go, neo-rs) and Neo X (neox-geth, neox-rs) — that runs a fleet of client processes, writes their configuration, holds their custody bindings, watches the chains they joined, and tells the operator when something is wrong. It is a single Rust binary with three surfaces over one workspace: a server-rendered axum console, a headless CLI with `--json` on every read, and a Prometheus exposition. It supervises processes on the machine it runs on today and is built to reach other machines tomorrow.

**The three questions it exists to answer, in this order:**

1. **Is anything wrong, and which node is it?** — the 03:00 question. Not "is the process up", which is a fact about an OS, but "is this node doing the job it is deployed to do on the chain it joined".
2. **What is wrong, and what do I do about it?** — a named state, the numbers that produced it, and one next action that is a link to a page that can change the thing.
3. **Will this node work before I start it?** — configuration, chain identity, duty producibility, custody, ports. Answered *before* launch, not discovered at 03:00.

Everything below is downstream of that ordering. A process-first console can ship forever without answering any of the three. This one cannot render its landing page without answering the first.

### 1.1 What is wrong today, in one paragraph

The console is a visual pastiche of EC2/CloudWatch/SSM/KMS in which values that AWS would bind are interpolated as literals: `fn active_alarms_table() -> String` takes no arguments and every alarm reads `● OK`; `render_tab_monitoring` prints `1.2% (Active)` / `64.5 MB` / `3.2 ms` for every running node; the Health page's "60-minute chart" is a fixed SVG path (**R1**). Beneath it there is no observation layer at all — two JSON-RPC calls, status derived from how many of the two answered, no latency, no peers, no height comparison, no metrics table (**R2**). And the capabilities that *are* complete — `src/chain_state/`, the support bundle, backup import, config drift, workspace integrity, the private-network planner, the snapshot compatibility filter — are each wired to exactly one surface with nothing failing the build when a route has no nav entry or a form posts to no route (**R3**). R1 fills R2's vacuum with green literals; R3 lets both regrow.

---

## 2. Principles

Nine rules. They are stated as rules because their job is to settle future arguments without re-running this analysis. Each names its enforcement mechanism; a principle with no mechanism is a preference.

### P1 — Every page has a subject that is a row, and every render function takes it

`fn active_alarms_table() -> String` is the register's structural tell (G1). The counter-rule: a public render function in `src/web/pages/` may not take only `&WebState`. It takes the thing it renders.

*Enforced by:* a test over `src/web/pages/` asserting no `pub fn render_*`/`pub fn *_table` has a parameter list that excludes its subject type; plus the type rules in P2, which make the violation uninteresting because there is nothing green to render with.

### P2 — Absence has a type, and none of its renderings is green

This is the anti-fabrication rule. It is a type-system property, not a lint, because a lint on the string `● OK` is one refactor away from being evaded.

```rust
/// Minted ONLY by src/observe/sample — from a response the sampler received.
pub struct Evidence {
    pub method: &'static str,      // "getversion"
    pub field:  &'static str,      // "protocol.network"
    pub value:  String,            // as read, not as interpreted
    pub endpoint: String,
    pub sampled_at_unix: u64,
}

pub enum Observation<T> {
    Known(T, Evidence),
    Unknown(NotSampled),              // carries the reason
    Unanswerable(&'static str),       // "neo-go exposes no plugin list"
}

pub enum HealthState { Stopped, Starting, Unreachable, Unknown,
                       Isolated, Stalled, Syncing, Degraded, Healthy }

pub struct Verdict {
    pub state: HealthState,
    pub scope: Option<StallScope>,           // Node | Chain — Stalled only
    pub reason: String,                      // one sentence, contains the numbers
    pub evidence: Vec<Evidence>,             // non-empty for every non-Unknown state
    pub suspected_cause: Option<Cause>,      // DiskNearFull{..} | NoPeers | ClockSkew | ...
    pub next_action: NextStep,               // NOT Option. See P3.
}
```

Hard constraints, each asserted by a compile-time property or a unit test:

- `HealthState` has **no `Default`**, **no `From<bool>`**, and `NodeHealth::from_record` takes `Option<&NodeSample>` — so `is_running()` can produce `Running` (axis A) and can never produce `Healthy` (axis B).
- `ChainFinding::new(..) -> Option<Self>` returns `None` when the evidence vector is empty. There is no other constructor.
- `Evidence` has no public constructor outside `src/observe/sample/`.
- `AlarmState` has no `Default` and its `NoData(NoDataReason)` variant is the value a never-evaluated rule takes.
- Status text is emitted only by `html::status::{process_badge, health_badge, duty_badge, alarm_badge, observation_cell, check_badge}`, each taking a domain enum. There is no `&str` path into a status badge.

*Enforced by:* the type signatures above, plus `src/vocabulary/`'s R-V2 lint (§8.4) as a backstop for prose, plus a test that renders every panel against an empty database and asserts the output contains no status colour other than the Unknown style.

**Corollary C2 — grey is the colour of "we don't know."** Green is only ever the output of a comparison that ran. `detail_tabs.rs:273` grants `🟢 2/2 System & Instance Checks Passed` when `node.rpc_port == 0` — a node that has *never been probed* renders identically to a passing one (G6). Under P2 that node reads **Not checked · RPC is disabled on this node (`rpc_port = 0`); chain state cannot be observed → Set an RPC port**.

### P3 — Every non-healthy verdict carries a next action, and the action is a link

`Verdict.next_action` is `NextStep`, not `Option<NextStep>`. A verdict with no known remediation cannot be constructed, therefore cannot be ranked, therefore cannot reach the attention queue as a dead end.

```rust
pub enum NextStep {
    Here { label: String, href: String },   // a page in this console that can change it
    External { text: String },              // the console cannot; say what can
}
```

`External` is the honest case for designation revocation ("designation is a committee transaction; NeoNexus cannot restore it") and for the neo-cli runtime-layout gate until Stage 6 fixes it. There is no third variant, so "advice with nowhere to go" is unrepresentable.

### P4 — Three axes, never fused

**Process** (what the supervisor sees) · **Chain** (what the chain says when asked) · **Duty** (what RoleManagement or the committee says). Three badges, three columns, three state machines, never one.

`Running` + `Stalled` + `Elected` is a legal and expensive state. It is the failure this product exists to catch and a single fused badge cannot express it. Any widget that collapses them is a bug by definition — including `list.rs:283` and `home.rs:42`, which map `is_running()` straight to `● 2/2 passed`.

### P5 — Thresholds are properties of the chain, not constants in the binary

Block interval from `getversion.protocol.msperblock`. Mempool capacity from `memorypoolmaxtransactions`. Expected peers from the network's own membership. Validator count from `protocol.validatorscount`. Stall window from `20 × msperblock`, clamped.

`classify_connectivity`'s `1..=2 => Sparse` and `classify_congestion`'s `500`/`2000` become functions of observed capacity (G12). A 4-node private network stops reading "Healthy" at 3 peers; 500 transactions stops reading "Elevated" on a chain configured for 5,000-tx blocks.

### P6 — Identity is persisted once and read from one place

Chain identity is a row. `ConfigGenerator::render_for_node` takes `&NetworkProfile`, **not** `Option<&RuntimeConfigProfile>`. Every `effective_*` fallback helper is deleted. There is exactly one loader.

This closes G18/G19/G22 by construction: a `profile: None` launch becomes untypeable, and the three independent re-derivations of `neox_chain_id` collapse into one field read.

### P7 — Scope is a type; links are constructed, never written

```rust
pub struct Scope { pub node: Option<NodeId>, pub network: Option<NetworkId>,
                   pub host: Option<HostId>, pub at: Option<u64>, pub window: Window }
pub fn link_to(key: NavKey, scope: &Scope) -> String;   // the ONLY internal href constructor
pub fn scope_chip(scope: &Scope) -> String;             // rendered by every scoped page
```

One query-parameter vocabulary: `node`, `network`, `host`, `at`, `window`, `state`, `q`. `/events?q={name}` against a handler reading `query` (G46) is unwritable once `link_to` is the only builder.

`at` is the single biggest incident ergonomic in the design and costs one `u64`: clicking "Stalled at 03:14" on Timeline and switching to Logs keeps 03:14.

### P8 — One capability, two surfaces, declared once

Every capability is one row in `CAPABILITIES` carrying a route **and** a CLI command, or an explicit justified `WebOnly(reason)` / `CliOnly(reason)` counted against a checked-in ratchet that may fall and never rise. Five CI gates (§9.3). This is the only durable answer to R3.

### P9 — One concept, one word, declared once in code

`nav::Destination.label` is the nav item, the `<h1>`, the `<title>`, the breadcrumb leaf, and the text of every in-product link. `page_head` and `layout` take a `NavKey`, not two independent `&str`s — which is how `/settings/api-tokens` came to mark Settings active while having no nav entry (G42).

*Enforced by:* the `NavKey` type; R-V14 (nav/title parity) and R-V15 (breadcrumb containment) in §8.4.

---

### 2.1 The three departures from the winning proposal, and why

The network-first IA is the spine of this design: the Network is a first-class entity with its own destinations, every fleet view is network-grouped, and every threshold currently hardcoded moves onto the network row. Three things change.

**(a) Node URLs are flat: `/nodes/{id}`, not `/n/{network}/nodes/{id}`.**
The winning proposal listed this as its own weakness: reassigning a node's network invalidates bookmarks, alert deep links and stored event links, and the permanent resolver redirect means two URL schemes forever. A node's identity is its id; its network is a property that can change. Network-scoped destinations — governance, designations, topology, identity — genuinely *are* properties of the network and nest under `/networks/{id}/…`. Nothing about the spine requires the node URL to nest, and one URL scheme is worth more than the taxonomic tidiness.

**(b) The landing page is the attention queue, not a grid of per-network cards.**
Three judges independently named the ranked *which node · what · what to do* queue as the strongest single widget across all four proposals. It is also what makes the observation layer load-bearing: there is no constant that can fill a row of it. The per-network verdict survives as the grouping of that queue and as `/networks/{id}`.

**(c) Observation ships before the `networks` migration.**
This is the most important change and it directly answers the winner's fatal criticisms. See §2.2.

### 2.2 Why observation comes first — and why that makes the migration safe

The winning proposal put `networks` at Stage 1 and observation at Stage 2. Both an operator judge and an engineer judge called that fatal, and the register's own suggested sequencing agrees: *delete fabrications* (1), *build the observation layer* (2), *make the first run work* (3) — where network identity lives. The observation layer is cheap, decisive, and additive; the `nodes` rebuild is the largest and riskiest change in the plan.

The objection to reordering is that head lag needs a set of nodes on one chain, and the set is the `networks` table. That objection is wrong, and the reason it is wrong is also the fix for the migration:

> **The reference-head grouping key is the *observed* chain key `(family, observed_magic, observed_genesis_hash)`, read from the running node — never the configured `Network` enum.**

`getversion.protocol.network` / `eth_chainId` and `getblockhash 0` / `eth_getBlockByNumber("0x0")` give the chain a node *actually joined*, from outside the config generator entirely. That is more honest than the configured value (it is what detects G18's private-magic node dialling mainnet seeds), it requires no schema for networks, and it is available on the first sample.

And it disarms the engineer judge's fatal criticism of the migration. Their objection: today `nodes.network` is `{Mainnet, Testnet, Private}`, so a migration to `network_id NOT NULL` can only produce one row per `(family, network)` — merging every unrelated private dev chain into one `networks` row, which then computes a fabricated fleet-median head and a fabricated `expected_peers = members − 1`. Correct, and the "same observed magic" filter does not save it, because every private node falls back to magic `1_230_000`.

**The rule, therefore: the migration never merges private networks.**

| Case | Migration produces |
|---|---|
| `network ∈ {mainnet, testnet}` | `network_id = '<family>-<network>'`, one of four seeded rows. Safe: these genuinely are one chain. |
| `network = 'private'` | **One `networks` row per node**, `id = 'net-<node_id>'`, `origin = 'quarantined'`, all identity columns `NULL`, `complete = 0`. |

A quarantined network has one member, so its reference is itself (rendered *"no reference — this is the only node on this network"*, never `0`) and its `expected_peers` is `0`. Nothing is fabricated because nothing is inferred.

Merging is an **operator action** (`POST /networks/merge`), and Stage 2's observation supplies the *evidence* for a proposal rather than a guess. `/networks` shows a "these may be the same chain" card when quarantined networks share **all three** of: observed magic, observed genesis hash, and mutual peer visibility (node A's `getpeers` contains node B's `address:port` — direct evidence, not inference). The operator confirms; NeoNexus merges and records the merge in `network_revisions` with an actor.

That is the whole argument for the ordering: **observation is not blocked by the migration, and observation is what makes the migration safe.**

---
