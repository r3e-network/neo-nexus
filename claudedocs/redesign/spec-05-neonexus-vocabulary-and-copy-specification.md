### NeoNexus Vocabulary and Copy Specification
A canonical vocabulary and copy specification for NeoNexus: one term per concept with exhaustive rejected-synonym lists (node not instance, duty vs designation, signer binding vs key vs wallet, runtime vs release vs binary, fast-sync archive vs workspace backup), a three-test rule that retires every AWS borrowing except CloudFormation-as-export-format, a page-naming rule making the nav label the single declared name rendered as h1/title/breadcrumb-leaf, a three-axis status vocabulary whose defining addition is a grey "Not checked" state, a four-part message shape with before/after rewrites of real strings, and a `src/vocabulary/` lint modelled on the repo's existing `--source-quality` gate with a shrinking allowlist ratchet. It rules against two things the brief provisionally allowed — "instance" and "lease" — on the grounds that the code already names the node and that "lease" imports the TTL/renewal connotation that generated three of the register's fabrications.

> Scope: this document decides **words and copy only**. It does not decide what NeoNexus
> observes (R2) or which surfaces a capability reaches (R3) — but it fixes the names those
> efforts will use, so they are not renamed again afterwards.
>
> Source of truth for the problem: `/Users/jinghuiliao/git/r3e/neo-nexus/claudedocs/NEONEXUS_GAP_REGISTER.md`, Theme 6 (G42–G47), plus the
> naming half of G1–G9, G44–G45.

---

# 0. The one rule everything else follows

**One concept, one word, declared once in code, rendered everywhere from that declaration.**

A word that appears as a string literal in a page is a word that can drift. Every canonical
term in this document therefore has a home: an enum variant's `label()`, a
`nav::Destination.label`, or an entry in the glossary data module (§6). Pages read; pages do
not author. This is what makes §6's lint possible at all — a lint that only greps for bad
words loses to the next author; a lint that forbids *any* literal in the class wins
permanently.

Two corollaries used throughout:

- **C1 — Prefer the compiler to the scanner.** Where a rule can be expressed as a type
  (`page_head(NavKey, …)` instead of `page_head(&str, …)`), express it as a type. Scanners are
  for what types cannot reach: prose inside string literals.
- **C2 — Grey is the colour of "we don't know".** Green is only ever the output of a comparison
  that ran. This is the copy-level statement of the register's first root cause.

---

# 1. Canonical glossary

Format: **Canonical term** — definition — where it lives in code — *Rejected:* synonyms to be
grepped out (with the replacement).

Rejected synonyms are listed exhaustively so §6's scanner can be generated from this section.
Counts in brackets are current occurrences in `src/web/` (case-insensitive, word boundary).

## 1.1 The managed thing

**node** — one Neo client process NeoNexus manages: a row in `nodes`, a `NodeConfig`, a
supervised OS process. Plural "nodes"; the whole set in one workspace is the **fleet**.
Home: `NodeConfig`, `nav::Destination { key: "nodes", label: "Nodes" }`.

*Rejected:* **instance** [118] → node. **EC2 instance** [31] → node. **resource** → node (or
the specific thing). **server** → node when it means a Neo client; **host** when it means the
machine (see §1.5). **workload**, **box**, **VM** → node.

> The register argues "instance" is defensible, and in a greenfield product it would be. It is
> not defensible *here*: the entire domain layer — `NodeConfig`, `NodeStatus`, `NodeType`,
> `NodeRole`, the `nodes` table, every `--node-*` CLI flag, and Neo's own documentation — already
> says node. "Instance" is not a competing concept; it is a second name for a thing that has
> a first name in 100% of the non-presentational code. The code already voted. One name.

**fleet** — every node in this workspace. Home: `/api/fleet`, nav "Fleet overview".
**Kept.** Plain English, accurate, already the API noun. *Rejected:* **estate**, **cluster**
(a cluster is a set of nodes that coordinate; a fleet is a set an operator manages — Neo nodes
in one NeoNexus workspace may be on unrelated networks).

## 1.2 What a node is for

**duty** — the job the operator assigns a node locally: RPC/API, State, Indexer, Consensus,
Oracle, State Validator, Notary, Observer. Home: `NodeRole` (type name to be renamed
`NodeDuty` in stage 4; persist keys pinned — see §7).

**designation** — the on-chain `RoleManagement` grant of a duty to a public key, made by a
committee-witnessed transaction. Home: `ChainRole`, `NodeRole::designation()`.

These are two different facts and must never share a word: a node can hold the Oracle *duty*
and have no Oracle *designation* — that is precisely the failure the register says NeoNexus
cannot report (G13). Copy: "Duty: Oracle · Designation: not designated at height 9,412,004".

*Rejected:* **role** as a bare UI noun — it is ambiguous between the two above and is also the
AWS IAM noun [34 "IAM"]. Use "duty" or "designation". `NodeRole`/`ChainRole` survive as type
names until stage 4; `role` survives as a URL query key (`/roles?node=`) because it is a
persisted key, and persisted keys are frozen (§7). *Rejected:* **job**, **profile** (see §1.4),
**function**, **workload type**.

**Consensus** (duty) vs **validator** (chain status). Neo N3's dBFT 2.0 elects a 21-member
committee by NEO vote; the top 7 are the consensus nodes. So:

- **Consensus** is the *duty* — "this node is configured to run dBFT". Local fact, operator-set.
- **validator** is a *chain status* — "this node's key is currently in the elected top 7".
  Chain fact, vote-set, changes without the operator doing anything.
- **committee member** is the *chain status* for the top 21.

This resolves G45's "Consensus vs ⚡ Validator on adjacent surfaces" by making them two words
for two facts rather than two words for one. Note `NodeRole::Consensus.slug()` is
**`"validator"`** (`src/roles/role/model.rs:48`) — a persisted key that must never reach a
rendered position; the fleet-list filter currently renders it (`list.rs:125 ("validator", "⚡ Validator")`).
Lint rule R-V11 (§6) forbids `slug()` in rendered text.

*Rejected in UI:* **⚡ Validator** as a duty label → "Consensus". **Consensus Role:**
(`list.rs:205`) → "Duty". **Consensus Nodes** as a tile counting *duty* assignment
(`list.rs:243`) → "Consensus duty" (it counts configuration, not election).

**Observer** — a real, separately selectable duty. It is **never** the rendering of "no duty
assigned". No duty assigned renders as **"No duty"** and nothing else. This is G9's fix stated
as vocabulary: `Option<NodeRole>` gets one renderer, `html::duty_label(Option<NodeRole>)`, and
`None => "No duty"`. *Rejected for `None`:* "Observer" [`detail_tabs.rs:28,:720`], "Node"
[`list.rs:297`], "observer" [`iac_spec.rs:44,78,156,212,286`, `hermes_mcp.rs:273`],
"standard" [`api.rs:73`], "rpc-api" [preselection, `presets.rs:98-102`].

## 1.3 Where a node runs

