# Changelog

All notable changes to NeoNexus are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [3.2.0] — 2026-07-15

### Added

- Compact **single-line** inventory and fleet `node_row` anatomy (40pt slots)
  after geometry proof: status dot, name, type/net badges, RPC port, status pill.
- Headless operator walkthrough suite (`tests/ui_operator_walkthrough.rs`):
  Comfortable↔Compact chrome invariance, primary surface paint, density reload.
- Compact inventory page-fit unit gate and denser `list_row_frame` vertical
  margins for ≤40pt slots.
- Runtime Install `loading_callout` when package install/download work is in
  progress; disable Install/Download while busy.

### Changed

- `DensityMetrics::COMPACT` list heights: inventory/fleet **40 / 40** (was 44/56);
  journal empty slots remain **52**; chrome remains density-invariant **60 / 28 / 212**.
- Residual view spacing tokenised to `theme::XS` / `theme::SM` on several surfaces.

### Documentation

- `docs/ui-v3-baseline.md` and `docs/ui-system-redesign-v3.1.md` updated for
  Compact single-line contracts and 3.2.0 ship status.

## [3.1.0] — 2026-07-15

### Added

- Full v3.1 UI visual system (PR-01–15): theme density metrics scaffold,
  frozen kit (`list_row_frame`, `confirm_bar`, `page_chrome`, `busy_inline`),
  shell chrome tokens, nodes tab + density persistence (`appearance.ui_density`),
  Settings Storage density control with immediate Compact control metrics.
- `page_chrome` on all primaries and nested hubs; readiness/journal list matrix.
- Density geometry contracts (`tests/ui_density_geometry.rs`).

### Changed

- Home fleet always uses `node_row` matrix (no alternate grid selection geometry).
- Nodes Studio tools migrated to `ToolbarAction` toolbar.

### Documentation

- Approved design: `docs/ui-system-redesign-v3.1.md`.

## [3.0.0] — prior

- Six-primary information architecture, partial widget kit, god-state split,
  headless UI contract tests. See `docs/ui-v3-baseline.md`.

[3.2.0]: https://github.com/r3e-network/neo-nexus/compare/v3.1.0...v3.2.0
[3.1.0]: https://github.com/r3e-network/neo-nexus/compare/v3.0.0...v3.1.0
