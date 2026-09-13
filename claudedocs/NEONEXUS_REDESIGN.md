# NeoNexus — The Redesign

Produced by a 22-agent design workflow grounded in
[the verified gap register](NEONEXUS_GAP_REGISTER.md). Four whole-product
information architectures were proposed from different organising principles —
chain-first, lifecycle-first, incident-first, duty-first — and each scored by
three judges (an on-call operator, the implementing engineer, a Neo newcomer).
Chain-first won at 83/100 with the others at 82, so this document is the
synthesis: the winner as spine, with the best idea from each runner-up grafted
in and every judge's fatal criticism answered.

It is split by section only to stay inside this workspace's own 1000-line limit
on documentation files (`cargo run -- --source-quality .`).

---

## The design

Read in order. This is the thing you build from.

### [00-preamble.md](redesign/00-preamble.md) · 179 lines
  - 1. What NeoNexus is
  - 1.1 What is wrong today, in one paragraph
  - 2. Principles
  - P1 — Every page has a subject that is a row, and every render function takes it
  - P2 — Absence has a type, and none of its renderings is green
  - P3 — Every non-healthy verdict carries a next action, and the action is a link
  - P4 — Three axes, never fused
  - P5 — Thresholds are properties of the chain, not constants in the binary

### [01-domain-model.md](redesign/01-domain-model.md) · 809 lines
  - 3. Domain model
  - 3.1 Entities
  - 3.2 SQL
  - 3.3 Migration
  - 3.4 The pre-flight conflict scan (step 001)

### [02-information-architecture.md](redesign/02-information-architecture.md) · 780 lines
  - 4. Information architecture
  - 4.1 Navigation
  - 4.2 Every page
  - 4.3 The attention ladder (fixed, not configurable)
  - 4.4 Fate of today's 17 destinations (+2 orphans)
  - 4.5 The two flows that must be designed, not fall out of the structure
  - 5. The node page
  - 5.1 The header — not a tab, never scrolls away

### [03-chain-surfaces.md](redesign/03-chain-surfaces.md) · 550 lines
  - 7. Chain surfaces
  - 7.1 Neo N3
  - 7.2 Neo X
  - 7.3 The duty × client truth table, derived
  - 8. Vocabulary
  - 8.1 The rule
  - 8.2 Glossary
  - 8.3 The AWS ruling

---

## Subsystem specifications

Reference material the design is built on: the observation layer, the target
domain model, the Neo N3 and Neo X chain surfaces, and the vocabulary.

### [spec-00-neonexus-observation-layer-specification-root-ca.md](redesign/spec-00-neonexus-observation-layer-specification-root-ca.md) · 900 lines
  - NeoNexus Observation Layer — specification (root cause R2)
  - 0. Design rules
  - 1. Module layout
  - 2. What is sampled
  - 2.1 Sample classes
  - 2.2 Neo N3
  - 2.3 Neo X
  - 2.4 Cadence is a policy, not a constant

### [spec-01-alarm-state-and-the-state-that-must-not-be-green.md](redesign/spec-01-alarm-state-and-the-state-that-must-not-be-green.md) · 895 lines
  - 7.2 Alarm state — and the state that must not be green
  - 7.3 Hysteresis and flap suppression
  - 7.4 Routing
  - 7.5 Seeded built-in rules
  - 8. Prometheus exposition
  - 9. Cost control and degradation
  - 9.1 Scheduler, not a spin loop
  - 9.2 Failure backoff

### [spec-02-nodes-rebuild.md](redesign/spec-02-nodes-rebuild.md) · 854 lines
  - 005 — `nodes` rebuild
  - 007 — signer custody
  - 008 — observation
  - 009 — alarms and routes
  - 010 — chain state
  - 011 — runtimes
  - 012 — renders and snapshots
  - 013 — events

### [spec-03-gates-the-model-makes-enforceable.md](redesign/spec-03-gates-the-model-makes-enforceable.md) · 889 lines
  - Gates the model makes enforceable
  - 9. Queries the model must answer
  - 10. Staging
  - The Neo N3 Chain Surface: Design Specification
  - 0. Three rules this surface is built on
  - 1. The read inventory
  - 1.1 Neo N3 — every node, every duty
  - 1.2 Neo N3 — duty-specific

### [spec-04-the-networks-entity.md](redesign/spec-04-the-networks-entity.md) · 466 lines
  - 1.1 The `networks` entity
  - 1.2 Flag-value extraction (the missing primitive)
  - 1.3 Resolution, validation, reporting
  - 1.4 The private-network safety property
  - 2. Bootstrapping
  - 2.1 Datadir as data
  - 2.2 The `geth init` action
  - 2.3 Init state

### [spec-05-neonexus-vocabulary-and-copy-specification.md](redesign/spec-05-neonexus-vocabulary-and-copy-specification.md) · 895 lines
  - NeoNexus Vocabulary and Copy Specification
  - 1.1 The managed thing
  - 1.2 What a node is for
  - 1.3 Where a node runs
  - 1.4 Custody: signer, wallet, key, binding
  - 1.5 Software: runtime, release, binary, client
  - 1.6 Chain, network, chain id, tag
  - 1.7 Observation: status, health, readiness, check
