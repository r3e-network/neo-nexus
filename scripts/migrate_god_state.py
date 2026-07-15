#!/usr/bin/env python3
"""Mechanical field migration for NeoNexusApp God State split.

Replaces flat field access with nested session/fleet/operations_ui paths.
Skips method-call sites (identifier immediately followed by '(').
"""

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# Longer / more specific field names first so prefixes do not double-replace.
SESSION_FIELDS = [
    "inspector_visible",
    "node_workspace_tab",
    "network_hub_section",
    "persisted_view",
    "selected_view",
    "notice",
    "toasts",
    "theme",
]

FLEET_FIELDS = [
    "overview_fleet_page",
    "node_status_filter",
    "pending_delete_node",
    "selected_node",
    "node_query",
    "node_page",
    "nodes",
    "draft",
]

# Map old flat name -> new nested path field name under operations_ui
OPS_MAP = [
    ("persisted_operations_section", "persisted_section"),
    ("operations_section", "section"),
    ("action_queue_severity_filter", "action_queue_severity_filter"),
    ("action_queue_resolution_filter", "action_queue_resolution_filter"),
    ("selected_readiness_action", "selected_readiness_action"),
    ("action_queue_query", "action_queue_query"),
    ("action_queue_page", "action_queue_page"),
    ("port_matrix_status_filter", "port_matrix_status_filter"),
    ("port_matrix_network_filter", "port_matrix_network_filter"),
    ("port_matrix_health_filter", "port_matrix_health_filter"),
    ("port_matrix_query", "port_matrix_query"),
    ("port_matrix_page", "port_matrix_page"),
    ("readiness_check_severity_filter", "readiness_check_severity_filter"),
    ("readiness_check_resolution_filter", "readiness_check_resolution_filter"),
    ("selected_readiness_check", "selected_readiness_check"),
    ("readiness_check_query", "readiness_check_query"),
    ("readiness_check_page", "readiness_check_page"),
    ("event_severity_filter", "event_severity_filter"),
    ("selected_event", "selected_event"),
    ("event_query", "event_query"),
    ("event_page", "event_page"),
]

RECEIVERS = ("self", "app")

# Skip these paths (definition sites rewritten by hand).
SKIP_PATHS = {
    "src/app/state.rs",
    "src/app/state/session.rs",
    "src/app/state/fleet.rs",
    "src/app/state/operations_ui.rs",
    "src/app/lifecycle/startup/initial_state.rs",
    "scripts/migrate_god_state.py",
}


def field_pattern(receiver: str, field: str) -> re.Pattern[str]:
    # Match receiver.field when not already nested and not a method call.
    return re.compile(
        rf"(?<![\w.]){receiver}\.{field}(?!\s*\()(?!\w)"
    )


def migrate_text(text: str) -> str:
    for field in SESSION_FIELDS:
        for receiver in RECEIVERS:
            text = field_pattern(receiver, field).sub(
                rf"{receiver}.session.{field}", text
            )
    for field in FLEET_FIELDS:
        for receiver in RECEIVERS:
            text = field_pattern(receiver, field).sub(
                rf"{receiver}.fleet.{field}", text
            )
    for old, new in OPS_MAP:
        for receiver in RECEIVERS:
            text = field_pattern(receiver, old).sub(
                rf"{receiver}.operations_ui.{new}", text
            )
    return text


def main() -> None:
    changed = 0
    for path in list((ROOT / "src").rglob("*.rs")) + list((ROOT / "tests").rglob("*.rs")):
        rel = path.relative_to(ROOT).as_posix()
        if rel in SKIP_PATHS:
            continue
        if "target/" in rel:
            continue
        original = path.read_text()
        updated = migrate_text(original)
        if updated != original:
            path.write_text(updated)
            changed += 1
            print(f"updated {rel}")
    print(f"files changed: {changed}")


if __name__ == "__main__":
    main()
