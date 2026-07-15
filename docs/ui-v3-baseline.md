# NeoNexus UI v3 Baseline

Started from app version 2.5.3. Domain and CLI behaviour stay frozen across
UI phases; shell, widgets, view presentation, and navigation IA change.
Shipped as **3.0.0** (breaking UI information architecture). Visual system
completed as **3.1.0** (`docs/ui-system-redesign-v3.1.md`): density preference,
page_chrome, list_row matrix, density-invariant chrome 60/28/212.
**3.2.0** unlocks Compact single-line inventory/fleet densification and adds
`tests/ui_operator_walkthrough.rs` regression coverage.

### Density contracts (3.2.0)

| Mode | Controls | Inventory / fleet rows | Journal | Chrome |
|------|----------|------------------------|---------|--------|
| Comfortable | interact 28, pad 14×8, spacing 10×8, nav 34 | two-line 44 / 56 | 52 | 60 / 28 / 212 |
| Compact | interact 24, pad 10×6, spacing 8×6, nav 28 | **single-line 40 / 40** | 52 | **same** |

Headless proof: `cargo test --test ui_density_geometry`,
`cargo test --test ui_operator_walkthrough`, and unit
`compact_inventory_page_fits_workbench_content_column`. Compact `node_row`
uses single-line anatomy (dot + name + type + net + port + status).

## Visual truth

Headless pixel-truth rasterization of a 1280×820 frame (painted fill
rectangles only — not full text). Useful for background-tier and chrome
surface checks.

| Capture | Theme | Path |
|---------|-------|------|
| Phase 1 after | Light | [ui-v3-baseline/phase1_truth_light.png](ui-v3-baseline/phase1_truth_light.png) |
| Phase 1 after | Dark | [ui-v3-baseline/phase1_truth_dark.png](ui-v3-baseline/phase1_truth_dark.png) |
| Latest alias | Light | [ui-v3-baseline/neonexus_truth_light.png](ui-v3-baseline/neonexus_truth_light.png) |
| Latest alias | Dark | [ui-v3-baseline/neonexus_truth_dark.png](ui-v3-baseline/neonexus_truth_dark.png) |

Regenerate with:

```bash
cargo test --release --test ui_visual_truth -- --nocapture --ignored
```

### Phase 1 presentation changes (not fully visible in fill-only PNGs)

- Shared widget kit: badges, node rows, toolbar, filter bar, page header, callout, form primitives
- Overview empty workspace → single Welcome CTA (instead of stacked empty panels)
- Inventory uses card-style node rows with status dots/badges
- Fleet snapshot uses status badges; card rows when the pane is narrow
- Selection panel uses grouped toolbar actions

### Phase 2 IA (landed)

- Sidebar primary nav reduced to **6** destinations: Home, Operations, Nodes, Runtimes, Network, Settings
- Nodes workspace tabs: Studio | Config | Logs | Plugins | Health
- Runtimes gains **Fast Sync** section; Settings gains **Alerts**; Network hub: Federation | Private Net | Wallets
- Legacy top-level views still resolve via `normalize_navigation_for_v3` + `View::primary_nav`
- Keyboard: cycle and number keys (1–6) operate on primary destinations only

### Phase 3 state foundations (landed)

- `ToastStack` mirrors `notice` changes; status bar shows coloured toast chips (auto-expire, sticky errors)
- God-state composition on `NeoNexusApp`:
  - `session: SessionUi` — theme, view, inspector, notice, toasts, node workspace tab, network hub
  - `fleet: FleetUi` — nodes, selection, draft, inventory filters/paging
  - `operations_ui: OperationsUi` — operations section + action queue / readiness / ports / journal list UI
  - `sections: WorkspaceSections` — settings/runtimes/snapshots/monitor/federation/roles tabs
  - `async_bus: AsyncProbeBus` — RPC health, federation, and alert delivery channels + pending sets
- Fleet helpers: `reset_fleet_paging` / `set_fleet_status_filter` / `select_fleet_node` / `running_node_count`

### Phase 4 surface polish (landed)

- **Home**: metrics + host resources + **Next actions** triage (top 5 readiness items) + fleet snapshot
- **Node Studio**: vertical form groups, toolbar actions, locked-node callout
- **Operations**: severity badges, card-style check rows, selected-action detail card, filter bar

### Phase 5 (landed)

- README documents six-primary workbench
- Version **3.0.0**

### v3.1 system design (approved)

Full visual/interaction system redesign (writer–reviewer consensus) lives in
[ui-system-redesign-v3.1.md](ui-system-redesign-v3.1.md). Implementation tracks
the PR Plan there (MS-1 cut line through PR-15).

## Current top-level views (14)

| Persist key | Label | Title | Group today | Inventory |
|-------------|-------|-------|-------------|-----------|
| `summary` | Summary | Overview | Workspace | yes |
| `operations` | Operations | Operations | Workspace | yes |
| `monitor` | Monitor | Resource Monitor | Workspace | yes |
| `logs` | Logs | Runtime Logs | Workspace | yes |
| `nodes` | Nodes | Node Studio | Nodes | yes |
| `runtimes` | Runtimes | Runtime Manager | Nodes | no |
| `snapshots` | Sync | Fast Sync | Nodes | no |
| `plugins` | Plugins | Plugin Manager | Nodes | yes |
| `config` | Config | Configuration | Nodes | yes |
| `federation` | Federation | Federation | Network | no |
| `roles` | Roles | Role Planner | Network | no |
| `wallets` | Wallets | Wallet Profiles | Network | no |
| `alerts` | Alerts | Alert Routing | Network | no |
| `settings` | Settings | Settings | System | no |

## Target IA mapping (Phase 2)

| v3 primary | Absorbs (persist keys) | Notes |
|------------|------------------------|-------|
| Home | `summary` | Fleet health, top readiness actions, quick create |
| Nodes | `nodes`, `plugins`, `config`, `logs` (+ health from monitor) | Node workspace tabs: Overview / Lifecycle / Config / Logs / Plugins / Health |
| Runtimes | `runtimes`, `snapshots` | Install, catalog, upgrades; Fast Sync as section |
| Network | `federation`, `roles`, `wallets` | Segmented: Federation / Private Network / Wallets |
| Operations | `operations`, part of `monitor` | Readiness, queue, ports, safety, journal; resource pressure can stay here or Home |
| Settings | `settings`, `alerts` | Policies, monitors, alert routing, paths, theme, release pack |

### Persist migration rules

- Unknown or legacy keys map via `View::from_persist_key` compatibility aliases
  (e.g. `logs` → Nodes with Logs tab, `alerts` → Settings Alerts section).
- Section persist keys inside multi-section pages keep existing keys where possible.

## Phase boundaries

| Phase | Scope | Domain/CLI |
|-------|-------|------------|
| 0 | Baseline docs + truth PNGs | frozen |
| 1 | Widget library + theme polish; Overview + Inventory pilot | frozen |
| 2 | IA collapse to ≤7 primary nav; node workspace tabs | frozen (routing only) |
| 3 | God-state split + toast stack | frozen |
| 4 | Deep page polish | frozen unless UX requires thin adapters |
| 5 | Docs, cleanup, 3.0.0 | version bump only |

## Known UX pain (baseline)

1. Fourteen peer navigation items for one product.
2. Global Start/Stop/Restart always in the header regardless of context.
3. Thin widget kit (cards + raw egui fields) → admin-tool aesthetic.
4. `NeoNexusApp` holds ~180 mixed UI/domain fields.
5. Repeated filter/paging field clusters per list surface.
