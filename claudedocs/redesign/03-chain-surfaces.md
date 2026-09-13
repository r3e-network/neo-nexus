## 7. Chain surfaces

### 7.1 Neo N3

#### 7.1.1 What public JSON-RPC can and cannot answer

| Operator question | Answerable? | How |
|---|---|---|
| Am I in the committee? | **Yes** | `getcommittee` |
| Am I in the top 7 — producing next round? | **Yes** | `getnextblockvalidators` |
| How many votes do I have, and how close to falling out? | **Yes, derived** | `getcandidates`, sorted; margin to rank 8 and rank 22 |
| Is my vote trending down? | **Only from our own samples** | No RPC returns historical vote totals. The chart names its sample count and start date. |
| Did the committee just change? | **Yes** | digest diff, with the in/out named |
| **Am I producing my assigned blocks?** | **Yes, by derivation** | §7.1.3, with three stated caveats |
| Are we in a view change right now? | **No** | §7.1.4 |
| What is my consensus peer set? | **No** | `getpeers` returns addresses, not public keys |
| Was I slashed? | **N/A** | dBFT has no slashing. Offering a slashing surface would teach a wrong protocol model. |

#### 7.1.2 Designation

Key resolution, source always shown: `node_signer_bindings` → `SignerRegistry::key_info()` → `KeyPublic.public_key`; else `node_wallets` → `NeoWalletProfile.contract_public_keys` (all keys compared, the matching one named); else **none**, which maps to `includes_node_key = NULL` and emits `ChainDesignationUnknown` at Warning — the node has a duty that requires designation and NeoNexus has no key to check.

Transitions and events: `ChainDesignationGranted` (Notice) · **`ChainDesignationRevoked` (Critical)** · `ChainDesignationSetChanged` (Info) · `ChainDesignationUnknown` (Warning) · `ChainDesignationUnreadable` (Warning after N consecutive failures).

The revocation message shape is specified verbatim in §4.2 and is the sentence the register says the product cannot say.

#### 7.1.3 Consensus liveness — the derivation, bound

This is the rung the judges called a word with no evidence source. Here it is, with a schema, a cadence and an alarm.

In dBFT 2.0 the primary for height *h* at view *v* is index `(h − v) mod n` over the ordered validator list, and the verbose block header carries the primary index. Therefore:

```
derived_view(h)             = (h − block.primary) mod n
assigned_to_me(h)           = (h mod n) == my_validator_index
missed_at_view_0(h, me)     = assigned_to_me(h) && block.primary != my_index
interval_on_my_turn(h)      = block_time(h) − block_time(h-1)   where assigned_to_me(h)
```

**Sampler.** The `consensus` class runs on the observation tick for any `chain_key` where the fleet holds a Consensus duty, fetching `getblock(h, 1)` for heights new since the last sample, **capped at 20 per tick** (two calls per tick at a 30 s interval on a 15 s chain). On first enable it backfills at most 200 heights and states how far back the data goes. On unreachability mid-walk it resumes from the last stored height; a gap is recorded, not interpolated. A **reorg** is detected when a stored height's hash differs from a re-read; the affected range is deleted and re-walked, and the affected heights are counted as unattributed.

**Three caveats, on the page, not only in this document:**

1. **Validator-set epochs.** An index means something only relative to the set in force at that height. `chain_blocks.validators_digest` records which set each block is attributed under; blocks straddling a change are excluded and counted as **unattributed**, and the count is shown.
2. **Coverage.** Attribution exists only for sampled heights. The panel states the height range and the number of gaps.
3. **Confidence.** This is the one derivation that *accuses* an operator's node of failing. `DutyState::NotPerforming` is set and `ChainValidatorMissedProposal` is emitted only when the block's validator set matches the set in force **and** there is no gap in the preceding 10 heights. If either fails, the counter renders and the event does not.

**Alarm.** `consensus-slots-skipped ≥ 3`, scope `Duty(Consensus)`, Critical, `for_seconds = 0`. Plus `consensus-interval-overrun-secs` for "the block interval on my turn exceeds `msperblock`". These are the two series that would have paged the operator twenty minutes earlier, and they are why the `Duty(Consensus)` alarm scope exists.

**Rung 6 by duty, stated:**

| Duty | Rung 6 |
|---|---|
| Consensus | **Bound** — assigned vs proposed over the window, from `chain_blocks` |
| StateValidator | **Partially bound** — `validatedrootindex` advancing (a fleet-level fact, not per-key attribution; the panel says so) |
| Oracle, Notary | **Unanswerable** — no standard method exposes pending request depth or this node's response count. Says so; shows levels 1–5. |
| RpcApi, Indexer, State, Observer | **NotApplicable** — no chain designation exists |

#### 7.1.4 What the page states as unanswerable

**View number.** No JSON-RPC method exposes the local dBFT state machine's current view, its `PrepareRequest`/`PrepareResponse`/`ChangeView` traffic, or its timer. The panel says that, then shows the two observable things: the **derived view of recent blocks** (labelled *inference from block headers*) and the **current gap** since the last block against `msperblock` — *"block production has not advanced for 71 s on a 15 s chain; this is consistent with a view change in progress, but the current view is not observable."* The node's log contains consensus messages and is named as a **future** source on the page, not silently synthesised into a number.

**Consensus peer identity.** `getpeers` returns addresses and ports, not public keys. The panel shows connected addresses, cross-references them against this fleet (which on a private network answers the question completely) and against an optional operator-declared validator endpoint list, and for everything else says *"peer identity is not exposed by getpeers; NeoNexus cannot confirm the validator mesh."*

#### 7.1.5 Account and GAS

Shown only where a duty gives the account chain-visible meaning (Consensus and the designated duties), never on RpcApi/Indexer/Observer. Reads, all `invokefunction` on native contracts and available on any RPC node: `NeoToken.balanceOf`, `GasToken.balanceOf`, `NeoToken.getAccountState` → `{balance, balanceHeight, voteTo}`, `NeoToken.unclaimedGas`, `NeoToken.getRegisterPrice`, `OracleContract.getPrice`. `getnep17balances` is **not** used — it requires the TokensTracker plugin and answers `-32601` without it.

The panel **does not** claim a burn rate, a runway, or that a duty will fail at a given balance. Neo's designated services are not uniformly funded from the node's own account (oracle response fees are escrowed by the requester; state-root witnesses are extensible P2P payloads, not transactions), and the product has no verified model of this. It reports balances and their movement, and offers an optional, default-off, operator-set threshold whose copy says *"you asked to be told when this falls below X"*, not *"this node will stop working"*.

### 7.2 Neo X

Neo X today is a Neo N3 node with a different binary. Five `match node_type` sites, no entity, no setting, no form field.

#### 7.2.1 The identity triple