**host** — the machine a node's process runs on. Today there is exactly one, seeded as
`local`. Home: the `hosts` table and `nodes.host_id` proposed by G34.
*Rejected:* **availability zone** [4], **`nexus-az-1a`** [5 surfaces], **region** [8],
**`neo:mesh-1a`**, **`vpc-{network}`** [3], **Account: 0123-4567-8901** — none of these name
anything NeoNexus models. All delete. Where a "where" is genuinely needed today, the answer is
`localhost` and the copy says so.

**peer workspace** — another NeoNexus installation this one reads aggregate counters from.
Home: `RemoteServerProfile`, `/federation`. **"Federation" is kept** as the section name for
NeoNexus↔NeoNexus, and must not be confused with **host** (where *our* nodes run) once G34
lands. *Rejected:* **federation server** → peer workspace (a "server" here is a NeoNexus, not a
node); **remote server** → peer workspace.

## 1.4 Custody: signer, wallet, key, binding

This is the register's worst cluster: one binding wearing five names (G44). Four terms, sharp
edges.

**key** — the custody-side object that can sign: identified by a `key_id` inside a signer
backend. Home: `SignerKeyRef`. *Rejected:* **KMS key** [16 "KMS"], **customer managed key**,
**CMK**, **key material** (use "key"), **credential** (that is an API token, §1.7).

**signer backend** — the system that holds keys: `LocalWallet`, `LocalSigner`, `NeoOsService`.
Home: `SignerBackendKind`, `SignerBackendProfile`.
*Rejected:* **signer profile** → signer backend. **custody provider**, **KMS**,
**key management** → signer backend. **profile** used bare [176 occurrences of "profile" in
`src/web/`] → always qualified: "signer backend", "wallet", "runtime catalog". "Profile" is
banned as a standalone UI noun; the codebase has four unrelated `*Profile` types
(`SignerBackendProfile`, `RuntimeConfigProfile`, wallet profile, catalog profile) and the word
carries no information.

**signer binding** — the exclusive association of one node with one key. Home:
`node_signer_bindings`, unique index at `migrations.rs:112`, enforced at
`signing/isolation.rs:66-72` and `node_signer.rs:160-172`. The binding **is exclusive** — that
is a real, enforced property and the copy should say it: "A key can be bound to one node at a
time."
*Rejected:* **IAM Signer Lease** [`detail_tabs.rs:81`], **3. IAM SIGNER LEASE** [`:348`],
**IAM Signer Role** [`list.rs:204`+`:180`], **IAM Signer** [`list.rs:401`], **IAM Instance
Profile & Signer Identity** [`node_editor/fields.rs:468`], **instance profile**, **signer
identity**, **signer attachment** → all → **signer binding**.

**Ruling on "lease" — retired, against the register's provisional defence.**
The register calls "lease" legitimate because exclusivity is genuinely enforced, and that half
is true. But a lease is not merely exclusive; a lease *expires* and is *renewed*, and nothing
in NeoNexus expires or renews. That surplus connotation is not incidental damage — it is the
generative cause of three separate fabrications: `NeoNode-SignerLease-Expiring` /
`SignerLeaseTTL < 300s threshold remaining` (`alerts.rs:158-165`), the unconditional
`● OK (Lease Valid)` tile on a function that receives no signer argument
(`detail_tabs.rs:537`), and the TTL/AutoRenew vocabulary around them. A word that invents its
own fields is a word that will invent them again. The property we actually have is carried
completely by "the binding is exclusive", in plainer English, at no cost.

*Condition for return:* if a backend is ever integrated that issues genuinely time-bounded
grants (a NeoOS custody session with a server-side expiry), "lease" returns **for that backend
only**, bound to a real `expires_at_unix` column, and never as a synonym for `binding`.

**wallet** — a NEP-6 wallet file (Neo N3). Home: `/wallets`, `SignerBackendKind::LocalWallet`,
`profile_from_path`. A wallet is *a kind of* signer backend; it is not a fourth peer concept
sitting beside signer/key/binding, and the nav must stop implying it is.
Chain-qualified, because Neo X key material is a different object:

- Neo N3 → **NEP-6 wallet** (secp256r1).
- Neo X → **keystore** (secp256k1/EIP-155). Never "wallet" for Neo X.

*Rejected:* **local encrypted wallet** → "NEP-6 wallet"; **wallet profile** → "wallet";
**wallet identity** → "key".

**key id** — the string that names a key inside its backend. It is what the binding form
demands and it currently appears nowhere the operator can read it (`panels.rs:123` has no
key_id column; the detail page shows a truncated *public key* instead). Copy consequence: the
key table gains a "Key id" column and the label for the public key is **"public key"**, never
"id", never "ARN".
*Rejected:* **ARN** [`arn:neo:iam::nexus:operator`, `arn:neo:agent::hermes-ai`] → id.

## 1.5 Software: runtime, release, binary, client

**client** — which implementation: neo-cli, neo-go, neo-rs, neox-geth, neox-reth. Home:
`NodeType` (type name to be renamed `ClientKind`, stage 4; `serde` renames frozen).
*Rejected:* **node type** in UI → client. **flavour**, **engine** (that is `StorageEngine`).

**release** — a versioned, downloadable artifact for a client+platform, listed in a catalog.
Home: `runtime/package/`, `FastSync…`-adjacent catalog types, `/runtimes` catalog flow.

**runtime** — an *installed* release on this host: a directory under
`<workspace>/runtimes/<client>/<version>/<platform>`. Home: `runtime_version`,
`upsert_runtime_installation`, nav "Runtimes".

**binary** — the executable file itself. Home: `binary_path`, `Command::new(&spec.binary_path)`.

Three words because there are three things and the register shows all three failing
separately: the catalog is unreachable (G25, *release*), the upgrader never records the install
(G31, *runtime*), and restore blanks the path (G27, *binary*).

*Rejected:* **AMI** [7] → runtime. **AMIs & Runtimes** / **AMIs & Node Runtime Catalogs**
[`runtimes.rs:116`] → "Runtimes". **image**, **machine image** → runtime. **package** when it
means an installed runtime → runtime (keep "package" only for the downloaded archive, i.e. the
release artifact). **instance type**, **`t3.{node_type}-{role_slug}`** [`detail_tabs.rs:29-33`],
**"⚡ Flavor: 8 vCPU · 32 GB RAM"** [`presets.rs:27,:122`] → **deleted, no replacement**:
NeoNexus allocates no CPU, RAM or IO, and `NewNode` has no such field. **x86_64 Native
Sandbox**, **Hypervisor: NeoNexus Workbench Daemon** [`detail_tabs.rs:144-147`] → deleted; the
honest line is "Runs as a child process of NeoNexus on {host}", with the real
`std::env::consts::ARCH` if an architecture is shown at all.

## 1.6 Chain, network, chain id, tag

**chain** — the protocol: **Neo N3** or **Neo X**. Home: `NodeType::family()`.
**network** — which instance of that chain a node joins: **Mainnet**, **Testnet**, **Private**.
Home: `Network`.
**chain id** — the numeric identity: N3's network magic, Neo X's EIP-155 chain id. Home:
`neox_chain_id`, `network.rs:13`. Always shown with its source ("from managed config",
"from `--networkid`") — this is the copy half of G22, where one report can print
`--networkid 12345` and "chain id 1230000" in the same breath.

