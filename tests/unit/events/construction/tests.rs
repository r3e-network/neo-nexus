//! Every event kind must have somewhere that constructs it.
//!
//! 51 of the 93 variants had no construction site anywhere in `src/`. Two
//! flavours, and both mislead:
//!
//! * operations that genuinely run and silently skip the journal —
//!   `--import-backup` and `--reconcile-node-config` mutated the workspace with
//!   no audit entry at all;
//! * kinds naming features with no caller whatsoever, which read as capability
//!   to anyone browsing the enum.
//!
//! Nothing failed when a kind went unconstructed, so the list only ever grew.
//! This is the gate: a variant either gets emitted somewhere, or it is listed
//! below as deliberately reserved, with a reason.

use std::{collections::BTreeSet, fs, path::Path};

use crate::events::EventKind;

/// Kinds that exist for a feature this build does not perform.
///
/// Each entry is a promise to either implement or delete, and the reason is
/// part of the entry. Listing one is a deliberate act; *not* listing one and
/// never emitting it is the silence this test exists to break.
const RESERVED: &[(&str, &str)] = &[
    // Ports are assigned by the planner during node creation and reported in
    // the form, not journalled — the node-created entry carries them.
    (
        "node-ports-assigned",
        "the port planner reports through the form, not the journal",
    ),
    // The launch-pack validator is CLI-only, and the pack it validates cannot
    // be produced by this build at all (G19).
    (
        "private-network-launch-pack-validated",
        "--validate-launch-pack does not journal, and nothing can produce a pack (G19)",
    ),
    // The private-network signer sidecar: a whole feature family with no
    // lifecycle to emit from. Either the sidecar ships or these names go.
    (
        "private-network-signer-sidecar-started",
        "the signer sidecar has no lifecycle in this build",
    ),
    (
        "private-network-signer-sidecar-stopped",
        "the signer sidecar has no lifecycle in this build",
    ),
    (
        "private-network-signer-sidecar-exited",
        "the signer sidecar has no lifecycle in this build",
    ),
    (
        "private-network-signer-sidecar-start-failed",
        "the signer sidecar has no lifecycle in this build",
    ),
    (
        "private-network-signer-sidecar-execution-blocked",
        "the signer sidecar has no lifecycle in this build",
    ),
    (
        "private-network-signer-sidecar-policy-updated",
        "the signer sidecar has no lifecycle in this build",
    ),
    (
        "private-network-signer-sidecar-health-checked",
        "the signer sidecar has no lifecycle in this build",
    ),
    // Runtime lifecycle detail the upgrade engine does not record (G31): it
    // downloads and reconciles, and reports only a fleet-level batch summary.
    (
        "runtime-downloaded",
        "the upgrade engine records only a batch summary (G31)",
    ),
    (
        "runtime-state-reconciled",
        "the upgrade engine records only a batch summary (G31)",
    ),
    // Release packaging runs in CI against a checkout, not against a workspace,
    // so there is no journal to write to.
    (
        "release-packaged",
        "packaging runs against a checkout, not a workspace",
    ),
    (
        "release-package-verified",
        "verification runs against a checkout, not a workspace",
    ),
    // Pruning the journal is a maintenance operation with no surface yet.
    ("events-pruned", "no prune control exists on any surface"),
    // Per-client lifecycle detail. NeoNexus supervises a process; it does not
    // observe the client loading its own plugins or initialising its own chain,
    // and would have to parse logs per client to do so.
    (
        "neo-cli-plugin-loaded",
        "NeoNexus does not observe a client loading its own plugins",
    ),
    (
        "neo-cli-plugin-unloaded",
        "NeoNexus does not observe a client unloading its own plugins",
    ),
    (
        "neo-go-module-enabled",
        "NeoNexus does not observe neo-go enabling its own modules",
    ),
    (
        "neo-rs-consensus-started",
        "NeoNexus does not observe neo-rs starting consensus",
    ),
    (
        "neox-geth-chain-initialized",
        "NeoNexus does not run or observe geth init",
    ),
    (
        "neox-reth-snapshot-created",
        "NeoNexus does not drive reth snapshots",
    ),
    // The exporter and scrape families belong to the unwired adapter hierarchy.
    ("metrics-exporter-started", "no exporter is started (G16)"),
    ("metrics-exporter-failed", "no exporter is started (G16)"),
    (
        "prometheus-scrape-completed",
        "NeoNexus is scraped; it does not scrape (G16)",
    ),
    (
        "prometheus-scrape-failed",
        "NeoNexus is scraped; it does not scrape (G16)",
    ),
    // Log handling: the collector reads a tail on an interval. It does not
    // rotate, archive, or announce a parser.
    (
        "log-parser-initialized",
        "parsers are selected per read, not initialised once",
    ),
    (
        "log-rotation-triggered",
        "NeoNexus does not rotate logs; the client or the host does",
    ),
    ("log-archive-created", "NeoNexus does not archive logs"),
    // Plugin dependency resolution and module loading are the client's, not
    // this product's.
    (
        "plugin-version-mismatch",
        "no version comparison runs against an installed plugin",
    ),
    ("module-enabled", "duplicate of plugin-updated; unused"),
    (
        "module-load-failed",
        "NeoNexus does not observe a client failing to load a module",
    ),
    (
        "plugin-dependencies-resolved",
        "no dependency resolution runs",
    ),
    (
        "plugin-configuration-validated",
        "plugin configuration is validated as part of the node config",
    ),
    // Private-network authoring: the planner and the deployment exporter are
    // implemented and have no callers outside tests, so nothing can emit these.
    (
        "private-network-materialized",
        "the planner has no caller (G19)",
    ),
    (
        "private-network-launch-pack-exported",
        "the deployment exporter has no caller (G19)",
    ),
    // Per-client metrics adapters: the substantive hierarchy is never
    // constructed, so nothing reaches the point of journalling (G16).
    (
        "neo-cli-metrics-exported",
        "the adapter is never constructed (G16)",
    ),
    (
        "neo-go-metrics-collected",
        "the adapter is never constructed (G16)",
    ),
    (
        "neo-rs-metrics-normalized",
        "the adapter is never constructed (G16)",
    ),
    (
        "neox-geth-metrics-exposed",
        "the adapter is never constructed (G16)",
    ),
    (
        "neox-reth-metrics-exposed",
        "the adapter is never constructed (G16)",
    ),
];