```rust
pub struct Attested<T> {
    pub declared: Option<T>,   // the networks / node row
    pub argv:     Option<T>,   // extracted from node.args
    pub observed: Option<T>,   // read from the running node
    pub observed_at_unix: Option<u64>,
}
pub enum Agreement { Confirmed, DeclaredOnly, Overridden, Mismatch, Unknown }
```

`Mismatch` on chain id or genesis is a Critical that outranks everything. `Unknown` renders the word, never a green pill. This is the Neo X answer to R1 and it is used on `/networks/{id}/identity` for N3 too.

#### 7.2.2 `src/launch/argv_read.rs` — the missing primitive

There is no flag-**value** extraction anywhere in `src/` (G22). One module, used by chain identity, datadir, namespaces, WS, peers, ports and their display.

```rust
pub enum ArgError { MissingValue(String), Malformed { flag: String, value: String } }
/// Last occurrence wins — both geth (urfave/cli) and reth (clap) resolve repeats
/// that way, and an operator who pasted a flag twice means the second.
pub fn flag_value<'a>(args: &'a [String], flag: &str) -> Result<Option<&'a str>, ArgError>;
pub fn flag_present(args: &[String], flag: &str) -> bool;
pub fn flag_values_csv<'a>(args: &'a [String], flag: &str) -> Result<Option<Vec<&'a str>>, ArgError>;
pub fn flag_u64(args: &[String], flag: &str) -> Result<Option<u64>, ArgError>;
```

Rules, each with a test: `--flag value` and `--flag=value` both parse; `--flag=` is `Malformed`, not empty-accepted; a value beginning with `--` is `MissingValue` (`--networkid --http` is an operator error that must surface, not a chain id of `--http`); non-numeric for a numeric flag is `Malformed` → Critical with the text quoted. `has_flag` and `has_chain_argument` collapse into this module, and **`--datadir.chain` is removed from the chain-detection list** entirely: it selects a *subdirectory name*, not a chain, and its presence currently clears a safety gate while selecting nothing.

**Parity gate:** for every value-taking flag the launch planner can emit, `argv_read` must have an extractor — asserted by a table test over one shared `NEOX_FLAGS` declaration that both the emitter and the reader consume.

#### 7.2.3 The private-network safety property

> **A node the operator called private must never dial a public network.**

Three enforcement points, because readiness alone is advisory:

**INV-X1 — identity completeness is a launch precondition.** `complete = 1` plus, for geth, a genesis artefact that exists and hashes to `genesis_sha256` and a datadir whose init marker matches; for neox-rs, `--chain` resolving to a preset or a file whose `config.chainId` equals the declared one. Enforced in `node_lifecycle` before `LaunchPlanner::plan`.

**INV-X2 — no public reachability.** Zero bootnodes drawn from the seeded public sets; `chain_id ∉ PUBLIC_CHAIN_IDS`; discovery off **only when there are no bootnodes**. The current blanket "a private network must not carry any bootnodes" critical is removed — it makes a correctly-wired private network unlaunchable (G23).

**INV-X3 — argv reconciliation.** Because `push_missing` lets operator flags win (correctly), the flags must be *read*. `Agreement::Overridden` on chain id is a Warning on a public network and a **Critical** on a private one, because there an override is the exact route back onto Neo X MainNet with keys the operator treats as throwaway.

#### 7.2.4 The EVM surface as data

`node_neox` carries `http_enabled`, `http_api`, `http_corsdomain`, `ws_enabled`, `ws_api`, `ws_origins`, `authrpc_port`, `metrics_enabled`, `dangerous_ack`, `init_state`.

**Namespaces** become a typed catalogue with per-client availability and a risk level: `eth`, `net` (**required, undeselectable** — the observation layer depends on them), `web3`, `txpool`, `rpc` (makes the namespace set observable via `rpc_modules`), `debug` (high), `trace` (neox-rs only, high), `admin` and `personal` (**dangerous**: require `dangerous_ack` **and** a loopback bind, else Critical). A namespace the chosen client lacks is not offered, and changing the client re-validates and names what was dropped — which is the visible answer to *"the two clients expose different RPC surfaces for reasons the operator cannot see"*.

**One default for both clients:** `DEFAULT_HTTP_API = [eth, net, web3, txpool]`. neox-rs now emits `--http.api`; today it inherits Reth's `STANDARD_MODULES = [eth, net, web3]`, so `txpool_status` silently cannot work on one of the two supported clients.

**WebSocket.** `ws_enabled` is the authority; `ws_port.is_some()` means only *reserved*. geth sets `WSHost`/`WSPort`/`WSModules`/`WSOrigins` together (host and port must travel together or geth opens its default 8546). The neox-rs plan gains `--ws --ws.addr --ws.port --ws.api [--ws.origins]`. The **Open** badge is replaced by observed reachability from a TCP connect on the probe tick: `listening` / `configured, not reachable` / `not configured`.

**Ports nobody is planning.** `authrpc` (geth opens 8551 by default — a guaranteed collision between two managed Neo X nodes on one host) and `metrics` join the planner and get emitted. P2P UDP: geth uses one number for both; reth takes `--port` and `--discovery.port`. Reserve one number, emit both where needed, say so in the ports card.

#### 7.2.5 Bootstrapping

`POST /nodes/{id}/neox/init` — a supervised one-shot `<binary> init --datadir <resolved> <genesis_path>`, output through the existing process-log path, writing `.neonexus-init.json` `{chain_id, genesis_sha256, genesis_path, client, client_version, initialised_at_unix}` and emitting `NeoXDataDirInitialised`. Refuses when the node is running, when the network has no genesis artefact, or when `chaindata` already exists — unless `reinitialise=1` plus a typed confirmation of the node name, because re-init discards the chain and must be as loud as a delete. CLI mirror: `neonexus neox init --node <id>`.

The UI is explicit that the marker is **NeoNexus's own record, not a client guarantee**; the authoritative check is the runtime block-0 hash. The marker exists to catch the wrong-genesis case *before* launching, which is otherwise diagnosed as a network problem.

**`GET /nodes/{id}/neox/enode`** derives the node's own enode locally from the node key on disk (`<datadir>/geth/nodekey`, `<datadir>/<chain>/discovery-secret`) — pure public-key derivation, no signing — and **works while the node is stopped**, which is exactly when the operator is wiring the network. The advertised host is `hosts.address`, never silently `127.0.0.1`; when unknown it renders `enode://<pubkey>@<your host>:30303` with the placeholder visibly a placeholder.

**NeoNexus does not compute a genesis hash offline** (that needs header RLP + keccak and would still not prove what the client did with the file) and **does not generate a Neo X genesis** (an invented allocation is a healthy-looking chain of one). The file's sha256 is the artefact identity — which is exactly what an operator verifies across hosts.

#### 7.2.6 Keys and duties