*Rejected:* **environment** [7] → **banned outright**. It names nothing NeoNexus models; G7's
hardcoded `Environment / Production` on every node including testnet is the whole of its
current use. **Production**, **Staging**, **Dev** as node attributes → deleted.

**tag** — *reserved, not yet shipped.* An operator-defined `key=value` on a node, for grouping
and for scoping alert rules (G39). The word is reserved here so that when the column lands it
is not named something else. Until the column exists, no page may render a tag — this is the
ruling rule (§2) applied prospectively: the word is fine, the thing does not exist yet.
*Rejected now:* the Tags tab's "cost allocation and access control" claim
[`detail_tabs.rs:745`] and the hardcoded `Environment / Production` row [`:716`] → both delete.
*Rejected as the name:* **label** (taken in code by `NodeRole::label()`, `status.label()`),
**annotation**, **metadata**.

## 1.7 Observation: status, health, readiness, check

Four words, four different questions. "State" is **banned as a UI word for any of them** —
in Neo, "state" means the state root / state service / `NodeRole::State` / StateValidator, and
overloading it is a domain error, not just a style one.

**status** — what the *supervisor* knows about the local OS process. Values in §4.1.
Home: `NodeStatus`.
**health** — what the *chain* says when we ask it. Values in §4.2. Home: `RpcHealthStatus`
(extended).
**readiness** — whether a node's *configuration* would produce a working node, evaluated
before launch. Home: `src/diagnostics/`, nav "Readiness".
**check** — one named, individually-reported test with an outcome. Values in §4.3.

*Rejected:* **state** as a synonym for status/health. **condition**, **posture**, **system
status**, **instance status** → status. **System & Instance Checks** [`detail_tabs.rs:273`] →
"checks". **telemetry** as a page noun → the specific measurement.

**alert** — a notification NeoNexus sends out (Slack, PagerDuty, …). Home: `src/alerts/`,
`list_alert_deliveries`.
**rule** — the condition that produces an alert. Today exactly one exists (a global severity
floor, `AlertRoutingPolicy`), and the copy must say that in one sentence rather than implying
a rule engine.
*Rejected:* **alarm** [`CloudWatch Alarms`, `active_alarms_table`, the four
`NeoNode-*` rows] → alert/rule. **SNS**, **notification topic** → **delivery channel**.
**metric condition** → rule.

**event** — a record of something that happened, in the journal. Home: `RuntimeEvent`,
`EventKind`, nav "Events".
*Rejected:* **CloudTrail** [8], **CloudTrail-grade immutable audit journal**
[`events.rs:77`], **Event History**, **audit journal** as the page name → "Events".
**User Identity** as the actor column head [`events.rs:130-134`] → **"Actor"**, and it renders
**"—"** until `RuntimeEvent` has an actor field. Never a grep of the message text.
The word **immutable** is banned until a hash chain exists (§5, rule M7).

## 1.8 The pair that points in opposite directions

This is the single most damaging naming defect in the register (G45), because the two things
move data in opposite directions and currently share a word and a button.

**fast-sync archive** — **inbound**. A third-party chain-data archive, downloaded and unpacked
*into* a node's data directory to skip initial sync. Home: `FastSyncSnapshot` (type renamed
`FastSyncArchive`, stage 4), `fast_sync_snapshots`, `control/maintenance.rs:47-49`.
Nav label: **"Fast-sync archives"**. Verb: **Apply**.

**workspace backup** — **outbound**. An export of the NeoNexus workspace database — node
definitions, signer backends, wallets, policies, events. Home: `WorkspaceSupportBundleExporter`'s
sibling `WorkspaceBackupImporter` / backup exporter, `/backup`.
Nav label: **"Backup"** under Workspace. Verbs: **Export** / **Import**.

**The word "snapshot" is retired entirely from the product.** Not narrowed — retired. Its
everyday meaning ("capture the current state and save it") is the exact inverse of the object
it names here, which is why someone could write `📸 Create Snapshot Backup`
(`detail_tabs.rs:698`) as a link to an *import* page, and why the register had to spend a
paragraph explaining the direction. A compound that states direction and purpose —
"fast-sync archive" — cannot be misread, and costs two words.

*Rejected:* **snapshot** → fast-sync archive. **EBS Snapshots** [`/snapshots` nav + service
menu] → "Fast-sync archives". **EBS**, **Elastic Block Store** [9] → deleted. **Create Snapshot
Backup** → deleted; the button becomes **"Apply fast-sync archive"** and lives on the node page.
**snapshot** meaning a workspace backup → workspace backup. **restore point**, **checkpoint**
→ fast-sync archive.

*Reserved, does not exist:* **data archive** — an outbound copy of a node's chain data. There is
no such capability. Neither page may imply there is; the honest answer to "can I back up this
node's chain data?" is "not from NeoNexus", and the storage tab should say exactly that rather
than linking to either page.

**support bundle** — a checksummed zip of readiness + integrity + metrics + redacted log
diagnosis, for attaching to a ticket. Home: `WorkspaceSupportBundleExporter::write`. Distinct
from both of the above; keep the name, it is unambiguous.

## 1.9 Storage and network surface

**data directory** — where a node keeps chain data. Home: `node_workspace_path(...)`,
`STORAGE_PATH_TEMPLATE`.
*Rejected:* **volume**, **`vol-{id}`**, **`/dev/xvda (Root)`**, **3000 IOPS (gp3)**,
**Attached**, **EBS** [`detail_tabs.rs:666-676`] → all delete. Only the storage-engine column
there is real. Disk size/IOPS are not measured anywhere in `src/`; no replacement, the rows go.

**storage engine** — LevelDB/RocksDB/etc. Home: `StorageEngine`. Keep. Delete the adjacent WAL
integrity guarantee [`:704`] — NeoNexus does not implement the storage engine.

**listening addresses** — the host:port pairs the *managed config* binds (RPC, P2P, WS).
Derived from the generated config, and labelled with the caveat that operator `args` pass
through verbatim and a `--config` arg suppresses the managed config entirely.
*Rejected:* **Inbound Security Group Rules (Firewall Ruleset)** [`detail_tabs.rs:629`],
**security group** [2], **Rule Status: Open**, **`0.0.0.0/0`** as a literal → all delete.
NeoNexus manages no firewall (`grep -rni "iptables|pfctl|ufw|nftables" src/` → UI captions
only), and the table contradicted the endpoints card one tab away.

## 1.10 Access and actors

**API token** — a credential for the machine API. Home: `api_tokens`, `/settings/api-tokens`.
*Rejected:* **IAM API Credentials** [service menu], **IAM** [34] → API token.

**operator** — the human using the console. **actor** — the field on an event saying who did
it (operator, watchdog, upgrader, an MCP agent). **agent** — an MCP consumer such as Hermes.
*Rejected:* `arn:neo:iam::nexus:operator` as a rendered identity; **User Identity** as a column
head; **Authenticated IAM Role** [`page.rs` header tooltip].

## 1.11 Reserved-word summary (the grep list)

Delete on sight from `src/web/` string literals; each maps to a replacement above:

