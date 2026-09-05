//! Liveness endpoint for cloud probes and load balancers. Public by design:
//! it reports only process liveness and the guardian's own verdict, never
//! workspace data.

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};

use super::WebState;

pub async fn healthz(State(state): State<WebState>) -> Response {
    // A green process line meant nothing while the guardian thread could fail
    // to start or stall silently; the heartbeat turns that into a verdict.
    let supervision = state.supervision.evaluate();
    let status = match supervision.liveness {
        crate::supervision_heartbeat::SupervisionLiveness::Failed => "degraded",
        _ => "ok",
    };
    let mut body = serde_json::json!({
        "status": status,
        "application": "NeoNexus",
        "version": env!("CARGO_PKG_VERSION"),
        "supervision": supervision.liveness,
    });
    if let Some(detail) = supervision.detail {
        body["supervision_detail"] = serde_json::Value::String(detail);
    }
    Json(body).into_response()
}