| | Neo N3 | Neo X |
|---|---|---|
| Curve | secp256r1 | secp256k1 |
| Container | NEP-6 JSON, scrypt | Web3 Secret Storage V3 keystore |
| Address | Base58Check, version 0x35 | 0x + keccak(pubkey)[12..] |
| Transaction | witnesses, magic in the signed hash | EIP-155 / typed, chain id in the signature |

`signer_keys.scheme` + `curve`; `SignerBackendKind` gains `LocalKeystore` rather than overloading `LocalWallet`. The binding rule `node.node_type.family().key_scheme() == key.scheme` is checked **at bind time**, with the picker filtered — today the equivalent check fires at Start, hours after the mistake, and only for one backend kind.

**Duty verdicts for Neo X**, derived from the launch path:

| Duty | Verdict |
|---|---|
| RpcApi, Observer | Launchable |
| State, Indexer | Launchable, **bound to concrete settings** (`eth` namespace + no state pruning; retained receipts/logs) — a duty that changes no setting is a label |
| **Consensus** | **Unsupported.** `role_availability(NeoXGeth\|NeoXReth, Consensus) => Supported` becomes `Unsupported`. Neither launch path emits any validator flag anywhere in `src/`; block production needs a keystore-resident dBFT key inside the client plus on-chain membership, and NeoNexus provisions neither. The option renders disabled with that sentence (G21). |
| Oracle, StateValidator, Notary | Unsupported — RoleManagement is an N3 native contract; the reason is **shown**, because it teaches the N3↔X difference at the moment it matters |

**A Neo X node with a launchable duty needs no key at all**, and the signer section says so rather than rendering an empty custody panel.

**Naming:** the reth-based client is `NeoXReth` in Rust, `neox-rs` in serde, "neox-reth" in log-parser metadata. Pick **neox-rs** everywhere operator-facing and rename the enum variant to `NeoXRs` to match; the `#[serde(rename = "neox-rs")]` already exists because the mismatch was a hazard once.

#### 7.2.7 Neo N3 ↔ Neo X: model the relationship, not the bridge

`networks.parent_network_id` links `neox-mainnet → neo-n3-mainnet`. Used for exactly two things: correct labelling (*"Neo X TestNet — sidechain of Neo N3 TestNet"*) and a **consistency warning** when one environment mixes maturity levels (a Neo X MainNet node alongside an N3 TestNet node in staging is almost always a mistake). Nothing more; it must not imply a data path NeoNexus does not observe.

One incident vocabulary: `HealthState` and `Verdict` are **family-neutral**; the family changes only the *evidence* rendered in one slot. Design rule: any new field that exists on one family and not the other goes in the evidence enum, never in the top-level observation.

### 7.3 The duty × client truth table, derived

`role_availability` is kept — its four doc-commented claims are researched facts about the client software — but becomes **one of four inputs**, not the answer.

```rust
pub enum DutySupport {
    Full,
    Caveat { level: LadderLevel, reason: String, remedy: Option<NextStep> },
    Unsupported { reason: String },
    Unverified { reason: String },
}
pub fn duty_support(node_type: NodeType, duty: NodeDuty) -> DutySupport;
pub fn duty_support_for_node(node: &NodeConfig, duty: NodeDuty, ctx: &NodeContext) -> DutySupport;
```

Four gates, composed worst-first:

1. **`role_availability`** — protocol facts about the client. Unchanged.
2. **`config_generator_emits`** — **probed, not declared.** Renders a synthetic node through the real `ConfigGenerator` and inspects the duty's enable switch (`AutoStart`, `Auto`Verify`, `Enabled`, the neo-go service block), returning Emitted-and-enabled / Emitted-but-disabled / Not-emitted. This is what makes G20 impossible to reintroduce: a generator change that disables a switch changes the matrix, the picker and the CI assertion in the same build.
3. **`signer_route_support(node_type, duty, backend_kind, curve)`** — a pure function extracted from the bodies of `ensure_local_wallet_runtime` and `ensure_sign_client_runtime` and called by **both** the launch path and the UI. Same function, two callers, no second declaration to drift. This is what turns neo-rs and Neo X Consensus from Supported-on-paper into Unsupported-in-fact.
4. **`runtime_layout_satisfied(node)`** — the neo-cli runtime-root gate, evaluated against an actual node, so it is a per-node blocker shown in the editor before Save and in `/readiness`, not only at Start.

#### 7.3.1 The honest matrix, and the one cell we actually fix

**Bold** marks cells where derived truth differs from what `role_availability` declares and the picker offers today.

| Duty | neo-cli | neo-go | neo-rs | neox-geth | neox-rs |
|---|---|---|---|---|---|
| RpcApi / State / Indexer / Observer | Full | Full | Full | Full | Full |
| Consensus | **Caveat → Full (Stage 6)** | Full | **Unsupported** | **Unsupported** | **Unsupported** |
| Oracle | **Caveat — provisioned, disabled** | Full | Unsupported | Unsupported | Unsupported |
| StateValidator | **Caveat — provisioned, disabled** | Full | Unsupported | Unsupported | Unsupported |
| Notary | Unsupported | Full | Unsupported | Unsupported | Unsupported |

**G24 is fixed, not merely surfaced earlier.** `ensure_neo_cli_runtime_root` requires the binary's parent directory to equal the node working directory; managed installs land under `<workspace>/runtimes/…` and copy only the executable, while plugins install under `<workspace>/nodes/<id>/Plugins` — so no chip the editor offers can satisfy the gate, and a private dBFT network needs shell access. The fix is a second install mode:

> **`nodes.runtime_layout = 'per-node'`.** The runtime installer materialises the complete runtime tree into `<workspace>/nodes/<id>/runtime/` (hardlink where the filesystem allows, copy otherwise), sets `binary_path` inside it, and installs plugins beside the binary. The node editor selects this automatically when `duty.requires_signer()` and `node_type == NeoCli`, and `/runtimes` shows which layout each node uses and the disk cost.

After Stage 6 the neo-cli Consensus cell is `Full`. Before it, the cell is `Caveat` with a `NextStep::External` naming the manual placement — honest, and it does not pretend the console can do it.

**neo-cli Oracle / StateValidator** stay `Caveat` until the generator changes: `"AutoStart": false` / `"Nodes": []` and `"AutoVerify": false` are hardcoded in zero-argument functions (one pinned by a test), while the dBFT sidecar *in the same file* varies `AutoStart` by signer — so this is asymmetry, not a platform limit. Stage 6 makes all three vary by signer and wallet, and makes `Nodes` a real `node_peers` kind (`oracle`) with a form field, since `GenerationContext` has no peers slot today and every Start rewrites the sidecar.

**The gate:** a test iterating `NodeType::ALL × NodeDuty::ALL` asserts that `duty_support`'s declared level equals the level obtained by actually rendering the config and calling the real signer-route function. A generator or gate change that makes a duty inert fails CI with the cell named.

---

## 8. Vocabulary

### 8.1 The rule

> **One concept, one word, declared once in code, rendered everywhere from that declaration.**