`instance`, `EC2`, `AMI`, `EBS`, `Elastic Block Store`, `IAM`, `KMS`, `ARN`, `arn:`,
`CloudWatch`, `CloudTrail`, `CloudFormation`¹, `Systems Manager`, `SSM`, `OpsCenter`, `SNS`,
`security group`, `availability zone`, `nexus-az`, `neo:mesh`, `vpc-`, `Account:`, `Region`²,
`t3.`, `gp3`, `IOPS`, `vol-`, `/dev/xvda`, `Hypervisor`, `Flavor`, `Environment`³,
`Production` (as a node attribute), `lease`, `instance profile`, `alarm`, `snapshot`⁴,
`immutable`, `Launch Instance`, `profile` (unqualified).

¹ survives **only** as an IaC export *format value* — see §2.3.
² survives only in third-party contexts NeoNexus genuinely talks to, of which there are none today.
³ including `Environment` as a tag key.
⁴ including the `snapshots` nav key; the route `/snapshots` stays as a redirect (§3.4).

---

# 2. Ruling on the AWS metaphor

## 2.1 The rule

> **A borrowed term may stay only if all three hold:**
> **(a) Substance** — NeoNexus implements the mechanism the term names. Not "something
> analogous"; the mechanism.
> **(b) Non-collision** — the term is not already occupied by a meaning in the Neo domain.
> **(c) Necessity** — no ordinary English word is as accurate. Borrowing must buy precision,
> not atmosphere.
>
> Fail any test and the term goes. Where the term names something that does not exist, it is
> **deleted with no replacement** — the absence is the honest state, and inventing a
> NeoNexus-flavoured name for a non-feature just launders the same lie.

**Vendor proper nouns fail (c) categorically and are deleted without case-by-case review.**
CloudWatch, CloudTrail, CloudFormation, Systems Manager, OpsCenter, SNS, IAM, KMS, EC2, EBS are
names of another company's products. They can never be the accurate name of a NeoNexus surface,
because the accurate name of a NeoNexus surface is what it does. This removes 29+31+8+11+16+31+9
= ~135 occurrences by rule, not by argument.

## 2.2 The verdict table

| Borrowed term | Substance? | Collision? | Necessity? | Verdict | Replaces with |
|---|---|---|---|---|---|
| **instance** (a node) | yes — the thing exists | no | **no** — "node" is the domain word, used by 100% of `src/` outside the web layer | **Go** | node |
| **fleet** | yes | no | yes — shorter and more exact than "all nodes in this workspace" | **Stay** | — |
| **lease** (signer binding) | **partial** — exclusivity yes, expiry/renewal no | no | no — "exclusive binding" says it | **Go** (see §1.4) | signer binding (+ "exclusive") |
| **instance profile** | no — nothing attaches an identity to a machine | — | — | **Go** | signer binding |
| **IAM** | no — no policies, principals, or roles exist | **yes** — "role" collides with duty *and* designation | no | **Go** | API token / operator / actor |
| **KMS** | no — no AWS SDK in `Cargo.toml`; `grep -rni "encrypt" src/config/` is empty | no | no | **Go** | signer backend / key |
| **SecureString / KMS-encrypted** [`config.rs:97,:112`] | **no** — asserts encryption that does not exist over files that hold **plaintext wallet unlock passwords** (`neo_cli.rs:116`, `neo_go/services.rs:57`) | — | — | **Go, urgently** | "Written 0600. Not encrypted at rest." |
| **AMI** | no — no images; a runtime is an unpacked directory | no | no | **Go** | runtime |
| **EBS / volume / IOPS** | no — no disk measurement anywhere in `src/` | no | no | **Go**, no replacement | data directory (path only) |
| **security group** | no — no firewall capability | no | no | **Go**, no replacement | listening addresses |
| **availability zone / region / VPC / account** | no | no | no | **Go**, no replacement | host (once G34 lands); `localhost` today |
| **instance type / t3.x / flavour** | no — nothing allocates CPU/RAM/IO | no | no | **Go**, no replacement | — |
| **CloudWatch** (metrics, logs, alarms) | partial — real process CPU/RSS exist at `monitor.rs:202-204` | no | **no** — vendor proper noun | **Go** | Health / Logs / Alerts |
| **alarm** | no — no alarm evaluator exists in the codebase | no | no | **Go** | rule (condition) + alert (delivery) |
| **CloudTrail** | no — plain table, no hash chain, no trigger, plus a live arbitrary-insert path via backup import | no | no | **Go** | Events |
| **Systems Manager / OpsCenter / OpsItem** | partial — a real readiness engine exists | no | no — vendor proper noun | **Go** | Readiness / finding |
| **Parameter Store** | no | no | no | **Go** | Configuration |
| **SNS** | partial — deliveries are real | no | no | **Go** | delivery channel |
| **EC2** | n/a — pure branding | — | — | **Go** | — |
| **CloudFormation** | **yes** — `IacFormat::CloudFormation` at `web/api/iac.rs:116` genuinely emits CloudFormation YAML | no | **yes** — it is the actual name of the actual output format | **Stay, narrowly** | see §2.3 |
| **tag** | **not yet** — no column exists | no | yes | **Reserved** (§1.6) | nothing rendered today |
| **console** | yes — it is a web console | no | yes | **Stay** | — |
| **launch** (starting a process) | yes — `launch_node` | **yes** — collides with creating a node | **Split** | "Start" (process) / "Add node" (create) |

## 2.3 The one survivor, and why it proves the rule

`CloudFormation` stays — as a **value**, never as a page name.

`GET /api/fleet/iac?format=cloudformation` really does produce a CloudFormation template
(`IacFormat::CloudFormation`, `src/web/api/iac.rs:116`). Calling that output "CloudFormation" is
not costume; it is the correct name of the artifact, sitting beside `terraform`, `k8s`,
`docker`. So it passes (a), (b) and (c) and stays, in exactly one place: the format selector and
the download filename.

What goes is `nav`/`page_head` use of the same word: the `/config` page's breadcrumb
"CloudFormation & Config", the service-menu entry, and the two `☁️ Export CloudFormation`
buttons on `/` and `/config` — which describe a *fleet IaC export* and belong on an "Export"
control with a format picker, not as a page identity.

This is the rule working as intended: the word survives exactly where the thing exists, and
dies everywhere it was decoration. Every other AWS term in the table fails test (a) or test (c),
and that is not a coincidence — it is what "costume" means.

## 2.4 What replaces the deleted chrome

Deleting is not always enough; three places lose a whole visual block and need a real one.

| Deleted | Replaced by | Source of truth |
|---|---|---|
| Security-group table (`detail_tabs.rs:583-629`) | **Listening addresses** — RPC / P2P / WS rows with the bound address, each marked *managed* or *overridden by `args`*; a `--config` arg collapses the card to "This node uses an operator-supplied config file; NeoNexus does not know its bind addresses." | the generated config + `node.args` |
| EBS volume card (`:666-676`) | **Data directory** — the real path, storage engine, and (only if measured) free space. Neo X `--datadir` override shown when present. | `node_workspace_path`, `StorageEngine`, `args` |
| Placement / region / account chrome (5 surfaces) | **Host** — `localhost` today, a `hosts` row after G34. | `nodes.host_id` |
| Tags tab | **Nothing**, until the column exists. The tab is removed, not emptied with a promise. | — |
| `Health 2/2` header pill (`page.rs:121-123`) | **Fleet health summary** bound to real counts, or **nothing**. A global pill that takes no fleet argument cannot be made honest; if `layout_with_density` will not be given fleet state, delete the pill. | §4.2 counts |

