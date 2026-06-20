use crate::{
    alerts::{AlertDelivery, AlertDeliveryStatus},
    events::{EventKind, EventSeverity, RuntimeEvent},
    rpc_health::{RpcHealthRecord, RpcHealthStatus},
};

use super::parse_field;

pub(in crate::repository) fn event_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RuntimeEvent> {
    let kind_raw: String = row.get(4)?;
    let severity_raw: String = row.get(5)?;
    let occurred_at_unix: u64 = row.get(1)?;

    Ok(RuntimeEvent {
        id: row.get(0)?,
        occurred_at_unix,
        node_id: row.get(2)?,
        node_name: row.get(3)?,
        kind: parse_field::<EventKind>(&kind_raw)?,
        severity: parse_field::<EventSeverity>(&severity_raw)?,
        message: row.get(6)?,
    })
}

pub(in crate::repository) fn alert_delivery_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<AlertDelivery> {
    let status_raw: String = row.get(5)?;
    Ok(AlertDelivery {
        id: row.get(0)?,
        event_id: row.get(1)?,
        attempted_at_unix: row.get(2)?,
        route_label: row.get(3)?,
        target: row.get(4)?,
        status: parse_field::<AlertDeliveryStatus>(&status_raw)?,
        http_status: row.get(6)?,
        message: row.get(7)?,
    })
}

pub(in crate::repository) fn rpc_health_record_from_row(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<RpcHealthRecord> {
    let status_raw: String = row.get(5)?;
    let checked_at_unix: u64 = row.get(1)?;

    Ok(RpcHealthRecord {
        id: row.get(0)?,
        checked_at_unix,
        node_id: row.get(2)?,
        node_name: row.get(3)?,
        endpoint: row.get(4)?,
        status: parse_field::<RpcHealthStatus>(&status_raw)?,
        version: row.get(6)?,
        block_count: row.get(7)?,
        message: row.get(8)?,
    })
}