A word that appears as a string literal in a page is a word that can drift. Every canonical term has a home: an enum's `label()`, a `nav::Destination.label`, or `src/vocabulary/glossary.rs`. Pages read; pages do not author.

### 8.2 Glossary

| Canonical | Definition | Home | Rejected → replacement |
|---|---|---|---|
| **node** | one Neo client process NeoNexus manages | `NodeConfig`, `nodes` | **instance** [118], **EC2 instance** [31], resource, server, workload, VM → node |
| **fleet** | every node in this workspace | `/api/fleet` | estate, cluster → fleet |
| **duty** | the job the operator assigns *locally* | `NodeDuty` (renamed from `NodeRole`) | **role** as a bare UI noun, job, profile, function → duty |
| **designation** | the on-chain `RoleManagement` grant to a key | `ChainRole` | — |
| **Consensus** | the *duty* — configured to run dBFT | `NodeDuty::Consensus` | **⚡ Validator** as a duty label → Consensus |
| **validator** | a *chain status* — this key is in the elected top 7 | `DutyState::Elected` | — |
| **committee member** | a *chain status* — top 21 | `DutyState::CommitteeOnly` | — |
| **No duty** | the rendering of `None` | `html::duty_label(Option<NodeDuty>)` | **Observer**, **Node**, **standard**, **observer**, preselected **rpc-api** → No duty |
| **host** | the machine a node's process runs on | `hosts` | availability zone, `nexus-az-1a`, region, `vpc-*`, `Account:` → host, or deleted |
| **peer workspace** | another NeoNexus this one reads from | `hosts.transport='neonexus-peer'` | federation server, remote server → peer workspace |
| **key** | the custody object that can sign | `signer_keys` | KMS key, CMK, credential, ARN → key |
| **signer backend** | the system that holds keys | `signer_backends` | signer profile, custody provider, KMS, bare **profile** [176] → signer backend |
| **signer binding** | the **exclusive** association of one node with one key | `node_signer_bindings` | IAM Signer Lease, IAM Signer Role, IAM Signer, IAM Instance Profile & Signer Identity, instance profile, signer identity, signer attachment, **lease** → signer binding |
| **NEP-6 wallet** / **keystore** | chain-qualified key containers | `SignerBackendKind` | local encrypted wallet, wallet profile, wallet identity → the qualified term |
| **key id** | the string naming a key inside its backend | `signer_keys.key_id` | ARN, id-as-public-key → key id |
| **client** | which implementation | `ClientKind` (renamed from `NodeType`) | node type, flavour, engine → client |
| **release** | a versioned downloadable artifact | `runtime_releases` | — |
| **runtime** | an *installed* release on a host | `runtime_installations` | **AMI** [7], image, machine image → runtime |
| **binary** | the executable file | `nodes.binary_path` | — |
| **chain** | Neo N3 \| Neo X | `ChainFamily` | — |
| **network** | which instance of that chain | `networks` | — |
| **chain id** | N3 magic / EIP-155 id, always with its source | `networks` | — |
| **tag** | operator `key=value`, from Stage 5 | `node_tags` | **environment** [7] as a node attribute, Production → deleted |
| **status** | what the *supervisor* knows | `NodeStatus` | system status, instance status, **state** → status |
| **health** | what the *chain* says when asked | `HealthState` | — |
| **readiness** | whether config would produce a working node | `src/diagnostics/` | Systems Manager, OpsCenter, OpsItem → readiness / finding |
| **check** | one named test with an outcome | `DiagnosticCheck` | System & Instance Checks → checks |
| **alert** | a notification NeoNexus sends | `src/alerts/` | **alarm**, SNS, notification topic → alert / rule / delivery channel |
| **rule** | the condition that produces an alert | `alarm_rules` | metric condition → rule |
| **event** | a record in the journal | `RuntimeEvent` | **CloudTrail**, Event History, audit journal → Timeline; **User Identity** → **Actor** |
| **fast-sync archive** | **inbound** third-party chain data | `fast_sync_archives` | **snapshot**, EBS Snapshots, Create Snapshot Backup, restore point, checkpoint → fast-sync archive |
| **workspace backup** | **outbound** export of the workspace DB | `/workspace?tab=backup` | snapshot → workspace backup |
| **support bundle** | checksummed diagnostics zip | `WorkspaceSupportBundleExporter` | kept, unambiguous |
| **data directory** | where a node keeps chain data | `nodes.data_dir` | volume, `vol-*`, `/dev/xvda`, IOPS, Attached, EBS → data directory, or deleted |
| **listening addresses** | what the managed config binds | derived from the render | security group, Rule Status: Open, `0.0.0.0/0` → deleted |
| **API token** | a credential for the machine API | `api_tokens` | IAM API Credentials, **IAM** [34] → API token |
| **actor** | who did it | `runtime_events.actor_kind` | `arn:neo:iam::nexus:operator` → actor |

**Two rulings against what the register called defensible:**

**"instance" goes.** In a greenfield product it would be fine. Here the entire domain layer — `NodeConfig`, `NodeStatus`, `NodeType`, `NodeRole`, the `nodes` table, every `--node-*` flag, and Neo's own documentation — already says node. "Instance" is not a competing concept; it is a second name for a thing that has a first name in 100% of the non-presentational code. The code already voted.

**"lease" goes**, against the register's provisional defence. Exclusivity is genuinely enforced (`signing/isolation.rs:66-72`, `migrations.rs:112`), and that half is true. But a lease also *expires* and is *renewed*, and nothing in NeoNexus does either. That surplus connotation is the generative cause of three separate fabrications: `NeoNode-SignerLease-Expiring` / `SignerLeaseTTL < 300s`, the unconditional `● OK (Lease Valid)` tile on a function that receives no signer argument, and the AutoRenew vocabulary around them. A word that invents its own fields will invent them again. The property we have is carried completely by *"the binding is exclusive"*, in plainer English, at no cost. **Condition for return:** a backend that issues genuinely time-bounded grants, bound to a real `expires_at_unix` column, for that backend only.

### 8.3 The AWS ruling

> A borrowed term stays only if **(a)** NeoNexus implements the mechanism it names — not something analogous, the mechanism; **(b)** the term is not already occupied in the Neo domain; **(c)** no ordinary English word is as accurate. Fail any test and it goes. Where the term names something that does not exist it is **deleted with no replacement** — inventing a NeoNexus-flavoured name for a non-feature just launders the same lie.

**Vendor proper nouns fail (c) categorically** and are deleted without case-by-case review: CloudWatch, CloudTrail, Systems Manager, OpsCenter, SNS, IAM, KMS, EC2, EBS. ~135 occurrences removed by rule, not argument.