---

# 3. Page-naming rule

## 3.1 The rule

> **The word the operator clicks is the word at the top of the page that opens, and it is the
> same string object.**
>
> A destination's name is declared exactly once, in `nav::Destination.label`. That one string
> is the nav item, the `<h1>`, the `<title>`, the breadcrumb leaf, and the text of every
> in-product link that points there. No page may author its own title.

Enforced by type, not by review (C1):

```rust
// nav.rs
pub struct NavKey(&'static str);          // constructible only from SECTIONS
pub const NODES: NavKey = ...;            // one const per destination
pub fn label_for(key: NavKey) -> &'static str;
pub fn link_to(key: NavKey) -> String;    // <a href=…>label</a>

// html/page.rs — first parameter is no longer a &str
pub fn page_head(key: NavKey, subtitle: &str, actions: &str) -> String;
pub fn layout(key: NavKey, flash: &str, body: &str) -> String;   // title & active derived
```

`layout` currently takes `title` and `active` as two independent `&str`s, which is how
`/settings/api-tokens` came to mark Settings active while having no nav entry (G42). One
`NavKey` makes that unrepresentable.

**Every destination gets a nav entry or ceases to be a destination.** `/backup` and
`/settings/api-tokens` currently have neither nav entry nor inbound link; they get entries
(Workspace → Backup; Settings → API tokens as a sub-destination).

## 3.2 The second navigation is deleted

The `Services ▾` menu (`html/page.rs:86-107`) is deleted outright. It is a second, divergent
naming of the same destinations, it omits nine of them, and it uniquely contains one.
If a services-style launcher is wanted later it is *generated* from `nav::SECTIONS`, so it
cannot diverge. Deleting is stage-1 work and removes ~15 AWS proper nouns by itself.

## 3.3 Breadcrumb rule

> **Breadcrumbs are derived from the URL path and express containment. A breadcrumb never links
> to the page it is on.**

```rust
// html/page.rs
pub fn breadcrumb_for(path: &str) -> String;   // the only public constructor
fn breadcrumb(items: &[(&str,&str)]) -> String; // becomes private
```

Rules:
1. Each crumb is a real ancestor route: a **strict prefix** of the current path that is a
   registered GET route and renders.
2. The last crumb is the current page, rendered as **text with no href**.
3. A one-level page (`/monitor`, `/events`, `/alerts`, `/config`) has **no breadcrumb at all**.
   A single crumb that links to itself is not navigation; it is decoration. This alone deletes
   the "`CloudWatch` / `Metrics` / `All metrics`, both crumbs href `/monitor`" defect (G43).
4. Crumb text is `nav::label_for` for nav destinations, and the **entity's own name** for entity
   segments — `Nodes / seed-01 / Edit`, not `EC2 / Instances / i-…`.
5. Depth is bounded by real path depth; no invented hierarchy. There is no "Images" level
   above Runtimes because there is no `/images` route.

## 3.4 The resulting name table

| Route | Nav section | **The one name** | Subtitle (one sentence, no claims) | Was called |
|---|---|---|---|---|
| `/` | Overview | **Fleet overview** | What each node is doing right now. | "AWS Management Console · Global Command Center" / "AWS Console / Console Home" |
| `/nodes` | Fleet | **Nodes** | Every node in this workspace. | "EC2 Instances" |
| `/nodes/new` | (action) | **Add node** | — | "Launch Instance" |
| `/nodes/{id}` | (child) | **{node name}** | — | "Instance i-…" |
| `/monitor` | Fleet | **Health** | Process and chain health for each node, and when it was last checked. | nav "Health" / crumb "CloudWatch / Metrics / All metrics" / title "CloudWatch Metrics & Telemetry" / menu "CloudWatch Metrics" |
| `/logs` | Fleet | **Logs** | Standard output and error from each managed process. | title "Logs" / crumb "CloudWatch / Logs / Log groups" |
| `/operations` | Operations | **Readiness** | Whether each node's configuration would produce a working node. | "Systems Manager OpsCenter" |
| `/events` | Operations | **Events** | Actions taken in this workspace, most recent first. | "CloudTrail Event History" |
| `/alerts` | Operations | **Alerts** | Where alerts are sent, and what has been sent. | "CloudWatch Alarms & SNS" |
| `/federation` | Network | **Peer workspaces** | Other NeoNexus workspaces this one reads counters from. | "Federation" |
| `/roles` | Fleet | **Duties** | Which duties each client can perform, and which are assigned. | nav **"Private network"** (names a different feature entirely) |
| `/runtimes` | Assets | **Runtimes** | Client releases available to download, and what is installed. | "AMIs & Node Runtime Catalogs" / crumb "EC2 / Images / AMIs & Node Runtimes" |
| `/snapshots` → `/archives` | Assets | **Fast-sync archives** | Chain-data archives that can be applied to a node to skip initial sync. | "EBS Snapshots & Fast-Sync" / crumb "EC2 / Elastic Block Store / Snapshots" |
| `/plugins` | Assets | **Plugins** | Client plugins installed per node. | *(no head, no breadcrumb)* |
| `/config` | Assets | **Configuration** | The config file NeoNexus writes for each node, and whether it matches disk. | "Systems Manager · Application Configuration" / crumb "Systems Manager / Application Management / Parameter Store & Config" / menu "CloudFormation & Config" |
| `/wallets` | Security | **Wallets** | NEP-6 wallet files known to this workspace. | *(no head, no breadcrumb; light-themed form inside the dark shell)* |
| `/signer` | Security | **Signers** | Signer backends, the keys in them, and which node each key is bound to. | nav "Signer" / crumb "KMS / Customer managed keys" |
| `/metrics` | Workspace | **Metrics** | *(merge candidate — see note)* | *(no head, no breadcrumb)* |
| `/backup` | Workspace | **Backup** | Export this workspace, or import one. | *(no nav entry at all)* |
| `/settings` | Workspace | **Settings** | — | *(no head; contains no `<a href>`)* |
| `/settings/api-tokens` | Workspace → Settings | **API tokens** | — | "IAM API Credentials" |

Two consequences worth stating explicitly:

- **`/roles` → "Duties"** and the nav entry named "Private network" is **removed**, not
  repointed. There is no private-network page today (G19: the planner has no callers in `src/`).
  A nav entry for a feature that does not exist is worse than its absence.
- **`/metrics` and `/monitor` render the same `collect_snapshot`** (G47), one as a raw `<pre>`
  with no header and strictly less information. Naming cannot fix that; `/metrics` is deleted
  and its route redirects to `/monitor`. One name, one page.
