//! The surfaces under contract: primary views, sub-tabs, and the nested ones.

/// Every primary destination, with the inspector column both open and closed —
/// Every sub-tab of every view, as (label, view, persistence key, section key).
///
/// The view-level contracts iterate `VIEWS`, which lands each view on whichever
/// sub-tab happened to be persisted — so a surface reachable only by clicking a
/// segment was never measured, and one of them shipped with its primary action
/// laid out below the panel. A view is not one surface; it is as many surfaces as
/// it has segments, and each has to be contained on its own.
pub(super) type NestedSurface = (
    &'static str,
    &'static str,
    [(&'static str, &'static str); 2],
);

pub(super) const NESTED: [NestedSurface; 2] = [
    (
        "nodes/roles/presets",
        "nodes",
        [
            ("workspace.section.nodes", "roles"),
            ("workspace.section.roles", "presets"),
        ],
    ),
    (
        "nodes/roles/plan",
        "nodes",
        [
            ("workspace.section.nodes", "roles"),
            ("workspace.section.roles", "plan"),
        ],
    ),
];