**One survivor, and it proves the rule.** `IacFormat::CloudFormation` at `src/web/api/iac.rs:116` really does emit CloudFormation YAML. Calling that output "CloudFormation" is the correct name of the artifact, beside `terraform`, `k8s`, `docker`. It passes all three tests and stays **in exactly one place: the format selector and the download filename.** What goes is the `/config` breadcrumb "CloudFormation & Config", the service-menu entry, and the two `☁️ Export CloudFormation` buttons, which describe a fleet IaC export and belong on an Export control with a picker, not as a page identity. Every other borrowed term fails (a) or (c) — that is not a coincidence; it is what "costume" means.

**What replaces the deleted chrome:**

| Deleted | Replaced by | Source of truth |
|---|---|---|
| Security-group table | **Listening addresses**, each marked *managed* or *overridden by args*; a `--config` arg collapses the card to *"This node uses an operator-supplied config file; NeoNexus does not know its bind addresses."* | the generated config + `node.args` |
| EBS volume card | **Data directory** — real path, storage engine, free space | `nodes.data_dir`, `node_samples.disk_free_bytes` |
| Placement / region / account | **Host** | `hosts` |
| Tags tab | **Nothing**, until `node_tags` exists in Stage 5 | — |
| `Health 2/2` pill | **Nothing.** A global pill on a function with no fleet parameter cannot be made honest; `/`'s attention queue is the right widget. | — |
| `x86_64 Native Sandbox` / `Hypervisor: NeoNexus Workbench Daemon` | *"Runs as a child process of NeoNexus on {host}"*, with the real `std::env::consts::ARCH` if an architecture is shown at all | `Command::new` |

### 8.4 The message shape, and the lint

> **`{Action} {outcome} for {node name}: {reason}. {Imperative next step}.`**

Typed, so the shape cannot be bypassed: `back_to_node(id, &format!("failed: {error}"))` stops compiling.

```rust
pub struct OperatorMessage { action: &'static str, subject: Subject, outcome: Outcome,
                             reason: String, next: NextStep }
pub(crate) fn back_to_node(id: &str, message: OperatorMessage) -> Response;
```

Rules: name the node by name, never by id, never "the instance" · a refusal says what was refused, verb first ("Start refused", not "failed") · **never name a CLI flag or a Rust API in web copy** (today: *"…or run --node-rebind-runtime before launch"*, *"Add one through the Rust API"*) · every refusal ends in an action here or an explicit statement that the console cannot do it and what can · a count of failures names the failures · **no emoji in operator copy** · **no claim about a mechanism unless the mechanism exists and can be named** (banned: *immutable, cryptographic, real-time, verified, encrypted, guaranteed, CloudTrail-grade, one-click*) · severity words come only from §4.3 · internal invariants get a distinct shape (*"NeoNexus could not handle that request (unknown action "xyz"). This is a bug; see Timeline."*) · CLI and web say the same words in their own registers, from the same `label()` methods.

Three before/after rewrites of real strings:

> **Config drift badge.** Before: `● In Sync`, from `Path::is_file()`, under a header claiming "configuration drift verification", beside `("KMS Encryption", "AWS-KMS (active)")`.
> After: **Not checked** — *NeoNexus has not compared this file to the config it would write.* → *Check for drift*; and the encryption row becomes **Permissions: 0600 · Not encrypted at rest.** *These files contain the wallet unlock password in plaintext; exclude the workspace folder from unencrypted backups.*
> This is the clearest case in the whole document: the "before" asserts a verification that never ran and an encryption that does not exist, over a file holding a plaintext password. The "after" is shorter, entirely true, and the only version that changes behaviour correctly.

> **No signer backend.** Before: *"No signer profile is configured. Configure the signer registry before binding this node."* — no form, no link, no named mechanism.
> After (until Stage 7): **No signer backend is available.** *NeoNexus reads signer backends from its own process environment at startup, and this build cannot add one from the console. Set the backend environment variables and restart NeoNexus.* After Stage 7: **No signer backend yet.** → *Add signer backend*.

> **Batch result.** Before: *"Batch action 'start' executed on 3 instances: 2 succeeded, 1 failed."*
> After: **Start: 2 of 3 nodes started.** *seed-03 did not start — no key is bound to it.* → *Open seed-03*.

**`src/vocabulary/`** is built in the image of the existing `src/source_quality/` gate (`rules.rs` / `scan.rs` / `checker.rs` / `model.rs`, a `--vocabulary` / `--vocabulary-json` CLI pair, a `VocabularyReport` with `exit_code()`, and a `src/ci_policy/` requirement asserting `ci.yml` runs it). `docs/VOCABULARY.md` is **generated** from `glossary.rs` and a test asserts the checked-in file matches regeneration, so the glossary cannot drift from the lint that enforces it.

| Id | Rule | Scope |
|---|---|---|
| R-V1 | every rejected synonym appears in no string literal; finding carries the replacement | `src/web/`, `src/cli/output*` |
| R-V2 | no literal `OK`/`Healthy`/`Running`/`Passed`/`In Sync`/`Active`/`Armed`/`Open`/`Attached`/`Valid`/`Operational` status word | `src/web/pages/` |
| R-V3 | no `badge running` / `status-dot …` class literal outside `src/web/html/` | `src/web/pages/` |
| R-V4 | no emoji codepoint in any string literal | `src/web/pages/`, `src/web/control/` |
| R-V5 | no `\d+/\d+` adjacent to a status word (`2/2 passed`, `Health 2/2`) | `src/web/` |
| R-V6 | no mechanism claim without a per-claim allowlist naming the mechanism | `src/web/` |
| R-V7 | no `--[a-z][a-z-]+` inside a string literal | `src/web/` |
| R-V11 | `.slug()` / `.persist_key()` never reaches rendered text (catches `("validator", "⚡ Validator")`) | `src/web/pages/` |
| R-V8…10, 12, 13 | **type rules**, not lints: `page_head`/`layout` take `NavKey`; `breadcrumb` is private behind `breadcrumb_for`; flashes take `OperatorMessage`; `duty_label(Option<NodeDuty>)` is the only duty renderer and `NodeDuty::label` is `pub(crate)`; `HealthState` has no `Default`/`From<bool>` | compiler |
| R-V14…18 | **tests:** nav/title parity; breadcrumb containment (every crumb is a registered route **and** a strict prefix, last crumb unlinked, depth-1 yields none); glossary integrity; doc sync; `src/web/html/page.rs` contains no `<a href="/` outside the generated nav (stops the Services menu regrowing) | CI |

**Ratchet, not cliff.** `vocabulary-allow.txt` (`path:line:rule`) seeded with today's full violation set; CI asserts every finding is fixed or allowlisted, **and** that the allowlist's line count is `<= vocabulary-allow.max`, which a PR may lower or leave equal and **never raise**. New code is held to the full rule immediately; the number in the repo is a visible debt counter.

### 8.5 Renames, behind frozen persist keys

`NodeRole → NodeDuty`, `NodeType → ClientKind`, `FastSyncSnapshot → FastSyncArchive`, `NodeStatus::Error → Failed`, plus the route renames of §4.4.