- Route renames (`/snapshots` → `/archives`, `/roles` → `/duties`) keep the old path as a 301
  so bookmarks and the `?node=` deep links survive.

## 3.5 Cross-page link text

Any in-product link to a destination uses `nav::link_to(key)` and therefore reads exactly the
nav label. This kills the "📊 CloudWatch Metrics" / "🚨 CloudWatch Alarms" /
"⚙️ SSM OpsCenter" button rows on `/events`, `/monitor`, `/alerts`, `/logs`, `/config`,
`/operations` and `/snapshots` in one edit — and it means an operator who clicks "Health" from
three different pages lands on a page headed "Health" every time.

---

# 4. Status vocabulary

Three axes. **No word appears on two axes.** Each value has exactly one visual treatment,
defined once in `styles.rs` and emitted only by a typed helper — pages may never write
`class="badge running"` (currently 35 occurrences of `badge running`, 21 of `badge stopped`,
against a 5-value CSS vocabulary that does not cover "unknown").

```rust
// html/status.rs — the only emitters
pub fn process_badge(s: NodeStatus) -> String;
pub fn health_badge(h: NodeHealth) -> String;      // NodeHealth = Option<RpcHealth> resolved
pub fn check_badge(o: CheckOutcome) -> String;
pub fn check_tally(checks: &[Check]) -> String;
```

## 4.1 Axis A — process status (what the supervisor sees)

| Value | Meaning | Dot | Colour token | Text |
|---|---|---|---|---|
| **Stopped** | no process; NeoNexus did not start one | hollow | `--muted` grey | Stopped |
| **Starting** | launched, inside the settle window | hollow, animated | `--amber` | Starting |
| **Running** | process alive, pid known | filled | `--jade` green | Running |
| **Failed** | exited uncleanly, or failed to launch | filled | `--red` | Failed |
| **Unknown** | NeoNexus cannot see this process (remote host, or supervisor not running) | hollow, `?` glyph | `--muted` grey | Unknown |

`NodeStatus::Error.label()` changes from "Error" to **"Failed"** — an error is a thing that
happened, a failure is a state a node is in. Enum variant renamed in stage 4; persist key
frozen. **Unknown** is added now even though only one host exists, because it is the value
G34's remote hosts will need and because it is the correct rendering for "the supervisor is
not running".

## 4.2 Axis B — chain health (what the chain says when asked)

| Value | Derived from | Dot | Colour | Text |
|---|---|---|---|---|
| **Not checked** | no `RpcHealthRecord` for this node, ever | hollow | grey | Not checked |
| **Stale** | newest record older than 2× the probe interval — *the observer itself is down* | hollow | grey | Last checked {t} |
| **Healthy** | probe answered **and** head advanced since the previous tick **and** lag within threshold | filled | green | Healthy |
| **Behind** | answered, advancing, but lag > threshold vs the reference head — normal during initial sync | filled | amber | Behind by {n} blocks |
| **Stalled** | answered, but height unchanged for N consecutive ticks — *running but not syncing* | filled | red | Stalled at block {h} |
| **Degraded** | some probe calls answered, not all | filled | amber | Degraded |
| **Unreachable** | no probe call answered | filled | red | Unreachable |

**"Not checked" is the load-bearing addition.** Today `detail_tabs.rs:273` grants
`🟢 2/2 System & Instance Checks Passed` when `latest_rpc.is_none_or(|r| r.status == Healthy)` —
a node that has *never been probed* is rendered identically to a passing one. That is C2
inverted, and it is the single most dangerous string in the console at 03:00. The rendering of
`None` is **grey "Not checked"**, and the type makes it unavoidable:

```rust
pub enum NodeHealth { NotChecked, Stale{ at: u64 }, Healthy, Behind{ blocks: u64 },
                      Stalled{ at_height: u64, since: u64 }, Degraded, Unreachable }
impl NodeHealth { pub fn from_record(r: Option<&RpcHealthRecord>, now: u64, …) -> Self; }
```

There is no `Default`, no `unwrap_or(Healthy)`, and no `From<bool>`.

**Staging.** `NotChecked`, `Stale`, `Healthy`, `Degraded`, `Unreachable` are derivable
**today** from `Option<RpcHealthRecord>` plus its timestamp — they land in stage 2, no schema
change. `Behind` and `Stalled` need the stored head-lag and last-height-change the register's
sequencing step 2 (G11) introduces, and land with it. Until then `Healthy` means only "the
probe answered", and the subtitle on `/monitor` says exactly that: *"Health means the node
answered a JSON-RPC call. NeoNexus does not yet compare block heights."* An honest limitation,
stated once, beats a green pill.

**Three words that must never be combined:** a node may be **Running** (axis A) and
**Stalled** (axis B) at the same time — that is the headline failure mode, and the fleet list
must be able to show both on one row. There is no single "node status" that fuses them, and any
copy implying one (`● 2/2 passed` derived from `is_running()`, `list.rs:283`, `home.rs:42`) is
a bug by definition.

## 4.3 Axis C — check outcomes

| Value | Meaning | Colour |
|---|---|---|
| **Pass** | evaluated, satisfied | green |
| **Warn** | evaluated, satisfied with a caveat | amber |
| **Fail** | evaluated, not satisfied | red |
| **Not applicable** | this check does not apply to this client/duty | grey |
| **Not evaluated** | the check did not run (missing input, node never probed) | grey |

**Tally rule:** a check summary **lists outcomes, never a fraction**.

- Rejected: `🟢 2/2 System & Instance Checks Passed`, `🔴 2/4 Checks Failed (Signer Missing)`,
  `● 2/2 passed`, `Health 2/2`.
- Canonical: **`3 pass · 1 warn · 2 not evaluated`**, from `check_tally()`, over a single
  `checks()` function with one denominator (G6). If a fraction must be shown, its denominator is
  the count of *evaluated* checks and the not-evaluated count is shown beside it.

## 4.4 Words banned outright as status text

`OK`, `● OK`, `All Systems Operational`, `In Sync`, `Active`, `Armed`, `Open`, `Attached`,
`Valid`, `Lease Valid`, `Operational`, `Nominal`, `passed` as a bare literal. Each currently
appears as a constant that is a function of nothing. They are not replaced by better constants;
they are replaced by the typed emitters above or they are deleted.

---

# 5. Message and error voice

## 5.1 Shape

> **`{Action} {outcome} for {node name}: {reason}. {Imperative next step}.`**
>
> Four parts, in that order: what was attempted, which node **by name**, why it did not happen,
> and what the operator does now. One sentence plus one imperative. No exclamation mark, no
> emoji, no severity emoji, no "failed:" prefix.

Typed, so the shape cannot be bypassed:

```rust
pub struct OperatorMessage { action: &'static str, subject: Subject,
                             outcome: Outcome, reason: String, next: NextStep }
pub enum Subject  { Node{ name: String, id: String }, Fleet, Workspace }
pub enum NextStep { Here{ label: String, href: String },   // a link on this page
                    Elsewhere{ text: String },            // console can't; say what can
                    None }                                // nothing to do
pub(crate) fn back_to_node(id: &str, message: OperatorMessage) -> Response;
```

