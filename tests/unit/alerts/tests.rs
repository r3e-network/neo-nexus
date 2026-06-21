use crate::events::{EventKind, NewRuntimeEvent};

use super::*;

mod delivery_history;
mod payload_shapes;
mod policy_provider;
mod targets;

fn event(severity: EventSeverity) -> RuntimeEvent {
    RuntimeEvent {
        id: 7,
        occurred_at_unix: 1_800_000_000,
        node_id: Some("node-1".to_string()),
        node_name: Some("validator".to_string()),
        kind: EventKind::RpcHealthChecked,
        severity,
        message: "RPC health unreachable".to_string(),
    }
}