**Frozen and pinned by test:** every `persist_key()`/`slug()` string — **including `Consensus => "validator"`** — every `serde` rename including `neox-rs`, every `EventKind` discriminant, every `ChainRole` discriminant (consensus-visible), and every CLI flag name. This is a display-layer rename: a workspace DB written by the old build must open unchanged in the new one, and a script parsing `--*-json` must keep working.

---

## 9. CLI parity

> **Every destination is a noun command, every POST is a verb command, and the route, the CLI command and the JSON come from one declaration.**

### 9.1 The registry

`src/capabilities.rs`:

```rust
pub struct Capability {
    pub id:         &'static str,        // "observe.node"
    pub question:   &'static str,        // "What is this node's chain state?"
    pub web:        WebSurface,          // Route("/nodes/{id}?tab=health") | None(&'static str)
    pub cli:        CliSurface,          // Command("node chain <id>")     | None(&'static str)
    pub json:       JsonShape,           // Shared(fn)                     | None
    pub scope_keys: &'static [ScopeKey], // which of node/network/host/at/window it parses
}
pub const CAPABILITIES: &[Capability] = &[ /* … */ ];
```

`nav::render` reads it. The router is asserted against it. The CLI dispatcher is asserted against it. `scope_keys` is what `link_to` drops keys against, so `q` vs `query` is a compile-and-test failure rather than a silent full-journal dump.

### 9.2 The command surface

```
neonexus now [--json]                                   # exit 2 any Critical, 1 any Warning, 0 else
neonexus node    list [filters] | show <id> | chain <id> | duty <id> | config <id> [--render|--diff]
                 | process <id> | timeline <id> [--since] | logs <id> [--since|--around]
                 | create | edit <id> --set k=v | duty-set <id> <duty> | bind-signer <id> <backend> <key>
                 | start|stop|restart|smoke|retire <id> | history <id> | reconcile-config <id>
neonexus network list | show <id> | create | edit <id> | identity <id> | governance <id>
                 | designations <id> | topology <id> | refresh <id> | merge <a> <b>
neonexus host    list | show <id> | add | edit <id> | delete <id> | probe <id>
neonexus health  history <node-id> --since <dur>
neonexus alarm   list [--state alarm|pending|no-data] | rule list|create|edit
                 | route list|create|test <id> | silence <node|network> <duration>
neonexus timeline list [filters] | export
neonexus readiness run [--node|--network] | export | support-bundle
neonexus duty    support [--json]
neonexus runtime catalog list|add | release list|verify <id> | install | upgrade run|status|history
                 | rollback <node>
neonexus archive list [--network] | verify|download|cache|apply --node <id>
neonexus signer  backend list|create | key list|generate|bind|unbind|rotate
neonexus neox    init --node <id> [--reinitialise] | enode --node <id>
neonexus workspace backup export|validate|import | integrity | settings get|set
```

Every read takes `--json`. **Exit codes carry meaning and never collapse unknown into bad:** **0** healthy/synced · **1** degraded, not-designated, drifted, or firing · **2** unknown, unreadable, or no key to compare. The last is the specific fix for `cli/actions/chain.rs:23-26`, where omitting a public key exits 1, identical to "not designated".

**Endpoint forms become node-keyed.** `--peer-health <rpc-endpoint> [neo-n3|neo-x]` becomes `neonexus node chain <id>`: `node_rpc_endpoint(node)` and `node_type.family()` both already exist, and making an operator retype an endpoint the database holds is the CLI form of the same defect (G12). `--*-endpoint <url> <family>` survives for ad-hoc use.

### 9.3 The five gates

1. **Route parity** — every `WebSurface::Route` resolves in the router; every registered GET page route appears in `CAPABILITIES` **or** `nav::SECTIONS`; **every `control_form` action string in `src/web/pages/` resolves to a registered POST route.** That last one alone catches the backup button that 404s (G26).
2. **CLI parity** — every `CliSurface::Command` dispatches; every dispatcher command is in `CAPABILITIES`.
3. **Nav reachability** — every registered GET page route has a nav entry or a declared inbound link from a rendered page. `/backup` and `/settings/api-tokens` fail this today, which is the proof it bites.
4. **JSON parity** — for a fixture workspace, `GET /api/…` and the CLI `--json` path produce **byte-identical** JSON from the same serializer function. Drift between console and CLI becomes a diff, not a discovery. Extended: for one node, the `ChainFinding` codes in the rendered HTML, in `node chain <id> --json`, and in the events emitted for the same sample are the **same set**.
5. **Event coverage** — every `EventKind` variant has a non-test construction site (51 of 93 fail today, G36), and every kind `/timeline` can filter by is constructible.

Plus the **single-surface ratchet**: the count of `CAPABILITIES` rows that are not both-surfaces is `<= capabilities-single-surface.max`, checked in, lowerable, never raisable. Each `None(reason)` carries a justification string that appears in the CLI's `--help` and on the page.

### 9.4 What the registry drags onto a surface

Nine CLI-only capabilities gain routes: support bundle → `/readiness`; readiness report export → `/readiness`; event journal export → `/timeline`; workspace integrity → `/workspace`; config drift + reconcile → node Config tab; alert preview → `/alerts` route test; wallet validation report → `/signers` import; archive catalogue + `compatible_entries` → `/archives`; release verification → `/runtimes`. Backup import → `/workspace`. Private-network planner + deployment exporter → `POST /networks` (§4.5). Peer workspace create/edit/delete → `/hosts`. `quarantined_runtime_spec` becomes `pub` on `WorkspaceQueries` → `/workspace?tab=integrity` with re-apply.

Route-only capabilities gain flags: plugin install, archive verify/download/cache/apply, signer key generate/state/policy, signer backend create, host toggle, agent token/healing/ping, settings get/set.

---

## 10. Delivery plan

Nine stages. Each compiles, passes, and ships on its own. Stage ordering follows the register's own suggested sequencing — *delete fabrications, build observation, make the first run work, add the parity gate, then data model and naming* — and deliberately does **not** put the `nodes` rebuild first (§2.2).

### Stage 0 — Subtract *(days, no schema)*

Delete the `Services ▾` menu · the four `● OK` alarm rows and the `0 In alarm` / `4 OK` tiles and their landing-page mirror · the Monitoring, Networking, Storage and Tags tabs · the security-group / EBS / instance-type / placement / hypervisor chrome on all five surfaces · the `Health 2/2` header pill · the `contains("Hermes")` actor heuristic and the "CloudTrail-grade immutable" claim · the KMS/SecureString assertions on `/config` · `/metrics` (301 → `/monitor`) · the compact-density drawer · the fabricated 60-minute chart, the inert time pills and the constant watchdog tile.
Land `src/vocabulary/` with R-V1/2/4/5/6/7, a seeded allowlist and `vocabulary-allow.max`.