`back_to_node(id, &format!("failed: {error}"))` (`control/node.rs:114,:119`) stops compiling.

## 5.2 Rules

- **M1 — Name the node by name, never by id, never "the instance."** The id may follow in mono
  as a `title` attribute.
- **M2 — A refusal says what was refused, verb first.** "Start refused", "Binding refused",
  "Import refused" — not "failed", not "error".
- **M3 — Never name a CLI flag or a Rust API in web copy.** Currently:
  `"…or run --node-rebind-runtime before launch"` (`preflight/command_path/check.rs:15`),
  `"Add one through the Rust API"` (`federation.rs:116`). The browser is a different audience;
  a CLI equivalent belongs in CLI output only.
- **M4 — Every refusal ends in one of exactly two things:** an action the operator can take
  **here** (a link), or an explicit statement that the console cannot do it and what can. A
  dead end that pretends to be advice is the worst outcome.
- **M5 — A count of failures names the failures.** "2 of 3 started" must be followed by which
  one did not and why.
- **M6 — No emoji in operator copy.** Banned in `src/web/pages/` string literals. Icons are the
  static SVG set in `nav::icon`; status is the dot in §4. This removes 📸 🚨 📊 ☁️ ⚡ ⚙️ 💾 🔒 🔑 ✦ 🟢 🔴.
- **M7 — No claim about a mechanism unless the mechanism exists and can be named.** Banned
  adjectives: *immutable*, *cryptographic*, *real-time*, *near real-time*, *verified*,
  *encrypted*, *CloudTrail-grade*, *hyperscaler*, *one-click*, *guaranteed*. Each may return
  the day it is true, attached to the name of the thing that makes it true ("SHA-256 checksum
  verified on download" is fine — that check exists).
- **M8 — Severity words come from §4.** A message may not introduce a status word outside the
  table. `EventSeverity::Critical` is reserved for *a node failing its duty now* — not for a
  smoke test on a binary that is not running (`control/node.rs:104`).
- **M9 — Internal invariants get a distinct shape**, so operators do not hunt for an operator
  error: "NeoNexus could not handle that request (unknown action "xyz"). This is a bug; see
  Events for details."
- **M10 — CLI and web say the same words, in their own registers.** The CLI's `key: value`
  lines take their values from the same `label()` methods; only the framing differs. `rpc-health: not-checked`
  is the CLI form of "Not checked" — never a different vocabulary.

## 5.3 Before / after, from real strings

**1. Launch refusal — neo-cli plugin directory** (`core/node_signer.rs:419-424` → flash via
`back_to_node`; G24)

> **Before:** `failed: neo-cli loads plugins beside its binary, but this node writes isolated plugin config under /ws/nodes/ab12c3/Plugins; place a complete per-node neo-cli runtime in that directory (current binary root: /ws/runtimes/neo-cli/3.7.4/macos-arm64)`
>
> **After:** **Start refused for seed-01.** neo-cli loads plugins from the folder holding its
> binary. Its binary is in the shared runtime folder, but this node's plugins are in its own
> node folder, so the Consensus plugins would not load.
> *NeoNexus cannot fix this from the console: a complete neo-cli runtime has to be placed in
> the node folder by hand. Show paths ▾*

Applies M1 (names seed-01), M2 (verb first), M4 (states honestly that the console cannot do
it — the register proves no chip the editor offers can satisfy this gate), and hides two long
paths behind a disclosure instead of leading with them.

**2. No signer backend** (`pages/nodes/binding.rs:182`; G28)

> **Before:** `No signer profile is configured. Configure the signer registry before binding this node.`
>
> **After (today, honest):** **No signer backend is available.** NeoNexus reads signer backends
> from its own process environment when it starts, and there is no way to add one from the
> console yet. Set the backend environment variables and restart NeoNexus.
>
> **After (once the form exists):** **No signer backend yet.** → *Add signer backend*

M4: the "before" tells the operator to do a thing with no form, no link, and no named
mechanism. Either version of the "after" ends somewhere.

**3. No peer workspaces** (`pages/federation.rs:116`; G29)

> **Before:** `No federation servers are configured. Add one through the Rust API or restore a backup that carries them.`
>
> **After (today):** **No peer workspaces.** This build cannot add one from the console;
> importing a workspace backup that contains peers will restore them. → *Import a backup*
>
> **After (once create lands):** **No peer workspaces yet.** → *Add peer workspace*

M3: never tell an operator to write Rust.

**4. Batch result** (`control/node.rs:249-252`; M5)

> **Before:** `Batch action 'start' executed on 3 instances: 2 succeeded, 1 failed`
>
> **After:** **Start: 2 of 3 nodes started.** seed-03 did not start — no key is bound to it.
> → *Open seed-03*

**5. Binary path preflight** (`preflight/command_path/check.rs:15`; M3)

> **Before:** `No trusted local runtime is bound. Select a local binary in the node editor or run --node-rebind-runtime before launch.`
>
> **Web:** **No runtime is bound to seed-01.** Choose a runtime in the node editor.
> → *Edit seed-01*
> **CLI:** `binary-path: unbound | fix: --node-rebind-runtime <node-id> <path>`

**6. Config drift badge** (`pages/config.rs:142-146`; G5, and §4)

> **Before:** `● In Sync` — derived from `Path::is_file()`, under a header claiming
> "configuration drift verification", beside `("KMS Encryption", "AWS-KMS (active)")`.
>
> **After:** **Not checked** — *NeoNexus has not compared this file to the config it would
> write. → Check for drift* … and the encryption row is deleted and replaced with:
> **Permissions: 0600 · Not encrypted at rest.** *These files contain the wallet unlock
> password in plaintext; exclude the workspace folder from unencrypted backups.*

This is the clearest case of the whole document: the "before" asserts a verification that never
ran and an encryption that does not exist, over a file that holds a plaintext password. The
"after" is shorter, entirely true, and is the only version that would change an operator's
behaviour correctly.

**7. Events actor column** (`pages/events.rs:130-134`; G8)

> **Before:** column `User Identity`, value `arn:neo:agent::hermes-ai` when the message text
> contains "Hermes" or "probe", else `arn:neo:iam::nexus:operator`.
>
> **After:** column **Actor**, value **—**, with the column's help text reading *"NeoNexus
> does not record who triggered an event yet."* When `RuntimeEvent` gains an actor field, the
> column fills with real values: `operator`, `watchdog`, `upgrader`, `agent:hermes`.

An empty honest column is a feature request the operator can see. A fabricated one is a lie
that also hides the feature request.

---

# 6. Enforceable lint

## 6.1 Where it lives

The repo already has the right shape for this: `src/source_quality/` (a marker scanner with
`rules.rs`/`scan.rs`/`checker.rs`/`model.rs`, a `--source-quality` / `--source-quality-json`
CLI pair, a `SourceQualityReport { findings }` with `exit_code()`, and a `tests/ci_policy` suite
that asserts `.github/workflows/ci.yml` actually runs it).

**Build `src/vocabulary/` in that exact image:**

```
src/vocabulary.rs
src/vocabulary/glossary.rs   // §1 as data — the single source of truth
src/vocabulary/rules.rs      // rule definitions, generated from glossary.rs
src/vocabulary/scan.rs
src/vocabulary/checker.rs    // VocabularyChecker::check(root) -> VocabularyReport
src/vocabulary/model.rs      // VocabularyReport / VocabularyFinding (Serialize)
src/vocabulary/allow.rs      // the shrinking allowlist
```

CLI: `--vocabulary <dir>` and `--vocabulary-json <dir>`, added to `ci.yml` and asserted by a new
`src/ci_policy/requirements/commands/vocabulary.rs` — so removing the gate from CI fails CI.

`docs/VOCABULARY.md` is **generated** from `glossary.rs` and checked in; a test asserts the
checked-in file matches regeneration. The glossary cannot drift from the lint that enforces it,
and a reviewer has one file to read.

## 6.2 The rules

Scanner rules (string literals only, so identifiers and type names are untouched):

| Id | Category | Checks | Scope |
|---|---|---|---|
| **R-V1** | `banned-term` | Every rejected synonym in §1/§1.11 appears in no string literal. Finding carries the canonical replacement. | `src/web/`, `src/cli/output*` |
| **R-V2** | `literal-status-word` | None of `OK`, `Healthy`, `Running`, `Stopped`, `Failed`, `Passed`, `In Sync`, `Active`, `Armed`, `Open`, `Attached`, `Valid`, `Degraded`, `Unreachable`, `Operational` appears in a string literal. Status text comes only from §4's emitters. | `src/web/pages/` |
| **R-V3** | `status-class-literal` | No `badge running`/`badge stopped`/`badge warn`/`badge danger`/`status-dot …` outside `src/web/html/`. | `src/web/pages/` |
| **R-V4** | `emoji-in-copy` | No codepoint in the emoji blocks in any string literal. | `src/web/pages/`, `src/web/control/` |
| **R-V5** | `fraction-status` | No `\d+/\d+` adjacent to a status word in a literal (catches `2/2 passed`, `Health 2/2`, `2/4 Checks Failed`). | `src/web/` |
| **R-V6** | `mechanism-claim` | No `immutable`, `cryptographic`, `real-time`, `verified`, `encrypted`, `guaranteed`, `CloudTrail-grade`, `hyperscaler` in a literal unless the file is on a per-claim allowlist naming the mechanism. | `src/web/` |
| **R-V7** | `cli-flag-in-web-copy` | No `--[a-z][a-z-]+` inside a string literal. | `src/web/` |
| **R-V11** | `persisted-key-rendered` | `.slug()` / `.persist_key()` never appears inside a `format!`/`push_str` position that reaches rendered text (catches `("validator", "⚡ Validator")`). | `src/web/pages/` |

Type rules — stronger, because they cannot be allowlisted away (C1):

| Id | Mechanism |
|---|---|
| **R-V8** | `page_head`/`layout` take `NavKey`, not `&str`. A page cannot author a title. |
| **R-V9** | `html::breadcrumb` becomes private; only `breadcrumb_for(path)` is public. |
| **R-V10** | `back_to_node` and every flash redirect take `OperatorMessage`, not `&str`. |
| **R-V12** | `html::duty_label(Option<NodeRole>)` is the only duty renderer; `NodeRole::label` is `pub(crate)`. Makes G9 unrepresentable. |
| **R-V13** | `NodeHealth` has no `Default`, no `From<bool>`, and `from_record` takes `Option<&RpcHealthRecord>` — so "never probed" cannot silently become green. |

Test rules — in `tests/ci_policy/` and `tests/ui_operator_walkthrough.rs`:

| Id | Assertion |
|---|---|
| **R-V14** | *nav/title parity:* for every `nav::Destination`, the rendered page at `href` contains `<h1>{label}</h1>` and `<title>{label} · NeoNexus</title>`. |
| **R-V15** | *breadcrumb containment:* every crumb href from `breadcrumb_for(p)` is a registered GET route **and** a strict prefix of `p`; the final crumb has an empty href; a depth-1 path yields no breadcrumb. |
| **R-V16** | *glossary integrity:* no string is both a canonical term and a rejected synonym; every rejected synonym has a replacement; every canonical term has a code home listed. |
| **R-V17** | *doc sync:* `docs/VOCABULARY.md` equals the regeneration of `glossary.rs`. |
| **R-V18** | *one navigation:* `src/web/html/page.rs` contains no `<a href="/` outside the generated nav. (Stops the Services menu regrowing.) |

R-V14/R-V15 overlap with the register's suggested **step 4 parity gate** (route ⇒ nav entry,
form action ⇒ route). They should be built as one gate, not two: the nav/title check and the
route-reachability check read the same `nav::SECTIONS` and the same router.