/// Source text of everything under `src/`, minus the enum's own definition.
///
/// The definition lists every variant by construction, so including it would
/// make this pass for free.
fn production_sources() -> String {
    fn walk(dir: &Path, out: &mut String) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs")
                && !path.ends_with("events/kind.rs")
            {
                if let Ok(text) = fs::read_to_string(&path) {
                    out.push_str(&text);
                    out.push('\n');
                }
            }
        }
    }
    let mut text = String::new();
    walk(Path::new("src"), &mut text);
    assert!(
        text.len() > 100_000,
        "only {} bytes of source read; the scan is looking in the wrong place",
        text.len()
    );
    text
}

/// `EventKind::NodeStarted` → `NodeStarted`, for grepping.
fn variant_name(kind: EventKind) -> String {
    format!("{kind:?}")
}

fn is_constructed(sources: &str, kind: EventKind) -> bool {
    sources.contains(&format!("EventKind::{}", variant_name(kind)))
}

#[test]
fn every_event_kind_is_either_constructed_or_deliberately_reserved() {
    let sources = production_sources();
    let reserved: BTreeSet<&str> = RESERVED.iter().map(|(label, _)| *label).collect();

    let unconstructed: Vec<String> = EventKind::ALL
        .iter()
        .copied()
        .filter(|kind| !reserved.contains(kind.label()))
        .filter(|kind| !is_constructed(&sources, *kind))
        .map(|kind| format!("{} ({})", kind.label(), variant_name(kind)))
        .collect();

    assert!(
        unconstructed.is_empty(),
        "these event kinds are never constructed anywhere in src/, so they name capability \
         that does not exist. Emit them, delete them, or add them to RESERVED with a \
         reason:\n  {}",
        unconstructed.join("\n  ")
    );
}

/// A reserved entry that *is* now emitted has to leave the list, or the list
/// becomes a place where obsolete excuses accumulate.
#[test]
fn nothing_reserved_is_actually_constructed() {
    let sources = production_sources();
    let stale: Vec<String> = RESERVED
        .iter()
        .filter_map(|(label, reason)| {
            let kind = EventKind::ALL.iter().find(|kind| kind.label() == *label)?;
            is_constructed(&sources, *kind)
                .then(|| format!("{label} — reserved as \"{reason}\", but it is emitted now"))
        })
        .collect();
    assert!(
        stale.is_empty(),
        "RESERVED holds kinds that are constructed after all; delete these entries:\n  {}",
        stale.join("\n  ")
    );
}

/// A reserved label naming no variant is a typo that silently disables the
/// check for whatever it was meant to cover.
#[test]
fn every_reserved_label_names_a_real_variant() {
    let known: BTreeSet<&str> = EventKind::ALL.iter().map(|kind| kind.label()).collect();
    let unknown: Vec<&str> = RESERVED
        .iter()
        .map(|(label, _)| *label)
        .filter(|label| !known.contains(label))
        .collect();
    assert!(
        unknown.is_empty(),
        "RESERVED names kinds that do not exist: {unknown:?}"
    );
}