**True afterwards:** the console stops lying during incidents. Purely subtractive; no migration; ships in days.
**Closes:** G1, G2, G3 (fabrications), G7, G8 (identity), G42, G47 (drawer + duplicate page), and the KMS half of G5.

### Stage 1 — Types and the parity skeleton *(no new data)*

`NavKey`; `page_head`/`layout`/`breadcrumb_for` signatures; `Scope` + `link_to` + `scope_chip`; `OperatorMessage` and the flash signature; `Observation<T>`, `Evidence`, `Verdict`, `NextStep`, `HealthState` with `NotChecked`/`Stale` derivable from `Option<&RpcHealthRecord>` today; `html::duty_label(Option<NodeDuty>)`; `CAPABILITIES` + the five gates with today's violations allowlisted and ratcheting.

**True afterwards:** fabrication becomes unrepresentable in the renderer rather than merely linted; `rpc_port == 0` reads **Not checked**; per-node scope is carried by a type; Theme 4 stops regrowing.
**Closes:** G6, G9, G43, G46, and structurally prevents G1/G2 recurring. Puts G26/G30/G36 under a failing-and-ratcheting gate.

### Stage 2 — Observation core *(additive tables only — no `nodes` rebuild)*

`src/observe/` with `head`/`head_time`/`peers`/`peers_detail`/`pool`/`identity`/`anchor`/`process` classes; the scheduler with backoff and the bounded pool; `node_samples`, `node_health_state`, `node_health_transitions`, `node_rpc_capabilities`, `node_sample_rollups`, `network_heads`, `reference_endpoints`; `derive.rs`; the nine-state machine with `StallScope` and `suspected_cause`; the reference ladder keyed on **observed** `chain_key`; retention + incremental vacuum. The long-lived metrics collector replaces every `MetricsCollector::new(Duration::ZERO)`.

Surfaces: `/` attention queue + chain-stall collapse + not-checked band + networks strip; node **Health** tab; `/health` with real rollups and real range parameters; `/nodes`'s Process/Chain columns; `neonexus now`, `node chain <id>`. `rpc_health_checks` dual-written for one release.