## 6.3 Staged adoption, so stage 1 is landable

`vocabulary-allow.txt`, one `path:line:rule` per line, seeded with today's full violation set.
CI asserts two things:

1. every finding is either fixed or allowlisted (so the gate is green on day one);
2. the allowlist's line count is `<=` a checked-in `vocabulary-allow.max` — which a PR may
   lower or leave equal, **never raise**.

A ratchet, not a cliff. New code is held to the full rule immediately (no new allowlist
entries), existing violations drain at whatever pace the team sets, and the number in the repo
is a visible debt counter. The `--vocabulary-json` output keeps this automatable, per the
headless-CLI constraint.

---

# 7. Sequencing

Each stage leaves the product working and is independently landable.

**Stage 1 — copy only, no schema, no types.** Delete the `Services ▾` menu; delete the AWS
chrome enumerated in §2.2 (security groups, EBS, tags, instance type, placement, KMS
assertions, ARNs); apply the §3.4 name table; delete the four fabricated alarm rows and the
`Health 2/2` pill. Land `src/vocabulary/` with R-V1/R-V2/R-V4/R-V5/R-V6/R-V7 and the seeded
allowlist. **This is subtractive and is most of the win** — it is also the register's own
sequencing step 1.

**Stage 2 — types.** `NavKey`, `page_head`/`layout` signatures (R-V8), `breadcrumb_for`
(R-V9), `OperatorMessage` (R-V10), `duty_label` (R-V12), `NodeHealth` with **Not checked** and
**Stale** (R-V13). Parity tests R-V14/R-V15. No new data required.

**Stage 3 — with the observation layer (register step 2).** `Behind` and `Stalled` join §4.2.
`/monitor`'s honest-limitation subtitle is replaced by the real thing. The alert page's rule
vocabulary becomes real.

**Stage 4 — renames, behind frozen persist keys.** `NodeRole` → `NodeDuty`,
`NodeType` → `ClientKind`, `FastSyncSnapshot` → `FastSyncArchive`, `NodeStatus::Error` →
`Failed`, routes `/snapshots` → `/archives` and `/roles` → `/duties` with 301s.
**Frozen and pinned by test:** every `persist_key()`/`slug()` string (including
`Consensus => "validator"`), every `serde` rename (including `neox-geth`), every `EventKind`
discriminant, every `ChainRole` discriminant (consensus-visible), and every CLI flag name. The
rename is a display-layer rename; a workspace DB written by the old build must open unchanged
in the new one, and a script parsing `--*-json` output must keep working.