**True afterwards:** *"Running but not syncing"* is detectable. Head lag, stall scope, peer expectation, mempool utilisation, RPC latency and disk are real. The landing page becomes the product.
**Closes:** G3 (the chart), G4, G11, G12, G17, G47 (`/metrics`). Partially G15/G16 (chain series + the adapter's first honest consumer).

**This is the largest and highest-value stage. It is second, not fifth, and it needs no migration.**

### Stage 3 — Alarms and routes

`alarm_rules` / `alarm_states` / `alarm_transitions` / `alert_routes`; the evaluator with hysteresis, flap suppression and `NoData`; scope resolution over `{all, node, host, chain-family, duty, node-type}` (environment/tag arrive in Stage 5); scoped routing; ten seeded rules, disabled; `/alerts` rebuilt with four tabs and route test-fire; silence writing `suppressed_until_unix`; heartbeat routes (§6.8).

**True afterwards:** four permanently-green rows become real, scoped, reviewed-before-enabled rules with a delivery journal, and "Critical to PagerDuty, Warning to Slack" is two rows.
**Closes:** G1 (fully), G14. Part of G30 (alert preview).

### Stage 4 — Consensus liveness, governance and designation

`governance` / `governance_deep` / `designation` / `consensus` / `state` sample classes, keyed on `chain_key`; `chain_governance_samples`, `chain_candidate_samples`, `chain_blocks`, `node_designations`, `node_designation_transitions`; the block walk with epoch, gap and reorg handling; the duty axis and the six-rung ladder; `duty_support` derived from four gates (§7.3).

Surfaces: node **Duty** tab with the ladder; `/duties` with the derived matrix and the picker generated from it; `/chain`-equivalent governance rendered under a synthetic chain key until Stage 5 re-keys it to `network_id`; the revocation and slots-skipped alarms; `neonexus duty support --json` so CI can assert the matrix.

**True afterwards:** *"Your Oracle designation was revoked at 03:14"* is sayable. *"You are elected, keyed, and produced 0 of 14 assigned slots"* is sayable, with an alarm. `src/chain_state/` gets its second surface. The duty picker stops offering Neo X Consensus.
**Closes:** G13, G21, and the *reporting* half of G20 (the ladder names `AutoStart = false` as the blocking rung; Stage 6 changes the generator).

### Stage 5 — Networks, hosts and the `nodes` rebuild

Migrations 001–007 and 012–016 (§3.3): pre-flight conflict scan; `hosts` + `local` + `host_port_reservations` + `host_probes`; `environments` + `node_tags`; `networks` + seed 4 public + **quarantine one row per private node** using Stage-2 observation evidence + the merge action; `nodes` rebuild with `network_id`/`host_id`/`data_dir`/`metrics_port`/`authrpc_port`/`runtime_layout`/`revision`/timestamps and the revision triggers; `node_duties`; `node_peers`; `node_supervision_overrides`; `signer_backends`/`signer_keys` + the binding rebuild; `node_config_renders`; `archive_applications`; `runtime_events` actor columns; `api_tokens` scoping; `node_neox` + `argv_read`.

`ConfigGenerator::render_for_node` takes a mandatory `&NetworkProfile`; every `effective_*` helper and the constants move behind the seeder; **Start refuses on `complete = 0`**; `node_endpoint(host, node, port)` replaces the `127.0.0.1` interpolation; federation merges into `/hosts`.

Surfaces: `/networks` + the five sub-destinations; `/hosts`; `/timeline` with the actor column and kind filter; the node **Config** / **Process** / **Timeline** tabs; `/readiness` with the new checks; alarm scopes gain environment and tag.

**True afterwards:** unbootable private configs become impossible. Chain identity is one row read by one loader. A fleet can express where it runs. History and drift are real.
**Closes:** G5, G18, G22, G23 (peers/WS/namespaces), G29, G32, G33, G34, G35, G37, G38, G39, G40. Partially G19.

**This is the riskiest stage and it is deliberately fifth.** Everything that would have told the operator earlier already shipped, so if this stalls in review the product is already materially better.

### Stage 6 — First run and private networks

Seeded default runtime catalog profile; the five-step **add node** flow with inline runtime install; the six-step **private network planner** at `POST /networks` (key generation, committee derivation, cross-host port allocation, seed-list derivation, one-transaction materialisation, launch-pack export); `runtime_layout = 'per-node'` materialisation for neo-cli signing duties; the generator changes making neo-cli `AutoStart`/`AutoVerify` vary by signer and `Nodes` a real `node_peers` kind with a form; Neo X `geth init` action, init state machine, enode derivation, INV-X1/2/3.

**True afterwards:** a newcomer can stand up an N3 RPC node and a 4-node private dBFT network from the console, without a shell and without sourcing constants from documentation.
**Closes:** G19, G20 (fully), G24, G25, G27, and the Neo X remainder of G23.

### Stage 7 — Supply and workspace parity

`/signers` backend create, key-id column, curve-filtered picker, wallet validation report, rotation; `/runtimes` catalogs + releases + plugin packages + upgrade runs and attempts with stage/message/rollback and `require_signed_catalog` finally read; `/archives` compatibility filter + application history; `/workspace` with backup export **and** import, integrity (table list generated from `create_tables`), support bundle, API tokens; `EventKind` construction sites for every variant; ratchet the single-surface count and the vocabulary allowlist toward zero.

**Closes:** G26, G28, G30, G31, G36, G41. Completes G45's direction split.

### Stage 8 — Renames and vocabulary at full strength

`NodeRole → NodeDuty`, `NodeType → ClientKind`, `FastSyncSnapshot → FastSyncArchive`, `NodeStatus::Error → Failed`, `NeoXReth → NeoXRs`; the route renames with permanent 301s; R-V3/8/9/10/11/12/13/14–18 at full strength; persist keys, serde renames, `EventKind` and `ChainRole` discriminants and CLI flag names frozen and pinned by test; allowlists at 0; `docs/VOCABULARY.md` generated.

**Closes:** G43, G44, G45 (fully), G42 (fully).

### 10.1 Dependency graph, stated plainly

```
0 ──► 1 ──► 2 ──► 3
             │    │
             ├────┴──► 4 ──┐
             │             ├──► 6 ──► 7 ──► 8
             └──────► 5 ───┘
```

- **2 does not depend on 5.** That is the whole point of the observed `chain_key` (§2.2).
- **4 does not depend on 5.** Governance and designation key on `chain_key` and are re-keyed to `network_id` inside 5's migration.
- **6 depends on both 4 and 5**: the planner needs `networks` rows and needs `duty_support` to know which client can do what.
- **5's migration is informed by 2's samples.** Running 2 for at least one observation cycle before 5 lands is not required but is strongly recommended: it is what turns the quarantine merge proposals from empty into useful.

### 10.2 Realism

Stages 0 and 1 are days each. Stage 2 is the single largest body of work in the plan — a sampler, a scheduler, a derivation module, a nine-state machine, six tables, retention, and four surfaces — and should be split into two landable halves (`sample+store+node tab`, then `derive+classify+rollups+attention queue`) with the first half dual-writing `rpc_health_checks`. Stage 5 is the second largest and is a schema rebuild of the product's central table; it should land in a release of its own with the pre-flight scan run against real workspaces first. Stage 4's block walk needs a fixture captured from a real validator on a chain that has experienced a view change before `ChainValidatorMissedProposal` is emitted — the counter can render before the event does.

Nothing here is estimated in weeks, because there is no evidence base for that estimate and inventing one would be the same failure this document exists to correct.

---

## 11. What we are deliberately not building

Stated so these are decisions, not gaps. Each names why.

**No transaction signing on any chain.** No vote, no `registerCandidate`, no `designateAsRole`, no key export. `src/chain_state.rs:8-12` is the policy and it holds: NeoNexus reads. It holds no committee key and must not ask for one. Where an action is needed, the page names the transaction, its on-chain price where readable, and who can perform it.

**No bridge.** Neo X is Neo N3's EVM sidechain and there is an official bridge. Rendering its state means indexing events on two chains with reorg handling — a block-explorer capability, not a node-manager one. It is also the largest available surface on which to fabricate numbers, and a "Bridge Health: ● OK" tile would be the most expensive possible lie because it is about money. If an operator runs bridge infrastructure, its nodes are nodes and get managed as nodes.

**No live dBFT view number and no consensus peer identity.** No JSON-RPC method exposes the local consensus state machine's view, and `getpeers` returns addresses rather than public keys. Both are labelled unanswerable on the page, with the observable substitutes named (§7.1.4). Naming the limit is what stops the next vacuum being filled with a green literal.

**No signature attribution.** Which designated keys witnessed a given state root, and which validators signed a given block, would require verifying each signature against each key. Not done, and said.

**No oracle request depth, no per-node oracle response count.** No standard method exposes either.

**No slashing surface.** Neo N3 dBFT has no slashing. Offering one would teach a wrong protocol model.

**No Neo X genesis generation, and no offline genesis-hash computation.** An invented allocation is a healthy-looking chain of one. Offline hashing needs header RLP + keccak and would still not prove what the client did with the file; the file's sha256 is the artefact identity and the chain's block-0 hash comes from the running node.

**No Neo X validator launch.** The duty is removed from the picker with the reason shown, because block production needs a keystore-resident dBFT key inside the client plus on-chain membership and NeoNexus provisions neither.

**No backup of a node's chain data.** NeoNexus cannot do it, `/archives` and `/workspace` both say so in one sentence, and neither implies otherwise.

**No immutability claim on the journal.** `runtime_events` is a plain table with no hash chain, no signature, no trigger, and a live arbitrary-insert path through backup import. The word `immutable` is banned until a `prev_hash`/`entry_hash` chain exists and restore writes to a separate `imported_events` table. Noted as a follow-on, not claimed now.

**No firewall management.** `grep -rni "iptables|pfctl|ufw|nftables" src/` hits UI captions only. The security-group table is deleted with no replacement; listening addresses are what NeoNexus actually knows.

**No instance/volume/IOPS/placement model.** Nothing allocates CPU, RAM or IO, and there is no disk-size measurement outside `node_samples.disk_free_bytes`. Deleted rather than renamed: a NeoNexus-flavoured name for a non-feature launders the same lie.

**No authentication or authorisation model beyond what exists.** There is a login and there are API tokens with namespace confinement. There is no user entity, no role-based access control, and `nodes.owner` is deliberately free text rather than a foreign key — a key to a user entity that cannot be authenticated implies an access control that does not exist. If multi-operator custody is wanted, it is its own design, and it must come with authentication.

**No SPA, no JS framework.** Server-rendered HTML, forms post and redirect. The only JavaScript in the product is `/`'s 15-second poll with a visible timestamp and a Pause control, and a `<noscript>` meta-refresh fallback. Nothing else polls.

**No guided wizard as the primary IA.** Two flows are designed as sequences (§4.5) because they are genuinely sequential and genuinely hard; everything else is a destination you can enter at any point. A wizard-shaped console optimises for the twenty minutes a node is configured over the months it is watched.

**No configurable attention ranking.** §4.3 is fixed so that the layout is the same at 03:00 every time. This is a judgement and it is stated as one.

**No log-derived sync progress.** Deleted, not repaired. Both Neo X parsers gate on strings the clients never emit, neo-rs has no parser, and the covering test's fixtures were written to match the parser. `eth_syncing` and `getblockheadercount` are structured and authoritative; a parser that can only be wrong in ways CI cannot see loses to a better source. Logs keep level, message, and fatal errors from a process that never came up to answer RPC — which is the one thing they are uniquely good at.
